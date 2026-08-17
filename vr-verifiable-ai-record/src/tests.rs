use anyhow::Result;
use vertrule_schemas::{
    AttestationPurpose, CapturedRequest, CapturedResponse, ClaimAdmissionDecision,
    ClaimRejectionReason, ExternalAdmissionSignal, IJsonUInt, ProposedTextClaim,
    ProviderInteractionPayload, TextClaimAgentProposal,
};

use super::*;

fn interaction(prompt: &str, response: &str, extraction: bool) -> Result<String> {
    let policy = if extraction {
        "openai-responses-claim-extraction/v1"
    } else {
        "openai-responses-text/v1"
    };
    let payload = ProviderInteractionPayload {
        payload_kind: PROVIDER_INTERACTION_PAYLOAD_KIND.to_owned(),
        schema: PROVIDER_INTERACTION_SCHEMA.to_owned(),
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
        request: CapturedRequest {
            prompt: prompt.to_owned(),
            prompt_digest: prompt_digest(prompt),
        },
        response: CapturedResponse {
            text: response.to_owned(),
            response_digest: response_digest(response),
        },
        provider_attestation: "not_provided".to_owned(),
    };
    let mut envelope: ReceiptEnvelope = serde_json::from_value(serde_json::json!({
        "envelope_version": SchemaVersion::V1,
        "receipt_type": ReceiptType::Llm,
        "context_digest": payload.request.prompt_digest,
        "schema_digest": interaction_schema_digest(PROVIDER_INTERACTION_SCHEMA),
        "policy_digest": capture_policy_digest(policy),
        "logical_time": "1",
        "event_hash": DigestBytes::from_array([0; 32]),
        "boundary_origin": BoundaryOrigin::Adapter,
        "digest_algorithm": SchemaVersion::V1.digest_algorithm(),
        "canonicalization": SchemaVersion::V1.canonicalization(),
        "payload": payload,
    }))?;
    envelope.event_hash = vr_receipt_identity::compute_event_hash(&envelope)?;
    Ok(canonical_string(&envelope)?)
}

fn complete_inputs() -> Result<(String, String, String)> {
    let source = interaction(
        "What does the contract say?",
        "The contract requires written notice and no fee.",
        false,
    )?;
    let extraction = interaction(
        "Extract claims from the captured answer.",
        r#"{"claims":["Written notice is required.","No fee is required."]}"#,
        true,
    )?;
    let source_envelope: ReceiptEnvelope = serde_json::from_str(&source)?;
    let extraction_envelope: ReceiptEnvelope = serde_json::from_str(&extraction)?;
    let proposal = TextClaimAgentProposal {
        source_interaction_digest: source_envelope.event_hash,
        extraction_interaction_digest: extraction_envelope.event_hash,
        claims: vec![
            ProposedTextClaim {
                ordinal: IJsonUInt::new(1)?,
                text: "Written notice is required.".to_owned(),
            },
            ProposedTextClaim {
                ordinal: IJsonUInt::new(2)?,
                text: "No fee is required.".to_owned(),
            },
        ],
    };
    let sealed = vr_proposal_admission::seal_proposal(&proposal)?;
    let signal = ExternalAdmissionSignal {
        purpose: AttestationPurpose::ProposalApproval,
        subject_proposal_receipt_digest: sealed.envelope().event_hash,
        context_digest: sealed.envelope().context_digest,
        actor_assertion: "test-reviewer".to_owned(),
        decisions: vec![
            ClaimAdmissionDecision::Edit {
                claim_ordinal: IJsonUInt::new(1)?,
                admitted_text: "The AI claims written notice is required.".to_owned(),
            },
            ClaimAdmissionDecision::Reject {
                claim_ordinal: IJsonUInt::new(2)?,
                reason: ClaimRejectionReason::UserRejected,
            },
        ],
    };
    let admitted = vr_proposal_admission::admit_proposal(sealed.envelope(), &signal)?;
    let proposal_admission = canonical_string(&admitted.bundle())?;
    Ok((source, extraction, proposal_admission))
}

#[test]
fn record_is_stable_structured_and_frozen() -> Result<()> {
    let (source, extraction, admission) = complete_inputs()?;
    let first = seal_record(&source, &extraction, &admission)?;
    let second = seal_record(&source, &extraction, &admission)?;
    assert_eq!(first.artifact(), second.artifact());
    assert!(first.artifact().starts_with("{\n"));
    assert!(!first.artifact().contains("\\\"boundary_origin"));
    let package: VerifiableAiRecordArtifact = serde_json::from_str(first.artifact())?;
    assert_eq!(package.format, VERIFIABLE_AI_RECORD_FORMAT);
    assert_eq!(package.record.event_hash, first.envelope().event_hash);
    assert_eq!(
        package.source_interaction.event_hash,
        source_envelope(&source)?
    );
    assert_eq!(
        first.envelope().event_hash.to_string(),
        "569b1a02e277ac7542a88ca76cafd400e62cdaef4ce671d30cd3526f3eb3f740"
    );
    Ok(())
}

fn source_envelope(canonical: &str) -> Result<DigestBytes> {
    Ok(serde_json::from_str::<ReceiptEnvelope>(canonical)?.event_hash)
}

#[test]
fn source_or_extraction_substitution_is_rejected() -> Result<()> {
    let (source, extraction, admission) = complete_inputs()?;
    let other_source = interaction("Another question", "Another answer", false)?;
    assert!(matches!(
        seal_record(&other_source, &extraction, &admission),
        Err(VerifiableAiRecordError::InteractionLineageMismatch)
    ));
    let other_extraction = interaction("Another extraction", "{}", true)?;
    assert!(matches!(
        seal_record(&source, &other_extraction, &admission),
        Err(VerifiableAiRecordError::InteractionLineageMismatch)
    ));
    Ok(())
}

#[test]
fn prompt_response_and_admitted_claim_mutations_are_rejected() -> Result<()> {
    let (source, extraction, admission) = complete_inputs()?;
    let prompt_tamper = source.replacen("contract", "contracts", 1);
    assert!(seal_record(&prompt_tamper, &extraction, &admission).is_err());
    let response_tamper = source.replacen("no fee", "a fee", 1);
    assert!(seal_record(&response_tamper, &extraction, &admission).is_err());
    let admitted_tamper = admission.replacen("written notice", "verbal notice", 1);
    assert!(seal_record(&source, &extraction, &admitted_tamper).is_err());
    Ok(())
}
