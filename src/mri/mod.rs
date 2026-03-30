//! MRI (Model Reasoning Instrumentation) payload schemas.
//!
//! Domain-specific types for batch-aware MRI receipt payloads.
//! These types are receipt-visible: they affect the meaning of
//! emitted invariant values and are validated by `vertrule-verifier`.

mod batch_payload;
mod gradient_coupling;
mod reduction;

pub use batch_payload::MriBatchPayload;
pub use gradient_coupling::GradientCouplingPayload;
pub use reduction::{
    BatchReduction, ReductionAxis, ReductionMode, ReductionProvenance, TokenReduction,
};
