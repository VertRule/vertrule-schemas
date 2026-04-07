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
/// Computed as `BLAKE3(b"vr.surface.decision@0.1")`. In production
/// this will be derived from the frozen schema JSON; for alpha this
/// deterministic placeholder is sufficient.
fn schema_decision_digest() -> DigestBytes {
    DigestBytes::from_array(*blake3::hash(b"vr.surface.decision@0.1").as_bytes())
}

/// Compute `BLAKE3(JCS(scope))` as the context digest.
fn compute_scope_digest(
    scope: &GovernanceScope,
) -> Result<DigestBytes, crate::DefinitionError> {
    let scope_bytes = serde_json::to_vec(scope)
        .map_err(crate::jcs::JcsError::Json)?;
    let canon = crate::jcs::to_canon_bytes_from_slice(&scope_bytes)?;
    Ok(DigestBytes::from_array(*blake3::hash(&canon).as_bytes()))
}

/// Compute `BLAKE3(binding_id)` as a placeholder policy digest.
fn compute_policy_digest(binding_id: &str) -> DigestBytes {
    DigestBytes::from_array(*blake3::hash(binding_id.as_bytes()).as_bytes())
}

impl ProjectsToReceiptEnvelope for GovernedDecisionPayload {
    fn project(&self) -> Result<ReceiptEnvelope, crate::DefinitionError> {
        let context_digest = compute_scope_digest(&self.scope)?;
        let schema_digest = schema_decision_digest();
        let policy_digest = compute_policy_digest(&self.policy_binding_id);

        let payload_value = serde_json::to_value(self)
            .map_err(crate::jcs::JcsError::Json)?;
        let payload = CanonicalPayload::new(payload_value)?;

        let mut envelope = ReceiptEnvelope {
            envelope_version: SchemaVersion::V1,
            receipt_type: ReceiptType::Governance,
            context_digest,
            schema_digest,
            policy_digest,
            logical_time: self.logical_time,
            event_hash: DigestBytes::from_array([0u8; 32]),
            parent_id: self.parent_id,
            boundary_origin: Some(BoundaryOrigin::Governance),
            digest_algorithm: Some("blake3".to_string()),
            canonicalization: Some("jcs".to_string()),
            payload,
        };
        envelope.event_hash = compute_event_hash(&envelope)?;
        Ok(envelope)
    }
}

#[cfg(test)]
#[path = "decision_tests.rs"]
mod decision_tests;
