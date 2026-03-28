//! RFC 8785 JSON Canonicalization Scheme (JCS).
//!
//! This module re-exports the canonical implementation from `vr_jcs`.
//! The `vr-jcs` crate is the single authoritative source for JCS in the
//! `VertRule` ecosystem.

pub use vr_jcs::to_canon_bytes;
pub use vr_jcs::to_canon_bytes_from_slice;
pub use vr_jcs::to_canon_string;
pub use vr_jcs::to_canon_string_from_str;
pub use vr_jcs::JcsError;

pub(crate) use vr_jcs::deserialize_json_value_no_duplicates;
pub(crate) use vr_jcs::is_safe_integer;
pub(crate) use vr_jcs::validate_string_contents;
