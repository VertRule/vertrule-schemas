//! Deserializer for an optional slot that is present on the wire.
//!
//! ADR-056 R8 (ADR-055 §5.1, incorporated): absence is representable only
//! by key omission, and the verifier never recognises a byte pattern as
//! absence. serde's default `Option<T>` deserializer maps JSON `null` to
//! `None`, which would make `"slot": null` a second absence encoding that
//! re-serialises as omission and so recomputes to the same
//! `receipt_digest`. This helper deserialises the slot's value type
//! directly and wraps it in `Some`, so a present key must carry a value of
//! that type: `null` is a type error, never `None`.
//!
//! Pair it with `#[serde(default, skip_serializing_if = "Option::is_none")]`
//! so a missing key still deserialises as `None` and `None` still
//! serialises as omission.

use serde::{Deserialize, Deserializer};

/// Deserialize a present optional slot as `Some(T)`, rejecting `null`.
///
/// # Errors
///
/// Returns the value type's own deserialization error for any value that
/// is not a `T`, including JSON `null`.
pub(super) fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}
