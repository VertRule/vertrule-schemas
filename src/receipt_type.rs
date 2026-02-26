//! Receipt type enumeration — schema discriminator for receipt classification.

use serde::{Deserialize, Serialize};

/// Classification of receipt origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
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
