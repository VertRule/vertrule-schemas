//! Policy binding reference — the noun, not the resolution logic.
//!
//! [`PolicyBindingRef`] maps governance coordinates to a policy.
//! Resolution and matching (wildcard handling, specificity ordering)
//! live in product/runtime crates, not here.

use serde::{Deserialize, Serialize};

use crate::PolicyId;

/// Policy binding reference.
///
/// Maps governance coordinates to a policy. This is the serializable
/// noun — resolution logic (wildcard matching, specificity ordering,
/// inheritance) lives in the gateway or runtime crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyBindingRef {
    /// Unique binding identifier.
    pub binding_id: String,
    /// Workspace scope this binding applies to.
    pub workspace_scope: String,
    /// Entity namespace filter. `None` = all namespaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_namespace: Option<String>,
    /// Action type filter. `None` = all action types.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
    /// Policy to evaluate when this binding matches.
    pub policy_id: PolicyId,
    /// Fixed v1 template.
    pub policy_template: PolicyTemplate,
}

/// Fixed v1 policy templates.
///
/// Each template maps to a deterministic evaluation path.
/// Custom rule languages are a post-GA concern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PolicyTemplate {
    /// Requires approval signal before allowing action.
    RequireApproval,
    /// Deny if subject lacks required fields.
    RequireFields {
        /// Field names that must be present.
        fields: Vec<String>,
    },
    /// Attach evidence on action (always allow, always receipt).
    AttachEvidence,
    /// Deny unconditionally with reason.
    DenyWithReason {
        /// Denial reason.
        reason: String,
    },
}

impl std::fmt::Display for PolicyTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequireApproval => f.write_str("require_approval"),
            Self::RequireFields { .. } => f.write_str("require_fields"),
            Self::AttachEvidence => f.write_str("attach_evidence"),
            Self::DenyWithReason { .. } => f.write_str("deny_with_reason"),
        }
    }
}

#[cfg(test)]
#[path = "binding_tests.rs"]
mod binding_tests;
