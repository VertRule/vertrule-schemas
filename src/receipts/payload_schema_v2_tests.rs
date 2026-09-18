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
fn identities_are_distinct() -> Result<(), anyhow::Error> {
    assert_ne!(
        PayloadSchemaV2::VR_SURFACE_DECISION_0_1.identity()?,
        PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1.identity()?
    );
    Ok(())
}
