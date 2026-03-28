use super::compute_event_hash;
use crate::{
    BoundaryOrigin, CanonicalPayload, DefinitionError, DigestBytes, IJsonUInt, ReceiptEnvelope,
    ReceiptType, SchemaVersion,
};

fn zero_digest() -> DigestBytes {
    DigestBytes::from_array([0u8; 32])
}

fn make_v1_envelope() -> Result<ReceiptEnvelope, DefinitionError> {
    let payload = CanonicalPayload::new(serde_json::json!({"v": 1}))
        .map_err(DefinitionError::InvalidPayload)?;
    let event_hash = compute_event_hash_for(&payload, SchemaVersion::V1)?;
    Ok(ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Event,
        context_digest: zero_digest(),
        schema_digest: zero_digest(),
        policy_digest: zero_digest(),
        logical_time: IJsonUInt::new(1)?,
        event_hash,
        parent_id: None,
        boundary_origin: Some(BoundaryOrigin::Engine),
        digest_algorithm: None,
        canonicalization: None,
        payload,
    })
}

fn make_v2_envelope() -> Result<ReceiptEnvelope, DefinitionError> {
    let payload = CanonicalPayload::new(serde_json::json!({"v": 2}))
        .map_err(DefinitionError::InvalidPayload)?;

    // Build envelope with placeholder event_hash, then compute real one
    let mut envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V2,
        receipt_type: ReceiptType::Event,
        context_digest: zero_digest(),
        schema_digest: zero_digest(),
        policy_digest: zero_digest(),
        logical_time: IJsonUInt::new(1)?,
        event_hash: zero_digest(), // placeholder
        parent_id: None,
        boundary_origin: Some(BoundaryOrigin::Engine),
        digest_algorithm: None,
        canonicalization: None,
        payload,
    };
    envelope.event_hash = compute_event_hash(&envelope)
        .map_err(DefinitionError::Jcs)?;
    Ok(envelope)
}

/// Helper: compute `event_hash` for a payload under a given version.
fn compute_event_hash_for(
    payload: &CanonicalPayload,
    _version: SchemaVersion,
) -> Result<DigestBytes, DefinitionError> {
    let canon_bytes = crate::jcs::to_canon_bytes(payload.as_value())
        .map_err(DefinitionError::Jcs)?;
    Ok(DigestBytes::from_array(*blake3::hash(&canon_bytes).as_bytes()))
}

// ── V1: payload-only commitment ────────────────────────────────────

#[test]
fn v1_event_hash_matches_payload_only() -> Result<(), DefinitionError> {
    let envelope = make_v1_envelope()?;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_eq!(envelope.event_hash, recomputed);
    Ok(())
}

#[test]
fn v1_changing_payload_changes_hash() -> Result<(), DefinitionError> {
    let e1 = make_v1_envelope()?;
    let payload2 = CanonicalPayload::new(serde_json::json!({"v": 999}))
        .map_err(DefinitionError::InvalidPayload)?;
    let e2_hash = compute_event_hash_for(&payload2, SchemaVersion::V1)?;
    assert_ne!(e1.event_hash, e2_hash);
    Ok(())
}

// ── V2: full-envelope commitment ───────────────────────────────────

#[test]
fn v2_event_hash_matches_full_envelope() -> Result<(), DefinitionError> {
    let envelope = make_v2_envelope()?;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_eq!(envelope.event_hash, recomputed);
    Ok(())
}

#[test]
fn v2_deterministic() -> Result<(), DefinitionError> {
    let e1 = make_v2_envelope()?;
    let e2 = make_v2_envelope()?;
    assert_eq!(e1.event_hash, e2.event_hash);
    Ok(())
}

// ── V2 tamper detection: every trust-bearing field ─────────────────

#[test]
fn v2_tamper_receipt_type() -> Result<(), DefinitionError> {
    let mut envelope = make_v2_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.receipt_type = ReceiptType::Governance;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(original_hash, recomputed, "changing receipt_type must change event_hash");
    Ok(())
}

#[test]
fn v2_tamper_context_digest() -> Result<(), DefinitionError> {
    let mut envelope = make_v2_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.context_digest = DigestBytes::from_array([1u8; 32]);
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(original_hash, recomputed, "changing context_digest must change event_hash");
    Ok(())
}

#[test]
fn v2_tamper_schema_digest() -> Result<(), DefinitionError> {
    let mut envelope = make_v2_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.schema_digest = DigestBytes::from_array([2u8; 32]);
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(original_hash, recomputed, "changing schema_digest must change event_hash");
    Ok(())
}

#[test]
fn v2_tamper_policy_digest() -> Result<(), DefinitionError> {
    let mut envelope = make_v2_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.policy_digest = DigestBytes::from_array([3u8; 32]);
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(original_hash, recomputed, "changing policy_digest must change event_hash");
    Ok(())
}

#[test]
fn v2_tamper_logical_time() -> Result<(), DefinitionError> {
    let mut envelope = make_v2_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.logical_time = IJsonUInt::new(9999)?;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(original_hash, recomputed, "changing logical_time must change event_hash");
    Ok(())
}

#[test]
fn v2_tamper_parent_id() -> Result<(), DefinitionError> {
    let mut envelope = make_v2_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.parent_id = Some(DigestBytes::from_array([4u8; 32]));
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(original_hash, recomputed, "changing parent_id must change event_hash");
    Ok(())
}

#[test]
fn v2_tamper_boundary_origin() -> Result<(), DefinitionError> {
    let mut envelope = make_v2_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.boundary_origin = Some(BoundaryOrigin::Adapter);
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(original_hash, recomputed, "changing boundary_origin must change event_hash");
    Ok(())
}

#[test]
fn v2_tamper_payload() -> Result<(), DefinitionError> {
    let mut envelope = make_v2_envelope()?;
    let original_hash = envelope.event_hash;
    envelope.payload = CanonicalPayload::new(serde_json::json!({"tampered": true}))
        .map_err(DefinitionError::InvalidPayload)?;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(original_hash, recomputed, "changing payload must change event_hash");
    Ok(())
}

// ── V1 vs V2 produce different hashes for same content ─────────────

#[test]
fn v1_and_v2_hashes_differ_for_same_payload() -> Result<(), DefinitionError> {
    let payload = CanonicalPayload::new(serde_json::json!({"same": true}))
        .map_err(DefinitionError::InvalidPayload)?;

    let v1 = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Event,
        context_digest: zero_digest(),
        schema_digest: zero_digest(),
        policy_digest: zero_digest(),
        logical_time: IJsonUInt::new(1)?,
        event_hash: zero_digest(), // placeholder
        parent_id: None,
        boundary_origin: None,
        digest_algorithm: None,
        canonicalization: None,
        payload: payload.clone(),
    };

    let mut v2 = ReceiptEnvelope {
        envelope_version: SchemaVersion::V2,
        receipt_type: ReceiptType::Event,
        context_digest: zero_digest(),
        schema_digest: zero_digest(),
        policy_digest: zero_digest(),
        logical_time: IJsonUInt::new(1)?,
        event_hash: zero_digest(), // placeholder
        parent_id: None,
        boundary_origin: None,
        digest_algorithm: None,
        canonicalization: None,
        payload,
    };

    let h1 = compute_event_hash(&v1).map_err(DefinitionError::Jcs)?;
    v2.event_hash = zero_digest(); // ensure placeholder for computation
    let h2 = compute_event_hash(&v2).map_err(DefinitionError::Jcs)?;

    assert_ne!(h1, h2, "V1 and V2 must produce different hashes (different commitment scopes)");
    Ok(())
}
