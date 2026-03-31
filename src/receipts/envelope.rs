//! Constitutional public receipt envelope.
//!
//! Pure data type with no construction or validation methods.
//! Construction helpers live in producer crates; integrity validation
//! lives in `vertrule-verifier`.

use serde::{Deserialize, Serialize};

use crate::{BoundaryOrigin, CanonicalPayload, DigestBytes, IJsonUInt};
use crate::{ReceiptType, SchemaVersion};

/// Public receipt envelope shared by producers and verifiers.
///
/// Marked `#[non_exhaustive]` so that new optional fields can be added
/// in minor versions without breaking downstream struct construction.
/// Consumers should use `..Default`-style patterns or builder helpers
/// to remain forward-compatible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
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

    /// Commitment digest over the full envelope.
    ///
    /// Computed as `BLAKE3(JCS(envelope \ {event_hash}))`: the canonical
    /// JSON of every field except `event_hash` itself is hashed. Mutating
    /// any trust-bearing field without recomputing this digest fails
    /// verification.
    pub event_hash: DigestBytes,

    /// Previous envelope `event_hash`, when this envelope is chained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<DigestBytes>,

    /// Optional provenance discriminator for the producing boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary_origin: Option<BoundaryOrigin>,

    /// Optional explicit digest binding marker.
    ///
    /// This crate accepts any string value. Validation that the marker
    /// matches the `envelope_version` identity triple is a
    /// `vertrule-verifier` responsibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest_algorithm: Option<String>,

    /// Optional explicit canonicalization binding marker.
    ///
    /// This crate accepts any string value. Validation that the marker
    /// matches the `envelope_version` identity triple is a
    /// `vertrule-verifier` responsibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonicalization: Option<String>,

    /// Domain-specific payload content.
    ///
    /// Guarded against floating-point numbers at all nesting depths.
    /// Floats are nondeterministic across platforms and forbidden in
    /// the receipt spine.
    pub payload: CanonicalPayload,
}

#[cfg(test)]
#[path = "envelope_tests.rs"]
mod envelope_tests;
