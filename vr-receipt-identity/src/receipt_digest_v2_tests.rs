//! Known-answer, mutation and disjointness tests for the V2
//! `receipt_digest` law (ADR-056 §3.2).
//!
//! The golden fixture `test-vectors/receipt_v2_governance_decision_001.json`
//! pins the JCS preimage, the sealed canonical bytes, the tag and the
//! digest. It is regenerated only by running the golden test with
//! `VR_WRITE_GOLDEN=1` (never set in CI); every ordinary run asserts
//! against the committed file.

use std::collections::BTreeMap;

use serde_json::Value;
use vertrule_schemas::governance::ScopeDigest;
use vertrule_schemas::{
    ActionNamespace, AdapterOriginId, AdapterReference, CanonicalPayload, DecisionPayload,
    DigestBytes, EntityNamespace, GovernancePrincipalId, GovernanceScope, GovernedAction,
    GovernedSubject, IJsonUInt, PayloadSchemaV2, ReceiptEnvelopeV2, ReceiptTypeV2, SchemaVersion,
    SurfaceInstanceId, Verdict,
};
use vr_identity::digest::{DigestDomain, DigestOf};
use vr_jcs::DigestStrategy;

use super::{
    compute_receipt_digest_v2, seal_receipt_v2, ReceiptDigestV2Identity, ReceiptV2Draft,
    RECEIPT_V2_DOMAIN_TAG,
};

/// One named draft mutation for the mutation-sensitivity sweep.
type DraftMutation = (&'static str, Box<dyn Fn(&mut ReceiptV2Draft)>);

const GOLDEN_CASE_ID: &str = "receipt_v2_governance_decision_001";
const EXPECTED_TAG_HEX: &str = "7665727472756c652e726563656970742e763200";

/// The fixed sealed policy digest carried both in the payload
/// (`sealed_policy_digest`) and in the envelope `policy_digest` slot.
fn sealed_policy_digest() -> DigestBytes {
    DigestBytes::from_array([0x77; 32])
}

/// The fixed `DecisionPayload` (shape shared with `decision_projection_tests`).
fn golden_decision() -> Result<DecisionPayload, anyhow::Error> {
    Ok(DecisionPayload {
        scope: GovernanceScope {
            governance_principal_id: GovernancePrincipalId::new("org-1".to_string())?,
            surface_instance_id: SurfaceInstanceId::new("jira:inst-1".to_string())?,
            adapter_origin: AdapterOriginId::jira()?,
            workspace_scope: "jira:org-1:PROJ".to_string(),
        },
        subject: GovernedSubject {
            subject_key: "jira:issue:PROJ-42".to_string(),
            entity_namespace: EntityNamespace::new("issue".to_string())?,
            entity_id: "PROJ-42".to_string(),
        },
        action: GovernedAction {
            action_namespace: ActionNamespace::new("workflow".to_string())?,
            action_type: "transition".to_string(),
            action_idempotency_hint: None,
        },
        adapter_ref: AdapterReference {
            adapter_origin: AdapterOriginId::jira()?,
            external_keys: BTreeMap::from([("issue_key".to_string(), "PROJ-42".to_string())]),
        },
        verdict: Verdict::Allow,
        reasons: vec![],
        policy_binding_id: "bind-1".to_string(),
        idempotency_key: DigestBytes::from_array([0; 32]),
        canonical_input_digest: DigestBytes::from_array([1; 32]),
        logical_time: IJsonUInt::new(1)?,
        parent_id: None,
        operation_receipt_digest: None,
        sealed_policy_digest: Some(sealed_policy_digest()),
    })
}

fn golden_draft() -> Result<ReceiptV2Draft, anyhow::Error> {
    let decision = golden_decision()?;
    let context_digest = ScopeDigest::from_governance_scope(&decision.scope)?.as_digest_bytes()?;
    let payload = CanonicalPayload::new(serde_json::to_value(&decision)?)?;
    Ok(ReceiptV2Draft {
        receipt_type: ReceiptTypeV2::GovernanceDecision,
        schema_digest: PayloadSchemaV2::VR_SURFACE_DECISION_0_1.identity(),
        context_digest: Some(context_digest),
        policy_digest: Some(sealed_policy_digest()),
        logical_time: 1,
        parent_id: None,
        payload,
    })
}

/// The preimage object: the serialised envelope minus `receipt_digest`.
fn preimage_of(envelope: &ReceiptEnvelopeV2) -> Result<Value, anyhow::Error> {
    let mut value = serde_json::to_value(envelope)?;
    let Value::Object(ref mut map) = value else {
        return Err(anyhow::anyhow!("envelope did not serialize to an object"));
    };
    map.remove("receipt_digest");
    Ok(value)
}

/// JCS text of a JSON value (serialised first, then canonicalised from
/// the strict-input entry point).
fn jcs_string(value: &Value) -> Result<String, anyhow::Error> {
    Ok(vr_jcs::to_canon_string_from_str(&serde_json::to_string(
        value,
    )?)?)
}

fn golden_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-vectors")
        .join(format!("{GOLDEN_CASE_ID}.json"))
}

fn field(fixture: &Value, key: &str) -> Result<String, anyhow::Error> {
    fixture[key]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("fixture field {key:?} missing or not a string"))
}

// ── Golden ───────────────────────────────────────────────────────────

#[test]
fn golden_fixture_matches_committed_file() -> Result<(), anyhow::Error> {
    let sealed = seal_receipt_v2(golden_draft()?)?;
    let preimage_jcs = jcs_string(&preimage_of(&sealed)?)?;
    let canonical_bytes = jcs_string(&serde_json::to_value(&sealed)?)?;
    let computed = serde_json::json!({
        "case_id": GOLDEN_CASE_ID,
        "canonical_bytes": canonical_bytes,
        "preimage_jcs": preimage_jcs,
        "receipt_digest": sealed.receipt_digest.to_hex(),
        "domain_tag_hex": hex::encode(RECEIPT_V2_DOMAIN_TAG),
    });

    if std::env::var_os("VR_WRITE_GOLDEN").is_some() {
        let mut text = serde_json::to_string_pretty(&computed)?;
        text.push('\n');
        std::fs::write(golden_path(), text)?;
    }

    let committed: Value = serde_json::from_slice(&std::fs::read(golden_path())?)?;
    assert_eq!(field(&committed, "case_id")?, GOLDEN_CASE_ID);
    assert_eq!(field(&committed, "domain_tag_hex")?, EXPECTED_TAG_HEX);
    assert_eq!(
        field(&committed, "preimage_jcs")?,
        field(&computed, "preimage_jcs")?,
        "V2 preimage drifted from the committed golden"
    );
    assert_eq!(
        field(&committed, "canonical_bytes")?,
        field(&computed, "canonical_bytes")?,
        "sealed V2 canonical bytes drifted from the committed golden"
    );
    assert_eq!(
        field(&committed, "receipt_digest")?,
        sealed.receipt_digest.to_hex(),
        "V2 receipt_digest drifted from the committed golden"
    );
    Ok(())
}

#[test]
fn golden_preimage_is_the_documented_wire_object() -> Result<(), anyhow::Error> {
    let sealed = seal_receipt_v2(golden_draft()?)?;
    let preimage = preimage_of(&sealed)?;
    let map = preimage
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("preimage is not an object"))?;
    // Key *set* only: `serde_json::Map` ordering is a build-feature
    // artefact (`preserve_order` unification); JCS sorts on the wire.
    let keys: std::collections::BTreeSet<&str> = map.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        std::collections::BTreeSet::from([
            "context_digest",
            "envelope_version",
            "logical_time",
            "payload",
            "policy_digest",
            "receipt_type",
            "schema_digest",
        ])
    );
    assert_eq!(preimage["envelope_version"], serde_json::json!(2));
    assert_eq!(preimage["logical_time"], serde_json::json!("1"));
    assert_eq!(
        preimage["receipt_type"],
        serde_json::json!("vr.governance.decision")
    );
    assert!(!map.contains_key("parent_id"));
    assert!(!map.contains_key("receipt_digest"));
    Ok(())
}

// ── Tag and domain pins ──────────────────────────────────────────────

#[test]
fn domain_tag_bytes_are_pinned() {
    assert_eq!(RECEIPT_V2_DOMAIN_TAG.len(), 20);
    assert_eq!(hex::encode(RECEIPT_V2_DOMAIN_TAG), EXPECTED_TAG_HEX);
    assert_eq!(RECEIPT_V2_DOMAIN_TAG[19], 0x00);
    assert_eq!(
        <ReceiptDigestV2Identity as DigestDomain>::DOMAIN_SEPARATION_TAG,
        RECEIPT_V2_DOMAIN_TAG
    );
}

#[test]
fn domain_id_is_pinned() {
    assert_eq!(
        DigestOf::<ReceiptDigestV2Identity>::domain_id().as_str(),
        "receipt.envelope-v2"
    );
}

// ── Seal / compute agreement ─────────────────────────────────────────

#[test]
fn seal_and_recompute_agree_and_are_deterministic() -> Result<(), anyhow::Error> {
    let first = seal_receipt_v2(golden_draft()?)?;
    let second = seal_receipt_v2(golden_draft()?)?;
    assert_eq!(first, second);
    assert_eq!(first.envelope_version, SchemaVersion::V2);
    assert_eq!(compute_receipt_digest_v2(&first)?, first.receipt_digest);
    assert_eq!(compute_receipt_digest_v2(&second)?, first.receipt_digest);
    Ok(())
}

#[test]
fn stored_receipt_digest_never_enters_its_own_preimage() -> Result<(), anyhow::Error> {
    let mut sealed = seal_receipt_v2(golden_draft()?)?;
    let expected = sealed.receipt_digest;
    sealed.receipt_digest = DigestBytes::from_array([0xaa; 32]);
    assert_eq!(compute_receipt_digest_v2(&sealed)?, expected);
    Ok(())
}

#[test]
fn preimage_digest_equals_tagged_formation_over_preimage() -> Result<(), anyhow::Error> {
    let sealed = seal_receipt_v2(golden_draft()?)?;
    let direct = DigestOf::<ReceiptDigestV2Identity>::derive(&preimage_of(&sealed)?)?;
    assert_eq!(
        direct.as_32_byte_array()?,
        *sealed.receipt_digest.as_bytes()
    );
    Ok(())
}

// ── Mutation sensitivity ─────────────────────────────────────────────

#[test]
fn every_field_mutation_changes_the_digest() -> Result<(), anyhow::Error> {
    let base = seal_receipt_v2(golden_draft()?)?.receipt_digest;
    let other = DigestBytes::from_array([0x5a; 32]);

    let mutations: Vec<DraftMutation> = vec![
        (
            "receipt_type",
            Box::new(|d| d.receipt_type = ReceiptTypeV2::RuntimePortSubmitOutcome),
        ),
        ("schema_digest", Box::new(move |d| d.schema_digest = other)),
        (
            "context_digest value",
            Box::new(move |d| d.context_digest = Some(other)),
        ),
        (
            "context_digest absent",
            Box::new(|d| d.context_digest = None),
        ),
        (
            "policy_digest value",
            Box::new(move |d| d.policy_digest = Some(other)),
        ),
        ("policy_digest absent", Box::new(|d| d.policy_digest = None)),
        ("logical_time", Box::new(|d| d.logical_time = 2)),
        (
            "parent_id present",
            Box::new(move |d| d.parent_id = Some(other)),
        ),
    ];
    for (name, mutate) in mutations {
        let mut draft = golden_draft()?;
        mutate(&mut draft);
        let mutated = seal_receipt_v2(draft)?.receipt_digest;
        assert_ne!(mutated, base, "mutating {name} must change receipt_digest");
    }

    let mut draft = golden_draft()?;
    let mut decision = golden_decision()?;
    decision.verdict = Verdict::Deny;
    draft.payload = CanonicalPayload::new(serde_json::to_value(&decision)?)?;
    let mutated = seal_receipt_v2(draft)?.receipt_digest;
    assert_ne!(mutated, base, "mutating payload must change receipt_digest");
    Ok(())
}

#[test]
fn absent_slot_and_sentinel_slot_are_distinct_receipts() -> Result<(), anyhow::Error> {
    let mut absent = golden_draft()?;
    absent.parent_id = None;
    let mut zero = golden_draft()?;
    zero.parent_id = Some(DigestBytes::from_array([0; 32]));
    assert_ne!(
        seal_receipt_v2(absent)?.receipt_digest,
        seal_receipt_v2(zero)?.receipt_digest
    );
    Ok(())
}

// ── Field-order independence ─────────────────────────────────────────

#[test]
fn key_order_does_not_affect_the_digest() -> Result<(), anyhow::Error> {
    let sealed = seal_receipt_v2(golden_draft()?)?;
    let canonical = serde_json::to_value(&sealed)?;
    let Value::Object(map) = canonical else {
        return Err(anyhow::anyhow!("envelope did not serialize to an object"));
    };
    // Rebuild the object with keys in reverse order and parse it back.
    let mut reversed = serde_json::Map::new();
    for (key, value) in map.iter().rev() {
        reversed.insert(key.clone(), value.clone());
    }
    let shuffled_text = serde_json::to_string(&Value::Object(reversed))?;
    let reparsed: ReceiptEnvelopeV2 = serde_json::from_str(&shuffled_text)?;
    assert_eq!(compute_receipt_digest_v2(&reparsed)?, sealed.receipt_digest);

    // The formation itself is order-independent over the raw preimage.
    let mut preimage_reversed = serde_json::Map::new();
    for (key, value) in map.iter().rev().filter(|(k, _)| *k != "receipt_digest") {
        preimage_reversed.insert(key.clone(), value.clone());
    }
    let direct = DigestOf::<ReceiptDigestV2Identity>::derive(&Value::Object(preimage_reversed))?;
    assert_eq!(
        direct.as_32_byte_array()?,
        *sealed.receipt_digest.as_bytes()
    );
    Ok(())
}

// ── Shape closure ────────────────────────────────────────────────────

#[test]
fn unknown_field_is_rejected_at_deserialization() -> Result<(), anyhow::Error> {
    let sealed = seal_receipt_v2(golden_draft()?)?;
    let mut value = serde_json::to_value(&sealed)?;
    value["event_hash"] = serde_json::json!(sealed.receipt_digest.to_hex());
    let result: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(value);
    assert!(
        result.is_err(),
        "a V2 envelope with `event_hash` must be rejected"
    );
    Ok(())
}

// ── Wire-form ambiguity closure (ADR-056 R8) ────────────────────────

/// The committed golden's canonical bytes, parsed as a mutable JSON value.
fn golden_canonical_document() -> Result<Value, anyhow::Error> {
    let committed: Value = serde_json::from_slice(&std::fs::read(golden_path())?)?;
    Ok(serde_json::from_str(&field(
        &committed,
        "canonical_bytes",
    )?)?)
}

#[test]
fn null_slot_document_does_not_deserialize() -> Result<(), anyhow::Error> {
    // `"slot": null` must never become the omitted form: if it did, the
    // null-bearing bytes would recompute to the omitted form's
    // `receipt_digest` and two byte-distinct documents would share one
    // identity.
    let sealed = seal_receipt_v2(golden_draft()?)?;
    for slot in ["context_digest", "policy_digest", "parent_id"] {
        let mut document = golden_canonical_document()?;
        document[slot] = Value::Null;
        let from_value: Result<ReceiptEnvelopeV2, _> = serde_json::from_value(document.clone());
        assert!(from_value.is_err(), "null {slot} must not deserialize");
        let from_str: Result<ReceiptEnvelopeV2, _> = serde_json::from_str(&document.to_string());
        assert!(from_str.is_err(), "null {slot} must not deserialize");
    }
    // The omitted form of `parent_id` (the golden itself) still parses and
    // recomputes to the committed digest.
    let parsed: ReceiptEnvelopeV2 = serde_json::from_value(golden_canonical_document()?)?;
    assert_eq!(compute_receipt_digest_v2(&parsed)?, sealed.receipt_digest);
    Ok(())
}

#[test]
fn non_canonical_logical_time_document_does_not_deserialize() -> Result<(), anyhow::Error> {
    for text in ["01", "+1"] {
        let mut document = golden_canonical_document()?;
        document["logical_time"] = serde_json::json!(text);
        let result: Result<ReceiptEnvelopeV2, _> = serde_json::from_str(&document.to_string());
        assert!(
            result.is_err(),
            "non-canonical logical_time {text:?} must not deserialize"
        );
    }
    // The ADR-mandated bare-number tolerance recomputes to the same digest
    // because the preimage re-serialises the canonical string form.
    let sealed = seal_receipt_v2(golden_draft()?)?;
    let mut document = golden_canonical_document()?;
    document["logical_time"] = serde_json::json!(1);
    let parsed: ReceiptEnvelopeV2 = serde_json::from_str(&document.to_string())?;
    assert_eq!(compute_receipt_digest_v2(&parsed)?, sealed.receipt_digest);
    Ok(())
}

#[test]
fn envelope_version_one_in_v2_shape_is_digested_not_judged() -> Result<(), anyhow::Error> {
    // Recorded behaviour: `compute_receipt_digest_v2` judges no law (K3).
    // A version-1 document in the V2 shape parses and digests under the V2
    // domain to a value distinct from the golden; the verifier's
    // `UnsupportedVersion` DENY runs before any recompute (ADR-056 §3.5).
    let sealed = seal_receipt_v2(golden_draft()?)?;
    let mut document = golden_canonical_document()?;
    document["envelope_version"] = serde_json::json!(1);
    let parsed: ReceiptEnvelopeV2 = serde_json::from_value(document)?;
    assert_eq!(parsed.envelope_version, SchemaVersion::V1);
    assert_ne!(compute_receipt_digest_v2(&parsed)?, sealed.receipt_digest);
    Ok(())
}

// ── V1 disjointness ──────────────────────────────────────────────────

#[test]
fn v1_event_hash_of_the_same_decision_differs_from_v2_receipt_digest() -> Result<(), anyhow::Error>
{
    let v1 = crate::project_decision_payload(&golden_decision()?)?;
    let v2 = seal_receipt_v2(golden_draft()?)?;
    assert_ne!(v1.event_hash, v2.receipt_digest);
    Ok(())
}

#[test]
fn untagged_jcs_digest_of_the_preimage_is_not_the_v2_digest() -> Result<(), anyhow::Error> {
    // The V1 law is `BLAKE3(JCS(preimage))` with no tag. Applying it to the
    // V2 preimage must not reproduce `receipt_digest`: the tag is load-bearing.
    let sealed = seal_receipt_v2(golden_draft()?)?;
    let untagged =
        vr_jcs::to_canon_digest_with(&preimage_of(&sealed)?, &DigestStrategy::blake3_untagged())?;
    assert_ne!(untagged.bytes.as_slice(), sealed.receipt_digest.as_bytes());
    Ok(())
}
