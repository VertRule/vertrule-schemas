use std::collections::BTreeMap;

use crate::governance::{
    ActionNamespace, AdapterOrigin, AdapterReference, EntityNamespace, GovernancePrincipalId,
    GovernanceScope, GovernedAction, GovernedDecisionPayload, GovernedSubject, SurfaceInstanceId,
    Verdict,
};
use crate::{DigestBytes, IJsonUInt, ProjectsToReceiptEnvelope};

fn sample_decision(verdict: Verdict) -> GovernedDecisionPayload {
    GovernedDecisionPayload {
        scope: GovernanceScope {
            governance_principal_id: GovernancePrincipalId::new("org-1".to_string())
                .expect("valid"),
            surface_instance_id: SurfaceInstanceId::new("jira:inst-1".to_string()).expect("valid"),
            adapter_origin: AdapterOrigin::Jira,
            workspace_scope: "jira:org-1:PROJ".to_string(),
        },
        subject: GovernedSubject {
            subject_key: "jira:issue:PROJ-42".to_string(),
            entity_namespace: EntityNamespace::new("issue".to_string()).expect("valid"),
            entity_id: "PROJ-42".to_string(),
        },
        action: GovernedAction {
            action_namespace: ActionNamespace::new("workflow".to_string()).expect("valid"),
            action_type: "transition".to_string(),
            action_idempotency_hint: None,
        },
        adapter_ref: AdapterReference {
            adapter_origin: AdapterOrigin::Jira,
            external_keys: BTreeMap::from([("issue_key".to_string(), "PROJ-42".to_string())]),
        },
        verdict,
        reasons: vec![],
        policy_binding_id: "bind-1".to_string(),
        idempotency_key: DigestBytes::from_array([0u8; 32]),
        canonical_input_digest: DigestBytes::from_array([1u8; 32]),
        logical_time: IJsonUInt::new(1).expect("valid"),
        parent_id: None,
    }
}

// ── Verdict serde ──────────────────────────────────────────────────

#[test]
fn verdict_allow_serializes() {
    let json = serde_json::to_string(&Verdict::Allow).expect("serialize");
    assert_eq!(json, r#""allow""#);
}

#[test]
fn verdict_deny_serializes() {
    let json = serde_json::to_string(&Verdict::Deny).expect("serialize");
    assert_eq!(json, r#""deny""#);
}

#[test]
fn verdict_conditional_serializes() {
    let v = Verdict::Conditional {
        requirements: vec!["approval_token".to_string()],
    };
    let json = serde_json::to_string(&v).expect("serialize");
    assert!(json.contains("conditional"));
    assert!(json.contains("approval_token"));
}

#[test]
fn verdict_roundtrip_all_variants() {
    let variants = [
        Verdict::Allow,
        Verdict::Deny,
        Verdict::Conditional {
            requirements: vec!["r1".to_string(), "r2".to_string()],
        },
    ];
    for v in &variants {
        let json = serde_json::to_string(v).expect("serialize");
        let back: Verdict = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(v, &back);
    }
}

// ── Verdict display ────────────────────────────────────────────────

#[test]
fn verdict_display() {
    assert_eq!(Verdict::Allow.to_string(), "allow");
    assert_eq!(Verdict::Deny.to_string(), "deny");
    assert_eq!(
        Verdict::Conditional {
            requirements: vec![]
        }
        .to_string(),
        "conditional"
    );
}

// ── GovernedDecisionPayload serde ──────────────────────────────────

#[test]
fn decision_allow_serde_roundtrip() {
    let decision = sample_decision(Verdict::Allow);
    let json = serde_json::to_string(&decision).expect("serialize");
    let back: GovernedDecisionPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decision, back);
}

#[test]
fn decision_deny_serde_roundtrip() {
    let mut decision = sample_decision(Verdict::Deny);
    decision.reasons = vec!["missing approval".to_string()];
    let json = serde_json::to_string(&decision).expect("serialize");
    let back: GovernedDecisionPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decision, back);
}

#[test]
fn decision_conditional_serde_roundtrip() {
    let decision = sample_decision(Verdict::Conditional {
        requirements: vec!["approval_token".to_string()],
    });
    let json = serde_json::to_string(&decision).expect("serialize");
    let back: GovernedDecisionPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decision, back);
}

// ── ProjectsToReceiptEnvelope ──────────────────────────────────────

#[test]
fn project_produces_valid_envelope() {
    let decision = sample_decision(Verdict::Allow);
    let envelope = decision.project().expect("project");
    assert_eq!(envelope.receipt_type.to_string(), "governance");
    assert_eq!(envelope.logical_time.get(), 1);
    assert!(envelope.parent_id.is_none());
    assert_ne!(envelope.event_hash, DigestBytes::from_array([0u8; 32]));
}

#[test]
fn project_is_deterministic() {
    let decision = sample_decision(Verdict::Allow);
    let a = decision.project().expect("project");
    let b = decision.project().expect("project");
    assert_eq!(a.event_hash, b.event_hash);
}

#[test]
fn project_different_verdicts_different_hashes() {
    let d1 = sample_decision(Verdict::Allow);
    let d2 = sample_decision(Verdict::Deny);
    let e1 = d1.project().expect("project");
    let e2 = d2.project().expect("project");
    assert_ne!(e1.event_hash, e2.event_hash);
}

#[test]
fn project_with_parent_id() {
    let mut decision = sample_decision(Verdict::Allow);
    decision.parent_id = Some(DigestBytes::from_array([99u8; 32]));
    let envelope = decision.project().expect("project");
    assert_eq!(
        envelope.parent_id,
        Some(DigestBytes::from_array([99u8; 32]))
    );
}

#[test]
fn project_deny_verdict_succeeds() {
    let mut decision = sample_decision(Verdict::Deny);
    decision.reasons = vec!["release freeze".to_string()];
    decision.logical_time = IJsonUInt::new(42).expect("valid");
    let envelope = decision.project().expect("project");
    assert_eq!(envelope.logical_time.get(), 42);
}

// ── Surface neutrality ─────────────────────────────────────────────

#[test]
fn decision_works_for_langchain() {
    let decision = GovernedDecisionPayload {
        scope: GovernanceScope {
            governance_principal_id: GovernancePrincipalId::new("org-lc".to_string())
                .expect("valid"),
            surface_instance_id: SurfaceInstanceId::new("langchain:ws-9".to_string())
                .expect("valid"),
            adapter_origin: AdapterOrigin::LangChain,
            workspace_scope: "langchain:ws-9:graph-a".to_string(),
        },
        subject: GovernedSubject {
            subject_key: "langchain:tool_call:run-abc:step-7".to_string(),
            entity_namespace: EntityNamespace::new("tool_call".to_string()).expect("valid"),
            entity_id: "step-7".to_string(),
        },
        action: GovernedAction {
            action_namespace: ActionNamespace::new("agent".to_string()).expect("valid"),
            action_type: "invoke_tool".to_string(),
            action_idempotency_hint: Some("run-abc:step-7:attempt-1".to_string()),
        },
        adapter_ref: AdapterReference {
            adapter_origin: AdapterOrigin::LangChain,
            external_keys: BTreeMap::from([
                ("run_id".to_string(), "run-abc".to_string()),
                ("step_index".to_string(), "7".to_string()),
                ("tool_name".to_string(), "web_search".to_string()),
            ]),
        },
        verdict: Verdict::Allow,
        reasons: vec![],
        policy_binding_id: "bind-lc-tool-gate".to_string(),
        idempotency_key: DigestBytes::from_array([2u8; 32]),
        canonical_input_digest: DigestBytes::from_array([3u8; 32]),
        logical_time: IJsonUInt::new(1).expect("valid"),
        parent_id: None,
    };
    // Serde round-trip
    let json = serde_json::to_string(&decision).expect("serialize");
    let back: GovernedDecisionPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decision, back);

    // Projection works for non-Jira surface
    let envelope = decision.project().expect("project");
    assert_eq!(envelope.receipt_type.to_string(), "governance");
    assert_ne!(envelope.event_hash, DigestBytes::from_array([0u8; 32]));
}
