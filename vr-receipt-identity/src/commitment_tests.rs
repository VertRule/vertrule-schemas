//! Constitutional receipt-law known-answer and invariant tests.

use super::compute_event_hash;
use crate::error::ReceiptIdentityError;
use vertrule_schemas::ReceiptEnvelope;

/// Canonical fixture matching `vertrule-schemas`'s `make_envelope()`
/// known-answer test (`commitment_tests.rs::known_answer_event_hash`):
/// Event receipt, zero digests, `logical_time` 1, Engine origin,
/// payload `{"v":1}`. `event_hash` is a placeholder (stripped from its
/// own preimage).
fn fixture_envelope() -> Result<ReceiptEnvelope, ReceiptIdentityError> {
    let zeros = "0".repeat(64);
    let json = serde_json::json!({
        "envelope_version": 1,
        "receipt_type": "event",
        "context_digest": zeros,
        "schema_digest": zeros,
        "policy_digest": zeros,
        "logical_time": 1,
        "event_hash": zeros,
        "boundary_origin": "engine",
        "payload": {"v": 1}
    });
    serde_json::from_value(json).map_err(|e| ReceiptIdentityError::InvalidPayload(e.to_string()))
}

/// G1 known-answer: the new owner reproduces the committed golden.
#[test]
fn g1_commitment_matches_golden() -> Result<(), ReceiptIdentityError> {
    const G1_GOLDEN: &str = "2b62926780e07ca5117c3befb3bf5064a682a6c8cff6389e4f2aa80fc9939cf2";
    let envelope = fixture_envelope()?;
    let digest = compute_event_hash(&envelope)?;
    assert_eq!(
        digest.to_hex(),
        G1_GOLDEN,
        "vr-receipt-identity commitment drifted from G1 golden"
    );
    Ok(())
}

#[test]
fn event_hash_placeholder_is_excluded_from_preimage() -> Result<(), ReceiptIdentityError> {
    let mut envelope = fixture_envelope()?;
    let base = compute_event_hash(&envelope)?;
    envelope.event_hash = vertrule_schemas::DigestBytes::from_array([0xAA; 32]);
    assert_eq!(base, compute_event_hash(&envelope)?);
    Ok(())
}

#[test]
fn trust_bearing_mutation_changes_identity() -> Result<(), ReceiptIdentityError> {
    let mut envelope = fixture_envelope()?;
    let base = compute_event_hash(&envelope)?;
    envelope.logical_time += 1;
    assert_ne!(base, compute_event_hash(&envelope)?);
    Ok(())
}
