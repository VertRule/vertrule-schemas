//! Envelope commitment computation.
//!
//! Computes `event_hash` for a [`ReceiptEnvelope`]:
//!
//! `BLAKE3(JCS(envelope \ {event_hash}))` — all trust-bearing fields
//! are committed. Mutating any field without recomputing `event_hash`
//! fails verification.

use serde_json::Value;

use crate::jcs::{to_canon_bytes, JcsError};
use crate::{DigestBytes, ReceiptEnvelope};

/// Compute the `event_hash` for an envelope.
///
/// Hashes the entire envelope with `event_hash` excluded from the
/// commitment input, committing every trust-bearing field.
///
/// # Errors
///
/// Returns [`JcsError`] if JCS canonicalization fails.
pub fn compute_event_hash(envelope: &ReceiptEnvelope) -> Result<DigestBytes, JcsError> {
    let mut value = serde_json::to_value(envelope)?;
    if let Value::Object(ref mut map) = value {
        map.remove("event_hash");
    }
    let canon_bytes = to_canon_bytes(&value)?;
    Ok(DigestBytes::from_array(*blake3::hash(&canon_bytes).as_bytes()))
}

#[cfg(test)]
#[path = "commitment_tests.rs"]
mod tests;
