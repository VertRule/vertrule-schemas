//! Golden serde/JCS tests for `TrainingReceipt`.

use crate::receipts::TrainingReceipt;
use crate::{DigestBytes, SchemaReceiptId};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

fn hex(fill: u8) -> String {
    DigestBytes::from_array([fill; 32]).to_hex()
}

fn sample() -> TrainingReceipt {
    TrainingReceipt::new(
        SchemaReceiptId::new("rcpt-1"),
        digest(0x11),
        digest(0x22),
        digest(0x33),
        digest(0x44),
        digest(0x55),
        digest(0x66),
        70,
        3,
        128,
        true,
    )
}

#[test]
fn round_trip() -> Result<(), anyhow::Error> {
    let r = sample();
    let back: TrainingReceipt = serde_json::from_str(&serde_json::to_string(&r)?)?;
    assert_eq!(back, r);
    Ok(())
}

#[test]
fn u64_counters_are_decimal_strings() -> Result<(), anyhow::Error> {
    let value = serde_json::to_value(sample())?;
    assert_eq!(value["logical_time"], serde_json::json!("70"));
    assert_eq!(value["step_index"], serde_json::json!("3"));
    assert_eq!(value["batch_size"], serde_json::json!("128"));
    Ok(())
}

#[test]
fn u64_counters_accept_legacy_numeric_form() -> Result<(), anyhow::Error> {
    // Backward tolerance: numeric counters still deserialize.
    let json = format!(
        concat!(
            r#"{{"receipt_id":"r","pre_weights_digest":"{d}","post_weights_digest":"{d}","#,
            r#""batch_digest":"{d}","context_digest":"{d}","schema_digest":"{d}","policy_digest":"{d}","#,
            r#""logical_time":70,"step_index":3,"batch_size":128,"reversible":false}}"#
        ),
        d = hex(0x11)
    );
    let parsed: TrainingReceipt = serde_json::from_str(&json)?;
    assert_eq!(parsed.logical_time, 70);
    assert_eq!(parsed.batch_size, 128);
    Ok(())
}

#[test]
fn optional_digests_skipped_when_none() -> Result<(), anyhow::Error> {
    let value = serde_json::to_value(sample())?;
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("expected object"))?;
    assert!(!obj.contains_key("optimizer_state_digest"));
    assert!(!obj.contains_key("parent_id"));
    Ok(())
}

#[test]
fn canonical_jcs_golden() -> Result<(), anyhow::Error> {
    let r = sample();
    let json = serde_json::to_vec(&r)?;
    let canon = vr_jcs::to_canon_bytes_from_slice(&json)?;
    let golden = format!(
        concat!(
            r#"{{"batch_digest":"{b33}","batch_size":"128","context_digest":"{b44}","#,
            r#""logical_time":"70","policy_digest":"{b66}","post_weights_digest":"{b22}","#,
            r#""pre_weights_digest":"{b11}","receipt_id":"rcpt-1","reversible":true,"#,
            r#""schema_digest":"{b55}","step_index":"3"}}"#
        ),
        b11 = hex(0x11),
        b22 = hex(0x22),
        b33 = hex(0x33),
        b44 = hex(0x44),
        b55 = hex(0x55),
        b66 = hex(0x66),
    );
    assert_eq!(canon, golden.as_bytes());
    Ok(())
}

#[test]
fn deny_unknown_fields_rejected() {
    let json = format!(
        concat!(
            r#"{{"receipt_id":"r","pre_weights_digest":"{d}","post_weights_digest":"{d}","#,
            r#""batch_digest":"{d}","context_digest":"{d}","schema_digest":"{d}","policy_digest":"{d}","#,
            r#""logical_time":"70","step_index":"3","batch_size":"128","reversible":false,"#,
            r#""surprise":true}}"#
        ),
        d = hex(0x11)
    );
    assert!(serde_json::from_str::<TrainingReceipt>(&json).is_err());
}
