//! Passive closed payload for `vr.runtime_port.submit_outcome` receipts
//! (ADR-056 §3.3, C8).
//!
//! The five `RuntimePort` submission facts that the V1 envelope committed
//! through the `runtime_port_event_preimage_v1` `event_hash` law
//! (`command_id`, `command_kind`, `command_schema_digest`,
//! `payload_digest`, `payload_bytes_digest`) are carried here as typed
//! payload fields: domain-object identities the receipt **commits** (R5),
//! never the receipt's own identity. The submitted command's schema, which
//! V1 bound in the envelope `schema_digest`, is the payload field
//! `command_schema_digest`; the V2 envelope `schema_digest` identifies
//! `vr.runtime_port.submit_outcome@0.1` (R6).
//!
//! Shape only: no construction or validation lives here. Relation laws
//! (`evidence_non_empty`, `command_kind_is_submit`) are registry rows in
//! `vertrule-verifier`.

use serde::{Deserialize, Serialize};

use crate::{CanonicalPayload, DecisionReceiptPayload, DigestBytes};

/// Closed `RuntimePort` command kind carried in a submit-outcome payload.
///
/// Only `Submit` is admitted on a `vr.runtime_port.submit_outcome` receipt;
/// the registry relation `command_kind_is_submit` enforces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePortCommandKind {
    /// A state-mutating submission evaluated before mutation.
    Submit,
}

/// Transition commitments produced when a `Submit` was applied.
///
/// Absent (`None` on the parent) when the decision denied mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionCommitment {
    /// Digest of the applied batch.
    pub batch_digest: DigestBytes,
    /// Digest of the state this transition produced.
    pub post_state_digest: DigestBytes,
    /// Digest of the engine's sealed execution context at application.
    pub engine_context_digest: DigestBytes,
    /// Digest of the preceding transition, when one exists. Omitted when
    /// absent; `null` is rejected.
    #[serde(
        default,
        deserialize_with = "crate::receipts::present_slot::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_digest: Option<DigestBytes>,
}

/// Payload of a `vr.runtime_port.submit_outcome` receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimePortSubmitOutcomePayload {
    /// Identity of the submitted command.
    pub command_id: DigestBytes,
    /// Kind of the submitted command; only `submit` is admitted.
    pub command_kind: RuntimePortCommandKind,
    /// Schema identity of the **submitted** command (V1 bound this in the
    /// envelope `schema_digest`); a domain-object identity (R5).
    pub command_schema_digest: DigestBytes,
    /// Caller-claimed digest of the submitted payload.
    pub payload_digest: DigestBytes,
    /// Runtime-observed digest of the submitted payload bytes.
    pub payload_bytes_digest: DigestBytes,
    /// Ordered evidence receipt digests consulted by the evaluation.
    /// Non-emptiness is a registry relation law.
    pub evidence_receipt_digests: Vec<DigestBytes>,
    /// The submitted value, echoed verbatim.
    pub submitted: CanonicalPayload,
    /// The `decision.v0` payload the evaluation produced.
    pub decision: DecisionReceiptPayload,
    /// Digest of the policy configuration the evaluation ran under.
    pub policy_config_digest: DigestBytes,
    /// Transition commitments when the submission was applied. Omitted
    /// when absent; `null` is rejected.
    #[serde(
        default,
        deserialize_with = "crate::receipts::present_slot::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub transition: Option<TransitionCommitment>,
}

#[cfg(test)]
#[path = "runtime_port_submit_outcome_tests.rs"]
mod runtime_port_submit_outcome_tests;
