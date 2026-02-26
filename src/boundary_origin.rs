//! Boundary origin enumeration — wire-compatible with the runtime.

use serde::{Deserialize, Serialize};

/// Which boundary produced the receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
