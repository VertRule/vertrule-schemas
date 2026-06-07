//! Error type for receipt-identity construction.

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
}
