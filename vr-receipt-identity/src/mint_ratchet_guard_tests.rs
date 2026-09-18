//! Structural guard for the ADR-056 R12 mint ratchet (C11).
//!
//! Reads this crate's own `src/` files and asserts that no V1
//! governance-decision mint path remains:
//!
//! (a) no source file names the V1 `governance` variant of the V1 receipt-type
//!     enum (path `ReceiptType` `::` `Governance`);
//! (b) no source file outside the `commitment` module writes the
//!     `event_hash` key (the module's test file is compiled *into*
//!     `commitment.rs` as its `tests` submodule and carries the frozen G1
//!     fixture, so it is part of that module);
//! (c) the V1 mint symbol `project_decision_payload` (without `_v2`) does
//!     not exist as a `fn`;
//! (d) no non-comment line outside the `commitment` module names the V1
//!     envelope type (`ReceiptEnvelope` as a whole identifier, not
//!     `ReceiptEnvelopeV2`), so a V1 envelope cannot be assembled by struct
//!     literal and hashed through the still-public `compute_event_hash`
//!     anywhere but the verify-only module.
//!
//! The scan walks `src/` recursively, so a new submodule directory cannot
//! side-step it. Every needle is assembled at runtime from fragments so
//! this file can never match itself.

use std::path::{Path, PathBuf};

fn src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// `(path relative to `src/`, contents)` for every `.rs` file anywhere
/// under `src/`, sorted by path.
fn source_files() -> Result<Vec<(String, String)>, std::io::Error> {
    let root = src_dir();
    let mut files = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let name = path
                .strip_prefix(&root)
                .ok()
                .and_then(|p| p.to_str())
                .map(str::to_string)
                .ok_or_else(|| std::io::Error::other("non-UTF-8 source file path"))?;
            files.push((name, std::fs::read_to_string(&path)?));
        }
    }
    files.sort();
    Ok(files)
}

fn offenders(needle: &str, exempt: &[&str]) -> Result<Vec<String>, std::io::Error> {
    Ok(source_files()?
        .into_iter()
        .filter(|(name, contents)| !exempt.contains(&name.as_str()) && contents.contains(needle))
        .map(|(name, _)| name)
        .collect())
}

const fn is_ident_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Whether `line` contains `ident` as a whole identifier (not as a prefix
/// or suffix of a longer one).
fn names_identifier(line: &str, ident: &str) -> bool {
    let bytes = line.as_bytes();
    line.match_indices(ident).any(|(start, _)| {
        let end = start + ident.len();
        let before = start.checked_sub(1).and_then(|i| bytes.get(i).copied());
        let after = bytes.get(end).copied();
        !before.is_some_and(is_ident_char) && !after.is_some_and(is_ident_char)
    })
}

/// `(path, line number)` of every non-comment line outside `exempt` that
/// names `ident` as a whole identifier.
fn identifier_offenders(
    ident: &str,
    exempt: &[&str],
) -> Result<Vec<(String, usize)>, std::io::Error> {
    Ok(source_files()?
        .iter()
        .filter(|(name, _)| !exempt.contains(&name.as_str()))
        .flat_map(|(name, contents)| {
            contents
                .lines()
                .enumerate()
                .filter(|(_, line)| !line.trim_start().starts_with("//"))
                .filter(|(_, line)| names_identifier(line, ident))
                .map(|(index, _)| (name.clone(), index + 1))
                .collect::<Vec<_>>()
        })
        .collect())
}

#[test]
fn guard_scans_the_crate_source_tree() -> Result<(), std::io::Error> {
    let files = source_files()?;
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    for required in ["lib.rs", "commitment.rs", "decision_projection.rs"] {
        assert!(
            names.contains(&required),
            "guard must see {required}; saw {names:?}"
        );
    }
    assert!(Path::new(&src_dir()).is_dir());
    Ok(())
}

#[test]
fn no_source_file_names_the_v1_governance_receipt_type() -> Result<(), std::io::Error> {
    let needle = ["ReceiptType", "::", "Governance"].concat();
    let found = offenders(&needle, &[])?;
    assert!(
        found.is_empty(),
        "V1 `{needle}` must not be named anywhere in vr-receipt-identity; found in {found:?}"
    );
    Ok(())
}

#[test]
fn only_the_commitment_module_writes_the_event_hash_key() -> Result<(), std::io::Error> {
    let key = ["event", "_hash"].concat();
    let json_key_write = format!("\"{key}\":");
    let map_insert = format!("insert(\"{key}\"");
    let exempt = ["commitment.rs", "commitment_tests.rs"];
    for needle in [json_key_write, map_insert] {
        let found = offenders(&needle, &exempt)?;
        assert!(
            found.is_empty(),
            "`{needle}` written outside the commitment module: {found:?}"
        );
    }
    Ok(())
}

#[test]
fn the_v1_decision_mint_symbol_does_not_exist_as_a_fn() -> Result<(), std::io::Error> {
    let needle = ["fn project_", "decision_payload", "("].concat();
    let found = offenders(&needle, &[])?;
    assert!(
        found.is_empty(),
        "the V1 mint `{needle}...)` must not exist (R12 MintDenied); found in {found:?}"
    );
    Ok(())
}

#[test]
fn only_the_commitment_module_names_the_v1_envelope_type() -> Result<(), std::io::Error> {
    let ident = ["Receipt", "Envelope"].concat();
    let exempt = ["commitment.rs", "commitment_tests.rs"];
    let found = identifier_offenders(&ident, &exempt)?;
    assert!(
        found.is_empty(),
        "the V1 `{ident}` type is named outside the verify-only commitment module \
         (a V1 envelope could be assembled and hashed there): {found:?}"
    );
    Ok(())
}

#[test]
fn identifier_matching_is_whole_word() {
    let ident = ["Receipt", "Envelope"].concat();
    assert!(names_identifier(&format!("let e: {ident} = x;"), &ident));
    assert!(names_identifier(&format!("use a::{{{ident}}};"), &ident));
    assert!(!names_identifier(&format!("let e: {ident}V2 = x;"), &ident));
    assert!(!names_identifier(&format!("let e: My{ident} = x;"), &ident));
    assert!(!names_identifier(&format!("{ident}_v2()"), &ident));
}
