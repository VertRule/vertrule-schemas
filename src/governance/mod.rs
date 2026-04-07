//! Governance surface nouns — constitutional types for multi-surface governance.
//!
//! This module defines the adapter-neutral ontology for governed events:
//! scope, subject, action, adapter reference, decision payload, and
//! policy binding reference.
//!
//! # Architectural invariant
//!
//! ```text
//! canonical_governance_identity ⟂ adapter_origin
//! ```
//!
//! Adapter-local coordinates (`issue_key`, `run_id`, `site_id`) are lookup
//! metadata only. They never root governance identity, policy binding,
//! storage keys, or idempotency. All adapter-local coordinates are
//! confined to [`AdapterReference::external_keys`].
//!
//! # Placement rule
//!
//! Only constitutional nouns belong in this module:
//!
//! - Pure data types with validated construction.
//! - Serde support with deterministic serialization.
//! - No I/O, no orchestration, no normalization logic.
//! - No idempotency computation, no storage, no adapter-specific code.
//!
//! Normalization logic, storage traits, orchestration pipelines, and
//! adapter-specific code live in `vr-adapter-boundary`, adapter crates,
//! and product crates respectively.
//!
//! # Surface neutrality
//!
//! Every type in this module must make sense for Jira, `LangChain`,
//! `ServiceNow`, Slack, and arbitrary future surfaces. If a field name
//! or type shape is meaningful only for one surface, it does not belong
//! here.

pub mod action;
pub mod adapter;
pub mod binding;
pub mod decision;
pub mod scope;
pub mod subject;

pub use action::{ActionNamespace, GovernedAction};
pub use adapter::{AdapterOrigin, AdapterReference};
pub use binding::{PolicyBindingRef, PolicyTemplate};
pub use decision::{GovernedDecisionPayload, Verdict};
pub use scope::{GovernancePrincipalId, GovernanceScope, SurfaceInstanceId};
pub use subject::{EntityNamespace, GovernedSubject};

use crate::DefinitionError;

/// Maximum length for opaque governance identifiers.
///
/// Applies to [`GovernancePrincipalId`] and [`SurfaceInstanceId`].
const MAX_OPAQUE_ID_LEN: usize = 128;

/// Maximum length for namespace identifiers.
///
/// Applies to [`EntityNamespace`] and [`ActionNamespace`].
const MAX_NAMESPACE_LEN: usize = 64;

/// Validate an opaque governance identifier.
///
/// Grammar: `[A-Za-z0-9._:-]{1,128}`.
///
/// `label` is included in error messages to identify which field failed.
pub(crate) fn validate_opaque_id(value: &str, label: &str) -> Result<(), DefinitionError> {
    if value.is_empty() {
        return Err(DefinitionError::InvalidGovernanceId(format!(
            "{label} must not be empty"
        )));
    }
    if value.len() > MAX_OPAQUE_ID_LEN {
        return Err(DefinitionError::InvalidGovernanceId(format!(
            "{label} exceeds max length of {MAX_OPAQUE_ID_LEN}"
        )));
    }
    for (i, ch) in value.char_indices() {
        let valid = ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '-');
        if !valid {
            return Err(DefinitionError::InvalidGovernanceId(format!(
                "{label}: invalid character `{ch}` at byte index {i}"
            )));
        }
    }
    Ok(())
}

/// Validate a namespace identifier.
///
/// Grammar: `[a-z][a-z0-9_]{0,63}`.
///
/// `label` is included in error messages to identify which field failed.
pub(crate) fn validate_namespace(value: &str, label: &str) -> Result<(), DefinitionError> {
    let mut chars = value.chars();

    let Some(first) = chars.next() else {
        return Err(DefinitionError::InvalidNamespace(format!(
            "{label} must not be empty"
        )));
    };

    if !first.is_ascii_lowercase() {
        return Err(DefinitionError::InvalidNamespace(format!(
            "{label} must start with [a-z], got `{first}`"
        )));
    }

    if value.len() > MAX_NAMESPACE_LEN {
        return Err(DefinitionError::InvalidNamespace(format!(
            "{label} exceeds max length of {MAX_NAMESPACE_LEN}"
        )));
    }

    for (i, ch) in value.char_indices().skip(1) {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_';
        if !valid {
            return Err(DefinitionError::InvalidNamespace(format!(
                "{label}: invalid character `{ch}` at byte index {i}"
            )));
        }
    }

    Ok(())
}
