//! V2 (`@0.2`) provider-interaction and verifiable-AI-record payload shapes
//! and the portable record artifact (ADR-050 under ADR-056; registry rows
//! P4 and P5; M2-0 rulings D1/D2).
//!
//! Passive shapes only. Construction lives in `vr-verifiable-ai-record`;
//! every V2 law is a registry row in `vertrule-verifier`.
//!
//! Compared with the frozen `@0.1` shapes ([`ProviderInteractionPayload`]
//! and [`VerifiableAiRecordPayload`](crate::VerifiableAiRecordPayload)):
//! `payload_kind` is dropped (ADR-056 R1); the interaction carries the
//! exact prompt and response text **without** the `@0.1` leaf digests
//! (D1 — a digest of data the receipt already commits requires independent
//! semantic demand, and none exists; the `vertrule.record.provider-*.v1`
//! laws are `VerifyAllowed ∧ MintDenied` with no successor formation); and
//! every record reference is the V2 `receipt_digest` of a V2 receipt (D2 —
//! same 32 bytes, different semantics, hence the version boundary).
//! `record_policy` remains a payload fact (G2-2).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{DigestBytes, ReceiptEnvelopeV2};

/// Portable V2 record artifact format: the record receipt plus its
/// digest-keyed evidence set.
pub const VERIFIABLE_AI_RECORD_FORMAT_V3: &str = "vr-verifiable-ai-record/v3";

/// Payload of a `vr.ai.provider_interaction` receipt
/// (`vr.ai.provider_interaction@0.2`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderInteractionPayloadV2 {
    /// Self-described payload schema label; must equal the registry label.
    pub schema: String,
    /// Adapter-stable provider name.
    pub provider: String,
    /// Model requested by server configuration.
    pub requested_model: String,
    /// Provider-declared model, when exposed. Omitted when absent; `null`
    /// is rejected (ADR-056 R8).
    #[serde(
        default,
        deserialize_with = "crate::receipts::present_slot::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub resolved_model: Option<String>,
    /// Provider response id, when exposed; captured metadata, not
    /// attestation. Omitted when absent; `null` is rejected.
    #[serde(
        default,
        deserialize_with = "crate::receipts::present_slot::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub provider_response_id: Option<String>,
    /// Frozen adapter projection policy; a payload fact, never a policy
    /// binding (G2-2).
    pub capture_policy_version: String,
    /// Exact UTF-8 prompt submitted through the provider boundary.
    pub prompt: String,
    /// Exact UTF-8 text projection captured from the provider response.
    pub response: String,
    /// Explicit provider-attestation truth boundary.
    pub provider_attestation: String,
}

/// Payload of a `vr.record.verifiable_ai_record` receipt
/// (`vr.record.verifiable_ai_record@0.2`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiableAiRecordPayloadV2 {
    /// Self-described payload schema label; must equal the registry label.
    pub schema: String,
    /// Frozen assurance boundary: integrity and lineage, not truth
    /// (`vr.record.integrity_not_truth@0.1`); a payload fact (G2-2).
    pub record_policy: String,
    /// `receipt_digest` of the source answer `vr.ai.provider_interaction`
    /// receipt.
    pub source_interaction_digest: DigestBytes,
    /// `receipt_digest` of the stochastic extraction
    /// `vr.ai.provider_interaction` receipt.
    pub extraction_interaction_digest: DigestBytes,
    /// `receipt_digest` of the `vr.workflow.agent_proposal` receipt.
    pub proposal_receipt_digest: DigestBytes,
    /// `receipt_digest` of the `vr.workflow.proposal_admission` receipt
    /// (the former cross-type `parent_id`).
    pub admission_receipt_digest: DigestBytes,
    /// Domain-separated admitted-proposal identity
    /// (`vertrule.admitted-proposal.v1`), re-derivable from the proposal
    /// and admission receipts.
    pub admitted_proposal_digest: DigestBytes,
}

/// Portable V2 record artifact: the root record receipt and its evidence
/// set, keyed by the `receipt_digest` each entry claims.
///
/// A presentation of the ADR-056 §3.5 evidence set, not a receipt: it is
/// deliberately not identity-bearing (whitespace and object formatting may
/// change). Every entry is independently ingested and its `receipt_digest`
/// recomputed against its key; the root record receipt remains the stable
/// record identity. The envelopes are obtained from
/// `vr_receipt_identity::seal_receipt_v2` or deserialised from verified
/// bytes, never assembled field by field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiableAiRecordArtifactV3 {
    /// Frozen portable-artifact format; must equal
    /// [`VERIFIABLE_AI_RECORD_FORMAT_V3`].
    #[serde(rename = "_format")]
    pub format: String,
    /// Root identity-bearing `vr.record.verifiable_ai_record` receipt.
    pub record: ReceiptEnvelopeV2,
    /// Evidence receipts (the two interactions, the proposal and the
    /// admission), keyed by claimed `receipt_digest`.
    pub evidence: BTreeMap<DigestBytes, ReceiptEnvelopeV2>,
}

#[cfg(test)]
#[path = "verifiable_ai_record_v2_tests.rs"]
mod verifiable_ai_record_v2_tests;
