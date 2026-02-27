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
