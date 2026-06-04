//! Governed decision payload — the receipt-facing representation.
//!
//! [`GovernedDecisionPayload`] is the `CanonicalPayload` content for
//! `vr.surface.decision@0.1` receipts. Pure data. Minting, projection,
//! and persistence logic live outside `vertrule-schemas`.

use serde::{Deserialize, Serialize};

use super::{AdapterReference, GovernanceScope, GovernedAction, GovernedSubject};
use crate::receipts::compute_event_hash;
use crate::{
    BoundaryOrigin, CanonicalPayload, DigestBytes, IJsonUInt, ProjectsToReceiptEnvelope,
    ReceiptEnvelope, ReceiptType, SchemaVersion,
};

/// Governed decision payload.
///
/// Contains everything needed to reconstruct and verify a governance
/// decision: scope, subject, action, verdict, policy reference, and
/// the canonical input digest that was evaluated.
///
/// Implements [`ProjectsToReceiptEnvelope`] to mint a canonical
/// [`ReceiptEnvelope`] directly, using `compute_event_hash()` from
/// the commitment module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedDecisionPayload {
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

impl ProjectsToReceiptEnvelope for GovernedDecisionPayload {
    fn project(&self) -> Result<ReceiptEnvelope, crate::DefinitionError> {
        let context_digest = compute_scope_digest(&self.scope)?;
        let schema_digest = schema_decision_digest();
        let policy_digest = compute_policy_digest(&self.policy_binding_id);

        let payload_value = serde_json::to_value(self).map_err(crate::jcs::JcsError::Json)?;
        let payload = CanonicalPayload::new(payload_value)?;

        let mut envelope = ReceiptEnvelope {
            envelope_version: SchemaVersion::V1,
            receipt_type: ReceiptType::Governance,
            context_digest,
            schema_digest,
            policy_digest,
            logical_time: self.logical_time,
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
