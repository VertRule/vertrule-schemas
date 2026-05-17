//! Sealed receipt-identity newtype.
//!
//! [`ReceiptDigest`] is the domain-named digest type for receipt
//! commitments emitted by `vertrule-schemas`. Construction is only
//! through approved constructors — the inner `CanonicalDigest` field
//! is private, and there is **no** `From<[u8; 32]>` or other raw-bytes
//! constructor. That preserves the algorithm-with-output binding from
//! ADR-002 and prevents downstream code from confusing "some 32 bytes"
//! with "a receipt commitment digest".
//!
//! Per the JCS Consumer Hardening Plan § Gate 2:
//!
//! - Consumers own domain-to-canonical-input conversion.
//! - `vr-jcs` owns canonicalization and digest computation.
//! - Nobody else owns both.
//!
//! `ReceiptDigest` is the schemas-owned domain wrapper. All plumbing
//! routes through `crate::canonical_identity` to keep digest-strategy
//! choice typed and centralized.

use serde_json::Value;
use vr_jcs::{CanonicalDigest, DigestAlgorithm, DigestStrategy};

use crate::canonical_identity::digest_trusted_value;
use crate::{DefinitionError, DigestBytes, ReceiptEnvelope};

/// A receipt-bearing canonical digest.
///
/// Wraps [`CanonicalDigest`] to make "this is a receipt-event-hash
/// digest" a type-level fact. The inner field is private; construction
/// is only through the domain constructors below.
///
/// `ReceiptDigest` carries the producing [`DigestAlgorithm`] verbatim,
/// so receipt envelopes can record the algorithm by name (ADR-002
/// algorithm-output binding).
#[derive(Debug, Clone)]
pub struct ReceiptDigest {
    inner: CanonicalDigest,
}

impl ReceiptDigest {
    /// Compute the event-hash digest of a `ReceiptEnvelope`.
    ///
    /// Implements the envelope-commitment rule:
    /// `digest(canonicalize(envelope \ {event_hash}))`. The
    /// `event_hash` field itself is excluded from the commitment input
    /// so the receipt can carry the digest of its own remainder.
    ///
    /// Strategy is [`DigestStrategy::blake3_untagged`] today, matching
    /// the v1 envelope commitment specification. A future schema
    /// version that wants a different strategy MUST add a sibling
    /// constructor rather than overloading this one.
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidPayload`] if the envelope
    /// fails to serialize as a JSON object. Returns
    /// [`DefinitionError::Jcs`] for any canonicalization or digest
    /// failure.
    pub fn from_envelope_commitment(
        envelope: &ReceiptEnvelope,
    ) -> Result<Self, DefinitionError> {
        let mut value = serde_json::to_value(envelope).map_err(crate::jcs::JcsError::from)?;
        let Value::Object(ref mut map) = value else {
            return Err(DefinitionError::InvalidPayload(
                "envelope did not serialize to a JSON object".to_string(),
            ));
        };
        map.remove("event_hash");

        let inner = digest_trusted_value(&value, &DigestStrategy::blake3_untagged())?;
        Ok(Self { inner })
    }

    /// Stable algorithm-name identifier for receipt-schema use
    /// (`"blake3-untagged"`, `"blake3-keyed"`, …). Pinned by ADR-002.
    #[must_use]
    pub const fn algorithm_name(&self) -> &'static str {
        self.inner.algorithm.name()
    }

    /// Borrow the underlying [`DigestAlgorithm`] for envelope metadata
    /// recording.
    #[must_use]
    pub const fn algorithm(&self) -> &DigestAlgorithm {
        &self.inner.algorithm
    }

    /// Borrow the raw digest bytes. Length is algorithm-dependent
    /// (32 bytes for all wired BLAKE3 modes today).
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.inner.bytes
    }

    /// Project to the wire-format [`DigestBytes`] shape used by
    /// receipt envelopes for serialization.
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidDigest`] if the digest is not
    /// exactly 32 bytes. All wired BLAKE3 modes produce 32 bytes, so
    /// this only fires if a future algorithm with a different output
    /// length is plumbed without an envelope-shape update.
    pub fn as_digest_bytes(&self) -> Result<DigestBytes, DefinitionError> {
        DigestBytes::from_slice(&self.inner.bytes)
    }

    /// Consume the wrapper and return the underlying
    /// [`CanonicalDigest`]. Use at wire-format / envelope-construction
    /// boundaries where the algorithm-bearing form is required.
    #[must_use]
    pub fn into_canonical_digest(self) -> CanonicalDigest {
        self.inner
    }
}

