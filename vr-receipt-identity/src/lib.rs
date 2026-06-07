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
//! ## Extraction status (Stage 1)
//!
//! During the transition this crate is an **independent reimplementation**
//! of the law that still lives in `vertrule-schemas`. The two are pinned
//! byte-for-byte to the committed Layer A golden vectors
//! (`docs/audits/junk-drawer-inventory/fixtures/receipt-identity/`). The
//! rule for the move is: *move ownership, do not change bytes.* Consumers
//! are repointed in later stages; the `vertrule-schemas` copy is removed
//! only once nothing depends on it.
//!
//! [`ReceiptEnvelope`]: vertrule_schemas::ReceiptEnvelope

#![warn(missing_docs)]

mod canonical_identity;
mod commitment;
mod error;

pub use commitment::compute_event_hash;
pub use error::ReceiptIdentityError;
