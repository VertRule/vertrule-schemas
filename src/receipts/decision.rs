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

/// One member of the support set a verdict rests on.
///
/// BTree-ordered at construction (variant order is load-bearing for
/// `event_hash` stability — do not reorder variants).
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
