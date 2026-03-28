//! Tests for `ReceiptEnvelope`.

use crate::{
    BoundaryOrigin, CanonicalPayload, DefinitionError, DigestBytes, IJsonUInt, ReceiptEnvelope,
    ReceiptType, SchemaVersion,
};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

fn payload(value: serde_json::Value) -> Result<CanonicalPayload, anyhow::Error> {
    CanonicalPayload::new(value).map_err(|e| anyhow::anyhow!(e))
}

fn payload_def(value: serde_json::Value) -> Result<CanonicalPayload, DefinitionError> {
    CanonicalPayload::new(value).map_err(DefinitionError::InvalidPayload)
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

// ── ReceiptEnvelope::new ───────────────────────────────────────────

#[test]
fn new_v2_produces_valid_envelope() -> Result<(), DefinitionError> {
    let envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Governance,
        digest(0xaa),
        digest(0xbb),
        digest(0xcc),
        IJsonUInt::new(100)?,
        None,
        Some(BoundaryOrigin::Engine),
        payload_def(serde_json::json!({"action": "test"}))?,
    )?;

    assert_eq!(envelope.envelope_version, SchemaVersion::V2);
    assert_eq!(envelope.digest_algorithm.as_deref(), Some("BLAKE3"));
    assert_eq!(envelope.canonicalization.as_deref(), Some("JCS"));
    envelope.validate_integrity()?;
    Ok(())
}

#[test]
fn new_v1_produces_valid_envelope() -> Result<(), DefinitionError> {
    let envelope = ReceiptEnvelope::new(
        SchemaVersion::V1,
        ReceiptType::Event,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(1)?,
        None,
        None,
        payload_def(serde_json::json!({"v": 1}))?,
    )?;

    assert_eq!(envelope.envelope_version, SchemaVersion::V1);
    envelope.validate_integrity()?;
    Ok(())
}

#[test]
fn new_sets_markers_from_version() -> Result<(), DefinitionError> {
    let envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Mri,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(1)?,
        None,
        Some(BoundaryOrigin::Engine),
        payload(serde_json::json!({"layer": 0}))
            .map_err(|e| DefinitionError::InvalidPayload(e.to_string()))?,
    )?;

    assert_eq!(
        envelope.digest_algorithm.as_deref(),
        Some(SchemaVersion::V2.digest_algorithm())
    );
    assert_eq!(
        envelope.canonicalization.as_deref(),
        Some(SchemaVersion::V2.canonicalization())
    );
    Ok(())
}

#[test]
fn new_with_parent_id() -> Result<(), DefinitionError> {
    let parent = digest(0xff);
    let envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Event,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(2)?,
        Some(parent),
        None,
        payload(serde_json::json!({"seq": 2}))
            .map_err(|e| DefinitionError::InvalidPayload(e.to_string()))?,
    )?;

    assert_eq!(envelope.parent_id, Some(parent));
    envelope.validate_integrity()?;
    Ok(())
}

// ── validate_integrity ─────────────────────────────────────────────

#[test]
fn validate_integrity_passes_for_correctly_built_envelope() -> Result<(), DefinitionError> {
    let envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Governance,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(1)?,
        None,
        Some(BoundaryOrigin::Governance),
        payload(serde_json::json!({"ok": true}))
            .map_err(|e| DefinitionError::InvalidPayload(e.to_string()))?,
    )?;

    envelope.validate_integrity()?;
    Ok(())
}

#[test]
fn validate_integrity_fails_on_tampered_event_hash() -> Result<(), DefinitionError> {
    let mut envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Event,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(1)?,
        None,
        None,
        payload_def(serde_json::json!({"v": 1}))?,
    )?;

    envelope.event_hash = digest(0xff);
    let result = envelope.validate_integrity();
    assert!(result.is_err());
    let err = result.err().ok_or_else(|| {
        DefinitionError::IntegrityViolation("expected Err but got Ok".to_string())
    })?;
    assert!(err.to_string().contains("integrity violation"));
    Ok(())
}

#[test]
fn validate_integrity_fails_on_wrong_digest_algorithm_marker() -> Result<(), DefinitionError> {
    let mut envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Event,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(1)?,
        None,
        None,
        payload_def(serde_json::json!({"v": 1}))?,
    )?;

    envelope.digest_algorithm = Some("SHA-256".to_string());
    let result = envelope.validate_integrity();
    assert!(result.is_err());
    let err = result.err().ok_or_else(|| {
        DefinitionError::IntegrityViolation("expected Err but got Ok".to_string())
    })?;
    assert!(err.to_string().contains("marker mismatch"));
    assert!(err.to_string().contains("digest_algorithm"));
    Ok(())
}

#[test]
fn validate_integrity_fails_on_wrong_canonicalization_marker() -> Result<(), DefinitionError> {
    let mut envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Event,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(1)?,
        None,
        None,
        payload_def(serde_json::json!({"v": 1}))?,
    )?;

    envelope.canonicalization = Some("XML-C14N".to_string());
    let result = envelope.validate_integrity();
    assert!(result.is_err());
    let err = result.err().ok_or_else(|| {
        DefinitionError::IntegrityViolation("expected Err but got Ok".to_string())
    })?;
    assert!(err.to_string().contains("marker mismatch"));
    assert!(err.to_string().contains("canonicalization"));
    Ok(())
}

#[test]
fn validate_integrity_passes_with_absent_markers() -> Result<(), DefinitionError> {
    // Markers are optional — absent means "implied by version"
    let mut envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Event,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(1)?,
        None,
        None,
        payload_def(serde_json::json!({"v": 1}))?,
    )?;

    // Remove markers — should still validate since absence is allowed
    envelope.digest_algorithm = None;
    envelope.canonicalization = None;
    // Must recompute event_hash since V2 commits these fields
    envelope.event_hash = crate::compute_event_hash(&envelope)?;
    envelope.validate_integrity()?;
    Ok(())
}

#[test]
fn validate_integrity_detects_tampered_payload_on_v2() -> Result<(), DefinitionError> {
    let mut envelope = ReceiptEnvelope::new(
        SchemaVersion::V2,
        ReceiptType::Event,
        digest(1),
        digest(2),
        digest(3),
        IJsonUInt::new(1)?,
        None,
        None,
        payload(serde_json::json!({"original": true}))
            .map_err(|e| DefinitionError::InvalidPayload(e.to_string()))?,
    )?;

    // Tamper payload without recomputing event_hash
    envelope.payload = payload(serde_json::json!({"tampered": true}))
        .map_err(|e| DefinitionError::InvalidPayload(e.to_string()))?;

    let result = envelope.validate_integrity();
    let err = result.err().ok_or_else(|| {
        DefinitionError::IntegrityViolation("expected Err but got Ok".to_string())
    })?;
    assert!(err.to_string().contains("integrity violation"));
    Ok(())
}
