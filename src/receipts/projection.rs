//! Canonical projection contract for proof-bearing receipts.
//!
//! Every proof-bearing receipt type across all `VertRule` repositories must
//! implement [`ProjectsToReceiptEnvelope`] to produce a canonical public
//! envelope. This is the structural bridge between internal receipt
//! representations and the public trust surface.
//!
//! # Preservation requirements
//!
//! Projection must preserve:
//!
//! | Property | Preserved | How |
//! |----------|-----------|-----|
//! | Identity | `event_hash` matches version commitment scope | Via [`compute_event_hash`](crate::receipts::compute_event_hash) |
//! | Provenance | `boundary_origin` maps to correct origin | Direct field mapping |
//! | Schema binding | `schema_digest` matches the schema governing payload | Emitter responsibility |
//! | Payload commitment | `payload` contains trust-bearing content | Domain-specific serialization |
//! | Chain linkage | `parent_id` preserves chain position if applicable | Optional but must not be lost |
//!
//! The commitment scope depends on `envelope_version`:
//! - **V1**: `BLAKE3(JCS(payload))` — payload only
//! - **V2**: `BLAKE3(JCS(envelope \ {event_hash}))` — all trust-bearing fields
//!
//! # What may remain private
//!
//! - Internal intermediate computation state
//! - Execution-local telemetry and timing
//! - Implementation-specific metadata
//! - Operational IDs that don't cross trust boundaries
//!
//! # Invariant
//!
//! ```text
//! verify(project(r)) = verify(canonical(r))
//! ```
//!
//! For all externally relevant trust claims.

use super::ReceiptEnvelope;
use crate::DefinitionError;

/// Canonical projection from a proof-bearing receipt to the public envelope.
///
/// Implementors must ensure:
/// - `event_hash` is computed via [`compute_event_hash`](crate::receipts::compute_event_hash)
/// - `schema_digest` matches the schema governing `payload`
/// - `boundary_origin` reflects the producing boundary
/// - projection is deterministic: same input produces same envelope
///
/// # Errors
///
/// Returns [`DefinitionError`] if the receipt cannot be validly projected
/// (e.g., payload contains floats, schema binding is missing, or a
/// trust-bearing field cannot be represented in the envelope).
pub trait ProjectsToReceiptEnvelope {
    /// Project this receipt to a canonical [`ReceiptEnvelope`].
    ///
    /// # Errors
    ///
    /// Returns [`DefinitionError`] if the projection would lose
    /// trust-bearing semantics or violate envelope invariants.
    fn project(&self) -> Result<ReceiptEnvelope, DefinitionError>;
}
