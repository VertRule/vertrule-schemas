use super::*;
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn to_canon_string_sorts_ascii_keys() -> Result<(), JcsError> {
    let value = json!({"z": 1, "a": 2, "m": 3});
    let string = to_canon_string_from_str(&serde_json::to_string(&value)?)?;
    assert_eq!(string, r#"{"a":2,"m":3,"z":1}"#);
    Ok(())
}

#[test]
fn to_canon_string_sorts_keys_by_utf16_code_units() -> Result<(), JcsError> {
    let value = json!({
        "\u{E000}": 2,
        "\u{10000}": 1
    });

    let string = to_canon_string_from_str(&serde_json::to_string(&value)?)?;
    let expected = format!(r#"{{"{}":1,"{}":2}}"#, '\u{10000}', '\u{E000}');
    assert_eq!(string, expected);
    Ok(())
}

#[test]
fn to_canon_string_matches_rfc_8785_primitive_example() -> Result<(), JcsError> {
    let input = r#"{
        "numbers": [333333333.33333329, 1E30, 4.50, 2e-3, 0.000000000000000000000000001],
        "string": "\u20ac$\u000F\u000aA'\u0042\u0022\u005c\\\"\/",
        "literals": [null, true, false]
    }"#;

    let string = to_canon_string_from_str(input)?;
    let expected = concat!(
        "{\"literals\":[null,true,false],",
        "\"numbers\":[333333333.3333333,1e+30,4.5,0.002,1e-27],",
        "\"string\":\"€$\\u000f\\nA'B\\\"\\\\\\\\\\\"/\"}"
    );
    assert_eq!(string, expected);
    Ok(())
}

#[test]
fn to_canon_string_matches_rfc_8785_property_sorting_example() -> Result<(), JcsError> {
    let input = r#"{
        "\u20ac": "Euro Sign",
        "\r": "Carriage Return",
        "\ufb33": "Hebrew Letter Dalet With Dagesh",
        "1": "One",
        "\ud83d\ude00": "Emoji: Grinning Face",
        "\u0080": "Control",
        "\u00f6": "Latin Small Letter O With Diaeresis"
    }"#;

    let string = to_canon_string_from_str(input)?;
    let expected = concat!(
        "{\"\\r\":\"Carriage Return\",",
        "\"1\":\"One\",",
        "\"\u{0080}\":\"Control\",",
        "\"\u{00F6}\":\"Latin Small Letter O With Diaeresis\",",
        "\"\u{20AC}\":\"Euro Sign\",",
        "\"\u{1F600}\":\"Emoji: Grinning Face\",",
        "\"\u{FB33}\":\"Hebrew Letter Dalet With Dagesh\"}"
    );
    assert_eq!(string, expected);
    Ok(())
}

#[test]
fn to_canon_string_uses_ecmascript_number_rendering_rules() -> Result<(), JcsError> {
    let value = json!([
        1e-6,
        0.000_001_2,
        1e-7,
        1e20,
        1e21,
        1_000_000.0,
        -0.0,
        0.0,
        1.0
    ]);
    let string = to_canon_string_from_str(&serde_json::to_string(&value)?)?;
    assert_eq!(
        string,
        "[0.000001,0.0000012,1e-7,100000000000000000000,1e+21,1000000,0,0,1]"
    );
    Ok(())
}

#[test]
fn to_canon_string_preserves_array_order_and_recurses_objects() -> Result<(), JcsError> {
    let value = json!({
        "z": [{"b": 2, "a": 1}],
        "a": [{"b": 4, "a": 3}]
    });

    let string = to_canon_string_from_str(&serde_json::to_string(&value)?)?;
    assert_eq!(string, r#"{"a":[{"a":3,"b":4}],"z":[{"a":1,"b":2}]}"#);
    Ok(())
}

#[test]
fn to_canon_bytes_struct() -> Result<(), JcsError> {
    #[derive(Serialize)]
    struct Receipt {
        id: u64,
        data: BTreeMap<String, i32>,
    }

    let mut data = BTreeMap::new();
    data.insert("zebra".to_string(), 3);
    data.insert("apple".to_string(), 1);
    data.insert("mango".to_string(), 2);

    let receipt = Receipt { id: 42, data };
    let json = serde_json::to_vec(&receipt)?;
    let bytes = to_canon_bytes_from_slice(&json)?;
    let string = String::from_utf8(bytes)
        .map_err(|e| JcsError::InvalidString(format!("canonical output was not UTF-8: {e}")))?;

    assert_eq!(
        string,
        r#"{"data":{"apple":1,"mango":2,"zebra":3},"id":42}"#
    );
    Ok(())
}

#[test]
fn canon_bytes_equals_canon_string_bytes() -> Result<(), JcsError> {
    let json = serde_json::to_string(&json!({"a": 1, "b": 2}))?;
    let bytes = to_canon_bytes_from_slice(json.as_bytes())?;
    let string = to_canon_string_from_str(&json)?;
    assert_eq!(bytes, string.as_bytes());
    Ok(())
}

#[test]
fn raw_json_rejects_duplicate_property_names() {
    let result = to_canon_bytes_from_slice(br#"{"a": 1, "a": 2}"#);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("duplicate property name"));
    }
}

#[test]
fn raw_json_rejects_nested_duplicate_property_names() {
    let result = to_canon_bytes_from_slice(br#"{"outer": {"a": 1, "a": 2}}"#);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("duplicate property name"));
    }
}

#[test]
fn raw_json_rejects_noncharacters() {
    let result = to_canon_string_from_str(r#"{"bad":"\uFDD0"}"#);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("forbidden noncharacter"));
    }
}

#[test]
fn to_canon_bytes_rejects_non_exact_large_integer() -> Result<(), serde_json::Error> {
    let json = serde_json::to_vec(&json!(9_007_199_254_740_993u64))?;
    let result = to_canon_bytes_from_slice(&json);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("not exactly representable"));
    }
    Ok(())
}

#[test]
fn to_canon_bytes_accepts_exact_large_integer() -> Result<(), JcsError> {
    let json = serde_json::to_string(&json!(9_007_199_254_740_992u64))?;
    let string = to_canon_string_from_str(&json)?;
    assert_eq!(string, "9007199254740992");
    Ok(())
}

#[test]
fn jcs_error_display_mentions_failure_type() {
    let err = JcsError::InvalidNumber("bad number".to_string());
    assert!(err.to_string().contains("number validation failed"));
}
