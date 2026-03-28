//! Schema version tag.
//!
//! Each schema version binds a schema identity triple:
//! `(spec_version, canonicalization, commitment_primitive)`.
//! Two envelopes are in the same verification domain only if
//! these bindings match.

use serde::{Deserialize, Deserializer, Serialize};

use crate::DefinitionError;

/// A schema version tag.
///
/// Each version binds a schema identity triple of
/// `(spec_version, canonicalization, commitment_primitive)` and a
/// commitment scope that defines which fields `event_hash` commits.
///
/// - **V1**: `event_hash` = `BLAKE3(JCS(payload))` — payload only
/// - **V2**: `event_hash` = `BLAKE3(JCS(envelope \ {event_hash}))` — all fields
///
/// Construction rejects unsupported version numbers. Only versions
/// with defined identity bindings can be represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SchemaVersion(u32);

impl SchemaVersion {
    /// Schema version 1: BLAKE3+JCS, payload-only commitment.
    pub const V1: Self = Self(1);

    /// Schema version 2: BLAKE3+JCS, full-envelope commitment.
    ///
    /// `event_hash` commits every trust-bearing field: `receipt_type`,
    /// `context_digest`, `schema_digest`, `policy_digest`, `logical_time`,
    /// `parent_id`, `boundary_origin`, and `payload`. Mutating any field
    /// without recomputing `event_hash` fails verification.
    pub const V2: Self = Self(2);

    /// Create a validated [`SchemaVersion`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::UnsupportedVersion`] if the version
    /// number does not have a defined identity binding.
    pub const fn new(version: u32) -> Result<Self, DefinitionError> {
        match version {
            1 | 2 => Ok(Self(version)),
            _ => Err(DefinitionError::UnsupportedVersion(version)),
        }
    }

    /// Return the inner version number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Return the commitment primitive bound to this version.
    #[must_use]
    pub const fn digest_algorithm(self) -> &'static str {
        match self.0 {
            1 | 2 => "BLAKE3",
            _ => "BLAKE3",
        }
    }

    /// Return the canonicalization scheme bound to this version.
    #[must_use]
    pub const fn canonicalization(self) -> &'static str {
        match self.0 {
            1 | 2 => "JCS",
            _ => "JCS",
        }
    }

    /// Whether this version commits the full envelope (not just payload).
    #[must_use]
    pub const fn commits_full_envelope(self) -> bool {
        self.0 >= 2
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let version = u32::deserialize(deserializer)?;
        Self::new(version).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[path = "versions_tests.rs"]
mod versions_tests;
