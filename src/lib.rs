//! # `vertrule-schemas` — Canonical schema types for `VertRule`
//!
//! Constitutional definitions and passive wire-format support.
//! Canonical JSON mechanics come from [`vr-jcs`]; receipt commitment
//! construction lives downstream in `vr-receipt-identity`.
//!
//! Each specification version defines an identity triple of
//! `(spec_version, canonicalization, commitment_primitive)`.
//! Types in this crate are shape types that enforce wire-format
//! constraints without binding to a specific primitive. The active
//! bindings are colocated on the types they constrain:
//! [`DigestBytes::BYTE_LEN`], [`DigestBytes::HEX_LEN`] for digest
//! shape, and [`SchemaVersion::digest_algorithm`],
//! [`SchemaVersion::canonicalization`] for version-derived identity.
//! Any change to canonicalization semantics, commitment primitives,
//! or envelope structure increments the specification version.
//!
//! [`vr-jcs`]: https://crates.io/crates/vr-jcs
//!
//! ## Module layout
//!
//! - [`common`] — Cross-cutting primitives: digest newtypes, identifiers,
//!   version tags.
//! - [`context`] — Execution-context types: identity continuity constraints.
//! - [`receipts`] — Receipt-spine discriminators and constitutional envelope types.
//! - [`mri`] — MRI (Model Reasoning Instrumentation) payload schemas.
//!
//! ## Re-exported types
//!
//! All public types are re-exported at the crate root for ergonomic access.
//!
//! ### Core types
//!
//! - [`DigestBytes`] — 32-byte cryptographic digest with strict hex serde
//! - [`IJsonUInt`] — non-negative integer guaranteed to round-trip in I-JSON
//! - [`CanonicalPayload`] — float-guarded JSON payload
//! - [`PolicyId`] — opaque policy identifier
//! - [`SchemaVersion`] — schema version tag (carries identity triple)
//! - [`DefinitionError`] — validation error types
//!
//! ### Receipt types
//!
//! - [`ReceiptEnvelope`] — constitutional public receipt envelope (V1)
//! - [`ReceiptType`] — receipt classification discriminator (V1)
//! - [`BoundaryOrigin`] — boundary provenance discriminator (V1)
//! - [`ProjectsToReceiptEnvelope`] — canonical projection trait
//!
//! ### Receipt types — V2 (ADR-056)
//!
//! - [`ReceiptEnvelopeV2`] — canonical V2 receipt envelope (`receipt_digest`)
//! - [`ReceiptTypeV2`] — closed semantic receipt-type vocabulary
//! - [`PayloadSchemaV2`] — admitted payload-schema labels; identities derive
//!   through `vr_identity::digest::SchemaLabelIdentity`
//! - [`RuntimePortSubmitOutcomePayload`], [`RuntimePortCommandKind`],
//!   [`TransitionCommitment`] — passive `vr.runtime_port.submit_outcome` payload
//!
//! ### Context types
//!
//! - [`RBHInvariant`] — constitutional identity continuity constraint (RBH)
//!
//! ### MRI domain types
//!
//! - [`MriBatchPayload`] — batch-aware MRI invariant payload
//! - [`GradientCouplingPayload`] — gradient coupling diagnostic payload
//! - [`ReductionProvenance`] — reduction pipeline provenance
//! - [`ReductionMode`] — batch reduction strategy
//! - [`ReductionAxis`] — tensor axis discriminator
//! - [`TokenReduction`] — token aggregation method
//! - [`BatchReduction`] — batch aggregation method
//!
//! ## Identity / schema separation (ADR-057)
//!
//! The `SchemaId` grammar (`vr.<domain>.<name>@<major>.<minor>`) is identity
//! admission and lives in `vr-identity` as `vr_identity::SchemaId`; it is not
//! re-exported here. Schema *meaning* — which label governs which payload,
//! receipt types, versions and envelopes — stays in this crate.
//!
//! ## Associated Constants
//!
//! - [`DigestBytes::BYTE_LEN`] — 32
//! - [`DigestBytes::HEX_LEN`] — 64
//! - [`SchemaVersion::V1`] — the V1 schema version (full-envelope `event_hash` commitment)
//! - [`SchemaVersion::V2`] — the V2 schema version (tagged `receipt_digest` commitment)
//! - [`SchemaVersion::digest_algorithm`] — `"BLAKE3"`
//! - [`SchemaVersion::canonicalization`] — `"JCS"`

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

// ── Module hierarchy ────────────────────────────────────────────────

pub mod bundle;
pub(crate) mod canonical_identity;
pub mod common;
pub mod context;
pub mod governance;
pub(crate) mod jcs;
pub mod manifest;
pub mod mgs;
pub mod mri;
pub mod pack;
pub mod policy;
pub mod receipts;

// ── Ergonomic re-exports ────────────────────────────────────────────

pub use bundle::BundleMode;
pub use common::{
    CanonicalPayload, ContentIdentityDigest, ContextDigest, DefinitionError, DigestBytes,
    IJsonUInt, PayloadDigest, PolicyDigest, PolicyId, ReceiptDigest, SchemaDigest, SchemaKeyId,
    SchemaModelId, SchemaPolicyPackId, SchemaPublicKeyHex, SchemaReceiptId, SchemaRunId,
    SchemaSuiteId, SchemaVersion,
};
pub use context::RBHInvariant;
pub use governance::{
    ActionNamespace, AdapterOriginId, AdapterReference, DecisionPayload, EntityNamespace,
    GovernancePrincipalId, GovernanceScope, GovernedAction, GovernedSubject, PolicyBindingRef,
    PolicyTemplate, SurfaceInstanceId, Verdict,
};
pub use mgs::{
    CertificateKind, CertificateSummary, SearchPosture, StatusTransition, TransitionJustification,
};
pub use mri::{
    BatchReduction, GradientCouplingPayload, MriBatchPayload, ReductionAxis, ReductionMode,
    ReductionProvenance, TokenReduction,
};
pub use policy::{
    ClaimEvidence, EvaluationInput, EvaluationInputKindV1, GovernanceEvaluationInputV1,
    GovernanceInputError, GovernanceOperationV1, GovernancePolicyStatusV1,
    GovernanceSystemStatusV1, GovernanceSystemSubjectV1, InputCanonicalizationError,
    LinkedPolicyStateV1, GOVERNANCE_INPUT_FORMAT, INPUT_FORMAT,
};
pub use receipts::{
    AdmissionReceiptPayload, AdmittedClaim, AdmittedClaimOperation, AdmittedProposal,
    AgentProposalReceiptPayload, AttestationPurpose, BoundaryOrigin, CapturedRequest,
    CapturedResponse, ClaimAdmissionDecision, ClaimRejectionReason, ClosureManifest,
    DecisionReceiptPayload, DecisionVerdict, DependencyRelation, DependencyRole,
    EventHashProfileId, ExternalAdmissionSignal, ModelReceiptPayload, PackReceiptPayload,
    PayloadSchemaV2, ProjectsToReceiptEnvelope, ProposalAdmissionBundle, ProposedTextClaim,
    ProviderInteractionPayload, ProviderReceiptPayload, ReceiptEnvelope, ReceiptEnvelopeV2,
    ReceiptType, ReceiptTypeV2, RejectedClaim, RuntimePortCommandKind,
    RuntimePortSubmitOutcomePayload, SupportMember, TextClaimAgentProposal, TrainingReceipt,
    TransitionCommitment, VerifiableAiRecordArtifact, VerifiableAiRecordArtifactV1,
    VerifiableAiRecordPayload, VerifiableAiRecordProposalAdmission, VerifiedReceiptMetadata,
    AGENT_PROPOSAL_PAYLOAD_KIND, AGENT_PROPOSAL_SCHEMA, CLOSURE_MANIFEST_SCHEMA,
    MODEL_PAYLOAD_KIND, PACK_PAYLOAD_KIND, PROPOSAL_ADMISSION_BUNDLE_FORMAT,
    PROPOSAL_ADMISSION_PAYLOAD_KIND, PROPOSAL_ADMISSION_SCHEMA, PROVIDER_INTERACTION_PAYLOAD_KIND,
    PROVIDER_INTERACTION_SCHEMA, PROVIDER_PAYLOAD_KIND, VERIFIABLE_AI_RECORD_FORMAT,
    VERIFIABLE_AI_RECORD_FORMAT_V1, VERIFIABLE_AI_RECORD_FORMAT_V2,
    VERIFIABLE_AI_RECORD_PAYLOAD_KIND, VERIFIABLE_AI_RECORD_POLICY, VERIFIABLE_AI_RECORD_SCHEMA,
};
