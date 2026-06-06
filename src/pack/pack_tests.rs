//! Golden serde/JCS tests for the pack-index DTO surface.

use crate::pack::{
    PackBundleId, PackBundleRef, PackIndex, SignedPackIndex, PACK_INDEX_SCHEMA,
    PACK_INDEX_SIG_DOMAIN, PACK_INDEX_VERSION,
};
use crate::{BundleMode, DigestBytes, SchemaKeyId, SchemaPublicKeyHex};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

fn canon(value: &impl serde::Serialize) -> Result<Vec<u8>, anyhow::Error> {
    let json = serde_json::to_vec(value)?;
    Ok(vr_jcs::to_canon_bytes_from_slice(&json)?)
}

fn sample_bundle_ref() -> PackBundleRef {
    PackBundleRef::new(
        PackBundleId::new("qwen-0p5b"),
        "bundles/qwen".to_string(),
        digest(0x22),
        BundleMode::ReplayableExternal,
    )
}

fn sample_index() -> PackIndex {
    PackIndex::new(
        "2026-01-01T00:00:00Z".to_string(),
        vec![sample_bundle_ref()],
        digest(0x11),
    )
}

#[test]
fn constants_are_frozen() {
    assert_eq!(PACK_INDEX_SCHEMA, "vertrule.pack_index.v1");
    assert_eq!(PACK_INDEX_VERSION, "1.0");
    assert_eq!(PACK_INDEX_SIG_DOMAIN, "VR-PackIndexSig|v1|");
}

#[test]
fn bundle_mode_wire_forms() -> Result<(), anyhow::Error> {
    assert_eq!(
        serde_json::to_string(&BundleMode::ReplayableIncluded)?,
        "\"replayable_included\""
    );
    assert_eq!(
        serde_json::to_string(&BundleMode::ReplayableExternal)?,
        "\"replayable_external\""
    );
    assert_eq!(
        serde_json::to_string(&BundleMode::AttestedExternal)?,
        "\"attested_external\""
    );
    assert_eq!(BundleMode::default(), BundleMode::ReplayableIncluded);
    let back: BundleMode = serde_json::from_str("\"attested_external\"")?;
    assert_eq!(back, BundleMode::AttestedExternal);
    Ok(())
}

#[test]
fn pack_bundle_id_is_transparent_string() -> Result<(), anyhow::Error> {
    let id = PackBundleId::new("grok");
    assert_eq!(serde_json::to_string(&id)?, "\"grok\"");
    let back: PackBundleId = serde_json::from_str("\"grok\"")?;
    assert_eq!(back, id);
    assert_eq!(back.as_str(), "grok");
    Ok(())
}

#[test]
fn pack_bundle_ref_round_trip_and_golden() -> Result<(), anyhow::Error> {
    let r = sample_bundle_ref();
    let back: PackBundleRef = serde_json::from_str(&serde_json::to_string(&r)?)?;
    assert_eq!(back, r);

    let golden = concat!(
        r#"{"bundle_id":"qwen-0p5b","bundle_mode":"replayable_external","#,
        r#""manifest_digest":"2222222222222222222222222222222222222222222222222222222222222222","#,
        r#""path":"bundles/qwen"}"#
    );
    assert_eq!(canon(&r)?, golden.as_bytes());
    Ok(())
}

#[test]
fn pack_index_round_trip_and_golden() -> Result<(), anyhow::Error> {
    let index = sample_index();
    let back: PackIndex = serde_json::from_str(&serde_json::to_string(&index)?)?;
    assert_eq!(back, index);
    assert_eq!(index.schema, "vertrule.pack_index.v1");
    assert_eq!(index.version, "1.0");

    let golden = concat!(
        r#"{"bundles":[{"bundle_id":"qwen-0p5b","bundle_mode":"replayable_external","#,
        r#""manifest_digest":"2222222222222222222222222222222222222222222222222222222222222222","#,
        r#""path":"bundles/qwen"}],"created_at":"2026-01-01T00:00:00Z","#,
        r#""index_digest":"1111111111111111111111111111111111111111111111111111111111111111","#,
        r#""schema":"vertrule.pack_index.v1","version":"1.0"}"#
    );
    assert_eq!(canon(&index)?, golden.as_bytes());
    Ok(())
}

#[test]
fn signed_pack_index_round_trip_and_golden() -> Result<(), anyhow::Error> {
    let signed = SignedPackIndex::new(
        sample_index(),
        "2026-01-02T00:00:00Z".to_string(),
        "c2lnbmF0dXJl".to_string(),
        SchemaPublicKeyHex::new("aabb"),
        SchemaKeyId::new("key-1"),
    );
    let back: SignedPackIndex = serde_json::from_str(&serde_json::to_string(&signed)?)?;
    assert_eq!(back, signed);

    let golden = concat!(
        r#"{"index":{"bundles":[{"bundle_id":"qwen-0p5b","bundle_mode":"replayable_external","#,
        r#""manifest_digest":"2222222222222222222222222222222222222222222222222222222222222222","#,
        r#""path":"bundles/qwen"}],"created_at":"2026-01-01T00:00:00Z","#,
        r#""index_digest":"1111111111111111111111111111111111111111111111111111111111111111","#,
        r#""schema":"vertrule.pack_index.v1","version":"1.0"},"key_id":"key-1","#,
        r#""public_key":"aabb","signature":"c2lnbmF0dXJl","signed_at":"2026-01-02T00:00:00Z"}"#
    );
    assert_eq!(canon(&signed)?, golden.as_bytes());
    Ok(())
}

#[test]
fn deny_unknown_fields_rejected() {
    let json = concat!(
        r#"{"schema":"vertrule.pack_index.v1","version":"1.0","#,
        r#""created_at":"2026-01-01T00:00:00Z","bundles":[],"#,
        r#""index_digest":"1111111111111111111111111111111111111111111111111111111111111111","#,
        r#""unexpected":true}"#
    );
    assert!(serde_json::from_str::<PackIndex>(json).is_err());
}
