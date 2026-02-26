//! Schema version tag.

use serde::{Deserialize, Serialize};

/// A schema version tag.
///
/// Wraps a `u32` to provide type safety for version comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(u32);

impl SchemaVersion {
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
