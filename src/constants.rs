//! Canonical constants for the `VertRule` receipt layer.

/// The only currently supported envelope schema version.
pub const ENVELOPE_VERSION_1: u32 = 1;

/// The digest algorithm used throughout `VertRule`.
pub const DIGEST_ALGORITHM: &str = "BLAKE3";

/// Expected hex string length for a BLAKE3 digest (64 hex chars = 32 bytes).
pub const DIGEST_HEX_LEN: usize = 64;

/// Expected byte length of a BLAKE3 digest.
pub const DIGEST_BYTE_LEN: usize = 32;

/// The canonicalization scheme used for receipt payloads.
pub const CANONICALIZATION: &str = "JCS";
