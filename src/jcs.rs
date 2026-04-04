//! RFC 8785 JSON Canonicalization Scheme (JCS).
//!
//! Internal re-exports from `vr_jcs` for use within this crate.
//! External consumers should depend on `vr-jcs` directly.

#[allow(clippy::redundant_pub_crate)]
pub(crate) use vr_jcs::to_canon_bytes_from_slice;
#[allow(clippy::redundant_pub_crate)]
pub(crate) use vr_jcs::JcsError;

#[cfg(test)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) use vr_jcs::to_canon_string_from_str;

#[allow(clippy::redundant_pub_crate)]
pub(crate) use vr_jcs::deserialize_json_value_no_duplicates;
#[allow(clippy::redundant_pub_crate)]
pub(crate) use vr_jcs::is_safe_integer;
#[allow(clippy::redundant_pub_crate)]
pub(crate) use vr_jcs::validate_string_contents;

#[cfg(test)]
#[path = "jcs_tests.rs"]
mod jcs_tests;
