//! Pack-index exchange DTOs.
//!
//! This module defines the **public schema surface** for verification-pack
//! indexes — the transport shapes exchanged between producers and verifiers.
//! It is schema land only: these types do **not** sign, verify signatures,
//! authorize, evaluate policy, hash, or access I/O. The pack-index signing and
//! verification algorithms live in the producer/verifier crates; only the
//! byte-frozen domain-separation constant is mirrored here as schema material.
//!
//! Identity-bearing fields use this crate's local schema carriers
//! ([`crate::SchemaKeyId`], [`crate::SchemaPublicKeyHex`]) and digest fields use
//! [`crate::DigestBytes`]; no runtime-internal identity/crypto crate is imported.

use serde::{Deserialize, Serialize};

use crate::{DigestBytes, SchemaKeyId, SchemaPublicKeyHex};

/// Schema identifier for pack indexes.
pub const PACK_INDEX_SCHEMA: &str = "vertrule.pack_index.v1";

/// Schema version string.
pub const PACK_INDEX_VERSION: &str = "1.0";

/// Domain-separation prefix for pack-index signatures.
///
/// This is **frozen byte-material**, not signing logic: it is the canonical
/// prefix that producer/verifier crates use when constructing the signing
/// preimage `VR-PackIndexSig|v1|<index_digest>|<signed_at>`. Changing it would
/// break verification of all existing signed pack indexes. This crate performs
/// no signing; it only publishes the constant.
pub const PACK_INDEX_SIG_DOMAIN: &str = "VR-PackIndexSig|v1|";

/// How model weights are handled in a bundle, determining verification strategy.
///
/// Shared schema enum between pack and manifest surfaces. Wire form is
/// `snake_case`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BundleMode {
    /// Weights included in the package; fully replayable.
    #[default]
    ReplayableIncluded,
    /// Weights external; verifier must supply them.
    ReplayableExternal,
    /// No replay possible; verify receipts plus provider attestation.
    AttestedExternal,
}

/// Bundle identifier within a verification pack.
///
/// Wire form: a transparent JSON string. Pure transport carrier (renamed from
/// the singular `BundleId` to avoid confusion with generic identity types).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PackBundleId(String);

impl PackBundleId {
    /// Create a `PackBundleId` from any string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PackBundleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Reference to a bundle within the pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct PackBundleRef {
    /// Bundle identifier, unique within the pack.
    pub bundle_id: PackBundleId,
    /// Relative path to the bundle directory within the pack.
    pub path: String,
    /// BLAKE3 digest of the bundle's `verify_manifest.json`.
    pub manifest_digest: DigestBytes,
    /// Bundle verification mode (mirrors the manifest's `bundle_mode`).
    pub bundle_mode: BundleMode,
}

impl PackBundleRef {
    /// Construct a bundle reference.
    #[must_use]
    pub const fn new(
        bundle_id: PackBundleId,
        path: String,
        manifest_digest: DigestBytes,
        bundle_mode: BundleMode,
    ) -> Self {
        Self {
            bundle_id,
            path,
            manifest_digest,
            bundle_mode,
        }
    }
}

/// Pack-index root structure binding multiple bundle manifests together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct PackIndex {
    /// Schema identifier: `"vertrule.pack_index.v1"`.
    pub schema: String,
    /// Schema version: `"1.0"`.
    pub version: String,
    /// When the index was created (RFC 3339 UTC).
    pub created_at: String,
    /// Bundle references, ordered by `bundle_id`.
    pub bundles: Vec<PackBundleRef>,
    /// BLAKE3 digest of the canonical JSON body (excluding this field).
    pub index_digest: DigestBytes,
}

impl PackIndex {
    /// Construct a pack index with the canonical schema and version constants.
    #[must_use]
    pub fn new(created_at: String, bundles: Vec<PackBundleRef>, index_digest: DigestBytes) -> Self {
        Self {
            schema: PACK_INDEX_SCHEMA.to_string(),
            version: PACK_INDEX_VERSION.to_string(),
            created_at,
            bundles,
            index_digest,
        }
    }
}

/// Signed pack-index wrapper.
///
/// `SignedPackIndex` is a **schema carrier for signature material**: it bundles
/// a [`PackIndex`] with signature-shaped fields. It does **not** imply that the
/// signature has been verified or is trusted — verification is a
/// producer/verifier responsibility outside this schema crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct SignedPackIndex {
    /// The pack index being signed.
    pub index: PackIndex,
    /// When the index was signed (RFC 3339 UTC); part of the signed payload.
    pub signed_at: String,
    /// Base64-encoded Ed25519 signature (opaque transport string).
    pub signature: String,
    /// Hex-encoded Ed25519 public key.
    pub public_key: SchemaPublicKeyHex,
    /// Key identifier derived from the public key.
    pub key_id: SchemaKeyId,
}

impl SignedPackIndex {
    /// Construct a signed pack-index carrier.
    #[must_use]
    pub const fn new(
        index: PackIndex,
        signed_at: String,
        signature: String,
        public_key: SchemaPublicKeyHex,
        key_id: SchemaKeyId,
    ) -> Self {
        Self {
            index,
            signed_at,
            signature,
            public_key,
            key_id,
        }
    }
}

#[cfg(test)]
#[path = "pack_tests.rs"]
mod pack_tests;
