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
use crate::DigestBytes;

/// Payload-level subtype discriminator for Provider receipts.
pub const PROVIDER_PAYLOAD_KIND: &str = "provider.v0";

/// Payload-level subtype discriminator for Model receipts.
pub const MODEL_PAYLOAD_KIND: &str = "model.v0";

/// Payload-level subtype discriminator for the root Pack receipt.
pub const PACK_PAYLOAD_KIND: &str = "pack.v0";

/// Schema id committed by a [`ClosureManifest`].
pub const CLOSURE_MANIFEST_SCHEMA: &str = "vr.closure_manifest.v0";

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

/// Pack receipt payload — the deployment-context **root** of a layered
/// evidence graph (ADR-040).
///
/// Only the root Pack receipt commits the closure: `closure_manifest_digest`
/// binds the exact resolved transitive closure (see [`ClosureManifest`]).
/// Provider and Model receipts commit only their direct typed edges and
/// stay independently reusable — they must not commit to every future
/// bundle that contains them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackReceiptPayload {
    /// Payload-level subtype discriminator ([`PACK_PAYLOAD_KIND`]).
    pub payload_kind: String,
    /// Stable pack identity (the deployer's release context id).
    pub pack_id: String,
    /// Root-only commitment to the resolved transitive closure:
    /// `BLAKE3(JCS(ClosureManifest \ {manifest_digest}))`.
    pub closure_manifest_digest: DigestBytes,
    /// Typed lineage to model receipts, BTree-ordered.
    pub support_set: Vec<SupportMember>,
}

/// Canonical bundle manifest committing the exact resolved **dependency**
/// closure of a layered evidence graph (ADR-040).
///
/// The self-commit pattern is the twin of
/// [`VerifyManifest`](crate::manifest::VerifyManifest): `manifest_digest =
/// BLAKE3(JCS(ClosureManifest \ {manifest_digest}))`. The binding to a
/// specific root is **forward-only**: the root Pack receipt commits this
/// `manifest_digest` as its `closure_manifest_digest`. The manifest itself
/// therefore must not reference the root's `event_hash` (the root cannot
/// commit to its own hash), so `receipt_closure` lists the transitive
/// dependencies reachable from the root, **excluding the root**. The
/// verifier resolves the actual graph and rejects any drift between the
/// reachable dependency set and `receipt_closure`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureManifest {
    /// Schema id ([`CLOSURE_MANIFEST_SCHEMA`]).
    pub schema: String,
    /// Every dependency receipt `event_hash` reachable from the root,
    /// excluding the root, lexicographically sorted (a set committed as a
    /// canonical list).
    pub receipt_closure: Vec<String>,
    /// `receipt_closure.len()` as a committed integer (redundant guard).
    pub dependency_count: u64,
    /// Self-commit digest over this manifest with `manifest_digest`
    /// excluded from the canonical input.
    pub manifest_digest: DigestBytes,
}
