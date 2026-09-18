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

// ── Frozen BEFORE fixture (ADR-056 §9, C11) ─────────────────────────

/// The committed V1 twin of the C4 golden `DecisionPayload`, minted once
/// through the V1 constructor before that constructor was removed (R12).
/// This test only *reads* the file: it never mints. It deserialises the
/// canonical bytes as the frozen V1 [`ReceiptEnvelope`], recomputes the
/// `event_hash` through the verifier-only path (K7) and asserts the pinned
/// bytes byte-for-byte.
#[test]
fn before_fixture_v1_governance_decision_is_frozen() -> Result<(), Box<dyn std::error::Error>> {
    const PINNED_EVENT_HASH: &str =
        "eca50079364b87be8f69f4ff9c4cb435a2db51ca2aa71c17e2a725f94e5a4ad3";
    const PINNED_SCHEMA_DIGEST: &str =
        "4c5d6d82fe9df544090a072dfbfe28a292d3c2315bd6d0045203856ddb157852";

    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-vectors")
        .join("receipt_v1_governance_decision_before_001.json");
    let fixture: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let field = |key: &str| -> Result<String, Box<dyn std::error::Error>> {
        fixture[key]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("fixture field {key:?} missing or not a string").into())
    };

    assert_eq!(
        field("case_id")?,
        "receipt_v1_governance_decision_before_001"
    );
    assert_eq!(field("event_hash")?, PINNED_EVENT_HASH);
    assert_eq!(field("schema_digest")?, PINNED_SCHEMA_DIGEST);

    let canonical_bytes = field("canonical_bytes")?;
    let envelope: ReceiptEnvelope = serde_json::from_str(&canonical_bytes)?;
    assert_eq!(envelope.envelope_version.get(), 1);
    assert_eq!(envelope.event_hash.to_hex(), PINNED_EVENT_HASH);
    assert_eq!(envelope.schema_digest.to_hex(), PINNED_SCHEMA_DIGEST);

    // Verifier-only recompute reproduces the pinned identity.
    assert_eq!(
        compute_event_hash(&envelope)?.to_hex(),
        PINNED_EVENT_HASH,
        "V1 event_hash of the BEFORE fixture drifted"
    );

    // The pinned bytes are exactly the JCS of the frozen envelope.
    let reserialised = vr_jcs::to_canon_string_from_str(&serde_json::to_string(&envelope)?)?;
    assert_eq!(
        reserialised, canonical_bytes,
        "BEFORE fixture canonical bytes drifted"
    );
    Ok(())
}
