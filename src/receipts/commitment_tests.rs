use super::compute_event_hash;
use crate::{
    BoundaryOrigin, CanonicalPayload, DefinitionError, DigestBytes, ReceiptEnvelope, ReceiptType,
    SchemaVersion,
};

fn zero_digest() -> DigestBytes {
    DigestBytes::from_array([0u8; 32])
}

fn make_envelope() -> Result<ReceiptEnvelope, DefinitionError> {
    let payload = CanonicalPayload::new(serde_json::json!({"v": 1}))?;

    let mut envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Event,
        context_digest: zero_digest(),
        schema_digest: zero_digest(),
        policy_digest: zero_digest(),
        logical_time: 1,
        event_hash: zero_digest(), // placeholder
        event_hash_profile: None,
        parent_id: None,
        boundary_origin: Some(BoundaryOrigin::Engine),
        digest_algorithm: None,
        canonicalization: None,
        payload,
    };
    envelope.event_hash = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    Ok(envelope)
}

// ── Commitment correctness ────────────────────────────────────────────

#[test]
fn event_hash_matches_recomputed() -> Result<(), DefinitionError> {
    let envelope = make_envelope()?;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_eq!(envelope.event_hash, recomputed);
    Ok(())
}

#[test]
fn deterministic() -> Result<(), DefinitionError> {
    let e1 = make_envelope()?;
    let e2 = make_envelope()?;
    assert_eq!(e1.event_hash, e2.event_hash);
    Ok(())
}

/// Frozen known-answer test: the `event_hash` of `make_envelope()` must
/// equal this specific hex digest. If this test fails, the commitment
/// model has changed — either intentionally (update the constant) or
/// as a regression.
#[test]
fn known_answer_event_hash() -> Result<(), DefinitionError> {
    const EXPECTED: &str = "2b62926780e07ca5117c3befb3bf5064a682a6c8cff6389e4f2aa80fc9939cf2";
    let envelope = make_envelope()?;
    assert_eq!(
        envelope.event_hash.to_hex(),
        EXPECTED,
        "compute_event_hash known-answer mismatch — commitment model may have changed"
    );
    Ok(())
}

// ── Tamper detection: every trust-bearing field ───────────────────────

#[test]
fn tamper_receipt_type() -> Result<(), DefinitionError> {
    let mut envelope = make_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.receipt_type = ReceiptType::Governance;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing receipt_type must change event_hash"
    );
    Ok(())
}

#[test]
fn tamper_context_digest() -> Result<(), DefinitionError> {
    let mut envelope = make_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.context_digest = DigestBytes::from_array([1u8; 32]);
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing context_digest must change event_hash"
    );
    Ok(())
}

#[test]
fn tamper_schema_digest() -> Result<(), DefinitionError> {
    let mut envelope = make_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.schema_digest = DigestBytes::from_array([2u8; 32]);
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing schema_digest must change event_hash"
    );
    Ok(())
}

#[test]
fn tamper_policy_digest() -> Result<(), DefinitionError> {
    let mut envelope = make_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.policy_digest = DigestBytes::from_array([3u8; 32]);
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing policy_digest must change event_hash"
    );
    Ok(())
}

#[test]
fn tamper_logical_time() -> Result<(), DefinitionError> {
    let mut envelope = make_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.logical_time = 9999;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing logical_time must change event_hash"
    );
    Ok(())
}

#[test]
fn tamper_parent_id() -> Result<(), DefinitionError> {
    let mut envelope = make_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.parent_id = Some(DigestBytes::from_array([4u8; 32]));
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing parent_id must change event_hash"
    );
    Ok(())
}

#[test]
fn tamper_boundary_origin() -> Result<(), DefinitionError> {
    let mut envelope = make_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.boundary_origin = Some(BoundaryOrigin::Adapter);
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing boundary_origin must change event_hash"
    );
    Ok(())
}

#[test]
fn tamper_payload() -> Result<(), DefinitionError> {
    let mut envelope = make_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.payload = CanonicalPayload::new(serde_json::json!({"tampered": true}))?;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing payload must change event_hash"
    );
    Ok(())
}

// ── Layer A G2: blake3_untagged helper cross-copy equivalence ──────────
//
// The schemas copy of the sealed canonical-identity helper
// (`canonical_identity::digest_trusted_value`) must produce the committed
// golden digests. Companion tests in `vertrule-verifier` (SidecarDigest)
// and `vertrule-crypto` (CanonicalReceiptDigest) pin the same goldens, so
// all three copies are byte-equivalent. Source of truth:
// docs/audits/junk-drawer-inventory/fixtures/receipt-identity/goldens.json
#[test]
fn g2_blake3_untagged_helper_equivalence() -> Result<(), DefinitionError> {
    use crate::canonical_identity::digest_trusted_value;
    use vr_jcs::{to_canon_digest_with, DigestStrategy};

    let strategy = DigestStrategy::blake3_untagged();
    let cases: [(serde_json::Value, &str); 5] = [
        (
            serde_json::json!({"a": 1, "b": 2}),
            "8e80439b77ac62d4194499edd46684c479da3aa1ac80dd5511468efae049166e",
        ),
        (
            // unsorted keys must equal v_plain — proves JCS key ordering
            serde_json::json!({"b": 2, "a": 1}),
            "8e80439b77ac62d4194499edd46684c479da3aa1ac80dd5511468efae049166e",
        ),
        (
            serde_json::json!({"z": [3, 1, 2], "a": {"k": "v"}}),
            "5ef47de6cdb1c8586547526ee1fb7726321452f65ce50ba1abef1d3bf650a08c",
        ),
        (
            serde_json::json!({"n": 9_007_199_254_740_991_i64}),
            "6f3adc03614205e4ef7d378c51d584a691c60baa2abcdfea5325018261a28fb6",
        ),
        (
            serde_json::json!({"s": "café\n\"q\""}),
            "770f998755f9ac91974ea4dc2e23d34144f5cd0ad3238c3403a0a1e797c26a3a",
        ),
    ];

    for (value, expected) in &cases {
        let helper = digest_trusted_value(value, &strategy)?;
        assert_eq!(
            hex::encode(&helper.bytes),
            *expected,
            "schemas digest_trusted_value drifted from golden"
        );
        // The committed golden IS the live vr-jcs reference output.
        let reference = to_canon_digest_with(value, &strategy).map_err(DefinitionError::Jcs)?;
        assert_eq!(
            hex::encode(&reference.bytes),
            *expected,
            "vr-jcs reference drifted from golden"
        );
    }
    Ok(())
}

#[test]
fn changing_payload_changes_hash() -> Result<(), DefinitionError> {
    let e1 = make_envelope()?;

    let payload2 = CanonicalPayload::new(serde_json::json!({"v": 999}))?;
    let mut e2 = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Event,
        context_digest: zero_digest(),
        schema_digest: zero_digest(),
        policy_digest: zero_digest(),
        logical_time: 1,
        event_hash: zero_digest(),
        event_hash_profile: None,
        parent_id: None,
        boundary_origin: Some(BoundaryOrigin::Engine),
        digest_algorithm: None,
        canonicalization: None,
        payload: payload2,
    };
    e2.event_hash = compute_event_hash(&e2).map_err(DefinitionError::Jcs)?;

    assert_ne!(e1.event_hash, e2.event_hash);
    Ok(())
}
