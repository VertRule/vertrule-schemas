//! Governed decision payload — the receipt-facing representation.
//!
//! [`DecisionPayload`] is the `CanonicalPayload` content for
//! `vr.surface.decision@0.1` receipts. Pure data. Minting, projection,
//! and persistence logic live outside `vertrule-schemas`.

use serde::{Deserialize, Serialize};

use super::{AdapterReference, GovernanceScope, GovernedAction, GovernedSubject};
use crate::receipts::compute_event_hash;
use crate::{
    BoundaryOrigin, CanonicalPayload, DigestBytes, IJsonUInt, ProjectsToReceiptEnvelope,
    ReceiptEnvelope, ReceiptType, SchemaVersion,
};

/// What a decision *says* — scope, subject, action, verdict, policy
/// reference, and the canonical input digest that was evaluated.
///
/// # This type carries no authority
///
/// ```text
/// Shape  ≠  Authority
/// ```
///
/// It is passive, public wire data: freely serializable **and freely
/// deserializable**, which means it can be constructed from any caller-supplied
/// bytes. Possession of one proves nothing about how it was produced.
///
/// It was previously named `GovernedDecisionPayload`, which asserted the
/// opposite — a name claiming governance on an object with no governance
/// construction law behind it (gremlin#207). Renamed rather than sealed:
/// private fields could not have made the claim true, because `Deserialize`
/// constructs the value regardless of field visibility. Sealing it would have
/// produced constructor privacy while still permitting arbitrary construction
/// through the wire interface — a type that looks sealed and is not.
///
/// # What makes a decision governed
///
/// ```text
/// GovernedDecision  ⇒  SanctionedEvaluationResult ∧ SealedRun ∧ OperationReceipt
/// ```
///
/// The evidence is the receipt minted by `SealedRun`, not this struct. The
/// same split `OperationReceipt` already uses: public payload shape,
/// runtime-only sanctioned minting.
///
/// Implements [`ProjectsToReceiptEnvelope`] to mint a canonical
/// [`ReceiptEnvelope`] directly, using `compute_event_hash()` from
/// the commitment module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionPayload {
    /// Governance scope that was evaluated.
    pub scope: GovernanceScope,
    /// Subject of the decision.
    pub subject: GovernedSubject,
    /// Action that was evaluated.
    pub action: GovernedAction,
    /// Adapter-local references for round-tripping.
    pub adapter_ref: AdapterReference,
    /// Policy outcome.
    pub verdict: Verdict,
    /// Human-readable reasons for the verdict.
    pub reasons: Vec<String>,
    /// Which policy binding produced this decision.
    pub policy_binding_id: String,
    /// Deterministic idempotency key (computed outside this crate).
    pub idempotency_key: DigestBytes,
    /// Digest of the canonical input that was evaluated.
    pub canonical_input_digest: DigestBytes,
    /// Monotonic logical clock value for this receipt.
    pub logical_time: IJsonUInt,
    /// Previous receipt `event_hash`, when this decision is chained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<DigestBytes>,

    /// Commitment to the `SealedRun`-minted `OperationReceipt` that produced
    /// this decision.
    ///
    /// This is what makes the persisted artifact able to prove its own
    /// provenance:
    ///
    /// ```text
    /// PersistedGovernedDecision ⇒ PublicCommitmentTo(OperationReceipt)
    /// ```
    ///
    /// It rides in the payload, so `event_hash` commits to it — a verifier that
    /// recomputes the envelope hash necessarily commits to the sealed-run
    /// receipt too.
    ///
    /// `None` is meaningful, not merely absent: it says no sealed run backs this
    /// decision. Optional so that pre-existing artifacts canonicalize to
    /// identical bytes and deployed verifiers keep parsing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_receipt_digest: Option<DigestBytes>,

    /// Digest of the policy that actually decided, as re-asserted by the
    /// sealed run.
    ///
    /// When present this replaces the `policy_binding_id` placeholder in the
    /// projected envelope's `policy_digest`. The binding id is a *label* chosen
    /// by whoever wrote the binding; this is the policy the runtime was sealed
    /// against. Presenting the former as policy provenance while the latter is
    /// the real authority is over-labelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sealed_policy_digest: Option<DigestBytes>,
}

/// Policy outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Verdict {
    /// Action is allowed.
    Allow,
    /// Action is denied.
    Deny,
    /// Action requires additional conditions.
    Conditional {
        /// What is still needed.
        requirements: Vec<String>,
    },
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => f.write_str("allow"),
            Self::Deny => f.write_str("deny"),
            Self::Conditional { .. } => f.write_str("conditional"),
        }
    }
}

// ── ProjectsToReceiptEnvelope ──────────────────────────────────────

/// Fixed schema digest for `vr.surface.decision@0.1`.
///
/// As of Gate 2 (JCS Consumer Hardening Plan), delegates to the sealed
/// [`SchemaDigest::for_decision_v0_1`] constructor. Byte-stable with
/// the prior `BLAKE3(b"vr.surface.decision@0.1")` implementation.
fn schema_decision_digest() -> DigestBytes {
    super::identity::SchemaDigest::for_decision_v0_1().as_digest_bytes()
}

/// Compute `BLAKE3(JCS(scope))` as the context digest.
///
/// As of Gate 2, delegates to [`ScopeDigest::from_governance_scope`].
fn compute_scope_digest(scope: &GovernanceScope) -> Result<DigestBytes, crate::DefinitionError> {
    super::identity::ScopeDigest::from_governance_scope(scope)?.as_digest_bytes()
}

/// Compute `BLAKE3(binding_id)` as a placeholder policy digest.
///
/// As of Gate 2, delegates to [`PolicyDigest::from_binding_id`]. Raw
/// label identity — not JCS.
fn compute_policy_digest(binding_id: &str) -> DigestBytes {
    super::identity::PolicyDigest::from_binding_id(binding_id).as_digest_bytes()
}

impl ProjectsToReceiptEnvelope for DecisionPayload {
    fn project(&self) -> Result<ReceiptEnvelope, crate::DefinitionError> {
        let context_digest = compute_scope_digest(&self.scope)?;
        let schema_digest = schema_decision_digest();
        // Prefer the policy the sealed run was bound to; fall back to the
        // binding-label placeholder only when no sealed run produced this.
        let policy_digest = self
            .sealed_policy_digest
            .unwrap_or_else(|| compute_policy_digest(&self.policy_binding_id));

        let payload_value = serde_json::to_value(self).map_err(crate::jcs::JcsError::Json)?;
        let payload = CanonicalPayload::new(payload_value)?;

        let mut envelope = ReceiptEnvelope {
            envelope_version: SchemaVersion::V1,
            receipt_type: ReceiptType::Governance,
            context_digest,
            schema_digest,
            policy_digest,
            logical_time: self.logical_time.into(),
            event_hash: DigestBytes::from_array([0u8; 32]),
            event_hash_profile: None,
            parent_id: self.parent_id,
            boundary_origin: Some(BoundaryOrigin::Governance),
            digest_algorithm: Some(SchemaVersion::V1.digest_algorithm().to_string()),
            canonicalization: Some(SchemaVersion::V1.canonicalization().to_string()),
            payload,
        };
        envelope.event_hash = compute_event_hash(&envelope)?;
        Ok(envelope)
    }
}

#[cfg(test)]
#[path = "decision_tests.rs"]
mod decision_tests;
