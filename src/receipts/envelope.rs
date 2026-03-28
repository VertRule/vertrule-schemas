//! Constitutional public receipt envelope.
//!
//! Provides safe construction via [`ReceiptEnvelope::new`] and integrity
//! validation via [`ReceiptEnvelope::validate_integrity`]. Producers
//! should always use these instead of hand-rolling commitment computation.

use serde::{Deserialize, Serialize};

use crate::receipts::commitment::compute_event_hash;
use crate::{BoundaryOrigin, CanonicalPayload, DefinitionError, DigestBytes, IJsonUInt};
use crate::{ReceiptType, SchemaVersion};

/// Public receipt envelope shared by producers and verifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptEnvelope {
    /// Envelope schema version.
    pub envelope_version: SchemaVersion,

    /// High-level receipt discriminator.
    pub receipt_type: ReceiptType,

    /// Digest of the governance or execution context.
    pub context_digest: DigestBytes,

    /// Digest of the schema/profile used to interpret the payload.
    pub schema_digest: DigestBytes,

    /// Digest of the governing policy or policy set in force.
    pub policy_digest: DigestBytes,

    /// Monotonic logical clock value.
    pub logical_time: IJsonUInt,

    /// Digest of the canonical payload bytes.
    pub event_hash: DigestBytes,

    /// Previous envelope `event_hash`, when this envelope is chained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<DigestBytes>,

    /// Optional provenance discriminator for the producing boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary_origin: Option<BoundaryOrigin>,

    /// Optional explicit digest binding marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest_algorithm: Option<String>,

    /// Optional explicit canonicalization binding marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonicalization: Option<String>,

    /// Domain-specific payload content.
    ///
    /// Guarded against floating-point numbers at all nesting depths.
    /// Floats are nondeterministic across platforms and forbidden in
    /// the receipt spine.
    pub payload: CanonicalPayload,
}

impl ReceiptEnvelope {
    /// Construct a valid `ReceiptEnvelope` with correct `event_hash` and
    /// algorithm markers derived from the schema version.
    ///
    /// This is the safe constructor that producers should use instead of
    /// building the struct directly. It:
    /// - sets `digest_algorithm` and `canonicalization` from the version
    /// - computes `event_hash` over the appropriate commitment scope
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::Jcs`] if canonicalization fails.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: SchemaVersion,
        receipt_type: ReceiptType,
        context_digest: DigestBytes,
        schema_digest: DigestBytes,
        policy_digest: DigestBytes,
        logical_time: IJsonUInt,
        parent_id: Option<DigestBytes>,
        boundary_origin: Option<BoundaryOrigin>,
        payload: CanonicalPayload,
    ) -> Result<Self, DefinitionError> {
        let mut envelope = Self {
            envelope_version: version,
            receipt_type,
            context_digest,
            schema_digest,
            policy_digest,
            logical_time,
            event_hash: DigestBytes::from_array([0u8; 32]),
            parent_id,
            boundary_origin,
            digest_algorithm: Some(version.digest_algorithm().to_string()),
            canonicalization: Some(version.canonicalization().to_string()),
            payload,
        };
        envelope.event_hash = compute_event_hash(&envelope)?;
        Ok(envelope)
    }

    /// Validate the integrity of this envelope.
    ///
    /// Checks:
    /// 1. `event_hash` matches the recomputed commitment for this version
    /// 2. If `digest_algorithm` is present, it matches the version's binding
    /// 3. If `canonicalization` is present, it matches the version's binding
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::IntegrityViolation`] if `event_hash`
    /// does not match, or [`DefinitionError::MarkerMismatch`] if algorithm
    /// markers conflict with the schema version.
    pub fn validate_integrity(&self) -> Result<(), DefinitionError> {
        // Check algorithm markers against version bindings
        let expected_algo = self.envelope_version.digest_algorithm();
        let expected_canon = self.envelope_version.canonicalization();

        if let Some(ref declared) = self.digest_algorithm {
            if declared != expected_algo {
                return Err(DefinitionError::MarkerMismatch(format!(
                    "digest_algorithm \"{declared}\" does not match \
                     version {}'s binding \"{expected_algo}\"",
                    self.envelope_version,
                )));
            }
        }

        if let Some(ref declared) = self.canonicalization {
            if declared != expected_canon {
                return Err(DefinitionError::MarkerMismatch(format!(
                    "canonicalization \"{declared}\" does not match \
                     version {}'s binding \"{expected_canon}\"",
                    self.envelope_version,
                )));
            }
        }

        // Recompute and compare event_hash
        let recomputed = compute_event_hash(self)?;
        if recomputed != self.event_hash {
            return Err(DefinitionError::IntegrityViolation(format!(
                "event_hash {} does not match recomputed {}",
                self.event_hash, recomputed,
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "envelope_tests.rs"]
mod envelope_tests;
