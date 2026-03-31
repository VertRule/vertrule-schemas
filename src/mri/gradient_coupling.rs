//! Gradient-coupling diagnostic payload for MRI receipt envelopes.
//!
//! Records per-layer gradient norm ratios `‖∇L_Q‖ / ‖∇L_LM‖` to quantify
//! how much of the gradient signal at each layer comes from the MRI
//! invariant loss vs the language model loss.
//!
//! All float-valued fields use `u32` IEEE-754 bit patterns (`F32Bits`
//! convention) to pass the [`CanonicalPayload`] float guard.
//!
//! [`CanonicalPayload`]: crate::CanonicalPayload

use serde::{Deserialize, Serialize};

use super::reduction::ReductionProvenance;
use crate::SchemaId;

/// Per-layer gradient coupling diagnostic payload.
///
/// Produced by computing two separate backward passes (one for `L_Q`,
/// one for `L_LM`) and comparing per-layer gradient norms.
///
/// `coupling_ratios` is a magnitude-only metric: higher values mean
/// the invariant loss contributes more gradient signal at that layer.
/// It does not indicate direction (cooperating vs fighting). For
/// directional alignment, use `profile_cosine`.
///
/// `profile_cosine` is the cosine similarity between the per-layer
/// norm profiles (the two norm vectors). Positive means the gradient
/// profiles are aligned; negative means they oppose.
///
/// When either gradient profile has zero norm, `profile_cosine` is
/// defined as `0.0` (orthogonal by convention).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GradientCouplingPayload {
    /// Schema identifier (e.g., `"vr.mri.gradient_coupling@0.1"`).
    pub schema: SchemaId,

    /// Training step at which this diagnostic was computed.
    pub step: u64,

    /// Number of model layers. Should be > 0 (producer obligation).
    pub num_layers: u32,

    /// Per-layer `‖∇L_Q‖`, `F32Bits` encoded. Length should equal
    /// `num_layers` (producer obligation, not enforced by this type).
    pub grad_q_norms: Vec<u32>,

    /// Per-layer `‖∇L_LM‖`, `F32Bits` encoded. Length should equal
    /// `num_layers` (producer obligation, not enforced by this type).
    pub grad_lm_norms: Vec<u32>,

    /// Per-layer magnitude ratio `‖∇L_Q‖ / ‖∇L_LM‖`, `F32Bits` encoded.
    /// Clamped to `0.0` when denominator is below epsilon.
    /// This is magnitude-only — does not indicate direction.
    /// Length should equal `num_layers` (producer obligation, not
    /// enforced by this type).
    pub coupling_ratios: Vec<u32>,

    /// Cosine similarity between the two per-layer norm profiles.
    /// Range: `[-1.0, 1.0]`. `F32Bits` encoded.
    /// Defined as `0.0` when either profile vector has zero norm.
    pub profile_cosine: u32,

    /// Provenance of the reduction applied to produce per-layer norms.
    pub provenance: ReductionProvenance,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mri::{BatchReduction, ReductionAxis, ReductionMode, TokenReduction};
    use crate::CanonicalPayload;

    fn sample_provenance() -> ReductionProvenance {
        ReductionProvenance {
            reduction_mode: ReductionMode::BatchCollapsed,
            reduced_axes: vec![ReductionAxis::Batch],
            token_reduction: TokenReduction::Mean,
            batch_reduction: BatchReduction::Mean,
        }
    }

    fn sample_payload() -> Result<GradientCouplingPayload, anyhow::Error> {
        Ok(GradientCouplingPayload {
            schema: SchemaId::new("vr.mri.gradient_coupling@0.1".to_string())?,
            step: 100,
            num_layers: 4,
            grad_q_norms: vec![0x3C23_D70A, 0x3CA3_D70A, 0x3D23_D70A, 0x3DA3_D70A],
            grad_lm_norms: vec![0x3F80_0000, 0x3F80_0000, 0x3F80_0000, 0x3F80_0000],
            coupling_ratios: vec![0x3C23_D70A, 0x3CA3_D70A, 0x3D23_D70A, 0x3DA3_D70A],
            profile_cosine: 0x3F60_0000, // ~0.875
            provenance: sample_provenance(),
        })
    }

    #[test]
    fn passes_canonical_payload_guard() -> Result<(), anyhow::Error> {
        let payload = sample_payload()?;
        let value = serde_json::to_value(&payload)?;
        assert!(CanonicalPayload::new(value).is_ok());
        Ok(())
    }

    #[test]
    fn roundtrips_through_json() -> Result<(), anyhow::Error> {
        let payload = sample_payload()?;
        let json = serde_json::to_string(&payload)?;
        let parsed: GradientCouplingPayload = serde_json::from_str(&json)?;
        assert_eq!(payload, parsed);
        Ok(())
    }

    #[test]
    fn zero_cosine_passes_guard() -> Result<(), anyhow::Error> {
        let mut payload = sample_payload()?;
        payload.profile_cosine = 0; // 0.0f32 as bits
        let value = serde_json::to_value(&payload)?;
        assert!(CanonicalPayload::new(value).is_ok());
        Ok(())
    }
}
