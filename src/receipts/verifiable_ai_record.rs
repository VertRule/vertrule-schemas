//! Provider-interaction and verifiable-AI-record wire profiles (ADR-050).
//!
//! These are passive shapes. Receipt construction and record identity live in
//! `vr-verifiable-ai-record`; untrusted artifact verification lives in
//! `vertrule-verifier`.

use serde::{Deserialize, Serialize};

use crate::{DigestBytes, ReceiptEnvelope};

/// Provider-interaction payload discriminator.
pub const PROVIDER_INTERACTION_PAYLOAD_KIND: &str = "provider_interaction.v0";
/// Provider-interaction schema identifier.
pub const PROVIDER_INTERACTION_SCHEMA: &str = "vr.ai.provider_interaction@0.1";
/// Verifiable-record payload discriminator.
pub const VERIFIABLE_AI_RECORD_PAYLOAD_KIND: &str = "record.verifiable_ai_record";
/// Verifiable-record schema identifier.
pub const VERIFIABLE_AI_RECORD_SCHEMA: &str = "vr.record.verifiable_ai_record@0.1";
/// Truth-bounded record policy identifier.
pub const VERIFIABLE_AI_RECORD_POLICY: &str = "vr.record.integrity_not_truth@0.1";
/// Legacy canonical-string portable record artifact format.
pub const VERIFIABLE_AI_RECORD_FORMAT_V1: &str = "vr-verifiable-ai-record/v1";
/// Structured, presentation-safe portable record artifact format.
pub const VERIFIABLE_AI_RECORD_FORMAT_V2: &str = "vr-verifiable-ai-record/v2";
/// Current portable record artifact format emitted by shared law.
pub const VERIFIABLE_AI_RECORD_FORMAT: &str = VERIFIABLE_AI_RECORD_FORMAT_V2;

/// Exact request bytes submitted through the provider boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedRequest {
    /// UTF-8 prompt text.
    pub prompt: String,
    /// Domain-separated digest of the exact UTF-8 prompt bytes.
    pub prompt_digest: DigestBytes,
}

/// Exact text projection captured from a provider response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedResponse {
    /// Captured UTF-8 response text.
    pub text: String,
    /// Domain-separated digest of the exact captured text bytes.
    pub response_digest: DigestBytes,
}

/// Provider-neutral payload committed by an interaction receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderInteractionPayload {
    /// Frozen payload discriminator.
    pub payload_kind: String,
    /// Frozen payload schema.
    pub schema: String,
    /// Adapter-stable provider name.
    pub provider: String,
    /// Model requested by server configuration.
    pub requested_model: String,
    /// Provider-declared model, when exposed.
    pub resolved_model: Option<String>,
    /// Provider response id, when exposed; captured metadata, not attestation.
    pub provider_response_id: Option<String>,
    /// Frozen adapter projection policy.
    pub capture_policy_version: String,
    /// Exact captured request.
    pub request: CapturedRequest,
    /// Exact captured response projection.
    pub response: CapturedResponse,
    /// Explicit provider-attestation truth boundary.
    pub provider_attestation: String,
}

/// Root record-receipt payload binding every disclosed child identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiableAiRecordPayload {
    /// Frozen payload discriminator.
    pub payload_kind: String,
    /// Frozen record encoding law.
    pub schema: String,
    /// Frozen assurance policy: integrity and lineage, not truth.
    pub record_policy: String,
    /// Original answer interaction receipt identity.
    pub source_interaction_digest: DigestBytes,
    /// Stochastic extraction interaction receipt identity.
    pub extraction_interaction_digest: DigestBytes,
    /// Immutable proposal receipt identity.
    pub proposal_receipt_digest: DigestBytes,
    /// Explicit admission receipt identity.
    pub admission_receipt_digest: DigestBytes,
    /// Domain-separated admitted-proposal identity.
    pub admitted_proposal_digest: DigestBytes,
}

/// Legacy portable record containing canonical child JSON as escaped strings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiableAiRecordArtifactV1 {
    /// Frozen portable-artifact format.
    #[serde(rename = "_format")]
    pub format: String,
    /// Exact canonical root record receipt.
    pub record_canonical: String,
    /// Exact canonical original answer interaction receipt.
    pub source_interaction_canonical: String,
    /// Exact canonical stochastic extraction interaction receipt.
    pub extraction_interaction_canonical: String,
    /// Exact canonical proposal/admission bundle.
    pub proposal_admission_canonical: String,
}

/// Structured proposal/admission disclosure inside the portable record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiableAiRecordProposalAdmission {
    /// Shared proposal/admission package format.
    #[serde(rename = "_format")]
    pub format: String,
    /// Immutable proposal receipt.
    pub proposal: ReceiptEnvelope,
    /// Admission receipt produced from the external signal.
    pub admission: ReceiptEnvelope,
}

/// Structured portable package.
///
/// This container is deliberately not identity-bearing: whitespace and object
/// formatting may change. Each contained receipt is independently
/// canonicalized and verified, and the root record receipt remains the stable
/// record identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiableAiRecordArtifact {
    /// Frozen portable-package format.
    #[serde(rename = "_format")]
    pub format: String,
    /// Root identity-bearing record receipt.
    pub record: ReceiptEnvelope,
    /// Original answer interaction receipt.
    pub source_interaction: ReceiptEnvelope,
    /// Stochastic extraction interaction receipt.
    pub extraction_interaction: ReceiptEnvelope,
    /// Structured proposal/admission disclosure.
    pub proposal_admission: VerifiableAiRecordProposalAdmission,
}
