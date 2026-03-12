//! # `vertrule-schemas` — Canonical schema types for `VertRule`
//!
//! Constitutional definitions with zero `vr-*` dependencies.
//!
//! Each specification version defines an identity triple of
//! `(spec_version, canonicalization, commitment_primitive)`.
//! Types in this crate are shape types that enforce wire-format
//! constraints without binding to a specific primitive. The active
//! bindings are declared in [`common::algorithms`] under the version
//! conservation clause: any change to canonicalization semantics,
//! commitment primitives, or envelope structure increments the
//! specification version.
//!
//! ## Module layout
//!
//! - [`common`] — Cross-cutting primitives: algorithm bindings, digest
//!   newtypes, identifiers, version tags.
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
//! ## Constants
//!
//! - [`common::algorithms::ENVELOPE_VERSION_1`]
//! - [`common::algorithms::DIGEST_ALGORITHM`]
//! - [`common::algorithms::DIGEST_HEX_LEN`]
//! - [`common::algorithms::DIGEST_BYTE_LEN`]
//! - [`common::algorithms::CANONICALIZATION`]

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

// ── Module hierarchy ────────────────────────────────────────────────

pub mod common;
pub mod context;
pub mod receipts;

// ── Crate-root modules (deferred from migration — see TODO in each) ─

mod boundary_origin;
mod error;
mod rbh_invariant;

// ── Ergonomic re-exports ────────────────────────────────────────────

pub use common::{DigestBytes, PolicyId, SchemaVersion};
pub use error::DefinitionError;
pub use receipts::{ReceiptEnvelope, ReceiptMetaV1, ReceiptType};

pub use boundary_origin::BoundaryOrigin;
pub use rbh_invariant::RBHInvariant;

// ── Tests (deferred types keep their test modules at crate root) ────

#[cfg(test)]
#[path = "boundary_origin_tests.rs"]
mod boundary_origin_tests;

#[cfg(test)]
#[path = "rbh_invariant_tests.rs"]
mod rbh_invariant_tests;
