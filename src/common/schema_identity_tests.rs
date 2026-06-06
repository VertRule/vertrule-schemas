//! Golden serde tests for schema identity wire carriers.
//!
//! These lock the byte-exact wire representation that pack/manifest/receipt
//! DTOs depend on, and that must match the runtime identity types.

use crate::{
    SchemaKeyId, SchemaModelId, SchemaPolicyPackId, SchemaPublicKeyHex, SchemaReceiptId,
    SchemaRunId, SchemaSuiteId,
};

#[test]
fn text_carriers_serialize_as_transparent_string() -> Result<(), anyhow::Error> {
    assert_eq!(serde_json::to_string(&SchemaReceiptId::new("rcpt-1"))?, "\"rcpt-1\"");
    assert_eq!(serde_json::to_string(&SchemaModelId::new("Qwen2-0.5B"))?, "\"Qwen2-0.5B\"");
    assert_eq!(
        serde_json::to_string(&SchemaPolicyPackId::new("determinism@0.1"))?,
        "\"determinism@0.1\""
    );
    assert_eq!(serde_json::to_string(&SchemaSuiteId::new("suite-7"))?, "\"suite-7\"");
    assert_eq!(serde_json::to_string(&SchemaPublicKeyHex::new("ab12"))?, "\"ab12\"");
    assert_eq!(serde_json::to_string(&SchemaKeyId::new("key-3"))?, "\"key-3\"");
    Ok(())
}

#[test]
fn text_carriers_round_trip() -> Result<(), anyhow::Error> {
    let id = SchemaReceiptId::new("rcpt-xyz");
    let back: SchemaReceiptId = serde_json::from_str(&serde_json::to_string(&id)?)?;
    assert_eq!(id, back);
    assert_eq!(back.as_str(), "rcpt-xyz");
    Ok(())
}

#[test]
fn run_id_wire_form_matches_runtime() -> Result<(), anyhow::Error> {
    // partition is a bare number; offset is a decimal string (full u64 range).
    let r = SchemaRunId::new(3, u64::MAX);
    let json = serde_json::to_string(&r)?;
    assert_eq!(json, r#"{"partition":3,"offset":"18446744073709551615"}"#);

    let back: SchemaRunId = serde_json::from_str(&json)?;
    assert_eq!(back, r);
    assert_eq!(back.partition(), 3);
    assert_eq!(back.offset(), u64::MAX);
    Ok(())
}

#[test]
fn run_id_accepts_legacy_u64_and_numeric_offset() -> Result<(), anyhow::Error> {
    // Legacy bare u64 → partition 0.
    let legacy: SchemaRunId = serde_json::from_str("42")?;
    assert_eq!(legacy, SchemaRunId::new(0, 42));

    // Numeric offset form is tolerated on read.
    let numeric: SchemaRunId = serde_json::from_str(r#"{"partition":1,"offset":7}"#)?;
    assert_eq!(numeric, SchemaRunId::new(1, 7));

    // Unknown fields are ignored.
    let extra: SchemaRunId =
        serde_json::from_str(r#"{"partition":2,"offset":"9","extra":true}"#)?;
    assert_eq!(extra, SchemaRunId::new(2, 9));
    Ok(())
}

#[test]
fn run_id_canonical_jcs_bytes_are_stable() -> Result<(), anyhow::Error> {
    let r = SchemaRunId::new(1, 1000);
    let json = serde_json::to_vec(&r)?;
    let canon = vr_jcs::to_canon_bytes_from_slice(&json)?;
    // JCS sorts keys: "offset" precedes "partition".
    assert_eq!(canon, br#"{"offset":"1000","partition":1}"#);
    Ok(())
}
