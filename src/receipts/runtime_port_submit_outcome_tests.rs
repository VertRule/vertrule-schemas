//! Tests for `RuntimePortSubmitOutcomePayload`.

use crate::receipts::DECISION_PAYLOAD_KIND;
use crate::{
    CanonicalPayload, DecisionReceiptPayload, DecisionVerdict, DigestBytes, RuntimePortCommandKind,
    RuntimePortSubmitOutcomePayload, TransitionCommitment,
};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

fn sample(
    transition: Option<TransitionCommitment>,
) -> Result<RuntimePortSubmitOutcomePayload, anyhow::Error> {
    Ok(RuntimePortSubmitOutcomePayload {
        command_id: digest(1),
        command_kind: RuntimePortCommandKind::Submit,
        command_schema_digest: digest(2),
        payload_digest: digest(3),
        payload_bytes_digest: digest(4),
        evidence_receipt_digests: vec![digest(5), digest(6)],
        submitted: CanonicalPayload::new(serde_json::json!({"op": "set", "k": 1}))?,
        decision: DecisionReceiptPayload {
            payload_kind: DECISION_PAYLOAD_KIND.to_string(),
            verdict: DecisionVerdict::Allow,
            support_set: vec![],
        },
        policy_config_digest: digest(7),
        transition,
    })
}

#[test]
fn round_trip_without_transition_omits_key() -> Result<(), anyhow::Error> {
    let payload = sample(None)?;
    let value = serde_json::to_value(&payload)?;
    let map = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("payload did not serialize to object"))?;
    assert!(!map.contains_key("transition"));
    assert_eq!(value["command_kind"], serde_json::json!("submit"));
    let parsed: RuntimePortSubmitOutcomePayload = serde_json::from_value(value)?;
    assert_eq!(parsed, payload);
    Ok(())
}

#[test]
fn round_trip_with_transition() -> Result<(), anyhow::Error> {
    let payload = sample(Some(TransitionCommitment {
        batch_digest: digest(8),
        post_state_digest: digest(9),
        engine_context_digest: digest(10),
        parent_digest: None,
    }))?;
    let value = serde_json::to_value(&payload)?;
    let transition = value["transition"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("transition did not serialize to object"))?;
    assert!(!transition.contains_key("parent_digest"));
    let parsed: RuntimePortSubmitOutcomePayload = serde_json::from_value(value)?;
    assert_eq!(parsed, payload);
    Ok(())
}

#[test]
fn unknown_field_is_rejected() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(sample(None)?)?;
    value["extra"] = serde_json::json!(1);
    let result: Result<RuntimePortSubmitOutcomePayload, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn unknown_transition_field_is_rejected() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(sample(Some(TransitionCommitment {
        batch_digest: digest(8),
        post_state_digest: digest(9),
        engine_context_digest: digest(10),
        parent_digest: Some(digest(11)),
    }))?)?;
    value["transition"]["extra"] = serde_json::json!(1);
    let result: Result<RuntimePortSubmitOutcomePayload, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn null_transition_is_rejected() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(sample(None)?)?;
    value["transition"] = serde_json::Value::Null;
    let from_value: Result<RuntimePortSubmitOutcomePayload, _> =
        serde_json::from_value(value.clone());
    assert!(from_value.is_err(), "null transition must not be absence");
    let from_str: Result<RuntimePortSubmitOutcomePayload, _> =
        serde_json::from_str(&value.to_string());
    assert!(from_str.is_err(), "null transition must not be absence");
    Ok(())
}

#[test]
fn null_transition_parent_digest_is_rejected() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(sample(Some(TransitionCommitment {
        batch_digest: digest(8),
        post_state_digest: digest(9),
        engine_context_digest: digest(10),
        parent_digest: None,
    }))?)?;
    value["transition"]["parent_digest"] = serde_json::Value::Null;
    let from_value: Result<RuntimePortSubmitOutcomePayload, _> =
        serde_json::from_value(value.clone());
    assert!(
        from_value.is_err(),
        "null parent_digest must not be absence"
    );
    let from_str: Result<RuntimePortSubmitOutcomePayload, _> =
        serde_json::from_str(&value.to_string());
    assert!(from_str.is_err(), "null parent_digest must not be absence");
    Ok(())
}

#[test]
fn only_submit_command_kind_deserializes() {
    let ok: Result<RuntimePortCommandKind, _> = serde_json::from_str("\"submit\"");
    assert_eq!(ok.ok(), Some(RuntimePortCommandKind::Submit));
    for bad in ["\"query\"", "\"Submit\"", "\"inspect\""] {
        let result: Result<RuntimePortCommandKind, _> = serde_json::from_str(bad);
        assert!(result.is_err(), "{bad} must be rejected");
    }
}
