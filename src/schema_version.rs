//! Schema version tag.
//!
//! Each schema version defines an identity triple:
//! `(spec_version, canonicalization, commitment_primitive)`.
//! The identity triple determines thermodynamic identity class —
//! two envelopes belong to the same class if and only if they share
//! the same triple.

use serde::{Deserialize, Serialize};

/// A schema version tag.
///
/// Wraps a `u32` to provide type safety for version comparisons.
///
/// Each version binds an identity triple of
/// `(spec_version, canonicalization, commitment_primitive)`.
/// Version 1 binds JCS canonicalization and BLAKE3 commitment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(u32);

impl SchemaVersion {
    /// Schema version 1 identity triple: JCS canonicalization, BLAKE3 commitment.
    pub const V1: Self = Self(1);

    /// Create a new [`SchemaVersion`].
    #[must_use]
    pub const fn new(version: u32) -> Self {
        Self(version)
    }

    /// Return the inner version number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
