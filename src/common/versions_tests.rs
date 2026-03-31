//! Tests for `SchemaVersion`.

use crate::SchemaVersion;

// ── Positive cases ──────────────────────────────────────────────────

#[test]
fn v1_constant() {
    assert_eq!(SchemaVersion::V1.get(), 1);
}

#[test]
fn new_v1() -> Result<(), anyhow::Error> {
    let v = SchemaVersion::new(1)?;
    assert_eq!(v, SchemaVersion::V1);
    assert_eq!(v.get(), 1);
    Ok(())
}

#[test]
fn display() {
    assert_eq!(format!("{}", SchemaVersion::V1), "1");
}

#[test]
fn equality() -> Result<(), anyhow::Error> {
    assert_eq!(SchemaVersion::new(1)?, SchemaVersion::V1);
    Ok(())
}

// ── Version-derived bindings ────────────────────────────────────────

#[test]
fn v1_digest_algorithm() {
    assert_eq!(SchemaVersion::V1.digest_algorithm(), "BLAKE3");
}

#[test]
fn v1_canonicalization() {
    assert_eq!(SchemaVersion::V1.canonicalization(), "JCS");
}

// ── Ordering ────────────────────────────────────────────────────────

#[test]
fn ord() {
    assert_eq!(SchemaVersion::V1, SchemaVersion::V1);
}

// ── Serde ───────────────────────────────────────────────────────────

#[test]
fn serde_round_trip() -> Result<(), anyhow::Error> {
    let v = SchemaVersion::V1;
    let json = serde_json::to_string(&v)?;
    assert_eq!(json, "1");
    let parsed: SchemaVersion = serde_json::from_str(&json)?;
    assert_eq!(parsed, v);
    Ok(())
}

// ── Negative cases ──────────────────────────────────────────────────

#[test]
fn rejects_zero() {
    assert!(SchemaVersion::new(0).is_err());
}

#[test]
fn rejects_two() {
    assert!(SchemaVersion::new(2).is_err());
}

#[test]
fn rejects_unsupported_version() {
    assert!(SchemaVersion::new(42).is_err());
}

#[test]
fn rejects_u32_max() {
    assert!(SchemaVersion::new(u32::MAX).is_err());
}

#[test]
fn deserialize_rejects_unsupported() {
    let result: Result<SchemaVersion, _> = serde_json::from_str("999");
    assert!(result.is_err());
}

#[test]
fn deserialize_rejects_zero() {
    let result: Result<SchemaVersion, _> = serde_json::from_str("0");
    assert!(result.is_err());
}

#[test]
fn deserialize_rejects_two() {
    let result: Result<SchemaVersion, _> = serde_json::from_str("2");
    assert!(result.is_err());
}
