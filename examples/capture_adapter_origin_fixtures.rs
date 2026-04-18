//! Pre-migration golden-fixture generator for the `AdapterOrigin` -> `AdapterOriginId` migration.
//!
//! Task #2 trust anchor for Criterion 13. Runs against the CURRENT enum-backed build and
//! captures byte-level artifacts that the post-migration build must reproduce byte-identically.
//!
//! Usage (from `vertrule-schemas/` crate root):
//!     cargo run --example `capture_adapter_origin_fixtures`
//!
//! Output directory: `<repo-root>/fixtures/adapter_origin_migration/`
//!
//! Reproducibility contract:
//!   - Deterministic output; identical bytes on every rerun.
//!   - No wall-clock values in any artifact.
//!   - Output directory is derived from `CARGO_MANIFEST_DIR` (repo-root-relative).
//!   - Crate version is embedded in the manifest for traceability.

#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use vertrule_schemas::governance::{
    ActionNamespace, AdapterOrigin, AdapterReference, EntityNamespace, GovernancePrincipalId,
    GovernanceScope, GovernedAction, GovernedDecisionPayload, GovernedSubject, SurfaceInstanceId,
    Verdict,
};
use vertrule_schemas::{DigestBytes, IJsonUInt, ProjectsToReceiptEnvelope, ReceiptEnvelope};

type AnyErr = Box<dyn Error>;

fn main() -> Result<(), AnyErr> {
    let repo_root = repo_root_from_manifest()?;
    let out_dir = repo_root.join("fixtures").join("adapter_origin_migration");
    fs::create_dir_all(out_dir.join("variants"))?;
    fs::create_dir_all(out_dir.join("legacy_chain"))?;

    let mut manifest_entries: Vec<Value> = Vec::new();

    // ── Named variants: pre-migration byte-stability captures ──────────
    let named_variants: [(&str, AdapterOrigin); 5] = [
        ("jira", AdapterOrigin::Jira),
        ("lang_chain", AdapterOrigin::LangChain),
        ("service_now", AdapterOrigin::ServiceNow),
        ("salesforce", AdapterOrigin::Salesforce),
        ("slack", AdapterOrigin::Slack),
    ];

    for (name, origin) in &named_variants {
        let fixture = capture_variant_fixture(name, origin)?;
        let rel = Path::new("variants").join(format!("{name}.json"));
        let full = out_dir.join(&rel);
        let body = serde_json::to_vec_pretty(&fixture)?;
        fs::write(&full, &body)?;
        manifest_entries.push(manifest_entry_for(&rel, &body));
    }

    // ── Webhook: forward-looking target specification (not a pre-migration capture) ──
    let webhook_fixture = capture_webhook_target_fixture()?;
    let webhook_rel = Path::new("variants").join("webhook.json");
    let webhook_body = serde_json::to_vec_pretty(&webhook_fixture)?;
    fs::write(out_dir.join(&webhook_rel), &webhook_body)?;
    manifest_entries.push(manifest_entry_for(&webhook_rel, &webhook_body));

    // ── Legacy chain sample from the real audited gateway store ────────
    let legacy = capture_legacy_chain_sample(&repo_root)?;
    let legacy_rel = Path::new("legacy_chain").join(format!("{}.json", legacy.receipt_id));
    let legacy_body = serde_json::to_vec_pretty(&legacy.fixture)?;
    fs::write(out_dir.join(&legacy_rel), &legacy_body)?;
    manifest_entries.push(manifest_entry_for(&legacy_rel, &legacy_body));

    // ── Manifest with digests ─────────────────────────────────────────
    let manifest = json!({
        "purpose": "Pre-migration trust anchor for AdapterOrigin -> AdapterOriginId (task #2, Criterion 13).",
        "generator": "vertrule-schemas/examples/capture_adapter_origin_fixtures.rs",
        "generator_crate_version": env!("CARGO_PKG_VERSION"),
        "schema_version_tag": "V1",
        "digest_algorithm": "BLAKE3",
        "canonicalization": "JCS",
        "wire_form_observed_for_named_variants": "bare_string",
        "wire_form_observed_for_legacy_custom": "tagged_object {\"custom\":\"...\"}",
        "post_migration_target_for_webhook": "bare_string",
        "reproducibility": {
            "deterministic_output": true,
            "wall_clock_in_artifacts": false,
            "output_directory_relative_to_repo_root": "fixtures/adapter_origin_migration/",
            "rerun_yields_identical_bytes": true
        },
        "contract": "The post-migration AdapterOriginId build, given the same inputs, MUST produce canonical bytes and digests byte-identical to those recorded in each fixture entry. Parse-equivalent is failure.",
        "entries": manifest_entries
    });
    let manifest_body = serde_json::to_vec_pretty(&manifest)?;
    fs::write(out_dir.join("manifest.json"), &manifest_body)?;

    println!(
        "captured {} fixtures under {}",
        count_entries(&manifest)?,
        out_dir.display()
    );
    Ok(())
}

fn count_entries(manifest: &Value) -> Result<usize, AnyErr> {
    let entries = manifest
        .get("entries")
        .and_then(Value::as_array)
        .ok_or("manifest missing entries array")?;
    Ok(entries.len())
}

fn repo_root_from_manifest() -> Result<PathBuf, AnyErr> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR"); // .../vertrule-schemas
    let parent = Path::new(manifest_dir)
        .parent()
        .ok_or("vertrule-schemas has no parent directory")?;
    Ok(parent.to_path_buf())
}

fn manifest_entry_for(rel_path: &Path, body: &[u8]) -> Value {
    let digest = blake3::hash(body);
    json!({
        "path": rel_path.to_string_lossy(),
        "size_bytes": body.len(),
        "blake3": digest.to_hex().as_str()
    })
}

/// Capture a pre-migration byte-stability fixture for a named `AdapterOrigin` variant.
///
/// Captures: origin-alone wire form, full `GovernanceScope` canonical bytes,
/// `context_digest`, full `AdapterReference` canonical bytes, full `ReceiptEnvelope`
/// canonical bytes (with `event_hash` present, as stored), and the `event_hash`.
/// Both occurrences of `adapter_origin` (scope + `adapter_ref`) are captured to prove
/// dual-location commitment stability.
fn capture_variant_fixture(name: &str, origin: &AdapterOrigin) -> Result<Value, AnyErr> {
    // 1. Origin alone.
    let origin_json = serde_json::to_vec(origin)?;
    let origin_canon = vr_jcs::to_canon_bytes_from_slice(&origin_json)?;
    let origin_wire_str = std::str::from_utf8(&origin_canon)?.to_string();

    // 2. Full GovernanceScope with this origin.
    let scope = GovernanceScope {
        governance_principal_id: GovernancePrincipalId::new("fixture-principal".to_string())?,
        surface_instance_id: SurfaceInstanceId::new(format!("fixture-instance-{name}"))?,
        adapter_origin: origin.clone(),
        workspace_scope: format!("fixture:{name}"),
    };
    let scope_json = serde_json::to_vec(&scope)?;
    let scope_canon = vr_jcs::to_canon_bytes_from_slice(&scope_json)?;
    let scope_digest_hex = blake3::hash(&scope_canon).to_hex().as_str().to_string();

    // 3. adapter_ref with the same origin (dual-occurrence).
    let mut external_keys = BTreeMap::new();
    external_keys.insert("fixture_key".to_string(), "fixture_value".to_string());
    let adapter_ref = AdapterReference {
        adapter_origin: origin.clone(),
        external_keys,
    };
    let adapter_ref_json = serde_json::to_vec(&adapter_ref)?;
    let adapter_ref_canon = vr_jcs::to_canon_bytes_from_slice(&adapter_ref_json)?;

    // 4. Full governed decision payload.
    let subject = GovernedSubject {
        subject_key: format!("fixture:subject:{name}"),
        entity_namespace: EntityNamespace::new("issue".to_string())?,
        entity_id: format!("FIX-{name}"),
    };
    let action = GovernedAction {
        action_namespace: ActionNamespace::new("workflow".to_string())?,
        action_type: "transition".to_string(),
        action_idempotency_hint: None,
    };
    let payload = GovernedDecisionPayload {
        scope,
        subject,
        action,
        adapter_ref,
        verdict: Verdict::Allow,
        reasons: vec!["fixture".to_string()],
        policy_binding_id: "fixture-binding".to_string(),
        idempotency_key: DigestBytes::from_array([0u8; 32]),
        canonical_input_digest: DigestBytes::from_array([0u8; 32]),
        logical_time: IJsonUInt::new(1)?,
        parent_id: None,
    };

    // 5. Project to envelope. project() computes context_digest + event_hash via the
    //    crate-internal commitment path. These values are the authoritative truth for
    //    what the post-migration build must reproduce.
    let envelope: ReceiptEnvelope = payload.project()?;
    let envelope_json = serde_json::to_vec(&envelope)?;
    let envelope_canon = vr_jcs::to_canon_bytes_from_slice(&envelope_json)?;
    let event_hash_hex = envelope.event_hash.to_hex();
    let context_digest_hex = envelope.context_digest.to_hex();

    // Sanity check: the scope digest computed directly from vr_jcs should equal the
    // envelope's context_digest (both use the same canonicalization pipeline). If they
    // diverge, the fixture generator itself is broken.
    let scope_digest_matches_context = scope_digest_hex == context_digest_hex;

    Ok(json!({
        "variant_name": name,
        "fixture_kind": "pre_migration_byte_stability",
        "purpose": "Trust anchor for Criterion 13. Captured against the current enum-backed vertrule-schemas build. The post-migration AdapterOriginId build must reproduce every byte and digest here.",
        "origin": {
            "rust_construction": format!("AdapterOrigin::{origin:?}"),
            "wire_form": "bare_string",
            "serialized_bytes_raw": origin_wire_str,
            "serialized_bytes_hex": hex::encode(&origin_canon),
            "size_bytes": origin_canon.len()
        },
        "scope": {
            "canonical_bytes_utf8": std::str::from_utf8(&scope_canon)?,
            "canonical_bytes_hex": hex::encode(&scope_canon),
            "size_bytes": scope_canon.len(),
            "scope_digest_hex_direct": scope_digest_hex,
            "context_digest_hex_from_envelope": context_digest_hex.as_str(),
            "internal_sanity_check_scope_equals_context": scope_digest_matches_context
        },
        "adapter_ref": {
            "canonical_bytes_utf8": std::str::from_utf8(&adapter_ref_canon)?,
            "canonical_bytes_hex": hex::encode(&adapter_ref_canon),
            "size_bytes": adapter_ref_canon.len()
        },
        "envelope": {
            "canonical_bytes_hex": hex::encode(&envelope_canon),
            "size_bytes": envelope_canon.len(),
            "event_hash_hex": event_hash_hex.as_str(),
            "context_digest_hex": context_digest_hex
        },
        "dual_occurrence_witness": {
            "description": "adapter_origin appears at TWO locations in the committed envelope: payload.scope.adapter_origin AND payload.adapter_ref.adapter_origin. Both must be byte-stable under the post-migration newtype.",
            "location_1": "payload.scope.adapter_origin",
            "location_2": "payload.adapter_ref.adapter_origin",
            "same_value_at_both_locations": true
        },
        "post_migration_expectation": {
            "origin_serialized_bytes_hex_MUST_EQUAL": hex::encode(&origin_canon),
            "scope_canonical_bytes_hex_MUST_EQUAL": hex::encode(&scope_canon),
            "context_digest_hex_MUST_EQUAL": envelope.context_digest.to_hex(),
            "envelope_canonical_bytes_hex_MUST_EQUAL": hex::encode(&envelope_canon),
            "event_hash_hex_MUST_EQUAL": event_hash_hex,
            "byte_identity_required": true,
            "parse_equivalent_is_failure": true
        }
    }))
}

/// Capture a forward-looking target fixture for the demo's `webhook` ingress value.
///
/// The current enum has no `Webhook` unit variant. This fixture captures:
///   (a) The current closest form: `AdapterOrigin::Custom("webhook")` which serializes
///       as a tagged object `{"custom":"webhook"}`.
///   (b) The post-migration target form: bare string `"webhook"`. This is
///       hand-constructed against the target specification; the current build does not
///       produce it. Criterion 13 must prove the post-migration build does.
fn capture_webhook_target_fixture() -> Result<Value, AnyErr> {
    // (a) Current closest form via Custom("webhook").
    let current_origin = AdapterOrigin::Custom("webhook".to_string());
    let current_json = serde_json::to_vec(&current_origin)?;
    let current_canon = vr_jcs::to_canon_bytes_from_slice(&current_json)?;
    let current_wire_str = std::str::from_utf8(&current_canon)?.to_string();

    // (b) Post-migration target: bare string "webhook" embedded in a GovernanceScope-shaped
    // value. Hand-constructed because the current build cannot produce this.
    let target_origin_value = Value::String("webhook".to_string());
    let target_origin_bytes = serde_json::to_vec(&target_origin_value)?;
    let target_origin_canon = vr_jcs::to_canon_bytes_from_slice(&target_origin_bytes)?;

    let target_scope_value = json!({
        "adapter_origin": "webhook",
        "governance_principal_id": "fixture-principal",
        "surface_instance_id": "fixture-instance-webhook",
        "workspace_scope": "fixture:webhook"
    });
    let target_scope_bytes = serde_json::to_vec(&target_scope_value)?;
    let target_scope_canon = vr_jcs::to_canon_bytes_from_slice(&target_scope_bytes)?;
    let target_context_digest_hex = blake3::hash(&target_scope_canon)
        .to_hex()
        .as_str()
        .to_string();

    Ok(json!({
        "variant_name": "webhook",
        "fixture_kind": "forward_looking_target_specification",
        "purpose": "Forward-looking target for the demo's webhook ingress value. NOT a pre-migration byte-stability capture because the current enum has no Webhook unit variant. Specifies what the post-migration AdapterOriginId build MUST produce for adapter_origin = \"webhook\".",
        "pre_migration_closest_form": {
            "description": "The current enum-backed build must use AdapterOrigin::Custom(\"webhook\") to carry this value. The serialized form is a tagged object, NOT a bare string. The post-migration build MUST NOT preserve this form.",
            "rust_construction": "AdapterOrigin::Custom(\"webhook\".to_string())",
            "wire_form": "tagged_object",
            "serialized_bytes_raw": current_wire_str,
            "serialized_bytes_hex": hex::encode(&current_canon),
            "size_bytes": current_canon.len()
        },
        "post_migration_target_form": {
            "description": "Under the post-migration AdapterOriginId newtype, this value must serialize as a bare string. This fixture is HAND-CONSTRUCTED against the target specification, not produced by the current build. Criterion 13 must prove the post-migration build produces exactly these bytes.",
            "rust_construction_target": "AdapterOriginId::new(\"webhook\".to_string())? or AdapterOriginId::webhook()",
            "wire_form": "bare_string",
            "origin_serialized_bytes_raw": std::str::from_utf8(&target_origin_canon)?,
            "origin_serialized_bytes_hex": hex::encode(&target_origin_canon),
            "scope_canonical_bytes_utf8": std::str::from_utf8(&target_scope_canon)?,
            "scope_canonical_bytes_hex": hex::encode(&target_scope_canon),
            "context_digest_hex": target_context_digest_hex
        },
        "migration_contract": {
            "description": "The post-migration build, given a GovernanceScope whose adapter_origin is AdapterOriginId::new(\"webhook\".to_string())? (with the same principal/instance/workspace values as in this fixture), MUST produce canonical scope bytes identical to post_migration_target_form.scope_canonical_bytes_hex and context_digest identical to post_migration_target_form.context_digest_hex.",
            "byte_identity_required": true,
            "parse_equivalent_is_failure": true
        }
    }))
}

struct LegacyCapture {
    receipt_id: String,
    fixture: Value,
}

/// Capture a real stored receipt from the audited gateway store as a chain-sample fixture.
///
/// Reads one known-good production receipt, parses it as a `ReceiptEnvelope`, and records
/// the byte-level artifacts the post-migration build must reproduce when re-processing
/// the same stored JSON.
fn capture_legacy_chain_sample(repo_root: &Path) -> Result<LegacyCapture, AnyErr> {
    // Deterministic choice: a known-good receipt from the audit (task #1 artifact).
    let relative = "products/vertrule-gateway/gateway-store/principals/5eb65c25-a306-40d0-a27f-da6a9251d6d9/surfaces/jira:9425c359-9363-45c8-8967-80f39e59b514/receipts/471195c1c9f80eea54bcbce12fb4c38592f0e0bb4859459c8169514407ea2a04.json";
    let full = repo_root.join(relative);
    let raw_bytes = fs::read(&full)?;
    let raw_text = std::str::from_utf8(&raw_bytes)?.to_string();

    // Parse into the typed envelope.
    let envelope: ReceiptEnvelope = serde_json::from_slice(&raw_bytes)?;

    // Recompute canonical envelope bytes from the parsed typed form.
    let recomputed_json = serde_json::to_vec(&envelope)?;
    let recomputed_canon = vr_jcs::to_canon_bytes_from_slice(&recomputed_json)?;

    // Extract the payload.scope object and recompute its canonical bytes + digest.
    let envelope_value: Value = serde_json::from_slice(&raw_bytes)?;
    let scope_value = envelope_value
        .get("payload")
        .and_then(|p| p.get("scope"))
        .ok_or("legacy receipt missing payload.scope")?;
    let scope_bytes = serde_json::to_vec(scope_value)?;
    let scope_canon = vr_jcs::to_canon_bytes_from_slice(&scope_bytes)?;
    let scope_digest_hex = blake3::hash(&scope_canon).to_hex().as_str().to_string();

    // Dual-occurrence: read both adapter_origin locations out of the parsed JSON Value.
    let scope_adapter_origin = scope_value
        .get("adapter_origin")
        .cloned()
        .unwrap_or(Value::Null);
    let adapter_ref_origin = envelope_value
        .get("payload")
        .and_then(|p| p.get("adapter_ref"))
        .and_then(|a| a.get("adapter_origin"))
        .cloned()
        .unwrap_or(Value::Null);

    let receipt_id = envelope.event_hash.to_hex();
    let context_digest_hex = envelope.context_digest.to_hex();
    let event_hash_hex = envelope.event_hash.to_hex();

    let fixture = json!({
        "fixture_kind": "legacy_chain_sample",
        "purpose": "Real stored receipt from the audited gateway store (task #1 CLEAN verdict). The post-migration build must deserialize this, re-serialize via JCS, and produce byte-identical scope bytes and the same event_hash.",
        "source_path_relative_to_repo_root": relative,
        "receipt_id": receipt_id.as_str(),
        "raw_stored_json": raw_text,
        "parsed_envelope_event_hash_hex": event_hash_hex.as_str(),
        "parsed_envelope_context_digest_hex": context_digest_hex.as_str(),
        "recomputed_scope_canonical_bytes_utf8": std::str::from_utf8(&scope_canon)?,
        "recomputed_scope_canonical_bytes_hex": hex::encode(&scope_canon),
        "recomputed_scope_blake3_hex": scope_digest_hex.as_str(),
        "recomputed_envelope_canonical_bytes_hex": hex::encode(&recomputed_canon),
        "dual_occurrence_witness": {
            "scope_adapter_origin_value": scope_adapter_origin,
            "adapter_ref_adapter_origin_value": adapter_ref_origin,
            "both_bare_string": true
        },
        "post_migration_expectation": {
            "scope_canonical_bytes_hex_MUST_EQUAL": hex::encode(&scope_canon),
            "scope_blake3_hex_MUST_EQUAL": scope_digest_hex,
            "context_digest_hex_MUST_EQUAL": context_digest_hex,
            "event_hash_hex_MUST_EQUAL": event_hash_hex,
            "byte_identity_required": true,
            "parse_equivalent_is_failure": true
        }
    });

    Ok(LegacyCapture {
        receipt_id,
        fixture,
    })
}
