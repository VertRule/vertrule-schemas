use crate::governance::{ActionNamespace, GovernedAction};
use crate::DefinitionError;

type R = Result<(), Box<dyn std::error::Error>>;

// ── ActionNamespace construction ───────────────────────────────────

#[test]
fn namespace_accepts_lowercase() -> R {
    let ns = ActionNamespace::new("workflow".to_string())?;
    assert_eq!(ns.as_str(), "workflow");
    Ok(())
}

#[test]
fn namespace_accepts_digits_and_underscores() -> R {
    let ns = ActionNamespace::new("tool_v2".to_string())?;
    assert_eq!(ns.as_str(), "tool_v2");
    Ok(())
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
fn namespace_serde_roundtrip() -> R {
    let ns = ActionNamespace::new("agent".to_string())?;
    let json = serde_json::to_string(&ns)?;
    assert_eq!(json, r#""agent""#);
    let back: ActionNamespace = serde_json::from_str(&json)?;
    assert_eq!(ns, back);
    Ok(())
}

#[test]
fn namespace_deserialize_rejects_invalid() {
    let result: Result<ActionNamespace, _> = serde_json::from_str(r#""Bad""#);
    assert!(result.is_err());
}

// ── GovernedAction serde ───────────────────────────────────────────

#[test]
fn action_serde_roundtrip_with_hint() -> R {
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("workflow".to_string())?,
        action_type: "transition".to_string(),
        action_idempotency_hint: Some("open:closed".to_string()),
    };
    let json = serde_json::to_string(&action)?;
    let back: GovernedAction = serde_json::from_str(&json)?;
    assert_eq!(action, back);
    Ok(())
}

#[test]
fn action_serde_roundtrip_without_hint() -> R {
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("agent".to_string())?,
        action_type: "invoke_tool".to_string(),
        action_idempotency_hint: None,
    };
    let json = serde_json::to_string(&action)?;
    // hint should be absent, not null
    assert!(!json.contains("action_idempotency_hint"));
    let back: GovernedAction = serde_json::from_str(&json)?;
    assert_eq!(action, back);
    Ok(())
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
fn action_works_for_langchain_tool_invocation() -> R {
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("agent".to_string())?,
        action_type: "invoke_tool".to_string(),
        action_idempotency_hint: Some("run-abc:step-7:attempt-1".to_string()),
    };
    let json = serde_json::to_string(&action)?;
    let back: GovernedAction = serde_json::from_str(&json)?;
    assert_eq!(action, back);
    Ok(())
}

#[test]
fn action_works_for_slack_approval() -> R {
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("approval".to_string())?,
        action_type: "approve".to_string(),
        action_idempotency_hint: None,
    };
    let json = serde_json::to_string(&action)?;
    let back: GovernedAction = serde_json::from_str(&json)?;
    assert_eq!(action, back);
    Ok(())
}
