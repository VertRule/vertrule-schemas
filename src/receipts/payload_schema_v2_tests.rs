//! Tests for `PayloadSchemaV2`.
//!
//! The identities are derived through the sealed `vr-identity` slot; the
//! former frozen hex pins are held here as known-answer vectors (ADR-057
//! D5, ADR-054 §7). No raw digest primitive is used here: the class-law
//! oracle and the legacy-law separation witnesses live inside `vr-identity`
//! (`digest/raw_label_tests.rs`), the only crate permitted to call
//! `blake3::derive_key` (ADR-054 §3).

use vr_identity::SchemaId;

use crate::PayloadSchemaV2;

/// ADR-054 §7: `SchemaLabel` / `vertrule.identity.schema-label.v1` /
/// `vr.surface.decision@0.1`.
const VR_SURFACE_DECISION_0_1_IDENTITY: &str =
    "48b92179bcf7f3663e761ef7c87e1378dd53c9d6d9f3a763bef9756b6151f460";

/// Derived once under the same context and pinned (`RECEIPT_V2_BOUNDARY.md`);
/// not an ADR-054 §7 row.
const VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1_IDENTITY: &str =
    "ffc6e9a7710bc35c196819607a1f269480efe36bf85f7cd64c6ed7d10e554b49";

/// Group-2 labels (M2-1, 2026-09-18): derived once under the `SchemaLabel`
/// context through the sealed slot and pinned here as known-answer
/// vectors. None is an ADR-054 §7 row; the §7 vector for
/// `vr.record.verifiable_ai_record@0.1` (`024c849b…7065`) is historical
/// evidence of the class law, not authority for the `@0.2` label (D2).
const GROUP2_IDENTITIES: [(PayloadSchemaV2, &str); 4] = [
    (
        PayloadSchemaV2::VR_AI_PROVIDER_INTERACTION_0_2,
        "a2617fad14ca007315793770e6c7a3bd126e3f0d170659c0898431dddbf56e3e",
    ),
    (
        PayloadSchemaV2::VR_WORKFLOW_AGENT_PROPOSAL_0_2,
        "8d99c4a6cfaf7e584b3ac6ec3cc648d1de9cf3c0b719d8b2833d5682f61c5acc",
    ),
    (
        PayloadSchemaV2::VR_WORKFLOW_PROPOSAL_ADMISSION_0_2,
        "ca219ef2b721f9de13cdff45a116fdc9cd73f18e0ee482b76ebcd328dda8af80",
    ),
    (
        PayloadSchemaV2::VR_RECORD_VERIFIABLE_AI_RECORD_0_2,
        "442646503423f1f0f8ae18b883ced086de87b18363d850627cf24f12dfe82830",
    ),
];

/// ADR-054 §7: `SchemaLabel` identity of the frozen `@0.1` record label —
/// must differ from the `@0.2` identity admitted here.
const VR_RECORD_VERIFIABLE_AI_RECORD_0_1_HISTORICAL: &str =
    "024c849b7df95517c5c0880fb24467f547a63993b0bdd663e14febf928977065";

const ALL: [PayloadSchemaV2; 6] = [
    PayloadSchemaV2::VR_SURFACE_DECISION_0_1,
    PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1,
    PayloadSchemaV2::VR_AI_PROVIDER_INTERACTION_0_2,
    PayloadSchemaV2::VR_WORKFLOW_AGENT_PROPOSAL_0_2,
    PayloadSchemaV2::VR_WORKFLOW_PROPOSAL_ADMISSION_0_2,
    PayloadSchemaV2::VR_RECORD_VERIFIABLE_AI_RECORD_0_2,
];

#[test]
fn labels_are_schema_id_valid() -> Result<(), anyhow::Error> {
    for schema in ALL {
        let id = SchemaId::new(schema.label().to_string())?;
        assert_eq!(id.as_str(), schema.label());
    }
    Ok(())
}

#[test]
fn decision_identity_equals_adr054_vector() -> Result<(), anyhow::Error> {
    assert_eq!(
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1
            .identity()?
            .to_hex(),
        VR_SURFACE_DECISION_0_1_IDENTITY
    );
    assert_eq!(
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1.label(),
        "vr.surface.decision@0.1"
    );
    Ok(())
}

#[test]
fn runtime_port_identity_equals_the_pinned_kat() -> Result<(), anyhow::Error> {
    assert_eq!(
        PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1
            .identity()?
            .to_hex(),
        VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1_IDENTITY
    );
    assert_eq!(
        PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1.label(),
        "vr.runtime_port.submit_outcome@0.1"
    );
    Ok(())
}

#[test]
fn group2_identities_equal_the_pinned_kats() -> Result<(), anyhow::Error> {
    for (schema, expected) in GROUP2_IDENTITIES {
        assert_eq!(schema.identity()?.to_hex(), expected, "{}", schema.label());
    }
    Ok(())
}

#[test]
fn group2_labels_are_the_0_2_versions() {
    let labels: Vec<&str> = GROUP2_IDENTITIES.iter().map(|(s, _)| s.label()).collect();
    assert_eq!(
        labels,
        [
            "vr.ai.provider_interaction@0.2",
            "vr.workflow.agent_proposal@0.2",
            "vr.workflow.proposal_admission@0.2",
            "vr.record.verifiable_ai_record@0.2",
        ]
    );
}

#[test]
fn record_0_2_identity_differs_from_the_historical_0_1_vector() -> Result<(), anyhow::Error> {
    assert_ne!(
        PayloadSchemaV2::VR_RECORD_VERIFIABLE_AI_RECORD_0_2
            .identity()?
            .to_hex(),
        VR_RECORD_VERIFIABLE_AI_RECORD_0_1_HISTORICAL
    );
    Ok(())
}

#[test]
fn identities_are_distinct() -> Result<(), anyhow::Error> {
    let mut seen = std::collections::BTreeSet::new();
    for schema in ALL {
        assert!(
            seen.insert(schema.identity()?),
            "{} collides with an earlier identity",
            schema.label()
        );
    }
    assert_eq!(seen.len(), ALL.len());
    Ok(())
}
