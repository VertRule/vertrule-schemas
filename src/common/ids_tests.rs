//! Tests for `PolicyId`.

use crate::PolicyId;

// ── Positive cases ──────────────────────────────────────────────────

#[test]
fn accepts_simple_name() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("determinism".to_string())?;
    assert_eq!(id.as_str(), "determinism");
    Ok(())
}

#[test]
fn accepts_hyphenated() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("repo-boundary".to_string())?;
    assert_eq!(id.as_str(), "repo-boundary");
    Ok(())
}

#[test]
fn accepts_versioned() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("numeric-safety@0.1".to_string())?;
    assert_eq!(id.as_str(), "numeric-safety@0.1");
    Ok(())
}

#[test]
fn accepts_dotted_path() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("vr.policy/determinism".to_string())?;
    assert_eq!(id.as_str(), "vr.policy/determinism");
    Ok(())
}

#[test]
fn accepts_colon_separated() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("policy:set-1".to_string())?;
    assert_eq!(id.as_str(), "policy:set-1");
    Ok(())
}

#[test]
fn accepts_single_char() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("x".to_string())?;
    assert_eq!(id.as_str(), "x");
    Ok(())
}

#[test]
fn accepts_max_length() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("a".repeat(PolicyId::MAX_LEN))?;
    assert_eq!(id.as_str().len(), PolicyId::MAX_LEN);
    Ok(())
}

// ── Display ─────────────────────────────────────────────────────────

#[test]
fn display() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("repo-boundary".to_string())?;
    assert_eq!(format!("{id}"), "repo-boundary");
    Ok(())
}

// ── Equality / ordering ─────────────────────────────────────────────

#[test]
fn equality() -> Result<(), anyhow::Error> {
    let a = PolicyId::new("determinism".to_string())?;
    let b = PolicyId::new("determinism".to_string())?;
    assert_eq!(a, b);
    Ok(())
}

#[test]
fn inequality() -> Result<(), anyhow::Error> {
    let a = PolicyId::new("determinism".to_string())?;
    let b = PolicyId::new("repo-boundary".to_string())?;
    assert_ne!(a, b);
    Ok(())
}

#[test]
fn ord_is_lexicographic() -> Result<(), anyhow::Error> {
    let a = PolicyId::new("aaa".to_string())?;
    let b = PolicyId::new("zzz".to_string())?;
    assert!(a < b);
    Ok(())
}

// ── Serde ───────────────────────────────────────────────────────────

#[test]
fn serde_round_trip() -> Result<(), anyhow::Error> {
    let id = PolicyId::new("determinism@0.1".to_string())?;
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

// ── Negative cases ──────────────────────────────────────────────────

#[test]
fn rejects_empty() {
    assert!(PolicyId::new(String::new()).is_err());
}

#[test]
fn rejects_whitespace_only() {
    assert!(PolicyId::new("   ".to_string()).is_err());
}

#[test]
fn rejects_leading_space() {
    assert!(PolicyId::new(" determinism".to_string()).is_err());
}

#[test]
fn rejects_trailing_space() {
    assert!(PolicyId::new("determinism ".to_string()).is_err());
}

#[test]
fn rejects_newline() {
    assert!(PolicyId::new("foo\nbar".to_string()).is_err());
}

#[test]
fn rejects_tab() {
    assert!(PolicyId::new("foo\tbar".to_string()).is_err());
}

#[test]
fn rejects_null_byte() {
    assert!(PolicyId::new("foo\0bar".to_string()).is_err());
}

#[test]
fn rejects_emoji() {
    assert!(PolicyId::new("policy-\u{1F600}".to_string()).is_err());
}

#[test]
fn rejects_over_max_length() {
    assert!(PolicyId::new("a".repeat(PolicyId::MAX_LEN + 1)).is_err());
}

#[test]
fn rejects_space_in_middle() {
    assert!(PolicyId::new("foo bar".to_string()).is_err());
}

#[test]
fn rejects_control_char() {
    assert!(PolicyId::new("foo\x07bar".to_string()).is_err());
}

// ── Serde rejection ─────────────────────────────────────────────────

#[test]
fn deserialize_rejects_empty() {
    let result: Result<PolicyId, _> = serde_json::from_str("\"\"");
    assert!(result.is_err());
}

#[test]
fn deserialize_rejects_invalid_char() {
    let result: Result<PolicyId, _> = serde_json::from_str("\"foo bar\"");
    assert!(result.is_err());
}
