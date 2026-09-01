//! `vr.policy.*` interchange carriers.
//!
//! Passive schema types shared by the pre-policy resolver
//! (`vertrule-runtime-port`), the policy host (`vertrule-policy-wasm`)
//! and the ABI v3 guest kernel (`vr-policy-kernel`). Nothing in this
//! module executes policy or holds an engine handle — that is what makes
//! it consumable from the pre-policy side without a dependency path to
//! policy execution.
//!
//! Only the **input** side lives here. `VERDICT_FORMAT` and the verdict
//! carrier stay in `vertrule-policy-wasm`: no pre-policy consumer needs
//! them, so relocating them would widen this crate's surface for nothing.

pub mod governance_input;
pub mod input;

pub use governance_input::{
    EvaluationInputKindV1, GovernanceEvaluationInputV1, GovernanceInputError,
    GovernanceOperationV1, GovernancePolicyStatusV1, GovernanceSystemStatusV1,
    GovernanceSystemSubjectV1, LinkedPolicyStateV1, GOVERNANCE_INPUT_FORMAT,
};
pub use input::{ClaimEvidence, EvaluationInput, InputCanonicalizationError, INPUT_FORMAT};
