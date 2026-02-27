//! # `vr-definitions` — Canonical type definitions for `VertRule`
//!
//! Constitutional definitions with zero `vr-*` dependencies.
//!
//! Each specification version defines an identity triple of
//! `(spec_version, canonicalization, commitment_primitive)`.
//! Types in this crate are shape types that enforce wire-format
//! constraints without binding to a specific primitive. The active
//! bindings are declared in [`constants`] under the version
//! conservation clause: any change to canonicalization semantics,
//! commitment primitives, or envelope structure increments the
//! specification version.
//!
//! ## Types
//!
//! - [`DigestBytes`] — 32-byte cryptographic digest with strict hex serde
//! - [`ReceiptType`] — Receipt classification discriminator
//! - [`BoundaryOrigin`] — Boundary provenance discriminator
//! - [`PolicyId`] — Opaque policy identifier
//! - [`RBHInvariant`] — Constitutional identity continuity constraint (RBH)
//! - [`SchemaVersion`] — Schema version tag (carries identity triple)
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
mod rbh_invariant;
mod receipt_type;
mod schema_version;

pub use boundary_origin::BoundaryOrigin;
pub use digest_bytes::DigestBytes;
pub use error::DefinitionError;
pub use policy_id::PolicyId;
pub use rbh_invariant::RBHInvariant;
pub use receipt_type::ReceiptType;
pub use schema_version::SchemaVersion;

#[cfg(test)]
#[path = "digest_bytes_tests.rs"]
mod digest_bytes_tests;

#[cfg(test)]
#[path = "rbh_invariant_tests.rs"]
mod rbh_invariant_tests;

#[cfg(test)]
#[path = "policy_id_tests.rs"]
mod policy_id_tests;

#[cfg(test)]
#[path = "schema_version_tests.rs"]
mod schema_version_tests;

#[cfg(test)]
#[path = "receipt_type_tests.rs"]
mod receipt_type_tests;

#[cfg(test)]
#[path = "boundary_origin_tests.rs"]
mod boundary_origin_tests;
