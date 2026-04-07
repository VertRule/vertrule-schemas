//! Governed subject — the thing being governed.
//!
//! Surface-neutral. Jira issues, `LangChain` runs, `ServiceNow` incidents
//! all normalize into [`GovernedSubject`].

use serde::{Deserialize, Deserializer, Serialize};

use crate::DefinitionError;

/// The thing being governed. Surface-neutral.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedSubject {
    /// Canonical key within the governance namespace.
    /// Storage keys off this, not the adapter's native ID.
    pub subject_key: String,
    /// Type namespace for the subject (e.g. `"issue"`, `"agent_run"`,
    /// `"tool_call"`).
    pub entity_namespace: EntityNamespace,
    /// Entity identifier within the namespace.
    pub entity_id: String,
}

/// Validated entity namespace.
///
/// Grammar: `[a-z][a-z0-9_]{0,63}`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EntityNamespace(String);

impl EntityNamespace {
    /// Create a validated [`EntityNamespace`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidNamespace`] if the value is
    /// empty, exceeds 64 characters, does not start with `[a-z]`, or
    /// contains characters outside `[a-z0-9_]`.
    pub fn new(value: String) -> Result<Self, DefinitionError> {
        super::validate_namespace(&value, "entity_namespace")?;
        Ok(Self(value))
    }

    /// The namespace string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for EntityNamespace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for EntityNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "subject_tests.rs"]
mod subject_tests;
