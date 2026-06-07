//! Sealed canonical-identity plumbing for `vr-receipt-identity`.
//!
//! Two narrow conversion shapes that every digest constructor in this
//! crate routes through. Both go through `vr_jcs::to_canon_digest_with`,
//! which binds the algorithm to its output (ADR-002).
//!
//! - [`digest_trusted_value`] — for `serde_json::Value` produced from a
//!   typed Rust value inside this crate (no duplicate-key ambiguity).
//! - [`digest_untrusted_json`] — for raw JSON bytes from outside; routes
//!   through strict admission first.
//!
//! This crate MUST NOT expose generic `hash_json`-style helpers or call
//! `blake3::*` directly: identity digests route through `vr-jcs` only.

use vr_jcs::{to_canon_digest_with, CanonicalDigest, DigestStrategy};

use crate::error::ReceiptIdentityError;

/// Digest a `serde_json::Value` produced from a typed value inside this
/// crate (caller controls construction; strict admission is skipped).
///
/// # Errors
///
/// Returns [`ReceiptIdentityError::Jcs`] if canonicalization or digest
/// computation fails.
#[allow(
    clippy::redundant_pub_crate,
    reason = "sealed-helper visibility intent; `unreachable_pub` (deny) requires \
              pub(crate) here, which nursery then flags as redundant"
)]
pub(crate) fn digest_trusted_value(
    value: &serde_json::Value,
    strategy: &DigestStrategy,
) -> Result<CanonicalDigest, ReceiptIdentityError> {
    Ok(to_canon_digest_with(value, strategy)?)
}

/// Digest raw JSON bytes that arrived from outside this crate.
///
/// Routes through `vr_jcs::strict_parse::parse_json_value_no_duplicates`
/// first so duplicate keys, I-JSON violations, non-exact numbers, and
/// depth-bound breaches are rejected before any digest computation.
///
/// # Errors
///
/// Returns [`ReceiptIdentityError::Jcs`] for any strict-admission or
/// digest failure.
#[allow(
    dead_code,
    clippy::redundant_pub_crate,
    reason = "Stage 1 plumbing: the untrusted path lands its first call site \
              when external-bytes consumers are repointed in a later stage; \
              `unreachable_pub` (deny) requires pub(crate)"
)]
pub(crate) fn digest_untrusted_json(
    json: &[u8],
    strategy: &DigestStrategy,
) -> Result<CanonicalDigest, ReceiptIdentityError> {
    let value = vr_jcs::strict_parse::parse_json_value_no_duplicates(json)?;
    Ok(to_canon_digest_with(&value, strategy)?)
}

#[cfg(test)]
#[path = "canonical_identity_tests.rs"]
mod tests;
