//! Layered receipt-family payloads (ADR-040).
//!
//! Provider → Model → Pack receipts joined by **typed** lineage. Each is
//! the payload of a [`ReceiptEnvelope`](crate::ReceiptEnvelope) under
//! `receipt_type: event` with a payload-level `payload_kind`
//! discriminator (the documented envelope classification pattern, like
//! [`crate::DecisionReceiptPayload`]). Receipt identity rides the existing
//! `ConstitutionalEnvelopeV1` `event_hash` law — no new digest strategy and
//! no new envelope.
//!
//! Between-node edges are [`SupportMember::TypedReceiptDependency`]
//! members in the payload's support set; because the support set is
//! committed into the payload, a role or target swap changes the receipt
//! `event_hash`. A layered-family receipt carries **only** typed edges —
//! an untyped [`SupportMember::DependedOnReceipt`] in a layered payload is
//! a verifier rejection (the law is enforced in `vertrule-verifier`).
//!
//! Closure commitment (`pack.v0` root + bundle manifest) is added by a
//! later slice; this module owns the leaf and intermediate layers.

use serde::{Deserialize, Serialize};

use super::decision::SupportMember;

/// Payload-level subtype discriminator for Provider receipts.
pub const PROVIDER_PAYLOAD_KIND: &str = "provider.v0";

/// Payload-level subtype discriminator for Model receipts.
pub const MODEL_PAYLOAD_KIND: &str = "model.v0";

/// Provider receipt payload — provider identity and provider-level
/// evidence (the supply-chain leaf).
///
/// The support set holds the provider's own evidence members
/// ([`SupportMember::EvidenceDigest`] / [`SupportMember::CitedLink`]); a
/// provider receipt has no typed lineage edges of its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderReceiptPayload {
    /// Payload-level subtype discriminator ([`PROVIDER_PAYLOAD_KIND`]).
    pub payload_kind: String,
    /// Stable provider identity (e.g. `anthropic`, `aws`).
    pub provider_id: String,
    /// Provider-level evidence support set, BTree-ordered.
    pub support_set: Vec<SupportMember>,
}

/// Model receipt payload — model identity plus typed provider roles.
///
/// The support set holds one or more
/// [`SupportMember::TypedReceiptDependency`] edges, each
/// `target_schema = provider.v0`, with a [`DependencyRole`](super::decision::DependencyRole)
/// (`maker`/`host`/…) naming the provider's role. The edge object is
/// committed, so swapping a role changes this receipt's `event_hash`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelReceiptPayload {
    /// Payload-level subtype discriminator ([`MODEL_PAYLOAD_KIND`]).
    pub payload_kind: String,
    /// Stable model identity (e.g. `claude-opus-4-8`).
    pub model_id: String,
    /// Typed lineage to provider receipts, BTree-ordered.
    pub support_set: Vec<SupportMember>,
}
