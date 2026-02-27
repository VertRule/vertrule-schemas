//! Tests for `SchemaVersion`.

use crate::SchemaVersion;

#[test]
fn v1_constant() {
    assert_eq!(SchemaVersion::V1.get(), 1);
}

#[test]
fn new_and_get() {
    let v = SchemaVersion::new(42);
    assert_eq!(v.get(), 42);
}

#[test]
fn display() {
    assert_eq!(format!("{}", SchemaVersion::V1), "1");
    assert_eq!(format!("{}", SchemaVersion::new(99)), "99");
}

#[test]
fn equality() {
    assert_eq!(SchemaVersion::new(1), SchemaVersion::V1);
}

#[test]
fn inequality() {
    assert_ne!(SchemaVersion::new(1), SchemaVersion::new(2));
}

#[test]
fn ord() {
    assert!(SchemaVersion::new(1) < SchemaVersion::new(2));
    assert!(SchemaVersion::new(10) > SchemaVersion::new(1));
}

#[test]
fn serde_round_trip() -> Result<(), anyhow::Error> {
    let v = SchemaVersion::V1;
    let json = serde_json::to_string(&v)?;
    assert_eq!(json, "1");
    let parsed: SchemaVersion = serde_json::from_str(&json)?;
    assert_eq!(parsed, v);
    Ok(())
}

#[test]
fn serde_transparent() -> Result<(), anyhow::Error> {
    let json = "5";
    let v: SchemaVersion = serde_json::from_str(json)?;
    assert_eq!(v.get(), 5);
    Ok(())
}
