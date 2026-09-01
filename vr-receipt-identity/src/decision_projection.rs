//! Governance-decision projection under the constitutional receipt law.
//!
//! The passive [`DecisionPayload`] shape lives in `vertrule-schemas`; this
//! module owns the behavior that turns it into an identity-bearing envelope.

use vertrule_schemas::governance::{PolicyDigest, SchemaDigest, ScopeDigest};
use vertrule_schemas::{
    BoundaryOrigin, CanonicalPayload, DecisionPayload, DigestBytes, ReceiptEnvelope, ReceiptType,
    SchemaVersion,
};

use crate::{compute_event_hash, ReceiptIdentityError};

/// Project a passive governance decision into a canonical receipt envelope.
///
/// The resulting envelope uses the frozen `vr.surface.decision@0.1` schema
/// binding and the `constitutional_envelope_v1` event-hash law.
///
/// # Errors
///
/// Returns a typed error when the scope or payload cannot be represented
/// canonically, or when receipt commitment construction fails.
pub fn project_decision_payload(
    decision: &DecisionPayload,
) -> Result<ReceiptEnvelope, ReceiptIdentityError> {
    let context_digest = ScopeDigest::from_governance_scope(&decision.scope)?.as_digest_bytes()?;
    let schema_digest = SchemaDigest::for_decision_v0_1().as_digest_bytes();
    let policy_digest = decision.sealed_policy_digest.unwrap_or_else(|| {
        PolicyDigest::from_binding_id(&decision.policy_binding_id).as_digest_bytes()
    });

    let payload_value = serde_json::to_value(decision).map_err(vr_jcs::JcsError::from)?;
    let payload = CanonicalPayload::new(payload_value)?;

    let mut envelope: ReceiptEnvelope = serde_json::from_value(serde_json::json!({
        "envelope_version": SchemaVersion::V1,
        "receipt_type": ReceiptType::Governance,
        "context_digest": context_digest,
        "schema_digest": schema_digest,
        "policy_digest": policy_digest,
        "logical_time": decision.logical_time.get().to_string(),
        "event_hash": DigestBytes::from_array([0u8; 32]),
        "parent_id": decision.parent_id,
        "boundary_origin": BoundaryOrigin::Governance,
        "digest_algorithm": SchemaVersion::V1.digest_algorithm(),
        "canonicalization": SchemaVersion::V1.canonicalization(),
        "payload": payload,
    }))
    .map_err(vr_jcs::JcsError::from)?;
    envelope.event_hash = compute_event_hash(&envelope)?;
    Ok(envelope)
}

#[cfg(test)]
#[path = "decision_projection_tests.rs"]
mod tests;
