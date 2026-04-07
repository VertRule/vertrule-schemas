use std::collections::BTreeMap;

use crate::governance::{AdapterOrigin, AdapterReference};

// ── AdapterOrigin serde ────────────────────────────────────────────

#[test]
fn origin_jira_serializes_as_snake_case() {
    let json = serde_json::to_string(&AdapterOrigin::Jira).expect("serialize");
    assert_eq!(json, r#""jira""#);
}

#[test]
fn origin_langchain_serializes_as_snake_case() {
    let json = serde_json::to_string(&AdapterOrigin::LangChain).expect("serialize");
    assert_eq!(json, r#""lang_chain""#);
}

#[test]
fn origin_service_now_serializes_as_snake_case() {
    let json = serde_json::to_string(&AdapterOrigin::ServiceNow).expect("serialize");
    assert_eq!(json, r#""service_now""#);
}

#[test]
fn origin_custom_serializes_with_value() {
    let json = serde_json::to_string(&AdapterOrigin::Custom("my_tool".to_string()))
        .expect("serialize");
    assert_eq!(json, r#"{"custom":"my_tool"}"#);
}

#[test]
fn origin_roundtrip_all_known_variants() {
    let variants = [
        AdapterOrigin::Jira,
        AdapterOrigin::LangChain,
        AdapterOrigin::ServiceNow,
        AdapterOrigin::Salesforce,
        AdapterOrigin::Slack,
        AdapterOrigin::Custom("x".to_string()),
    ];
    for v in &variants {
        let json = serde_json::to_string(v).expect("serialize");
        let back: AdapterOrigin = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(v, &back);
    }
}

// ── AdapterOrigin display ──────────────────────────────────────────

#[test]
fn origin_display_matches_serde_for_unit_variants() {
    assert_eq!(AdapterOrigin::Jira.to_string(), "jira");
    assert_eq!(AdapterOrigin::LangChain.to_string(), "lang_chain");
    assert_eq!(AdapterOrigin::ServiceNow.to_string(), "service_now");
    assert_eq!(AdapterOrigin::Salesforce.to_string(), "salesforce");
    assert_eq!(AdapterOrigin::Slack.to_string(), "slack");
}

#[test]
fn origin_display_custom() {
    let origin = AdapterOrigin::Custom("gitlab_ci".to_string());
    assert_eq!(origin.to_string(), "custom(gitlab_ci)");
}

// ── AdapterReference serde ─────────────────────────────────────────

#[test]
fn reference_serde_roundtrip() {
    let mut keys = BTreeMap::new();
    keys.insert("issue_key".to_string(), "PROJ-123".to_string());
    keys.insert("site_id".to_string(), "abc".to_string());

    let reference = AdapterReference {
        adapter_origin: AdapterOrigin::Jira,
        external_keys: keys,
    };
    let json = serde_json::to_string(&reference).expect("serialize");
    let back: AdapterReference = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(reference, back);
}

#[test]
fn reference_empty_keys_roundtrip() {
    let reference = AdapterReference {
        adapter_origin: AdapterOrigin::Slack,
        external_keys: BTreeMap::new(),
    };
    let json = serde_json::to_string(&reference).expect("serialize");
    let back: AdapterReference = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(reference, back);
}

#[test]
fn reference_keys_serialize_in_sorted_order() {
    let mut keys = BTreeMap::new();
    keys.insert("z_key".to_string(), "last".to_string());
    keys.insert("a_key".to_string(), "first".to_string());
    keys.insert("m_key".to_string(), "middle".to_string());

    let reference = AdapterReference {
        adapter_origin: AdapterOrigin::LangChain,
        external_keys: keys,
    };
    let json = serde_json::to_string(&reference).expect("serialize");
    // BTreeMap guarantees sorted key order in serialization
    let a_pos = json.find("a_key").expect("a_key present");
    let m_pos = json.find("m_key").expect("m_key present");
    let z_pos = json.find("z_key").expect("z_key present");
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

// ── Surface neutrality ─────────────────────────────────────────────

#[test]
fn reference_works_for_langchain() {
    let mut keys = BTreeMap::new();
    keys.insert("run_id".to_string(), "run-abc".to_string());
    keys.insert("step_index".to_string(), "7".to_string());
    keys.insert("tool_name".to_string(), "web_search".to_string());

    let reference = AdapterReference {
        adapter_origin: AdapterOrigin::LangChain,
        external_keys: keys,
    };
    let json = serde_json::to_string(&reference).expect("serialize");
    let back: AdapterReference = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(reference, back);
}
