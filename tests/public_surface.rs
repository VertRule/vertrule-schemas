//! Public surface regression test for vertrule-schemas.
//!
//! Asserts that the blessed public API symbols (constitutional nouns only)
//! compile and are usable. Review against `PUBLIC_SURFACE.md` when preparing
//! releases.
//!
//! Notably absent (by design):
//! - JCS functions (live in vr-jcs)
//! - `compute_event_hash` (lives in `vr-receipt-identity`)
//! - `ReceiptEnvelope` methods (nouns only, no construction or judgment)

#![deny(unused_imports)]

// Wire shapes
use vertrule_schemas::ReceiptEnvelope;
use vertrule_schemas::ReceiptEnvelopeV2;

// Discriminators
use vertrule_schemas::BoundaryOrigin;
use vertrule_schemas::ReceiptType;
use vertrule_schemas::ReceiptTypeV2;

// V2 payload-schema identities and passive payload shapes (ADR-056)
use vertrule_schemas::AgentProposalPayloadV2;
use vertrule_schemas::PayloadSchemaV2;
use vertrule_schemas::ProposalAdmissionBundleV2;
use vertrule_schemas::ProposalAdmissionPayloadV2;
use vertrule_schemas::ProviderInteractionPayloadV2;
use vertrule_schemas::RuntimePortCommandKind;
use vertrule_schemas::RuntimePortSubmitOutcomePayload;
use vertrule_schemas::TransitionCommitment;
use vertrule_schemas::VerifiableAiRecordArtifactV3;
use vertrule_schemas::VerifiableAiRecordPayloadV2;
use vertrule_schemas::PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2;
use vertrule_schemas::VERIFIABLE_AI_RECORD_FORMAT_V3;

// Validated scalars
use vertrule_schemas::CanonicalPayload;
use vertrule_schemas::DigestBytes;
use vertrule_schemas::IJsonUInt;
use vertrule_schemas::PolicyId;
use vertrule_schemas::SchemaVersion;

// Context
use vertrule_schemas::RBHInvariant;

// Projection trait
use vertrule_schemas::ProjectsToReceiptEnvelope;

// Error
use vertrule_schemas::DefinitionError;

// MRI domain types
use vertrule_schemas::BatchReduction;
use vertrule_schemas::GradientCouplingPayload;
use vertrule_schemas::MriBatchPayload;
use vertrule_schemas::ReductionAxis;
use vertrule_schemas::ReductionMode;
use vertrule_schemas::ReductionProvenance;
use vertrule_schemas::TokenReduction;

#[test]
fn public_surface_nouns_are_usable() -> Result<(), anyhow::Error> {
    // DigestBytes
    let d = DigestBytes::from_array([0xaa; 32]);
    assert_eq!(DigestBytes::BYTE_LEN, 32);
    assert_eq!(DigestBytes::HEX_LEN, 64);
    assert_eq!(d.as_bytes().len(), 32);

    // IJsonUInt
    let t = IJsonUInt::new(42)?;
    assert_eq!(t.get(), 42);

    // SchemaVersion
    assert_eq!(SchemaVersion::V1.get(), 1);
    assert_eq!(SchemaVersion::V1.digest_algorithm(), "BLAKE3");
    assert_eq!(SchemaVersion::V1.canonicalization(), "JCS");

    // ReceiptType and BoundaryOrigin exist as enums
    let _ = ReceiptType::Governance;
    let _ = BoundaryOrigin::Engine;

    // CanonicalPayload
    let payload = CanonicalPayload::new(serde_json::json!({"k": "v"}))?;
    assert!(payload.as_value().is_object());

    // ReceiptEnvelope is #[non_exhaustive] — construct via deserialization
    let envelope_json = serde_json::json!({
        "envelope_version": 1,
        "receipt_type": "governance",
        "context_digest": d.to_hex(),
        "schema_digest": d.to_hex(),
        "policy_digest": d.to_hex(),
        "logical_time": t.get(),
        "event_hash": d.to_hex(),
        "payload": payload.as_value(),
    });
    let envelope: ReceiptEnvelope = serde_json::from_value(envelope_json)?;
    let _json = serde_json::to_string(&envelope)?;

    // V2 surface (ADR-056): closed type vocabulary, frozen schema identities,
    // and the #[non_exhaustive] envelope constructed via deserialization.
    assert_eq!(SchemaVersion::V2.get(), 2);
    assert_eq!(
        ReceiptTypeV2::GovernanceDecision.label(),
        "vr.governance.decision"
    );
    assert_eq!(ReceiptTypeV2::ADMITTED.len(), 6);
    assert_eq!(
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1.label(),
        "vr.surface.decision@0.1"
    );
    let envelope_v2_json = serde_json::json!({
        "envelope_version": 2,
        "receipt_type": ReceiptTypeV2::GovernanceDecision.label(),
        "schema_digest": PayloadSchemaV2::VR_SURFACE_DECISION_0_1.identity()?.to_hex(),
        "logical_time": "1",
        "payload": payload.as_value(),
        "receipt_digest": d.to_hex(),
    });
    let envelope_v2: ReceiptEnvelopeV2 = serde_json::from_value(envelope_v2_json)?;
    assert_eq!(envelope_v2.context_digest, None);
    let _ = RuntimePortCommandKind::Submit;
    let _ = std::any::type_name::<RuntimePortSubmitOutcomePayload>();
    let _ = std::any::type_name::<TransitionCommitment>();
    // Group-2 V2 shapes (M2-1): four types, four `@0.2` labels, two containers.
    assert_eq!(
        ReceiptTypeV2::AiProviderInteraction.label(),
        "vr.ai.provider_interaction"
    );
    assert_eq!(
        PayloadSchemaV2::VR_RECORD_VERIFIABLE_AI_RECORD_0_2.label(),
        "vr.record.verifiable_ai_record@0.2"
    );
    assert_eq!(
        PROPOSAL_ADMISSION_BUNDLE_FORMAT_V2,
        "vr-proposal-admission/v2"
    );
    assert_eq!(VERIFIABLE_AI_RECORD_FORMAT_V3, "vr-verifiable-ai-record/v3");
    let _ = std::any::type_name::<AgentProposalPayloadV2>();
    let _ = std::any::type_name::<ProposalAdmissionPayloadV2>();
    let _ = std::any::type_name::<ProposalAdmissionBundleV2>();
    let _ = std::any::type_name::<ProviderInteractionPayloadV2>();
    let _ = std::any::type_name::<VerifiableAiRecordPayloadV2>();
    let _ = std::any::type_name::<VerifiableAiRecordArtifactV3>();

    // Suppress unused-import warnings for types used only as existence checks
    let _ = std::any::type_name::<PolicyId>();
    let _ = std::any::type_name::<RBHInvariant>();
    let _ = std::any::type_name::<DefinitionError>();
    let _ = std::any::type_name::<dyn ProjectsToReceiptEnvelope>();

    // MRI domain types exist
    let _ = std::any::type_name::<MriBatchPayload>();
    let _ = std::any::type_name::<GradientCouplingPayload>();
    let _ = std::any::type_name::<ReductionProvenance>();
    let _ = ReductionMode::PerExampleThenMean;
    let _ = ReductionAxis::Token;
    let _ = TokenReduction::Mean;
    let _ = BatchReduction::Mean;

    Ok(())
}
