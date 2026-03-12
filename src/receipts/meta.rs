//! Constitutional receipt metadata header.

use serde::{Deserialize, Serialize};

use crate::{DigestBytes, SchemaVersion};

/// Metadata header for receipt format identification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptMetaV1 {
    /// Envelope schema version.
    pub envelope_version: SchemaVersion,

    /// Format identifier string.
    pub format_id: String,

    /// Digest of the schema used.
    pub schema_digest: DigestBytes,
}

#[cfg(test)]
mod tests {
    use super::ReceiptMetaV1;
    use crate::{DigestBytes, SchemaVersion};

    #[test]
    fn serde_round_trip() -> Result<(), serde_json::Error> {
        let meta = ReceiptMetaV1 {
            envelope_version: SchemaVersion::V1,
            format_id: "receipt-meta-v1".to_string(),
            schema_digest: DigestBytes::from_array([0x11; 32]),
        };

        let json = serde_json::to_string(&meta)?;
        let parsed: ReceiptMetaV1 = serde_json::from_str(&json)?;

        assert_eq!(parsed, meta);
        Ok(())
    }
}
