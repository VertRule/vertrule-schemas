//! Golden serde/JCS tests for the verification-manifest DTO surface.

use crate::manifest::{
    CanonFixed, CaptureMode, ManifestEntry, ModelFileRef, ModelManifestSection, ModelRef,
    PolicyPackManifestEntry, ProviderAttestation, RoundingMode, RunManifestEntry, SignedManifest,
    SuiteManifestEntry, VerifyManifest, CANON_DOMAIN_V1, MANIFEST_SIG_DOMAIN,
    VERIFY_MANIFEST_SCHEMA, VERIFY_MANIFEST_VERSION,
};
use crate::{
    BundleMode, DigestBytes, SchemaKeyId, SchemaModelId, SchemaPolicyPackId, SchemaPublicKeyHex,
    SchemaRunId, SchemaSuiteId,
};

fn digest(fill: u8) -> DigestBytes {
    DigestBytes::from_array([fill; 32])
}

fn hex(fill: u8) -> String {
    DigestBytes::from_array([fill; 32]).to_hex()
}

fn canon(value: &impl serde::Serialize) -> Result<Vec<u8>, anyhow::Error> {
    let json = serde_json::to_vec(value)?;
    Ok(vr_jcs::to_canon_bytes_from_slice(&json)?)
}

#[test]
fn constants_are_frozen() {
    assert_eq!(VERIFY_MANIFEST_SCHEMA, "vertrule.verify_manifest.v1");
    assert_eq!(VERIFY_MANIFEST_VERSION, "1.0");
    assert_eq!(MANIFEST_SIG_DOMAIN, "VR-ManifestSig|v1|");
    assert_eq!(CANON_DOMAIN_V1, "VR-Canon|v1|fixed10e6|toward_zero|JCS");
}

#[test]
fn enum_wire_forms() -> Result<(), anyhow::Error> {
    assert_eq!(serde_json::to_string(&RoundingMode::TowardZero)?, "\"toward_zero\"");
    assert_eq!(serde_json::to_string(&RoundingMode::TowardNegInf)?, "\"toward_neg_inf\"");
    assert_eq!(serde_json::to_string(&RoundingMode::Nearest)?, "\"nearest\"");
    assert_eq!(serde_json::to_string(&CaptureMode::DigestsOnly)?, "\"digests_only\"");
    assert_eq!(serde_json::to_string(&CaptureMode::FullCapture)?, "\"full_capture\"");
    assert_eq!(RoundingMode::default(), RoundingMode::TowardZero);
    assert_eq!(CaptureMode::default(), CaptureMode::DigestsOnly);
    Ok(())
}

#[test]
fn canon_fixed_golden() -> Result<(), anyhow::Error> {
    let cf = CanonFixed::new(6, RoundingMode::TowardZero, 3_141_592);
    let back: CanonFixed = serde_json::from_str(&serde_json::to_string(&cf)?)?;
    assert_eq!(back, cf);
    assert_eq!(
        canon(&cf)?,
        br#"{"raw":3141592,"rounding":"toward_zero","scale_pow10":6}"#
    );
    Ok(())
}

#[test]
fn model_ref_round_trip() -> Result<(), anyhow::Error> {
    let m = ModelRef::new(
        SchemaModelId::new("Qwen2"),
        "huggingface:Qwen/Qwen2@abc".to_string(),
        vec![ModelFileRef::new("w.safetensors".to_string(), digest(0x44), 1024)],
    );
    let back: ModelRef = serde_json::from_str(&serde_json::to_string(&m)?)?;
    assert_eq!(back, m);
    Ok(())
}

#[test]
fn provider_attestation_digests_only_golden() -> Result<(), anyhow::Error> {
    let pa = ProviderAttestation::new(
        CaptureMode::DigestsOnly,
        "xAI".to_string(),
        "grok-1".to_string(),
        "/v1/chat".to_string(),
        "2026-01-01T00:00:00Z".to_string(),
        digest(0x55),
        digest(0x66),
    );
    let back: ProviderAttestation = serde_json::from_str(&serde_json::to_string(&pa)?)?;
    assert_eq!(back, pa);
    // Optional payload/headers fields are skipped when None.
    let golden = format!(
        concat!(
            r#"{{"capture_mode":"digests_only","endpoint":"/v1/chat","model":"grok-1","#,
            r#""provider":"xAI","request_digest":"{}","response_digest":"{}","#,
            r#""timestamp_utc":"2026-01-01T00:00:00Z"}}"#
        ),
        hex(0x55),
        hex(0x66)
    );
    assert_eq!(canon(&pa)?, golden.as_bytes());
    Ok(())
}

#[test]
fn manifest_entry_variants_golden() -> Result<(), anyhow::Error> {
    let blob = ManifestEntry::Blob {
        path: "a.txt".to_string(),
        digest: digest(0x33),
        size: 10,
        optional: false,
    };
    let back: ManifestEntry = serde_json::from_str(&serde_json::to_string(&blob)?)?;
    assert_eq!(back, blob);
    assert_eq!(blob.path(), "a.txt");
    // `optional: false` is skipped.
    let blob_golden = format!(
        r#"{{"digest":"{}","path":"a.txt","size":10,"type":"Blob"}}"#,
        hex(0x33)
    );
    assert_eq!(canon(&blob)?, blob_golden.as_bytes());

    let dir = ManifestEntry::WitnessedDir {
        path: "weights".to_string(),
        witness_path: "weights/layers.b3".to_string(),
        witness_digest: digest(0x77),
    };
    let dir_golden = format!(
        r#"{{"path":"weights","type":"WitnessedDir","witness_digest":"{}","witness_path":"weights/layers.b3"}}"#,
        hex(0x77)
    );
    assert_eq!(canon(&dir)?, dir_golden.as_bytes());
    Ok(())
}

#[test]
fn run_manifest_entry_golden() -> Result<(), anyhow::Error> {
    let run = RunManifestEntry::new(
        SchemaRunId::new(1, 1000),
        "runs/0".to_string(),
        digest(0x88),
        7,
        digest(0x99),
    );
    let back: RunManifestEntry = serde_json::from_str(&serde_json::to_string(&run)?)?;
    assert_eq!(back, run);
    // run_id is the nested {partition, offset-string} carrier.
    let golden = format!(
        concat!(
            r#"{{"chain_tip_digest":"{}","event_count":7,"run_dir":"runs/0","#,
            r#""run_header_digest":"{}","run_id":{{"offset":"1000","partition":1}}}}"#
        ),
        hex(0x99),
        hex(0x88)
    );
    assert_eq!(canon(&run)?, golden.as_bytes());
    Ok(())
}

#[test]
fn suite_and_policy_entries_round_trip() -> Result<(), anyhow::Error> {
    let s = SuiteManifestEntry::new(SchemaSuiteId::new("demo"), "suites/demo".to_string(), digest(1));
    let sb: SuiteManifestEntry = serde_json::from_str(&serde_json::to_string(&s)?)?;
    assert_eq!(sb, s);

    let p = PolicyPackManifestEntry::new(
        SchemaPolicyPackId::new("fence@0.1"),
        "packs/fence".to_string(),
        digest(2),
    );
    let pb: PolicyPackManifestEntry = serde_json::from_str(&serde_json::to_string(&p)?)?;
    assert_eq!(pb, p);
    Ok(())
}

#[test]
fn model_section_skips_optional_digests() -> Result<(), anyhow::Error> {
    let m = ModelManifestSection::new(SchemaModelId::new("Qwen2"), "models/qwen".to_string());
    assert_eq!(
        canon(&m)?,
        br#"{"model_dir":"models/qwen","model_id":"Qwen2"}"#
    );
    Ok(())
}

fn sample_manifest() -> VerifyManifest {
    let mut m = VerifyManifest::new("2026-01-01T00:00:00Z".to_string());
    m.model = ModelManifestSection::new(SchemaModelId::new("Qwen2"), "models/qwen".to_string());
    m.manifest_digest = digest(0xab);
    m
}

#[test]
fn verify_manifest_golden() -> Result<(), anyhow::Error> {
    let m = sample_manifest();
    let back: VerifyManifest = serde_json::from_str(&serde_json::to_string(&m)?)?;
    assert_eq!(back, m);
    assert_eq!(m.schema, "vertrule.verify_manifest.v1");
    assert_eq!(m.bundle_mode, BundleMode::ReplayableIncluded);

    let golden = format!(
        concat!(
            r#"{{"bundle_mode":"replayable_included","canon_domain":"VR-Canon|v1|fixed10e6|toward_zero|JCS","#,
            r#""created_at":"2026-01-01T00:00:00Z","entries":[],"#,
            r#""manifest_digest":"{}","model":{{"model_dir":"models/qwen","model_id":"Qwen2"}},"#,
            r#""policy_packs":[],"runs":[],"schema":"vertrule.verify_manifest.v1","#,
            r#""suites":[],"version":"1.0"}}"#
        ),
        hex(0xab)
    );
    assert_eq!(canon(&m)?, golden.as_bytes());
    Ok(())
}

#[test]
fn signed_manifest_round_trip() -> Result<(), anyhow::Error> {
    let signed = SignedManifest::new(
        sample_manifest(),
        "2026-01-02T00:00:00Z".to_string(),
        "c2ln".to_string(),
        SchemaPublicKeyHex::new("aabb"),
        SchemaKeyId::new("key-1"),
    );
    let back: SignedManifest = serde_json::from_str(&serde_json::to_string(&signed)?)?;
    assert_eq!(back, signed);
    // Re-canonicalization is deterministic.
    assert_eq!(canon(&signed)?, canon(&signed)?);
    Ok(())
}

#[test]
fn deny_unknown_fields_rejected() {
    let json = format!(
        concat!(
            r#"{{"schema":"vertrule.verify_manifest.v1","version":"1.0","#,
            r#""created_at":"2026-01-01T00:00:00Z","model":{{"model_id":"Q","model_dir":"d"}},"#,
            r#""suites":[],"policy_packs":[],"runs":[],"entries":[],"#,
            r#""manifest_digest":"{}","surprise":true}}"#
        ),
        hex(0xab)
    );
    assert!(serde_json::from_str::<VerifyManifest>(&json).is_err());
}
