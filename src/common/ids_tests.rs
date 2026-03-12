//! Tests for `PolicyId`.

use crate::PolicyId;

#[test]
fn new_and_as_str() {
    let id = PolicyId::new("determinism".to_string());
    assert_eq!(id.as_str(), "determinism");
}

#[test]
fn display() {
    let id = PolicyId::new("repo-boundary".to_string());
    assert_eq!(format!("{id}"), "repo-boundary");
}

#[test]
fn equality() {
    let a = PolicyId::new("determinism".to_string());
    let b = PolicyId::new("determinism".to_string());
    assert_eq!(a, b);
}

#[test]
fn inequality() {
    let a = PolicyId::new("determinism".to_string());
    let b = PolicyId::new("repo-boundary".to_string());
    assert_ne!(a, b);
}

#[test]
fn ord_is_lexicographic() {
    let a = PolicyId::new("aaa".to_string());
    let b = PolicyId::new("zzz".to_string());
    assert!(a < b);
}

#[test]
fn serde_round_trip() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("determinism@0.1".to_string());
    let json = serde_json::to_string(&id)?;
    assert_eq!(json, "\"determinism@0.1\"");
    let parsed: PolicyId = serde_json::from_str(&json)?;
    assert_eq!(parsed, id);
    Ok(())
}

#[test]
fn serde_transparent() -> Result<(), anyhow::Error> {
    let json = "\"some-policy\"";
    let id: PolicyId = serde_json::from_str(json)?;
    assert_eq!(id.as_str(), "some-policy");
    Ok(())
}
