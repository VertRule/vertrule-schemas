//! Sealed governance-identity newtypes.
//!
//! One canonical-JSON identity type survives here:
//!
//! | Class | Type | Strategy |
//! |---|---|---|
//! | Canonical JSON identity | [`ScopeDigest`] | `vr-jcs` strategy-bearing digest |
//!
//! The two raw-label types that once shared this module (`PolicyDigest`
//! over a binding id and `SchemaDigest::for_decision_v0_1`, both
//! `BLAKE3(label_bytes)` — the ADR-054 L1 law) were removed by the ADR-054
//! G1-5 census: no production or verifier path called them (L1 is
//! `DeclarationOnly`; no verifier recomputes it), and the raw-label class
//! law now lives in the sealed `vr-identity` slot
//! (`vr_identity::digest::SchemaLabelIdentity`, ADR-057 §2.4–2.7). The
//! legacy L1 known-answer for `vr.surface.decision@0.1` is documented in
//! ADR-054 §7 and the G1 BEFORE fixture only.
//!
//! The type has a private field and one domain-specific constructor. No
//! `From<[u8; 32]>` is provided.

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
