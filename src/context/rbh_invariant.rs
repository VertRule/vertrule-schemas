//! Constitutional identity continuity constraint across sealed execution contexts.

use serde::{Deserialize, Serialize};

use crate::DigestBytes;

/// Defines a constitutional identity continuity constraint between sealed
/// execution contexts.
///
/// `RBHInvariant` encodes the requirement that a downstream context must
/// reference a specific upstream context, policy, and receipt before execution
/// can proceed. Federation is one deployment topology — the invariant is
/// compositional: identity conservation is closed under composition.
///
/// When present on an envelope, the runtime must verify all three digests
/// match the upstream execution context before admitting execution. When
/// absent, no cross-context constraint applies and canonical bytes are
/// unchanged.
///
/// # Identity Non-Modification Invariant
///
/// `RBHInvariant` does not alter identity derivation. It does not change
/// canonicalization, digest computation, or receipt hashing. It constrains
/// continuation admissibility, not identity itself.
///
/// # Non-Expansion Guarantee
///
/// `RBHInvariant` does not expand identity scope. It cannot introduce new
/// identity material. It only constrains admissibility of continuation
/// based on existing sealed identity. All digest fields reference upstream
/// state that was already sealed — no new identity is derived from the
/// invariant itself.
///
/// # Canonicalization
///
/// All fields are [`DigestBytes`] values serialized as 64 lowercase hex
/// characters. JCS field ordering is deterministic.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RBHInvariant {
    /// BLAKE3 digest of the parent (upstream) sealed execution context.
    pub parent_context_digest: DigestBytes,

    /// BLAKE3 digest of the policy pack required to be in effect.
    pub required_policy_digest: DigestBytes,

    /// BLAKE3 digest of the upstream receipt that must be referenced.
    pub required_receipt_digest: DigestBytes,
}

impl RBHInvariant {
    /// Create a new `RBHInvariant`.
    #[must_use]
    pub const fn new(
        parent_context_digest: DigestBytes,
        required_policy_digest: DigestBytes,
        required_receipt_digest: DigestBytes,
    ) -> Self {
        Self {
            parent_context_digest,
            required_policy_digest,
            required_receipt_digest,
        }
    }
}
