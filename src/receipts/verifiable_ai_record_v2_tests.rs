//! Tests for the `@0.2` provider-interaction and record payload shapes.

use std::collections::BTreeMap;

use crate::receipts::verifiable_ai_record_v2::VERIFIABLE_AI_RECORD_FORMAT_V3;
use crate::{
    DigestBytes, PayloadSchemaV2, ProviderInteractionPayloadV2, ReceiptEnvelopeV2,
    VerifiableAiRecordArtifactV3, VerifiableAiRecordPayloadV2, VERIFIABLE_AI_RECORD_POLICY,
};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

/// A structurally valid V2 envelope document of `receipt_type` whose
/// `receipt_digest` is `fill`-filled (no digest law is judged here).
fn envelope(receipt_type: &str, fill: u8) -> Result<ReceiptEnvelopeV2, anyhow::Error> {
    Ok(serde_json::from_value(serde_json::json!({
        "envelope_version": 2,
        "receipt_type": receipt_type,
        "schema_digest": digest(0x5c).to_hex(),
        "logical_time": "1",
        "payload": {"schema": "x"},
        "receipt_digest": digest(fill).to_hex(),
    }))?)
}

fn artifact() -> Result<VerifiableAiRecordArtifactV3, anyhow::Error> {
    Ok(VerifiableAiRecordArtifactV3 {
        format: VERIFIABLE_AI_RECORD_FORMAT_V3.to_string(),
        record: envelope("vr.record.verifiable_ai_record", 0xf0)?,
        evidence: BTreeMap::from([
            (
                digest(0xb0),
                envelope("vr.workflow.proposal_admission", 0xb0)?,
            ),
            (digest(0x0a), envelope("vr.ai.provider_interaction", 0x0a)?),
            (digest(0x7c), envelope("vr.workflow.agent_proposal", 0x7c)?),
        ]),
    })
}

fn interaction(resolved_model: Option<String>) -> ProviderInteractionPayloadV2 {
    ProviderInteractionPayloadV2 {
        schema: PayloadSchemaV2::VR_AI_PROVIDER_INTERACTION_0_2
            .label()
            .to_string(),
        provider: "openai".to_string(),
        requested_model: "gpt-x".to_string(),
        resolved_model,
        provider_response_id: None,
        capture_policy_version: "openai-responses-text/v1".to_string(),
        prompt: "What colour is the sky?".to_string(),
        response: "Blue.".to_string(),
        provider_attestation: "not_provided".to_string(),
    }
}

fn record() -> VerifiableAiRecordPayloadV2 {
    VerifiableAiRecordPayloadV2 {
        schema: PayloadSchemaV2::VR_RECORD_VERIFIABLE_AI_RECORD_0_2
            .label()
            .to_string(),
        record_policy: VERIFIABLE_AI_RECORD_POLICY.to_string(),
        source_interaction_digest: digest(1),
        extraction_interaction_digest: digest(2),
        proposal_receipt_digest: digest(3),
        admission_receipt_digest: digest(4),
        admitted_proposal_digest: digest(5),
    }
}

#[test]
fn interaction_round_trips_without_leaf_digests_or_payload_kind() -> Result<(), anyhow::Error> {
    let payload = interaction(None);
    let value = serde_json::to_value(&payload)?;
    let map = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("payload did not serialize to object"))?;
    for absent in [
        "payload_kind",
        "request",
        "response_digest",
        "prompt_digest",
        "resolved_model",
        "provider_response_id",
    ] {
        assert!(!map.contains_key(absent), "{absent} must be absent");
    }
    assert_eq!(value["schema"], "vr.ai.provider_interaction@0.2");
    assert_eq!(value["prompt"], "What colour is the sky?");
    assert_eq!(value["response"], "Blue.");
    let parsed: ProviderInteractionPayloadV2 = serde_json::from_value(value)?;
    assert_eq!(parsed, payload);
    Ok(())
}

#[test]
fn interaction_present_optional_facts_round_trip() -> Result<(), anyhow::Error> {
    let payload = ProviderInteractionPayloadV2 {
        provider_response_id: Some("resp_1".to_string()),
        ..interaction(Some("gpt-x-2026".to_string()))
    };
    let value = serde_json::to_value(&payload)?;
    assert_eq!(value["resolved_model"], "gpt-x-2026");
    assert_eq!(value["provider_response_id"], "resp_1");
    let parsed: ProviderInteractionPayloadV2 = serde_json::from_value(value)?;
    assert_eq!(parsed, payload);
    Ok(())
}

#[test]
fn interaction_rejects_null_optional_facts() -> Result<(), anyhow::Error> {
    for slot in ["resolved_model", "provider_response_id"] {
        let mut value = serde_json::to_value(interaction(None))?;
        value[slot] = serde_json::Value::Null;
        let from_value: Result<ProviderInteractionPayloadV2, _> =
            serde_json::from_value(value.clone());
        assert!(from_value.is_err(), "null {slot} must be rejected");
        let from_str: Result<ProviderInteractionPayloadV2, _> =
            serde_json::from_str(&value.to_string());
        assert!(from_str.is_err(), "null {slot} must be rejected from text");
    }
    Ok(())
}

#[test]
fn interaction_rejects_v1_leaf_digest_shape() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(interaction(None))?;
    value["request"] = serde_json::json!({
        "prompt": "What colour is the sky?",
        "prompt_digest": digest(9).to_hex(),
    });
    let result: Result<ProviderInteractionPayloadV2, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn record_round_trips_without_payload_kind() -> Result<(), anyhow::Error> {
    let payload = record();
    let value = serde_json::to_value(&payload)?;
    let map = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("payload did not serialize to object"))?;
    assert!(!map.contains_key("payload_kind"));
    assert_eq!(value["schema"], "vr.record.verifiable_ai_record@0.2");
    assert_eq!(value["record_policy"], "vr.record.integrity_not_truth@0.1");
    let parsed: VerifiableAiRecordPayloadV2 = serde_json::from_value(value)?;
    assert_eq!(parsed, payload);
    Ok(())
}

#[test]
fn record_rejects_unknown_field() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(record())?;
    value["payload_kind"] = serde_json::json!("record.verifiable_ai_record");
    let result: Result<VerifiableAiRecordPayloadV2, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn artifact_format_label_is_v3() {
    assert_eq!(VERIFIABLE_AI_RECORD_FORMAT_V3, "vr-verifiable-ai-record/v3");
}

#[test]
fn artifact_v3_round_trips_with_hex_digest_keys_in_digest_order() -> Result<(), anyhow::Error> {
    let artifact = artifact()?;
    let text = serde_json::to_string(&artifact)?;
    let value: serde_json::Value = serde_json::from_str(&text)?;
    let map = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("artifact did not serialize to object"))?;
    assert_eq!(map["_format"], "vr-verifiable-ai-record/v3");
    assert!(!map.contains_key("format"));
    assert_eq!(
        value["record"]["receipt_type"],
        "vr.record.verifiable_ai_record"
    );
    let evidence = value["evidence"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("evidence did not serialize to object"))?;
    let keys: Vec<&String> = evidence.keys().collect();
    assert_eq!(
        keys,
        [
            &digest(0x0a).to_hex(),
            &digest(0x7c).to_hex(),
            &digest(0xb0).to_hex()
        ],
        "evidence keys are 64-hex digests in digest order"
    );
    for (key, entry) in evidence {
        assert_eq!(key.len(), 64);
        assert_eq!(entry["receipt_digest"], serde_json::json!(key));
    }
    let parsed: VerifiableAiRecordArtifactV3 = serde_json::from_str(&text)?;
    assert_eq!(parsed, artifact);
    Ok(())
}

#[test]
fn artifact_v3_rejects_the_v2_container_shape() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(artifact()?)?;
    value["source_interaction"] = serde_json::json!({});
    let result: Result<VerifiableAiRecordArtifactV3, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn artifact_v3_rejects_null_or_missing_evidence() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(artifact()?)?;
    value["evidence"] = serde_json::Value::Null;
    let null: Result<VerifiableAiRecordArtifactV3, _> = serde_json::from_value(value.clone());
    assert!(null.is_err());
    value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("artifact did not serialize to object"))?
        .remove("evidence");
    let missing: Result<VerifiableAiRecordArtifactV3, _> = serde_json::from_value(value);
    assert!(missing.is_err());
    Ok(())
}

#[test]
fn artifact_v3_rejects_a_non_digest_evidence_key() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(artifact()?)?;
    let entry = value["evidence"][digest(0x0a).to_hex()].clone();
    value["evidence"][digest(0x0a).to_hex().to_uppercase()] = entry;
    let result: Result<VerifiableAiRecordArtifactV3, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}
