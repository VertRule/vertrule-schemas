use crate::governance::{AdapterOrigin, GovernancePrincipalId, GovernanceScope, SurfaceInstanceId};
use crate::DefinitionError;

// ── GovernancePrincipalId construction ──────────────────────────────

#[test]
fn principal_id_accepts_alphanumeric() {
    let id = GovernancePrincipalId::new("org-123".to_string()).expect("valid id");
    assert_eq!(id.as_str(), "org-123");
}

#[test]
fn principal_id_accepts_dots_colons_underscores_dashes() {
    let id = GovernancePrincipalId::new("a.b:c_d-e".to_string());
    assert!(id.is_ok());
}

#[test]
fn principal_id_accepts_single_char() {
    let id = GovernancePrincipalId::new("x".to_string());
    assert!(id.is_ok());
}

#[test]
fn principal_id_accepts_max_length() {
    let val = "a".repeat(128);
    let id = GovernancePrincipalId::new(val);
    assert!(id.is_ok());
}

#[test]
fn principal_id_rejects_empty() {
    let id = GovernancePrincipalId::new(String::new());
    assert!(matches!(id, Err(DefinitionError::InvalidGovernanceId(_))));
}

#[test]
fn principal_id_rejects_exceeds_max_length() {
    let val = "a".repeat(129);
    let id = GovernancePrincipalId::new(val);
    assert!(matches!(id, Err(DefinitionError::InvalidGovernanceId(_))));
}

#[test]
fn principal_id_rejects_spaces() {
    let id = GovernancePrincipalId::new("org 123".to_string());
    assert!(matches!(id, Err(DefinitionError::InvalidGovernanceId(_))));
}

#[test]
fn principal_id_rejects_slash() {
    let id = GovernancePrincipalId::new("org/123".to_string());
    assert!(matches!(id, Err(DefinitionError::InvalidGovernanceId(_))));
}

#[test]
fn principal_id_rejects_at_sign() {
    let id = GovernancePrincipalId::new("user@org".to_string());
    assert!(matches!(id, Err(DefinitionError::InvalidGovernanceId(_))));
}

#[test]
fn principal_id_rejects_non_ascii() {
    let id = GovernancePrincipalId::new("org-\u{00e9}".to_string());
    assert!(matches!(id, Err(DefinitionError::InvalidGovernanceId(_))));
}

// ── SurfaceInstanceId construction ─────────────────────────────────

#[test]
fn surface_id_accepts_colon_separated() {
    let id = SurfaceInstanceId::new("jira:install-abc".to_string()).expect("valid id");
    assert_eq!(id.as_str(), "jira:install-abc");
}

#[test]
fn surface_id_rejects_empty() {
    let id = SurfaceInstanceId::new(String::new());
    assert!(matches!(id, Err(DefinitionError::InvalidGovernanceId(_))));
}

#[test]
fn surface_id_rejects_exceeds_max_length() {
    let val = "b".repeat(129);
    let id = SurfaceInstanceId::new(val);
    assert!(matches!(id, Err(DefinitionError::InvalidGovernanceId(_))));
}

// ── Display ────────────────────────────────────────────────────────

#[test]
fn principal_id_display_matches_inner() {
    let id = GovernancePrincipalId::new("org-42".to_string()).expect("valid");
    assert_eq!(id.to_string(), "org-42");
}

#[test]
fn surface_id_display_matches_inner() {
    let id = SurfaceInstanceId::new("jira:site-7".to_string()).expect("valid");
    assert_eq!(id.to_string(), "jira:site-7");
}

// ── Serde round-trip: newtypes ─────────────────────────────────────

#[test]
fn principal_id_serde_roundtrip() {
    let id = GovernancePrincipalId::new("org-99".to_string()).expect("valid");
    let json = serde_json::to_string(&id).expect("serialize");
    assert_eq!(json, r#""org-99""#);
    let back: GovernancePrincipalId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(id, back);
}

#[test]
fn surface_id_serde_roundtrip() {
    let id = SurfaceInstanceId::new("langchain:ws-1".to_string()).expect("valid");
    let json = serde_json::to_string(&id).expect("serialize");
    assert_eq!(json, r#""langchain:ws-1""#);
    let back: SurfaceInstanceId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(id, back);
}

#[test]
fn principal_id_deserialize_rejects_invalid() {
    let result: Result<GovernancePrincipalId, _> = serde_json::from_str(r#""has space""#);
    assert!(result.is_err());
}

#[test]
fn principal_id_deserialize_rejects_empty() {
    let result: Result<GovernancePrincipalId, _> = serde_json::from_str(r#""""#);
    assert!(result.is_err());
}

#[test]
fn surface_id_deserialize_rejects_invalid() {
    let result: Result<SurfaceInstanceId, _> = serde_json::from_str(r#""has/slash""#);
    assert!(result.is_err());
}

// ── Serde round-trip: GovernanceScope ──────────────────────────────

#[test]
fn governance_scope_serde_roundtrip() {
    let scope = GovernanceScope {
        governance_principal_id: GovernancePrincipalId::new("org-1".to_string()).expect("valid"),
        surface_instance_id: SurfaceInstanceId::new("jira:install-2".to_string()).expect("valid"),
        adapter_origin: AdapterOrigin::Jira,
        workspace_scope: "jira:org-1:PROJECT".to_string(),
    };
    let json = serde_json::to_string(&scope).expect("serialize");
    let back: GovernanceScope = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(scope, back);
}

#[test]
fn governance_scope_deserialize_rejects_invalid_principal() {
    let json = r#"{
        "governance_principal_id": "",
        "surface_instance_id": "jira:x",
        "adapter_origin": "jira",
        "workspace_scope": "w"
    }"#;
    let result: Result<GovernanceScope, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ── Surface neutrality ─────────────────────────────────────────────

#[test]
fn scope_works_for_langchain() {
    let scope = GovernanceScope {
        governance_principal_id: GovernancePrincipalId::new("org-lc".to_string()).expect("valid"),
        surface_instance_id: SurfaceInstanceId::new("langchain:ws-9".to_string()).expect("valid"),
        adapter_origin: AdapterOrigin::LangChain,
        workspace_scope: "langchain:ws-9:graph-alpha".to_string(),
    };
    let json = serde_json::to_string(&scope).expect("serialize");
    let back: GovernanceScope = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(scope, back);
}

#[test]
fn scope_works_for_custom_adapter() {
    let scope = GovernanceScope {
        governance_principal_id: GovernancePrincipalId::new("tenant-x".to_string()).expect("valid"),
        surface_instance_id: SurfaceInstanceId::new("custom:inst-1".to_string()).expect("valid"),
        adapter_origin: AdapterOrigin::Custom("my_system".to_string()),
        workspace_scope: "custom:inst-1:env-prod".to_string(),
    };
    let json = serde_json::to_string(&scope).expect("serialize");
    let back: GovernanceScope = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(scope, back);
}

// ── No Jira-specific fields in core type ───────────────────────────

/// Compile-time assertion: `GovernanceScope` has exactly the four
/// surface-neutral fields declared in the spec. Any Jira-specific
/// field (issue_key, project_id, tenant_id, site_id) would be a
/// constitutional violation. This test documents the invariant.
#[test]
fn scope_has_no_adapter_local_fields() {
    // Constructing with only the four constitutional fields proves
    // no adapter-specific field is required.
    let _scope = GovernanceScope {
        governance_principal_id: GovernancePrincipalId::new("p".to_string()).expect("valid"),
        surface_instance_id: SurfaceInstanceId::new("s".to_string()).expect("valid"),
        adapter_origin: AdapterOrigin::Slack,
        workspace_scope: String::new(),
    };
}
