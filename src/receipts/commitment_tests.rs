use super::compute_event_hash;
use crate::{
    BoundaryOrigin, CanonicalPayload, DefinitionError, DigestBytes, IJsonUInt, ReceiptEnvelope,
    ReceiptType, SchemaVersion,
};

fn zero_digest() -> DigestBytes {
    DigestBytes::from_array([0u8; 32])
}

fn make_envelope() -> Result<ReceiptEnvelope, DefinitionError> {
    let payload = CanonicalPayload::new(serde_json::json!({"v": 1}))
        .map_err(DefinitionError::InvalidPayload)?;

    let mut envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
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
    envelope.logical_time = IJsonUInt::new(9999)?;
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
    envelope.payload = CanonicalPayload::new(serde_json::json!({"tampered": true}))
        .map_err(DefinitionError::InvalidPayload)?;
    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    assert_ne!(
        original_hash, recomputed,
        "changing payload must change event_hash"
    );
    Ok(())
}

#[test]
fn changing_payload_changes_hash() -> Result<(), DefinitionError> {
    let e1 = make_envelope()?;

    let payload2 = CanonicalPayload::new(serde_json::json!({"v": 999}))
        .map_err(DefinitionError::InvalidPayload)?;
    let mut e2 = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Event,
        context_digest: zero_digest(),
        schema_digest: zero_digest(),
        policy_digest: zero_digest(),
        logical_time: IJsonUInt::new(1)?,
        event_hash: zero_digest(),
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
