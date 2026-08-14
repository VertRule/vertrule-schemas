//! `vr.policy.input@0.1` — the evaluation input document.
//!
//! Carries the operation date and the claim evidence the gates
//! inspect. All values are exact types (integers, ISO-8601 date
//! strings, booleans) — no floats, no wall-clock, no host lookup.
//!
//! ## Passive carrier
//!
//! This type is data plus serialization. It holds no engine handle, no
//! runtime capability and no policy-execution method, which is what lets
//! the pre-policy resolver consume it without acquiring a dependency path
//! to policy execution (gremlin#183, gremlin#193). It relocated here from
//! `vertrule-policy-wasm` so that consuming the carrier no longer drags in
//! `WasmPolicyEngine` (vertrule-schemas#3).
//!
//! ## Byte law — do not "simplify" the canonicalization
//!
//! [`EvaluationInput::to_canonical_bytes`] is the `claim_preimage` that the
//! `EvaluationInputDigestV1` law digests (ADR-038 Byte Law 2), pinned by a
//! golden at `e97d9550b78532c0…`. The law is **serialize to JSON bytes,
//! then canonicalize the slice**:
//!
//! ```text
//! serde_json::to_vec  →  vr_jcs::to_canon_bytes_from_slice
//! ```
//!
//! `vr_jcs::to_canon_bytes` looks equivalent and is **not**: it routes
//! through `serde_json::Value` (`to_value → to_canon_bytes_value`), and
//! this crate enables `serde_json/arbitrary_precision`, so the two paths
//! are not interchangeable by inspection. It is also deprecated. Keep the
//! slice form.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Evaluation-input format identifier.
///
/// Schema-owned rather than ABI-owned: changing it changes the carrier's
/// encoding version, which is the deciding predicate
/// (`Δ INPUT_FORMAT ⇒ Δ EvaluationInputEncoding`). It previously sat in
/// `vertrule_policy_wasm::abi` beside genuinely ABI-owned constants;
/// co-location was not ownership.
pub const INPUT_FORMAT: &str = "vr.policy.input@0.1";

/// One evaluation input: operation date + claim evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationInput {
    /// Must equal [`INPUT_FORMAT`].
    pub input_format: String,
    /// ISO-8601 date of the governed operation (deterministic string
    /// compare against dated-document floors; never a wall-clock read).
    pub operation_date: String,
    /// Claim evidence keyed by claim key (`BTreeMap`: deterministic
    /// order by construction).
    pub claims: BTreeMap<String, ClaimEvidence>,
}

/// Evidence attached to one claim key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ClaimEvidence {
    /// Integer value (numeric-limit gates).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<u64>,
    /// ISO-8601 valid-through date (dated-document gates).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_through: Option<String>,
    /// Public URL (public-link gates).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Whether the claim cites a source (require-source gates).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cited: Option<bool>,
}

impl EvaluationInput {
    /// Canonical `JCS(EvaluationInput)` bytes — the `claim_preimage` the
    /// `EvaluationInputDigestV1` law digests (ADR-038 Byte Law 2).
    ///
    /// Ungated here, unlike at its previous home: canonicalization now
    /// goes through `vr-jcs`, which this crate already depends on, so the
    /// carrier is self-contained at the schemas layer and does not pull
    /// the kernel upward.
    ///
    /// See the module byte-law note before changing the call sequence.
    ///
    /// # Errors
    /// Returns [`InputCanonicalizationError`] if JCS canonicalization fails.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, InputCanonicalizationError> {
        let json_bytes =
            serde_json::to_vec(self).map_err(|e| InputCanonicalizationError(e.to_string()))?;
        vr_jcs::to_canon_bytes_from_slice(&json_bytes)
            .map_err(|e| InputCanonicalizationError(e.to_string()))
    }

    /// Parse canonical evaluation-input bytes into a typed input, rejecting
    /// malformed or unknown-field documents deterministically.
    ///
    /// # Errors
    /// Returns [`InputCanonicalizationError`] if `bytes` is not a valid
    /// `EvaluationInput` document.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, InputCanonicalizationError> {
        serde_json::from_slice(bytes).map_err(|e| InputCanonicalizationError(e.to_string()))
    }
}

/// Failure canonicalizing or parsing an [`EvaluationInput`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("evaluation input canonicalization failed: {0}")]
pub struct InputCanonicalizationError(String);

#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;
