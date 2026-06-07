//! Full-envelope `event_hash` commitment law.
//!
//! `event_hash = BLAKE3(JCS(envelope \ {event_hash}))` — every
//! trust-bearing field is committed; the `event_hash` field itself is
//! excluded from its own preimage. This is the `constitutional_envelope_v1`
//! law (ADR-028R).
//!
//! This is an independent reimplementation of the law currently in
//! `vertrule-schemas::receipts::compute_event_hash`; the two are pinned
//! byte-for-byte by the Layer A golden vectors and the parity test below.

use serde_json::Value;
use vr_jcs::DigestStrategy;
use vertrule_schemas::{DigestBytes, ReceiptEnvelope};

use crate::canonical_identity::digest_trusted_value;
use crate::error::ReceiptIdentityError;

/// Compute the `event_hash` for a [`ReceiptEnvelope`] under the
/// `constitutional_envelope_v1` law.
///
/// Serializes the envelope, removes the `event_hash` field, and digests
/// the canonical remainder with [`DigestStrategy::blake3_untagged`]. The
/// strategy is fixed by the v1 envelope-commitment specification; a
/// future schema version that needs a different strategy MUST add a
/// sibling constructor rather than overload this one.
///
/// # Errors
///
/// Returns [`ReceiptIdentityError::InvalidPayload`] if the envelope does
/// not serialize to a JSON object, [`ReceiptIdentityError::Jcs`] for any
/// canonicalization or digest failure, and
/// [`ReceiptIdentityError::InvalidDigest`] if the digest is not the wire
/// length.
pub fn compute_event_hash(
    envelope: &ReceiptEnvelope,
) -> Result<DigestBytes, ReceiptIdentityError> {
    let mut value = serde_json::to_value(envelope).map_err(vr_jcs::JcsError::from)?;
    let Value::Object(ref mut map) = value else {
        return Err(ReceiptIdentityError::InvalidPayload(
            "envelope did not serialize to a JSON object".to_string(),
        ));
    };
    map.remove("event_hash");

    let digest = digest_trusted_value(&value, &DigestStrategy::blake3_untagged())?;
    DigestBytes::from_slice(&digest.bytes)
        .map_err(|e| ReceiptIdentityError::InvalidDigest(e.to_string()))
}

#[cfg(test)]
#[path = "commitment_tests.rs"]
mod tests;
