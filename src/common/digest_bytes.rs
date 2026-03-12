//! `DigestBytes` newtype — a 32-byte cryptographic digest with strict hex serde.

use std::fmt;

use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::common::algorithms::{DIGEST_BYTE_LEN, DIGEST_HEX_LEN};
use crate::error::DefinitionError;

/// A 32-byte cryptographic digest with strict hex serialization.
///
/// This is a shape type: it enforces the 32-byte / 64-hex-char wire
/// format without binding to a specific commitment primitive. The
/// active commitment primitive is declared by the specification version.
///
/// Serializes as 64 lowercase hex characters.
/// Deserialization rejects uppercase, non-canonical length, whitespace,
/// and non-hex characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DigestBytes([u8; 32]);

impl DigestBytes {
    /// Create from a raw byte array (infallible).
    #[must_use]
    pub const fn from_array(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Parse from a 64-character lowercase hex string.
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidDigest`] if the string is not exactly
    /// 64 lowercase hex characters.
    pub fn from_hex(hex: &str) -> Result<Self, DefinitionError> {
        validate_hex(hex)?;
        let mut buf = [0u8; 32];
        hex::decode_to_slice(hex, &mut buf)
            .map_err(|e| DefinitionError::InvalidDigest(format!("hex decode failed: {e}")))?;
        Ok(Self(buf))
    }

    /// Create from a byte slice (must be exactly 32 bytes).
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError::InvalidDigest`] if the slice length is not 32.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DefinitionError> {
        let arr: [u8; 32] = bytes.try_into().map_err(|_| {
            DefinitionError::InvalidDigest(format!(
                "expected {} bytes, got {}",
                DIGEST_BYTE_LEN,
                bytes.len()
            ))
        })?;
        Ok(Self(arr))
    }

    /// Return a reference to the inner 32 bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Encode as a 64-character lowercase hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for DigestBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl Serialize for DigestBytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for DigestBytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(de::Error::custom)
    }
}

/// Validate that a hex string is exactly 64 lowercase hex characters.
fn validate_hex(hex: &str) -> Result<(), DefinitionError> {
    if hex.len() != DIGEST_HEX_LEN {
        return Err(DefinitionError::InvalidDigest(format!(
            "expected {DIGEST_HEX_LEN} hex chars, got {}",
            hex.len()
        )));
    }
    if !hex
        .bytes()
        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(DefinitionError::InvalidDigest(
            "contains non-lowercase-hex characters".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "digest_bytes_tests.rs"]
mod digest_bytes_tests;
