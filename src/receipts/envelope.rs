//! Constitutional public receipt envelope.
//!
//! Pure data type with no construction or validation methods.
//! Construction helpers live in producer crates; integrity validation
//! lives in `vertrule-verifier`.

use serde::{Deserialize, Serialize};

use crate::{BoundaryOrigin, CanonicalPayload, DigestBytes};
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
    ///
    /// Canonical wire form is a decimal string per
    /// `VR-CANONICAL-U64-STRING-POLICY-V1` (JCS/I-JSON reject bare integers
    /// above `2^53 - 1`); deserialization also accepts a bare number for
    /// backward tolerance.
    #[serde(with = "canonical_u64")]
    pub logical_time: u64,

    /// Event-identity digest. Its preimage law is selected by the
    /// receipt's law profile (ADR-029 Receipt Law Profile Matrix), not by
    /// this field's name:
    ///
    /// - `constitutional_envelope_v1` (ADR-028R): self-commitment
    ///   `BLAKE3(JCS(envelope \ {event_hash}))` — every field except
    ///   `event_hash` is canonicalized and hashed.
    /// - `runtime_port_event_preimage_v1` (ADR-016 / DEC-3 Law 1):
    ///   `BLAKE3(JCS(RuntimePortEventHashPreimageV1))` over the bound input
    ///   set; verifiers commit to those inputs, not to the envelope's own
    ///   bytes.
    ///
    /// This crate stores the digest verbatim. Profile resolution and digest
    /// verification are `vertrule-verifier` responsibilities; a multi-law
    /// `receipt_type` must carry an explicit profile discriminator and is
    /// never resolved by inference (ADR-029 §3).
    pub event_hash: DigestBytes,

    /// Digest-law profile that produced `event_hash` (ADR-029 §3).
    ///
    /// Required on a multi-law `receipt_type` (e.g. `event`), where the same
    /// `event_hash` field can carry more than one law; omitted for single-law
    /// receipts. Absence is resolved from `receipt_type` alone and is never
    /// inferred for a multi-law type. This crate stores the marker verbatim;
    /// `vertrule-verifier` enforces presence and admissibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_hash_profile: Option<EventHashProfileId>,

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

/// Digest-law profile identifier for a public [`ReceiptEnvelope`]'s
/// `event_hash` (ADR-029 Receipt Law Profile Matrix).
///
/// Names the verifier-law profile so the same `event_hash` field can carry
/// different laws explicitly rather than by inference. The set is closed: a new
/// profile is a verifier-law delta requiring a successor ADR, so this enum is
/// deliberately **not** `#[non_exhaustive]`. Only the two profiles that may
/// lawfully appear on a public envelope's `event_hash` are admitted; internal
/// laws (`SEK` `receipt_digest`, `unsigned_receipt_digest`, the `MGS`
/// substrate digest, and payload-only quarantine) never serialize here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventHashProfileId {
    /// `constitutional_envelope_v1` (ADR-028R): envelope-minus self-commitment,
    /// `event_hash = BLAKE3(JCS(envelope \ {event_hash}))`.
    #[serde(rename = "constitutional_envelope_v1")]
    ConstitutionalEnvelopeV1,
    /// `runtime_port_event_preimage_v1` (ADR-016 / DEC-3 Law 1): `RuntimePort`
    /// typed preimage, `event_hash = BLAKE3(JCS(RuntimePortEventHashPreimageV1))`.
    #[serde(rename = "runtime_port_event_preimage_v1")]
    RuntimePortEventPreimageV1,
}

/// Canonical decimal-string serde for the digest-critical `logical_time`
/// field (`VR-CANONICAL-U64-STRING-POLICY-V1`).
///
/// Mirrors the byte representation of `vr_identity::canonical_u64_serde`,
/// kept local so this published crate stays free of runtime-internal
/// dependencies. Serializes as a decimal string; deserializes from either a
/// string (canonical) or a bare number (backward-tolerant).
mod canonical_u64 {
    use serde::de::{self, Visitor};

    // serde's `serialize_with` mandates the `&T` receiver shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(super) fn serialize<S: serde::Serializer>(
        value: &u64,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<u64, D::Error> {
        struct CanonicalU64Visitor;

        impl Visitor<'_> for CanonicalU64Visitor {
            type Value = u64;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a u64 as a decimal string or number")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
                Ok(v)
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<u64, E> {
                u64::try_from(v).map_err(|_| E::custom("negative integers are not valid u64"))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
                v.parse::<u64>().map_err(E::custom)
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<u64, E> {
                self.visit_str(&v)
            }
        }

        // Human-readable formats (JSON) accept both string and number via
        // `deserialize_any`; non-self-describing binary formats use
        // `deserialize_str`.
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(CanonicalU64Visitor)
        } else {
            deserializer.deserialize_str(CanonicalU64Visitor)
        }
    }
}

#[cfg(test)]
#[path = "envelope_tests.rs"]
mod envelope_tests;
