//! Error type for receipt-identity construction.

use vertrule_schemas::DefinitionError;
use vr_identity::IdentityError;
use vr_jcs::JcsError;

/// Failure during receipt-identity commitment construction.
#[derive(Debug, thiserror::Error)]
pub enum ReceiptIdentityError {
    /// Canonicalization or digest computation failed.
    #[error("canonical digest failed: {0}")]
    Jcs(#[from] JcsError),

    /// The envelope did not serialize to a JSON object.
    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    /// The computed digest was not the expected length for the wire shape.
    #[error("invalid digest: {0}")]
    InvalidDigest(String),

    /// A schema shape could not be projected into a canonical receipt.
    #[error("receipt projection failed: {0}")]
    Definition(#[from] DefinitionError),

    /// A declared digest-domain formation law rejected its input.
    #[error("identity formation failed: {0}")]
    Identity(#[from] IdentityError),
}
