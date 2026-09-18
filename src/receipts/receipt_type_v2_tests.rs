//! Tests for `ReceiptTypeV2`.

use crate::ReceiptTypeV2;

/// Minimal checker for the ADR-056 §3.1 label grammar:
/// `"vr" ("." segment)+`, `segment ::= [a-z0-9][a-z0-9_-]*`.
fn matches_label_grammar(label: &str) -> bool {
    let Some(rest) = label.strip_prefix("vr.") else {
        return false;
    };
    let segment_ok = |segment: &str| {
        let mut chars = segment.chars();
        chars
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    };
    rest.split('.').all(segment_ok)
}

#[test]
fn grammar_checker_rejects_malformed_labels() {
    for bad in [
        "vr",
        "vr.",
        "vr..a",
        "vr.A",
        "vr.a.",
        "x.governance.decision",
        "vr.-a",
        "vr.a b",
    ] {
        assert!(!matches_label_grammar(bad), "{bad:?} must be rejected");
    }
    assert!(matches_label_grammar("vr.a0.b_c-d"));
}

#[test]
fn every_admitted_label_matches_the_grammar() {
    for receipt_type in ReceiptTypeV2::ADMITTED {
        let label = receipt_type.label();
        assert!(
            matches_label_grammar(label),
            "{label:?} does not match `vr(.segment)+`"
        );
    }
}

#[test]
fn admitted_labels_are_distinct() {
    let labels: std::collections::BTreeSet<&str> =
        ReceiptTypeV2::ADMITTED.iter().map(|t| t.label()).collect();
    assert_eq!(labels.len(), ReceiptTypeV2::ADMITTED.len());
}

#[test]
fn serde_round_trip_all_variants() -> Result<(), anyhow::Error> {
    let variants = [
        (
            ReceiptTypeV2::GovernanceDecision,
            "\"vr.governance.decision\"",
        ),
        (
            ReceiptTypeV2::RuntimePortSubmitOutcome,
            "\"vr.runtime_port.submit_outcome\"",
        ),
    ];
    for (variant, expected_json) in variants {
        let json = serde_json::to_string(&variant)?;
        assert_eq!(
            json, expected_json,
            "serialization mismatch for {variant:?}"
        );
        let parsed: ReceiptTypeV2 = serde_json::from_str(&json)?;
        assert_eq!(parsed, variant, "round-trip mismatch for {variant:?}");
        assert_eq!(json, format!("\"{}\"", variant.label()));
    }
    Ok(())
}

#[test]
fn display_is_the_label() {
    assert_eq!(
        ReceiptTypeV2::GovernanceDecision.to_string(),
        "vr.governance.decision"
    );
    assert_eq!(
        ReceiptTypeV2::RuntimePortSubmitOutcome.to_string(),
        "vr.runtime_port.submit_outcome"
    );
}

#[test]
fn deserialize_rejects_unknown_label() {
    let result: Result<ReceiptTypeV2, _> = serde_json::from_str("\"vr.unknown.type\"");
    assert!(result.is_err());
}

#[test]
fn deserialize_rejects_v1_vocabulary() {
    for v1 in ["\"governance\"", "\"event\"", "\"llm\""] {
        let result: Result<ReceiptTypeV2, _> = serde_json::from_str(v1);
        assert!(result.is_err(), "V1 label {v1} must not deserialize as V2");
    }
}

#[test]
fn deserialize_rejects_case_variants() {
    let result: Result<ReceiptTypeV2, _> = serde_json::from_str("\"VR.GOVERNANCE.DECISION\"");
    assert!(result.is_err());
}

#[test]
fn ordering_follows_declaration() {
    assert!(ReceiptTypeV2::GovernanceDecision < ReceiptTypeV2::RuntimePortSubmitOutcome);
}
