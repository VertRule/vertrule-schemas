//! Adapter origin and external reference types.
//!
//! [`AdapterOrigin`] discriminates which external surface produced an event.
//! [`AdapterReference`] carries adapter-native lookup keys for round-tripping.
//!
//! Neither type participates in governance identity, policy evaluation, or
//! idempotency computation. They are committed by `event_hash` when present
//! in a receipt payload (integrity), but do not route decisions.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod adapter_tests;
