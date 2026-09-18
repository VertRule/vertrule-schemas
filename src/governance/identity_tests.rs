//! Byte-stability tests for the sealed governance-identity newtype.
//!
//! The scope digest is computed two ways:
//! 1. via the sealed type's constructor
//! 2. via the legacy `BLAKE3(JCS(scope))` path inline (mirroring
//!    `governance/decision.rs`'s pre-Gate-2 implementation)
//!
//! Assertion: byte-for-byte identical. The raw-label byte-stability
//! tests that once lived here left with `PolicyDigest` / `SchemaDigest`
//! (ADR-054 G1-5); the legacy L1 known-answer is pinned in ADR-054 §7.

use vr_jcs::DigestStrategy;

use super::identity::ScopeDigest;
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
