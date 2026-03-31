//! Batch-aware MRI invariant payload for receipt envelopes.
//!
//! All float-valued invariants are encoded as `u32` IEEE-754 bit patterns
//! (the `F32Bits` convention) to pass the [`CanonicalPayload`] float guard.

use serde::{Deserialize, Serialize};

use super::reduction::ReductionProvenance;
use crate::SchemaId;

/// Batch-aware MRI invariant payload.
///
/// Extends the scalar-per-layer MRI format with optional per-example
/// vector fields and explicit reduction provenance. Designed to be
/// serialized into a [`CanonicalPayload`] inside a [`ReceiptEnvelope`].
///
/// Float values use `u32` IEEE-754 bit patterns (the `F32Bits` convention)
/// to satisfy the canonical payload float guard.
///
/// **Producer obligation**: per-example vector fields (`*_per_example`)
/// and `degenerate_mask` should have length equal to `batch_len` when
/// present. This type does not enforce that invariant at construction;
/// enforcement is a verifier responsibility. The `degenerate_mask`
/// records which (layer, example) pairs were excluded from loss
/// computation.
///
/// [`CanonicalPayload`]: crate::CanonicalPayload
/// [`ReceiptEnvelope`]: crate::ReceiptEnvelope
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct MriBatchPayload {
    /// Schema identifier (e.g., `"vr.mri.batch_invariant@0.1"`).
    pub schema: SchemaId,
    /// Layer index (0-based).
    pub layer: u32,

    // ── Scalar summaries (F32Bits) ──────────────────────────────────
    /// Scalar summary of Q (tension ratio), encoded as `F32Bits`.
    pub q_scalar: u32,
    /// Scalar summary of E (expansion energy), encoded as `F32Bits`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e_scalar: Option<u32>,
    /// Scalar summary of H (attention entropy), encoded as `F32Bits`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h_scalar: Option<u32>,
    /// Scalar summary of C (compression increment), encoded as `F32Bits`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_scalar: Option<u32>,

    /// How the scalars were produced.
    pub provenance: ReductionProvenance,

    // ── Batch metadata ─────────────────────────────────────────────
    /// Batch length (number of examples). Required when any
    /// per-example vector field is present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_len: Option<u32>,

    // ── Per-example vectors (F32Bits, length should equal batch_len) ──
    /// Optional per-example Q values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q_per_example: Option<Vec<u32>>,
    /// Optional per-example E (expansion energy) values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e_per_example: Option<Vec<u32>>,
    /// Optional per-example H (attention entropy) values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h_per_example: Option<Vec<u32>>,
    /// Optional per-example C (compression increment) values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_per_example: Option<Vec<u32>>,

    // ── Degeneracy mask ────────────────────────────────────────────
    /// Per-example degeneracy flags (1 = degenerate / excluded from
    /// loss computation, 0 = normal). Length should equal `batch_len`
    /// (producer obligation, not enforced by this type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degenerate_mask: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mri::{BatchReduction, ReductionAxis, ReductionMode, TokenReduction};
    use crate::CanonicalPayload;

    fn sample_provenance() -> ReductionProvenance {
        ReductionProvenance {
            reduction_mode: ReductionMode::PerExampleThenMean,
            reduced_axes: vec![
                ReductionAxis::Token,
                ReductionAxis::Hidden,
                ReductionAxis::Batch,
            ],
            token_reduction: TokenReduction::Mean,
            batch_reduction: BatchReduction::Mean,
        }
    }

    fn scalar_only() -> Result<MriBatchPayload, anyhow::Error> {
        Ok(MriBatchPayload {
            schema: SchemaId::new("vr.mri.batch_invariant@0.1".to_string())?,
            layer: 0,
            q_scalar: 0x3F80_0000,
            e_scalar: None,
            h_scalar: None,
            c_scalar: None,
            provenance: sample_provenance(),
            batch_len: None,
            q_per_example: None,
            e_per_example: None,
            h_per_example: None,
            c_per_example: None,
            degenerate_mask: None,
        })
    }

    fn mri_schema() -> Result<SchemaId, anyhow::Error> {
        Ok(SchemaId::new("vr.mri.batch_invariant@0.1".to_string())?)
    }

    #[test]
    fn scalar_only_payload_passes_float_guard() -> Result<(), anyhow::Error> {
        let payload = scalar_only()?;
        let value = serde_json::to_value(&payload)?;
        let result = CanonicalPayload::new(value);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn full_vector_payload_passes_float_guard() -> Result<(), anyhow::Error> {
        let payload = MriBatchPayload {
            schema: mri_schema()?,
            layer: 3,
            q_scalar: 0x3F80_0000,
            e_scalar: Some(0x4000_0000),
            h_scalar: Some(0x4040_0000),
            c_scalar: Some(0x4080_0000),
            provenance: sample_provenance(),
            batch_len: Some(2),
            q_per_example: Some(vec![0x3F80_0000, 0x4000_0000]),
            e_per_example: Some(vec![0x4040_0000, 0x4080_0000]),
            h_per_example: Some(vec![0x40A0_0000, 0x40C0_0000]),
            c_per_example: Some(vec![0x40E0_0000, 0x4100_0000]),
            degenerate_mask: Some(vec![0, 1]),
        };
        let value = serde_json::to_value(&payload)?;
        let result = CanonicalPayload::new(value);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn full_payload_roundtrips_through_json() -> Result<(), anyhow::Error> {
        let payload = MriBatchPayload {
            schema: mri_schema()?,
            layer: 5,
            q_scalar: 0x4120_0000,
            e_scalar: Some(0x4140_0000),
            h_scalar: Some(0x4160_0000),
            c_scalar: Some(0x4180_0000),
            provenance: sample_provenance(),
            batch_len: Some(2),
            q_per_example: Some(vec![0x3F80_0000, 0x4000_0000]),
            e_per_example: Some(vec![0x4040_0000, 0x4080_0000]),
            h_per_example: None,
            c_per_example: None,
            degenerate_mask: Some(vec![0, 0]),
        };
        let json = serde_json::to_string(&payload)?;
        let parsed: MriBatchPayload = serde_json::from_str(&json)?;
        assert_eq!(payload, parsed);
        Ok(())
    }

    #[test]
    fn absent_fields_omitted_from_json() -> Result<(), anyhow::Error> {
        let payload = scalar_only()?;
        let json = serde_json::to_string(&payload)?;
        assert!(!json.contains("batch_len"));
        assert!(!json.contains("q_per_example"));
        assert!(!json.contains("e_scalar"));
        assert!(!json.contains("h_scalar"));
        assert!(!json.contains("c_scalar"));
        assert!(!json.contains("e_per_example"));
        assert!(!json.contains("h_per_example"));
        assert!(!json.contains("c_per_example"));
        assert!(!json.contains("degenerate_mask"));
        Ok(())
    }
}
