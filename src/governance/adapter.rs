//! Adapter origin and external reference types.
//!
//! [`AdapterOrigin`] (legacy enum) and [`AdapterOriginId`] (validated
//! newtype, its replacement) each discriminate the external surface that
//! produced an event. [`AdapterReference`] carries adapter-native lookup
//! keys for round-tripping.
//!
//! # Commitment participation
//!
//! The adapter origin **does** participate in receipt commitment. When
//! embedded in a [`GovernanceScope`], it flows through `compute_scope_digest`
//! into `context_digest`, which `event_hash` then commits via
//! `BLAKE3(JCS(envelope \ {event_hash}))`. Any change to its serialized
//! form changes the canonical scope bytes, `context_digest`, and
//! `event_hash`.
//!
//! # Separation of commitment from routing
//!
//! Commitment does **not** imply routing, policy evaluation, or
//! idempotency-key derivation. A consumer may choose to match on the
//! adapter origin to dispatch normalization logic (as the gateway
//! orchestrator does today), but that is a consumer choice, not a
//! property of these types. `AdapterReference::external_keys` remains
//! strictly lookup metadata and never feeds commitment, routing, or
//! idempotency on its own.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::DefinitionError;

/// Which external surface produced this event.
///
/// Used for routing, display, and adapter-index partitioning.
/// Never used as a governance identity root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AdapterOrigin {
    /// Atlassian Jira / JSM.
    Jira,
    /// `LangChain` / `LangGraph`.
    LangChain,
    /// `ServiceNow`.
    ServiceNow,
    /// Salesforce.
    Salesforce,
    /// Slack.
    Slack,
    /// Custom adapter with freeform identifier.
    Custom(String),
}

impl std::fmt::Display for AdapterOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Jira => f.write_str("jira"),
            Self::LangChain => f.write_str("lang_chain"),
            Self::ServiceNow => f.write_str("service_now"),
            Self::Salesforce => f.write_str("salesforce"),
            Self::Slack => f.write_str("slack"),
            Self::Custom(s) => write!(f, "custom({s})"),
        }
    }
}

/// External coordinate bag. Lookup metadata only.
///
/// Never participates in policy evaluation or idempotency computation.
/// Allows round-tripping from governance receipt back to adapter-native
/// object.
///
/// `BTreeMap` for deterministic serialization order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterReference {
    /// Which adapter produced these coordinates.
    pub adapter_origin: AdapterOrigin,
    /// Adapter-native keys (e.g.
    /// `{"issue_key": "PROJ-123", "site_id": "abc"}`).
    pub external_keys: BTreeMap<String, String>,
}

/// Validated adapter-origin identifier.
///
/// Surface-neutral newtype over a validated namespace string. Introduced
/// alongside [`AdapterOrigin`] as the replacement shape that will eventually
/// supersede the variant-centric enum. During the transition both types
/// coexist; construction of `AdapterOriginId` goes through [`Self::new`],
/// which enforces the grammar below.
///
/// Grammar: `[a-z][a-z0-9_]{0,63}`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct AdapterOriginId(String);

impl AdapterOriginId {
    /// Create a validated [`AdapterOriginId`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidNamespace`] if the value is
    /// empty, exceeds 64 characters, does not start with `[a-z]`, or
    /// contains characters outside `[a-z0-9_]`.
    pub fn new(value: String) -> Result<Self, DefinitionError> {
        super::validate_namespace(&value, "adapter_origin_id")?;
        Ok(Self(value))
    }

    /// The identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convenience constructor for the `"jira"` surface.
    ///
    /// # Errors
    ///
    /// Delegates to [`Self::new`]. The literal `"jira"` is grammar-valid,
    /// so this call cannot fail in practice; the `Result` is propagated
    /// to preserve the single-validation-path invariant (no `unwrap`, no
    /// second validator).
    pub fn jira() -> Result<Self, DefinitionError> {
        Self::new("jira".to_string())
    }

    /// Convenience constructor for the `"lang_chain"` surface.
    ///
    /// # Errors
    ///
    /// Delegates to [`Self::new`]; cannot fail for this literal. See
    /// [`Self::jira`] for the shared single-validation-path rationale.
    pub fn lang_chain() -> Result<Self, DefinitionError> {
        Self::new("lang_chain".to_string())
    }

    /// Convenience constructor for the `"service_now"` surface.
    ///
    /// # Errors
    ///
    /// Delegates to [`Self::new`]; cannot fail for this literal. See
    /// [`Self::jira`] for the shared single-validation-path rationale.
    pub fn service_now() -> Result<Self, DefinitionError> {
        Self::new("service_now".to_string())
    }

    /// Convenience constructor for the `"salesforce"` surface.
    ///
    /// # Errors
    ///
    /// Delegates to [`Self::new`]; cannot fail for this literal. See
    /// [`Self::jira`] for the shared single-validation-path rationale.
    pub fn salesforce() -> Result<Self, DefinitionError> {
        Self::new("salesforce".to_string())
    }

    /// Convenience constructor for the `"slack"` surface.
    ///
    /// # Errors
    ///
    /// Delegates to [`Self::new`]; cannot fail for this literal. See
    /// [`Self::jira`] for the shared single-validation-path rationale.
    pub fn slack() -> Result<Self, DefinitionError> {
        Self::new("slack".to_string())
    }

    /// Convenience constructor for the `"webhook"` surface.
    ///
    /// # Errors
    ///
    /// Delegates to [`Self::new`]; cannot fail for this literal. See
    /// [`Self::jira`] for the shared single-validation-path rationale.
    pub fn webhook() -> Result<Self, DefinitionError> {
        Self::new("webhook".to_string())
    }
}

impl<'de> Deserialize<'de> for AdapterOriginId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for AdapterOriginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod adapter_tests;
