//! Tests for `ReceiptEnvelope`.

use crate::{CanonicalPayload, DigestBytes, ReceiptEnvelope, ReceiptType, SchemaVersion};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

fn payload(value: serde_json::Value) -> Result<CanonicalPayload, anyhow::Error> {
    CanonicalPayload::new(value).map_err(|e| anyhow::anyhow!(e))
}

#[test]
fn serde_round_trip_minimal_envelope() -> Result<(), anyhow::Error> {
    let envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Governance,
        context_digest: digest(1),
        schema_digest: digest(2),
        policy_digest: digest(3),
        logical_time: 7,
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
        logical_time: 1,
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
        logical_time: 9,
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
