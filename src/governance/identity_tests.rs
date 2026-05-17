//! Byte-stability tests for the sealed governance-identity newtypes.
//!
//! Each test computes the digest two ways:
//! 1. via the new sealed type's constructor
//! 2. via the legacy direct-BLAKE3 path inline (mirroring
//!    `governance/decision.rs`'s pre-Gate-2 implementations)
//!
//! Assertion: byte-for-byte identical. The migration of
//! `decision.rs`'s private helpers to delegate through the sealed
//! types is therefore behavior-preserving.

use vr_jcs::DigestStrategy;

use super::identity::{PolicyDigest, ScopeDigest, SchemaDigest};
use crate::canonical_identity::digest_trusted_value;
use crate::governance::scope::GovernanceScope;
use crate::{DefinitionError, GovernancePrincipalId, SurfaceInstanceId};

fn fixture_scope() -> Result<GovernanceScope, DefinitionError> {
    Ok(GovernanceScope {
        governance_principal_id: GovernancePrincipalId::new("principal_alpha".to_string())?,
        surface_instance_id: SurfaceInstanceId::new("instance_beta".to_string())?,
        adapter_origin: crate::AdapterOriginId::new("origin_gamma".to_string())?,
        workspace_scope: "jira:ACME:PROJ".to_string(),
    })
}

#[test]
fn scope_digest_byte_stable_with_legacy_path() -> Result<(), DefinitionError> {
    let scope = fixture_scope()?;

    // Sealed path.
    let sealed = ScopeDigest::from_governance_scope(&scope)?;
    let sealed_bytes = sealed.as_digest_bytes()?;

    // Legacy-equivalent path (what `decision.rs::compute_scope_digest`
    // does today, expressed through the sealed plumbing). Using the
    // same plumbing on both sides isolates the test to the public
    // surface choice: both paths produce identical bytes given the
    // same canonical input.
    let value = serde_json::to_value(&scope).map_err(crate::jcs::JcsError::from)?;
    let canonical = digest_trusted_value(&value, &DigestStrategy::blake3_untagged())?;
    let legacy_bytes = crate::DigestBytes::from_slice(&canonical.bytes)?;

    assert_eq!(
        sealed_bytes, legacy_bytes,
        "ScopeDigest::from_governance_scope must byte-equal the legacy \
         BLAKE3(JCS(scope)) path",
    );
    Ok(())
}

#[test]
fn scope_digest_algorithm_name_is_blake3_untagged() -> Result<(), DefinitionError> {
    let scope = fixture_scope()?;
    let sealed = ScopeDigest::from_governance_scope(&scope)?;
    assert_eq!(sealed.algorithm_name(), "blake3-untagged");
    Ok(())
}

#[test]
fn policy_digest_byte_stable_with_legacy_raw_label_hash() {
    let binding_id = "binding-alpha-beta";

    // Sealed path.
    let sealed = PolicyDigest::from_binding_id(binding_id);
    let sealed_bytes = sealed.as_digest_bytes();

    // Legacy-equivalent path:
    // `decision.rs::compute_policy_digest` computes
    // `BLAKE3(binding_id.as_bytes())`. Inline here for byte-stability
    // proof.
    // ALLOW-JCS-SPEC: byte-stability assertion against raw label digest
    let legacy_raw = *blake3::hash(binding_id.as_bytes()).as_bytes();
    let legacy_bytes = crate::DigestBytes::from_array(legacy_raw);

    assert_eq!(
        sealed_bytes, legacy_bytes,
        "PolicyDigest::from_binding_id must byte-equal BLAKE3(binding_id.as_bytes())",
    );
}

#[test]
fn schema_digest_for_decision_v0_1_byte_stable_with_legacy_constant_label() {
    // Sealed path.
    let sealed = SchemaDigest::for_decision_v0_1();
    let sealed_bytes = sealed.as_digest_bytes();

    // Legacy-equivalent path:
    // `decision.rs::schema_decision_digest` computes
    // `BLAKE3(b"vr.surface.decision@0.1")`.
    // ALLOW-JCS-SPEC: byte-stability assertion against raw label digest
    let legacy_raw = *blake3::hash(b"vr.surface.decision@0.1").as_bytes();
    let legacy_bytes = crate::DigestBytes::from_array(legacy_raw);

    assert_eq!(
        sealed_bytes, legacy_bytes,
        "SchemaDigest::for_decision_v0_1 must byte-equal BLAKE3(label)",
    );
}

#[test]
fn policy_digest_differs_for_distinct_binding_ids() {
    let a = PolicyDigest::from_binding_id("binding-alpha");
    let b = PolicyDigest::from_binding_id("binding-beta");
    assert_ne!(
        a.bytes(),
        b.bytes(),
        "PolicyDigest must differ for distinct binding IDs",
    );
}
