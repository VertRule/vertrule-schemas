use crate::governance::{EntityNamespace, GovernedSubject};
use crate::DefinitionError;

// ── EntityNamespace construction ───────────────────────────────────

#[test]
fn namespace_accepts_lowercase_alpha() {
    let ns = EntityNamespace::new("issue".to_string()).expect("valid");
    assert_eq!(ns.as_str(), "issue");
}

#[test]
fn namespace_accepts_lowercase_with_digits_and_underscores() {
    let ns = EntityNamespace::new("agent_run_2".to_string()).expect("valid");
    assert_eq!(ns.as_str(), "agent_run_2");
}

#[test]
fn namespace_accepts_single_char() {
    let ns = EntityNamespace::new("a".to_string()).expect("valid");
    assert_eq!(ns.as_str(), "a");
}

#[test]
fn namespace_accepts_max_length() {
    let val = format!("a{}", "b".repeat(63));
    assert_eq!(val.len(), 64);
    let ns = EntityNamespace::new(val);
    assert!(ns.is_ok());
}

#[test]
fn namespace_rejects_empty() {
    let ns = EntityNamespace::new(String::new());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_exceeds_max_length() {
    let val = format!("a{}", "b".repeat(64));
    assert_eq!(val.len(), 65);
    let ns = EntityNamespace::new(val);
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_uppercase_start() {
    let ns = EntityNamespace::new("Issue".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_digit_start() {
    let ns = EntityNamespace::new("1issue".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_underscore_start() {
    let ns = EntityNamespace::new("_issue".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_uppercase_interior() {
    let ns = EntityNamespace::new("agentRun".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_dash() {
    let ns = EntityNamespace::new("agent-run".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_dot() {
    let ns = EntityNamespace::new("agent.run".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

#[test]
fn namespace_rejects_space() {
    let ns = EntityNamespace::new("agent run".to_string());
    assert!(matches!(ns, Err(DefinitionError::InvalidNamespace(_))));
}

// ── EntityNamespace serde ──────────────────────────────────────────

#[test]
fn namespace_serde_roundtrip() {
    let ns = EntityNamespace::new("tool_call".to_string()).expect("valid");
    let json = serde_json::to_string(&ns).expect("serialize");
    assert_eq!(json, r#""tool_call""#);
    let back: EntityNamespace = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(ns, back);
}

#[test]
fn namespace_deserialize_rejects_invalid() {
    let result: Result<EntityNamespace, _> = serde_json::from_str(r#""Agent""#);
    assert!(result.is_err());
}

#[test]
fn namespace_display() {
    let ns = EntityNamespace::new("checkpoint".to_string()).expect("valid");
    assert_eq!(ns.to_string(), "checkpoint");
}

// ── GovernedSubject serde ──────────────────────────────────────────

#[test]
fn subject_serde_roundtrip() {
    let subject = GovernedSubject {
        subject_key: "jira:issue:PROJ-123".to_string(),
        entity_namespace: EntityNamespace::new("issue".to_string()).expect("valid"),
        entity_id: "PROJ-123".to_string(),
    };
    let json = serde_json::to_string(&subject).expect("serialize");
    let back: GovernedSubject = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(subject, back);
}

#[test]
fn subject_deserialize_rejects_invalid_namespace() {
    let json = r#"{
        "subject_key": "k",
        "entity_namespace": "Bad",
        "entity_id": "1"
    }"#;
    let result: Result<GovernedSubject, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ── Surface neutrality ─────────────────────────────────────────────

#[test]
fn subject_works_for_langchain_run() {
    let subject = GovernedSubject {
        subject_key: "langchain:agent_run:run-abc".to_string(),
        entity_namespace: EntityNamespace::new("agent_run".to_string()).expect("valid"),
        entity_id: "run-abc".to_string(),
    };
    let json = serde_json::to_string(&subject).expect("serialize");
    let back: GovernedSubject = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(subject, back);
}

#[test]
fn subject_works_for_slack_message() {
    let subject = GovernedSubject {
        subject_key: "slack:message:ts-123".to_string(),
        entity_namespace: EntityNamespace::new("message".to_string()).expect("valid"),
        entity_id: "ts-123".to_string(),
    };
    let json = serde_json::to_string(&subject).expect("serialize");
    let back: GovernedSubject = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(subject, back);
}
