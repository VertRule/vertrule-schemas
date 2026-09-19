//! V2 (`@0.2`) payload shapes and portable bundle for the text-claim
//! proposal/admission profile (ADR-049 under ADR-056; registry rows P2 and
//! P3; M2-0 ruling D3).
//!
//! Passive wire shapes only. Construction and the admission transition
//! live in `vr-proposal-admission`; every V2 law is a registry row in
//! `vertrule-verifier`.
//!
//! Compared with the frozen `@0.1` shapes ([`AgentProposalReceiptPayload`]
//! and [`AdmissionReceiptPayload`](crate::AdmissionReceiptPayload)):
//! `payload_kind` is dropped (ADR-056 R1 — `receipt_type` is the sole
//! discriminator); every receipt reference is the V2 `receipt_digest` of a
//! V2 receipt (ADR-056 §3.4 — a cross-type reference is a payload fact,
//! never `parent_id` or `context_digest`); and the admission payload's
//! outer `context_digest` is dropped (D3 — `signal.context_digest` is the
//! one authoritative representation of the review context). The domain
//! objects [`TextClaimAgentProposal`], [`ExternalAdmissionSignal`],
//! [`AdmittedClaim`], [`RejectedClaim`] and [`AdmittedProposal`] keep their
//! shapes and identity domains; only the receipt identities they embed
//! change.

use serde::{Deserialize, Serialize};

use crate::{
    AdmittedClaim, AdmittedProposal, DigestBytes, ExternalAdmissionSignal, ReceiptEnvelopeV2,
    RejectedClaim, TextClaimAgentProposal,
};

/// Portable V2 bundle format.
pub const PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2: &str = "vr-proposal-admission/v2";

/// Payload of a `vr.workflow.agent_proposal` receipt
/// (`vr.workflow.agent_proposal@0.2`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentProposalPayloadV2 {
    /// Self-described payload schema label; must equal the registry label.
    pub schema: String,
    /// Domain-separated identity of `proposal`
    /// (`vertrule.agent-proposal.v1` over the JCS object).
    pub proposal_digest: DigestBytes,
    /// Exact non-authoritative proposal. Its `source_interaction_digest`
    /// and `extraction_interaction_digest` are `receipt_digest` values of
    /// `vr.ai.provider_interaction` receipts.
    pub proposal: TextClaimAgentProposal,
}

/// Payload of a `vr.workflow.proposal_admission` receipt
/// (`vr.workflow.proposal_admission@0.2`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalAdmissionPayloadV2 {
    /// Self-described payload schema label; must equal the registry label.
    pub schema: String,
    /// `receipt_digest` of the `vr.workflow.agent_proposal` receipt this
    /// transition admitted (the former cross-type `parent_id`).
    pub proposal_receipt_digest: DigestBytes,
    /// Domain-separated identity of `signal`
    /// (`vertrule.admission-signal.v1` over the JCS object).
    pub admission_signal_digest: DigestBytes,
    /// Complete external signal consumed by admission. Its
    /// `subject_proposal_receipt_digest` names the same receipt as
    /// `proposal_receipt_digest`; its `context_digest` is the proposal's
    /// `source_interaction_digest`.
    pub signal: ExternalAdmissionSignal,
    /// Approved/edited claims.
    pub admitted_claims: Vec<AdmittedClaim>,
    /// Rejected claims, preserved explicitly.
    pub rejected_claims: Vec<RejectedClaim>,
}

/// Portable V2 closure for one proposal/admission transition.
///
/// A presentation of the evidence set, not a receipt: it carries the two
/// V2 receipts verbatim plus the re-derivable [`AdmittedProposal`]
/// projection and its domain identity. Whitespace and object formatting
/// may change; each receipt is independently verified by its own
/// `receipt_digest`. The envelopes are obtained from
/// `vr_receipt_identity::seal_receipt_v2` or deserialised from verified
/// bytes, never assembled field by field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalAdmissionBundleV2 {
    /// Frozen bundle format; must equal [`PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2`].
    #[serde(rename = "_format")]
    pub format: String,
    /// The `vr.workflow.agent_proposal` receipt.
    pub proposal: ReceiptEnvelopeV2,
    /// The `vr.workflow.proposal_admission` receipt.
    pub admission: ReceiptEnvelopeV2,
    /// Admitted proposal wire projection; its two receipt references are
    /// V2 `receipt_digest` values.
    pub admitted_proposal: AdmittedProposal,
    /// Domain-separated identity of `admitted_proposal`
    /// (`vertrule.admitted-proposal.v1` over the JCS object).
    pub admitted_proposal_digest: DigestBytes,
}

#[cfg(test)]
#[path = "proposal_admission_v2_tests.rs"]
mod proposal_admission_v2_tests;
