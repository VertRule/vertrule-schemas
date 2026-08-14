//! Tests for `vr.policy.input@0.1` parsing.
//!
//! Relocated verbatim in substance from
//! `vertrule-policy-wasm/src/input_tests.rs`; the `vr_test!` macro is a
//! runtime `test-utils` facility unavailable here, so each case became a
//! plain `#[test]` returning `Result`. Same three cases, same assertions.

use super::*;

type TestResult = Result<(), serde_json::Error>;

#[test]
fn input_round_trips_and_orders_claims() -> TestResult {
    let json = format!(
        "{{\"claims\":{{\"zeta\":{{\"cited\":true}},\"alpha\":{{\"value\":5}}}},\
         \"input_format\":\"{INPUT_FORMAT}\",\"operation_date\":\"2026-06-12\"}}"
    );
    let input: EvaluationInput = serde_json::from_str(&json)?;
    assert_eq!(input.input_format, INPUT_FORMAT);
    let keys: Vec<&String> = input.claims.keys().collect();
    assert_eq!(keys, vec!["alpha", "zeta"], "claims must be BTree-ordered");
    Ok(())
}

#[test]
fn unknown_input_fields_are_rejected() {
    let json = format!(
        "{{\"input_format\":\"{INPUT_FORMAT}\",\"operation_date\":\"2026-06-12\",\
         \"claims\":{{}},\"surprise\":1}}"
    );
    let parsed: Result<EvaluationInput, _> = serde_json::from_str(&json);
    assert!(parsed.is_err());
}

#[test]
fn float_claim_values_are_rejected() {
    let json = format!(
        "{{\"input_format\":\"{INPUT_FORMAT}\",\"operation_date\":\"2026-06-12\",\
         \"claims\":{{\"n\":{{\"value\":1.5}}}}}}"
    );
    let parsed: Result<EvaluationInput, _> = serde_json::from_str(&json);
    assert!(parsed.is_err(), "claim values are exact integers only");
}

// ── Relocation guard ────────────────────────────────────────────────
//
// Net-new here, and deliberately so: the move's whole risk is that the
// canonicalization path changes shape while still looking correct. This
// pins the byte law at the carrier's new home, independently of the
// consumer-side golden in `vr-policy-substrate`.

#[test]
fn canonical_bytes_preserve_the_relocated_byte_law() -> Result<(), Box<dyn std::error::Error>> {
    let input: EvaluationInput = serde_json::from_str(
        "{\"input_format\":\"vr.policy.input@0.1\",\
         \"operation_date\":\"2026-06-13\",\
         \"claims\":{\"bias_audit\":{\"valid_through\":\"2026-12-31\"},\
         \"request_count\":{\"value\":42}}}",
    )?;
    assert_eq!(
        std::str::from_utf8(&input.to_canonical_bytes()?)?,
        "{\"claims\":{\"bias_audit\":{\"valid_through\":\"2026-12-31\"},\
         \"request_count\":{\"value\":42}},\
         \"input_format\":\"vr.policy.input@0.1\",\
         \"operation_date\":\"2026-06-13\"}"
    );
    Ok(())
}
