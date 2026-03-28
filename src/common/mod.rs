//! Cross-cutting schema primitives shared across all receipt and context types.
//!
//! This module collects types that are not scoped to a single domain
//! (receipts, context, policy) but are used structurally throughout the
//! constitutional layer: digest newtypes, identifiers, version tags,
//! canonical payload guards, and validation errors.

mod canonical_payload;
mod digest_bytes;
mod error;
mod i_json_uint;
mod ids;
mod schema_id;
mod versions;

pub use canonical_payload::CanonicalPayload;
pub use digest_bytes::DigestBytes;
pub use error::DefinitionError;
pub use i_json_uint::IJsonUInt;
pub use ids::PolicyId;
pub use schema_id::SchemaId;
pub use versions::SchemaVersion;
