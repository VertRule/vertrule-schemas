//! Reduction mode and provenance types for MRI invariant computation.
//!
//! These types are receipt-visible: they determine the meaning of
//! emitted E/C/Q values. Defined once here — no copies permitted.

use serde::{Deserialize, Serialize};

/// How the batch dimension was handled during invariant computation.
///
/// Receipt-visible: changes the interpretation of scalar summaries
/// and determines whether per-example vectors are meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReductionMode {
    /// Batch dimension collapsed first: `mean([B,T,D])` then invariant.
    BatchCollapsed,
    /// Invariant computed per-example, then batch-aggregated.
    PerExampleThenMean,
    /// Microbatch accumulation: invariants computed over microbatches
    /// and accumulated, producing equivalent results to full-batch
    /// computation under the accumulation law.
    MicrobatchEquivalent,
}

/// Tensor axis that was reduced during invariant computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReductionAxis {
    /// Batch dimension.
    Batch,
    /// Token / sequence-position dimension.
    Token,
    /// Hidden / feature dimension.
    Hidden,
    /// Attention head dimension.
    Head,
}

/// How tokens were aggregated within each example.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenReduction {
    /// Mean across all token positions.
    Mean,
    /// Value at the last token position only.
    LastToken,
    /// Value at the first token position only.
    FirstToken,
}

/// How batch elements were aggregated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchReduction {
    /// Mean across all batch elements.
    Mean,
    /// No batch aggregation (per-example values preserved).
    None,
}

/// Full provenance of how an invariant value was reduced from tensor data.
///
/// Records the exact reduction pipeline so that consumers can determine
/// whether two invariant values are comparable, and whether the scalar
/// summary was formed before or after semantic collapse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReductionProvenance {
    /// Which reduction mode was applied.
    pub reduction_mode: ReductionMode,
    /// Axes that were reduced, in reduction order.
    pub reduced_axes: Vec<ReductionAxis>,
    /// How tokens were aggregated within each example.
    pub token_reduction: TokenReduction,
    /// How batch elements were aggregated.
    pub batch_reduction: BatchReduction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduction_mode_roundtrips() -> Result<(), anyhow::Error> {
        let modes = [
            ReductionMode::BatchCollapsed,
            ReductionMode::PerExampleThenMean,
            ReductionMode::MicrobatchEquivalent,
        ];
        for mode in &modes {
            let json = serde_json::to_string(mode)?;
            let parsed: ReductionMode = serde_json::from_str(&json)?;
            assert_eq!(*mode, parsed);
        }
        Ok(())
    }

    #[test]
    fn reduction_axis_roundtrips() -> Result<(), anyhow::Error> {
        let axes = [
            ReductionAxis::Batch,
            ReductionAxis::Token,
            ReductionAxis::Hidden,
            ReductionAxis::Head,
        ];
        for axis in &axes {
            let json = serde_json::to_string(axis)?;
            let parsed: ReductionAxis = serde_json::from_str(&json)?;
            assert_eq!(*axis, parsed);
        }
        Ok(())
    }

    #[test]
    fn provenance_canonical_json() -> Result<(), anyhow::Error> {
        let prov = ReductionProvenance {
            reduction_mode: ReductionMode::PerExampleThenMean,
            reduced_axes: vec![
                ReductionAxis::Token,
                ReductionAxis::Hidden,
                ReductionAxis::Batch,
            ],
            token_reduction: TokenReduction::Mean,
            batch_reduction: BatchReduction::Mean,
        };
        let json1 = serde_json::to_string(&prov)?;
        let json2 = serde_json::to_string(&prov)?;
        assert_eq!(json1, json2);
        Ok(())
    }

    #[test]
    fn unknown_reduction_mode_is_parse_failure() {
        let bad = r#""invented_mode""#;
        let result: Result<ReductionMode, _> = serde_json::from_str(bad);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_axis_is_parse_failure() {
        let bad = r#""spatial""#;
        let result: Result<ReductionAxis, _> = serde_json::from_str(bad);
        assert!(result.is_err());
    }
}
