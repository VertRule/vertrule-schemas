//! Verified receipt metadata — a pure carrier for the fields extracted
//! from a structurally verified external receipt.
//!
//! Relocated from `vertrule-verifier::rbh` (ADR-038 Phase 1) so the carrier
//! sits below both the verifier (which produces it) and the policy substrate
//! (which consumes it), removing the substrate → verifier production
//! dependency. In production this value is still constructed only by
//! `vertrule_verifier::verify_external_receipt`; [`VerifiedReceiptMetadata::new`]
//! exists so that producer can build it across the crate boundary. The wire
//! form is the default field-name serialization (no renames) and is part of
//! the contract — preserve field names and order.

use serde::{Deserialize, Serialize};

/// Metadata extracted from a structurally verified external receipt.
///
/// A pure data carrier: fields are private and read through accessor methods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedReceiptMetadata {
    context_digest: String,
    policy_digest: String,
    schema_digest: String,
    event_hash: String,
    receipt_type: String,
    logical_time: u64,
    boundary_origin: Option<String>,
    payload: serde_json::Value,
}

impl VerifiedReceiptMetadata {
    /// Construct verified receipt metadata from its already-extracted fields.
    ///
    /// Production construction flows through
    /// `vertrule_verifier::verify_external_receipt`; this constructor gives
    /// that producer access now that the carrier lives in `vertrule-schemas`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        context_digest: String,
        policy_digest: String,
        schema_digest: String,
        event_hash: String,
        receipt_type: String,
        logical_time: u64,
        boundary_origin: Option<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            context_digest,
            policy_digest,
            schema_digest,
            event_hash,
            receipt_type,
            logical_time,
            boundary_origin,
            payload,
        }
    }

    /// BLAKE3 digest of the originating execution context.
    #[must_use]
    pub fn context_digest(&self) -> &str {
        &self.context_digest
    }

    /// BLAKE3 digest of the policy pack active at evidence time.
    #[must_use]
    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }

    /// BLAKE3 digest of the schema used.
    #[must_use]
    pub fn schema_digest(&self) -> &str {
        &self.schema_digest
    }

    /// BLAKE3 hash of the canonical payload.
    #[must_use]
    pub fn event_hash(&self) -> &str {
        &self.event_hash
    }

    /// Receipt type discriminator (e.g., "Governance").
    #[must_use]
    pub fn receipt_type(&self) -> &str {
        &self.receipt_type
    }

    /// Monotonic logical timestamp from the originating context.
    #[must_use]
    pub const fn logical_time(&self) -> u64 {
        self.logical_time
    }

    /// Boundary origin tag, if present.
    #[must_use]
    pub fn boundary_origin(&self) -> Option<&str> {
        self.boundary_origin.as_deref()
    }

    /// The receipt payload as structured JSON.
    #[must_use]
    pub const fn payload(&self) -> &serde_json::Value {
        &self.payload
    }
}

#[cfg(test)]
#[path = "verified_metadata_tests.rs"]
mod verified_metadata_tests;
