//! Structural guard for the ADR-056 R12 mint ratchet on this crate (M2-1;
//! M2-0 D1).
//!
//! Reads this crate's own `src/` and asserts that no V1 `@0.1` mint path
//! remains and that no V2 constructor reaches for a leaf content identity:
//!
//! (a) no non-test source names a V1 receipt-type variant (`ReceiptType`
//!     `::`), writes the V1 `event_hash` key, or names the V1 envelope type
//!     (`ReceiptEnvelope` as a whole identifier, not `ReceiptEnvelopeV2`);
//! (b) the V1 mint symbols `seal_provider_interaction` and `seal_record`
//!     (without `_v2`) do not exist as functions;
//! (c) each of the six frozen `@0.1` derivations (`prompt_digest`, …,
//!     `record_policy_digest`) is named exactly once in non-test source —
//!     its definition — so no constructor calls it (D1: no leaf content
//!     identity in a V2 interaction).
//!
//! Every needle is assembled at runtime from fragments so this file can
//! never match itself.

use std::path::PathBuf;

fn non_test_sources() -> Result<Vec<(String, String)>, std::io::Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
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
            if name.ends_with("tests.rs") {
                continue;
            }
            files.push((name, std::fs::read_to_string(&path)?));
        }
    }
    files.sort();
    Ok(files)
}

const fn is_ident_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn count_identifier(text: &str, ident: &str) -> usize {
    let bytes = text.as_bytes();
    text.match_indices(ident)
        .filter(|(start, _)| {
            let end = start + ident.len();
            let before = start.checked_sub(1).and_then(|i| bytes.get(i).copied());
            let after = bytes.get(end).copied();
            !before.is_some_and(is_ident_char) && !after.is_some_and(is_ident_char)
        })
        .count()
}

#[test]
fn no_v1_envelope_type_variant_or_event_hash_write_remains() -> Result<(), std::io::Error> {
    let v1_type_path = ["Receipt", "Type", "::"].concat();
    let event_hash_key = ["\"event_", "hash\""].concat();
    let v1_envelope = ["Receipt", "Envelope"].concat();
    for (name, contents) in non_test_sources()? {
        assert!(
            !contents.contains(v1_type_path.as_str()),
            "{name} names a V1 receipt-type variant"
        );
        assert!(
            !contents.contains(event_hash_key.as_str()),
            "{name} writes the V1 event_hash key"
        );
        assert_eq!(
            count_identifier(&contents, &v1_envelope),
            0,
            "{name} names the V1 envelope type"
        );
    }
    Ok(())
}

#[test]
fn v1_mint_symbols_are_absent() -> Result<(), std::io::Error> {
    let needles = [
        ["fn seal_provider_", "interaction("].concat(),
        ["fn seal_", "record("].concat(),
    ];
    for (name, contents) in non_test_sources()? {
        for needle in &needles {
            assert!(
                !contents.contains(needle.as_str()),
                "{name} carries the V1 mint symbol {needle:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn no_constructor_calls_a_frozen_v1_derivation() -> Result<(), std::io::Error> {
    let derivations = [
        ["prompt_", "digest"].concat(),
        ["response_", "digest"].concat(),
        ["interaction_schema_", "digest"].concat(),
        ["capture_policy_", "digest"].concat(),
        ["record_schema_", "digest"].concat(),
        ["record_policy_", "digest"].concat(),
    ];
    for (name, contents) in non_test_sources()? {
        for derivation in &derivations {
            let mentions = contents
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .map(|line| count_identifier(line, derivation))
                .sum::<usize>();
            assert!(
                mentions <= 1,
                "{name} names {derivation} {mentions} times; only its definition may"
            );
        }
    }
    Ok(())
}
