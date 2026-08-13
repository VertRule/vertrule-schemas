//! Sealed governance-identity newtypes.
//!
//! Three domain-specific digest types covering the three identity
//! classes the Hardening Plan distinguishes:
//!
//! | Class | Type | Strategy |
//! |---|---|---|
//! | Canonical JSON identity | [`ScopeDigest`] | `vr-jcs` strategy-bearing digest |
//! | Raw label identity (binding ID) | [`PolicyDigest`] | `BLAKE3(label_bytes)` directly |
//! | Raw label identity (schema constant) | [`SchemaDigest`] | `BLAKE3(constant_bytes)` directly |
//!
//! The two raw-label types do **NOT** route through `vr-jcs`. Their
//! input IS the canonical representation — there is no JSON shape to
//! canonicalize. A future format-change ADR may migrate them to
//! `DigestStrategy::blake3_domain_separated` for spec-conformant
//! domain separation, but that would change the bytes and is out of
//! scope for Gate 2 (which preserves byte-stability with the legacy
//! `decision.rs` implementations).
//!
//! All three types have private fields and only domain-specific
//! constructors. No `From<[u8; 32]>` is provided.

use vr_jcs::{CanonicalDigest, DigestAlgorithm, DigestStrategy};

use crate::canonical_identity::digest_trusted_value;
use crate::governance::scope::GovernanceScope;
use crate::{DefinitionError, DigestBytes};

// ── ScopeDigest ───────────────────────────────────────────────────
// Canonical JSON identity: routes through vr-jcs.

/// Sealed canonical-JSON digest of a [`GovernanceScope`].
///
/// Wraps [`CanonicalDigest`] so callers cannot confuse a scope digest
/// with any other 32-byte value. The inner field is private; the only
/// constructor is [`ScopeDigest::from_governance_scope`].
///
/// Byte-stable with the legacy `compute_scope_digest` function in
/// `governance/decision.rs` (Gate 2 preservation requirement).
#[derive(Debug, Clone)]
pub struct ScopeDigest {
    inner: CanonicalDigest,
}

impl ScopeDigest {
    /// Compute the scope digest: `BLAKE3(JCS(scope))`.
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::Jcs`] if canonicalization fails.
    pub fn from_governance_scope(scope: &GovernanceScope) -> Result<Self, DefinitionError> {
        let value = serde_json::to_value(scope).map_err(crate::jcs::JcsError::from)?;
        let inner = digest_trusted_value(&value, &DigestStrategy::blake3_untagged())?;
        Ok(Self { inner })
    }

    /// Stable algorithm-name identifier (`"blake3-untagged"`) for
    /// receipt-schema metadata.
    #[must_use]
    pub const fn algorithm_name(&self) -> &'static str {
        self.inner.algorithm.name()
    }

    /// Borrow the underlying [`DigestAlgorithm`].
    #[must_use]
    pub const fn algorithm(&self) -> &DigestAlgorithm {
        &self.inner.algorithm
    }

    /// Borrow the raw digest bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.inner.bytes
    }

    /// Project to the wire-format [`DigestBytes`] shape.
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidDigest`] if the digest is not
    /// exactly 32 bytes.
    pub fn as_digest_bytes(&self) -> Result<DigestBytes, DefinitionError> {
        DigestBytes::from_slice(&self.inner.bytes)
    }

    /// Consume and return the algorithm-bearing [`CanonicalDigest`].
    #[must_use]
    pub fn into_canonical_digest(self) -> CanonicalDigest {
        self.inner
    }
}

// ── PolicyDigest ──────────────────────────────────────────────────
// Raw label identity: BLAKE3 over the binding ID bytes, NOT JCS.

/// Sealed policy-binding identity digest.
///
/// Raw label identity: derived from `BLAKE3(binding_id.as_bytes())`.
/// This is **not** a JCS digest — the binding ID string is its own
/// canonical representation; there is no JSON shape involved.
///
/// Byte-stable with the legacy `compute_policy_digest` function in
/// `governance/decision.rs`.
///
/// A future format-change ADR may migrate to
/// `DigestStrategy::blake3_domain_separated("vr.policy.binding")` for
/// BLAKE3-spec domain separation, but that would change the bytes and
/// is out of scope for Gate 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicyDigest {
    bytes: [u8; 32],
}

impl PolicyDigest {
    /// Compute the policy digest for a binding ID:
    /// `BLAKE3(binding_id.as_bytes())`.
    ///
    /// # ALLOW-JCS-BYPASS
    ///
    /// Raw label identity, not canonical JSON identity. The binding ID
    /// is the canonical form; no JCS round-trip applies. Per the
    /// Hardening Plan's three-class identity model, this is a legitimate
    /// non-JCS digest contract.
    #[must_use]
    pub fn from_binding_id(binding_id: &str) -> Self {
        // ALLOW-JCS-BYPASS: raw label identity, not canonical JSON identity.
        // Derivation authority sealed 2026-08-11; law unchanged.
        Self {
            bytes: *vertrule_crypto::identity::OpaqueBytesDigest::compute(binding_id.as_bytes())
                .bytes(),
        }
    }

    /// Borrow the raw digest bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Project to the wire-format [`DigestBytes`] shape.
    #[must_use]
    pub const fn as_digest_bytes(&self) -> DigestBytes {
        DigestBytes::from_array(self.bytes)
    }
}

// ── SchemaDigest ──────────────────────────────────────────────────
// Raw label identity: BLAKE3 over a constant schema label, NOT JCS.

/// Sealed schema-identity digest for `VertRule` constitutional schemas.
///
/// Raw label identity: each constructor uses `BLAKE3` over a constant
/// byte label. **Not** a JCS digest — the label is the identity.
///
/// Byte-stable with the legacy `schema_decision_digest` function in
/// `governance/decision.rs`.
///
/// A future format-change ADR may migrate to
/// `DigestStrategy::blake3_domain_separated(label)` for BLAKE3-spec
/// domain separation, but that would change the bytes and is out of
/// scope for Gate 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaDigest {
    bytes: [u8; 32],
}

impl SchemaDigest {
    /// Schema digest for `vr.surface.decision@0.1`.
    ///
    /// Bytes: `BLAKE3(b"vr.surface.decision@0.1")`.
    ///
    /// # ALLOW-JCS-BYPASS
    ///
    /// Raw label identity, not canonical JSON identity.
    #[must_use]
    pub fn for_decision_v0_1() -> Self {
        // ALLOW-JCS-BYPASS: raw label identity, not canonical JSON identity.
        // Derivation authority sealed 2026-08-11; law unchanged.
        Self {
            bytes: *vertrule_crypto::identity::OpaqueBytesDigest::compute(
                b"vr.surface.decision@0.1",
            )
            .bytes(),
        }
    }

    /// Borrow the raw digest bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Project to the wire-format [`DigestBytes`] shape.
    #[must_use]
    pub const fn as_digest_bytes(&self) -> DigestBytes {
        DigestBytes::from_array(self.bytes)
    }
}
