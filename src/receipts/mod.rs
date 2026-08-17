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
mod proposal_admission;
mod receipt_type;
mod training_receipt;
mod verifiable_ai_record;
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
    ClosureManifest, ModelReceiptPayload, PackReceiptPayload, ProviderReceiptPayload,
    CLOSURE_MANIFEST_SCHEMA, MODEL_PAYLOAD_KIND, PACK_PAYLOAD_KIND, PROVIDER_PAYLOAD_KIND,
};
pub use projection::ProjectsToReceiptEnvelope;
pub use proposal_admission::{
    AdmissionReceiptPayload, AdmittedClaim, AdmittedClaimOperation, AdmittedProposal,
    AgentProposalReceiptPayload, AttestationPurpose, ClaimAdmissionDecision, ClaimRejectionReason,
    ExternalAdmissionSignal, ProposalAdmissionBundle, ProposedTextClaim, RejectedClaim,
    TextClaimAgentProposal, AGENT_PROPOSAL_PAYLOAD_KIND, AGENT_PROPOSAL_SCHEMA,
    PROPOSAL_ADMISSION_BUNDLE_FORMAT, PROPOSAL_ADMISSION_PAYLOAD_KIND, PROPOSAL_ADMISSION_SCHEMA,
};
pub use receipt_type::ReceiptType;
pub use training_receipt::TrainingReceipt;
pub use verifiable_ai_record::{
    CapturedRequest, CapturedResponse, ProviderInteractionPayload, VerifiableAiRecordArtifact,
    VerifiableAiRecordArtifactV1, VerifiableAiRecordPayload, VerifiableAiRecordProposalAdmission,
    PROVIDER_INTERACTION_PAYLOAD_KIND, PROVIDER_INTERACTION_SCHEMA, VERIFIABLE_AI_RECORD_FORMAT,
    VERIFIABLE_AI_RECORD_FORMAT_V1, VERIFIABLE_AI_RECORD_FORMAT_V2,
    VERIFIABLE_AI_RECORD_PAYLOAD_KIND, VERIFIABLE_AI_RECORD_POLICY, VERIFIABLE_AI_RECORD_SCHEMA,
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

#[cfg(test)]
#[path = "identity_tests.rs"]
mod identity_tests;
