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
/// `event_hash` = `BLAKE3(JCS(envelope \ {event_hash}))` — all trust-bearing
/// fields are committed.
///
/// Construction rejects unsupported version numbers. Only versions
/// with defined identity bindings can be represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SchemaVersion(u32);

impl SchemaVersion {
    /// The current and only supported schema version.
    ///
    /// Identity triple: BLAKE3 + JCS.
    /// Commitment scope: full envelope (`event_hash` commits every
    /// trust-bearing field).
    pub const V1: Self = Self(1);

    /// Create a validated [`SchemaVersion`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::UnsupportedVersion`] if the version
    /// number does not have a defined identity binding.
    pub const fn new(version: u32) -> Result<Self, DefinitionError> {
        match version {
            1 => Ok(Self(version)),
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
        "BLAKE3"
    }

    /// Return the canonicalization scheme bound to this version.
    #[must_use]
    pub const fn canonicalization(self) -> &'static str {
        "JCS"
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
