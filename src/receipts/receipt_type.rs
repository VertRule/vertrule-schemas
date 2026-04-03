//! Receipt type enumeration — schema discriminator for receipt classification.

use serde::{Deserialize, Serialize};

/// Classification of receipt origin.
///
/// These are top-level envelope discriminators. Within the `Governance`
/// type, payload-level subtypes (e.g. `GovernanceChange`,
/// `PolicyPackProposal`) provide finer classification — those are
/// schema-level concerns carried in the payload, not envelope-level
/// discriminators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl std::fmt::Display for ReceiptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Event => f.write_str("event"),
            Self::Llm => f.write_str("llm"),
            Self::Mri => f.write_str("mri"),
            Self::Governance => f.write_str("governance"),
            Self::Adapter => f.write_str("adapter"),
            Self::Projection => f.write_str("projection"),
            Self::Training => f.write_str("training"),
        }
    }
}

#[cfg(test)]
#[path = "receipt_type_tests.rs"]
mod receipt_type_tests;
