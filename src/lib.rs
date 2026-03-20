//! # `vertrule-schemas` — Canonical schema types for `VertRule`
//!
//! Constitutional definitions with zero `vr-*` dependencies.
//!
//! Each specification version defines an identity triple of
//! `(spec_version, canonicalization, commitment_primitive)`.
//! Types in this crate are shape types that enforce wire-format
//! constraints without binding to a specific primitive. The active
//! bindings are colocated on the types they constrain:
//! [`DigestBytes::BYTE_LEN`], [`DigestBytes::HEX_LEN`] for digest
//! shape, and [`SchemaVersion::digest_algorithm`],
//! [`SchemaVersion::canonicalization`] for version-derived identity.
//! Any change to canonicalization semantics, commitment primitives,
//! or envelope structure increments the specification version.
//!
//! ## Module layout
//!
//! - [`common`] — Cross-cutting primitives: digest newtypes, identifiers,
//!   version tags.
//! - [`context`] — Execution-context types (placeholder, Phase 2).
//! - [`receipts`] — Receipt-spine discriminators and constitutional envelope types.
//!
//! ## Re-exported types
//!
//! All public types are re-exported at the crate root for ergonomic access.
//!
//! - [`DigestBytes`] — 32-byte cryptographic digest with strict hex serde
//! - [`ReceiptEnvelope`] — Constitutional public receipt envelope
//! - [`ReceiptMetaV1`] — Constitutional receipt metadata header
//! - [`ReceiptType`] — Receipt classification discriminator
//! - [`BoundaryOrigin`] — Boundary provenance discriminator
//! - [`PolicyId`] — Opaque policy identifier
//! - [`RBHInvariant`] — Constitutional identity continuity constraint (RBH)
//! - [`SchemaVersion`] — Schema version tag (carries identity triple)
//! - [`DefinitionError`] — Validation error types
//!
//! ## Associated Constants
//!
//! - [`DigestBytes::BYTE_LEN`] — 32
//! - [`DigestBytes::HEX_LEN`] — 64
//! - [`SchemaVersion::V1`] — spec version 1
//! - [`SchemaVersion::digest_algorithm`] — `"BLAKE3"` (v1)
//! - [`SchemaVersion::canonicalization`] — `"JCS"` (v1)

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

// ── Module hierarchy ────────────────────────────────────────────────

pub mod common;
pub mod context;
pub mod jcs;
pub mod receipts;

// ── Crate-root modules (deferred from migration — see TODO in each) ─

mod boundary_origin;
mod canonical_payload;
mod error;
mod rbh_invariant;

// ── Ergonomic re-exports ────────────────────────────────────────────

pub use common::{DigestBytes, PolicyId, SchemaVersion};
pub use error::DefinitionError;
pub use jcs::{canonicalize, to_canon_bytes, to_canon_string, JcsError};
pub use receipts::{ReceiptEnvelope, ReceiptMetaV1, ReceiptType};

pub use boundary_origin::BoundaryOrigin;
pub use canonical_payload::CanonicalPayload;
pub use rbh_invariant::RBHInvariant;

// ── Tests (deferred types keep their test modules at crate root) ────

#[cfg(test)]
#[path = "boundary_origin_tests.rs"]
mod boundary_origin_tests;

#[cfg(test)]
#[path = "rbh_invariant_tests.rs"]
mod rbh_invariant_tests;
