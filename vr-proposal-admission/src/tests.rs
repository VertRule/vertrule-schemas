use anyhow::Result;
use vertrule_schemas::{
    AttestationPurpose, ClaimAdmissionDecision, ClaimRejectionReason, DigestBytes,
    ExternalAdmissionSignal, IJsonUInt, ProposedTextClaim, TextClaimAgentProposal,
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

#[test]
fn claim_change_changes_proposal_receipt_identity() -> Result<()> {
    let a = seal_proposal(&proposal("Termination requires 30 days notice.")?)?;
    let b = seal_proposal(&proposal("Termination requires 31 days notice.")?)?;
    assert_ne!(a.envelope().event_hash, b.envelope().event_hash);
    assert_ne!(
        agent_proposal_digest(&proposal("Termination requires 30 days notice.")?)?,
        agent_proposal_digest(&proposal("Termination requires 31 days notice.")?)?
    );
    Ok(())
}

#[test]
fn proposal_and_context_substitution_are_rejected() -> Result<()> {
    let a = seal_proposal(&proposal("Termination requires 30 days notice.")?)?;
    let b = seal_proposal(&proposal("Termination requires 31 days notice.")?)?;
    let signal_for_a = signal(a.envelope().event_hash, a.envelope().context_digest)?;
    assert!(matches!(
        admit_proposal(b.envelope(), &signal_for_a),
        Err(ProposalAdmissionError::ProposalSubjectMismatch)
    ));

    let wrong_context = signal(a.envelope().event_hash, DigestBytes::from_array([0x99; 32]))?;
    assert!(matches!(
        admit_proposal(a.envelope(), &wrong_context),
        Err(ProposalAdmissionError::AdmissionContextMismatch)
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
fn admission_preserves_approve_edit_reject_lineage() -> Result<()> {
    let sealed = seal_proposal(&proposal("Termination requires 30 days notice.")?)?;
    let admitted = admit_proposal(
        sealed.envelope(),
        &signal(
            sealed.envelope().event_hash,
            sealed.envelope().context_digest,
        )?,
    )?;
    let wire = admitted.admitted_proposal();
    assert_eq!(wire.claims.len(), 2);
    assert_eq!(wire.claims[0].text, "Termination requires 30 days notice.");
    assert_eq!(wire.claims[1].text, "The AI claims notice must be written.");
    assert_eq!(wire.rejected_claims.len(), 1);
    assert_eq!(
        admitted.admission_receipt().parent_id,
        Some(sealed.envelope().event_hash)
    );
    Ok(())
}

#[test]
fn golden_vector_is_frozen() -> Result<()> {
    let sealed = seal_proposal(&proposal("Termination requires 30 days notice.")?)?;
    let admitted = admit_proposal(
        sealed.envelope(),
        &signal(
            sealed.envelope().event_hash,
            sealed.envelope().context_digest,
        )?,
    )?;
    assert_eq!(
        sealed.envelope().event_hash.to_string(),
        "4628ab8233f73e163d5b8e9312dc2943dd786cbed1763ed741b8630f0b846924"
    );
    assert_eq!(
        admitted.admission_receipt().event_hash.to_string(),
        "3a44bf59d65cd3010157fc0d97a4590e172095f6c754715045a9ad3b7e2522a0"
    );
    assert_eq!(
        admitted.admitted_proposal_digest().to_string(),
        "a616b4fa2a36d98b4d7f99a82659bac86ca6765b22311b7ef6df3a1c0a3fbed8"
    );
    assert_eq!(
        sealed.canonical().as_bytes(),
        vr_jcs::to_canon_bytes_from_slice(sealed.canonical().as_bytes())?
    );
    Ok(())
}
