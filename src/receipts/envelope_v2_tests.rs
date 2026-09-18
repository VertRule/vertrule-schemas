//! Tests for `ReceiptEnvelopeV2`.

use crate::{DigestBytes, ReceiptEnvelopeV2, ReceiptTypeV2, SchemaVersion};

fn hex(fill: u8) -> String {
    DigestBytes::from_array([fill; 32]).to_hex()
}

fn minimal_document() -> serde_json::Value {
    serde_json::json!({
        "envelope_version": 2,
        "receipt_type": "vr.governance.decision",
        "schema_digest": hex(2),
        "logical_time": "7",
        "payload": {"hello": "world"},
        "receipt_digest": hex(9),
    })
}

fn object(
    value: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, anyhow::Error> {
    value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("envelope did not serialize to object"))
}

#[test]
fn round_trip_with_absent_slots_omits_keys() -> Result<(), anyhow::Error> {
    let envelope: ReceiptEnvelopeV2 = serde_json::from_value(minimal_document())?;
    assert_eq!(envelope.envelope_version, SchemaVersion::V2);
    assert_eq!(envelope.receipt_type, ReceiptTypeV2::GovernanceDecision);
    assert_eq!(envelope.context_digest, None);
    assert_eq!(envelope.policy_digest, None);
    assert_eq!(envelope.parent_id, None);
    assert_eq!(envelope.logical_time, 7);

    let value = serde_json::to_value(&envelope)?;
    let map = object(&value)?;
    assert!(!map.contains_key("context_digest"));
    assert!(!map.contains_key("policy_digest"));
    assert!(!map.contains_key("parent_id"));
    assert_eq!(map.len(), 6);
    assert_eq!(value["logical_time"], serde_json::json!("7"));

    let parsed: ReceiptEnvelopeV2 = serde_json::from_value(value)?;
    assert_eq!(parsed, envelope);
    Ok(())
}

#[test]
fn round_trip_with_present_slots_keeps_keys() -> Result<(), anyhow::Error> {
    let mut document = minimal_document();
    document["context_digest"] = serde_json::json!(hex(1));
    document["policy_digest"] = serde_json::json!(hex(3));
    document["parent_id"] = serde_json::json!(hex(5));
    let envelope: ReceiptEnvelopeV2 = serde_json::from_value(document)?;
    assert_eq!(
        envelope.context_digest,
        Some(DigestBytes::from_array([1; 32]))
    );
    assert_eq!(
        envelope.policy_digest,
        Some(DigestBytes::from_array([3; 32]))
    );
    assert_eq!(envelope.parent_id, Some(DigestBytes::from_array([5; 32])));

    let value = serde_json::to_value(&envelope)?;
    let map = object(&value)?;
    assert_eq!(map.len(), 9);
    let parsed: ReceiptEnvelopeV2 = serde_json::from_str(&serde_json::to_string(&envelope)?)?;
    assert_eq!(parsed, envelope);
    Ok(())
}

#[test]
fn logical_time_accepts_bare_number_on_input() -> Result<(), anyhow::Error> {
    let mut document = minimal_document();
    document["logical_time"] = serde_json::json!(7);
    let envelope: ReceiptEnvelopeV2 = serde_json::from_value(document)?;
    assert_eq!(envelope.logical_time, 7);
    let value = serde_json::to_value(&envelope)?;
    assert_eq!(value["logical_time"], serde_json::json!("7"));
    Ok(())
}

#[test]
fn logical_time_accepts_only_the_canonical_string_form() -> Result<(), anyhow::Error> {
    for (text, expected) in [("0", 0_u64), ("1", 1), ("18446744073709551615", u64::MAX)] {
        let mut document = minimal_document();
        document["logical_time"] = serde_json::json!(text);
        let envelope: ReceiptEnvelopeV2 = serde_json::from_value(document)?;
        assert_eq!(envelope.logical_time, expected, "{text:?}");
    }
    Ok(())
}

#[test]
fn logical_time_rejects_non_canonical_decimal_strings() {
    for text in [
        "01",
        "+1",
        "00",
        "+0",
        " 1",
        "1 ",
        "-1",
        "1.5",
        "",
        "18446744073709551616",
    ] {
        let mut document = minimal_document();
        document["logical_time"] = serde_json::json!(text);
        let from_value: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document.clone());
        assert!(
            from_value.is_err(),
            "non-canonical logical_time {text:?} must be rejected (from_value)"
        );
        let from_str: Result<ReceiptEnvelopeV2, _> = serde_json::from_str(&document.to_string());
        assert!(
            from_str.is_err(),
            "non-canonical logical_time {text:?} must be rejected (from_str)"
        );
    }
}

#[test]
fn null_slot_is_rejected() {
    for slot in ["context_digest", "policy_digest", "parent_id"] {
        let mut document = minimal_document();
        document[slot] = serde_json::Value::Null;
        let from_value: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document.clone());
        assert!(
            from_value.is_err(),
            "null {slot} must be a type error, never absence (from_value)"
        );
        let from_str: Result<ReceiptEnvelopeV2, _> = serde_json::from_str(&document.to_string());
        assert!(
            from_str.is_err(),
            "null {slot} must be a type error, never absence (from_str)"
        );
    }
}

#[test]
fn null_slot_never_reaches_the_absent_form() -> Result<(), anyhow::Error> {
    // The omitted form parses; the null form does not; so the two wire
    // documents can never share a `ReceiptEnvelopeV2` value (ADR-056 R8).
    let absent: ReceiptEnvelopeV2 = serde_json::from_value(minimal_document())?;
    assert_eq!(absent.parent_id, None);
    let mut document = minimal_document();
    document["parent_id"] = serde_json::Value::Null;
    let null_form: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document);
    assert!(null_form.is_err());
    Ok(())
}

#[test]
fn envelope_version_one_parses_in_v2_shape() -> Result<(), anyhow::Error> {
    // Recorded behaviour, not a law: the shape is passive data and
    // `SchemaVersion::new` accepts 1. Requiring version 2 is the verifier's
    // `UnsupportedVersion` DENY (ADR-056 §3.5, K3).
    let mut document = minimal_document();
    document["envelope_version"] = serde_json::json!(1);
    let envelope: ReceiptEnvelopeV2 = serde_json::from_value(document)?;
    assert_eq!(envelope.envelope_version, SchemaVersion::V1);
    Ok(())
}

#[test]
fn v1_document_is_rejected() {
    let v1 = serde_json::json!({
        "envelope_version": 1,
        "receipt_type": "governance",
        "context_digest": hex(1),
        "schema_digest": hex(2),
        "policy_digest": hex(3),
        "logical_time": "7",
        "event_hash": hex(4),
        "payload": {"hello": "world"},
    });
    let result: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(v1);
    assert!(result.is_err(), "a V1 document must not parse as V2");
}

#[test]
fn v2_document_is_rejected_by_v1_envelope() {
    let result: Result<crate::ReceiptEnvelope, _> = serde_json::from_value(minimal_document());
    assert!(result.is_err(), "a V2 document must not parse as V1");
}

#[test]
fn missing_receipt_digest_is_rejected() {
    let mut document = minimal_document();
    if let Some(map) = document.as_object_mut() {
        map.remove("receipt_digest");
    }
    let result: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document);
    assert!(result.is_err());
}

#[test]
fn missing_schema_digest_is_rejected() {
    let mut document = minimal_document();
    if let Some(map) = document.as_object_mut() {
        map.remove("schema_digest");
    }
    let result: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document);
    assert!(result.is_err());
}

#[test]
fn unknown_field_is_rejected() {
    for key in [
        "event_hash",
        "event_hash_profile",
        "boundary_origin",
        "digest_algorithm",
        "canonicalization",
        "extra",
    ] {
        let mut document = minimal_document();
        document[key] = serde_json::json!("x");
        let result: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document);
        assert!(result.is_err(), "unknown field {key:?} must be rejected");
    }
}

#[test]
fn unsupported_version_is_rejected() {
    let mut document = minimal_document();
    document["envelope_version"] = serde_json::json!(3);
    let result: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document);
    assert!(result.is_err());
}

#[test]
fn float_payload_is_rejected() {
    let mut document = minimal_document();
    document["payload"] = serde_json::json!({"x": 1.5});
    let result: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document);
    assert!(result.is_err());
}
