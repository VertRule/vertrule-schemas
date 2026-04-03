//! Boundary origin enumeration — schema discriminator for boundary provenance.

use serde::{Deserialize, Serialize};

/// Which boundary produced the receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl std::fmt::Display for BoundaryOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Engine => f.write_str("engine"),
            Self::Adapter => f.write_str("adapter"),
            Self::Numeric => f.write_str("numeric"),
            Self::Governance => f.write_str("governance"),
            Self::Model => f.write_str("model"),
            Self::Training => f.write_str("training"),
        }
    }
}
