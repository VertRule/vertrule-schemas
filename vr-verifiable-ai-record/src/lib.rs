//! Deterministic Verifiable AI Record sealing law (ADR-050).
//!
//! This crate validates the disclosed child artifacts and is the only shared
//! constructor for the root record receipt. It performs no I/O, clock read,
//! randomness, provider call, policy evaluation, or truth assessment.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

use vertrule_schemas::{
    AdmissionReceiptPayload, AdmittedProposal, AgentProposalReceiptPayload, BoundaryOrigin,
    DigestBytes, ProposalAdmissionBundle, ProviderInteractionPayload, ReceiptEnvelope, ReceiptType,
    SchemaVersion, VerifiableAiRecordArtifact, VerifiableAiRecordPayload,
    VerifiableAiRecordProposalAdmission, AGENT_PROPOSAL_PAYLOAD_KIND, AGENT_PROPOSAL_SCHEMA,
    PROPOSAL_ADMISSION_BUNDLE_FORMAT, PROPOSAL_ADMISSION_PAYLOAD_KIND, PROPOSAL_ADMISSION_SCHEMA,
    PROVIDER_INTERACTION_PAYLOAD_KIND, PROVIDER_INTERACTION_SCHEMA, VERIFIABLE_AI_RECORD_FORMAT,
    VERIFIABLE_AI_RECORD_PAYLOAD_KIND, VERIFIABLE_AI_RECORD_POLICY, VERIFIABLE_AI_RECORD_SCHEMA,
};

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

/// Sealed root receipt and portable record bytes.
#[derive(Debug, Clone)]
pub struct SealedVerifiableAiRecord {
    envelope: ReceiptEnvelope,
    payload: VerifiableAiRecordPayload,
    admitted_proposal: AdmittedProposal,
    artifact: String,
}

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

/// Shared, canonical provider-interaction receipt construction result.
#[derive(Debug, Clone)]
pub struct SealedProviderInteraction {
    envelope: ReceiptEnvelope,
    payload: ProviderInteractionPayload,
    canonical: String,
}

impl SealedProviderInteraction {
    /// Borrow the interaction receipt.
    #[must_use]
    pub const fn envelope(&self) -> &ReceiptEnvelope {
        &self.envelope
    }

    /// Borrow the committed provider-neutral payload.
    #[must_use]
    pub const fn payload(&self) -> &ProviderInteractionPayload {
        &self.payload
    }

    /// Borrow exact JCS-canonical interaction receipt bytes as UTF-8.
    #[must_use]
    pub fn canonical(&self) -> &str {
        &self.canonical
    }
}

impl SealedVerifiableAiRecord {
    /// Borrow the root record receipt.
    #[must_use]
    pub const fn envelope(&self) -> &ReceiptEnvelope {
        &self.envelope
    }

    /// Borrow the record payload.
    #[must_use]
    pub const fn payload(&self) -> &VerifiableAiRecordPayload {
        &self.payload
    }

    /// Borrow the admitted proposal reconstructed by the shared admission law.
    #[must_use]
    pub const fn admitted_proposal(&self) -> &AdmittedProposal {
        &self.admitted_proposal
    }

    /// Borrow the deterministic, pretty-printed portable package as UTF-8.
    ///
    /// Package whitespace is presentation-only. Contained receipts retain
    /// their independently reconstructed canonical identities.
    #[must_use]
    pub fn artifact(&self) -> &str {
        &self.artifact
    }
}

/// Seal adapter-captured input at the provider interaction boundary.
///
/// # Errors
///
/// Returns a typed error when the input cannot be projected or canonicalized.
pub fn seal_provider_interaction(
    input: ProviderInteractionCaptureInput,
) -> Result<SealedProviderInteraction, VerifiableAiRecordError> {
    let prompt_digest = prompt_digest(&input.prompt);
    let response_digest = response_digest(&input.response);
    let payload = ProviderInteractionPayload {
        payload_kind: PROVIDER_INTERACTION_PAYLOAD_KIND.to_owned(),
        schema: PROVIDER_INTERACTION_SCHEMA.to_owned(),
        provider: input.provider,
        requested_model: input.requested_model,
        resolved_model: input.resolved_model,
        provider_response_id: input.provider_response_id,
        capture_policy_version: input.capture_policy_version,
        request: vertrule_schemas::CapturedRequest {
            prompt: input.prompt,
            prompt_digest,
        },
        response: vertrule_schemas::CapturedResponse {
            text: input.response,
            response_digest,
        },
        provider_attestation: "not_provided".to_owned(),
    };
    let mut envelope: ReceiptEnvelope = serde_json::from_value(serde_json::json!({
        "envelope_version": SchemaVersion::V1,
        "receipt_type": ReceiptType::Llm,
        "context_digest": prompt_digest,
        "schema_digest": interaction_schema_digest(PROVIDER_INTERACTION_SCHEMA),
        "policy_digest": capture_policy_digest(&payload.capture_policy_version),
        "logical_time": "1",
        "event_hash": DigestBytes::from_array([0; 32]),
        "boundary_origin": BoundaryOrigin::Adapter,
        "digest_algorithm": SchemaVersion::V1.digest_algorithm(),
        "canonicalization": SchemaVersion::V1.canonicalization(),
        "payload": payload,
    }))?;
    envelope.event_hash = vr_receipt_identity::compute_event_hash(&envelope)
        .map_err(|error| VerifiableAiRecordError::Receipt(error.to_string()))?;
    let canonical = canonical_string(&envelope)?;
    let payload = serde_json::from_value(envelope.payload.as_value().clone())?;
    Ok(SealedProviderInteraction {
        envelope,
        payload,
        canonical,
    })
}

/// Seal exact canonical interactions and a proposal/admission bundle as one
/// record-specific portable artifact.
///
/// # Errors
///
/// Fails closed for noncanonical or invalid child artifacts, lineage
/// substitution, unsupported schemas, or canonicalization failure.
pub fn seal_record(
    source_interaction_canonical: &str,
    extraction_interaction_canonical: &str,
    proposal_admission_canonical: &str,
) -> Result<SealedVerifiableAiRecord, VerifiableAiRecordError> {
    let (source_envelope, _) = validate_interaction(source_interaction_canonical)?;
    let (extraction_envelope, _) = validate_interaction(extraction_interaction_canonical)?;
    let (proposal_envelope, admission_envelope, admitted_digest, admitted_proposal) =
        validate_proposal_admission(proposal_admission_canonical)?;

    let proposal_payload: AgentProposalReceiptPayload =
        serde_json::from_value(proposal_envelope.payload.as_value().clone())?;
    if proposal_payload.proposal.source_interaction_digest != source_envelope.event_hash
        || proposal_payload.proposal.extraction_interaction_digest != extraction_envelope.event_hash
    {
        return Err(VerifiableAiRecordError::InteractionLineageMismatch);
    }

    let payload = VerifiableAiRecordPayload {
        payload_kind: VERIFIABLE_AI_RECORD_PAYLOAD_KIND.to_owned(),
        schema: VERIFIABLE_AI_RECORD_SCHEMA.to_owned(),
        record_policy: VERIFIABLE_AI_RECORD_POLICY.to_owned(),
        source_interaction_digest: source_envelope.event_hash,
        extraction_interaction_digest: extraction_envelope.event_hash,
        proposal_receipt_digest: proposal_envelope.event_hash,
        admission_receipt_digest: admission_envelope.event_hash,
        admitted_proposal_digest: admitted_digest,
    };
    let envelope = seal_record_envelope(&payload)?;
    let artifact_value = VerifiableAiRecordArtifact {
        format: VERIFIABLE_AI_RECORD_FORMAT.to_owned(),
        record: envelope.clone(),
        source_interaction: source_envelope,
        extraction_interaction: extraction_envelope,
        proposal_admission: VerifiableAiRecordProposalAdmission {
            format: PROPOSAL_ADMISSION_BUNDLE_FORMAT.to_owned(),
            proposal: proposal_envelope,
            admission: admission_envelope,
        },
    };
    let artifact = serde_json::to_string_pretty(&artifact_value)?;
    Ok(SealedVerifiableAiRecord {
        envelope,
        payload,
        admitted_proposal,
        artifact,
    })
}

/// Domain-separated prompt digest.
#[must_use]
pub fn prompt_digest(prompt: &str) -> DigestBytes {
    digest_bytes(PROMPT_DOMAIN, prompt.as_bytes())
}

/// Domain-separated captured-response digest.
#[must_use]
pub fn response_digest(response: &str) -> DigestBytes {
    digest_bytes(RESPONSE_DOMAIN, response.as_bytes())
}

/// Domain-separated interaction-schema label digest.
#[must_use]
pub fn interaction_schema_digest(schema: &str) -> DigestBytes {
    digest_bytes(INTERACTION_SCHEMA_DOMAIN, schema.as_bytes())
}

/// Domain-separated capture-policy label digest.
#[must_use]
pub fn capture_policy_digest(policy: &str) -> DigestBytes {
    digest_bytes(CAPTURE_POLICY_DOMAIN, policy.as_bytes())
}

/// Domain-separated record-schema label digest.
#[must_use]
pub fn record_schema_digest(schema: &str) -> DigestBytes {
    digest_bytes(RECORD_SCHEMA_DOMAIN, schema.as_bytes())
}

/// Domain-separated record-policy label digest.
#[must_use]
pub fn record_policy_digest(policy: &str) -> DigestBytes {
    digest_bytes(RECORD_POLICY_DOMAIN, policy.as_bytes())
}

fn validate_interaction(
    canonical: &str,
) -> Result<(ReceiptEnvelope, ProviderInteractionPayload), VerifiableAiRecordError> {
    require_canonical(canonical)?;
    let envelope: ReceiptEnvelope = serde_json::from_str(canonical)?;
    verify_envelope_hash(&envelope)?;
    let payload: ProviderInteractionPayload =
        serde_json::from_value(envelope.payload.as_value().clone())?;
    if payload.payload_kind != PROVIDER_INTERACTION_PAYLOAD_KIND
        || payload.schema != PROVIDER_INTERACTION_SCHEMA
        || envelope.receipt_type != ReceiptType::Llm
        || envelope.boundary_origin != Some(BoundaryOrigin::Adapter)
    {
        return Err(VerifiableAiRecordError::UnsupportedInteractionSchema);
    }
    if payload.request.prompt_digest != prompt_digest(&payload.request.prompt)
        || payload.response.response_digest != response_digest(&payload.response.text)
        || envelope.context_digest != payload.request.prompt_digest
        || envelope.schema_digest != interaction_schema_digest(&payload.schema)
        || envelope.policy_digest != capture_policy_digest(&payload.capture_policy_version)
    {
        return Err(VerifiableAiRecordError::InteractionBindingMismatch);
    }
    Ok((envelope, payload))
}

fn validate_proposal_admission(
    canonical: &str,
) -> Result<
    (
        ReceiptEnvelope,
        ReceiptEnvelope,
        DigestBytes,
        AdmittedProposal,
    ),
    VerifiableAiRecordError,
> {
    require_canonical(canonical)?;
    let bundle: ProposalAdmissionBundle = serde_json::from_str(canonical)?;
    if bundle.format != PROPOSAL_ADMISSION_BUNDLE_FORMAT {
        return Err(VerifiableAiRecordError::UnsupportedProposalAdmissionSchema);
    }
    require_canonical(&bundle.proposal_canonical)?;
    require_canonical(&bundle.admission_canonical)?;
    let proposal: ReceiptEnvelope = serde_json::from_str(&bundle.proposal_canonical)?;
    let admission: ReceiptEnvelope = serde_json::from_str(&bundle.admission_canonical)?;
    verify_envelope_hash(&proposal)?;
    verify_envelope_hash(&admission)?;
    let proposal_payload: AgentProposalReceiptPayload =
        serde_json::from_value(proposal.payload.as_value().clone())?;
    let admission_payload: AdmissionReceiptPayload =
        serde_json::from_value(admission.payload.as_value().clone())?;
    if proposal_payload.payload_kind != AGENT_PROPOSAL_PAYLOAD_KIND
        || proposal_payload.schema != AGENT_PROPOSAL_SCHEMA
        || admission_payload.payload_kind != PROPOSAL_ADMISSION_PAYLOAD_KIND
        || admission_payload.schema != PROPOSAL_ADMISSION_SCHEMA
        || admission.parent_id != Some(proposal.event_hash)
    {
        return Err(VerifiableAiRecordError::UnsupportedProposalAdmissionSchema);
    }
    let expected = vr_proposal_admission::admit_proposal(&proposal, &admission_payload.signal)
        .map_err(|error| VerifiableAiRecordError::ProposalAdmission(error.to_string()))?;
    if expected.admission_receipt() != &admission
        || expected.admitted_proposal() != &bundle.admitted_proposal
        || expected.admitted_proposal_digest() != bundle.admitted_proposal_digest
    {
        return Err(VerifiableAiRecordError::ProposalAdmissionMismatch);
    }
    Ok((
        proposal,
        admission,
        bundle.admitted_proposal_digest,
        bundle.admitted_proposal,
    ))
}

fn seal_record_envelope(
    payload: &VerifiableAiRecordPayload,
) -> Result<ReceiptEnvelope, VerifiableAiRecordError> {
    let mut envelope: ReceiptEnvelope = serde_json::from_value(serde_json::json!({
        "envelope_version": SchemaVersion::V1,
        "receipt_type": ReceiptType::Event,
        "context_digest": payload.source_interaction_digest,
        "schema_digest": record_schema_digest(VERIFIABLE_AI_RECORD_SCHEMA),
        "policy_digest": record_policy_digest(VERIFIABLE_AI_RECORD_POLICY),
        "logical_time": "3",
        "event_hash": DigestBytes::from_array([0; 32]),
        "parent_id": payload.admission_receipt_digest,
        "boundary_origin": BoundaryOrigin::Governance,
        "digest_algorithm": SchemaVersion::V1.digest_algorithm(),
        "canonicalization": SchemaVersion::V1.canonicalization(),
        "payload": payload,
    }))?;
    envelope.event_hash = vr_receipt_identity::compute_event_hash(&envelope)
        .map_err(|error| VerifiableAiRecordError::Receipt(error.to_string()))?;
    Ok(envelope)
}

fn verify_envelope_hash(envelope: &ReceiptEnvelope) -> Result<(), VerifiableAiRecordError> {
    let recomputed = vr_receipt_identity::compute_event_hash(envelope)
        .map_err(|error| VerifiableAiRecordError::Receipt(error.to_string()))?;
    if recomputed == envelope.event_hash {
        Ok(())
    } else {
        Err(VerifiableAiRecordError::ReceiptDigestMismatch)
    }
}

fn require_canonical(value: &str) -> Result<(), VerifiableAiRecordError> {
    if vr_jcs::to_canon_bytes_from_slice(value.as_bytes())? == value.as_bytes() {
        Ok(())
    } else {
        Err(VerifiableAiRecordError::NonCanonical)
    }
}

fn canonical_string<T: serde::Serialize>(value: &T) -> Result<String, VerifiableAiRecordError> {
    let encoded = serde_json::to_vec(value)?;
    let canonical = vr_jcs::to_canon_bytes_from_slice(&encoded)?;
    String::from_utf8(canonical).map_err(|_| VerifiableAiRecordError::NonUtf8Canonical)
}

fn digest_bytes(domain: &str, bytes: &[u8]) -> DigestBytes {
    let mut hasher = blake3::Hasher::new_derive_key(domain);
    hasher.update(bytes);
    DigestBytes::from_array(*hasher.finalize().as_bytes())
}

/// Deterministic record-sealing failures.
#[derive(Debug, thiserror::Error)]
pub enum VerifiableAiRecordError {
    /// Artifact bytes are valid JSON but not their canonical representation.
    #[error("artifact is not JCS-canonical")]
    NonCanonical,
    /// Canonical JSON unexpectedly was not UTF-8.
    #[error("canonical artifact is not UTF-8")]
    NonUtf8Canonical,
    /// Receipt self-commitment failed.
    #[error("receipt digest mismatch")]
    ReceiptDigestMismatch,
    /// Provider-interaction schema/boundary is unsupported.
    #[error("unsupported provider-interaction schema")]
    UnsupportedInteractionSchema,
    /// Provider-interaction leaf/schema/policy binding failed.
    #[error("provider-interaction binding mismatch")]
    InteractionBindingMismatch,
    /// Proposal/admission format or schemas are unsupported.
    #[error("unsupported proposal/admission schema")]
    UnsupportedProposalAdmissionSchema,
    /// Proposal/admission deterministic reconstruction failed.
    #[error("proposal/admission mismatch")]
    ProposalAdmissionMismatch,
    /// Proposal references different interaction receipts.
    #[error("proposal interaction lineage mismatch")]
    InteractionLineageMismatch,
    /// Shared proposal-admission law rejected the bundle.
    #[error("proposal/admission reconstruction failed: {0}")]
    ProposalAdmission(String),
    /// JSON projection failed.
    #[error("JSON projection failed: {0}")]
    Json(#[from] serde_json::Error),
    /// JCS canonicalization failed.
    #[error("canonicalization failed: {0}")]
    Jcs(#[from] vr_jcs::JcsError),
    /// Constitutional receipt identity failed.
    #[error("receipt identity failed: {0}")]
    Receipt(String),
}

#[cfg(test)]
mod tests;
