//! Narrow text-claim proposal/admission profile (ADR-049).
//!
//! These are passive wire shapes. Authority-bearing construction and identity
//! live in the sibling `vr-proposal-admission` crate.

use serde::{Deserialize, Serialize};

use crate::{DigestBytes, IJsonUInt};

/// Proposal receipt payload discriminator.
pub const AGENT_PROPOSAL_PAYLOAD_KIND: &str = "workflow.agent_proposal";
/// Proposal receipt schema identifier.
pub const AGENT_PROPOSAL_SCHEMA: &str = "vr.workflow.agent_proposal@0.1";
/// Admission receipt payload discriminator.
pub const PROPOSAL_ADMISSION_PAYLOAD_KIND: &str = "workflow.proposal_admission";
/// Admission receipt schema identifier.
pub const PROPOSAL_ADMISSION_SCHEMA: &str = "vr.workflow.proposal_admission@0.1";
/// Portable bundle format.
pub const PROPOSAL_ADMISSION_BUNDLE_FORMAT: &str = "vr-proposal-admission/v1";

/// One ordered text claim proposed by a stochastic extractor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposedTextClaim {
    /// One-based order within the proposal.
    pub ordinal: IJsonUInt,
    /// Exact proposed text; not an admitted fact.
    pub text: String,
}

/// Workflow-specific, non-authoritative agent proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextClaimAgentProposal {
    /// Source answer-interaction receipt identity.
    pub source_interaction_digest: DigestBytes,
    /// Stochastic extraction-interaction receipt identity.
    pub extraction_interaction_digest: DigestBytes,
    /// Proposed claims in stable ordinal order.
    pub claims: Vec<ProposedTextClaim>,
}

/// Proposal receipt payload committing the proposal identity and lineage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentProposalReceiptPayload {
    /// Stable semantic subtype.
    pub payload_kind: String,
    /// Versioned encoding law.
    pub schema: String,
    /// Domain-separated identity of `proposal`.
    pub proposal_digest: DigestBytes,
    /// Exact non-authoritative proposal.
    pub proposal: TextClaimAgentProposal,
}

/// Semantic purpose of an external authority signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationPurpose {
    /// Human judgement about a specific proposal receipt.
    ProposalApproval,
    /// Permission for a principal/action pair; not proposal approval.
    ActionAuthorization,
}

/// Closed rejection reasons for the initial review surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimRejectionReason {
    /// Reviewer explicitly declines the proposed claim.
    UserRejected,
    /// Reviewer considers the proposed text incorrect.
    Incorrect,
    /// Reviewer considers the proposed text unsupported.
    Unsupported,
}

/// Human decision about one proposed claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClaimAdmissionDecision {
    /// Admit the exact proposed text.
    Approve {
        /// One-based proposed claim ordinal.
        claim_ordinal: IJsonUInt,
    },
    /// Admit edited text while retaining proposed-value lineage.
    Edit {
        /// One-based proposed claim ordinal.
        claim_ordinal: IJsonUInt,
        /// Exact human-edited admitted text.
        admitted_text: String,
    },
    /// Preserve an explicit rejection.
    Reject {
        /// One-based proposed claim ordinal.
        claim_ordinal: IJsonUInt,
        /// Closed rejection reason.
        reason: ClaimRejectionReason,
    },
}

impl ClaimAdmissionDecision {
    /// Claim ordinal targeted by this decision.
    #[must_use]
    pub const fn claim_ordinal(&self) -> IJsonUInt {
        match self {
            Self::Approve { claim_ordinal }
            | Self::Edit { claim_ordinal, .. }
            | Self::Reject { claim_ordinal, .. } => *claim_ordinal,
        }
    }
}

/// External human signal offered to the deterministic admission transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdmissionSignal {
    /// Must be `proposal_approval` for this transition.
    pub purpose: AttestationPurpose,
    /// Exact proposal receipt `event_hash` being reviewed.
    pub subject_proposal_receipt_digest: DigestBytes,
    /// Context in which that proposal is reviewed.
    pub context_digest: DigestBytes,
    /// Recorded actor assertion; this profile does not authenticate it.
    pub actor_assertion: String,
    /// Exactly one decision per proposed claim.
    pub decisions: Vec<ClaimAdmissionDecision>,
}

/// One admitted claim with immutable lineage to its proposed value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmittedClaim {
    /// Original one-based ordinal.
    pub ordinal: IJsonUInt,
    /// Admitted text (original for approve, new for edit).
    pub text: String,
    /// Domain-separated identity of the original proposed claim.
    pub proposed_claim_digest: DigestBytes,
    /// `approve` or `edit`.
    pub operation: AdmittedClaimOperation,
}

/// How an admitted claim was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmittedClaimOperation {
    /// Exact text admitted.
    Approve,
    /// Human-edited text admitted.
    Edit,
}

/// Explicitly rejected proposed claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectedClaim {
    /// Original one-based ordinal.
    pub ordinal: IJsonUInt,
    /// Domain-separated identity of the rejected proposed claim.
    pub proposed_claim_digest: DigestBytes,
    /// Closed rejection reason.
    pub reason: ClaimRejectionReason,
}

/// Admission receipt payload committing the signal and complete outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionReceiptPayload {
    /// Stable semantic subtype.
    pub payload_kind: String,
    /// Versioned encoding law.
    pub schema: String,
    /// Proposal receipt `event_hash` admitted by this transition.
    pub proposal_receipt_digest: DigestBytes,
    /// Required proposal/admission context.
    pub context_digest: DigestBytes,
    /// Domain-separated identity of the complete external signal.
    pub admission_signal_digest: DigestBytes,
    /// Complete external signal consumed by admission.
    pub signal: ExternalAdmissionSignal,
    /// Approved/edited claims.
    pub admitted_claims: Vec<AdmittedClaim>,
    /// Rejected claims, preserved explicitly.
    pub rejected_claims: Vec<RejectedClaim>,
}

/// Wire projection of the authority-bearing admitted proposal.
///
/// Deserializing this passive shape does not confer authority. Consumers must
/// obtain it from `vr-proposal-admission` or verify its complete bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmittedProposal {
    /// Source proposal receipt identity.
    pub proposal_receipt_digest: DigestBytes,
    /// Admission receipt identity.
    pub admission_receipt_digest: DigestBytes,
    /// Admitted claims.
    pub claims: Vec<AdmittedClaim>,
    /// Explicit rejections.
    pub rejected_claims: Vec<RejectedClaim>,
}

/// Portable closure for one proposal/admission transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalAdmissionBundle {
    /// Frozen bundle format.
    #[serde(rename = "_format")]
    pub format: String,
    /// Exact JCS-canonical proposal receipt envelope.
    pub proposal_canonical: String,
    /// Exact JCS-canonical admission receipt envelope.
    pub admission_canonical: String,
    /// Admitted proposal wire projection.
    pub admitted_proposal: AdmittedProposal,
    /// Domain-separated identity of `admitted_proposal`.
    pub admitted_proposal_digest: DigestBytes,
}
