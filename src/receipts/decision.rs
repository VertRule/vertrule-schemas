//! Decision Receipt payload schema — the governed decision commitment.
//!
//! A Decision Receipt is minted by a Decision Resolver (the browser-hosted
//! seam in `vr-browser-runtime`) and carried as the payload of a
//! [`ReceiptEnvelope`](crate::ReceiptEnvelope) under `receipt_type: event`
//! with the payload-level [`DECISION_PAYLOAD_KIND`] discriminator
//! (payload-level subtypes are the documented envelope classification
//! pattern — see [`crate::ReceiptType`] docs).
//!
//! The payload commits the **verdict** and the **support set** (MSS): the
//! minimal justification set the verdict rests on. The envelope already
//! commits `context_digest` and `policy_digest`; together these four are
//! the Decision Receipt contract. The first-match trace is presentational
//! and is deliberately NOT part of the payload — the support set is what
//! a stranger recomputes.
//!
//! Shapes are sourced from the authoring grammar (verdict + typed support
//! members), not from any evaluator's `PolicyConfig`. Serde shapes are
//! byte-law-pinned: the harvesting move from `vr-browser-runtime` was a
//! canonical-bytes no-op, guarded by goldens on both sides.

use serde::{Deserialize, Serialize};

/// Payload-level subtype discriminator for Decision Receipts.
pub const DECISION_PAYLOAD_KIND: &str = "decision.v0";

/// Schema identifier committed by Decision Receipt envelopes.
pub const DECISION_PAYLOAD_SCHEMA: &str = "vr.decision.v0";

/// The outcome of a governed policy decision.
///
/// `NoMatch` is never silently an Allow — the caller decides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecisionVerdict {
    /// Operation is allowed.
    Allow,
    /// Operation is denied.
    Deny {
        /// Presentational reason (the support set is what a stranger
        /// recomputes).
        reason: String,
        /// Structured violation code label.
        code: String,
    },
    /// Operation requires additional conditions.
    Conditional {
        /// Outstanding requirements.
        requirements: Vec<String>,
        /// Presentational reason.
        reason: String,
    },
    /// No applicable rule was definitive — the caller decides.
    NoMatch,
}

/// The relation an edge asserts between two receipts (ADR-040).
///
/// Closed set: a new relation is a lineage-law delta, so this enum is
/// deliberately **not** `#[non_exhaustive]`. The first layered-receipt
/// law traverses `depends_on` only; other active edge classes
/// (supersession etc.) are introduced by their own successor issue and
/// are excluded from the active closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyRelation {
    /// A required, acyclic justification edge. The only relation in the
    /// first layered-receipt active-closure law.
    DependsOn,
}

/// The role a depended-on receipt plays in its parent's supply chain
/// (ADR-040). Closed set — not `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyRole {
    /// Originator of the depended-on entity (e.g. the model maker).
    Maker,
    /// Operator that hosts/serves the depended-on entity.
    Host,
    /// Party that fine-tuned/adapted the depended-on entity.
    Tuner,
    /// Party that distributes the depended-on entity.
    Distributor,
}

/// One member of the support set a verdict rests on.
///
/// BTree-ordered at construction (variant order is load-bearing for
/// `event_hash` stability — do not reorder variants; add only at the end).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "member_kind", rename_all = "snake_case")]
pub enum SupportMember {
    /// A cited bare link (provenance-by-reference; can rot — the
    /// snapshot durability gradient applies).
    CitedLink {
        /// Claim id the link sources.
        id: String,
        /// The source URL or reference.
        url: String,
    },
    /// A receipt the decision depends on (receipt+verified channel).
    DependedOnReceipt {
        /// `event_hash` of the depended-on receipt.
        event_hash: String,
    },
    /// Digest of asserted evidence the firing rule consulted.
    EvidenceDigest {
        /// Stable identifier of the evidence slot.
        id: String,
        /// BLAKE3 hex digest of the evidence bytes.
        digest: String,
    },
    /// A committed selector value the firing rule consulted.
    SelectorValue {
        /// Context parameter key.
        key: String,
        /// Rendered value.
        value: String,
    },
    /// A **typed** lineage edge to a depended-on receipt (ADR-040).
    ///
    /// Distinct from [`SupportMember::DependedOnReceipt`] on purpose: a
    /// layered-family receipt (provider/model/pack) carries only typed
    /// edges, so a verifier can reject an untyped legacy dependency in a
    /// layered receipt. The edge object (relation, role, `target_schema`)
    /// is committed into the payload, so swapping a role changes the
    /// receipt `event_hash`.
    TypedReceiptDependency {
        /// `event_hash` of the depended-on receipt.
        event_hash: String,
        /// The relation this edge asserts.
        relation: DependencyRelation,
        /// The role the depended-on receipt plays.
        role: DependencyRole,
        /// Expected payload-kind schema id of the depended-on receipt
        /// (e.g. `provider.v0`). A mismatch is a verifier rejection.
        target_schema: String,
    },
}

/// The payload a Decision Receipt envelope commits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionReceiptPayload {
    /// Payload-level subtype discriminator ([`DECISION_PAYLOAD_KIND`]).
    pub payload_kind: String,
    /// The verdict.
    pub verdict: DecisionVerdict,
    /// The minimal support set, BTree-ordered.
    pub support_set: Vec<SupportMember>,
}
