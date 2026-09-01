//! # `vr-receipt-identity` — Receipt-identity law owner
//!
//! This crate owns **how a [`ReceiptEnvelope`] becomes a commitment**:
//! the canonical-identity digest helper and the full-envelope
//! `event_hash` law. It is the long-term home for receipt-identity
//! construction, separated from `vertrule-schemas` (which owns only the
//! wire *shape*).
//!
//! Boundary law:
//!
//! - `vertrule-schemas` defines `ReceiptEnvelope` shape.
//! - `vr-receipt-identity` defines how that shape becomes a commitment.
//! - producers call this crate to **mint** `event_hash`.
//! - verifiers call this crate to **recompute** `event_hash`.
//! - `vr-jcs` supplies canonicalization; crypto supplies primitives.
//!
//! Load-bearing invariant: `vertrule-schemas` MUST NOT depend on this
//! crate (the dependency direction is one-way), so the graph stays
//! acyclic.
//!
//! `vertrule-schemas` contains no receipt-identity constructor. This crate is
//! the sole shared owner used by producers and verifiers.
//!
//! [`ReceiptEnvelope`]: vertrule_schemas::ReceiptEnvelope

#![warn(missing_docs)]

mod canonical_identity;
mod commitment;
mod decision_projection;
mod error;

pub use commitment::compute_event_hash;
pub use decision_projection::project_decision_payload;
pub use error::ReceiptIdentityError;
