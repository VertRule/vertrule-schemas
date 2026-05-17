//! Envelope commitment computation.
//!
//! Computes `event_hash` for a [`ReceiptEnvelope`]:
//!
//! `BLAKE3(JCS(envelope \ {event_hash}))` — all trust-bearing fields
//! are committed. Mutating any field without recomputing `event_hash`
//! fails verification.
//!
//! As of Gate 2 (JCS Consumer Hardening Plan), this function is a thin
//! adapter over [`crate::receipts::identity::ReceiptDigest`]. The
//! sealed `ReceiptDigest` is the canonical commitment path; this
//! function exists to preserve the legacy `JcsError`-typed return
//! signature for existing callers.

use crate::jcs::JcsError;
use crate::receipts::identity::ReceiptDigest;
use crate::{DefinitionError, DigestBytes, ReceiptEnvelope};

/// Compute the `event_hash` for an envelope.
///
/// Hashes the entire envelope with `event_hash` excluded from the
/// commitment input, committing every trust-bearing field. Delegates
/// to [`ReceiptDigest::from_envelope_commitment`] — both paths produce
/// byte-identical output (proved by `identity_tests`).
///
/// # Errors
///
/// Returns [`JcsError`] if JCS canonicalization fails or the envelope
/// did not serialize to a JSON object.
pub fn compute_event_hash(envelope: &ReceiptEnvelope) -> Result<DigestBytes, JcsError> {
    let digest = ReceiptDigest::from_envelope_commitment(envelope).map_err(|e| match e {
        DefinitionError::Jcs(jcs_err) => jcs_err,
        DefinitionError::InvalidPayload(msg) => JcsError::InvalidString(msg),
        other => JcsError::InvalidString(other.to_string()),
    })?;
    digest.as_digest_bytes().map_err(|e| match e {
        DefinitionError::Jcs(jcs_err) => jcs_err,
        other => JcsError::InvalidString(other.to_string()),
    })
}

#[cfg(test)]
#[path = "commitment_tests.rs"]
mod tests;
