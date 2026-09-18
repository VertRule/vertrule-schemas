//! Canonical V2 public receipt envelope (ADR-056 §4).
//!
//! Pure data type with no construction or validation methods. Producers
//! mint through `vr_receipt_identity::seal_receipt_v2` (the only mint
//! path); every V2 law is judged in `vertrule-verifier`.
//!
//! Compared with the V1 [`ReceiptEnvelope`](crate::ReceiptEnvelope) this
//! shape deliberately drops `event_hash`, `event_hash_profile`,
//! `boundary_origin`, `digest_algorithm` and `canonicalization`: the
//! version number commits the identity triple and the receipt type commits
//! provenance. V1 documents fail to deserialise here (`event_hash` is an
//! unknown field; `receipt_digest` is missing), and V2 documents fail to
//! deserialise as V1 symmetrically.

use serde::{Deserialize, Serialize};

use crate::{CanonicalPayload, DigestBytes, ReceiptTypeV2, SchemaVersion};

/// Canonical V2 public receipt envelope.
///
/// `#[non_exhaustive]` for construction only: producers go through
/// `seal_receipt_v2`, and the field set is frozen by ADR-056 (a new field
/// is a V3). `#[serde(deny_unknown_fields)]` keeps the wire shape closed.
///
/// Binding slots (`context_digest`, `policy_digest`, `parent_id`) are
/// `Option<DigestBytes>`: absence is key omission (ADR-056 R8) and no byte
/// value ever means "absent". A present key must carry a digest: JSON
/// `null` is a deserialization error, never `None`, so `null` cannot be a
/// second absence encoding that recomputes to the omitted form's
/// `receipt_digest`. Which cardinality applies to each slot is a registry
/// fact of the receipt type, never read from the receipt.
///
/// `envelope_version` is passive data here: a document carrying version 1
/// in this shape still deserialises (`SchemaVersion::new` accepts 1 and
/// 2). Requiring version 2 is the verifier's `UnsupportedVersion` DENY,
/// which runs before any digest recompute (ADR-056 §3.5, K3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ReceiptEnvelopeV2 {
    /// Envelope schema version; must be [`SchemaVersion::V2`].
    pub envelope_version: SchemaVersion,

    /// The complete semantic discriminator (ADR-056 R1–R2).
    pub receipt_type: ReceiptTypeV2,

    /// Identity of the schema governing `payload` — and nothing else
    /// (ADR-056 R6). Required for every type.
    pub schema_digest: DigestBytes,

    /// Context binding slot; subject and cardinality are registry facts of
    /// `receipt_type`. Omitted when absent; `null` is rejected.
    #[serde(
        default,
        deserialize_with = "crate::receipts::present_slot::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub context_digest: Option<DigestBytes>,

    /// Policy binding slot; subject and cardinality are registry facts of
    /// `receipt_type`. Omitted when absent; `null` is rejected.
    #[serde(
        default,
        deserialize_with = "crate::receipts::present_slot::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub policy_digest: Option<DigestBytes>,

    /// Monotonic logical clock value.
    ///
    /// Canonical wire form is a decimal string per
    /// `VR-CANONICAL-U64-STRING-POLICY-V1`. Deserialization accepts only
    /// that canonical string or a bare number (ADR-056 §4); a
    /// non-canonical string (`"01"`, `"+1"`) is rejected so no two
    /// byte-distinct strings name one `logical_time`.
    #[serde(
        serialize_with = "crate::receipts::canonical_u64::serialize",
        deserialize_with = "crate::receipts::canonical_u64::deserialize_strict"
    )]
    pub logical_time: u64,

    /// `receipt_digest` of the previous receipt of the **same type** in the
    /// same chain (ADR-056 §3.4). Omitted when absent (`null` is rejected);
    /// a reference to a receipt of another type is a payload relation,
    /// never `parent_id`.
    #[serde(
        default,
        deserialize_with = "crate::receipts::present_slot::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_id: Option<DigestBytes>,

    /// Domain-specific payload content, float-guarded at every depth.
    pub payload: CanonicalPayload,

    /// The receipt's one identity:
    /// `BLAKE3(D_RECEIPT_V2 ‖ JCS(envelope \ {receipt_digest}))`
    /// (ADR-056 R3–R4). Stored verbatim here; formed only by
    /// `vr-receipt-identity`.
    pub receipt_digest: DigestBytes,
}

#[cfg(test)]
#[path = "envelope_v2_tests.rs"]
mod envelope_v2_tests;
