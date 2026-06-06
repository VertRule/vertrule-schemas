//! Training-transition receipt schema carrier.
//!
//! `TrainingReceipt` is a **wire/schema carrier** for training-receipt data. It
//! is not a minted receipt, not a verified receipt, and not proof that training
//! occurred — it merely describes the declared digests and counters a training
//! transition reports. Minting, verification, and policy evaluation live in
//! producer/verifier crates; this crate performs none of them and imports no
//! runtime-internal crate.

use serde::{Deserialize, Serialize};

use crate::{DigestBytes, SchemaReceiptId};

/// Receipt carrier for a governed training transition.
///
/// All fields are receipt-safe (no floats, canonical ordering). The three
/// `u64` counters are encoded as canonical decimal strings
/// (`VR-CANONICAL-U64-STRING-POLICY-V1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct TrainingReceipt {
    /// Unique receipt identifier.
    pub receipt_id: SchemaReceiptId,
    /// BLAKE3 digest of model weights before this training step.
    pub pre_weights_digest: DigestBytes,
    /// BLAKE3 digest of model weights after this training step.
    pub post_weights_digest: DigestBytes,
    /// BLAKE3 digest of the training data batch.
    pub batch_digest: DigestBytes,
    /// Sealed execution context digest.
    pub context_digest: DigestBytes,
    /// Schema digest for this receipt version.
    pub schema_digest: DigestBytes,
    /// BLAKE3 digest of the governing policy pack.
    pub policy_digest: DigestBytes,
    /// Logical time from the injected clock.
    #[serde(with = "canonical_u64")]
    pub logical_time: u64,
    /// Training step index (monotonic within a run).
    #[serde(with = "canonical_u64")]
    pub step_index: u64,
    /// Number of samples in the batch.
    #[serde(with = "canonical_u64")]
    pub batch_size: u64,
    /// Whether this training step is reversible.
    pub reversible: bool,
    /// BLAKE3 digest of optimizer state (for reversibility proof).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optimizer_state_digest: Option<DigestBytes>,
    /// Optional parent receipt digest for chain linkage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<DigestBytes>,
}

impl TrainingReceipt {
    /// Construct a training receipt; optional digests default to `None` and may
    /// be set on the returned value.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        receipt_id: SchemaReceiptId,
        pre_weights_digest: DigestBytes,
        post_weights_digest: DigestBytes,
        batch_digest: DigestBytes,
        context_digest: DigestBytes,
        schema_digest: DigestBytes,
        policy_digest: DigestBytes,
        logical_time: u64,
        step_index: u64,
        batch_size: u64,
        reversible: bool,
    ) -> Self {
        Self {
            receipt_id,
            pre_weights_digest,
            post_weights_digest,
            batch_digest,
            context_digest,
            schema_digest,
            policy_digest,
            logical_time,
            step_index,
            batch_size,
            reversible,
            optimizer_state_digest: None,
            parent_id: None,
        }
    }
}

/// Canonical decimal-string serde for the digest-critical `u64` counters
/// (`VR-CANONICAL-U64-STRING-POLICY-V1`).
///
/// Mirrors the byte representation of `vr_identity::canonical_u64_serde`, kept
/// local so this published crate stays free of runtime-internal dependencies.
mod canonical_u64 {
    use serde::de::{self, Visitor};

    // serde's `serialize_with` mandates the `&T` receiver shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(super) fn serialize<S: serde::Serializer>(
        value: &u64,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<u64, D::Error> {
        struct CanonicalU64Visitor;

        impl Visitor<'_> for CanonicalU64Visitor {
            type Value = u64;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a u64 as a decimal string or number")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
                Ok(v)
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<u64, E> {
                u64::try_from(v).map_err(|_| E::custom("negative integers are not valid u64"))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
                v.parse::<u64>().map_err(E::custom)
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<u64, E> {
                self.visit_str(&v)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_any(CanonicalU64Visitor)
        } else {
            deserializer.deserialize_str(CanonicalU64Visitor)
        }
    }
}

#[cfg(test)]
#[path = "training_receipt_tests.rs"]
mod training_receipt_tests;
