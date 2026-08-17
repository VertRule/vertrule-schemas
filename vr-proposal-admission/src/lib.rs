//! Deterministic text-claim proposal/admission law (ADR-049).
//!
//! This crate owns canonical identity, receipt projection, validation of an
//! external approval signal, and the sole authority-bearing
//! [`SealedAdmittedProposal`] constructor. It performs no I/O, policy
//! evaluation, action authorization, actuation, clock read, or randomness.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

use std::collections::BTreeSet;

use vertrule_schemas::{
    AdmissionReceiptPayload, AdmittedClaim, AdmittedClaimOperation, AdmittedProposal,
    AgentProposalReceiptPayload, AttestationPurpose, BoundaryOrigin, ClaimAdmissionDecision,
    DigestBytes, ExternalAdmissionSignal, ProposalAdmissionBundle, ProposedTextClaim,
    ReceiptEnvelope, ReceiptType, RejectedClaim, SchemaVersion, TextClaimAgentProposal,
    AGENT_PROPOSAL_PAYLOAD_KIND, AGENT_PROPOSAL_SCHEMA, PROPOSAL_ADMISSION_BUNDLE_FORMAT,
    PROPOSAL_ADMISSION_PAYLOAD_KIND, PROPOSAL_ADMISSION_SCHEMA,
};
use vr_jcs::DigestStrategy;

/// Frozen BLAKE3 domain for one proposed text claim.
pub const PROPOSED_TEXT_CLAIM_DOMAIN: &str = "vertrule.proposed-text-claim.v1";
/// Frozen BLAKE3 domain for an agent proposal.
pub const AGENT_PROPOSAL_DOMAIN: &str = "vertrule.agent-proposal.v1";
/// Frozen BLAKE3 domain for an external admission signal.
pub const ADMISSION_SIGNAL_DOMAIN: &str = "vertrule.admission-signal.v1";
/// Frozen BLAKE3 domain for an admitted proposal.
pub const ADMITTED_PROPOSAL_DOMAIN: &str = "vertrule.admitted-proposal.v1";
/// Frozen schema-binding domain for proposal receipts.
pub const AGENT_PROPOSAL_SCHEMA_DOMAIN: &str = "vertrule.agent-proposal-schema.v1";
/// Frozen schema-binding domain for admission receipts.
pub const PROPOSAL_ADMISSION_SCHEMA_DOMAIN: &str = "vertrule.proposal-admission-schema.v1";
/// Frozen policy-binding domain for proposal receipts.
pub const AGENT_PROPOSAL_POLICY_DOMAIN: &str = "vertrule.agent-proposal-policy.v1";
/// Frozen policy-binding domain for admission receipts.
pub const PROPOSAL_ADMISSION_POLICY_DOMAIN: &str = "vertrule.proposal-admission-policy.v1";
/// Frozen proposal projection rule identity.
pub const AGENT_PROPOSAL_POLICY: &str = "vr.workflow.agent_proposal_rule@0.1";
/// Frozen admission transition rule identity.
pub const PROPOSAL_ADMISSION_POLICY: &str = "vr.workflow.proposal_admission_rule@0.1";

const MAX_CLAIMS: usize = 16;
const MAX_CLAIM_BYTES: usize = 4 * 1024;
const MAX_ACTOR_ASSERTION_BYTES: usize = 256;

/// Proposal receipt plus its exact canonical serialization.
#[derive(Debug, Clone)]
pub struct SealedAgentProposal {
    envelope: ReceiptEnvelope,
    canonical: String,
}

impl SealedAgentProposal {
    /// Borrow the proposal receipt.
    #[must_use]
    pub const fn envelope(&self) -> &ReceiptEnvelope {
        &self.envelope
    }

    /// Borrow exact JCS-canonical receipt bytes as UTF-8.
    #[must_use]
    pub fn canonical(&self) -> &str {
        &self.canonical
    }
}

/// Authority-bearing admitted proposal. No public unchecked constructor.
#[derive(Debug, Clone)]
pub struct SealedAdmittedProposal {
    proposal_canonical: String,
    admission_envelope: ReceiptEnvelope,
    admission_canonical: String,
    admitted_proposal: AdmittedProposal,
    admitted_proposal_digest: DigestBytes,
}

impl SealedAdmittedProposal {
    /// Borrow the admitted proposal wire projection.
    #[must_use]
    pub const fn admitted_proposal(&self) -> &AdmittedProposal {
        &self.admitted_proposal
    }

    /// Borrow the admission receipt.
    #[must_use]
    pub const fn admission_receipt(&self) -> &ReceiptEnvelope {
        &self.admission_envelope
    }

    /// Domain-separated admitted-proposal identity.
    #[must_use]
    pub const fn admitted_proposal_digest(&self) -> DigestBytes {
        self.admitted_proposal_digest
    }

    /// Build the portable closure verified by `vertrule-verifier`.
    #[must_use]
    pub fn bundle(&self) -> ProposalAdmissionBundle {
        ProposalAdmissionBundle {
            format: PROPOSAL_ADMISSION_BUNDLE_FORMAT.to_owned(),
            proposal_canonical: self.proposal_canonical.clone(),
            admission_canonical: self.admission_canonical.clone(),
            admitted_proposal: self.admitted_proposal.clone(),
            admitted_proposal_digest: self.admitted_proposal_digest,
        }
    }

    /// Serialize the complete proposal/admission bundle as exact JCS UTF-8.
    ///
    /// # Errors
    ///
    /// Returns a typed error if serialization or canonicalization fails.
    pub fn artifact(&self) -> Result<String, ProposalAdmissionError> {
        let encoded = serde_json::to_vec(&self.bundle())?;
        let canonical = vr_jcs::to_canon_bytes_from_slice(&encoded)?;
        String::from_utf8(canonical).map_err(|_| ProposalAdmissionError::NonUtf8Canonical)
    }
}

/// Seal a non-authoritative proposal into an identity-bearing receipt.
///
/// # Errors
///
/// Returns a typed error for invalid claims, canonicalization, or receipt
/// projection failure.
pub fn seal_proposal(
    proposal: &TextClaimAgentProposal,
) -> Result<SealedAgentProposal, ProposalAdmissionError> {
    validate_proposal(proposal)?;
    let proposal_digest = agent_proposal_digest(proposal)?;
    let payload = AgentProposalReceiptPayload {
        payload_kind: AGENT_PROPOSAL_PAYLOAD_KIND.to_owned(),
        schema: AGENT_PROPOSAL_SCHEMA.to_owned(),
        proposal_digest,
        proposal: proposal.clone(),
    };
    let envelope = seal_envelope(
        ReceiptType::Llm,
        BoundaryOrigin::Model,
        proposal.source_interaction_digest,
        digest_label(AGENT_PROPOSAL_SCHEMA_DOMAIN, AGENT_PROPOSAL_SCHEMA)?,
        digest_label(AGENT_PROPOSAL_POLICY_DOMAIN, AGENT_PROPOSAL_POLICY)?,
        1,
        None,
        serde_json::to_value(payload)?,
    )?;
    let canonical = canonical_envelope(&envelope)?;
    Ok(SealedAgentProposal {
        envelope,
        canonical,
    })
}

/// Consume a qualifying external approval signal and deterministically admit
/// or reject every proposed claim.
///
/// # Errors
///
/// Returns a typed denial for invalid proposal receipt, purpose/subject/context
/// substitution, incomplete decisions, invalid edits, or receipt failure.
pub fn admit_proposal(
    proposal_receipt: &ReceiptEnvelope,
    signal: &ExternalAdmissionSignal,
) -> Result<SealedAdmittedProposal, ProposalAdmissionError> {
    let payload = validate_proposal_receipt(proposal_receipt)?;
    validate_signal(proposal_receipt, &payload.proposal, signal)?;
    let (admitted_claims, rejected_claims) = derive_outcome(&payload.proposal, signal)?;
    let signal_digest = admission_signal_digest(signal)?;
    let admission_payload = AdmissionReceiptPayload {
        payload_kind: PROPOSAL_ADMISSION_PAYLOAD_KIND.to_owned(),
        schema: PROPOSAL_ADMISSION_SCHEMA.to_owned(),
        proposal_receipt_digest: proposal_receipt.event_hash,
        context_digest: proposal_receipt.context_digest,
        admission_signal_digest: signal_digest,
        signal: signal.clone(),
        admitted_claims: admitted_claims.clone(),
        rejected_claims: rejected_claims.clone(),
    };
    let admission_envelope = seal_envelope(
        ReceiptType::Governance,
        BoundaryOrigin::Governance,
        proposal_receipt.context_digest,
        digest_label(PROPOSAL_ADMISSION_SCHEMA_DOMAIN, PROPOSAL_ADMISSION_SCHEMA)?,
        digest_label(PROPOSAL_ADMISSION_POLICY_DOMAIN, PROPOSAL_ADMISSION_POLICY)?,
        2,
        Some(proposal_receipt.event_hash),
        serde_json::to_value(admission_payload)?,
    )?;
    let admitted_proposal = AdmittedProposal {
        proposal_receipt_digest: proposal_receipt.event_hash,
        admission_receipt_digest: admission_envelope.event_hash,
        claims: admitted_claims,
        rejected_claims,
    };
    let admitted_proposal_digest = admitted_proposal_digest(&admitted_proposal)?;

    Ok(SealedAdmittedProposal {
        proposal_canonical: canonical_envelope(proposal_receipt)?,
        admission_canonical: canonical_envelope(&admission_envelope)?,
        admission_envelope,
        admitted_proposal,
        admitted_proposal_digest,
    })
}

/// Domain-separated identity of one proposed claim.
///
/// # Errors
///
/// Returns an error if canonicalization fails.
pub fn proposed_claim_digest(
    claim: &ProposedTextClaim,
) -> Result<DigestBytes, ProposalAdmissionError> {
    digest_typed(PROPOSED_TEXT_CLAIM_DOMAIN, claim)
}

/// Domain-separated identity of a complete agent proposal.
///
/// # Errors
///
/// Returns an error if canonicalization fails.
pub fn agent_proposal_digest(
    proposal: &TextClaimAgentProposal,
) -> Result<DigestBytes, ProposalAdmissionError> {
    digest_typed(AGENT_PROPOSAL_DOMAIN, proposal)
}

/// Domain-separated identity of an external admission signal.
///
/// # Errors
///
/// Returns an error if canonicalization fails.
pub fn admission_signal_digest(
    signal: &ExternalAdmissionSignal,
) -> Result<DigestBytes, ProposalAdmissionError> {
    digest_typed(ADMISSION_SIGNAL_DOMAIN, signal)
}

/// Domain-separated identity of an admitted proposal.
///
/// # Errors
///
/// Returns an error if canonicalization fails.
pub fn admitted_proposal_digest(
    proposal: &AdmittedProposal,
) -> Result<DigestBytes, ProposalAdmissionError> {
    digest_typed(ADMITTED_PROPOSAL_DOMAIN, proposal)
}

/// Check whether an attestation purpose matches the required semantic axis.
#[must_use]
pub const fn purpose_matches(actual: AttestationPurpose, required: AttestationPurpose) -> bool {
    matches!(
        (actual, required),
        (
            AttestationPurpose::ProposalApproval,
            AttestationPurpose::ProposalApproval
        ) | (
            AttestationPurpose::ActionAuthorization,
            AttestationPurpose::ActionAuthorization
        )
    )
}

/// Recompute the admitted/rejected partition from proposal plus signal.
///
/// Used by the independent verifier; it confers no authority by itself.
///
/// # Errors
///
/// Returns a typed error for incomplete/duplicate decisions or invalid edits.
pub fn derive_outcome(
    proposal: &TextClaimAgentProposal,
    signal: &ExternalAdmissionSignal,
) -> Result<(Vec<AdmittedClaim>, Vec<RejectedClaim>), ProposalAdmissionError> {
    if signal.decisions.len() != proposal.claims.len() {
        return Err(ProposalAdmissionError::AdmissionIncomplete);
    }
    let mut seen = BTreeSet::new();
    let mut admitted = Vec::new();
    let mut rejected = Vec::new();
    for decision in &signal.decisions {
        let ordinal = decision.claim_ordinal();
        if !seen.insert(ordinal) {
            return Err(ProposalAdmissionError::AdmissionDecisionInvalid);
        }
        let claim = proposal
            .claims
            .iter()
            .find(|candidate| candidate.ordinal == ordinal)
            .ok_or(ProposalAdmissionError::AdmissionDecisionInvalid)?;
        let claim_digest = proposed_claim_digest(claim)?;
        match decision {
            ClaimAdmissionDecision::Approve { .. } => admitted.push(AdmittedClaim {
                ordinal,
                text: claim.text.clone(),
                proposed_claim_digest: claim_digest,
                operation: AdmittedClaimOperation::Approve,
            }),
            ClaimAdmissionDecision::Edit { admitted_text, .. } => {
                validate_text(admitted_text)?;
                admitted.push(AdmittedClaim {
                    ordinal,
                    text: admitted_text.clone(),
                    proposed_claim_digest: claim_digest,
                    operation: AdmittedClaimOperation::Edit,
                });
            }
            ClaimAdmissionDecision::Reject { reason, .. } => rejected.push(RejectedClaim {
                ordinal,
                proposed_claim_digest: claim_digest,
                reason: *reason,
            }),
        }
    }
    admitted.sort_by_key(|claim| claim.ordinal);
    rejected.sort_by_key(|claim| claim.ordinal);
    Ok((admitted, rejected))
}

fn validate_signal(
    proposal_receipt: &ReceiptEnvelope,
    proposal: &TextClaimAgentProposal,
    signal: &ExternalAdmissionSignal,
) -> Result<(), ProposalAdmissionError> {
    if !purpose_matches(signal.purpose, AttestationPurpose::ProposalApproval) {
        return Err(ProposalAdmissionError::AttestationPurposeMismatch);
    }
    if signal.subject_proposal_receipt_digest != proposal_receipt.event_hash {
        return Err(ProposalAdmissionError::ProposalSubjectMismatch);
    }
    if signal.context_digest != proposal_receipt.context_digest
        || signal.context_digest != proposal.source_interaction_digest
    {
        return Err(ProposalAdmissionError::AdmissionContextMismatch);
    }
    if signal.actor_assertion.trim().is_empty()
        || signal.actor_assertion.len() > MAX_ACTOR_ASSERTION_BYTES
    {
        return Err(ProposalAdmissionError::AdmissionSignalInvalid);
    }
    derive_outcome(proposal, signal).map(|_| ())
}

fn validate_proposal_receipt(
    envelope: &ReceiptEnvelope,
) -> Result<AgentProposalReceiptPayload, ProposalAdmissionError> {
    if envelope.receipt_type != ReceiptType::Llm
        || envelope.boundary_origin != Some(BoundaryOrigin::Model)
    {
        return Err(ProposalAdmissionError::UnsupportedProposalSchema);
    }
    if vr_receipt_identity::compute_event_hash(envelope)
        .map_err(|error| ProposalAdmissionError::Receipt(error.to_string()))?
        != envelope.event_hash
    {
        return Err(ProposalAdmissionError::ProposalReceiptDigestMismatch);
    }
    let payload: AgentProposalReceiptPayload =
        serde_json::from_value(envelope.payload.as_value().clone())?;
    if payload.payload_kind != AGENT_PROPOSAL_PAYLOAD_KIND
        || payload.schema != AGENT_PROPOSAL_SCHEMA
    {
        return Err(ProposalAdmissionError::UnsupportedProposalSchema);
    }
    validate_proposal(&payload.proposal)?;
    if payload.proposal_digest != agent_proposal_digest(&payload.proposal)? {
        return Err(ProposalAdmissionError::ProposalDigestMismatch);
    }
    if envelope.context_digest != payload.proposal.source_interaction_digest {
        return Err(ProposalAdmissionError::AdmissionContextMismatch);
    }
    Ok(payload)
}

fn validate_proposal(proposal: &TextClaimAgentProposal) -> Result<(), ProposalAdmissionError> {
    if proposal.claims.len() > MAX_CLAIMS {
        return Err(ProposalAdmissionError::TooManyClaims);
    }
    for (index, claim) in proposal.claims.iter().enumerate() {
        let expected = u64::try_from(index + 1)
            .map_err(|_| ProposalAdmissionError::AdmissionDecisionInvalid)?;
        if claim.ordinal.get() != expected {
            return Err(ProposalAdmissionError::ClaimOrderInvalid);
        }
        validate_text(&claim.text)?;
    }
    Ok(())
}

fn validate_text(text: &str) -> Result<(), ProposalAdmissionError> {
    if text.trim().is_empty() || text.len() > MAX_CLAIM_BYTES {
        Err(ProposalAdmissionError::ClaimTextInvalid)
    } else {
        Ok(())
    }
}

fn seal_envelope(
    receipt_type: ReceiptType,
    boundary_origin: BoundaryOrigin,
    context_digest: DigestBytes,
    schema_digest: DigestBytes,
    policy_digest: DigestBytes,
    logical_time: u64,
    parent_id: Option<DigestBytes>,
    payload: serde_json::Value,
) -> Result<ReceiptEnvelope, ProposalAdmissionError> {
    let mut value = serde_json::json!({
        "envelope_version": SchemaVersion::V1,
        "receipt_type": receipt_type,
        "context_digest": context_digest,
        "schema_digest": schema_digest,
        "policy_digest": policy_digest,
        "logical_time": logical_time.to_string(),
        "event_hash": DigestBytes::from_array([0; 32]),
        "boundary_origin": boundary_origin,
        "digest_algorithm": SchemaVersion::V1.digest_algorithm(),
        "canonicalization": SchemaVersion::V1.canonicalization(),
        "payload": payload,
    });
    if let Some(parent) = parent_id {
        value["parent_id"] = serde_json::to_value(parent)?;
    }
    let mut envelope: ReceiptEnvelope = serde_json::from_value(value)?;
    envelope.event_hash = vr_receipt_identity::compute_event_hash(&envelope)
        .map_err(|error| ProposalAdmissionError::Receipt(error.to_string()))?;
    Ok(envelope)
}

fn canonical_envelope(envelope: &ReceiptEnvelope) -> Result<String, ProposalAdmissionError> {
    let bytes = serde_json::to_vec(envelope)?;
    let canonical = vr_jcs::to_canon_bytes_from_slice(&bytes)?;
    String::from_utf8(canonical).map_err(|_| ProposalAdmissionError::NonUtf8Canonical)
}

fn digest_label(domain: &str, label: &str) -> Result<DigestBytes, ProposalAdmissionError> {
    digest_typed(domain, &label)
}

fn digest_typed<T: serde::Serialize>(
    domain: &str,
    value: &T,
) -> Result<DigestBytes, ProposalAdmissionError> {
    let json = serde_json::to_value(value)?;
    let digest =
        vr_jcs::to_canon_digest_with(&json, &DigestStrategy::blake3_domain_separated(domain))?;
    DigestBytes::from_slice(&digest.bytes).map_err(ProposalAdmissionError::from)
}

/// Deterministic proposal/admission failures.
#[derive(Debug, thiserror::Error)]
pub enum ProposalAdmissionError {
    /// Proposal payload/schema is unsupported.
    #[error("unsupported proposal schema")]
    UnsupportedProposalSchema,
    /// Proposal receipt self-commitment failed.
    #[error("proposal receipt digest mismatch")]
    ProposalReceiptDigestMismatch,
    /// Proposal payload digest failed.
    #[error("proposal digest mismatch")]
    ProposalDigestMismatch,
    /// External signal used the wrong semantic purpose.
    #[error("attestation purpose mismatch")]
    AttestationPurposeMismatch,
    /// External signal names another proposal receipt.
    #[error("proposal subject mismatch")]
    ProposalSubjectMismatch,
    /// External signal and proposal context differ.
    #[error("admission context mismatch")]
    AdmissionContextMismatch,
    /// External signal actor assertion is invalid.
    #[error("admission signal is invalid")]
    AdmissionSignalInvalid,
    /// Not every proposed claim received exactly one decision.
    #[error("admission is incomplete")]
    AdmissionIncomplete,
    /// Claim decision is duplicate or references no proposed claim.
    #[error("admission decision is invalid")]
    AdmissionDecisionInvalid,
    /// Proposed claim ordering is invalid.
    #[error("proposed claim order is invalid")]
    ClaimOrderInvalid,
    /// Proposed/edited claim text is invalid.
    #[error("claim text is invalid")]
    ClaimTextInvalid,
    /// Claim-count limit exceeded.
    #[error("too many proposed claims")]
    TooManyClaims,
    /// Canonical output unexpectedly was not UTF-8.
    #[error("canonical artifact is not UTF-8")]
    NonUtf8Canonical,
    /// JSON projection failed.
    #[error("JSON projection failed: {0}")]
    Json(#[from] serde_json::Error),
    /// JCS canonicalization/digest failed.
    #[error("canonicalization failed: {0}")]
    Jcs(#[from] vr_jcs::JcsError),
    /// Shared schema validation failed.
    #[error("schema validation failed: {0}")]
    Schema(#[from] vertrule_schemas::DefinitionError),
    /// Constitutional receipt identity failed.
    #[error("receipt identity failed: {0}")]
    Receipt(String),
}

#[cfg(test)]
mod tests;
