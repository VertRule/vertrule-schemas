//! V2 receipt type — the complete semantic discriminator (ADR-056 §3.1).
//!
//! Every admitted label names one **semantic receipt object** and maps,
//! in the verifier-owned registry, to exactly one immutable `ReceiptLaw`.
//! A different law is a different label. Labels follow the grammar
//!
//! ```text
//! receipt_type  ::=  "vr" ("." segment)+          segment ::= [a-z0-9][a-z0-9_-]*
//! ```
//!
//! The vocabulary is **closed**: this enum is deliberately not
//! `#[non_exhaustive]`, because admitting a label is a constitutional change
//! (a `vertrule-schemas` release *and* a registry row in
//! `vertrule-verifier`). The V1 [`ReceiptType`](crate::ReceiptType)
//! vocabulary is not carried into V2 in any form.

use serde::{Deserialize, Serialize};

/// Closed vocabulary of admitted V2 semantic receipt types (ADR-056 R1–R2).
///
/// Serialised as the lowercase dotted label carried in `receipt_type`.
/// An unknown label fails to deserialise; the verifier reports it as
/// `UnknownReceiptType`, never accepting it by guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ReceiptTypeV2 {
    /// A governance decision over a governed subject and action
    /// (payload schema `vr.surface.decision@0.1`).
    #[serde(rename = "vr.governance.decision")]
    GovernanceDecision,
    /// The outcome of a `RuntimePort` `Submit` command
    /// (payload schema `vr.runtime_port.submit_outcome@0.1`).
    #[serde(rename = "vr.runtime_port.submit_outcome")]
    RuntimePortSubmitOutcome,
    /// One captured provider interaction: the exact prompt and the exact
    /// captured response text (payload schema
    /// `vr.ai.provider_interaction@0.2`; ADR-050 under ADR-056).
    #[serde(rename = "vr.ai.provider_interaction")]
    AiProviderInteraction,
    /// A non-authoritative agent-extracted text-claim proposal with its
    /// interaction lineage (payload schema `vr.workflow.agent_proposal@0.2`;
    /// ADR-049 under ADR-056).
    #[serde(rename = "vr.workflow.agent_proposal")]
    WorkflowAgentProposal,
    /// The outcome of one deterministic admission transition over exactly
    /// one sealed proposal (payload schema
    /// `vr.workflow.proposal_admission@0.2`; ADR-049 under ADR-056).
    #[serde(rename = "vr.workflow.proposal_admission")]
    WorkflowProposalAdmission,
    /// The root Verifiable AI Record binding its four child receipts and
    /// the admitted-proposal identity (payload schema
    /// `vr.record.verifiable_ai_record@0.2`; ADR-050 under ADR-056).
    #[serde(rename = "vr.record.verifiable_ai_record")]
    RecordVerifiableAiRecord,
}

impl ReceiptTypeV2 {
    /// Every admitted label, in declaration order.
    pub const ADMITTED: [Self; 6] = [
        Self::GovernanceDecision,
        Self::RuntimePortSubmitOutcome,
        Self::AiProviderInteraction,
        Self::WorkflowAgentProposal,
        Self::WorkflowProposalAdmission,
        Self::RecordVerifiableAiRecord,
    ];

    /// The wire label of this receipt type.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::GovernanceDecision => "vr.governance.decision",
            Self::RuntimePortSubmitOutcome => "vr.runtime_port.submit_outcome",
            Self::AiProviderInteraction => "vr.ai.provider_interaction",
            Self::WorkflowAgentProposal => "vr.workflow.agent_proposal",
            Self::WorkflowProposalAdmission => "vr.workflow.proposal_admission",
            Self::RecordVerifiableAiRecord => "vr.record.verifiable_ai_record",
        }
    }
}

impl std::fmt::Display for ReceiptTypeV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
#[path = "receipt_type_v2_tests.rs"]
mod receipt_type_v2_tests;
