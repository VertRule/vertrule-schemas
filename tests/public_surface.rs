//! Public surface regression test for vertrule-schemas v0.1.
//!
//! Asserts that the blessed public API symbols (constitutional nouns only)
//! compile and are usable. Review against `PUBLIC_SURFACE.md` when preparing
//! releases.
//!
//! Notably absent (by design):
//! - JCS functions (live in vr-jcs)
//! - `compute_event_hash` (not root-exported; available via `receipts::`)
//! - `ReceiptEnvelope` methods (nouns only, no construction or judgment)

#![deny(unused_imports)]

// Wire shapes
use vertrule_schemas::ReceiptEnvelope;
use vertrule_schemas::ReceiptMetaV1;

// Discriminators
use vertrule_schemas::BoundaryOrigin;
use vertrule_schemas::ReceiptType;

// Validated scalars
use vertrule_schemas::CanonicalPayload;
use vertrule_schemas::DigestBytes;
use vertrule_schemas::IJsonUInt;
use vertrule_schemas::PolicyId;
use vertrule_schemas::SchemaId;
use vertrule_schemas::SchemaVersion;

// Context
use vertrule_schemas::RBHInvariant;

// Projection trait
use vertrule_schemas::ProjectsToReceiptEnvelope;

// Error
use vertrule_schemas::DefinitionError;

// MRI domain types
use vertrule_schemas::BatchReduction;
use vertrule_schemas::GradientCouplingPayload;
use vertrule_schemas::MriBatchPayload;
use vertrule_schemas::ReductionAxis;
use vertrule_schemas::ReductionMode;
use vertrule_schemas::ReductionProvenance;
use vertrule_schemas::TokenReduction;

// Scoped export (not root)
use vertrule_schemas::receipts::compute_event_hash;

#[test]
fn public_surface_nouns_are_usable() -> Result<(), anyhow::Error> {
    // DigestBytes
    let d = DigestBytes::from_array([0xaa; 32]);
    assert_eq!(DigestBytes::BYTE_LEN, 32);
    assert_eq!(DigestBytes::HEX_LEN, 64);
    assert_eq!(d.as_bytes().len(), 32);

    // IJsonUInt
    let t = IJsonUInt::new(42)?;
    assert_eq!(t.get(), 42);

    // SchemaVersion
    assert_eq!(SchemaVersion::V1.get(), 1);
    assert_eq!(SchemaVersion::V2.get(), 2);
    assert_eq!(SchemaVersion::V1.digest_algorithm(), "BLAKE3");
    assert_eq!(SchemaVersion::V1.canonicalization(), "JCS");

    // ReceiptType and BoundaryOrigin exist as enums
    let _ = ReceiptType::Governance;
    let _ = BoundaryOrigin::Engine;

    // CanonicalPayload
    let payload =
        CanonicalPayload::new(serde_json::json!({"k": "v"})).map_err(|e| anyhow::anyhow!("{e}"))?;
    assert!(payload.as_value().is_object());

    // ReceiptEnvelope is a pure data struct
    let envelope = ReceiptEnvelope {
        envelope_version: SchemaVersion::V1,
        receipt_type: ReceiptType::Governance,
        context_digest: d,
        schema_digest: d,
        policy_digest: d,
        logical_time: t,
        event_hash: d,
        parent_id: None,
        boundary_origin: None,
        digest_algorithm: None,
        canonicalization: None,
        payload,
    };
    let _json = serde_json::to_string(&envelope)?;

    // Scoped: compute_event_hash is accessible via receipts::
    let _hash = compute_event_hash(&envelope).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Suppress unused-import warnings for types used only as existence checks
    let _ = std::any::type_name::<ReceiptMetaV1>();
    let _ = std::any::type_name::<PolicyId>();
    let _ = std::any::type_name::<SchemaId>();
    let _ = std::any::type_name::<RBHInvariant>();
    let _ = std::any::type_name::<DefinitionError>();
    let _ = std::any::type_name::<dyn ProjectsToReceiptEnvelope>();

    // MRI domain types exist
    let _ = std::any::type_name::<MriBatchPayload>();
    let _ = std::any::type_name::<GradientCouplingPayload>();
    let _ = std::any::type_name::<ReductionProvenance>();
    let _ = ReductionMode::PerExampleThenMean;
    let _ = ReductionAxis::Token;
    let _ = TokenReduction::Mean;
    let _ = BatchReduction::Mean;

    Ok(())
}
