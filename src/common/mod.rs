//! Cross-cutting schema primitives shared across all receipt and context types.
//!
//! This module collects types that are not scoped to a single domain
//! (receipts, context, policy) but are used structurally throughout the
//! constitutional layer: digest newtypes, identifiers, and version tags.

mod digest_bytes;
mod ids;
mod versions;

pub use digest_bytes::DigestBytes;
pub use ids::PolicyId;
pub use versions::SchemaVersion;
