//! Receipt-spine schema types.
//!
//! Types in this module define the structural discriminators and shape
//! types for the receipt layer. Constitutional envelope/header nouns live
//! here. Verification behavior does not.

mod envelope;
mod meta;
mod receipt_type;

pub use envelope::ReceiptEnvelope;
pub use meta::ReceiptMetaV1;
pub use receipt_type::ReceiptType;
