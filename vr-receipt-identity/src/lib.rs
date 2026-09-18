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
//! - producers call this crate to **mint** through [`seal_receipt_v2`] —
//!   the only V2 mint path — and its typed projections such as
//!   [`project_decision_payload_v2`].
//! - verifiers call this crate to **recompute** ([`compute_event_hash`]
//!   for historical V1 artifacts; [`compute_receipt_digest_v2`] for V2).
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
//! ## Mint ratchet (ADR-056 §3.6, R12)
//!
//! Admitting a type for V2 minting denies its V1 mint path. At C11 the
//! `vr.governance.decision` family entered V2: the V1 governance-decision
//! constructor (`project_decision_payload`, receipt type `governance`) is
//! **`MintDenied`** and no longer exists in this crate. Its frozen BEFORE
//! fixture is `test-vectors/receipt_v1_governance_decision_before_001.json`
//! (event hash `eca50079…4ad3`, V1 schema digest `4c5d6d82…7852`); the
//! AFTER fixture is `test-vectors/receipt_v2_governance_decision_001.json`
//! (`receipt_digest` `b2fcea60…ff58`). V1 verification of historical bytes
//! stays available through [`compute_event_hash`] (K7); a structural guard
//! test asserts no V1 governance-decision mint path remains.
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
pub use decision_projection::project_decision_payload_v2;
pub use error::ReceiptIdentityError;
pub use receipt_digest_v2::{
    compute_receipt_digest_v2, seal_receipt_v2, ReceiptDigestV2Identity, ReceiptV2Draft,
    RECEIPT_V2_DOMAIN_TAG,
};

#[cfg(test)]
#[path = "mint_ratchet_guard_tests.rs"]
mod mint_ratchet_guard_tests;
