//! Governed action — what is being done to the subject.
//!
//! Surface-neutral. Jira transitions, `LangChain` tool invocations, and
//! Slack approvals all normalize into [`GovernedAction`].

use serde::{Deserialize, Deserializer, Serialize};

use crate::DefinitionError;

/// What is being done to the subject. Surface-neutral.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedAction {
    /// Type namespace for the action (e.g. `"workflow"`, `"agent"`, `"tool"`).
    pub action_namespace: ActionNamespace,
    /// Specific action (e.g. `"transition"`, `"invoke"`, `"approve"`,
    /// `"deny"`).
    pub action_type: String,
    /// Optional adapter-provided hint for idempotency.
    /// Included in key computation if present, but never the sole key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_idempotency_hint: Option<String>,
}

/// Validated action namespace.
///
/// Grammar: `[a-z][a-z0-9_]{0,63}`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ActionNamespace(String);

impl ActionNamespace {
    /// Create a validated [`ActionNamespace`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidNamespace`] if the value is
    /// empty, exceeds 64 characters, does not start with `[a-z]`, or
    /// contains characters outside `[a-z0-9_]`.
    pub fn new(value: String) -> Result<Self, DefinitionError> {
        super::validate_namespace(&value, "action_namespace")?;
        Ok(Self(value))
    }

    /// The namespace string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ActionNamespace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for ActionNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod action_tests;
