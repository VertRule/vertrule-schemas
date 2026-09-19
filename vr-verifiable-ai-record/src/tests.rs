use anyhow::Result;
use vertrule_schemas::{
    AttestationPurpose, ClaimAdmissionDecision, ClaimRejectionReason, ExternalAdmissionSignal,
    IJsonUInt, ProposedTextClaim, TextClaimAgentProposal,
};

use super::*;

fn ordinal(value: u64) -> Result<IJsonUInt> {
    Ok(IJsonUInt::new(value)?)
}

fn capture(prompt: &str, response: &str, extraction: bool) -> ProviderInteractionCaptureInput {
    let policy = if extraction {
        "openai-responses-claim-extraction/v1"
    } else {
        "openai-responses-text/v1"
    };
    ProviderInteractionCaptureInput {
        prompt: prompt.to_owned(),
        provider: "OpenAI".to_owned(),
        requested_model: "gpt-test".to_owned(),
        resolved_model: Some("gpt-test-2026-01-01".to_owned()),
        provider_response_id: Some(
            if extraction {
                "resp_extract"
            } else {
                "resp_answer"
            }
            .to_owned(),
        ),
        capture_policy_version: policy.to_owned(),
        response: response.to_owned(),
    }
}

struct Children {
    source: ReceiptEnvelopeV2,
    extraction: ReceiptEnvelopeV2,
    proposal: ReceiptEnvelopeV2,
    admission: ReceiptEnvelopeV2,
}

fn children() -> Result<Children> {
    let source = seal_provider_interaction_v2(&capture(
        "What does the contract say?",
        "The contract requires written notice and no fee.",
        false,
    ))?;
    let extraction = seal_provider_interaction_v2(&capture(
        "Extract the claims.",
        "1. Notice must be written.\n2. No termination fee is required.",
        true,
    ))?;
    let proposal = vr_proposal_admission::seal_proposal_v2(&TextClaimAgentProposal {
        source_interaction_digest: source.receipt_digest,
        extraction_interaction_digest: extraction.receipt_digest,
        claims: vec![
            ProposedTextClaim {
                ordinal: ordinal(1)?,
                text: "Notice must be written.".to_owned(),
            },
            ProposedTextClaim {
                ordinal: ordinal(2)?,
                text: "No termination fee is required.".to_owned(),
            },
        ],
    })?;
    let signal = ExternalAdmissionSignal {
        purpose: AttestationPurpose::ProposalApproval,
        subject_proposal_receipt_digest: proposal.receipt_digest,
        context_digest: source.receipt_digest,
        actor_assertion: "local-reviewer".to_owned(),
        decisions: vec![
            ClaimAdmissionDecision::Edit {
                claim_ordinal: ordinal(1)?,
                admitted_text: "The AI claims notice must be written.".to_owned(),
            },
            ClaimAdmissionDecision::Reject {
                claim_ordinal: ordinal(2)?,
                reason: ClaimRejectionReason::UserRejected,
            },
        ],
    };
    let admission = vr_proposal_admission::admit_proposal_v2(&proposal, &signal)?
        .admission_receipt()
        .clone();
    Ok(Children {
        source,
        extraction,
        proposal,
        admission,
    })
}

#[test]
fn interaction_v2_carries_text_without_leaf_digests_or_slots() -> Result<()> {
    let sealed = seal_provider_interaction_v2(&capture("q", "a", false))?;
    assert_eq!(sealed.receipt_type, ReceiptTypeV2::AiProviderInteraction);
    assert_eq!(sealed.context_digest, None);
    assert_eq!(sealed.policy_digest, None);
    assert_eq!(sealed.parent_id, None);
    assert_eq!(
        sealed.schema_digest,
        PayloadSchemaV2::VR_AI_PROVIDER_INTERACTION_0_2.identity()?
    );
    let payload = sealed.payload.as_value();
    assert_eq!(payload["schema"], "vr.ai.provider_interaction@0.2");
    assert_eq!(payload["prompt"], "q");
    assert_eq!(payload["response"], "a");
    assert_eq!(payload["provider_attestation"], "not_provided");
    let keys: Vec<&String> = payload
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("payload is not an object"))?
        .keys()
        .collect();
    assert!(!keys
        .iter()
        .any(|k| k.contains("digest") || *k == "payload_kind"));
    Ok(())
}

#[test]
fn interaction_v2_omits_absent_optional_facts() -> Result<()> {
    let mut input = capture("q", "a", false);
    input.resolved_model = None;
    input.provider_response_id = None;
    let sealed = seal_provider_interaction_v2(&input)?;
    let payload = sealed.payload.as_value();
    assert!(payload.get("resolved_model").is_none());
    assert!(payload.get("provider_response_id").is_none());
    Ok(())
}

#[test]
fn record_v2_is_stable_and_binds_every_child_by_receipt_digest() -> Result<()> {
    let c = children()?;
    let first = seal_record_v2(&c.source, &c.extraction, &c.proposal, &c.admission)?;
    let second = seal_record_v2(&c.source, &c.extraction, &c.proposal, &c.admission)?;
    assert_eq!(first, second);
    let record = first.record();
    assert_eq!(record.receipt_type, ReceiptTypeV2::RecordVerifiableAiRecord);
    assert_eq!(record.context_digest, None);
    assert_eq!(record.policy_digest, None);
    assert_eq!(record.parent_id, None);
    let payload = first.payload();
    assert_eq!(payload.schema, "vr.record.verifiable_ai_record@0.2");
    assert_eq!(payload.record_policy, "vr.record.integrity_not_truth@0.1");
    assert_eq!(payload.source_interaction_digest, c.source.receipt_digest);
    assert_eq!(
        payload.extraction_interaction_digest,
        c.extraction.receipt_digest
    );
    assert_eq!(payload.proposal_receipt_digest, c.proposal.receipt_digest);
    assert_eq!(payload.admission_receipt_digest, c.admission.receipt_digest);
    assert_eq!(
        payload.admitted_proposal_digest,
        vr_proposal_admission::admitted_proposal_digest(first.admitted_proposal())?
    );
    assert_eq!(first.admitted_proposal().claims.len(), 1);
    assert_eq!(first.admitted_proposal().rejected_claims.len(), 1);
    let artifact = first.artifact();
    assert_eq!(artifact.format, "vr-verifiable-ai-record/v3");
    assert_eq!(artifact.record, *record);
    assert_eq!(artifact.evidence.len(), 4);
    for (digest, receipt) in &artifact.evidence {
        assert_eq!(*digest, receipt.receipt_digest);
    }
    Ok(())
}

#[test]
fn source_or_extraction_substitution_is_rejected() -> Result<()> {
    let c = children()?;
    let other_source =
        seal_provider_interaction_v2(&capture("Another question", "Another answer", false))?;
    assert!(matches!(
        seal_record_v2(&other_source, &c.extraction, &c.proposal, &c.admission),
        Err(VerifiableAiRecordError::InteractionLineageMismatch)
    ));
    let other_extraction =
        seal_provider_interaction_v2(&capture("Another extraction", "{}", true))?;
    assert!(matches!(
        seal_record_v2(&c.source, &other_extraction, &c.proposal, &c.admission),
        Err(VerifiableAiRecordError::InteractionLineageMismatch)
    ));
    Ok(())
}

#[test]
fn mistyped_children_and_foreign_admissions_are_rejected() -> Result<()> {
    let c = children()?;
    assert!(matches!(
        seal_record_v2(&c.proposal, &c.extraction, &c.proposal, &c.admission),
        Err(VerifiableAiRecordError::UnsupportedInteractionSchema)
    ));
    assert!(matches!(
        seal_record_v2(&c.source, &c.extraction, &c.admission, &c.proposal),
        Err(VerifiableAiRecordError::UnsupportedProposalAdmissionSchema)
    ));
    let other = children()?;
    let other_proposal = vr_proposal_admission::seal_proposal_v2(&TextClaimAgentProposal {
        source_interaction_digest: other.source.receipt_digest,
        extraction_interaction_digest: other.extraction.receipt_digest,
        claims: vec![ProposedTextClaim {
            ordinal: ordinal(1)?,
            text: "Only one claim.".to_owned(),
        }],
    })?;
    let foreign_admission = vr_proposal_admission::admit_proposal_v2(
        &other_proposal,
        &ExternalAdmissionSignal {
            purpose: AttestationPurpose::ProposalApproval,
            subject_proposal_receipt_digest: other_proposal.receipt_digest,
            context_digest: other.source.receipt_digest,
            actor_assertion: "local-reviewer".to_owned(),
            decisions: vec![ClaimAdmissionDecision::Approve {
                claim_ordinal: ordinal(1)?,
            }],
        },
    )?
    .admission_receipt()
    .clone();
    assert!(matches!(
        seal_record_v2(&c.source, &c.extraction, &c.proposal, &foreign_admission),
        Err(VerifiableAiRecordError::ProposalAdmissionMismatch)
    ));
    Ok(())
}

/// The six frozen `@0.1` derivations stay byte-identical for the
/// historical fixtures (ADR-054 §7 legacy L4 rows; the committed
/// `verifiable_ai_record_v2_valid.json` fixture).
#[test]
fn frozen_v1_derivations_are_verify_computable_and_unchanged() {
    assert_eq!(
        prompt_digest("What does the contract say?").to_hex(),
        "7980879494f37a4c115d639b40bc050dfcb634d9021eb001d37c00e7f7a82e40"
    );
    assert_eq!(
        response_digest("The contract requires written notice and no fee.").to_hex(),
        "e93b16cd89d54045b526e25537e07da3663e7c9a52b7973738900d965b3f1c04"
    );
    assert_eq!(
        interaction_schema_digest("vr.ai.provider_interaction@0.1").to_hex(),
        "abd7723f9146b4a8e53cdd8a8ea623e4b289254e48861fe6ca1712ac94938269"
    );
    assert_eq!(
        capture_policy_digest("openai-responses-text/v1").to_hex(),
        "ed5bc5cb996de9c12ce48666de8a0ba5752ff3cae203a5a5de0be0cb53851ffd"
    );
    assert_eq!(
        record_schema_digest("vr.record.verifiable_ai_record@0.1").to_hex(),
        "ff4ad60c528e452f96c1ecd7086e517daa1971b8a3be43b61f9a973ac95dcd26"
    );
    assert_eq!(
        record_policy_digest("vr.record.integrity_not_truth@0.1").to_hex(),
        "45ba9e98499b0aa49a6ccf2e1041d8e598079e9c6c99a79d49aa7e594ed9cd4f"
    );
}
