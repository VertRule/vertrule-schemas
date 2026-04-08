use crate::governance::{PolicyBindingRef, PolicyTemplate};
use crate::PolicyId;

fn sample_binding(template: PolicyTemplate) -> PolicyBindingRef {
    PolicyBindingRef {
        binding_id: "bind-1".to_string(),
        workspace_scope: "jira:site-1:PROJ".to_string(),
        entity_namespace: None,
        action_type: None,
        policy_id: PolicyId::new("vr.surface.gate/require-approval".to_string())
            .expect("valid policy id"),
        policy_template: template,
    }
}

// ── PolicyTemplate serde ───────────────────────────────────────────

#[test]
fn template_require_approval_roundtrip() {
    let t = PolicyTemplate::RequireApproval;
    let json = serde_json::to_string(&t).expect("serialize");
    assert_eq!(json, r#""require_approval""#);
    let back: PolicyTemplate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(t, back);
}

#[test]
fn template_require_fields_roundtrip() {
    let t = PolicyTemplate::RequireFields {
        fields: vec!["assignee".to_string(), "priority".to_string()],
    };
    let json = serde_json::to_string(&t).expect("serialize");
    let back: PolicyTemplate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(t, back);
}

#[test]
fn template_attach_evidence_roundtrip() {
    let t = PolicyTemplate::AttachEvidence;
    let json = serde_json::to_string(&t).expect("serialize");
    assert_eq!(json, r#""attach_evidence""#);
    let back: PolicyTemplate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(t, back);
}

#[test]
fn template_deny_with_reason_roundtrip() {
    let t = PolicyTemplate::DenyWithReason {
        reason: "frozen for release".to_string(),
    };
    let json = serde_json::to_string(&t).expect("serialize");
    let back: PolicyTemplate = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(t, back);
}

// ── PolicyTemplate display ─────────────────────────────────────────

#[test]
fn template_display() {
    assert_eq!(
        PolicyTemplate::RequireApproval.to_string(),
        "require_approval"
    );
    assert_eq!(
        PolicyTemplate::RequireFields { fields: vec![] }.to_string(),
        "require_fields"
    );
    assert_eq!(
        PolicyTemplate::AttachEvidence.to_string(),
        "attach_evidence"
    );
    assert_eq!(
        PolicyTemplate::DenyWithReason {
            reason: String::new()
        }
        .to_string(),
        "deny_with_reason"
    );
}

// ── PolicyBindingRef serde ─────────────────────────────────────────

#[test]
fn binding_full_roundtrip() {
    let binding = PolicyBindingRef {
        binding_id: "bind-2".to_string(),
        workspace_scope: "langchain:ws-9:graph-a".to_string(),
        entity_namespace: Some("tool_call".to_string()),
        action_type: Some("invoke_tool".to_string()),
        policy_id: PolicyId::new("vr.surface.gate/tool-audit".to_string())
            .expect("valid policy id"),
        policy_template: PolicyTemplate::AttachEvidence,
    };
    let json = serde_json::to_string(&binding).expect("serialize");
    let back: PolicyBindingRef = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(binding, back);
}

#[test]
fn binding_wildcard_omits_optional_fields() {
    let binding = sample_binding(PolicyTemplate::RequireApproval);
    let json = serde_json::to_string(&binding).expect("serialize");
    // None fields should not appear in JSON
    assert!(!json.contains("entity_namespace"));
    assert!(!json.contains("action_type"));
    let back: PolicyBindingRef = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(binding, back);
}

#[test]
fn binding_with_filters_includes_optional_fields() {
    let mut binding = sample_binding(PolicyTemplate::AttachEvidence);
    binding.entity_namespace = Some("issue".to_string());
    binding.action_type = Some("transition".to_string());
    let json = serde_json::to_string(&binding).expect("serialize");
    assert!(json.contains("entity_namespace"));
    assert!(json.contains("action_type"));
}

// ── Surface neutrality ─────────────────────────────────────────────

#[test]
fn binding_works_for_slack() {
    let binding = PolicyBindingRef {
        binding_id: "bind-slack-1".to_string(),
        workspace_scope: "slack:team-1:channel-general".to_string(),
        entity_namespace: Some("approval_request".to_string()),
        action_type: Some("approve".to_string()),
        policy_id: PolicyId::new("vr.surface.gate/slack-approval".to_string())
            .expect("valid policy id"),
        policy_template: PolicyTemplate::RequireApproval,
    };
    let json = serde_json::to_string(&binding).expect("serialize");
    let back: PolicyBindingRef = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(binding, back);
}
