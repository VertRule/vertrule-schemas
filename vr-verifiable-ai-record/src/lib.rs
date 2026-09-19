//! Deterministic Verifiable AI Record sealing law (ADR-050 under ADR-056).
//!
//! This crate is the only shared constructor of the V2 provider-interaction
//! receipt ([`seal_provider_interaction_v2`]) and of the root record receipt
//! ([`seal_record_v2`]). It performs no I/O, clock read, randomness, provider
//! call, policy evaluation, or truth assessment. Every V2 law is judged by
//! `vertrule-verifier`; the producer only refuses inputs it cannot bind
//! (wrong receipt type, broken lineage).
//!
//! The V1 (`@0.1`) mint is `MintDenied` (ADR-056 R12; M2-1). The six frozen
//! `vertrule.record.*.v1` derivations below are `VerifyAllowed` for the
//! historical fixtures (ADR-054 §2 rows 1–6, E2) and are never called by a
//! V2 constructor: under M2-0 D1 the V2 interaction carries the prompt and
//! response text and no leaf content identity.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use vertrule_schemas::{
    AdmittedProposal, AgentProposalPayloadV2, CanonicalPayload, DigestBytes, PayloadSchemaV2,
    ProposalAdmissionPayloadV2, ProviderInteractionPayloadV2, ReceiptEnvelopeV2, ReceiptTypeV2,
    VerifiableAiRecordArtifactV3, VerifiableAiRecordPayloadV2, VERIFIABLE_AI_RECORD_FORMAT_V3,
    VERIFIABLE_AI_RECORD_POLICY,
};
use vr_receipt_identity::{seal_receipt_v2, ReceiptV2Draft};

/// Frozen prompt leaf-digest domain, promoted byte-neutrally from Slice 2.
pub const PROMPT_DOMAIN: &str = "vertrule.record.provider-prompt.v1";
/// Frozen response leaf-digest domain, promoted byte-neutrally from Slice 2.
pub const RESPONSE_DOMAIN: &str = "vertrule.record.provider-response.v1";
/// Frozen interaction-schema binding domain.
pub const INTERACTION_SCHEMA_DOMAIN: &str = "vertrule.record.interaction-schema.v1";
/// Frozen interaction capture-policy binding domain.
pub const CAPTURE_POLICY_DOMAIN: &str = "vertrule.record.capture-policy.v1";
/// Frozen record-schema binding domain.
pub const RECORD_SCHEMA_DOMAIN: &str = "vertrule.record.verifiable-ai-record-schema.v1";
/// Frozen record-policy binding domain.
pub const RECORD_POLICY_DOMAIN: &str = "vertrule.record.verifiable-ai-record-policy.v1";

/// Provider-neutral captured inputs offered by an adapter to the shared
/// interaction receipt constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInteractionCaptureInput {
    /// Exact prompt submitted through the provider adapter.
    pub prompt: String,
    /// Adapter-stable provider name.
    pub provider: String,
    /// Requested model identifier.
    pub requested_model: String,
    /// Provider-declared model identifier, when exposed.
    pub resolved_model: Option<String>,
    /// Provider response identifier, when exposed.
    pub provider_response_id: Option<String>,
    /// Frozen adapter projection policy.
    pub capture_policy_version: String,
    /// Exact captured response projection.
    pub response: String,
}

/// Sealed root record receipt with the evidence set it binds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedVerifiableAiRecordV2 {
    record: ReceiptEnvelopeV2,
    payload: VerifiableAiRecordPayloadV2,
    admitted_proposal: AdmittedProposal,
    evidence: BTreeMap<DigestBytes, ReceiptEnvelopeV2>,
}

impl SealedVerifiableAiRecordV2 {
    /// Borrow the root `vr.record.verifiable_ai_record` receipt.
    #[must_use]
    pub const fn record(&self) -> &ReceiptEnvelopeV2 {
        &self.record
    }

    /// Borrow the record payload.
    #[must_use]
    pub const fn payload(&self) -> &VerifiableAiRecordPayloadV2 {
        &self.payload
    }

    /// Borrow the admitted proposal projection the record's
    /// `admitted_proposal_digest` identifies.
    #[must_use]
    pub const fn admitted_proposal(&self) -> &AdmittedProposal {
        &self.admitted_proposal
    }

    /// Borrow the four child receipts, keyed by `receipt_digest`.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeMap<DigestBytes, ReceiptEnvelopeV2> {
        &self.evidence
    }

    /// The portable evidence-set presentation (`vr-verifiable-ai-record/v3`).
    #[must_use]
    pub fn artifact(&self) -> VerifiableAiRecordArtifactV3 {
        VerifiableAiRecordArtifactV3 {
            format: VERIFIABLE_AI_RECORD_FORMAT_V3.to_owned(),
            record: self.record.clone(),
            evidence: self.evidence.clone(),
        }
    }
}

/// Seal adapter-captured input into a `vr.ai.provider_interaction` receipt
/// (`vr.ai.provider_interaction@0.2`): the exact prompt and response text,
/// the provider/model facts and the capture-policy version, with no
/// binding slot and no leaf content identity (M2-0 D1).
///
/// # Errors
///
/// Returns a typed error when the payload cannot be projected or the
/// receipt cannot be formed.
pub fn seal_provider_interaction_v2(
    input: &ProviderInteractionCaptureInput,
) -> Result<ReceiptEnvelopeV2, VerifiableAiRecordError> {
    let payload = ProviderInteractionPayloadV2 {
        schema: PayloadSchemaV2::VR_AI_PROVIDER_INTERACTION_0_2
            .label()
            .to_owned(),
        provider: input.provider.clone(),
        requested_model: input.requested_model.clone(),
        resolved_model: input.resolved_model.clone(),
        provider_response_id: input.provider_response_id.clone(),
        capture_policy_version: input.capture_policy_version.clone(),
        prompt: input.prompt.clone(),
        response: input.response.clone(),
        provider_attestation: "not_provided".to_owned(),
    };
    seal_v2(
        ReceiptTypeV2::AiProviderInteraction,
        PayloadSchemaV2::VR_AI_PROVIDER_INTERACTION_0_2,
        &payload,
    )
}

/// Seal the root `vr.record.verifiable_ai_record` receipt
/// (`vr.record.verifiable_ai_record@0.2`) over its four V2 children.
///
/// Binds, by `receipt_digest`, the source answer interaction, the
/// extraction interaction, the proposal and the admission, plus the
/// admitted-proposal identity derived from the admission's committed
/// partition. The producer refuses a child of the wrong type, a proposal
/// whose lineage is not the two interactions given, or an admission that
/// names another proposal; every other law — each child's own laws, the
/// admission's outcome reconstruction, the record's digest — is the
/// verifier's, and the caller must verify the sealed record against the
/// evidence set before treating it as a record.
///
/// # Errors
///
/// Returns a typed error for a mistyped child, a lineage mismatch, an
/// admission of another proposal, a payload that does not deserialise as
/// its `@0.2` shape, or a receipt formation failure.
pub fn seal_record_v2(
    source_interaction: &ReceiptEnvelopeV2,
    extraction_interaction: &ReceiptEnvelopeV2,
    proposal: &ReceiptEnvelopeV2,
    admission: &ReceiptEnvelopeV2,
) -> Result<SealedVerifiableAiRecordV2, VerifiableAiRecordError> {
    for interaction in [source_interaction, extraction_interaction] {
        if interaction.receipt_type != ReceiptTypeV2::AiProviderInteraction {
            return Err(VerifiableAiRecordError::UnsupportedInteractionSchema);
        }
    }
    if proposal.receipt_type != ReceiptTypeV2::WorkflowAgentProposal
        || admission.receipt_type != ReceiptTypeV2::WorkflowProposalAdmission
    {
        return Err(VerifiableAiRecordError::UnsupportedProposalAdmissionSchema);
    }
    let proposal_payload: AgentProposalPayloadV2 =
        serde_json::from_value(proposal.payload.as_value().clone())?;
    let admission_payload: ProposalAdmissionPayloadV2 =
        serde_json::from_value(admission.payload.as_value().clone())?;
    if proposal_payload.proposal.source_interaction_digest != source_interaction.receipt_digest
        || proposal_payload.proposal.extraction_interaction_digest
            != extraction_interaction.receipt_digest
    {
        return Err(VerifiableAiRecordError::InteractionLineageMismatch);
    }
    if admission_payload.proposal_receipt_digest != proposal.receipt_digest {
        return Err(VerifiableAiRecordError::ProposalAdmissionMismatch);
    }
    let admitted_proposal = AdmittedProposal {
        proposal_receipt_digest: proposal.receipt_digest,
        admission_receipt_digest: admission.receipt_digest,
        claims: admission_payload.admitted_claims,
        rejected_claims: admission_payload.rejected_claims,
    };
    let admitted_proposal_digest =
        vr_proposal_admission::admitted_proposal_digest(&admitted_proposal)
            .map_err(|error| VerifiableAiRecordError::ProposalAdmission(error.to_string()))?;
    let payload = VerifiableAiRecordPayloadV2 {
        schema: PayloadSchemaV2::VR_RECORD_VERIFIABLE_AI_RECORD_0_2
            .label()
            .to_owned(),
        record_policy: VERIFIABLE_AI_RECORD_POLICY.to_owned(),
        source_interaction_digest: source_interaction.receipt_digest,
        extraction_interaction_digest: extraction_interaction.receipt_digest,
        proposal_receipt_digest: proposal.receipt_digest,
        admission_receipt_digest: admission.receipt_digest,
        admitted_proposal_digest,
    };
    let record = seal_v2(
        ReceiptTypeV2::RecordVerifiableAiRecord,
        PayloadSchemaV2::VR_RECORD_VERIFIABLE_AI_RECORD_0_2,
        &payload,
    )?;
    let evidence = [
        source_interaction,
        extraction_interaction,
        proposal,
        admission,
    ]
    .into_iter()
    .map(|receipt| (receipt.receipt_digest, receipt.clone()))
    .collect();
    Ok(SealedVerifiableAiRecordV2 {
        record,
        payload,
        admitted_proposal,
        evidence,
    })
}

/// Seal a group-2 V2 receipt: every binding slot absent, no chain
/// (ADR-056 §3.4; registry rows P4/P5).
fn seal_v2<T: serde::Serialize>(
    receipt_type: ReceiptTypeV2,
    schema: PayloadSchemaV2,
    payload: &T,
) -> Result<ReceiptEnvelopeV2, VerifiableAiRecordError> {
    let schema_digest = schema
        .identity()
        .map_err(|error| VerifiableAiRecordError::Receipt(error.to_string()))?;
    seal_receipt_v2(ReceiptV2Draft {
        receipt_type,
        schema_digest,
        context_digest: None,
        policy_digest: None,
        logical_time: 1,
        parent_id: None,
        payload: CanonicalPayload::new(serde_json::to_value(payload)?)?,
    })
    .map_err(|error| VerifiableAiRecordError::Receipt(error.to_string()))
}

/// Domain-separated prompt digest (frozen `@0.1` law; `VerifyAllowed`,
/// never minted in V2).
#[must_use]
pub fn prompt_digest(prompt: &str) -> DigestBytes {
    digest_bytes(PROMPT_DOMAIN, prompt.as_bytes())
}

/// Domain-separated captured-response digest (frozen `@0.1` law;
/// `VerifyAllowed`, never minted in V2).
#[must_use]
pub fn response_digest(response: &str) -> DigestBytes {
    digest_bytes(RESPONSE_DOMAIN, response.as_bytes())
}

/// Domain-separated interaction-schema label digest (frozen `@0.1` law).
#[must_use]
pub fn interaction_schema_digest(schema: &str) -> DigestBytes {
    digest_bytes(INTERACTION_SCHEMA_DOMAIN, schema.as_bytes())
}

/// Domain-separated capture-policy label digest (frozen `@0.1` law).
#[must_use]
pub fn capture_policy_digest(policy: &str) -> DigestBytes {
    digest_bytes(CAPTURE_POLICY_DOMAIN, policy.as_bytes())
}

/// Domain-separated record-schema label digest (frozen `@0.1` law).
#[must_use]
pub fn record_schema_digest(schema: &str) -> DigestBytes {
    digest_bytes(RECORD_SCHEMA_DOMAIN, schema.as_bytes())
}

/// Domain-separated record-policy label digest (frozen `@0.1` law).
#[must_use]
pub fn record_policy_digest(policy: &str) -> DigestBytes {
    digest_bytes(RECORD_POLICY_DOMAIN, policy.as_bytes())
}

fn digest_bytes(domain: &str, bytes: &[u8]) -> DigestBytes {
    let mut hasher = blake3::Hasher::new_derive_key(domain);
    hasher.update(bytes);
    DigestBytes::from_array(*hasher.finalize().as_bytes())
}

/// Deterministic record-sealing failures.
#[derive(Debug, thiserror::Error)]
pub enum VerifiableAiRecordError {
    /// A child offered as a provider interaction is not a
    /// `vr.ai.provider_interaction` receipt.
    #[error("unsupported provider-interaction schema")]
    UnsupportedInteractionSchema,
    /// The proposal or admission child is not of its expected receipt type.
    #[error("unsupported proposal/admission schema")]
    UnsupportedProposalAdmissionSchema,
    /// The admission does not name the proposal offered.
    #[error("proposal/admission mismatch")]
    ProposalAdmissionMismatch,
    /// The proposal references different interaction receipts.
    #[error("proposal interaction lineage mismatch")]
    InteractionLineageMismatch,
    /// Shared proposal-admission law failed.
    #[error("proposal/admission reconstruction failed: {0}")]
    ProposalAdmission(String),
    /// JSON projection failed.
    #[error("JSON projection failed: {0}")]
    Json(#[from] serde_json::Error),
    /// Payload canonical admission failed.
    #[error("payload admission failed: {0}")]
    Payload(#[from] vertrule_schemas::DefinitionError),
    /// Constitutional receipt identity failed.
    #[error("receipt identity failed: {0}")]
    Receipt(String),
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mint_ratchet_guard_tests;
