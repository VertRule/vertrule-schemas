//! Fixture-driven integration tests for vertrule-schemas.
//!
//! Loads committed test vectors from `test-vectors/` and validates
//! that the schema crate produces correct outputs and rejections.
//!
//! Layer coverage:
//! - L1: known-answer vectors (JCS roundtrip, digest bytes valid, envelope roundtrip)
//! - L2: adversarial rejection (digest short, digest chars, envelope missing field)
//!
//! Rejection strength:
//! - Digest rejection tests use R1 (typed `DefinitionError::InvalidDigest` match)
//! - Envelope rejection tests use R2 (deserialization error class)

mod common;

use common::{assert_error_contains, load_vector, need};

use vertrule_schemas::{DefinitionError, DigestBytes, ReceiptEnvelope};
use vr_jcs::to_canon_bytes_from_slice;

// ---------------------------------------------------------------------------
// L1: Known-answer vectors — JCS
// ---------------------------------------------------------------------------

/// L1 vector: JCS canonicalization produces expected canonical string and BLAKE3 digest.
#[test]
fn jcs_roundtrip_001() -> anyhow::Result<()> {
    let vector = load_vector("jcs_roundtrip_001")?;
    let input = &vector["input"]["json"];
    let expected_string = need(
        vector["expected"]["canonical_string"].as_str(),
        "expected.canonical_string",
    )?;
    let expected_blake3 = need(
        vector["expected"]["blake3_hex"].as_str(),
        "expected.blake3_hex",
    )?;

    let json = serde_json::to_vec(input)?;
    let canon_bytes = to_canon_bytes_from_slice(&json)?;
    let canon_string = String::from_utf8(canon_bytes.clone())
        .map_err(|e| anyhow::anyhow!("canonical bytes not valid UTF-8: {e}"))?;

    need(
        (canon_string == expected_string).then_some(()),
        "canonical string mismatch",
    )?;

    let digest = blake3::hash(&canon_bytes);
    let digest_hex = digest.to_hex().to_string();

    need(
        (digest_hex == expected_blake3).then_some(()),
        "BLAKE3 digest mismatch",
    )?;

    Ok(())
}

/// L1 vector: raw RFC 8785 primitive sample canonicalizes to the known answer.
#[test]
fn jcs_rfc8785_primitives_001() -> anyhow::Result<()> {
    let vector = load_vector("jcs_rfc8785_primitives_001")?;
    let input = need(vector["input"]["json"].as_str(), "input.json")?;
    let expected_string = need(
        vector["expected"]["canonical_string"].as_str(),
        "expected.canonical_string",
    )?;

    let canon_bytes = to_canon_bytes_from_slice(input.as_bytes())?;
    let canon_string = String::from_utf8(canon_bytes)
        .map_err(|e| anyhow::anyhow!("canonical bytes not valid UTF-8: {e}"))?;

    need(
        (canon_string == expected_string).then_some(()),
        "canonical string mismatch",
    )?;

    Ok(())
}

/// L1 vector: UTF-16 property sorting matches the RFC 8785 known-answer example.
#[test]
fn jcs_utf16_sort_001() -> anyhow::Result<()> {
    let vector = load_vector("jcs_utf16_sort_001")?;
    let input = need(vector["input"]["json"].as_str(), "input.json")?;
    let expected_string = need(
        vector["expected"]["canonical_string"].as_str(),
        "expected.canonical_string",
    )?;

    let canon_bytes = to_canon_bytes_from_slice(input.as_bytes())?;
    let canon_string = String::from_utf8(canon_bytes)
        .map_err(|e| anyhow::anyhow!("canonical bytes not valid UTF-8: {e}"))?;

    need(
        (canon_string == expected_string).then_some(()),
        "canonical string mismatch",
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// L1: Known-answer vectors — DigestBytes
// ---------------------------------------------------------------------------

/// L1 vector: valid hex string round-trips through `DigestBytes`.
#[test]
fn digest_bytes_valid_001() -> anyhow::Result<()> {
    let vector = load_vector("digest_bytes_valid_001")?;
    let input_hex = need(vector["input"]["hex"].as_str(), "input.hex")?;
    let expected_hex = need(
        vector["expected"]["hex_roundtrip"].as_str(),
        "expected.hex_roundtrip",
    )?;

    let digest = DigestBytes::from_hex(input_hex)?;

    need(
        (digest.to_hex() == expected_hex).then_some(()),
        "hex round-trip mismatch",
    )?;

    need(
        (digest.as_bytes().len() == 32).then_some(()),
        "byte length not 32",
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// L2: Rejection — DigestBytes (R1: typed error variant match)
// ---------------------------------------------------------------------------

/// L2 reject, R1 strength: short hex string rejected as `InvalidDigest`.
#[test]
fn digest_bytes_reject_short_001() -> anyhow::Result<()> {
    let vector = load_vector("digest_bytes_reject_short_001")?;
    let input_hex = need(vector["input"]["hex"].as_str(), "input.hex")?;

    let result = DigestBytes::from_hex(input_hex);

    // R1: match the exact error variant
    match result {
        Ok(_) => {
            return Err(anyhow::anyhow!(
                "[digest_bytes_reject_short_001] expected rejection, got success"
            ))
        }
        Err(DefinitionError::InvalidDigest(msg)) => {
            need(
                msg.contains("64").then_some(()),
                "InvalidDigest message should mention expected length 64",
            )?;
        }
        Err(other) => {
            return Err(anyhow::anyhow!(
                "[digest_bytes_reject_short_001] expected InvalidDigest, got: {other}"
            ))
        }
    }

    Ok(())
}

/// L2 reject, R1 strength: uppercase hex rejected as `InvalidDigest`.
#[test]
fn digest_bytes_reject_chars_001() -> anyhow::Result<()> {
    let vector = load_vector("digest_bytes_reject_chars_001")?;
    let input_hex = need(vector["input"]["hex"].as_str(), "input.hex")?;

    let result = DigestBytes::from_hex(input_hex);

    // R1: match the exact error variant
    match result {
        Ok(_) => {
            return Err(anyhow::anyhow!(
                "[digest_bytes_reject_chars_001] expected rejection, got success"
            ))
        }
        Err(DefinitionError::InvalidDigest(msg)) => {
            need(
                msg.contains("non-lowercase-hex").then_some(()),
                "InvalidDigest message should mention non-lowercase-hex",
            )?;
        }
        Err(other) => {
            return Err(anyhow::anyhow!(
                "[digest_bytes_reject_chars_001] expected InvalidDigest, got: {other}"
            ))
        }
    }

    Ok(())
}

/// L2 reject, R2 strength: duplicate object members are rejected on raw JSON input.
#[test]
fn jcs_reject_duplicate_key_001() -> anyhow::Result<()> {
    let vector = load_vector("jcs_reject_duplicate_key_001")?;
    let input = need(vector["input"]["json"].as_str(), "input.json")?;
    let expected_contains = need(
        vector["expected_error"]["contains"].as_str(),
        "expected_error.contains",
    )?;

    let result = to_canon_bytes_from_slice(input.as_bytes());
    assert_error_contains(result, expected_contains, "jcs_reject_duplicate_key_001")?;

    Ok(())
}

/// L2 reject, R2 strength: forbidden noncharacters are rejected on raw JSON input.
#[test]
fn jcs_reject_noncharacter_001() -> anyhow::Result<()> {
    let vector = load_vector("jcs_reject_noncharacter_001")?;
    let input = need(vector["input"]["json"].as_str(), "input.json")?;
    let expected_contains = need(
        vector["expected_error"]["contains"].as_str(),
        "expected_error.contains",
    )?;

    let result = to_canon_bytes_from_slice(input.as_bytes());
    assert_error_contains(result, expected_contains, "jcs_reject_noncharacter_001")?;

    Ok(())
}

// ---------------------------------------------------------------------------
// L1: Known-answer vectors — ReceiptEnvelope
// ---------------------------------------------------------------------------

/// L1 vector: minimal envelope serializes, deserializes, and round-trips identically.
/// Also verifies JCS canonical BLAKE3 digest matches frozen value.
#[test]
fn envelope_roundtrip_001() -> anyhow::Result<()> {
    let vector = load_vector("envelope_roundtrip_001")?;
    let envelope_json = &vector["input"]["envelope"];
    let expected_blake3 = need(
        vector["expected"]["canonical_blake3_hex"].as_str(),
        "expected.canonical_blake3_hex",
    )?;

    // Deserialize from fixture
    let envelope: ReceiptEnvelope = serde_json::from_value(envelope_json.clone())?;

    // Round-trip: serialize then deserialize
    let serialized = serde_json::to_string(&envelope)?;
    let reparsed: ReceiptEnvelope = serde_json::from_str(&serialized)?;

    need(
        (envelope == reparsed).then_some(()),
        "envelope round-trip mismatch",
    )?;

    // Verify canonical BLAKE3 digest
    let json = serde_json::to_vec(&envelope)?;
    let canon_bytes = to_canon_bytes_from_slice(&json)?;
    let digest_hex = blake3::hash(&canon_bytes).to_hex().to_string();

    need(
        (digest_hex == expected_blake3).then_some(()),
        "canonical BLAKE3 digest mismatch",
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// L2: Rejection — ReceiptEnvelope (R2: deserialization error class)
// ---------------------------------------------------------------------------

/// L2 reject, R2 strength: envelope missing `event_hash` fails deserialization.
#[test]
fn envelope_reject_missing_field_001() -> anyhow::Result<()> {
    let vector = load_vector("envelope_reject_missing_field_001")?;
    let envelope_json = &vector["input"]["envelope_json"];
    let expected_contains = need(
        vector["expected_error"]["contains"].as_str(),
        "expected_error.contains",
    )?;

    let result: Result<ReceiptEnvelope, _> = serde_json::from_value(envelope_json.clone());

    assert_error_contains(
        result,
        expected_contains,
        "envelope_reject_missing_field_001",
    )?;

    Ok(())
}
