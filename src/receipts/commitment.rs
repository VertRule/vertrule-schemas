//! Envelope commitment computation.
//!
//! Computes `event_hash` for a [`ReceiptEnvelope`] according to the
//! commitment scope defined by its [`SchemaVersion`]:
//!
//! - **V1**: `BLAKE3(JCS(payload))` — payload only
//! - **V2**: `BLAKE3(JCS(envelope \ {event_hash}))` — all trust-bearing fields
//!
//! V2 ensures that mutating any field (`receipt_type`, `context_digest`,
//! `schema_digest`, `policy_digest`, `logical_time`, `parent_id`, `boundary_origin`,
//! or `payload`) without recomputing the commitment fails verification.

use serde_json::Value;

use crate::jcs::{to_canon_bytes, JcsError};
use crate::{DigestBytes, ReceiptEnvelope};

/// Compute the `event_hash` for an envelope according to its version's
/// commitment scope.
///
/// For V1 envelopes, this hashes only the payload.
/// For V2 envelopes, this hashes the entire envelope with `event_hash`
/// excluded from the commitment input.
///
/// # Errors
///
/// Returns [`JcsError`] if JCS canonicalization fails.
pub fn compute_event_hash(envelope: &ReceiptEnvelope) -> Result<DigestBytes, JcsError> {
    if envelope.envelope_version.commits_full_envelope() {
        compute_full_envelope_hash(envelope)
    } else {
        compute_payload_only_hash(envelope)
    }
}

/// V1: `BLAKE3(JCS(payload))`
fn compute_payload_only_hash(envelope: &ReceiptEnvelope) -> Result<DigestBytes, JcsError> {
    let canon_bytes = to_canon_bytes(envelope.payload.as_value())?;
    Ok(digest_bytes_from_blake3(&canon_bytes))
}

/// V2: `BLAKE3(JCS(envelope \ {event_hash}))`
///
/// Serializes the full envelope to a JSON object, removes the
/// `event_hash` field, then canonicalizes and hashes the remainder.
/// This commits every trust-bearing field in one operation.
fn compute_full_envelope_hash(envelope: &ReceiptEnvelope) -> Result<DigestBytes, JcsError> {
    let mut value = serde_json::to_value(envelope)?;
    if let Value::Object(ref mut map) = value {
        map.remove("event_hash");
    }
    let canon_bytes = to_canon_bytes(&value)?;
    Ok(digest_bytes_from_blake3(&canon_bytes))
}

fn digest_bytes_from_blake3(data: &[u8]) -> DigestBytes {
    DigestBytes::from_array(*blake3::hash(data).as_bytes())
}

#[cfg(test)]
#[path = "commitment_tests.rs"]
mod tests;
