//! Constitutional public receipt envelope.
//!
//! Pure data type with no construction or validation methods.
//! Construction helpers live in producer crates; integrity validation
//! lives in `vertrule-verifier`.

use serde::{Deserialize, Serialize};

use crate::{BoundaryOrigin, CanonicalPayload, DigestBytes, IJsonUInt};
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

#[cfg(test)]
#[path = "envelope_tests.rs"]
mod envelope_tests;
