//! Tests for `DigestBytes`.

use crate::DigestBytes;

#[test]
fn round_trip_from_array() {
    let bytes = [0xab_u8; 32];
    let digest = DigestBytes::from_array(bytes);
    assert_eq!(digest.as_bytes(), &bytes);
    assert_eq!(digest.to_hex(), "ab".repeat(32));
}

#[test]
fn round_trip_hex() -> Result<(), anyhow::Error> {
    let hex = "a1".repeat(32);
    let digest = DigestBytes::from_hex(&hex)?;
    assert_eq!(digest.to_hex(), hex);
    Ok(())
}

#[test]
fn from_slice_valid() -> Result<(), anyhow::Error> {
    let bytes = [0x01_u8; 32];
    let digest = DigestBytes::from_slice(&bytes)?;
    assert_eq!(digest.as_bytes(), &bytes);
    Ok(())
}

#[test]
fn from_slice_wrong_length() {
    let result = DigestBytes::from_slice(&[0u8; 31]);
    assert!(result.is_err());
}

#[test]
fn rejects_uppercase_hex() {
    let hex = format!("{}A1", "a1".repeat(31));
    let result = DigestBytes::from_hex(&hex);
    assert!(result.is_err());
}

#[test]
fn rejects_short_hex() {
    let hex = "ab".repeat(31);
    let result = DigestBytes::from_hex(&hex);
    assert!(result.is_err());
}

#[test]
fn rejects_long_hex() {
    let hex = "ab".repeat(33);
    let result = DigestBytes::from_hex(&hex);
    assert!(result.is_err());
}

#[test]
fn rejects_whitespace() {
    let hex = format!(" {}", "ab".repeat(31));
    let result = DigestBytes::from_hex(&hex);
    assert!(result.is_err());
}

#[test]
fn rejects_non_hex_chars() {
    let mut hex = "ab".repeat(32);
    hex.replace_range(0..2, "zz");
    let result = DigestBytes::from_hex(&hex);
    assert!(result.is_err());
}

#[test]
fn serde_round_trip() -> Result<(), anyhow::Error> {
    let hex = "cd".repeat(32);
    let digest = DigestBytes::from_hex(&hex)?;
    let json = serde_json::to_string(&digest)?;
    assert_eq!(json, format!("\"{hex}\""));
    let parsed: DigestBytes = serde_json::from_str(&json)?;
    assert_eq!(parsed, digest);
    Ok(())
}

#[test]
fn deserialize_rejects_uppercase() {
    let json = format!("\"{}\"", "AB".repeat(32));
    let result: Result<DigestBytes, _> = serde_json::from_str(&json);
    assert!(result.is_err());
}

#[test]
fn display_matches_hex() {
    let digest = DigestBytes::from_array([0xff; 32]);
    assert_eq!(format!("{digest}"), digest.to_hex());
}

#[test]
fn deterministic_across_runs() -> Result<(), anyhow::Error> {
    let hex = "deadbeef".repeat(8);
    let d1 = DigestBytes::from_hex(&hex)?;
    let d2 = DigestBytes::from_hex(&hex)?;
    let d3 = DigestBytes::from_hex(&hex)?;
    assert_eq!(d1, d2);
    assert_eq!(d2, d3);
    assert_eq!(d1.to_hex(), d2.to_hex());
    assert_eq!(d2.to_hex(), d3.to_hex());
    Ok(())
}
