//! Receipt-spine schema types.
//!
//! Types in this module define the structural discriminators and shape
//! types for the receipt layer. Constitutional envelope/header nouns live
//! here. Verification behavior does not.
//!
//! Two envelope versions coexist: the frozen V1 [`ReceiptEnvelope`] and
//! the V2 [`ReceiptEnvelopeV2`] (ADR-056), whose closed
//! [`ReceiptTypeV2`] vocabulary, admitted [`PayloadSchemaV2`] identities
//! and passive payload shapes also live here.

mod boundary_origin;
mod canonical_u64;
mod decision;
mod envelope;
mod envelope_v2;
mod layered;
mod payload_schema_v2;
mod present_slot;
mod projection;
mod proposal_admission;
mod proposal_admission_v2;
mod receipt_type;
mod receipt_type_v2;
mod runtime_port_submit_outcome;
mod training_receipt;
mod verifiable_ai_record;
mod verifiable_ai_record_v2;
mod verified_metadata;

pub use boundary_origin::BoundaryOrigin;
pub use decision::{
    DecisionReceiptPayload, DecisionVerdict, DependencyRelation, DependencyRole, SupportMember,
    DECISION_PAYLOAD_KIND, DECISION_PAYLOAD_SCHEMA,
};
pub use envelope::{EventHashProfileId, ReceiptEnvelope};
pub use envelope_v2::ReceiptEnvelopeV2;
pub use layered::{
    ClosureManifest, ModelReceiptPayload, PackReceiptPayload, ProviderReceiptPayload,
    CLOSURE_MANIFEST_SCHEMA, MODEL_PAYLOAD_KIND, PACK_PAYLOAD_KIND, PROVIDER_PAYLOAD_KIND,
};
pub use payload_schema_v2::PayloadSchemaV2;
pub use projection::ProjectsToReceiptEnvelope;
pub use proposal_admission::{
    AdmissionReceiptPayload, AdmittedClaim, AdmittedClaimOperation, AdmittedProposal,
    AgentProposalReceiptPayload, AttestationPurpose, ClaimAdmissionDecision, ClaimRejectionReason,
    ExternalAdmissionSignal, ProposalAdmissionBundle, ProposedTextClaim, RejectedClaim,
    TextClaimAgentProposal, AGENT_PROPOSAL_PAYLOAD_KIND, AGENT_PROPOSAL_SCHEMA,
    PROPOSAL_ADMISSION_BUNDLE_FORMAT, PROPOSAL_ADMISSION_PAYLOAD_KIND, PROPOSAL_ADMISSION_SCHEMA,
};
pub use proposal_admission_v2::{
    AgentProposalPayloadV2, ProposalAdmissionBundleV2, ProposalAdmissionPayloadV2,
    PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2,
};
pub use receipt_type::ReceiptType;
pub use receipt_type_v2::ReceiptTypeV2;
pub use runtime_port_submit_outcome::{
    RuntimePortCommandKind, RuntimePortSubmitOutcomePayload, TransitionCommitment,
};
pub use training_receipt::TrainingReceipt;
pub use verifiable_ai_record::{
    CapturedRequest, CapturedResponse, ProviderInteractionPayload, VerifiableAiRecordArtifact,
    VerifiableAiRecordArtifactV1, VerifiableAiRecordPayload, VerifiableAiRecordProposalAdmission,
    PROVIDER_INTERACTION_PAYLOAD_KIND, PROVIDER_INTERACTION_SCHEMA, VERIFIABLE_AI_RECORD_FORMAT,
    VERIFIABLE_AI_RECORD_FORMAT_V1, VERIFIABLE_AI_RECORD_FORMAT_V2,
    VERIFIABLE_AI_RECORD_PAYLOAD_KIND, VERIFIABLE_AI_RECORD_POLICY, VERIFIABLE_AI_RECORD_SCHEMA,
};
pub use verifiable_ai_record_v2::{
    ProviderInteractionPayloadV2, VerifiableAiRecordArtifactV3, VerifiableAiRecordPayloadV2,
    VERIFIABLE_AI_RECORD_FORMAT_V3,
};
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
