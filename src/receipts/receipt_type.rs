//! Receipt type enumeration — schema discriminator for receipt classification.

use serde::{Deserialize, Deserializer, Serialize};

/// Classification of receipt origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

impl<'de> Deserialize<'de> for ReceiptType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.to_ascii_lowercase().as_str() {
            "event" => Ok(Self::Event),
            "llm" => Ok(Self::Llm),
            "mri" => Ok(Self::Mri),
            "governance" => Ok(Self::Governance),
            "adapter" => Ok(Self::Adapter),
            "projection" => Ok(Self::Projection),
            "training" => Ok(Self::Training),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &[
                    "event",
                    "llm",
                    "mri",
                    "governance",
                    "adapter",
                    "projection",
                    "training",
                ],
            )),
        }
    }
}

#[cfg(test)]
#[path = "receipt_type_tests.rs"]
mod receipt_type_tests;
