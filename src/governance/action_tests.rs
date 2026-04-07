use crate::governance::{ActionNamespace, GovernedAction};
use crate::DefinitionError;

// ── ActionNamespace construction ───────────────────────────────────

#[test]
fn namespace_accepts_lowercase() {
    let ns = ActionNamespace::new("workflow".to_string()).expect("valid");
    assert_eq!(ns.as_str(), "workflow");
}

#[test]
fn namespace_accepts_digits_and_underscores() {
    let ns = ActionNamespace::new("tool_v2".to_string()).expect("valid");
    assert_eq!(ns.as_str(), "tool_v2");
}

#[test]
fn namespace_rejects_empty() {
    let ns = ActionNamespace::new(String::new());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_uppercase_start() {
    let ns = ActionNamespace::new("Workflow".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_dash() {
    let ns = ActionNamespace::new("tool-use".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_exceeds_max_length() {
    let val = format!("a{}", "b".repeat(64));
    let ns = ActionNamespace::new(val);
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

// ── ActionNamespace serde ──────────────────────────────────────────

#[test]
fn namespace_serde_roundtrip() {
    let ns = ActionNamespace::new("agent".to_string()).expect("valid");
    let json = serde_json::to_string(&ns).expect("serialize");
    assert_eq!(json, r#""agent""#);
    let back: ActionNamespace = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(ns, back);
}

#[test]
fn namespace_deserialize_rejects_invalid() {
    let result: Result<ActionNamespace, _> = serde_json::from_str(r#""Bad""#);
    assert!(result.is_err());
}

// ── GovernedAction serde ───────────────────────────────────────────

#[test]
fn action_serde_roundtrip_with_hint() {
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("workflow".to_string()).expect("valid"),
        action_type: "transition".to_string(),
        action_idempotency_hint: Some("open:closed".to_string()),
    };
    let json = serde_json::to_string(&action).expect("serialize");
    let back: GovernedAction = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(action, back);
}

#[test]
fn action_serde_roundtrip_without_hint() {
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("agent".to_string()).expect("valid"),
        action_type: "invoke_tool".to_string(),
        action_idempotency_hint: None,
    };
    let json = serde_json::to_string(&action).expect("serialize");
    // hint should be absent, not null
    assert!(!json.contains("action_idempotency_hint"));
    let back: GovernedAction = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(action, back);
}

#[test]
fn action_deserialize_rejects_invalid_namespace() {
    let json = r#"{
        "action_namespace": "BAD",
        "action_type": "invoke"
    }"#;
    let result: Result<GovernedAction, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ── Surface neutrality ─────────────────────────────────────────────

#[test]
fn action_works_for_langchain_tool_invocation() {
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("agent".to_string()).expect("valid"),
        action_type: "invoke_tool".to_string(),
        action_idempotency_hint: Some("run-abc:step-7:attempt-1".to_string()),
    };
    let json = serde_json::to_string(&action).expect("serialize");
    let back: GovernedAction = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(action, back);
}

#[test]
fn action_works_for_slack_approval() {
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("approval".to_string()).expect("valid"),
        action_type: "approve".to_string(),
        action_idempotency_hint: None,
    };
    let json = serde_json::to_string(&action).expect("serialize");
    let back: GovernedAction = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(action, back);
}
