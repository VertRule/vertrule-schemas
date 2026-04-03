use super::CanonicalPayload;

#[test]
fn accepts_integer_payload() {
    let value = serde_json::json!({"count": 42, "name": "test"});
    assert!(CanonicalPayload::new(value).is_ok());
}

#[test]
fn accepts_nested_integers() {
    let value = serde_json::json!({"outer": {"inner": 7}, "list": [1, 2, 3]});
    assert!(CanonicalPayload::new(value).is_ok());
}

#[test]
fn accepts_strings_bools_nulls() {
    let value = serde_json::json!({"s": "hello", "b": true, "n": null});
    assert!(CanonicalPayload::new(value).is_ok());
}

#[test]
fn rejects_top_level_float() {
    let value = serde_json::json!(3.7);
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some());
    assert!(err
        .as_ref()
        .is_some_and(|e| e.to_string().contains("float")));
}

#[test]
fn rejects_nested_float() {
    let value = serde_json::json!({"data": {"temperature": 98.6}});
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some());
    assert!(err
        .as_ref()
        .is_some_and(|e| e.to_string().contains("temperature")));
}

#[test]
fn rejects_float_in_array() {
    let value = serde_json::json!({"values": [1, 2.5, 3]});
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some());
    assert!(err.as_ref().is_some_and(|e| e.to_string().contains("[1]")));
}

#[test]
fn rejects_integer_outside_i_json_range() {
    let value = serde_json::json!({"count": 9_007_199_254_740_992u64});
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some());
    assert!(err
        .as_ref()
        .is_some_and(|e| e.to_string().contains("interoperable I-JSON range")));
}

#[test]
fn rejects_noncharacters_in_strings() {
    let value = serde_json::json!({"bad": "\u{FDD0}"});
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some());
    assert!(err
        .as_ref()
        .is_some_and(|e| e.to_string().contains("noncharacter")));
}

#[test]
fn serde_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let value = serde_json::json!({"schema": "test@0.1", "count": 42});
    let payload = CanonicalPayload::new(value)?;
    let json = serde_json::to_string(&payload)?;
    let parsed: CanonicalPayload = serde_json::from_str(&json)?;
    assert_eq!(parsed, payload);
    Ok(())
}

#[test]
fn deserialization_rejects_float() {
    let json = r#"{"x": 3.14}"#;
    let result: Result<CanonicalPayload, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn deserialization_rejects_duplicate_object_keys() {
    let json = r#"{"x": 1, "x": 2}"#;
    let result: Result<CanonicalPayload, _> = serde_json::from_str(json);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("duplicate property name"));
    }
}

#[test]
fn accepts_zero_as_integer() {
    let value = serde_json::json!({"count": 0});
    assert!(CanonicalPayload::new(value).is_ok());
}

#[test]
fn accepts_negative_integer() {
    let value = serde_json::json!({"offset": -12});
    assert!(CanonicalPayload::new(value).is_ok());
}

#[test]
fn accepts_empty_object() {
    let value = serde_json::json!({});
    assert!(CanonicalPayload::new(value).is_ok());
}

// ── Arbitrary-precision boundary ───────────────────────────────────

#[test]
fn rejects_arbitrary_precision_overflow() -> Result<(), Box<dyn std::error::Error>> {
    // 1e400 is valid JSON per RFC 8259 but is neither i64, u64, nor
    // finite f64. With `arbitrary_precision` enabled, serde_json stores
    // it as a raw string. This must be rejected.
    let value: serde_json::Value = serde_json::from_str(r#"{"x": 1e400}"#)?;
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some(), "1e400 must be rejected");
    assert!(err.as_ref().is_some_and(|e| e
        .to_string()
        .contains("not representable as a safe integer")));
    Ok(())
}

#[test]
fn rejects_arbitrary_precision_negative_overflow() -> Result<(), Box<dyn std::error::Error>> {
    let value: serde_json::Value = serde_json::from_str(r#"{"x": -1e400}"#)?;
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some(), "-1e400 must be rejected");
    Ok(())
}

#[test]
fn accepts_max_safe_integer_via_raw_json() -> Result<(), Box<dyn std::error::Error>> {
    // 2^53 - 1 = 9007199254740991 — the largest safe integer
    let value: serde_json::Value = serde_json::from_str(r#"{"n": 9007199254740991}"#)?;
    assert!(CanonicalPayload::new(value).is_ok());
    Ok(())
}

#[test]
fn rejects_one_above_max_safe_integer_via_raw_json() -> Result<(), Box<dyn std::error::Error>> {
    // 2^53 = 9007199254740992 — one past the safe boundary
    let value: serde_json::Value = serde_json::from_str(r#"{"n": 9007199254740992}"#)?;
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some(), "2^53 must be rejected");
    assert!(err
        .as_ref()
        .is_some_and(|e| e.to_string().contains("interoperable I-JSON range")));
    Ok(())
}

#[test]
fn rejects_float_via_raw_json() -> Result<(), Box<dyn std::error::Error>> {
    let value: serde_json::Value = serde_json::from_str(r#"{"t": 3.14}"#)?;
    let err = CanonicalPayload::new(value).err();
    assert!(err.is_some(), "float must be rejected");
    assert!(err
        .as_ref()
        .is_some_and(|e| e.to_string().contains("float")));
    Ok(())
}
