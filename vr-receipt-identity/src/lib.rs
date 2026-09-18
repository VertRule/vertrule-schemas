//! # `vr-receipt-identity` — Receipt-identity law owner
//!
//! This crate owns **how a receipt envelope becomes a commitment**: the
//! V1 full-envelope `event_hash` law over [`ReceiptEnvelope`] and the V2
//! tagged `receipt_digest` law over [`ReceiptEnvelopeV2`] (ADR-056 §3.2).
//! It is the long-term home for receipt-identity construction, separated
//! from `vertrule-schemas` (which owns only the wire *shape*).
//!
//! Boundary law:
//!
//! - `vertrule-schemas` defines `ReceiptEnvelope` / `ReceiptEnvelopeV2` shape.
//! - `vr-receipt-identity` defines how that shape becomes a commitment.
//! - producers call this crate to **mint** (`event_hash` for V1;
//!   [`seal_receipt_v2`] for V2 — the only V2 mint path).
//! - verifiers call this crate to **recompute** (`event_hash` for V1;
//!   [`compute_receipt_digest_v2`] for V2).
//! - `vr-jcs` supplies canonicalization; `vr-identity` supplies the declared
//!   digest domain ([`ReceiptDigestV2Identity`]); crypto supplies primitives.
//!
//! Load-bearing invariant: `vertrule-schemas` MUST NOT depend on this
//! crate (the dependency direction is one-way), so the graph stays
//! acyclic.
//!
//! `vertrule-schemas` contains no receipt-identity constructor. This crate is
//! the sole shared owner used by producers and verifiers.
//!
//! [`ReceiptEnvelope`]: vertrule_schemas::ReceiptEnvelope
//! [`ReceiptEnvelopeV2`]: vertrule_schemas::ReceiptEnvelopeV2

#![warn(missing_docs)]

mod canonical_identity;
mod commitment;
mod decision_projection;
mod error;
mod receipt_digest_v2;

pub use commitment::compute_event_hash;
pub use decision_projection::project_decision_payload;
pub use error::ReceiptIdentityError;
pub use receipt_digest_v2::{
    compute_receipt_digest_v2, seal_receipt_v2, ReceiptDigestV2Identity, ReceiptV2Draft,
    RECEIPT_V2_DOMAIN_TAG,
};
