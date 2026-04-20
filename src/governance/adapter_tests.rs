use std::collections::BTreeMap;

use crate::governance::{AdapterOriginId, AdapterReference};

type R = Result<(), Box<dyn std::error::Error>>;

// ── AdapterOriginId serde (transparent: bare string) ────────────────

#[test]
fn origin_jira_serializes_as_bare_string() -> R {
    let json = serde_json::to_string(&AdapterOriginId::jira()?)?;
    assert_eq!(json, r#""jira""#);
    Ok(())
}

#[test]
fn origin_langchain_serializes_as_bare_string() -> R {
    let json = serde_json::to_string(&AdapterOriginId::lang_chain()?)?;
    assert_eq!(json, r#""lang_chain""#);
    Ok(())
}

#[test]
fn origin_service_now_serializes_as_bare_string() -> R {
    let json = serde_json::to_string(&AdapterOriginId::service_now()?)?;
    assert_eq!(json, r#""service_now""#);
    Ok(())
}

#[test]
fn origin_webhook_serializes_as_bare_string() -> R {
    let json = serde_json::to_string(&AdapterOriginId::webhook()?)?;
    assert_eq!(json, r#""webhook""#);
    Ok(())
}

#[test]
fn origin_custom_grammar_value_serializes_as_bare_string() -> R {
    let id = AdapterOriginId::new("my_tool".to_string())?;
    let json = serde_json::to_string(&id)?;
    assert_eq!(json, r#""my_tool""#);
    Ok(())
}

#[test]
fn origin_roundtrip_all_known_constructors() -> R {
    let variants = [
        AdapterOriginId::jira()?,
        AdapterOriginId::lang_chain()?,
        AdapterOriginId::service_now()?,
        AdapterOriginId::salesforce()?,
        AdapterOriginId::slack()?,
        AdapterOriginId::webhook()?,
        AdapterOriginId::new("gitlab_ci".to_string())?,
    ];
    for v in &variants {
        let json = serde_json::to_string(v)?;
        let back: AdapterOriginId = serde_json::from_str(&json)?;
        assert_eq!(v, &back);
    }
    Ok(())
}

// ── AdapterOriginId grammar enforcement ────────────────────────────

#[test]
fn origin_rejects_empty() {
    let result = AdapterOriginId::new(String::new());
    assert!(result.is_err());
}

#[test]
fn origin_rejects_uppercase() {
    let result = AdapterOriginId::new("Jira".to_string());
    assert!(result.is_err());
}

#[test]
fn origin_rejects_leading_digit() {
    let result = AdapterOriginId::new("1jira".to_string());
    assert!(result.is_err());
}

#[test]
fn origin_rejects_hyphen() {
    let result = AdapterOriginId::new("lang-chain".to_string());
    assert!(result.is_err());
}

#[test]
fn origin_rejects_dot() {
    let result = AdapterOriginId::new("lang.chain".to_string());
    assert!(result.is_err());
}

#[test]
fn origin_rejects_exceeds_max_length() {
    let val = "a".repeat(65);
    let result = AdapterOriginId::new(val);
    assert!(result.is_err());
}

#[test]
fn origin_accepts_max_length() -> R {
    let val = "a".repeat(64);
    let result = AdapterOriginId::new(val)?;
    assert_eq!(result.as_str().len(), 64);
    Ok(())
}

// ── AdapterOriginId deserialize validation ─────────────────────────

#[test]
fn origin_deserialize_rejects_invalid_grammar() {
    let result: Result<AdapterOriginId, _> = serde_json::from_str(r#""Has-Uppercase""#);
    assert!(result.is_err());
}

#[test]
fn origin_deserialize_rejects_empty_string() {
    let result: Result<AdapterOriginId, _> = serde_json::from_str(r#""""#);
    assert!(result.is_err());
}

// ── AdapterOriginId display ─────────────────────────────────────────

#[test]
fn origin_display_matches_inner() -> R {
    assert_eq!(AdapterOriginId::jira()?.to_string(), "jira");
    assert_eq!(AdapterOriginId::lang_chain()?.to_string(), "lang_chain");
    assert_eq!(AdapterOriginId::service_now()?.to_string(), "service_now");
    assert_eq!(AdapterOriginId::salesforce()?.to_string(), "salesforce");
    assert_eq!(AdapterOriginId::slack()?.to_string(), "slack");
    assert_eq!(AdapterOriginId::webhook()?.to_string(), "webhook");
    Ok(())
}

// ── AdapterReference serde ─────────────────────────────────────────

#[test]
fn reference_serde_roundtrip() -> R {
    let mut keys = BTreeMap::new();
    keys.insert("issue_key".to_string(), "PROJ-123".to_string());
    keys.insert("site_id".to_string(), "abc".to_string());

    let reference = AdapterReference {
        adapter_origin: AdapterOriginId::jira()?,
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
        adapter_origin: AdapterOriginId::slack()?,
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
        adapter_origin: AdapterOriginId::lang_chain()?,
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
        adapter_origin: AdapterOriginId::lang_chain()?,
        external_keys: keys,
    };
    let json = serde_json::to_string(&reference)?;
    let back: AdapterReference = serde_json::from_str(&json)?;
    assert_eq!(reference, back);
    Ok(())
}
