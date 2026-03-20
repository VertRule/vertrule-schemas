//! Error types for definition validation.

/// Errors produced during type validation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DefinitionError {
    /// A digest hex string failed validation.
    #[error("invalid digest: {0}")]
    InvalidDigest(String),

    /// A payload contained nondeterministic content (e.g. floats).
    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    /// A policy identifier failed validation.
    #[error("invalid policy id: {0}")]
    InvalidPolicyId(String),

    /// A schema version number has no defined identity binding.
    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),

    /// JSON canonicalization failed.
    #[error("canonicalization failed: {0}")]
    Jcs(#[from] crate::jcs::JcsError),
}
