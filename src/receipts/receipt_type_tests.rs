//! Tests for `ReceiptType`.

use crate::ReceiptType;

#[test]
fn serde_round_trip_all_variants() -> Result<(), anyhow::Error> {
    let variants = [
        (ReceiptType::Event, "\"event\""),
        (ReceiptType::Llm, "\"llm\""),
        (ReceiptType::Mri, "\"mri\""),
        (ReceiptType::Governance, "\"governance\""),
        (ReceiptType::Adapter, "\"adapter\""),
        (ReceiptType::Projection, "\"projection\""),
        (ReceiptType::Training, "\"training\""),
    ];
    for (variant, expected_json) in variants {
        let json = serde_json::to_string(&variant)?;
        assert_eq!(
            json, expected_json,
            "serialization mismatch for {variant:?}"
        );
        let parsed: ReceiptType = serde_json::from_str(&json)?;
        assert_eq!(parsed, variant, "round-trip mismatch for {variant:?}");
    }
    Ok(())
}

#[test]
fn deserialize_rejects_unknown_variant() {
    let result: Result<ReceiptType, _> = serde_json::from_str("\"unknown\"");
    assert!(result.is_err());
}

#[test]
fn deserialize_accepts_uppercase() -> Result<(), anyhow::Error> {
    let parsed: ReceiptType = serde_json::from_str("\"Event\"")?;
    assert_eq!(parsed, ReceiptType::Event);
    Ok(())
}

#[test]
fn debug_format() {
    let dbg = format!("{:?}", ReceiptType::Event);
    assert_eq!(dbg, "Event");
}

#[test]
fn equality() {
    assert_eq!(ReceiptType::Llm, ReceiptType::Llm);
    assert_ne!(ReceiptType::Event, ReceiptType::Governance);
}

#[test]
fn ord_is_variant_order() {
    assert!(ReceiptType::Event < ReceiptType::Llm);
    assert!(ReceiptType::Llm < ReceiptType::Mri);
    assert!(ReceiptType::Mri < ReceiptType::Governance);
}

#[test]
fn copy_semantics() {
    let a = ReceiptType::Event;
    let b = a;
    assert_eq!(a, b);
}
