//! Receipt type enumeration — wire-compatible with the runtime.

use serde::{Deserialize, Serialize};

/// Classification of receipt origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReceiptType {
    /// Domain event receipt.
    Event,
    /// LLM interaction receipt.
    Llm,
    /// MRI instrumentation receipt.
    Mri,
    /// Governance action receipt.
    Governance,
    /// Adapter boundary receipt.
    Adapter,
    /// Projection receipt.
    Projection,
    /// Training receipt.
    Training,
}
