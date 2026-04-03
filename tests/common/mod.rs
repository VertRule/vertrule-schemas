//! Shared test helpers for vertrule-schemas integration tests.

// Included as `mod common` from multiple test binaries — not all
// functions are used in every binary.
#![allow(dead_code)]

/// Convert an `Option` to a `Result` with a static error message.
pub(crate) fn need<T>(option: Option<T>, what: &'static str) -> anyhow::Result<T> {
    option.ok_or_else(|| anyhow::anyhow!(what))
}

/// Load a test vector file by name (without `.json` extension).
pub(crate) fn load_vector(name: &str) -> anyhow::Result<serde_json::Value> {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("test-vectors").join(format!("{name}.json"));
    let bytes = std::fs::read(&path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    Ok(value)
}

/// Assert that a rejection test produces an error whose Display output
/// contains the expected substring. This is R2/R3 strength.
/// For R1 strength, callers should match the concrete error variant directly.
pub(crate) fn assert_error_contains<T, E: std::fmt::Display>(
    result: Result<T, E>,
    expected_substring: &str,
    case_id: &str,
) -> anyhow::Result<()> {
    match result {
        Ok(_) => Err(anyhow::anyhow!(
            "[{case_id}] expected rejection, got success"
        )),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains(expected_substring) {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "[{case_id}] error message {msg:?} does not contain {expected_substring:?}"
                ))
            }
        }
    }
}

/// Assert that a function produces identical bytes across N invocations.
pub(crate) fn assert_deterministic<F>(f: F, n: usize, case_id: &str) -> anyhow::Result<()>
where
    F: Fn() -> anyhow::Result<Vec<u8>>,
{
    let first = f()?;
    for i in 1..n {
        let next = f()?;
        if first != next {
            return Err(anyhow::anyhow!(
                "[{case_id}] invocation {i} produced different bytes (first: {} bytes, got: {} bytes)",
                first.len(),
                next.len()
            ));
        }
    }
    Ok(())
}
