//! Governance scope — the canonical isolation boundary.
//!
//! Every governed event resolves into a [`GovernanceScope`].
//! Storage, policy binding, and receipt chains are all scoped to this.
//! No adapter-local ID appears in these fields.

use serde::{Deserialize, Deserializer, Serialize};

use super::AdapterOrigin;
use crate::DefinitionError;

/// Governance-scoped identity.
///
/// Every governed event resolves into one of these. No field here is
/// adapter-local — adapters populate these via normalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceScope {
    /// Top-level principal (customer/org). Opaque to core.
    pub governance_principal_id: GovernancePrincipalId,
    /// Specific adapter installation or connection.
    pub surface_instance_id: SurfaceInstanceId,
    /// Which adapter produced this scope.
    pub adapter_origin: AdapterOrigin,
    /// Adapter-defined workspace path (e.g. `"jira:SITE:PROJ"`
    /// or `"langchain:WS:GRAPH"`). Hierarchical by convention,
    /// opaque to storage.
    pub workspace_scope: String,
}

/// Opaque principal identifier.
///
/// Validated grammar: `[A-Za-z0-9._:-]{1,128}`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct GovernancePrincipalId(String);

impl GovernancePrincipalId {
    /// Create a validated [`GovernancePrincipalId`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidGovernanceId`] if the value is
    /// empty, exceeds 128 characters, or contains characters outside
    /// the portable grammar.
    pub fn new(value: String) -> Result<Self, DefinitionError> {
        super::validate_opaque_id(&value, "governance_principal_id")?;
        Ok(Self(value))
    }

    /// The identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for GovernancePrincipalId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for GovernancePrincipalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Opaque surface instance identifier.
///
/// Validated grammar: `[A-Za-z0-9._:-]{1,128}`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SurfaceInstanceId(String);

impl SurfaceInstanceId {
    /// Create a validated [`SurfaceInstanceId`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidGovernanceId`] if the value is
    /// empty, exceeds 128 characters, or contains characters outside
    /// the portable grammar.
    pub fn new(value: String) -> Result<Self, DefinitionError> {
        super::validate_opaque_id(&value, "surface_instance_id")?;
        Ok(Self(value))
    }

    /// The identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SurfaceInstanceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for SurfaceInstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "scope_tests.rs"]
mod scope_tests;
