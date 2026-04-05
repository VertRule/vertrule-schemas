//! Determinism-safe payload wrapper for receipt envelopes.
//!
//! Wraps `serde_json::Value` with a construction guard that rejects
//! values that the receipt trust surface does not admit: floating-point
//! numbers, integers outside the interoperable I-JSON range, and strings
//! containing forbidden noncharacters.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::jcs::{deserialize_json_value_no_duplicates, is_safe_integer, validate_string_contents};
use crate::DefinitionError;

/// A JSON value guaranteed free of floating-point numbers.
///
/// Construction rejects any `serde_json::Value` tree containing
/// `Value::Number` entries with fractional parts. Integer numbers are
/// permitted only when they fit within the interoperable I-JSON integer
/// range `[-(2^53)+1, (2^53)-1]`.
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
    /// Returns [`DefinitionError::InvalidPayload`] if the tree contains
    /// floats, integers outside the I-JSON range, or forbidden noncharacters.
    pub fn new(value: serde_json::Value) -> Result<Self, DefinitionError> {
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
        let value = deserialize_json_value_no_duplicates(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Recursively reject values outside the payload determinism contract.
///
/// Path strings are only constructed on the error path. The `path`
/// parameter is passed through for leaf errors but child paths are
/// built lazily — only when a recursive call actually fails.
fn reject_floats(value: &serde_json::Value, path: &str) -> Result<(), DefinitionError> {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if !is_safe_integer(i) {
                    let display_path = if path.is_empty() { "<root>" } else { path };
                    return Err(DefinitionError::InvalidPayload(format!(
                        "integer at {display_path}: {i} exceeds the interoperable I-JSON range"
                    )));
                }
                return Ok(());
            }

            if let Some(u) = n.as_u64() {
                if u > crate::IJsonUInt::MAX {
                    let display_path = if path.is_empty() { "<root>" } else { path };
                    return Err(DefinitionError::InvalidPayload(format!(
                        "integer at {display_path}: {u} exceeds the interoperable I-JSON range"
                    )));
                }
                return Ok(());
            }

            if n.is_f64() {
                let display_path = if path.is_empty() { "<root>" } else { path };
                return Err(DefinitionError::InvalidPayload(format!(
                    "float at {display_path}: {n} — floats are nondeterministic and forbidden in receipt payloads"
                )));
            }

            // With `arbitrary_precision`, a number token can be neither
            // i64, u64, nor finite f64 (e.g. `1e400`). Reject it.
            let display_path = if path.is_empty() { "<root>" } else { path };
            Err(DefinitionError::InvalidPayload(format!(
                "number at {display_path}: {n} is not representable as a safe integer"
            )))
        }
        serde_json::Value::String(s) => {
            let display_path = if path.is_empty() { "<root>" } else { path };
            validate_string_contents(s, display_path).map_err(DefinitionError::InvalidPayload)
        }
        serde_json::Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                if reject_floats(item, path).is_err() {
                    let child_path = format!("{path}[{i}]");
                    reject_floats(item, &child_path)?;
                }
            }
            Ok(())
        }
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                validate_string_contents(key, "object property name")
                    .map_err(DefinitionError::InvalidPayload)?;
                if reject_floats(val, path).is_err() {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    reject_floats(val, &child_path)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "canonical_payload_tests.rs"]
mod tests;
