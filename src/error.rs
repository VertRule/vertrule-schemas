//! Error types for definition validation.

/// Errors produced during type validation.
#[derive(Debug, thiserror::Error)]
pub enum DefinitionError {
    /// A digest hex string failed validation.
    #[error("invalid digest: {0}")]
    InvalidDigest(String),
}
