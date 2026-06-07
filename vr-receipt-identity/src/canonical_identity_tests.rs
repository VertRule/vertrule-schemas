//! Layer A G2: the new owner's `digest_trusted_value` copy must produce
//! the committed golden digests, byte-equivalent to the schemas / verifier
//! / crypto copies. Source of truth:
//! docs/audits/junk-drawer-inventory/fixtures/receipt-identity/goldens.json

use super::digest_trusted_value;
use crate::error::ReceiptIdentityError;
use vr_jcs::DigestStrategy;

#[test]
fn g2_blake3_untagged_helper_matches_goldens() -> Result<(), ReceiptIdentityError> {
    let strategy = DigestStrategy::blake3_untagged();
    let cases: [(serde_json::Value, &str); 5] = [
        (
            serde_json::json!({"a": 1, "b": 2}),
            "8e80439b77ac62d4194499edd46684c479da3aa1ac80dd5511468efae049166e",
        ),
        (
            // unsorted keys must equal v_plain — proves JCS key ordering
            serde_json::json!({"b": 2, "a": 1}),
            "8e80439b77ac62d4194499edd46684c479da3aa1ac80dd5511468efae049166e",
        ),
        (
            serde_json::json!({"z": [3, 1, 2], "a": {"k": "v"}}),
            "5ef47de6cdb1c8586547526ee1fb7726321452f65ce50ba1abef1d3bf650a08c",
        ),
        (
            serde_json::json!({"n": 9_007_199_254_740_991_i64}),
            "6f3adc03614205e4ef7d378c51d584a691c60baa2abcdfea5325018261a28fb6",
        ),
        (
            serde_json::json!({"s": "café\n\"q\""}),
            "770f998755f9ac91974ea4dc2e23d34144f5cd0ad3238c3403a0a1e797c26a3a",
        ),
    ];

    for (value, expected) in &cases {
        let digest = digest_trusted_value(value, &strategy)?;
        assert_eq!(
            hex::encode(&digest.bytes),
            *expected,
            "vr-receipt-identity digest_trusted_value drifted from golden"
        );
    }
    Ok(())
}
