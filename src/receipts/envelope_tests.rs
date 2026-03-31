//! Tests for `ReceiptEnvelope`.

use crate::{
    CanonicalPayload, DigestBytes, IJsonUInt, ReceiptEnvelope, ReceiptType, SchemaVersion,
};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

fn payload(value: serde_json::Value) -> Result<CanonicalPayload, anyhow::Error> {
    Ok(CanonicalPayload::new(value)?)
}

#[test]
fn serde_round_trip_minimal_envelope() -> Result<(), anyhow::Error> {
    let envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Governance,
        context_digest: digest(1),
        schema_digest: digest(2),
        policy_digest: digest(3),
        logical_time: IJsonUInt::new(7)?,
        event_hash: digest(4),
        parent_id: None,
        boundary_origin: None,
        digest_algorithm: None,
        canonicalization: None,
        payload: payload(serde_json::json!({"hello": "world"}))?,
    };

    let json = serde_json::to_string(&envelope)?;
    let parsed: ReceiptEnvelope = serde_json::from_str(&json)?;
    assert_eq!(parsed, envelope);
    Ok(())
}

#[test]
fn optional_fields_serialize_only_when_present() -> Result<(), anyhow::Error> {
    let envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Event,
        context_digest: digest(1),
        schema_digest: digest(2),
        policy_digest: digest(3),
        logical_time: IJsonUInt::new(1)?,
        event_hash: digest(4),
        parent_id: None,
        boundary_origin: None,
        digest_algorithm: None,
        canonicalization: None,
        payload: payload(serde_json::json!({"k": "v"}))?,
    };

    let value = serde_json::to_value(&envelope)?;
    let object = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("envelope did not serialize to object"))?;
    assert!(!object.contains_key("parent_id"));
    assert!(!object.contains_key("boundary_origin"));
    assert!(!object.contains_key("digest_algorithm"));
    assert!(!object.contains_key("canonicalization"));
    Ok(())
}

#[test]
fn algorithm_markers_round_trip_when_present() -> Result<(), anyhow::Error> {
    let envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Training,
        context_digest: digest(1),
        schema_digest: digest(2),
        policy_digest: digest(3),
        logical_time: IJsonUInt::new(9)?,
        event_hash: digest(4),
        parent_id: Some(digest(5)),
        boundary_origin: Some(crate::BoundaryOrigin::Training),
        digest_algorithm: Some(SchemaVersion::V1.digest_algorithm().to_string()),
        canonicalization: Some(SchemaVersion::V1.canonicalization().to_string()),
        payload: payload(serde_json::json!({"loss": 0}))?,
    };

    let json = serde_json::to_string(&envelope)?;
    let parsed: ReceiptEnvelope = serde_json::from_str(&json)?;
    assert_eq!(parsed, envelope);
    Ok(())
}

#[test]
fn rejects_logical_time_outside_i_json_range() {
    let json = format!(
        r#"{{
            "envelope_version":1,
            "receipt_type":"governance",
            "context_digest":"{}",
            "schema_digest":"{}",
            "policy_digest":"{}",
            "logical_time":9007199254740992,
            "event_hash":"{}",
            "payload":{{"hello":"world"}}
        }}"#,
        digest(1),
        digest(2),
        digest(3),
        digest(4)
    );

    let result = serde_json::from_str::<ReceiptEnvelope>(&json);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("invalid I-JSON number"));
    }
}

#[test]
fn rejects_unknown_fields() {
    let json = format!(
        r#"{{
            "envelope_version":1,
            "receipt_type":"governance",
            "context_digest":"{}",
            "schema_digest":"{}",
            "policy_digest":"{}",
            "logical_time":1,
            "event_hash":"{}",
            "payload":{{"k":"v"}},
            "unexpected_field":"should fail"
        }}"#,
        digest(1),
        digest(2),
        digest(3),
        digest(4)
    );

    let result = serde_json::from_str::<ReceiptEnvelope>(&json);
    assert!(result.is_err(), "unknown fields must be rejected");
    if let Err(err) = result {
        assert!(
            err.to_string().contains("unknown field"),
            "error should mention unknown field, got: {err}"
        );
    }
}
