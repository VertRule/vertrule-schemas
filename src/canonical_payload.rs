//! Determinism-safe payload wrapper for receipt envelopes.
//!
//! Wraps `serde_json::Value` with a construction guard that rejects
//! floating-point numbers at any nesting depth. Floats are nondeterministic
//! across platforms and serialization libraries — they must not enter
//! the receipt spine.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A JSON value guaranteed free of floating-point numbers.
///
/// Construction rejects any `serde_json::Value` tree containing
/// `Value::Number` entries with fractional parts. Integer numbers
/// are permitted.
///
/// This closes the determinism gap at the type boundary: if a
/// `CanonicalPayload` exists, its contents are safe for canonical
/// serialization and digest computation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPayload(serde_json::Value);

impl CanonicalPayload {
    /// Wrap a JSON value, rejecting any floats in the tree.
    ///
    /// # Errors
    ///
    /// Returns a description of the first float found.
    pub fn new(value: serde_json::Value) -> Result<Self, String> {
        reject_floats(&value, "")?;
        Ok(Self(value))
    }

    /// Access the inner JSON value.
    #[must_use]
    pub const fn as_value(&self) -> &serde_json::Value {
        &self.0
    }

    /// Consume the wrapper and return the inner value.
    #[must_use]
    pub fn into_value(self) -> serde_json::Value {
        self.0
    }
}

impl Serialize for CanonicalPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CanonicalPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Recursively reject floating-point numbers in a JSON value tree.
fn reject_floats(value: &serde_json::Value, path: &str) -> Result<(), String> {
    match value {
        serde_json::Value::Number(n) => {
            if n.is_f64() && !n.is_i64() && !n.is_u64() {
                let display_path = if path.is_empty() { "<root>" } else { path };
                return Err(format!(
                    "float at {display_path}: {n} — floats are nondeterministic and forbidden in receipt payloads"
                ));
            }
            Ok(())
        }
        serde_json::Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                let child_path = format!("{path}[{i}]");
                reject_floats(item, &child_path)?;
            }
            Ok(())
        }
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                reject_floats(val, &child_path)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "canonical_payload_tests.rs"]
mod tests;
