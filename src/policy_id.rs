//! Opaque policy identifier.

use serde::{Deserialize, Serialize};

/// An opaque policy identifier.
///
/// Wraps a `String` to provide type safety for policy references
/// without exposing internal policy resolution semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PolicyId(String);

impl PolicyId {
    /// Create a new [`PolicyId`] from a string.
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self(id)
    }

    /// Return the inner string as a slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PolicyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
