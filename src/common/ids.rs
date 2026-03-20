//! Opaque policy identifier with validated grammar.

use serde::{Deserialize, Deserializer, Serialize};

use crate::DefinitionError;

/// An opaque policy identifier.
///
/// Policy identifiers are validated to ensure they are non-empty,
/// bounded in length, and contain only portable visible characters.
///
/// The admitted grammar is `[A-Za-z0-9._:/@-]{1,128}`. Leading and
/// trailing whitespace is rejected.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct PolicyId(String);

impl PolicyId {
    /// Maximum permitted length of a policy identifier.
    pub const MAX_LEN: usize = 128;

    /// Create a validated [`PolicyId`] from a string.
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidPolicyId`] if the string is
    /// empty, exceeds [`Self::MAX_LEN`], contains leading/trailing
    /// whitespace, or contains characters outside the portable grammar.
    pub fn new(id: String) -> Result<Self, DefinitionError> {
        validate_policy_id(&id)?;
        Ok(Self(id))
    }

    /// Return the inner string as a slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for PolicyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        Self::new(id).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for PolicyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Validate a policy identifier against the portable grammar.
fn validate_policy_id(id: &str) -> Result<(), DefinitionError> {
    if id.is_empty() {
        return Err(DefinitionError::InvalidPolicyId(
            "must not be empty".to_string(),
        ));
    }

    if id.len() > PolicyId::MAX_LEN {
        return Err(DefinitionError::InvalidPolicyId(format!(
            "exceeds max length of {}",
            PolicyId::MAX_LEN
        )));
    }

    if id.trim() != id {
        return Err(DefinitionError::InvalidPolicyId(
            "contains leading or trailing whitespace".to_string(),
        ));
    }

    for (index, ch) in id.char_indices() {
        let valid = ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '/' | '-' | '@');
        if !valid {
            return Err(DefinitionError::InvalidPolicyId(format!(
                "invalid character `{ch}` at byte index {index}"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "ids_tests.rs"]
mod ids_tests;
