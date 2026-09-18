//! Governance-decision projection under the V2 receipt constitution
//! (ADR-056, registry row P1 `vr.governance.decision`).
//!
//! The passive [`DecisionPayload`] shape lives in `vertrule-schemas`; this
//! module owns the behavior that turns it into an identity-bearing
//! [`ReceiptEnvelopeV2`] through [`seal_receipt_v2`], the only V2 mint path.
//!
//! **V1 mint denied (R12).** The superseded V1 constructor for this family
//! (`project_decision_payload`, `receipt_type: governance`,
//! `constitutional_envelope_v1` `event_hash`) was removed at C11 of the
//! ADR-056 migration. Its frozen BEFORE fixture is
//! `test-vectors/receipt_v1_governance_decision_before_001.json`; V1
//! *verification* of historical artifacts stays available through
//! [`compute_event_hash`](crate::compute_event_hash) (K7). A structural
//! guard test in this crate asserts no V1 governance-decision mint path
//! remains.

use vertrule_schemas::governance::ScopeDigest;
use vertrule_schemas::{
    CanonicalPayload, DecisionPayload, PayloadSchemaV2, ReceiptEnvelopeV2, ReceiptTypeV2,
};

use crate::{seal_receipt_v2, ReceiptIdentityError, ReceiptV2Draft};

/// Project a passive governance decision into a sealed
/// `vr.governance.decision` V2 receipt.
///
/// Facts placed in the draft (registry row P1):
///
/// - `schema_digest` = [`PayloadSchemaV2::VR_SURFACE_DECISION_0_1`] identity;
/// - `context_digest` = `ScopeDigest` of `decision.scope` (recomputed by
///   the verifier);
/// - `policy_digest` = `decision.sealed_policy_digest` — the policy that
///   actually decided, lifted from the payload;
/// - `logical_time` and `parent_id` echo the payload's own values;
/// - `payload` = the canonical `DecisionPayload`.
///
/// # Errors
///
/// Returns [`ReceiptIdentityError::MintDenied`] when
/// `decision.sealed_policy_digest` or `decision.operation_receipt_digest`
/// is `None`: a decision without sealed-run provenance has no policy value
/// to bind and no operation commitment to carry, and the row admits no
/// fallback (there is no `BLAKE3(binding_id)` arm and no re-typing).
/// Returns the other [`ReceiptIdentityError`] variants when the scope or
/// payload cannot be represented canonically, the row's payload-schema
/// identity cannot be formed ([`ReceiptIdentityError::Identity`]), or
/// sealing fails.
pub fn project_decision_payload_v2(
    decision: &DecisionPayload,
) -> Result<ReceiptEnvelopeV2, ReceiptIdentityError> {
    let receipt_type = ReceiptTypeV2::GovernanceDecision;
    let Some(sealed_policy_digest) = decision.sealed_policy_digest else {
        return Err(ReceiptIdentityError::MintDenied {
            receipt_type,
            reason: "sealed_policy_digest is absent: no sealed-run policy provenance to bind"
                .to_string(),
        });
    };
    if decision.operation_receipt_digest.is_none() {
        return Err(ReceiptIdentityError::MintDenied {
            receipt_type,
            reason: "operation_receipt_digest is absent: no sealed-run operation commitment"
                .to_string(),
        });
    }

    let context_digest = ScopeDigest::from_governance_scope(&decision.scope)?.as_digest_bytes()?;
    let payload_value = serde_json::to_value(decision).map_err(vr_jcs::JcsError::from)?;
    let payload = CanonicalPayload::new(payload_value)?;

    seal_receipt_v2(ReceiptV2Draft {
        receipt_type,
        schema_digest: PayloadSchemaV2::VR_SURFACE_DECISION_0_1.identity()?,
        context_digest: Some(context_digest),
        policy_digest: Some(sealed_policy_digest),
        logical_time: decision.logical_time.get(),
        parent_id: decision.parent_id,
        payload,
    })
}

#[cfg(test)]
#[path = "decision_projection_tests.rs"]
mod tests;
