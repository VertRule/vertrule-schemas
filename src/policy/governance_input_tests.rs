use super::*;
use crate::INPUT_FORMAT;

fn link(
    id: &str,
    status: GovernancePolicyStatusV1,
) -> Result<LinkedPolicyStateV1, GovernanceInputError> {
    LinkedPolicyStateV1::new(id.to_string(), status)
}

fn input_with_links(
    links: Vec<LinkedPolicyStateV1>,
) -> Result<GovernanceEvaluationInputV1, GovernanceInputError> {
    let subject = GovernanceSystemSubjectV1::new(
        "system-1".to_string(),
        GovernanceSystemStatusV1::Active,
        links,
    )?;
    Ok(GovernanceEvaluationInputV1::new(
        GovernanceOperationV1::SystemPromote,
        subject,
    ))
}

#[test]
fn canonical_bytes_are_stable_and_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let input = input_with_links(vec![
        link("policy-b", GovernancePolicyStatusV1::Draft)?,
        link("policy-a", GovernancePolicyStatusV1::Active)?,
    ])?;
    let bytes = input.to_canonical_bytes()?;
    assert_eq!(
        std::str::from_utf8(&bytes)?,
        "{\"input_format\":\"vr.policy.governance-input@0.1\",\
         \"operation\":\"system_promote\",\"subject\":{\
         \"linked_policies\":[{\"policy_id\":\"policy-a\",\"policy_status\":\"active\"},{\
         \"policy_id\":\"policy-b\",\"policy_status\":\"draft\"}],\
         \"system_id\":\"system-1\",\"system_status\":\"active\"}}"
    );
    assert_eq!(
        GovernanceEvaluationInputV1::from_canonical_bytes(&bytes)?,
        input
    );
    Ok(())
}

#[test]
fn reversed_policy_input_order_has_one_identity() -> Result<(), Box<dyn std::error::Error>> {
    let forward = input_with_links(vec![
        link("policy-a", GovernancePolicyStatusV1::Active)?,
        link("policy-b", GovernancePolicyStatusV1::Draft)?,
    ])?;
    let reversed = input_with_links(vec![
        link("policy-b", GovernancePolicyStatusV1::Draft)?,
        link("policy-a", GovernancePolicyStatusV1::Active)?,
    ])?;
    assert_eq!(forward, reversed);
    assert_eq!(
        forward.to_canonical_bytes()?,
        reversed.to_canonical_bytes()?
    );
    Ok(())
}

#[test]
fn duplicate_policy_identity_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let result = input_with_links(vec![
        link("policy-a", GovernancePolicyStatusV1::Draft)?,
        link("policy-a", GovernancePolicyStatusV1::Active)?,
    ]);
    assert_eq!(
        result,
        Err(GovernanceInputError::DuplicatePolicyId(
            "policy-a".to_string()
        ))
    );
    Ok(())
}

#[test]
fn deserialization_cannot_bypass_duplicate_rejection() {
    let json = b"{\"input_format\":\"vr.policy.governance-input@0.1\",\
        \"operation\":\"system_promote\",\"subject\":{\"linked_policies\":[\
        {\"policy_id\":\"policy-a\",\"policy_status\":\"draft\"},\
        {\"policy_id\":\"policy-a\",\"policy_status\":\"active\"}],\
        \"system_id\":\"system-1\",\"system_status\":\"active\"}}";
    assert!(GovernanceEvaluationInputV1::from_canonical_bytes(json).is_err());
}

#[test]
fn wrong_format_unknown_status_and_unknown_field_are_rejected() {
    for json in [
        "{\"input_format\":\"vr.policy.input@0.1\",\"operation\":\"system_promote\",\"subject\":{\"linked_policies\":[],\"system_id\":\"system-1\",\"system_status\":\"active\"}}",
        "{\"input_format\":\"vr.policy.governance-input@0.1\",\"operation\":\"system_promote\",\"subject\":{\"linked_policies\":[],\"system_id\":\"system-1\",\"system_status\":\"current\"}}",
        "{\"input_format\":\"vr.policy.governance-input@0.1\",\"operation\":\"system_promote\",\"subject\":{\"linked_policies\":[{\"policy_id\":\"policy-1\",\"policy_status\":\"current\"}],\"system_id\":\"system-1\",\"system_status\":\"active\"}}",
        "{\"input_format\":\"vr.policy.governance-input@0.1\",\"operation\":\"system_readiness_evaluate\",\"subject\":{\"linked_policies\":[],\"system_id\":\"system-1\",\"system_status\":\"active\"}}",
        "{\"extra\":true,\"input_format\":\"vr.policy.governance-input@0.1\",\"operation\":\"system_promote\",\"subject\":{\"linked_policies\":[],\"system_id\":\"system-1\",\"system_status\":\"active\"}}",
    ] {
        assert!(GovernanceEvaluationInputV1::from_canonical_bytes(json.as_bytes()).is_err());
    }
}

#[test]
fn workspace_status_vocabulary_is_exact() -> Result<(), serde_json::Error> {
    let system_statuses = [
        GovernanceSystemStatusV1::Active,
        GovernanceSystemStatusV1::UnderReview,
        GovernanceSystemStatusV1::Approved,
        GovernanceSystemStatusV1::Restricted,
        GovernanceSystemStatusV1::Retired,
    ];
    let policy_statuses = [
        GovernancePolicyStatusV1::Draft,
        GovernancePolicyStatusV1::Active,
        GovernancePolicyStatusV1::Retired,
    ];
    assert_eq!(
        serde_json::to_string(&system_statuses)?,
        "[\"active\",\"under_review\",\"approved\",\"restricted\",\"retired\"]"
    );
    assert_eq!(
        serde_json::to_string(&policy_statuses)?,
        "[\"draft\",\"active\",\"retired\"]"
    );
    Ok(())
}

#[test]
fn noncanonical_json_is_rejected() {
    let noncanonical = b"{ \"input_format\": \"vr.policy.governance-input@0.1\", \"operation\": \"system_promote\", \"subject\": { \"linked_policies\": [], \"system_id\": \"system-1\", \"system_status\": \"active\" } }";
    assert_eq!(
        GovernanceEvaluationInputV1::from_canonical_bytes(noncanonical),
        Err(GovernanceInputError::NonCanonicalEncoding)
    );
}

#[test]
fn duplicate_json_member_is_rejected() {
    let duplicate = b"{\"input_format\":\"vr.policy.governance-input@0.1\",\"input_format\":\"vr.policy.governance-input@0.1\",\"operation\":\"system_promote\",\"subject\":{\"linked_policies\":[],\"system_id\":\"system-1\",\"system_status\":\"active\"}}";
    assert!(GovernanceEvaluationInputV1::from_canonical_bytes(duplicate).is_err());
}

#[test]
fn kind_maps_to_exact_format_without_fallback() {
    assert_eq!(
        EvaluationInputKindV1::PolicyInputV0_1.format(),
        INPUT_FORMAT
    );
    assert_eq!(
        EvaluationInputKindV1::GovernanceInputV0_1.format(),
        GOVERNANCE_INPUT_FORMAT
    );
}
