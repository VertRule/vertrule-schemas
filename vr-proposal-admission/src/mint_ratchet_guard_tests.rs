//! Structural guard for the ADR-056 R12 mint ratchet on this crate
//! (M2-0 D4: `VerifyConstructionAllowed ⇏ MintConstructionAllowed`).
//!
//! The V1 `@0.1` proposal/admission mint is `MintDenied`, but the V1
//! admission construction must stay verifier-computable for the frozen
//! history (ADR-054 E2; G2-2 §4), so the C11 needle set (no `event_hash`
//! write, no V1 envelope identifier) cannot apply here. The guard is
//! written against the API shape instead: reading this crate's own `src/`,
//!
//! (a) no `pub` function's return type names the V1 envelope type
//!     (`ReceiptEnvelope` as a whole identifier, not `ReceiptEnvelopeV2`),
//!     so no V1 envelope — freshly sealed or rebuilt — can escape;
//! (b) no `pub struct` field or accessor of a V1 container exists: the V1
//!     symbols `SealedAgentProposal`, `SealedAdmittedProposal`,
//!     `seal_proposal` and `admit_proposal` (without `_v2`) are absent as
//!     items;
//! (c) exactly one function writes the V1 `event_hash` key, and it is the
//!     verify-only rebuild.
//!
//! Every needle is assembled at runtime from fragments so this file can
//! never match itself.

use std::path::PathBuf;

fn source_files() -> Result<Vec<(String, String)>, std::io::Error> {
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
            files.push((name, std::fs::read_to_string(&path)?));
        }
    }
    files.sort();
    Ok(files)
}

const GUARD_FILE: &str = "mint_ratchet_guard_tests.rs";

const fn is_ident_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn names_identifier(text: &str, ident: &str) -> bool {
    let bytes = text.as_bytes();
    text.match_indices(ident).any(|(start, _)| {
        let end = start + ident.len();
        let before = start.checked_sub(1).and_then(|i| bytes.get(i).copied());
        let after = bytes.get(end).copied();
        !before.is_some_and(is_ident_char) && !after.is_some_and(is_ident_char)
    })
}

/// Every `pub` function signature (from `pub` to the opening brace) in
/// non-test source.
fn pub_fn_signatures() -> Result<Vec<(String, String)>, std::io::Error> {
    let mut signatures = Vec::new();
    for (name, contents) in source_files()? {
        if name.ends_with("tests.rs") {
            continue;
        }
        let mut rest = contents.as_str();
        let marker = ["pub", " fn "].concat();
        let const_marker = ["pub", " const fn "].concat();
        while let Some(start) = rest
            .find(&marker)
            .into_iter()
            .chain(rest.find(&const_marker))
            .min()
        {
            let tail = &rest[start..];
            let Some(brace) = tail.find('{') else { break };
            signatures.push((name.clone(), tail[..brace].to_string()));
            rest = &tail[brace..];
        }
    }
    Ok(signatures)
}

#[test]
fn no_public_function_returns_a_v1_envelope() -> Result<(), std::io::Error> {
    let v1_envelope = ["Receipt", "Envelope"].concat();
    let offenders: Vec<String> = pub_fn_signatures()?
        .into_iter()
        .filter(|(_, signature)| {
            signature
                .split("->")
                .nth(1)
                .is_some_and(|ret| names_identifier(ret, &v1_envelope))
        })
        .map(|(name, signature)| format!("{name}: {}", signature.trim()))
        .collect();
    assert!(
        offenders.is_empty(),
        "a public function returns the V1 envelope type (MintDenied, D4): {offenders:?}"
    );
    Ok(())
}

#[test]
fn v1_mint_symbols_and_containers_are_absent() -> Result<(), std::io::Error> {
    let needles = [
        ["fn seal_", "proposal("].concat(),
        ["fn admit_", "proposal("].concat(),
        ["struct Sealed", "AgentProposal"].concat(),
        ["struct Sealed", "AdmittedProposal "].concat(),
        ["struct Sealed", "AdmittedProposal{"].concat(),
    ];
    for (name, contents) in source_files()? {
        if name == GUARD_FILE {
            continue;
        }
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
fn exactly_the_verify_only_rebuild_writes_the_v1_event_hash_key() -> Result<(), std::io::Error> {
    let key_write = ["\"event_", "hash\":"].concat();
    let mut writers = Vec::new();
    for (name, contents) in source_files()? {
        if name == GUARD_FILE {
            continue;
        }
        let count = contents.matches(key_write.as_str()).count();
        if count > 0 {
            writers.push((name, count));
        }
    }
    assert_eq!(
        writers,
        vec![("lib.rs".to_string(), 1)],
        "the V1 event_hash key may be written only once, by the verify-only rebuild"
    );
    Ok(())
}
