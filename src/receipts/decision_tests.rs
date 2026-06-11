use super::decision::{
    DecisionReceiptPayload, DecisionVerdict, SupportMember, DECISION_PAYLOAD_KIND,
};

/// Byte-pin for the Decision Receipt payload canonical form. These bytes
/// were captured from the `vr-browser-runtime` Decision Resolver BEFORE
/// the types were harvested into this crate; the harvest must remain a
/// byte no-op (same payload → same JCS bytes → same `event_hash`).
#[test]
fn payload_canonical_bytes_are_pinned() -> Result<(), String> {
    let deny = DecisionReceiptPayload {
        payload_kind: DECISION_PAYLOAD_KIND.to_string(),
        verdict: DecisionVerdict::Deny {
            reason: "Uncited claim(s) [missing] stopped at citation gate (rule: citation_gate)"
                .to_string(),
            code: "UncitedClaim".to_string(),
        },
        support_set: vec![SupportMember::SelectorValue {
            key: "uncited_claim".to_string(),
            value: "missing".to_string(),
        }],
    };
    let json = serde_json::to_string(&deny).map_err(|e| format!("{e}"))?;
    let canon = vr_jcs::to_canon_string_from_str(&json).map_err(|e| format!("{e}"))?;
    assert_eq!(
        canon,
        r#"{"payload_kind":"decision.v0","support_set":[{"key":"uncited_claim","member_kind":"selector_value","value":"missing"}],"verdict":{"code":"UncitedClaim","kind":"deny","reason":"Uncited claim(s) [missing] stopped at citation gate (rule: citation_gate)"}}"#
    );

    let allow = DecisionReceiptPayload {
        payload_kind: DECISION_PAYLOAD_KIND.to_string(),
        verdict: DecisionVerdict::Allow,
        support_set: vec![
            SupportMember::CitedLink {
                id: "site".to_string(),
                url: "https://example.org/x".to_string(),
            },
            SupportMember::DependedOnReceipt {
                event_hash: "abc123".to_string(),
            },
            SupportMember::EvidenceDigest {
                id: "dpa".to_string(),
                digest: "00ff".to_string(),
            },
        ],
    };
    let json = serde_json::to_string(&allow).map_err(|e| format!("{e}"))?;
    let canon = vr_jcs::to_canon_string_from_str(&json).map_err(|e| format!("{e}"))?;
    assert_eq!(
        canon,
        r#"{"payload_kind":"decision.v0","support_set":[{"id":"site","member_kind":"cited_link","url":"https://example.org/x"},{"event_hash":"abc123","member_kind":"depended_on_receipt"},{"digest":"00ff","id":"dpa","member_kind":"evidence_digest"}],"verdict":{"kind":"allow"}}"#
    );
    Ok(())
}

/// Round-trip through serde preserves equality for every variant shape.
#[test]
fn payload_round_trips() -> Result<(), String> {
    let payload = DecisionReceiptPayload {
        payload_kind: DECISION_PAYLOAD_KIND.to_string(),
        verdict: DecisionVerdict::Conditional {
            requirements: vec!["approval_token".to_string()],
            reason: "needs approval".to_string(),
        },
        support_set: vec![SupportMember::SelectorValue {
            key: "content_length".to_string(),
            value: "500".to_string(),
        }],
    };
    let json = serde_json::to_string(&payload).map_err(|e| format!("{e}"))?;
    let back: DecisionReceiptPayload = serde_json::from_str(&json).map_err(|e| format!("{e}"))?;
    assert_eq!(payload, back);

    let no_match: DecisionVerdict =
        serde_json::from_str(r#"{"kind":"no_match"}"#).map_err(|e| format!("{e}"))?;
    assert_eq!(no_match, DecisionVerdict::NoMatch);
    Ok(())
}

/// Variant order is load-bearing for support-set `BTree` ordering.
#[test]
fn support_member_ordering_is_stable() {
    let link = SupportMember::CitedLink {
        id: "a".to_string(),
        url: "u".to_string(),
    };
    let receipt = SupportMember::DependedOnReceipt {
        event_hash: "h".to_string(),
    };
    let evidence = SupportMember::EvidenceDigest {
        id: "a".to_string(),
        digest: "d".to_string(),
    };
    let selector = SupportMember::SelectorValue {
        key: "k".to_string(),
        value: "v".to_string(),
    };
    assert!(link < receipt);
    assert!(receipt < evidence);
    assert!(evidence < selector);
}
