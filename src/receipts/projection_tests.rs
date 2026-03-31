use super::commitment::compute_event_hash;
use super::projection::ProjectsToReceiptEnvelope;
use crate::{
    BoundaryOrigin, CanonicalPayload, DefinitionError, DigestBytes, IJsonUInt, ReceiptEnvelope,
    ReceiptType, SchemaVersion,
};

/// Build a minimal valid envelope for testing.
fn test_envelope(payload_json: serde_json::Value) -> Result<ReceiptEnvelope, DefinitionError> {
    let payload = CanonicalPayload::new(payload_json).map_err(DefinitionError::InvalidPayload)?;
    let zero_digest = DigestBytes::from_array([0u8; 32]);
    let logical_time = IJsonUInt::new(1)?;

    let mut envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Event,
        context_digest: zero_digest,
        schema_digest: zero_digest,
        policy_digest: zero_digest,
        logical_time,
        event_hash: zero_digest, // placeholder
        parent_id: None,
        boundary_origin: Some(BoundaryOrigin::Engine),
        digest_algorithm: None,
        canonicalization: None,
        payload,
    };
    envelope.event_hash = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    Ok(envelope)
}

/// A trivial receipt type for testing the projection contract.
struct TestReceipt {
    value: u64,
}

impl ProjectsToReceiptEnvelope for TestReceipt {
    fn project(&self) -> Result<ReceiptEnvelope, DefinitionError> {
        let payload_json = serde_json::json!({ "value": self.value });
        test_envelope(payload_json)
    }
}

#[test]
fn projection_produces_valid_envelope() -> Result<(), DefinitionError> {
    let receipt = TestReceipt { value: 42 };
    let envelope = receipt.project()?;

    assert_eq!(envelope.envelope_version, SchemaVersion::V1);
    assert_eq!(envelope.receipt_type, ReceiptType::Event);
    Ok(())
}

#[test]
fn projected_event_hash_matches_recomputed() -> Result<(), DefinitionError> {
    let receipt = TestReceipt { value: 42 };
    let envelope = receipt.project()?;

    let recomputed = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;

    assert_eq!(
        envelope.event_hash, recomputed,
        "event_hash must equal recomputed commitment"
    );
    Ok(())
}

#[test]
fn projection_is_deterministic() -> Result<(), DefinitionError> {
    let receipt = TestReceipt { value: 99 };
    let e1 = receipt.project()?;
    let e2 = receipt.project()?;

    assert_eq!(
        e1.event_hash, e2.event_hash,
        "same input must produce same event_hash"
    );
    assert_eq!(
        crate::jcs::to_canon_bytes(&e1).map_err(DefinitionError::Jcs)?,
        crate::jcs::to_canon_bytes(&e2).map_err(DefinitionError::Jcs)?,
        "same input must produce identical canonical bytes"
    );
    Ok(())
}

#[test]
fn different_inputs_produce_different_hashes() -> Result<(), DefinitionError> {
    let e1 = TestReceipt { value: 1 }.project()?;
    let e2 = TestReceipt { value: 2 }.project()?;

    assert_ne!(
        e1.event_hash, e2.event_hash,
        "different inputs must produce different event_hash"
    );
    Ok(())
}

#[test]
fn projected_envelope_round_trips_through_json() -> Result<(), DefinitionError> {
    let receipt = TestReceipt { value: 42 };
    let envelope = receipt.project()?;

    let json = crate::jcs::to_canon_string(&envelope).map_err(DefinitionError::Jcs)?;
    let parsed: ReceiptEnvelope = serde_json::from_str(&json)
        .map_err(|e| DefinitionError::Jcs(crate::jcs::JcsError::from(e)))?;

    assert_eq!(envelope.event_hash, parsed.event_hash);
    assert_eq!(envelope.receipt_type, parsed.receipt_type);
    assert_eq!(envelope.payload, parsed.payload);
    Ok(())
}
