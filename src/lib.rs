//! # `vertrule-schemas` — Canonical schema types for `VertRule`
//!
//! Constitutional definitions and protocol-scoped commitment support.
//! The only `vr-*` dependency is [`vr-jcs`] (JCS canonicalization
//! primitive).
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
//! [`vr-jcs`]: https://crates.io/crates/vr-jcs
//!
//! ## Module layout
//!
//! - [`common`] — Cross-cutting primitives: digest newtypes, identifiers,
//!   version tags.
//! - [`context`] — Execution-context types: identity continuity constraints.
//! - [`receipts`] — Receipt-spine discriminators and constitutional envelope types.
//! - [`mri`] — MRI (Model Reasoning Instrumentation) payload schemas.
//!
//! ## Re-exported types
//!
//! All public types are re-exported at the crate root for ergonomic access.
//!
//! ### Core types
//!
//! - [`DigestBytes`] — 32-byte cryptographic digest with strict hex serde
//! - [`IJsonUInt`] — non-negative integer guaranteed to round-trip in I-JSON
//! - [`CanonicalPayload`] — float-guarded JSON payload
//! - [`PolicyId`] — opaque policy identifier
//! - [`SchemaId`] — validated schema identifier (`vr.<domain>.<name>@<major>.<minor>`)
//! - [`SchemaVersion`] — schema version tag (carries identity triple)
//! - [`DefinitionError`] — validation error types
//!
//! ### Receipt types
//!
//! - [`ReceiptEnvelope`] — constitutional public receipt envelope
//! - [`ReceiptType`] — receipt classification discriminator
//! - [`BoundaryOrigin`] — boundary provenance discriminator
//! - [`ProjectsToReceiptEnvelope`] — canonical projection trait
//!
//! ### Context types
//!
//! - [`RBHInvariant`] — constitutional identity continuity constraint (RBH)
//!
//! ### MRI domain types
//!
//! - [`MriBatchPayload`] — batch-aware MRI invariant payload
//! - [`GradientCouplingPayload`] — gradient coupling diagnostic payload
//! - [`ReductionProvenance`] — reduction pipeline provenance
//! - [`ReductionMode`] — batch reduction strategy
//! - [`ReductionAxis`] — tensor axis discriminator
//! - [`TokenReduction`] — token aggregation method
//! - [`BatchReduction`] — batch aggregation method
//!
//! ## Associated Constants
//!
//! - [`DigestBytes::BYTE_LEN`] — 32
//! - [`DigestBytes::HEX_LEN`] — 64
//! - [`SchemaVersion::V1`] — the current schema version (full-envelope commitment)
//! - [`SchemaVersion::digest_algorithm`] — `"BLAKE3"`
//! - [`SchemaVersion::canonicalization`] — `"JCS"`

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

// ── Module hierarchy ────────────────────────────────────────────────

pub mod common;
pub mod context;
pub mod governance;
pub(crate) mod jcs;
pub mod mgs;
pub mod mri;
pub mod receipts;

// ── Ergonomic re-exports ────────────────────────────────────────────

pub use common::{
    CanonicalPayload, DefinitionError, DigestBytes, IJsonUInt, PolicyId, SchemaId, SchemaVersion,
};
pub use context::RBHInvariant;
pub use governance::{
    ActionNamespace, AdapterOrigin, AdapterReference, EntityNamespace, GovernancePrincipalId,
    GovernanceScope, GovernedAction, GovernedDecisionPayload, GovernedSubject, PolicyBindingRef,
    PolicyTemplate, SurfaceInstanceId, Verdict,
};
pub use mgs::{
    CertificateKind, CertificateSummary, SearchPosture, StatusTransition, TransitionJustification,
};
pub use mri::{
    BatchReduction, GradientCouplingPayload, MriBatchPayload, ReductionAxis, ReductionMode,
    ReductionProvenance, TokenReduction,
};
pub use receipts::{BoundaryOrigin, ProjectsToReceiptEnvelope, ReceiptEnvelope, ReceiptType};
