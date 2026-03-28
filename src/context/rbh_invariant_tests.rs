//! Tests for `RBHInvariant`.

use crate::{DigestBytes, RBHInvariant};

#[test]
fn new_constructor() {
    let parent = DigestBytes::from_array([0x01; 32]);
    let policy = DigestBytes::from_array([0x02; 32]);
    let receipt = DigestBytes::from_array([0x03; 32]);

    let inv = RBHInvariant::new(parent, policy, receipt);
    assert_eq!(inv.parent_context_digest, parent);
    assert_eq!(inv.required_policy_digest, policy);
    assert_eq!(inv.required_receipt_digest, receipt);
}

#[test]
fn equality() {
    let a = RBHInvariant::new(
        DigestBytes::from_array([0x01; 32]),
        DigestBytes::from_array([0x02; 32]),
        DigestBytes::from_array([0x03; 32]),
    );
    let b = RBHInvariant::new(
        DigestBytes::from_array([0x01; 32]),
        DigestBytes::from_array([0x02; 32]),
        DigestBytes::from_array([0x03; 32]),
    );
    assert_eq!(a, b);
}

#[test]
fn inequality_on_parent() {
    let a = RBHInvariant::new(
        DigestBytes::from_array([0x01; 32]),
        DigestBytes::from_array([0x02; 32]),
        DigestBytes::from_array([0x03; 32]),
    );
    let b = RBHInvariant::new(
        DigestBytes::from_array([0xff; 32]),
        DigestBytes::from_array([0x02; 32]),
        DigestBytes::from_array([0x03; 32]),
    );
    assert_ne!(a, b);
}

#[test]
fn clone() {
    let a = RBHInvariant::new(
        DigestBytes::from_array([0x01; 32]),
        DigestBytes::from_array([0x02; 32]),
        DigestBytes::from_array([0x03; 32]),
    );
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn debug_format() {
    let inv = RBHInvariant::new(
        DigestBytes::from_array([0x00; 32]),
        DigestBytes::from_array([0x00; 32]),
        DigestBytes::from_array([0x00; 32]),
    );
    let dbg = format!("{inv:?}");
    assert!(dbg.contains("RBHInvariant"));
}

#[test]
fn serde_round_trip() -> Result<(), anyhow::Error> {
    let inv = RBHInvariant::new(
        DigestBytes::from_array([0xaa; 32]),
        DigestBytes::from_array([0xbb; 32]),
        DigestBytes::from_array([0xcc; 32]),
    );
    let json = serde_json::to_string(&inv)?;
    let parsed: RBHInvariant = serde_json::from_str(&json)?;
    assert_eq!(inv, parsed);
    Ok(())
}

#[test]
fn serde_field_names() -> Result<(), anyhow::Error> {
    let inv = RBHInvariant::new(
        DigestBytes::from_array([0x01; 32]),
        DigestBytes::from_array([0x02; 32]),
        DigestBytes::from_array([0x03; 32]),
    );
    let json = serde_json::to_string(&inv)?;
    assert!(json.contains("parent_context_digest"));
    assert!(json.contains("required_policy_digest"));
    assert!(json.contains("required_receipt_digest"));
    Ok(())
}

#[test]
fn option_none_serializes_absent() -> Result<(), anyhow::Error> {
    let opt: Option<RBHInvariant> = None;
    let json = serde_json::to_string(&opt)?;
    assert_eq!(json, "null");
    Ok(())
}
