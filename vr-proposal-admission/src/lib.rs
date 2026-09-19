//! Deterministic text-claim proposal/admission law (ADR-049 under ADR-056).
//!
//! This crate owns the domain-object identities, the admission transition
//! (`derive_outcome`), the signal and proposal shape laws, and the sole
//! authority-bearing V2 constructors [`seal_proposal_v2`] and
//! [`admit_proposal_v2`]. It performs no I/O, policy evaluation, action
//! authorization, actuation, clock read, or randomness.
//!
//! The V1 (`@0.1`) mint is `MintDenied` (ADR-056 R12; M2-0 D4): no public
//! item returns a V1 envelope. What survives of the V1 construction is the
//! verify-only [`reconstruct_admission_v1`], which rebuilds a historical
//! admission envelope in memory to judge a presented one and returns only
//! the [`AdmittedProposal`] projection, its identity and the comparison —
//! `VerifyConstructionAllowed ⇏ MintConstructionAllowed`.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

use std::collections::BTreeSet;

use vertrule_schemas::{
    AdmissionReceiptPayload, AdmittedClaim, AdmittedClaimOperation, AdmittedProposal,
    AgentProposalPayloadV2, AgentProposalReceiptPayload, AttestationPurpose, BoundaryOrigin,
    CanonicalPayload, ClaimAdmissionDecision, DigestBytes, ExternalAdmissionSignal,
    PayloadSchemaV2, ProposalAdmissionBundleV2, ProposalAdmissionPayloadV2, ProposedTextClaim,
    ReceiptEnvelope, ReceiptEnvelopeV2, ReceiptType, ReceiptTypeV2, RejectedClaim, SchemaVersion,
    TextClaimAgentProposal, AGENT_PROPOSAL_PAYLOAD_KIND, AGENT_PROPOSAL_SCHEMA,
    PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2, PROPOSAL_ADMISSION_PAYLOAD_KIND,
    PROPOSAL_ADMISSION_SCHEMA,
};
use vr_jcs::DigestStrategy;
use vr_receipt_identity::{seal_receipt_v2, ReceiptV2Draft};

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

/// Authority-bearing outcome of one V2 admission transition. No public
/// unchecked constructor.
#[derive(Debug, Clone)]
pub struct SealedAdmittedProposalV2 {
    proposal: ReceiptEnvelopeV2,
    admission: ReceiptEnvelopeV2,
    admitted_proposal: AdmittedProposal,
    admitted_proposal_digest: DigestBytes,
}

impl SealedAdmittedProposalV2 {
    /// Borrow the `vr.workflow.agent_proposal` receipt that was admitted.
    #[must_use]
    pub const fn proposal_receipt(&self) -> &ReceiptEnvelopeV2 {
        &self.proposal
    }

    /// Borrow the `vr.workflow.proposal_admission` receipt.
    #[must_use]
    pub const fn admission_receipt(&self) -> &ReceiptEnvelopeV2 {
        &self.admission
    }

    /// Borrow the admitted proposal wire projection.
    #[must_use]
    pub const fn admitted_proposal(&self) -> &AdmittedProposal {
        &self.admitted_proposal
    }

    /// Domain-separated admitted-proposal identity.
    #[must_use]
    pub const fn admitted_proposal_digest(&self) -> DigestBytes {
        self.admitted_proposal_digest
    }

    /// The portable evidence-set presentation of this transition: the two
    /// receipts and nothing else.
    #[must_use]
    pub fn bundle(&self) -> ProposalAdmissionBundleV2 {
        ProposalAdmissionBundleV2 {
            format: PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2.to_owned(),
            proposal: self.proposal.clone(),
            admission: self.admission.clone(),
        }
    }
}

/// Seal a non-authoritative proposal into a `vr.workflow.agent_proposal`
/// receipt (`vr.workflow.agent_proposal@0.2`).
///
/// # Errors
///
/// Returns a typed error for an invalid proposal shape, canonicalization,
/// or receipt formation failure.
pub fn seal_proposal_v2(
    proposal: &TextClaimAgentProposal,
) -> Result<ReceiptEnvelopeV2, ProposalAdmissionError> {
    validate_proposal(proposal)?;
    let payload = AgentProposalPayloadV2 {
        schema: PayloadSchemaV2::VR_WORKFLOW_AGENT_PROPOSAL_0_2
            .label()
            .to_owned(),
        proposal_digest: agent_proposal_digest(proposal)?,
        proposal: proposal.clone(),
    };
    seal_v2(
        ReceiptTypeV2::WorkflowAgentProposal,
        PayloadSchemaV2::VR_WORKFLOW_AGENT_PROPOSAL_0_2,
        &payload,
    )
}

/// Consume a qualifying external approval signal over a sealed V2 proposal
/// receipt and deterministically admit or reject every proposed claim,
/// sealing a `vr.workflow.proposal_admission` receipt
/// (`vr.workflow.proposal_admission@0.2`).
///
/// The proposal receipt is checked structurally (type, shape, proposal
/// identity); its full law is the verifier's. The signal must name this
/// receipt's `receipt_digest` as subject and the proposal's
/// `source_interaction_digest` as context.
///
/// # Errors
///
/// Returns a typed denial for an unsupported proposal receipt,
/// purpose/subject/context substitution, incomplete decisions, invalid
/// edits, or receipt formation failure.
pub fn admit_proposal_v2(
    proposal_receipt: &ReceiptEnvelopeV2,
    signal: &ExternalAdmissionSignal,
) -> Result<SealedAdmittedProposalV2, ProposalAdmissionError> {
    let payload = validate_proposal_receipt_v2(proposal_receipt)?;
    validate_signal_v2(proposal_receipt.receipt_digest, &payload.proposal, signal)?;
    let (admitted_claims, rejected_claims) = derive_outcome(&payload.proposal, signal)?;
    let admission_payload = ProposalAdmissionPayloadV2 {
        schema: PayloadSchemaV2::VR_WORKFLOW_PROPOSAL_ADMISSION_0_2
            .label()
            .to_owned(),
        proposal_receipt_digest: proposal_receipt.receipt_digest,
        admission_signal_digest: admission_signal_digest(signal)?,
        signal: signal.clone(),
        admitted_claims: admitted_claims.clone(),
        rejected_claims: rejected_claims.clone(),
    };
    let admission = seal_v2(
        ReceiptTypeV2::WorkflowProposalAdmission,
        PayloadSchemaV2::VR_WORKFLOW_PROPOSAL_ADMISSION_0_2,
        &admission_payload,
    )?;
    let admitted_proposal = AdmittedProposal {
        proposal_receipt_digest: proposal_receipt.receipt_digest,
        admission_receipt_digest: admission.receipt_digest,
        claims: admitted_claims,
        rejected_claims,
    };
    let admitted_proposal_digest = admitted_proposal_digest(&admitted_proposal)?;
    Ok(SealedAdmittedProposalV2 {
        proposal: proposal_receipt.clone(),
        admission,
        admitted_proposal,
        admitted_proposal_digest,
    })
}

/// What verify-only reconstruction of a historical (V1) admission yields.
///
/// Carries no envelope: the admission rebuilt in memory is compared to the
/// presented one and only the comparison escapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconstructedAdmissionV1 {
    /// The admitted proposal projection the transition yields.
    pub admitted_proposal: AdmittedProposal,
    /// Domain-separated identity of `admitted_proposal`.
    pub admitted_proposal_digest: DigestBytes,
    /// Whether the admission envelope rebuilt from the proposal and the
    /// presented signal is byte-for-byte the presented admission envelope.
    pub admission_matches_presented: bool,
}

/// Verify-only reconstruction of a historical `@0.1` admission (ADR-054 E2
/// `VerifyAllowed`; ADR-056 R12 `MintDenied`; M2-0 D4).
///
/// Re-runs the V1 admission law over `proposal_receipt` and the signal the
/// presented admission committed, rebuilds the V1 admission envelope in
/// memory, and reports whether it equals `presented_admission`. The rebuilt
/// envelope never escapes; this function confers no minting authority.
///
/// # Errors
///
/// Returns the typed V1 denial for an invalid proposal receipt, a
/// malformed admission payload, purpose/subject/context substitution,
/// incomplete decisions, invalid edits, or a formation failure.
pub fn reconstruct_admission_v1(
    proposal_receipt: &ReceiptEnvelope,
    presented_admission: &ReceiptEnvelope,
) -> Result<ReconstructedAdmissionV1, ProposalAdmissionError> {
    let payload = validate_proposal_receipt(proposal_receipt)?;
    let presented: AdmissionReceiptPayload =
        serde_json::from_value(presented_admission.payload.as_value().clone())?;
    let signal = &presented.signal;
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
    let rebuilt = rebuild_admission_envelope_v1(proposal_receipt, &admission_payload)?;
    let admitted_proposal = AdmittedProposal {
        proposal_receipt_digest: proposal_receipt.event_hash,
        admission_receipt_digest: rebuilt.event_hash,
        claims: admitted_claims,
        rejected_claims,
    };
    let admitted_proposal_digest = admitted_proposal_digest(&admitted_proposal)?;
    Ok(ReconstructedAdmissionV1 {
        admitted_proposal,
        admitted_proposal_digest,
        admission_matches_presented: rebuilt == *presented_admission,
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

/// Validate an external approval signal against the V2 proposal receipt it
/// names, without a V1 envelope (ADR-056; registry row P3).
///
/// The signal's subject must be `proposal_receipt_digest` (the V2
/// `receipt_digest` of the proposal receipt), its context the proposal's
/// `source_interaction_digest` (the one authoritative representation of the
/// review context, M2-0 D3), its purpose `proposal_approval`, its actor
/// assertion non-blank and bounded, and its decisions must derive a
/// complete outcome over `proposal`. Shared by the V2 producer and the
/// verifier's `outcome_reconstruction` relation; it confers no authority.
///
/// # Errors
///
/// Returns the typed denial for purpose/subject/context substitution, an
/// invalid actor assertion, or an incomplete/duplicate/invalid decision set.
pub fn validate_signal_v2(
    proposal_receipt_digest: DigestBytes,
    proposal: &TextClaimAgentProposal,
    signal: &ExternalAdmissionSignal,
) -> Result<(), ProposalAdmissionError> {
    if !purpose_matches(signal.purpose, AttestationPurpose::ProposalApproval) {
        return Err(ProposalAdmissionError::AttestationPurposeMismatch);
    }
    if signal.subject_proposal_receipt_digest != proposal_receipt_digest {
        return Err(ProposalAdmissionError::ProposalSubjectMismatch);
    }
    if signal.context_digest != proposal.source_interaction_digest {
        return Err(ProposalAdmissionError::AdmissionContextMismatch);
    }
    validate_signal_shape(signal)?;
    derive_outcome(proposal, signal).map(|_| ())
}

/// The closed shape law of an external admission signal, judged over the
/// signal alone (registry row P3 `signal_shape`): the actor assertion is
/// non-blank and at most 256 bytes, and no two decisions name the same
/// claim ordinal. Completeness against the proposal is `derive_outcome`.
///
/// # Errors
///
/// Returns `AdmissionSignalInvalid` for an invalid actor assertion or
/// `AdmissionDecisionInvalid` for a duplicate ordinal.
pub fn validate_signal_shape(
    signal: &ExternalAdmissionSignal,
) -> Result<(), ProposalAdmissionError> {
    if signal.actor_assertion.trim().is_empty()
        || signal.actor_assertion.len() > MAX_ACTOR_ASSERTION_BYTES
    {
        return Err(ProposalAdmissionError::AdmissionSignalInvalid);
    }
    let mut seen = BTreeSet::new();
    if signal
        .decisions
        .iter()
        .any(|decision| !seen.insert(decision.claim_ordinal()))
    {
        return Err(ProposalAdmissionError::AdmissionDecisionInvalid);
    }
    Ok(())
}

/// Structural admission of a V2 proposal receipt: type, shape, proposal
/// identity. The receipt's full law (schema identity, digest, evidence)
/// is the verifier's.
fn validate_proposal_receipt_v2(
    receipt: &ReceiptEnvelopeV2,
) -> Result<AgentProposalPayloadV2, ProposalAdmissionError> {
    if receipt.receipt_type != ReceiptTypeV2::WorkflowAgentProposal {
        return Err(ProposalAdmissionError::UnsupportedProposalSchema);
    }
    let payload: AgentProposalPayloadV2 =
        serde_json::from_value(receipt.payload.as_value().clone())?;
    if payload.schema != PayloadSchemaV2::VR_WORKFLOW_AGENT_PROPOSAL_0_2.label() {
        return Err(ProposalAdmissionError::UnsupportedProposalSchema);
    }
    validate_proposal(&payload.proposal)?;
    if payload.proposal_digest != agent_proposal_digest(&payload.proposal)? {
        return Err(ProposalAdmissionError::ProposalDigestMismatch);
    }
    Ok(payload)
}

/// Seal a group-2 V2 receipt: every binding slot absent, no chain
/// (ADR-056 §3.4; registry rows P2/P3).
fn seal_v2<T: serde::Serialize>(
    receipt_type: ReceiptTypeV2,
    schema: PayloadSchemaV2,
    payload: &T,
) -> Result<ReceiptEnvelopeV2, ProposalAdmissionError> {
    let schema_digest = schema
        .identity()
        .map_err(|error| ProposalAdmissionError::Receipt(error.to_string()))?;
    seal_receipt_v2(ReceiptV2Draft {
        receipt_type,
        schema_digest,
        context_digest: None,
        policy_digest: None,
        logical_time: 1,
        parent_id: None,
        payload: CanonicalPayload::new(serde_json::to_value(payload)?)?,
    })
    .map_err(|error| ProposalAdmissionError::Receipt(error.to_string()))
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

/// Validate the closed proposal shape: at most 16 claims, ordinals
/// contiguous from 1 in order, each text non-blank and at most 4096 bytes.
/// Shared by the producers and the verifier's `proposal_shape` relation.
///
/// # Errors
///
/// Returns the typed denial for too many claims, an out-of-order ordinal,
/// or invalid claim text.
pub fn validate_proposal(proposal: &TextClaimAgentProposal) -> Result<(), ProposalAdmissionError> {
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

/// Rebuild, in memory only, the V1 `@0.1` admission envelope the frozen
/// historical constructor sealed: `governance` type, `Governance` boundary,
/// the proposal's context, the legacy L5 schema and policy labels,
/// `logical_time` 2 and the cross-type parent link. Verify-only; the result
/// never leaves [`reconstruct_admission_v1`].
fn rebuild_admission_envelope_v1(
    proposal_receipt: &ReceiptEnvelope,
    admission_payload: &AdmissionReceiptPayload,
) -> Result<ReceiptEnvelope, ProposalAdmissionError> {
    let value = serde_json::json!({
        "envelope_version": SchemaVersion::V1,
        "receipt_type": ReceiptType::Governance,
        "context_digest": proposal_receipt.context_digest,
        "schema_digest": digest_label(PROPOSAL_ADMISSION_SCHEMA_DOMAIN, PROPOSAL_ADMISSION_SCHEMA)?,
        "policy_digest": digest_label(PROPOSAL_ADMISSION_POLICY_DOMAIN, PROPOSAL_ADMISSION_POLICY)?,
        "logical_time": "2",
        "event_hash": DigestBytes::from_array([0; 32]),
        "boundary_origin": BoundaryOrigin::Governance,
        "digest_algorithm": SchemaVersion::V1.digest_algorithm(),
        "canonicalization": SchemaVersion::V1.canonicalization(),
        "payload": serde_json::to_value(admission_payload)?,
        "parent_id": proposal_receipt.event_hash,
    });
    let mut envelope: ReceiptEnvelope = serde_json::from_value(value)?;
    envelope.event_hash = vr_receipt_identity::compute_event_hash(&envelope)
        .map_err(|error| ProposalAdmissionError::Receipt(error.to_string()))?;
    Ok(envelope)
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

#[cfg(test)]
mod mint_ratchet_guard_tests;
