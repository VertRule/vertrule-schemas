//! `IJsonUInt` — non-negative integers that round-trip exactly in I-JSON.

use serde::{Deserialize, Deserializer, Serialize};

use crate::DefinitionError;

/// A non-negative integer guaranteed to fit in the interoperable I-JSON range.
///
/// This type rejects values larger than `2^53 - 1`, which is the largest
/// integer that all IEEE 754 double-precision implementations can round-trip
/// exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct IJsonUInt(u64);

impl IJsonUInt {
    /// Largest interoperable integer permitted by I-JSON.
    pub const MAX: u64 = 9_007_199_254_740_991;

    /// Create a validated [`IJsonUInt`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidIJsonNumber`] if `value` exceeds
    /// [`Self::MAX`].
    pub fn new(value: u64) -> Result<Self, DefinitionError> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(DefinitionError::InvalidIJsonNumber(format!(
                "expected an integer in the inclusive range 0..={}, got {}",
                Self::MAX,
                value
            )))
        }
    }

    /// The inner `u64`.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for IJsonUInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl From<IJsonUInt> for u64 {
    fn from(value: IJsonUInt) -> Self {
        value.0
    }
}

impl std::fmt::Display for IJsonUInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[path = "i_json_uint_tests.rs"]
mod i_json_uint_tests;
