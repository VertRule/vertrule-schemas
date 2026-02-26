//! # `vr-definitions` — Canonical type definitions for `VertRule`
//!
//! Layer 0 of the constitutional layer. Zero `vr-*` dependencies.
//!
//! Provides schema-shape types and validation rules used across
//! the `VertRule` ecosystem. Does not perform hashing — only
//! validates digest shape (hex length, lowercase, byte count).
//!
//! ## Types
//!
//! - [`DigestBytes`] — 32-byte BLAKE3 digest with strict hex serde
//! - [`ReceiptType`] — Receipt classification enum
//! - [`BoundaryOrigin`] — Boundary origin enum
//! - [`PolicyId`] — Opaque policy identifier
//! - [`SchemaVersion`] — Schema version tag
//! - [`DefinitionError`] — Validation error types
//!
//! ## Constants
//!
//! - [`constants::ENVELOPE_VERSION_1`]
//! - [`constants::DIGEST_ALGORITHM`]
//! - [`constants::DIGEST_HEX_LEN`]
//! - [`constants::DIGEST_BYTE_LEN`]
//! - [`constants::CANONICALIZATION`]

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

mod boundary_origin;
pub mod constants;
mod digest_bytes;
mod error;
mod policy_id;
mod receipt_type;
mod schema_version;

pub use boundary_origin::BoundaryOrigin;
pub use digest_bytes::DigestBytes;
pub use error::DefinitionError;
pub use policy_id::PolicyId;
pub use receipt_type::ReceiptType;
pub use schema_version::SchemaVersion;

#[cfg(test)]
#[path = "digest_bytes_tests.rs"]
mod digest_bytes_tests;
