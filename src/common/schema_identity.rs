//! Public schema/wire identity carrier types.
//!
//! These are **portable wire carriers** for identity-bearing fields in receipt,
//! pack, and manifest DTOs — **not** runtime authority types. The authoritative
//! identities (with stronger constructors, invariants, and conversions) live in
//! runtime-internal crates (`vr-identity`, `vertrule-crypto`). Those crates are
//! `publish = false` and must never be a dependency of this published,
//! standalone schema crate, so the wire representation is mirrored locally here.
//!
//! Carriers are deliberately dumb transport: a value round-trips through JSON
//! exactly, with no grammar validation beyond shape. Wire forms mirror the
//! runtime types byte-for-byte:
//! - text carriers: a transparent JSON string;
//! - [`SchemaRunId`]: `{"partition": <u32>, "offset": "<decimal-u64 string>"}`,
//!   accepting a legacy bare `u64` (partition defaults to `0`) on read.
//
// TODO(phase2-closure): once a published identity-crate boundary exists, provide
// `From`/`TryFrom` conversions between these carriers and the runtime identity
// types instead of duplicating the representation. Conversions belong at the
// runtime boundary, not in this schema crate.

use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Define a transparent `String`-backed schema wire carrier.
macro_rules! schema_text_carrier {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        ///
        /// Wire form: a transparent JSON string. Pure transport carrier — no
        /// grammar is enforced (see module docs).
        #[derive(
            Debug,
            Clone,
            Default,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            Serialize,
            Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[doc = concat!("Create a `", stringify!($name), "` from any string.")]
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// The carrier's string form.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

schema_text_carrier!(SchemaReceiptId, "Receipt identifier (schema wire carrier).");
schema_text_carrier!(SchemaModelId, "Model identifier (schema wire carrier).");
schema_text_carrier!(
    SchemaPolicyPackId,
    "Policy-pack identifier (schema wire carrier)."
);
schema_text_carrier!(SchemaSuiteId, "Prompt-suite identifier (schema wire carrier).");
schema_text_carrier!(
    SchemaPublicKeyHex,
    "Hex-encoded public-key identifier (schema wire carrier)."
);
schema_text_carrier!(SchemaKeyId, "Signing-key identifier (schema wire carrier).");

/// Run identifier carrier: a `(partition, offset)` pair.
///
/// Wire form mirrors the runtime `RunId` byte-for-byte:
/// `{"partition": <u32 number>, "offset": "<decimal-u64 string>"}`. The offset
/// is string-encoded for JCS/I-JSON safety (`VR-CANONICAL-U64-STRING-POLICY-V1`).
/// A legacy bare `u64` is accepted on read, mapping to partition `0`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SchemaRunId {
    partition: u32,
    offset: u64,
}

impl SchemaRunId {
    /// Create a run identifier from its partition and offset components.
    #[must_use]
    pub const fn new(partition: u32, offset: u64) -> Self {
        Self { partition, offset }
    }

    /// The partition component.
    #[must_use]
    pub const fn partition(self) -> u32 {
        self.partition
    }

    /// The offset component.
    #[must_use]
    pub const fn offset(self) -> u64 {
        self.offset
    }
}

impl std::fmt::Display for SchemaRunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:016x}", self.partition, self.offset)
    }
}

impl Serialize for SchemaRunId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SchemaRunId", 2)?;
        state.serialize_field("partition", &self.partition)?;
        // Offset as a decimal string for JCS/I-JSON safety (matches runtime RunId).
        state.serialize_field("offset", &self.offset.to_string())?;
        state.end()
    }
}

/// Seed parsing a `u64` from either a decimal string (canonical) or a bare
/// number (backward-tolerant), mirroring the runtime `RunId` offset law.
struct OffsetU64;

impl<'de> DeserializeSeed<'de> for OffsetU64 {
    type Value = u64;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<u64, D::Error> {
        struct OffsetVisitor;

        impl Visitor<'_> for OffsetVisitor {
            type Value = u64;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a u64 offset as a decimal string or number")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
                Ok(v)
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
                v.parse::<u64>().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(OffsetVisitor)
    }
}

impl<'de> Deserialize<'de> for SchemaRunId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RunIdVisitor;

        impl<'de> Visitor<'de> for RunIdVisitor {
            type Value = SchemaRunId;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a run id as a legacy u64 or { partition, offset }")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<SchemaRunId, E> {
                Ok(SchemaRunId::new(0, v))
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<SchemaRunId, M::Error> {
                let mut partition: Option<u32> = None;
                let mut offset: Option<u64> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "partition" => {
                            if partition.is_some() {
                                return Err(de::Error::duplicate_field("partition"));
                            }
                            partition = Some(map.next_value()?);
                        }
                        "offset" => {
                            if offset.is_some() {
                                return Err(de::Error::duplicate_field("offset"));
                            }
                            offset = Some(map.next_value_seed(OffsetU64)?);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let partition =
                    partition.ok_or_else(|| de::Error::missing_field("partition"))?;
                let offset = offset.ok_or_else(|| de::Error::missing_field("offset"))?;
                Ok(SchemaRunId::new(partition, offset))
            }
        }

        deserializer.deserialize_any(RunIdVisitor)
    }
}

#[cfg(test)]
#[path = "schema_identity_tests.rs"]
mod schema_identity_tests;
