//! Tests for the `@0.2` proposal/admission payload shapes and bundle.

use crate::receipts::proposal_admission_v2::PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2;
use crate::{
    AdmittedClaim, AdmittedClaimOperation, AdmittedProposal, AgentProposalPayloadV2,
    AttestationPurpose, ClaimAdmissionDecision, ClaimRejectionReason, DigestBytes,
    ExternalAdmissionSignal, IJsonUInt, PayloadSchemaV2, ProposalAdmissionBundleV2,
    ProposalAdmissionPayloadV2, ProposedTextClaim, ReceiptEnvelopeV2, RejectedClaim,
    TextClaimAgentProposal,
};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

/// A structurally valid V2 envelope document of `receipt_type` whose
/// `receipt_digest` is `fill`-filled (no digest law is judged here).
fn envelope_document(receipt_type: &str, fill: u8) -> serde_json::Value {
    serde_json::json!({
        "envelope_version": 2,
        "receipt_type": receipt_type,
        "schema_digest": digest(0x5c).to_hex(),
        "logical_time": "1",
        "payload": {"schema": "x"},
        "receipt_digest": digest(fill).to_hex(),
    })
}

fn envelope(receipt_type: &str, fill: u8) -> Result<ReceiptEnvelopeV2, anyhow::Error> {
    Ok(serde_json::from_value(envelope_document(
        receipt_type,
        fill,
    ))?)
}

fn bundle() -> Result<ProposalAdmissionBundleV2, anyhow::Error> {
    Ok(ProposalAdmissionBundleV2 {
        format: PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2.to_string(),
        proposal: envelope("vr.workflow.agent_proposal", 4)?,
        admission: envelope("vr.workflow.proposal_admission", 8)?,
        admitted_proposal: AdmittedProposal {
            proposal_receipt_digest: digest(4),
            admission_receipt_digest: digest(8),
            claims: admission_payload()?.admitted_claims,
            rejected_claims: admission_payload()?.rejected_claims,
        },
        admitted_proposal_digest: digest(9),
    })
}

fn proposal_payload() -> Result<AgentProposalPayloadV2, anyhow::Error> {
    Ok(AgentProposalPayloadV2 {
        schema: PayloadSchemaV2::VR_WORKFLOW_AGENT_PROPOSAL_0_2
            .label()
            .to_string(),
        proposal_digest: digest(3),
        proposal: TextClaimAgentProposal {
            source_interaction_digest: digest(1),
            extraction_interaction_digest: digest(2),
            claims: vec![ProposedTextClaim {
                ordinal: IJsonUInt::new(1)?,
                text: "The sky is blue.".to_string(),
            }],
        },
    })
}

fn admission_payload() -> Result<ProposalAdmissionPayloadV2, anyhow::Error> {
    Ok(ProposalAdmissionPayloadV2 {
        schema: PayloadSchemaV2::VR_WORKFLOW_PROPOSAL_ADMISSION_0_2
            .label()
            .to_string(),
        proposal_receipt_digest: digest(4),
        admission_signal_digest: digest(5),
        signal: ExternalAdmissionSignal {
            purpose: AttestationPurpose::ProposalApproval,
            subject_proposal_receipt_digest: digest(4),
            context_digest: digest(1),
            actor_assertion: "reviewer".to_string(),
            decisions: vec![
                ClaimAdmissionDecision::Approve {
                    claim_ordinal: IJsonUInt::new(1)?,
                },
                ClaimAdmissionDecision::Reject {
                    claim_ordinal: IJsonUInt::new(2)?,
                    reason: ClaimRejectionReason::UserRejected,
                },
            ],
        },
        admitted_claims: vec![AdmittedClaim {
            ordinal: IJsonUInt::new(1)?,
            text: "The sky is blue.".to_string(),
            proposed_claim_digest: digest(6),
            operation: AdmittedClaimOperation::Approve,
        }],
        rejected_claims: vec![RejectedClaim {
            ordinal: IJsonUInt::new(2)?,
            proposed_claim_digest: digest(7),
            reason: ClaimRejectionReason::UserRejected,
        }],
    })
}

#[test]
fn proposal_payload_round_trips_and_carries_no_payload_kind() -> Result<(), anyhow::Error> {
    let payload = proposal_payload()?;
    let value = serde_json::to_value(&payload)?;
    let map = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("payload did not serialize to object"))?;
    assert!(!map.contains_key("payload_kind"));
    assert_eq!(value["schema"], "vr.workflow.agent_proposal@0.2");
    let parsed: AgentProposalPayloadV2 = serde_json::from_value(value)?;
    assert_eq!(parsed, payload);
    Ok(())
}

#[test]
fn proposal_payload_rejects_v1_payload_kind() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(proposal_payload()?)?;
    value["payload_kind"] = serde_json::json!("workflow.agent_proposal");
    let result: Result<AgentProposalPayloadV2, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn admission_payload_round_trips_and_carries_no_outer_context() -> Result<(), anyhow::Error> {
    let payload = admission_payload()?;
    let value = serde_json::to_value(&payload)?;
    let map = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("payload did not serialize to object"))?;
    assert!(!map.contains_key("payload_kind"));
    assert!(!map.contains_key("context_digest"));
    assert_eq!(value["signal"]["context_digest"], digest(1).to_hex());
    let parsed: ProposalAdmissionPayloadV2 = serde_json::from_value(value)?;
    assert_eq!(parsed, payload);
    Ok(())
}

#[test]
fn admission_payload_rejects_v1_outer_context_digest() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(admission_payload()?)?;
    value["context_digest"] = serde_json::json!(digest(1).to_hex());
    let result: Result<ProposalAdmissionPayloadV2, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn admission_payload_rejects_v1_payload_kind() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(admission_payload()?)?;
    value["payload_kind"] = serde_json::json!("workflow.proposal_admission");
    let result: Result<ProposalAdmissionPayloadV2, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn bundle_v2_round_trips_under_the_format_key() -> Result<(), anyhow::Error> {
    let bundle = bundle()?;
    let value = serde_json::to_value(&bundle)?;
    let map = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("bundle did not serialize to object"))?;
    assert_eq!(map["_format"], "vr-proposal-admission/v2");
    assert!(!map.contains_key("format"));
    assert!(!map.contains_key("proposal_canonical"));
    assert_eq!(value["proposal"]["receipt_digest"], digest(4).to_hex());
    assert_eq!(
        value["admission"]["receipt_type"],
        "vr.workflow.proposal_admission"
    );
    let parsed: ProposalAdmissionBundleV2 = serde_json::from_str(&value.to_string())?;
    assert_eq!(parsed, bundle);
    Ok(())
}

#[test]
fn bundle_v2_rejects_v1_canonical_string_shape() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(bundle()?)?;
    value["proposal_canonical"] = serde_json::json!("{}");
    let result: Result<ProposalAdmissionBundleV2, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn bundle_v2_rejects_a_v1_envelope() -> Result<(), anyhow::Error> {
    let mut value = serde_json::to_value(bundle()?)?;
    value["proposal"] = serde_json::json!({
        "envelope_version": 1,
        "receipt_type": "llm",
        "context_digest": digest(1).to_hex(),
        "schema_digest": digest(2).to_hex(),
        "policy_digest": digest(3).to_hex(),
        "logical_time": "1",
        "event_hash": digest(4).to_hex(),
        "payload": {},
    });
    let result: Result<ProposalAdmissionBundleV2, _> = serde_json::from_value(value);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn admitted_proposal_projection_is_unchanged_in_shape() -> Result<(), anyhow::Error> {
    let projection = AdmittedProposal {
        proposal_receipt_digest: digest(4),
        admission_receipt_digest: digest(8),
        claims: admission_payload()?.admitted_claims,
        rejected_claims: admission_payload()?.rejected_claims,
    };
    let value = serde_json::to_value(&projection)?;
    let keys: std::collections::BTreeSet<&str> = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("projection did not serialize to object"))?
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        std::collections::BTreeSet::from([
            "admission_receipt_digest",
            "claims",
            "proposal_receipt_digest",
            "rejected_claims",
        ])
    );
    Ok(())
}

#[test]
fn bundle_format_label_is_v2() {
    assert_eq!(
        PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2,
        "vr-proposal-admission/v2"
    );
}
