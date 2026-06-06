//! Verification-manifest exchange DTOs.
//!
//! This module defines the **public schema surface** for verification-package
//! manifests — the transport shapes that declare the inputs and identities
//! needed to verify a package. It is schema land only: these types do **not**
//! verify signatures, evaluate policy, resolve models, load packs, access I/O,
//! or import runtime-internal crates. Identity fields use this crate's local
//! schema carriers ([`crate::SchemaModelId`], [`crate::SchemaRunId`], etc.),
//! digests use [`crate::DigestBytes`], and bundle mode is the shared
//! [`crate::BundleMode`].
//!
//! [`VerifyManifest`] is a schema carrier for *verification inputs*: it
//! describes the declared identities/digests a verifier needs. It does **not**
//! imply that verification has occurred. Likewise [`SignedManifest`] carries
//! signature-shaped material without implying the signature has been verified.

use serde::{Deserialize, Serialize};

use crate::{
    BundleMode, DigestBytes, SchemaKeyId, SchemaModelId, SchemaPolicyPackId, SchemaPublicKeyHex,
    SchemaRunId, SchemaSuiteId,
};

/// Schema identifier for verification manifests.
pub const VERIFY_MANIFEST_SCHEMA: &str = "vertrule.verify_manifest.v1";

/// Schema version string.
pub const VERIFY_MANIFEST_VERSION: &str = "1.0";

/// Domain-separation prefix for manifest signatures.
///
/// Frozen byte-material, not signing logic: the canonical prefix that
/// producer/verifier crates use to build the signing preimage
/// `VR-ManifestSig|v1|<manifest_digest>|<signed_at>`. This crate performs no
/// signing; it only publishes the constant.
pub const MANIFEST_SIG_DOMAIN: &str = "VR-ManifestSig|v1|";

/// Canonicalization domain string.
///
/// Frozen once published: old verifiers reject manifests with unknown canon
/// domains rather than partially validating. Format:
/// `VR-Canon|<version>|<scale>|<rounding>|<json_canon>`.
pub const CANON_DOMAIN_V1: &str = "VR-Canon|v1|fixed10e6|toward_zero|JCS";

/// Rounding mode for fixed-point canonicalization.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RoundingMode {
    /// Round toward zero (truncate).
    #[default]
    TowardZero,
    /// Round toward negative infinity (floor).
    TowardNegInf,
    /// Round toward positive infinity (ceiling).
    TowardPosInf,
    /// Round to nearest, ties to even.
    Nearest,
}

/// What data is captured in a provider attestation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CaptureMode {
    /// Only digests stored; payloads live in separate run artifacts.
    #[default]
    DigestsOnly,
    /// Full payloads included alongside digests.
    FullCapture,
}

/// Canonical fixed-point representation for verification.
///
/// All numeric values that affect hashes use this type instead of
/// floating-point, ensuring byte-identical hashes across platforms. The value
/// is `raw / 10^scale_pow10`.
///
/// ```
/// use vertrule_schemas::manifest::{CanonFixed, RoundingMode};
///
/// // Represent 3.141592 with 6 decimal places.
/// let pi = CanonFixed::new(6, RoundingMode::TowardZero, 3_141_592);
/// assert_eq!(pi.raw, 3_141_592);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct CanonFixed {
    /// Scale exponent: actual value = `raw / 10^scale_pow10`.
    pub scale_pow10: i32,
    /// Rounding mode used when converting to fixed-point.
    pub rounding: RoundingMode,
    /// Integer representation of the scaled value.
    pub raw: i64,
}

impl CanonFixed {
    /// Construct a canonical fixed-point value.
    #[must_use]
    pub const fn new(scale_pow10: i32, rounding: RoundingMode, raw: i64) -> Self {
        Self {
            scale_pow10,
            rounding,
            raw,
        }
    }
}

/// Reference to a single model file for external verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ModelFileRef {
    /// Relative path within the model directory.
    pub path: String,
    /// Expected BLAKE3 digest of the file.
    pub digest: DigestBytes,
    /// File size in bytes.
    pub size: u64,
}

impl ModelFileRef {
    /// Construct a model file reference.
    #[must_use]
    pub const fn new(path: String, digest: DigestBytes, size: u64) -> Self {
        Self { path, digest, size }
    }
}

/// Reference to external model weights (used when weights are external).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ModelRef {
    /// Model identifier (must match the bundle's model section).
    pub model_id: SchemaModelId,
    /// Source identifier for provenance tracking
    /// (`<provider>:<repo>@<commit_or_version>`).
    pub source: String,
    /// Expected files with their digests.
    pub files: Vec<ModelFileRef>,
}

impl ModelRef {
    /// Construct a model reference.
    #[must_use]
    pub const fn new(model_id: SchemaModelId, source: String, files: Vec<ModelFileRef>) -> Self {
        Self {
            model_id,
            source,
            files,
        }
    }
}

/// Provider attestation for attested-external bundles.
///
/// Binds a provider response to stored run artifacts via digests. Carries the
/// declared attestation inputs only; it performs no verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ProviderAttestation {
    /// What data was captured in this attestation.
    pub capture_mode: CaptureMode,
    /// Provider name (e.g. `"xAI"`).
    pub provider: String,
    /// Model name as reported by the provider.
    pub model: String,
    /// API endpoint or route label.
    pub endpoint: String,
    /// When the response was captured (RFC 3339 UTC).
    pub timestamp_utc: String,
    /// BLAKE3 digest of the JCS-canonicalized request parameters.
    pub request_digest: DigestBytes,
    /// BLAKE3 digest of the response text bytes.
    pub response_digest: DigestBytes,
    /// BLAKE3 digest of relevant headers (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_digest: Option<DigestBytes>,
    /// Full request parameters (only when `capture_mode == FullCapture`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<serde_json::Value>,
    /// Full response text (only when `capture_mode == FullCapture`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_text: Option<String>,
    /// Relevant headers as key-value pairs (only when `capture_mode == FullCapture`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<(String, String)>>,
}

impl ProviderAttestation {
    /// Construct a digests-bearing attestation; optional payloads default to
    /// `None` and may be set on the returned value.
    #[must_use]
    pub const fn new(
        capture_mode: CaptureMode,
        provider: String,
        model: String,
        endpoint: String,
        timestamp_utc: String,
        request_digest: DigestBytes,
        response_digest: DigestBytes,
    ) -> Self {
        Self {
            capture_mode,
            provider,
            model,
            endpoint,
            timestamp_utc,
            request_digest,
            response_digest,
            headers_digest: None,
            request: None,
            response_text: None,
            headers: None,
        }
    }
}

/// Helper for serde `skip_serializing_if` on bool fields.
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_false(b: &bool) -> bool {
    !*b
}

/// Entry types in the manifest: a direct blob or a witnessed directory.
///
/// The `type` field is the serde tag for polymorphic serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ManifestEntry {
    /// Direct blob: digest computed over raw file bytes.
    Blob {
        /// Relative path from package root.
        path: String,
        /// BLAKE3 digest of the file content.
        digest: DigestBytes,
        /// File size in bytes.
        size: u64,
        /// Whether this file is optional (missing = warning, not error).
        #[serde(default, skip_serializing_if = "is_false")]
        optional: bool,
    },
    /// Witnessed directory: verified via a witness manifest file.
    WitnessedDir {
        /// Directory path relative to package root.
        path: String,
        /// Path to the witness manifest file.
        witness_path: String,
        /// BLAKE3 digest of the witness file content.
        witness_digest: DigestBytes,
    },
}

impl ManifestEntry {
    /// The path for this entry.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Blob { path, .. } | Self::WitnessedDir { path, .. } => path,
        }
    }
}

/// Model identity and boundary information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ModelManifestSection {
    /// Model identifier.
    pub model_id: SchemaModelId,
    /// Relative path to the model directory within the package.
    pub model_dir: String,
    /// TIDAL model digest (weights + config + tokenizer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tidal_model_digest: Option<DigestBytes>,
    /// HF checkpoint digest (weights only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hf_checkpoint_digest: Option<DigestBytes>,
}

impl ModelManifestSection {
    /// Construct a model section; optional digests default to `None`.
    #[must_use]
    pub const fn new(model_id: SchemaModelId, model_dir: String) -> Self {
        Self {
            model_id,
            model_dir,
            tidal_model_digest: None,
            hf_checkpoint_digest: None,
        }
    }
}

/// Suite entry in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct SuiteManifestEntry {
    /// Suite identifier.
    pub suite_id: SchemaSuiteId,
    /// Relative path to the suite directory within the package.
    pub path: String,
    /// BLAKE3 digest of the canonical suite JSON.
    pub suite_digest: DigestBytes,
}

impl SuiteManifestEntry {
    /// Construct a suite entry.
    #[must_use]
    pub const fn new(suite_id: SchemaSuiteId, path: String, suite_digest: DigestBytes) -> Self {
        Self {
            suite_id,
            path,
            suite_digest,
        }
    }
}

/// Policy-pack entry in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct PolicyPackManifestEntry {
    /// Pack identifier.
    pub pack_id: SchemaPolicyPackId,
    /// Relative path to the policy-pack directory within the package.
    pub path: String,
    /// BLAKE3 digest of the `policy.toml` content.
    pub pack_digest: DigestBytes,
}

impl PolicyPackManifestEntry {
    /// Construct a policy-pack entry.
    #[must_use]
    pub const fn new(pack_id: SchemaPolicyPackId, path: String, pack_digest: DigestBytes) -> Self {
        Self {
            pack_id,
            path,
            pack_digest,
        }
    }
}

/// Run entry in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RunManifestEntry {
    /// Run identifier.
    pub run_id: SchemaRunId,
    /// Relative path to the run directory within the package.
    pub run_dir: String,
    /// BLAKE3 digest of `run_header.json`.
    pub run_header_digest: DigestBytes,
    /// Number of events in the receipt chain.
    pub event_count: u64,
    /// BLAKE3 digest of the final event in the chain (chain tip).
    pub chain_tip_digest: DigestBytes,
}

impl RunManifestEntry {
    /// Construct a run entry.
    #[must_use]
    pub const fn new(
        run_id: SchemaRunId,
        run_dir: String,
        run_header_digest: DigestBytes,
        event_count: u64,
        chain_tip_digest: DigestBytes,
    ) -> Self {
        Self {
            run_id,
            run_dir,
            run_header_digest,
            event_count,
            chain_tip_digest,
        }
    }
}

/// Default canon domain for serde.
fn default_canon_domain() -> String {
    CANON_DOMAIN_V1.to_string()
}

/// Verification-package manifest.
///
/// Root structure binding all package artifacts together with their digests.
/// A schema carrier for *verification inputs* — it declares identities and
/// digests; it does not itself verify anything.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct VerifyManifest {
    /// Schema identifier: `"vertrule.verify_manifest.v1"`.
    pub schema: String,
    /// Schema version: `"1.0"`.
    pub version: String,
    /// When the manifest was created (RFC 3339 UTC).
    pub created_at: String,
    /// Canonicalization domain ([`CANON_DOMAIN_V1`]).
    #[serde(default = "default_canon_domain")]
    pub canon_domain: String,
    /// Bundle verification mode.
    #[serde(default)]
    pub bundle_mode: BundleMode,
    /// External model reference (present when weights are external).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_ref: Option<ModelRef>,
    /// Provider attestation (present for attested-external bundles).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_attestation: Option<ProviderAttestation>,
    /// Model identity and boundary information.
    pub model: ModelManifestSection,
    /// Prompt suites included in the package.
    pub suites: Vec<SuiteManifestEntry>,
    /// Policy packs included in the package.
    pub policy_packs: Vec<PolicyPackManifestEntry>,
    /// Run artifacts included in the package.
    pub runs: Vec<RunManifestEntry>,
    /// All file entries, sorted lexicographically by path.
    pub entries: Vec<ManifestEntry>,
    /// BLAKE3 digest of the canonical JSON body (excluding this field).
    pub manifest_digest: DigestBytes,
}

impl VerifyManifest {
    /// Construct a manifest with the canonical schema/version/canon-domain
    /// constants and empty sections; populate the public fields as needed.
    #[must_use]
    pub fn new(created_at: String) -> Self {
        Self {
            schema: VERIFY_MANIFEST_SCHEMA.to_string(),
            version: VERIFY_MANIFEST_VERSION.to_string(),
            created_at,
            canon_domain: CANON_DOMAIN_V1.to_string(),
            bundle_mode: BundleMode::default(),
            model_ref: None,
            provider_attestation: None,
            model: ModelManifestSection::default(),
            suites: Vec::new(),
            policy_packs: Vec::new(),
            runs: Vec::new(),
            entries: Vec::new(),
            manifest_digest: DigestBytes::from_array([0u8; 32]),
        }
    }
}

/// Signed manifest wrapper.
///
/// A **schema carrier for signature material**: it bundles a [`VerifyManifest`]
/// with signature-shaped fields. It does **not** imply that the signature has
/// been verified — verification is a producer/verifier responsibility outside
/// this schema crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct SignedManifest {
    /// The verification manifest being signed.
    pub manifest: VerifyManifest,
    /// When the manifest was signed (RFC 3339 UTC); part of the signed payload.
    pub signed_at: String,
    /// Base64-encoded Ed25519 signature (opaque transport string).
    pub signature: String,
    /// Hex-encoded Ed25519 public key.
    pub public_key: SchemaPublicKeyHex,
    /// Key identifier derived from the public key.
    pub key_id: SchemaKeyId,
}

impl SignedManifest {
    /// Construct a signed-manifest carrier.
    #[must_use]
    pub const fn new(
        manifest: VerifyManifest,
        signed_at: String,
        signature: String,
        public_key: SchemaPublicKeyHex,
        key_id: SchemaKeyId,
    ) -> Self {
        Self {
            manifest,
            signed_at,
            signature,
            public_key,
            key_id,
        }
    }
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod manifest_tests;
