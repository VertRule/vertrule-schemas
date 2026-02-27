//! Tests for `BoundaryOrigin`.

use crate::BoundaryOrigin;

#[test]
fn serde_round_trip_all_variants() -> Result<(), anyhow::Error> {
    let variants = [
        (BoundaryOrigin::Engine, "\"engine\""),
        (BoundaryOrigin::Adapter, "\"adapter\""),
        (BoundaryOrigin::Numeric, "\"numeric\""),
        (BoundaryOrigin::Governance, "\"governance\""),
        (BoundaryOrigin::Model, "\"model\""),
        (BoundaryOrigin::Training, "\"training\""),
    ];
    for (variant, expected_json) in variants {
        let json = serde_json::to_string(&variant)?;
        assert_eq!(
            json, expected_json,
            "serialization mismatch for {variant:?}"
        );
        let parsed: BoundaryOrigin = serde_json::from_str(&json)?;
        assert_eq!(parsed, variant, "round-trip mismatch for {variant:?}");
    }
    Ok(())
}

#[test]
fn deserialize_rejects_unknown_variant() {
    let result: Result<BoundaryOrigin, _> = serde_json::from_str("\"unknown\"");
    assert!(result.is_err());
}

#[test]
fn deserialize_rejects_uppercase() {
    let result: Result<BoundaryOrigin, _> = serde_json::from_str("\"Engine\"");
    assert!(result.is_err());
}

#[test]
fn debug_format() {
    let dbg = format!("{:?}", BoundaryOrigin::Engine);
    assert_eq!(dbg, "Engine");
}

#[test]
fn equality() {
    assert_eq!(BoundaryOrigin::Adapter, BoundaryOrigin::Adapter);
    assert_ne!(BoundaryOrigin::Engine, BoundaryOrigin::Model);
}

#[test]
fn ord_is_variant_order() {
    assert!(BoundaryOrigin::Engine < BoundaryOrigin::Adapter);
    assert!(BoundaryOrigin::Adapter < BoundaryOrigin::Numeric);
    assert!(BoundaryOrigin::Numeric < BoundaryOrigin::Governance);
}

#[test]
fn copy_semantics() {
    let a = BoundaryOrigin::Numeric;
    let b = a;
    assert_eq!(a, b);
}
