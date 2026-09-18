//! Canonical decimal-string serde for digest-critical `u64` fields
//! (`VR-CANONICAL-U64-STRING-POLICY-V1`).
//!
//! Shared by the V1 [`ReceiptEnvelope`](crate::ReceiptEnvelope) and the V2
//! [`ReceiptEnvelopeV2`](crate::ReceiptEnvelopeV2) `logical_time` fields;
//! the serialized bytes are identical for both.
//!
//! Mirrors the byte representation of `vr_identity::canonical_u64_serde`,
//! kept local so this published crate stays free of runtime-internal
//! dependencies. Serializes as a decimal string; deserializes from either a
//! string or a bare number (backward-tolerant).
//!
//! Two deserializers share one serializer:
//!
//! - [`deserialize`] (V1, frozen) accepts any string `u64::from_str`
//!   accepts, including a leading `+` and leading zeros.
//! - [`deserialize_strict`] (V2) accepts only the canonical decimal string
//!   (the string must equal the value's own `to_string()`) or a bare
//!   number, so no two byte-distinct strings name one `logical_time`.

use serde::de::{self, Visitor};

// serde's `serialize_with` mandates the `&T` receiver shape.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) fn serialize<S: serde::Serializer>(
    value: &u64,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

/// V1 deserializer: canonical string, tolerant string, or bare number.
pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<u64, D::Error> {
    deserialize_with_visitor(deserializer, CanonicalU64Visitor { strict: false })
}

/// V2 deserializer: canonical decimal string or bare number only.
///
/// A string that parses but does not equal the value's canonical decimal
/// form (`"01"`, `"+1"`) is rejected, so a V2 document has exactly one
/// string encoding per `logical_time` value.
pub(super) fn deserialize_strict<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<u64, D::Error> {
    deserialize_with_visitor(deserializer, CanonicalU64Visitor { strict: true })
}

struct CanonicalU64Visitor {
    strict: bool,
}

impl Visitor<'_> for CanonicalU64Visitor {
    type Value = u64;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.strict {
            f.write_str("a u64 as a canonical decimal string or number")
        } else {
            f.write_str("a u64 as a decimal string or number")
        }
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
        Ok(v)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<u64, E> {
        u64::try_from(v).map_err(|_| E::custom("negative integers are not valid u64"))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
        let parsed = v.parse::<u64>().map_err(E::custom)?;
        if self.strict && parsed.to_string() != v {
            return Err(E::custom(format!(
                "non-canonical u64 string {v:?}: expected {parsed}"
            )));
        }
        Ok(parsed)
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<u64, E> {
        self.visit_str(&v)
    }
}

fn deserialize_with_visitor<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
    visitor: CanonicalU64Visitor,
) -> Result<u64, D::Error> {
    // Human-readable formats (JSON) accept both string and number via
    // `deserialize_any`; non-self-describing binary formats use
    // `deserialize_str`.
    if deserializer.is_human_readable() {
        deserializer.deserialize_any(visitor)
    } else {
        deserializer.deserialize_str(visitor)
    }
}
