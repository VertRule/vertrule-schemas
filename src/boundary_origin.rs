// TODO(phase2): Migrate to `receipts/` once the adapter shape is defined.
// Deferred because `BoundaryOrigin` is semantically scoped to adapter-boundary
// receipts and placing it without its natural sibling (`receipts/adapter.rs`)
// would be premature classification.

//! Boundary origin enumeration — schema discriminator for boundary provenance.

use serde::{Deserialize, Deserializer, Serialize};

/// Which boundary produced the receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum BoundaryOrigin {
    /// Engine-internal receipt.
    Engine,
    /// Adapter boundary receipt.
    Adapter,
    /// Numeric boundary receipt.
    Numeric,
    /// Governance boundary receipt.
    Governance,
    /// Model boundary receipt.
    Model,
    /// Training boundary receipt.
    Training,
}

impl<'de> Deserialize<'de> for BoundaryOrigin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.to_ascii_lowercase().as_str() {
            "engine" => Ok(Self::Engine),
            "adapter" => Ok(Self::Adapter),
            "numeric" => Ok(Self::Numeric),
            "governance" => Ok(Self::Governance),
            "model" => Ok(Self::Model),
            "training" => Ok(Self::Training),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &[
                    "engine",
                    "adapter",
                    "numeric",
                    "governance",
                    "model",
                    "training",
                ],
            )),
        }
    }
}
