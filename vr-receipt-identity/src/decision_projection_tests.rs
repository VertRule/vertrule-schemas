use std::collections::BTreeMap;

use vertrule_schemas::{
    ActionNamespace, AdapterOriginId, AdapterReference, DecisionPayload, DigestBytes,
    EntityNamespace, GovernancePrincipalId, GovernanceScope, GovernedAction, GovernedSubject,
    IJsonUInt, SurfaceInstanceId, Verdict,
};

use super::project_decision_payload;
use crate::ReceiptIdentityError;

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
        operation_receipt_digest: None,
        sealed_policy_digest: None,
    })
}

#[test]
fn projection_is_identity_bearing_and_deterministic() -> Result<(), ReceiptIdentityError> {
    let decision = sample_decision(Verdict::Allow)?;
    let first = project_decision_payload(&decision)?;
    let second = project_decision_payload(&decision)?;
    assert_ne!(first.event_hash, DigestBytes::from_array([0; 32]));
    assert_eq!(first.event_hash, second.event_hash);
    assert_eq!(first.event_hash, crate::compute_event_hash(&first)?);
    Ok(())
}

#[test]
fn decision_mutation_changes_receipt_identity() -> Result<(), ReceiptIdentityError> {
    let allow = project_decision_payload(&sample_decision(Verdict::Allow)?)?;
    let deny = project_decision_payload(&sample_decision(Verdict::Deny)?)?;
    assert_ne!(allow.event_hash, deny.event_hash);
    Ok(())
}

#[test]
fn projection_preserves_parent_and_sealed_policy() -> Result<(), ReceiptIdentityError> {
    let mut decision = sample_decision(Verdict::Allow)?;
    let parent = DigestBytes::from_array([99; 32]);
    let policy = DigestBytes::from_array([77; 32]);
    decision.parent_id = Some(parent);
    decision.sealed_policy_digest = Some(policy);
    let envelope = project_decision_payload(&decision)?;
    assert_eq!(envelope.parent_id, Some(parent));
    assert_eq!(envelope.policy_digest, policy);
    Ok(())
}
