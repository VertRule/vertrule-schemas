//! Receipt-spine schema types.
//!
//! Types in this module define the structural discriminators and shape
//! types for the receipt layer. Constitutional envelope/header nouns live
//! here. Verification behavior does not.

mod boundary_origin;
mod commitment;
mod decision;
mod envelope;
mod identity;
mod layered;
mod projection;
mod receipt_type;
mod training_receipt;
mod verified_metadata;

pub use boundary_origin::BoundaryOrigin;
pub use commitment::compute_event_hash;
pub use decision::{
    DecisionReceiptPayload, DecisionVerdict, DependencyRelation, DependencyRole, SupportMember,
    DECISION_PAYLOAD_KIND, DECISION_PAYLOAD_SCHEMA,
};
pub use envelope::{EventHashProfileId, ReceiptEnvelope};
pub use identity::ReceiptDigest;
pub use layered::{
    ModelReceiptPayload, ProviderReceiptPayload, MODEL_PAYLOAD_KIND, PROVIDER_PAYLOAD_KIND,
};
pub use projection::ProjectsToReceiptEnvelope;
pub use receipt_type::ReceiptType;
pub use training_receipt::TrainingReceipt;
pub use verified_metadata::VerifiedReceiptMetadata;

#[cfg(test)]
#[path = "boundary_origin_tests.rs"]
mod boundary_origin_tests;

#[cfg(test)]
#[path = "decision_tests.rs"]
mod decision_tests;

#[cfg(test)]
#[path = "projection_tests.rs"]
mod projection_tests;

#[cfg(test)]
#[path = "identity_tests.rs"]
mod identity_tests;
