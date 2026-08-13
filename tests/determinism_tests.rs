//! Determinism proof tests for vertrule-schemas P0 surfaces.
//!
//! Layer coverage:
//! - L1: shuffled-input canonical equality (determinism contract: `shuffled_field_order`)
//! - L3: cross-run determinism (same input, repeated invocations → identical bytes)
//! - L4: receipt commitment contract — the `event_hash` is a reproducible,
//!   encoding-invariant, tamper-evident commitment via `BLAKE3(JCS(envelope \\ {event_hash}))`
//!
//! Determinism matrix axes covered:
//! - Same input, same process, repeated N times → identical bytes
//! - Same input, repeated construction → identical bytes
//! - Same semantic input, shuffled field order → identical canonical bytes
//! - Same envelope → same `event_hash`; shuffled key order → same `event_hash`;
//!   one mutated trust-bearing field → different `event_hash`

mod common;

use common::{assert_deterministic, load_vector, need};

use vertrule_schemas::receipts::compute_event_hash;
use vertrule_schemas::{DigestBytes, ReceiptEnvelope};
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

// ---------------------------------------------------------------------------
// L4: Receipt commitment side-channel contract — ReceiptEnvelope + event_hash
// The three properties that make a receipt a truth channel, in one place,
// exercised through the real constitutional law
// `BLAKE3(JCS(envelope \ {event_hash}))` (not a re-implementation):
//   1. reproducible       — same envelope → same event_hash across invocations
//   2. encoding-invariant — different JSON key order deserializes to the same
//                           committed identity (wire layout does not bind)
//   3. tamper-evident     — one mutated trust-bearing field → different digest
// ---------------------------------------------------------------------------

/// Deep-copy a JSON value with the key order of every object reversed.
/// Array element order is preserved (arrays are ordered data, not layout).
/// With `serde_json`'s `preserve_order` on, this yields a re-serialization
/// whose bytes differ only in key order — the input for the invariance limb.
fn reverse_key_order(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.iter()
                .rev()
                .map(|(k, v)| (k.clone(), reverse_key_order(v)))
                .collect(),
        ),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(reverse_key_order).collect())
        }
        other => other.clone(),
    }
}

/// Determinism + integrity axis: an `event_hash` is a reproducible,
/// encoding-invariant, tamper-evident commitment to its envelope.
#[test]
fn receipt_commitment_side_channel_invariants() -> anyhow::Result<()> {
    let vector = load_vector("envelope_roundtrip_001")?;
    let envelope_json = &vector["input"]["envelope"];
    let envelope: ReceiptEnvelope = serde_json::from_value(envelope_json.clone())?;

    // 1. Reproducible: the commitment is byte-stable across repeated calls.
    let base = compute_event_hash(&envelope).map_err(|e| anyhow::anyhow!("{e}"))?;
    assert_deterministic(
        || {
            compute_event_hash(&envelope)
                .map(|d| d.as_bytes().to_vec())
                .map_err(|e| anyhow::anyhow!("{e}"))
        },
        4,
        "receipt_commitment_reproducible",
    )?;

    // 2. Encoding-invariant: reversing every object's key order changes the
    //    wire bytes but not the committed identity.
    let shuffled_json = reverse_key_order(envelope_json);
    // Value equality is key-order-insensitive under `preserve_order`; compare
    // the object's key sequence directly (deterministic IndexMap iteration) to
    // confirm the shuffle actually reordered keys — no serialization needed.
    let key_order = |v: &serde_json::Value| -> Vec<String> {
        v.as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    };
    need(
        (key_order(&shuffled_json) != key_order(envelope_json)).then_some(()),
        "shuffle fixture did not change key order (invariance not under test)",
    )?;
    let shuffled: ReceiptEnvelope = serde_json::from_value(shuffled_json)?;
    let shuffled_hash = compute_event_hash(&shuffled).map_err(|e| anyhow::anyhow!("{e}"))?;
    need(
        (shuffled_hash == base).then_some(()),
        "shuffled key order produced a different event_hash",
    )?;

    // 3. Tamper-evident: mutating one trust-bearing field flips the digest.
    let mut tampered = envelope;
    tampered.logical_time = tampered.logical_time.wrapping_add(1);
    let tampered_hash = compute_event_hash(&tampered).map_err(|e| anyhow::anyhow!("{e}"))?;
    need(
        (tampered_hash != base).then_some(()),
        "a mutated trust-bearing field left event_hash unchanged",
    )?;

    Ok(())
}
