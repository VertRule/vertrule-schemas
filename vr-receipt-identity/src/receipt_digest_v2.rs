//! The one V2 `receipt_digest` formation law (ADR-056 §3.2).
//!
//! ```text
//! ReceiptDigestV2(R)  =  BLAKE3( D_RECEIPT_V2 ‖ JCS( R \ {receipt_digest} ) )
//! D_RECEIPT_V2        =  UTF-8 "vertrule.receipt.v2" ‖ 0x00        (20 bytes)
//! ```
//!
//! Declared once, here, through `vr_identity::declare_digest_domain!` with
//! the `TaggedJcsCanonicalJson` formation: domain id `receipt.envelope-v2`,
//! tag [`RECEIPT_V2_DOMAIN_TAG`]. The preimage is the **serialised** V2
//! envelope minus its `receipt_digest` key: a present optional slot
//! contributes its key and value; an absent one contributes nothing
//! (R8). The NUL terminator makes the tag boundary unambiguous because JCS
//! output never contains `0x00`.
//!
//! There is no `event_hash`, no profile, and no second constructor in V2.
//! [`seal_receipt_v2`] is the only mint path; [`compute_receipt_digest_v2`]
//! is the verifier's recompute path. Both form byte-identical preimages.
//! Neither judges a law: cardinality, schema and relation laws live in
//! `vertrule-verifier` (K3).

use serde_json::{Map, Value};
use vertrule_schemas::{
    CanonicalPayload, DigestBytes, ReceiptEnvelopeV2, ReceiptTypeV2, SchemaVersion,
};
use vr_identity::digest::{DigestOf, TaggedJcsCanonicalJson};

use crate::error::ReceiptIdentityError;

/// `D_RECEIPT_V2`: the frozen V2 domain-separation tag,
/// `"vertrule.receipt.v2"` followed by one NUL byte (20 bytes, hex
/// `76657274 72756c65 2e726563 65697074 2e763200`).
///
/// New under ADR-056; it inherits no existing string.
pub const RECEIPT_V2_DOMAIN_TAG: &[u8] = b"vertrule.receipt.v2\0";

vr_identity::declare_digest_domain! {
    /// Identity domain of a canonical V2 receipt envelope (ADR-056 R3).
    ///
    /// Formation: `BLAKE3(RECEIPT_V2_DOMAIN_TAG ‖ JCS(envelope \ {receipt_digest}))`.
    pub ReceiptDigestV2Identity {
        id: "receipt.envelope-v2",
        formation: TaggedJcsCanonicalJson,
        tag: RECEIPT_V2_DOMAIN_TAG,
    }
}

/// Producer facts for one V2 receipt, before sealing.
///
/// Plain data: producers provide facts, the verifier-owned registry
/// provides laws (R9). No slot carries a cardinality, subject or mode;
/// absence is `None` and serialises as key omission (R8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptV2Draft {
    /// The complete semantic discriminator.
    pub receipt_type: ReceiptTypeV2,
    /// Identity of the schema governing `payload` (R6).
    pub schema_digest: DigestBytes,
    /// Context binding slot, when present.
    pub context_digest: Option<DigestBytes>,
    /// Policy binding slot, when present.
    pub policy_digest: Option<DigestBytes>,
    /// Monotonic logical clock value.
    pub logical_time: u64,
    /// `receipt_digest` of the previous same-type receipt, when chained.
    pub parent_id: Option<DigestBytes>,
    /// Domain-specific payload content.
    pub payload: CanonicalPayload,
}

/// Derive the V2 receipt identity over an already-formed preimage object
/// (the envelope without its `receipt_digest` key).
fn digest_preimage(preimage: &Value) -> Result<DigestBytes, ReceiptIdentityError> {
    let digest = DigestOf::<ReceiptDigestV2Identity>::derive(preimage)?;
    Ok(DigestBytes::from_array(digest.as_32_byte_array()?))
}

/// Compute `receipt_digest` for a [`ReceiptEnvelopeV2`] (ADR-056 §3.2).
///
/// Serialises the envelope, removes the `receipt_digest` key, and digests
/// the remainder under [`ReceiptDigestV2Identity`]. The stored
/// `receipt_digest` value never enters its own preimage, so this is the
/// verifier's recompute path.
///
/// This function judges no law (K3). In particular it does **not** judge
/// `envelope_version`: an envelope carrying version 1 in the V2 shape is
/// digested as-is under the V2 domain. The verifier's `UnsupportedVersion`
/// DENY must run before any recompute (ADR-056 §3.5).
///
/// # Errors
///
/// Returns [`ReceiptIdentityError::InvalidPayload`] if the envelope does
/// not serialize to a JSON object, [`ReceiptIdentityError::Jcs`] if
/// serialization fails, and [`ReceiptIdentityError::Identity`] for any
/// formation failure.
pub fn compute_receipt_digest_v2(
    envelope: &ReceiptEnvelopeV2,
) -> Result<DigestBytes, ReceiptIdentityError> {
    let mut value = serde_json::to_value(envelope).map_err(vr_jcs::JcsError::from)?;
    let Value::Object(ref mut map) = value else {
        return Err(ReceiptIdentityError::InvalidPayload(
            "envelope did not serialize to a JSON object".to_string(),
        ));
    };
    map.remove("receipt_digest");
    digest_preimage(&value)
}

/// Seal a [`ReceiptV2Draft`] into a [`ReceiptEnvelopeV2`] — the only V2
/// mint path (ADR-056 §4).
///
/// Builds the wire object `{envelope_version: 2, receipt_type,
/// schema_digest, context_digest?, policy_digest?, logical_time: "<dec>",
/// parent_id?, payload}` with absent slots omitted, digests it under
/// [`ReceiptDigestV2Identity`] **without** any `receipt_digest` key
/// present, then inserts the digest and deserialises the closed shape.
/// No placeholder digest is ever written into the object before
/// commitment.
///
/// # Errors
///
/// Returns [`ReceiptIdentityError::Jcs`] if a fact fails to serialize,
/// [`ReceiptIdentityError::Identity`] if formation fails, and
/// [`ReceiptIdentityError::InvalidPayload`] if the sealed object does not
/// deserialize as a [`ReceiptEnvelopeV2`].
pub fn seal_receipt_v2(draft: ReceiptV2Draft) -> Result<ReceiptEnvelopeV2, ReceiptIdentityError> {
    let mut map = Map::new();
    map.insert(
        "envelope_version".to_string(),
        serde_json::to_value(SchemaVersion::V2).map_err(vr_jcs::JcsError::from)?,
    );
    map.insert(
        "receipt_type".to_string(),
        serde_json::to_value(draft.receipt_type).map_err(vr_jcs::JcsError::from)?,
    );
    map.insert(
        "schema_digest".to_string(),
        Value::String(draft.schema_digest.to_hex()),
    );
    if let Some(context_digest) = draft.context_digest {
        map.insert(
            "context_digest".to_string(),
            Value::String(context_digest.to_hex()),
        );
    }
    if let Some(policy_digest) = draft.policy_digest {
        map.insert(
            "policy_digest".to_string(),
            Value::String(policy_digest.to_hex()),
        );
    }
    map.insert(
        "logical_time".to_string(),
        Value::String(draft.logical_time.to_string()),
    );
    if let Some(parent_id) = draft.parent_id {
        map.insert("parent_id".to_string(), Value::String(parent_id.to_hex()));
    }
    map.insert("payload".to_string(), draft.payload.into_value());

    let preimage = Value::Object(map);
    let receipt_digest = digest_preimage(&preimage)?;

    let Value::Object(mut map) = preimage else {
        return Err(ReceiptIdentityError::InvalidPayload(
            "V2 preimage is not a JSON object".to_string(),
        ));
    };
    map.insert(
        "receipt_digest".to_string(),
        Value::String(receipt_digest.to_hex()),
    );
    serde_json::from_value(Value::Object(map))
        .map_err(|e| ReceiptIdentityError::InvalidPayload(e.to_string()))
}

#[cfg(test)]
#[path = "receipt_digest_v2_tests.rs"]
mod tests;
