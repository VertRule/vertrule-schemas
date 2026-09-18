//! Error type for receipt-identity construction.

use vertrule_schemas::{DefinitionError, ReceiptTypeV2};
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

    /// The producer refused to mint a receipt of `receipt_type` (ADR-056
    /// R12): the facts presented cannot satisfy the type's registry law, and
    /// no fallback value, re-typing or placeholder is ever substituted.
    #[error("mint denied for {receipt_type}: {reason}")]
    MintDenied {
        /// The V2 receipt type whose mint was refused.
        receipt_type: ReceiptTypeV2,
        /// Which fact was missing or inadmissible.
        reason: String,
    },
}
