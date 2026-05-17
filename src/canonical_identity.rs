//! Sealed canonical-identity plumbing for `vertrule-schemas`.
//!
//! This module is **`pub(crate)` only**. It provides the two narrow
//! conversion shapes that every domain-specific digest constructor in
//! this crate must route through:
//!
//! - [`digest_trusted_value`] — for `serde_json::Value` instances that
//!   come from caller-controlled construction inside this crate (typed
//!   Rust structs serialized via `serde_json::to_value`). Duplicate-key
//!   ambiguity cannot exist on this path because the source is a typed
//!   struct, not raw bytes.
//! - [`digest_untrusted_json`] — for raw JSON bytes that arrived from
//!   outside the crate. Routes through `vr_jcs::strict_parse` before
//!   digesting so duplicate keys, I-JSON violations, and depth-bound
//!   breaches are rejected before any digest computation.
//!
//! Both helpers always go through `vr_jcs::to_canon_digest_with`, which
//! returns a `CanonicalDigest` carrying the algorithm-with-output
//! binding (ADR-002 Decision item 3). Domain newtypes (e.g.
//! `ReceiptDigest`) wrap that `CanonicalDigest` and project to wire
//! formats only at boundary methods.
//!
//! # Sealed-helper invariant (JCS Consumer Hardening Plan § Gate 2)
//!
//! - This crate MUST NOT expose generic `hash_json`-style helpers.
//! - This crate MUST NOT call `blake3::*` primitives directly outside
//!   `vr_jcs`-routed paths once Gate 2 is complete in callers.
//! - The two helpers below are `pub(crate)`. The public API of this
//!   crate exposes domain-specific digest newtypes only.

use vr_jcs::{to_canon_digest_with, CanonicalDigest, DigestStrategy};

use crate::DefinitionError;

/// Digest a `serde_json::Value` produced from a typed Rust value inside
/// this crate.
///
/// Use only when the caller controls construction of the input value
/// (e.g. `serde_json::to_value(&typed_struct)?`). The strict-admission
/// parse is skipped because typed structs cannot carry duplicate keys
/// or other raw-JSON ambiguities.
///
/// # Errors
///
/// Returns [`DefinitionError::Jcs`] if canonicalization or digest
/// computation fails (depth overflow, I-JSON validation, non-finite
/// number, etc.).
#[allow(
    clippy::redundant_pub_crate,
    reason = "explicit pub(crate) documents the Gate 2 sealed-helper visibility intent; \
              the module is already pub(crate) but this keeps the contract visible at call sites"
)]
pub(crate) fn digest_trusted_value(
    value: &serde_json::Value,
    strategy: &DigestStrategy,
) -> Result<CanonicalDigest, DefinitionError> {
    Ok(to_canon_digest_with(value, strategy)?)
}

/// Digest raw JSON bytes that came from outside this crate.
///
/// Routes through `vr_jcs::strict_parse::parse_json_value_no_duplicates`
/// first so duplicate keys, I-JSON noncharacters, non-exact numbers,
/// and depth-bound breaches are rejected before any digest computation.
///
/// # Errors
///
/// Returns [`DefinitionError::Jcs`] for any strict-admission or digest
/// failure.
#[allow(
    dead_code,
    clippy::redundant_pub_crate,
    reason = "Plumbing scaffold for Gate 2; first call sites land as bypass migrations complete"
)]
pub(crate) fn digest_untrusted_json(
    json: &[u8],
    strategy: &DigestStrategy,
) -> Result<CanonicalDigest, DefinitionError> {
    let value = vr_jcs::strict_parse::parse_json_value_no_duplicates(json)?;
    Ok(to_canon_digest_with(&value, strategy)?)
}
