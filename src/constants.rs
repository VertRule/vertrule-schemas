//! Spec v1 bindings for the `VertRule` receipt layer.
//!
//! These constants are bound to specification version 1. Any change to
//! canonicalization semantics, commitment primitives, receipt envelope
//! structure, or context binding rules MUST increment the specification
//! version and MUST NOT preserve identity equivalence.

/// The only currently supported envelope schema version.
pub const ENVELOPE_VERSION_1: u32 = 1;

/// Spec v1 binding: the commitment primitive.
pub const DIGEST_ALGORITHM: &str = "BLAKE3";

/// Spec v1 binding: expected hex string length for a digest (64 hex chars = 32 bytes).
pub const DIGEST_HEX_LEN: usize = 64;

/// Spec v1 binding: expected byte length of a digest.
pub const DIGEST_BYTE_LEN: usize = 32;

/// Spec v1 binding: the canonicalization scheme for receipt payloads.
pub const CANONICALIZATION: &str = "JCS";
