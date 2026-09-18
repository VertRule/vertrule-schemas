//! Tests for `PayloadSchemaV2`.
//!
//! The identities are constitutional hex (ADR-054 §7). The re-derivation
//! oracle below uses the raw primitive on purpose (test-only, F9): it
//! confirms the frozen bytes against the class law rather than routing
//! through any sealed constructor.

use crate::{PayloadSchemaV2, SchemaId};

const SCHEMA_LABEL_CONTEXT: &str = "vertrule.identity.schema-label.v1";

const ALL: [PayloadSchemaV2; 2] = [
    PayloadSchemaV2::VR_SURFACE_DECISION_0_1,
    PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1,
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
fn decision_identity_equals_adr054_vector() {
    // ADR-054 §7: SchemaLabel / vertrule.identity.schema-label.v1 / vr.surface.decision@0.1
    assert_eq!(
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1.identity().to_hex(),
        "48b92179bcf7f3663e761ef7c87e1378dd53c9d6d9f3a763bef9756b6151f460"
    );
    assert_eq!(
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1.label(),
        "vr.surface.decision@0.1"
    );
}

#[test]
fn runtime_port_identity_is_pinned() {
    assert_eq!(
        PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1
            .identity()
            .to_hex(),
        "ffc6e9a7710bc35c196819607a1f269480efe36bf85f7cd64c6ed7d10e554b49"
    );
    assert_eq!(
        PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1.label(),
        "vr.runtime_port.submit_outcome@0.1"
    );
}

#[test]
fn identities_re_derive_under_the_schema_label_class() {
    for schema in ALL {
        let derived = blake3::derive_key(SCHEMA_LABEL_CONTEXT, schema.label().as_bytes());
        assert_eq!(
            derived,
            *schema.identity().as_bytes(),
            "{} drifted from derive_key({SCHEMA_LABEL_CONTEXT:?}, label)",
            schema.label()
        );
    }
}

#[test]
fn identities_are_distinct() {
    assert_ne!(
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1.identity(),
        PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1.identity()
    );
}

#[test]
fn identity_is_not_the_legacy_raw_label_digest() {
    // ADR-054 §7 legacy L1: BLAKE3("vr.surface.decision@0.1") = 4c5d6d82…7852.
    let legacy = blake3::hash(b"vr.surface.decision@0.1");
    assert_ne!(
        legacy.as_bytes(),
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1
            .identity()
            .as_bytes()
    );
}
