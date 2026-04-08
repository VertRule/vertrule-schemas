use std::collections::BTreeMap;

use crate::governance::{AdapterOrigin, AdapterReference};

type R = Result<(), Box<dyn std::error::Error>>;

// ── AdapterOrigin serde ────────────────────────────────────────────

#[test]
fn origin_jira_serializes_as_snake_case() -> R {
    let json = serde_json::to_string(&AdapterOrigin::Jira)?;
    assert_eq!(json, r#""jira""#);
    Ok(())
}

#[test]
fn origin_langchain_serializes_as_snake_case() -> R {
    let json = serde_json::to_string(&AdapterOrigin::LangChain)?;
    assert_eq!(json, r#""lang_chain""#);
    Ok(())
}

#[test]
fn origin_service_now_serializes_as_snake_case() -> R {
    let json = serde_json::to_string(&AdapterOrigin::ServiceNow)?;
    assert_eq!(json, r#""service_now""#);
    Ok(())
}

#[test]
fn origin_custom_serializes_with_value() -> R {
    let json = serde_json::to_string(&AdapterOrigin::Custom("my_tool".to_string()))?;
    assert_eq!(json, r#"{"custom":"my_tool"}"#);
    Ok(())
}

#[test]
fn origin_roundtrip_all_known_variants() -> R {
    let variants = [
        AdapterOrigin::Jira,
        AdapterOrigin::LangChain,
        AdapterOrigin::ServiceNow,
        AdapterOrigin::Salesforce,
        AdapterOrigin::Slack,
        AdapterOrigin::Custom("x".to_string()),
    ];
    for v in &variants {
        let json = serde_json::to_string(v)?;
        let back: AdapterOrigin = serde_json::from_str(&json)?;
        assert_eq!(v, &back);
    }
    Ok(())
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
fn reference_serde_roundtrip() -> R {
    let mut keys = BTreeMap::new();
    keys.insert("issue_key".to_string(), "PROJ-123".to_string());
    keys.insert("site_id".to_string(), "abc".to_string());

    let reference = AdapterReference {
        adapter_origin: AdapterOrigin::Jira,
        external_keys: keys,
    };
    let json = serde_json::to_string(&reference)?;
    let back: AdapterReference = serde_json::from_str(&json)?;
    assert_eq!(reference, back);
    Ok(())
}

#[test]
fn reference_empty_keys_roundtrip() -> R {
    let reference = AdapterReference {
        adapter_origin: AdapterOrigin::Slack,
        external_keys: BTreeMap::new(),
    };
    let json = serde_json::to_string(&reference)?;
    let back: AdapterReference = serde_json::from_str(&json)?;
    assert_eq!(reference, back);
    Ok(())
}

#[test]
fn reference_keys_serialize_in_sorted_order() -> R {
    let mut keys = BTreeMap::new();
    keys.insert("z_key".to_string(), "last".to_string());
    keys.insert("a_key".to_string(), "first".to_string());
    keys.insert("m_key".to_string(), "middle".to_string());

    let reference = AdapterReference {
        adapter_origin: AdapterOrigin::LangChain,
        external_keys: keys,
    };
    let json = serde_json::to_string(&reference)?;
    // BTreeMap guarantees sorted key order in serialization
    let a_pos = json.find("a_key").ok_or("a_key not found")?;
    let m_pos = json.find("m_key").ok_or("m_key not found")?;
    let z_pos = json.find("z_key").ok_or("z_key not found")?;
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
    Ok(())
}

// ── Surface neutrality ─────────────────────────────────────────────

#[test]
fn reference_works_for_langchain() -> R {
    let mut keys = BTreeMap::new();
    keys.insert("run_id".to_string(), "run-abc".to_string());
    keys.insert("step_index".to_string(), "7".to_string());
    keys.insert("tool_name".to_string(), "web_search".to_string());

    let reference = AdapterReference {
        adapter_origin: AdapterOrigin::LangChain,
        external_keys: keys,
    };
    let json = serde_json::to_string(&reference)?;
    let back: AdapterReference = serde_json::from_str(&json)?;
    assert_eq!(reference, back);
    Ok(())
}
