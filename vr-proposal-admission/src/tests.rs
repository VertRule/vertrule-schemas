use anyhow::Result;
use vertrule_schemas::{
    AttestationPurpose, ClaimAdmissionDecision, ClaimRejectionReason, DigestBytes,
    ExternalAdmissionSignal, IJsonUInt, ProposalAdmissionBundle, ProposedTextClaim,
    ReceiptEnvelope, TextClaimAgentProposal,
};

use super::*;

fn ordinal(value: u64) -> Result<IJsonUInt> {
    Ok(IJsonUInt::new(value)?)
}

fn proposal(first_text: &str) -> Result<TextClaimAgentProposal> {
    Ok(TextClaimAgentProposal {
        source_interaction_digest: DigestBytes::from_array([0x11; 32]),
        extraction_interaction_digest: DigestBytes::from_array([0x22; 32]),
        claims: vec![
            ProposedTextClaim {
                ordinal: ordinal(1)?,
                text: first_text.to_owned(),
            },
            ProposedTextClaim {
                ordinal: ordinal(2)?,
                text: "Notice must be written.".to_owned(),
            },
            ProposedTextClaim {
                ordinal: ordinal(3)?,
                text: "No termination fee is required.".to_owned(),
            },
        ],
    })
}

fn signal(
    receipt_digest: DigestBytes,
    context_digest: DigestBytes,
) -> Result<ExternalAdmissionSignal> {
    Ok(ExternalAdmissionSignal {
        purpose: AttestationPurpose::ProposalApproval,
        subject_proposal_receipt_digest: receipt_digest,
        context_digest,
        actor_assertion: "local-reviewer".to_owned(),
        decisions: vec![
            ClaimAdmissionDecision::Approve {
                claim_ordinal: ordinal(1)?,
            },
            ClaimAdmissionDecision::Edit {
                claim_ordinal: ordinal(2)?,
                admitted_text: "The AI claims notice must be written.".to_owned(),
            },
            ClaimAdmissionDecision::Reject {
                claim_ordinal: ordinal(3)?,
                reason: ClaimRejectionReason::UserRejected,
            },
        ],
    })
}

/// The frozen V1 golden bundle (byte-identical to
/// `vertrule-verifier/tests/fixtures/proposal_admission_v1_valid.json`),
/// carrying the crate's historical golden identities.
const V1_GOLDEN_BUNDLE: &str = include_str!("../test-vectors/proposal_admission_v1_valid_001.json");

fn v1_golden() -> Result<(ReceiptEnvelope, ReceiptEnvelope, ProposalAdmissionBundle)> {
    let bundle: ProposalAdmissionBundle = serde_json::from_str(V1_GOLDEN_BUNDLE)?;
    let proposal: ReceiptEnvelope = serde_json::from_str(&bundle.proposal_canonical)?;
    let admission: ReceiptEnvelope = serde_json::from_str(&bundle.admission_canonical)?;
    Ok((proposal, admission, bundle))
}

#[test]
fn claim_change_changes_proposal_receipt_identity() -> Result<()> {
    let a = seal_proposal_v2(&proposal("Termination requires 30 days notice.")?)?;
    let b = seal_proposal_v2(&proposal("Termination requires 31 days notice.")?)?;
    assert_ne!(a.receipt_digest, b.receipt_digest);
    assert_ne!(
        agent_proposal_digest(&proposal("Termination requires 30 days notice.")?)?,
        agent_proposal_digest(&proposal("Termination requires 31 days notice.")?)?
    );
    Ok(())
}

#[test]
fn sealed_v2_proposal_has_no_binding_slots_and_the_0_2_schema() -> Result<()> {
    let sealed = seal_proposal_v2(&proposal("Termination requires 30 days notice.")?)?;
    assert_eq!(sealed.receipt_type, ReceiptTypeV2::WorkflowAgentProposal);
    assert_eq!(sealed.context_digest, None);
    assert_eq!(sealed.policy_digest, None);
    assert_eq!(sealed.parent_id, None);
    assert_eq!(
        sealed.schema_digest,
        PayloadSchemaV2::VR_WORKFLOW_AGENT_PROPOSAL_0_2.identity()?
    );
    let payload: AgentProposalPayloadV2 =
        serde_json::from_value(sealed.payload.as_value().clone())?;
    assert_eq!(payload.schema, "vr.workflow.agent_proposal@0.2");
    assert_eq!(
        payload.proposal_digest,
        agent_proposal_digest(&proposal("Termination requires 30 days notice.")?)?
    );
    Ok(())
}

#[test]
fn proposal_and_context_substitution_are_rejected() -> Result<()> {
    let a = seal_proposal_v2(&proposal("Termination requires 30 days notice.")?)?;
    let b = seal_proposal_v2(&proposal("Termination requires 31 days notice.")?)?;
    let signal_for_a = signal(a.receipt_digest, DigestBytes::from_array([0x11; 32]))?;
    assert!(matches!(
        admit_proposal_v2(&b, &signal_for_a),
        Err(ProposalAdmissionError::ProposalSubjectMismatch)
    ));

    let wrong_context = signal(a.receipt_digest, DigestBytes::from_array([0x99; 32]))?;
    assert!(matches!(
        admit_proposal_v2(&a, &wrong_context),
        Err(ProposalAdmissionError::AdmissionContextMismatch)
    ));
    Ok(())
}

#[test]
fn a_non_proposal_receipt_cannot_be_admitted() -> Result<()> {
    let a = seal_proposal_v2(&proposal("Termination requires 30 days notice.")?)?;
    let admitted = admit_proposal_v2(
        &a,
        &signal(a.receipt_digest, DigestBytes::from_array([0x11; 32]))?,
    )?;
    let admission = admitted.admission_receipt().clone();
    assert!(matches!(
        admit_proposal_v2(
            &admission,
            &signal(
                admission.receipt_digest,
                DigestBytes::from_array([0x11; 32])
            )?
        ),
        Err(ProposalAdmissionError::UnsupportedProposalSchema)
    ));
    Ok(())
}

#[test]
fn approval_and_authorization_purposes_cannot_substitute() {
    assert!(!purpose_matches(
        AttestationPurpose::ActionAuthorization,
        AttestationPurpose::ProposalApproval
    ));
    assert!(!purpose_matches(
        AttestationPurpose::ProposalApproval,
        AttestationPurpose::ActionAuthorization
    ));
}

#[test]
fn admission_preserves_approve_edit_reject_lineage_with_v2_references() -> Result<()> {
    let sealed = seal_proposal_v2(&proposal("Termination requires 30 days notice.")?)?;
    let admitted = admit_proposal_v2(
        &sealed,
        &signal(sealed.receipt_digest, DigestBytes::from_array([0x11; 32]))?,
    )?;
    let wire = admitted.admitted_proposal();
    assert_eq!(wire.claims.len(), 2);
    assert_eq!(wire.claims[0].text, "Termination requires 30 days notice.");
    assert_eq!(wire.claims[1].text, "The AI claims notice must be written.");
    assert_eq!(wire.rejected_claims.len(), 1);
    assert_eq!(wire.proposal_receipt_digest, sealed.receipt_digest);
    assert_eq!(
        wire.admission_receipt_digest,
        admitted.admission_receipt().receipt_digest
    );
    // The former cross-type parent link is a payload fact, never parent_id.
    assert_eq!(admitted.admission_receipt().parent_id, None);
    let payload: ProposalAdmissionPayloadV2 =
        serde_json::from_value(admitted.admission_receipt().payload.as_value().clone())?;
    assert_eq!(payload.proposal_receipt_digest, sealed.receipt_digest);
    assert_eq!(
        payload.signal.subject_proposal_receipt_digest,
        sealed.receipt_digest
    );
    assert_eq!(
        admitted.admitted_proposal_digest(),
        admitted_proposal_digest(wire)?
    );
    let bundle = admitted.bundle();
    assert_eq!(bundle.format, "vr-proposal-admission/v2");
    assert_eq!(bundle.proposal, sealed);
    Ok(())
}

#[test]
fn v2_sealing_is_deterministic() -> Result<()> {
    let a = seal_proposal_v2(&proposal("Termination requires 30 days notice.")?)?;
    let b = seal_proposal_v2(&proposal("Termination requires 30 days notice.")?)?;
    assert_eq!(a, b);
    let sig = signal(a.receipt_digest, DigestBytes::from_array([0x11; 32]))?;
    assert_eq!(
        admit_proposal_v2(&a, &sig)?.admission_receipt(),
        admit_proposal_v2(&b, &sig)?.admission_receipt()
    );
    Ok(())
}

#[test]
fn v1_golden_reconstructs_verify_only_and_stays_frozen() -> Result<()> {
    let (proposal, admission, bundle) = v1_golden()?;
    assert_eq!(
        proposal.event_hash.to_string(),
        "4628ab8233f73e163d5b8e9312dc2943dd786cbed1763ed741b8630f0b846924"
    );
    assert_eq!(
        admission.event_hash.to_string(),
        "3a44bf59d65cd3010157fc0d97a4590e172095f6c754715045a9ad3b7e2522a0"
    );
    let reconstructed = reconstruct_admission_v1(&proposal, &admission)?;
    assert!(reconstructed.admission_matches_presented);
    assert_eq!(reconstructed.admitted_proposal, bundle.admitted_proposal);
    assert_eq!(
        reconstructed.admitted_proposal_digest.to_string(),
        "a616b4fa2a36d98b4d7f99a82659bac86ca6765b22311b7ef6df3a1c0a3fbed8"
    );
    assert_eq!(
        reconstructed.admitted_proposal_digest,
        bundle.admitted_proposal_digest
    );
    Ok(())
}

#[test]
fn v1_reconstruction_reports_a_presented_admission_that_does_not_match() -> Result<()> {
    let (proposal, admission, _) = v1_golden()?;
    let mut altered = serde_json::to_value(&admission)?;
    altered["logical_time"] = serde_json::json!("3");
    let altered: ReceiptEnvelope = serde_json::from_value(altered)?;
    let reconstructed = reconstruct_admission_v1(&proposal, &altered)?;
    assert!(!reconstructed.admission_matches_presented);
    assert_eq!(
        reconstructed.admitted_proposal_digest.to_string(),
        "a616b4fa2a36d98b4d7f99a82659bac86ca6765b22311b7ef6df3a1c0a3fbed8"
    );
    Ok(())
}

#[test]
fn v1_reconstruction_refuses_a_v1_proposal_it_cannot_admit() -> Result<()> {
    let (proposal, admission, _) = v1_golden()?;
    let mut other = serde_json::to_value(&proposal)?;
    other["context_digest"] = serde_json::json!(DigestBytes::from_array([0x99; 32]).to_hex());
    let other: ReceiptEnvelope = serde_json::from_value(other)?;
    assert!(reconstruct_admission_v1(&other, &admission).is_err());
    Ok(())
}
