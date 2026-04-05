//! Validate the `.vr/` governance surface for structural and semantic correctness.
//!
//! This binary reads repo-local governance files and validates invariants.
//! It does not verify upstream governance sources or produce receipts.
//! Exit 0 on success, exit 1 on any violation.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process;

use serde_json::Value;

// ── Constants ──────────────────────────────────────────────────────────

const AUTHORITY_SET_PATH: &str = ".vr/governance/bindings/authority-set.json";
const POLICY_SET_PATH: &str = ".vr/governance/bindings/policy-set.json";
const MANIFEST_PATH: &str = ".vr/governance/manifest.toml";
const CHAIN_MANIFEST_PATH: &str = ".vr/receipts/chain-manifest.json";
const CARGO_TOML_PATH: &str = "Cargo.toml";
const BINDINGS_DIR: &str = ".vr/governance/bindings";

const ALLOWED_GRADES: &[&str] = &["development", "managed", "production"];
const ALLOWED_CHAIN_STATUSES: &[&str] = &["genesis", "active", "sealed"];
const BLAKE3_HEX_LEN: usize = 64;

// ── Main ───────────────────────────────────────────────────────────────

fn main() {
    let errors = validate();
    if errors.is_empty() {
        eprintln!("governance surface: all checks passed");
        process::exit(0);
    }
    for e in &errors {
        eprintln!("  FAIL: {e}");
    }
    eprintln!("\ngovernance surface: {} failure(s)", errors.len());
    process::exit(1);
}

// ── Orchestrator ───────────────────────────────────────────────────────

fn validate() -> Vec<String> {
    let mut errors = Vec::new();

    // I8: bindings directory must exist
    if !Path::new(BINDINGS_DIR).is_dir() {
        errors.push(format!("I8: {BINDINGS_DIR} directory missing (fatal)"));
        return errors;
    }

    // I1: required file presence
    let required = [
        AUTHORITY_SET_PATH,
        POLICY_SET_PATH,
        MANIFEST_PATH,
        CHAIN_MANIFEST_PATH,
    ];
    let mut missing_any = false;
    for path in &required {
        if !Path::new(path).is_file() {
            errors.push(format!("I1: required file missing: {path}"));
            missing_any = true;
        }
    }
    if missing_any {
        return errors;
    }

    // I2 + I3 + I4: authority-set
    let authority_obj = validate_json_file(AUTHORITY_SET_PATH, &mut errors);

    // I2 + I3 + I5: policy-set
    let policy_obj = validate_json_file(POLICY_SET_PATH, &mut errors);

    // I2 + I6: manifest
    let manifest_table = validate_toml_file(MANIFEST_PATH, &mut errors);

    // I2 + I7: chain-manifest
    let chain_obj = validate_json_file(CHAIN_MANIFEST_PATH, &mut errors);

    // I4: authority-set invariants
    if let Some(obj) = &authority_obj {
        validate_authority_set(obj, &mut errors);
    }

    // I5: policy-set invariants
    let policy_digests_from_bindings = policy_obj
        .as_ref()
        .map_or_else(BTreeMap::new, |obj| validate_policy_set(obj, &mut errors));

    // I6: manifest invariants
    if let Some(table) = &manifest_table {
        validate_manifest(table, &policy_digests_from_bindings, &mut errors);
    }

    // I7: chain-manifest invariants
    if let Some(obj) = &chain_obj {
        validate_chain_manifest(obj, &mut errors);
    }

    errors
}

// ── File parsing ───────────────────────────────────────────────────────

fn validate_json_file(
    path: &str,
    errors: &mut Vec<String>,
) -> Option<serde_json::Map<String, Value>> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("I2: cannot read {path}: {e}"));
            return None;
        }
    };
    let value: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("I2: {path}: invalid JSON: {e}"));
            return None;
        }
    };
    if let Value::Object(map) = value {
        Some(map)
    } else {
        errors.push(format!("I2: {path}: top-level value is not a JSON object"));
        None
    }
}

fn validate_toml_file(path: &str, errors: &mut Vec<String>) -> Option<toml::Table> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("I2: cannot read {path}: {e}"));
            return None;
        }
    };
    match content.parse::<toml::Table>() {
        Ok(t) => Some(t),
        Err(e) => {
            errors.push(format!("I2: {path}: invalid TOML: {e}"));
            None
        }
    }
}

// ── I3 + I4: authority-set ─────────────────────────────────────────────

fn validate_authority_set(obj: &serde_json::Map<String, Value>, errors: &mut Vec<String>) {
    let path = AUTHORITY_SET_PATH;

    // I3: no legacy source field
    if obj.contains_key("source") {
        errors.push(format!("I3: {path}: forbidden 'source' field present"));
    }

    // I4: required non-empty strings
    require_non_empty_string(obj, "authority_set_id", path, "I4", errors);
    require_non_empty_string(obj, "digest", path, "I4", errors);
    require_non_empty_string(obj, "digest_of", path, "I4", errors);

    // I4: digest format (BLAKE3 lowercase hex)
    if let Some(Value::String(d)) = obj.get("digest") {
        validate_blake3_hex(d, path, "digest", errors);
    }

    // I4: grade must be an allowed value
    if let Some(grade_val) = obj.get("grade") {
        match grade_val.as_str() {
            Some(g) if ALLOWED_GRADES.contains(&g) => {}
            Some(g) => errors.push(format!(
                "I4: {path}: grade '{g}' not in allowed set: {ALLOWED_GRADES:?}"
            )),
            None => errors.push(format!("I4: {path}: grade must be a string")),
        }
    }

    // I4: epoch must be a non-negative integer
    if let Some(epoch_val) = obj.get("epoch") {
        if epoch_val.as_u64().is_none() {
            errors.push(format!("I4: {path}: epoch must be a non-negative integer"));
        }
    }
}

// ── I3 + I5: policy-set ───────────────────────────────────────────────

fn validate_policy_set(
    obj: &serde_json::Map<String, Value>,
    errors: &mut Vec<String>,
) -> BTreeMap<String, String> {
    let path = POLICY_SET_PATH;
    let mut policy_digests: BTreeMap<String, String> = BTreeMap::new();

    // I5: bindings array must exist
    let bindings = match obj.get("bindings") {
        Some(Value::Array(arr)) => arr,
        Some(_) => {
            errors.push(format!("I5: {path}: 'bindings' must be an array"));
            return policy_digests;
        }
        None => {
            errors.push(format!("I5: {path}: 'bindings' field missing"));
            return policy_digests;
        }
    };

    for (i, binding) in bindings.iter().enumerate() {
        let Some(binding_obj) = binding.as_object() else {
            errors.push(format!("I5: {path}: bindings[{i}] is not a JSON object"));
            continue;
        };

        // I3: no source field in bindings
        if binding_obj.contains_key("source") {
            errors.push(format!(
                "I3: {path}: bindings[{i}] contains forbidden 'source' field"
            ));
        }

        // I5: policy_id required and non-empty
        let policy_id = match binding_obj.get("policy_id").and_then(Value::as_str) {
            Some(id) if !id.is_empty() => id.to_owned(),
            Some(_) => {
                errors.push(format!("I5: {path}: bindings[{i}].policy_id is empty"));
                continue;
            }
            None => {
                errors.push(format!(
                    "I5: {path}: bindings[{i}].policy_id missing or not a string"
                ));
                continue;
            }
        };

        // I5: no duplicate policy_id
        let digest_str = binding_obj
            .get("digest")
            .and_then(Value::as_str)
            .map_or_else(String::new, String::from);
        if policy_digests
            .insert(policy_id.clone(), digest_str)
            .is_some()
        {
            errors.push(format!("I5: {path}: duplicate policy_id '{policy_id}'"));
        }

        // I5: digest must be valid BLAKE3 hex if present
        if let Some(Value::String(d)) = binding_obj.get("digest") {
            validate_blake3_hex(d, path, &format!("bindings[{i}].digest"), errors);
        }

        // I5: overlay path must exist if declared
        if let Some(Value::String(overlay)) = binding_obj.get("overlay") {
            if !overlay.is_empty() {
                let overlay_path = format!(".vr/governance/{overlay}");
                if !Path::new(&overlay_path).is_file() {
                    errors.push(format!(
                        "I5: {path}: bindings[{i}].overlay '{overlay}' \
                         resolved to '{overlay_path}' which does not exist"
                    ));
                }
            }
        }
    }

    // I5: repo_local_policies structure
    if let Some(rlp) = obj.get("repo_local_policies") {
        if !rlp.is_array() {
            errors.push(format!("I5: {path}: repo_local_policies must be an array"));
        }
    }

    policy_digests
}

// ── I6: manifest ──────────────────────────────────────────────────────

fn validate_manifest(
    table: &toml::Table,
    policy_digests_from_bindings: &BTreeMap<String, String>,
    errors: &mut Vec<String>,
) {
    let path = MANIFEST_PATH;

    // I6: version must match Cargo.toml
    let manifest_version = table
        .get("crate")
        .and_then(toml::Value::as_table)
        .and_then(|c| c.get("version"))
        .and_then(toml::Value::as_str);

    let cargo_version = read_cargo_version();

    match (manifest_version, &cargo_version) {
        (Some(mv), Some(cv)) if mv != cv.as_str() => {
            errors.push(format!(
                "I6: {path}: [crate].version = '{mv}' but {CARGO_TOML_PATH} \
                 package.version = '{cv}'"
            ));
        }
        (None, _) => {
            errors.push(format!("I6: {path}: [crate].version missing"));
        }
        _ => {}
    }

    // I6: stage must be declared
    if table.get("stage").is_none() {
        errors.push(format!("I6: {path}: [stage] section missing"));
    }

    // I6: cross-check manifest policy_bindings against policy-set (IDs and digests)
    if let Some(pb) = table.get("policy_bindings").and_then(toml::Value::as_table) {
        let binding_ids: BTreeSet<&String> = policy_digests_from_bindings.keys().collect();

        // Build manifest policy map: full_id -> digest
        let manifest_policies: BTreeMap<String, String> = pb
            .keys()
            .map(|k| {
                let entry = pb.get(k).and_then(toml::Value::as_table);
                let version = entry
                    .and_then(|t| t.get("version"))
                    .and_then(toml::Value::as_str)
                    .map_or("?", |v| v);
                let digest = entry
                    .and_then(|t| t.get("digest"))
                    .and_then(toml::Value::as_str)
                    .map_or_else(String::new, String::from);
                (format!("{k}@{version}"), digest)
            })
            .collect();

        let manifest_ids: BTreeSet<&String> = manifest_policies.keys().collect();

        for id in manifest_ids.difference(&binding_ids) {
            errors.push(format!(
                "I6: {path}: policy '{id}' in manifest [policy_bindings] \
                 but absent from {POLICY_SET_PATH} bindings"
            ));
        }
        for id in binding_ids.difference(&manifest_ids) {
            errors.push(format!(
                "I6: {path}: policy '{id}' in {POLICY_SET_PATH} bindings \
                 but absent from manifest [policy_bindings]"
            ));
        }

        // I6: digest values must match where both files carry the same policy
        for (id, manifest_digest) in &manifest_policies {
            if let Some(binding_digest) = policy_digests_from_bindings.get(id) {
                if !manifest_digest.is_empty()
                    && !binding_digest.is_empty()
                    && manifest_digest != binding_digest
                {
                    errors.push(format!(
                        "I6: {path}: policy '{id}' digest mismatch: \
                         manifest='{manifest_digest}' vs \
                         {POLICY_SET_PATH}='{binding_digest}'"
                    ));
                }
            }
        }
    }
}

fn read_cargo_version() -> Option<String> {
    let content = std::fs::read_to_string(CARGO_TOML_PATH).ok()?;
    let table: toml::Table = content.parse().ok()?;
    table
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|p| p.get("version"))
        .and_then(toml::Value::as_str)
        .map(String::from)
}

// ── I7: chain-manifest ────────────────────────────────────────────────

fn validate_chain_manifest(obj: &serde_json::Map<String, Value>, errors: &mut Vec<String>) {
    let path = CHAIN_MANIFEST_PATH;

    // I7: chain_status must be one of allowed values
    let status = match obj.get("chain_status").and_then(Value::as_str) {
        Some(s) if ALLOWED_CHAIN_STATUSES.contains(&s) => Some(s),
        Some(s) => {
            errors.push(format!(
                "I7: {path}: chain_status '{s}' not in allowed set: \
                 {ALLOWED_CHAIN_STATUSES:?}"
            ));
            None
        }
        None => {
            errors.push(format!("I7: {path}: chain_status missing or not a string"));
            None
        }
    };

    // I7: receipt_count must be a non-negative integer
    let receipt_count = obj.get("receipt_count").map_or_else(
        || {
            errors.push(format!("I7: {path}: receipt_count missing"));
            None
        },
        serde_json::Value::as_u64,
    );

    // I7: genesis state constraints
    if status == Some("genesis") {
        if let Some(count) = receipt_count {
            if count != 0 {
                errors.push(format!(
                    "I7: {path}: chain_status is 'genesis' but \
                     receipt_count is {count} (must be 0)"
                ));
            }
        }
        // chain_root and latest_receipt_id must be null or absent
        if let Some(root) = obj.get("chain_root") {
            if !root.is_null() {
                if let Some(s) = root.as_str() {
                    if !s.is_empty() {
                        errors.push(format!(
                            "I7: {path}: chain_status is 'genesis' but \
                             chain_root is non-null/non-empty"
                        ));
                    }
                }
            }
        }
        if let Some(latest) = obj.get("latest_receipt_id") {
            if !latest.is_null() {
                if let Some(s) = latest.as_str() {
                    if !s.is_empty() {
                        errors.push(format!(
                            "I7: {path}: chain_status is 'genesis' but \
                             latest_receipt_id is non-null/non-empty"
                        ));
                    }
                }
            }
        }
    }

    // I7: non-genesis with receipt_count > 0 must have chain_root and latest_receipt_id
    if let Some(count) = receipt_count {
        if count > 0 {
            let has_root = obj
                .get("chain_root")
                .and_then(Value::as_str)
                .is_some_and(|s| !s.is_empty());
            let has_latest = obj
                .get("latest_receipt_id")
                .and_then(Value::as_str)
                .is_some_and(|s| !s.is_empty());

            if !has_root {
                errors.push(format!(
                    "I7: {path}: receipt_count is {count} but chain_root \
                     is absent or empty"
                ));
            }
            if !has_latest {
                errors.push(format!(
                    "I7: {path}: receipt_count is {count} but \
                     latest_receipt_id is absent or empty"
                ));
            }
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

fn require_non_empty_string(
    obj: &serde_json::Map<String, Value>,
    field: &str,
    file_path: &str,
    invariant: &str,
    errors: &mut Vec<String>,
) {
    match obj.get(field) {
        Some(Value::String(s)) if !s.is_empty() => {}
        Some(Value::String(_)) => {
            errors.push(format!("{invariant}: {file_path}: '{field}' is empty"));
        }
        Some(_) => {
            errors.push(format!(
                "{invariant}: {file_path}: '{field}' must be a string"
            ));
        }
        None => {
            errors.push(format!("{invariant}: {file_path}: '{field}' is missing"));
        }
    }
}

fn validate_blake3_hex(value: &str, file_path: &str, field_name: &str, errors: &mut Vec<String>) {
    if value.len() != BLAKE3_HEX_LEN {
        errors.push(format!(
            "I4: {file_path}: {field_name} length is {} (expected {BLAKE3_HEX_LEN})",
            value.len()
        ));
        return;
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        errors.push(format!(
            "I4: {file_path}: {field_name} is not lowercase hex"
        ));
    }
}
