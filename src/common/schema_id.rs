//! Validated schema identifier with grammar enforcement.
//!
//! Schema identifiers follow the grammar `vr.<domain>.<name>@<major>.<minor>`.
//! This type validates that structure at construction time without
//! maintaining a registry of known identifiers — it enforces shape,
//! not vocabulary.

use serde::{Deserialize, Deserializer, Serialize};

use crate::DefinitionError;

/// A validated schema identifier.
///
/// The admitted grammar is:
///
/// ```text
/// vr.<domain>.<name>@<major>.<minor>
/// ```
///
/// Where:
/// - `domain` matches `[a-z0-9][a-z0-9_-]*` (namespace for the producing system)
/// - `name` matches `[a-z0-9][a-z0-9_-]*` (receipt kind within the namespace)
/// - `major` and `minor` are unsigned decimal integers
///
/// This is a shape type: it validates identifier structure without
/// binding to any particular domain vocabulary. The constitutional
/// layer enforces the grammar; each adapter owns its own constants.
///
/// # Examples
///
/// ```
/// use vertrule_schemas::SchemaId;
///
/// # fn main() -> Result<(), vertrule_schemas::DefinitionError> {
/// let id = SchemaId::new("vr.openclaw.ingress@0.1".to_string())?;
/// assert_eq!(id.domain(), "openclaw");
/// assert_eq!(id.name(), "ingress");
/// assert_eq!(id.version(), "0.1");
/// assert_eq!(id.as_str(), "vr.openclaw.ingress@0.1");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SchemaId(String);

impl SchemaId {
    /// Maximum permitted length of a schema identifier.
    pub const MAX_LEN: usize = 128;

    /// Create a validated [`SchemaId`] from a string.
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidSchemaId`] if the string does
    /// not match the `vr.<domain>.<name>@<major>.<minor>` grammar.
    pub fn new(id: String) -> Result<Self, DefinitionError> {
        validate_schema_id(&id)?;
        Ok(Self(id))
    }

    /// Return the full identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the domain segment (e.g., `"openclaw"` from `"vr.openclaw.ingress@0.1"`).
    #[must_use]
    pub fn domain(&self) -> &str {
        // Safety: validated at construction — "vr." prefix and first "." after it are guaranteed.
        let result = self
            .0
            .strip_prefix("vr.")
            .and_then(|rest| rest.find('.').map(|pos| &rest[..pos]))
            .unwrap_or("");
        debug_assert!(!result.is_empty());
        result
    }

    /// Return the name segment (e.g., `"ingress"` from `"vr.openclaw.ingress@0.1"`).
    #[must_use]
    pub fn name(&self) -> &str {
        // Safety: validated at construction — second "." and "@" are guaranteed.
        let result = self
            .0
            .strip_prefix("vr.")
            .and_then(|rest| {
                let dot = rest.find('.')?;
                let after_domain = &rest[dot + 1..];
                let at = after_domain.find('@')?;
                Some(&after_domain[..at])
            })
            .unwrap_or("");
        debug_assert!(!result.is_empty());
        result
    }

    /// Return the version string (e.g., `"0.1"` from `"vr.openclaw.ingress@0.1"`).
    #[must_use]
    pub fn version(&self) -> &str {
        // Safety: validated at construction — "@" separator is guaranteed.
        let result = self.0.find('@').map_or("", |pos| &self.0[pos + 1..]);
        debug_assert!(!result.is_empty());
        result
    }
}

impl<'de> Deserialize<'de> for SchemaId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        Self::new(id).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for SchemaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Validate a schema identifier against the constitutional grammar.
///
/// Grammar: `vr.<domain>.<name>@<major>.<minor>`
fn validate_schema_id(id: &str) -> Result<(), DefinitionError> {
    let err = |msg: String| DefinitionError::InvalidSchemaId(msg);

    if id.len() > SchemaId::MAX_LEN {
        return Err(err(format!("exceeds max length of {}", SchemaId::MAX_LEN)));
    }

    // Split prefix
    let rest = id
        .strip_prefix("vr.")
        .ok_or_else(|| err("must start with \"vr.\"".to_string()))?;

    // Split at "@" for version
    let at_pos = rest
        .find('@')
        .ok_or_else(|| err("missing \"@\" version separator".to_string()))?;

    let path = &rest[..at_pos];
    let version = &rest[at_pos + 1..];

    // Path must have exactly one "." separating domain and name
    let dot_pos = path
        .find('.')
        .ok_or_else(|| err("missing \".\" between domain and name".to_string()))?;

    let domain = &path[..dot_pos];
    let name = &path[dot_pos + 1..];

    // No additional dots in domain or name
    if name.contains('.') {
        return Err(err(
            "name segment must not contain \".\" (expected vr.<domain>.<name>@<version>)"
                .to_string(),
        ));
    }

    // Validate domain segment
    validate_segment(domain, "domain")?;

    // Validate name segment
    validate_segment(name, "name")?;

    // Validate version: <major>.<minor>
    let version_dot = version
        .find('.')
        .ok_or_else(|| err("version must be <major>.<minor>".to_string()))?;

    let major = &version[..version_dot];
    let minor = &version[version_dot + 1..];

    if major.is_empty() || minor.is_empty() {
        return Err(err("version components must not be empty".to_string()));
    }

    if !major.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err(format!(
            "major version \"{major}\" is not a decimal integer"
        )));
    }

    if !minor.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err(format!(
            "minor version \"{minor}\" is not a decimal integer"
        )));
    }

    // No extra content after minor version
    if minor.contains('.') {
        return Err(err(
            "version must be exactly <major>.<minor>, not <major>.<minor>.<patch>".to_string(),
        ));
    }

    Ok(())
}

/// Validate a domain or name segment: `[a-z0-9][a-z0-9_-]*`.
fn validate_segment(segment: &str, label: &str) -> Result<(), DefinitionError> {
    let err = |msg: String| DefinitionError::InvalidSchemaId(msg);

    if segment.is_empty() {
        return Err(err(format!("{label} segment must not be empty")));
    }

    let first = segment.as_bytes()[0];
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(err(format!(
            "{label} segment must start with a lowercase letter or digit, got '{}'",
            first as char
        )));
    }

    for (i, ch) in segment.char_indices() {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-';
        if !valid {
            return Err(err(format!(
                "invalid character '{ch}' at byte index {i} in {label} segment"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "schema_id_tests.rs"]
mod tests;
