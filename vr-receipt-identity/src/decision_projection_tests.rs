//! Tests for the V2 governance-decision projection (registry row P1).
//!
//! The sample decision is the exact payload of the C4/C7 golden
//! (`receipt_digest_v2_tests::golden_decision`), so the production
//! projection is pinned to the committed witness digest: the projection
//! and the witness are one receipt.

use std::collections::BTreeMap;

use vertrule_schemas::governance::ScopeDigest;
use vertrule_schemas::{
    ActionNamespace, AdapterOriginId, AdapterReference, DecisionPayload, DigestBytes,
    EntityNamespace, GovernancePrincipalId, GovernanceScope, GovernedAction, GovernedSubject,
    IJsonUInt, PayloadSchemaV2, ReceiptTypeV2, SchemaVersion, SurfaceInstanceId, Verdict,
};

use super::project_decision_payload_v2;
use crate::{compute_receipt_digest_v2, ReceiptIdentityError};

/// The C4/C7 golden `receipt_digest` (`RECEIPT_DIGEST_V2_LAW.md` §2).
const GOLDEN_RECEIPT_DIGEST: &str =
    "b2fcea60d2a648226fffd981aa65ff883205ed89993af84d977de2ce0a42ff58";

fn sealed_policy_digest() -> DigestBytes {
    DigestBytes::from_array([0x77; 32])
}

fn operation_receipt_digest() -> DigestBytes {
    DigestBytes::from_array([0x5e; 32])
}

/// The golden `DecisionPayload`, with the verdict parameterised.
fn sample_decision(verdict: Verdict) -> Result<DecisionPayload, ReceiptIdentityError> {
    Ok(DecisionPayload {
        scope: GovernanceScope {
            governance_principal_id: GovernancePrincipalId::new("org-1".to_string())?,
            surface_instance_id: SurfaceInstanceId::new("jira:inst-1".to_string())?,
            adapter_origin: AdapterOriginId::jira()?,
            workspace_scope: "jira:org-1:PROJ".to_string(),
        },
        subject: GovernedSubject {
            subject_key: "jira:issue:PROJ-42".to_string(),
            entity_namespace: EntityNamespace::new("issue".to_string())?,
            entity_id: "PROJ-42".to_string(),
        },
        action: GovernedAction {
            action_namespace: ActionNamespace::new("workflow".to_string())?,
            action_type: "transition".to_string(),
            action_idempotency_hint: None,
        },
        adapter_ref: AdapterReference {
            adapter_origin: AdapterOriginId::jira()?,
            external_keys: BTreeMap::from([("issue_key".to_string(), "PROJ-42".to_string())]),
        },
        verdict,
        reasons: vec![],
        policy_binding_id: "bind-1".to_string(),
        idempotency_key: DigestBytes::from_array([0; 32]),
        canonical_input_digest: DigestBytes::from_array([1; 32]),
        logical_time: IJsonUInt::new(1)?,
        parent_id: None,
        operation_receipt_digest: Some(operation_receipt_digest()),
        sealed_policy_digest: Some(sealed_policy_digest()),
    })
}

#[test]
fn production_projection_of_the_golden_decision_is_the_c4_witness(
) -> Result<(), ReceiptIdentityError> {
    let sealed = project_decision_payload_v2(&sample_decision(Verdict::Allow)?)?;
    assert_eq!(
        sealed.receipt_digest.to_hex(),
        GOLDEN_RECEIPT_DIGEST,
        "project_decision_payload_v2(golden) must be the C4/C7 golden receipt"
    );
    Ok(())
}

#[test]
fn projection_places_every_row_p1_fact() -> Result<(), ReceiptIdentityError> {
    let decision = sample_decision(Verdict::Allow)?;
    let sealed = project_decision_payload_v2(&decision)?;
    assert_eq!(sealed.envelope_version, SchemaVersion::V2);
    assert_eq!(sealed.receipt_type, ReceiptTypeV2::GovernanceDecision);
    assert_eq!(
        sealed.schema_digest,
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1.identity()
    );
    assert_eq!(
        sealed.context_digest,
        Some(ScopeDigest::from_governance_scope(&decision.scope)?.as_digest_bytes()?)
    );
    assert_eq!(sealed.policy_digest, Some(sealed_policy_digest()));
    assert_eq!(sealed.logical_time, 1);
    assert_eq!(sealed.parent_id, None);
    assert_eq!(
        sealed.payload.as_value(),
        &serde_json::to_value(&decision).map_err(vr_jcs::JcsError::from)?
    );
    Ok(())
}

#[test]
fn projection_is_identity_bearing_and_deterministic() -> Result<(), ReceiptIdentityError> {
    let decision = sample_decision(Verdict::Allow)?;
    let first = project_decision_payload_v2(&decision)?;
    let second = project_decision_payload_v2(&decision)?;
    assert_eq!(first, second);
    assert_eq!(first.receipt_digest, compute_receipt_digest_v2(&first)?);
    Ok(())
}

#[test]
fn decision_mutation_changes_receipt_identity() -> Result<(), ReceiptIdentityError> {
    let allow = project_decision_payload_v2(&sample_decision(Verdict::Allow)?)?;
    let deny = project_decision_payload_v2(&sample_decision(Verdict::Deny)?)?;
    assert_ne!(allow.receipt_digest, deny.receipt_digest);
    Ok(())
}

#[test]
fn projection_preserves_parent_and_sealed_policy() -> Result<(), ReceiptIdentityError> {
    let mut decision = sample_decision(Verdict::Allow)?;
    let parent = DigestBytes::from_array([99; 32]);
    let policy = DigestBytes::from_array([88; 32]);
    decision.parent_id = Some(parent);
    decision.sealed_policy_digest = Some(policy);
    decision.logical_time = IJsonUInt::new(7)?;
    let sealed = project_decision_payload_v2(&decision)?;
    assert_eq!(sealed.parent_id, Some(parent));
    assert_eq!(sealed.policy_digest, Some(policy));
    assert_eq!(sealed.logical_time, 7);
    Ok(())
}

// ── R12: no fallback, no re-typing ───────────────────────────────────

#[test]
fn a_decision_without_sealed_policy_provenance_is_mint_denied() -> Result<(), ReceiptIdentityError>
{
    let mut decision = sample_decision(Verdict::Allow)?;
    decision.sealed_policy_digest = None;
    let Err(error) = project_decision_payload_v2(&decision) else {
        return Err(ReceiptIdentityError::InvalidPayload(
            "expected MintDenied for a decision without sealed_policy_digest".to_string(),
        ));
    };
    assert!(
        matches!(
            &error,
            ReceiptIdentityError::MintDenied {
                receipt_type: ReceiptTypeV2::GovernanceDecision,
                reason
            } if reason.contains("sealed_policy_digest")
        ),
        "expected MintDenied naming sealed_policy_digest, got {error}"
    );
    Ok(())
}

#[test]
fn a_decision_without_an_operation_receipt_commitment_is_mint_denied(
) -> Result<(), ReceiptIdentityError> {
    let mut decision = sample_decision(Verdict::Allow)?;
    decision.operation_receipt_digest = None;
    let Err(error) = project_decision_payload_v2(&decision) else {
        return Err(ReceiptIdentityError::InvalidPayload(
            "expected MintDenied for a decision without operation_receipt_digest".to_string(),
        ));
    };
    assert!(
        matches!(
            &error,
            ReceiptIdentityError::MintDenied {
                receipt_type: ReceiptTypeV2::GovernanceDecision,
                reason
            } if reason.contains("operation_receipt_digest")
        ),
        "expected MintDenied naming operation_receipt_digest, got {error}"
    );
    Ok(())
}

#[test]
fn mint_denied_display_names_the_type_and_reason() {
    let error = ReceiptIdentityError::MintDenied {
        receipt_type: ReceiptTypeV2::GovernanceDecision,
        reason: "why".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "mint denied for vr.governance.decision: why"
    );
}
