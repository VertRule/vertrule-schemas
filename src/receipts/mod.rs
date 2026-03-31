//! Receipt-spine schema types.
//!
//! Types in this module define the structural discriminators and shape
//! types for the receipt layer. Constitutional envelope/header nouns live
//! here. Verification behavior does not.

mod boundary_origin;
mod commitment;
mod envelope;
mod projection;
mod receipt_type;

pub use boundary_origin::BoundaryOrigin;
pub use commitment::compute_event_hash;
pub use envelope::ReceiptEnvelope;
pub use projection::ProjectsToReceiptEnvelope;
pub use receipt_type::ReceiptType;

#[cfg(test)]
#[path = "boundary_origin_tests.rs"]
mod boundary_origin_tests;

#[cfg(test)]
#[path = "projection_tests.rs"]
mod projection_tests;
