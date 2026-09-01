//! Determinism proof tests for vertrule-schemas P0 surfaces.
//!
//! Layer coverage:
//! - L1: shuffled-input canonical equality (determinism contract: `shuffled_field_order`)
//! - L3: cross-run determinism (same input, repeated invocations → identical bytes)
//!
//! Determinism matrix axes covered:
//! - Same input, same process, repeated N times → identical bytes
//! - Same input, repeated construction → identical bytes
//! - Same semantic input, shuffled field order → identical canonical bytes

mod common;

use common::{assert_deterministic, load_vector, need};

use vertrule_schemas::DigestBytes;
use vr_jcs::to_canon_bytes_from_slice;

// ---------------------------------------------------------------------------
// L1: Shuffled-input canonical equality — JCS
// Determinism contract: shuffled_field_order
// ---------------------------------------------------------------------------

/// Determinism axis: shuffled field order → identical canonical bytes.
/// Two JSON objects with identical content but different key insertion
/// order produce bitwise-identical canonical bytes and BLAKE3 digests.
#[test]
fn jcs_shuffle_invariant_001() -> anyhow::Result<()> {
    let vector = load_vector("jcs_shuffle_invariant_001")?;
    let variant_a = &vector["input"]["variant_a"];
    let variant_b = &vector["input"]["variant_b"];
    let expected_string = need(
        vector["expected"]["canonical_string"].as_str(),
        "expected.canonical_string",
    )?;
    let expected_blake3 = need(
        vector["expected"]["blake3_hex"].as_str(),
        "expected.blake3_hex",
    )?;

    let json_a = serde_json::to_vec(variant_a)?;
    let json_b = serde_json::to_vec(variant_b)?;
    let bytes_a = to_canon_bytes_from_slice(&json_a)?;
    let bytes_b = to_canon_bytes_from_slice(&json_b)?;

    // Bitwise identity between shuffled variants
    need(
        (bytes_a == bytes_b).then_some(()),
        "shuffled variants produced different canonical bytes",
    )?;

    // Match expected canonical string
    let canonical = String::from_utf8(bytes_a.clone())
        .map_err(|e| anyhow::anyhow!("canonical bytes not valid UTF-8: {e}"))?;
    need(
        (canonical == expected_string).then_some(()),
        "canonical string does not match expected",
    )?;

    // Match expected BLAKE3 digest
    let digest = blake3::hash(&bytes_a).to_hex().to_string();
    need(
        (digest == expected_blake3).then_some(()),
        "BLAKE3 digest does not match expected",
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// L3: Cross-run determinism — JCS
// Determinism axis: same input, repeated N times → identical bytes
// ---------------------------------------------------------------------------

/// Determinism axis: repeated invocation → identical bytes.
/// `to_canon_bytes_from_slice` called 5 times on the same input produces
/// identical output.
#[test]
fn jcs_repeated_invocation_determinism() -> anyhow::Result<()> {
    let input = serde_json::json!({
        "z_field": 999,
        "a_field": 1,
        "nested": {"z": true, "a": false},
        "array": [3, 2, 1]
    });

    let json = serde_json::to_vec(&input)?;
    assert_deterministic(
        || to_canon_bytes_from_slice(&json).map_err(|e| anyhow::anyhow!("{e}")),
        5,
        "jcs_repeated_invocation",
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// L3: Cross-run determinism — DigestBytes
// Determinism axis: repeated construction → identical bytes
// ---------------------------------------------------------------------------

/// Determinism axis: repeated construction → identical bytes.
/// `DigestBytes::from_hex` called 5 times produces identical internal state.
#[test]
fn digest_bytes_repeated_construction_determinism() -> anyhow::Result<()> {
    let hex = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

    assert_deterministic(
        || {
            let d = DigestBytes::from_hex(hex)?;
            Ok(d.as_bytes().to_vec())
        },
        5,
        "digest_bytes_repeated_construction",
    )?;

    Ok(())
}
