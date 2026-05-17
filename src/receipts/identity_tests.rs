//! Tests for the sealed `ReceiptDigest` newtype (JCS Consumer
//! Hardening Plan § Gate 2 scaffold for `vertrule-schemas`).

use super::commitment::compute_event_hash;
use super::identity::ReceiptDigest;
use crate::{
    BoundaryOrigin, CanonicalPayload, DefinitionError, DigestBytes, IJsonUInt,
    ReceiptEnvelope, ReceiptType, SchemaVersion,
};

fn minimal_envelope_with_zero_event_hash() -> Result<ReceiptEnvelope, DefinitionError> {
    let payload = CanonicalPayload::new(serde_json::json!({"value": 42}))?;
    let zero = DigestBytes::from_array([0u8; 32]);
    let logical_time = IJsonUInt::new(1)?;

    Ok(ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Event,
        context_digest: zero,
        schema_digest: zero,
        policy_digest: zero,
        logical_time,
        event_hash: zero,
        parent_id: None,
        boundary_origin: Some(BoundaryOrigin::Engine),
        digest_algorithm: None,
        canonicalization: None,
        payload,
    })
}

#[test]
fn receipt_digest_from_envelope_matches_legacy_compute_event_hash()
-> Result<(), DefinitionError> {
    // The legacy `compute_event_hash` and the sealed
    // `ReceiptDigest::from_envelope_commitment` must produce identical
    // bytes for the same envelope input. Proves the sealed wrapper is
    // a drop-in replacement for the legacy bypass.
    let envelope = minimal_envelope_with_zero_event_hash()?;

    let legacy = compute_event_hash(&envelope).map_err(DefinitionError::Jcs)?;
    let sealed = ReceiptDigest::from_envelope_commitment(&envelope)?;
    let sealed_bytes = sealed.as_digest_bytes()?;

    assert_eq!(
        legacy, sealed_bytes,
        "ReceiptDigest::from_envelope_commitment must equal compute_event_hash byte-for-byte",
    );
    Ok(())
}

#[test]
fn receipt_digest_algorithm_name_is_blake3_untagged() -> Result<(), DefinitionError> {
    let envelope = minimal_envelope_with_zero_event_hash()?;
    let sealed = ReceiptDigest::from_envelope_commitment(&envelope)?;
    assert_eq!(sealed.algorithm_name(), "blake3-untagged");
    Ok(())
}

#[test]
fn receipt_digest_bytes_length_is_32() -> Result<(), DefinitionError> {
    let envelope = minimal_envelope_with_zero_event_hash()?;
    let sealed = ReceiptDigest::from_envelope_commitment(&envelope)?;
    assert_eq!(sealed.bytes().len(), 32);
    Ok(())
}

#[test]
fn receipt_digest_into_canonical_digest_preserves_algorithm()
-> Result<(), DefinitionError> {
    let envelope = minimal_envelope_with_zero_event_hash()?;
    let sealed = ReceiptDigest::from_envelope_commitment(&envelope)?;
    let alg_name_before = sealed.algorithm_name();
    let canonical = sealed.into_canonical_digest();
    assert_eq!(canonical.algorithm.name(), alg_name_before);
    Ok(())
}
