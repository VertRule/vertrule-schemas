use super::*;

fn sample() -> VerifiedReceiptMetadata {
    VerifiedReceiptMetadata::new(
        "ctx".to_string(),
        "pol".to_string(),
        "sch".to_string(),
        "evh".to_string(),
        "Governance".to_string(),
        7,
        Some("bo".to_string()),
        serde_json::json!({ "k": "v" }),
    )
}

// Wire-form contract: default field-name serialization in declaration
// order. This golden pins byte-stability across the relocation from
// `vertrule-verifier::rbh` (ADR-038 Phase 1).
#[test]
fn wire_form_is_field_named_and_ordered() -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(&sample())?;
    assert_eq!(
        json,
        r#"{"context_digest":"ctx","policy_digest":"pol","schema_digest":"sch","event_hash":"evh","receipt_type":"Governance","logical_time":7,"boundary_origin":"bo","payload":{"k":"v"}}"#
    );
    Ok(())
}

#[test]
fn round_trips_through_serde() -> Result<(), serde_json::Error> {
    let original = sample();
    let bytes = serde_json::to_vec(&original)?;
    let back: VerifiedReceiptMetadata = serde_json::from_slice(&bytes)?;
    assert_eq!(original, back);
    Ok(())
}

#[test]
fn absent_boundary_origin_serializes_as_null() -> Result<(), serde_json::Error> {
    let m = VerifiedReceiptMetadata::new(
        "c".to_string(),
        "p".to_string(),
        "s".to_string(),
        "e".to_string(),
        "Governance".to_string(),
        0,
        None,
        serde_json::Value::Null,
    );
    let json = serde_json::to_string(&m)?;
    assert_eq!(
        json,
        r#"{"context_digest":"c","policy_digest":"p","schema_digest":"s","event_hash":"e","receipt_type":"Governance","logical_time":0,"boundary_origin":null,"payload":null}"#
    );
    Ok(())
}

#[test]
fn accessors_read_every_field() {
    let m = sample();
    assert_eq!(m.context_digest(), "ctx");
    assert_eq!(m.policy_digest(), "pol");
    assert_eq!(m.schema_digest(), "sch");
    assert_eq!(m.event_hash(), "evh");
    assert_eq!(m.receipt_type(), "Governance");
    assert_eq!(m.logical_time(), 7);
    assert_eq!(m.boundary_origin(), Some("bo"));
    assert_eq!(m.payload(), &serde_json::json!({ "k": "v" }));
}
