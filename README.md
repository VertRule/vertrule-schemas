# `vertrule-schemas`

Canonical wire-format types for VertRule receipt envelopes.

This crate defines the public verification contract: the shape of receipt
envelopes, digest types, receipt classifications, and boundary origins that
any verifier implementation must understand.

## Design Constraints

- Single `vr-*` dependency: [`vr-jcs`](https://crates.io/crates/vr-jcs) (JCS canonicalization primitive)
- Zero unsafe code
- No floating-point values in trust-critical payloads
- RFC 8785 canonical JSON for digest / signature inputs

## Public Types

### Core

| Type | Purpose |
|------|---------|
| `DigestBytes` | 32-byte cryptographic digest with strict lowercase hex serde |
| `IJsonUInt` | Non-negative integer guaranteed to round-trip in I-JSON |
| `CanonicalPayload` | Float-guarded JSON payload |
| `PolicyId` | Opaque policy identifier |
| `SchemaId` | Validated schema identifier (`vr.<domain>.<name>@<major>.<minor>`) |
| `SchemaVersion` | Schema version tag (carries identity triple) |
| `DefinitionError` | Validation error type |

### Receipts

| Type | Purpose |
|------|---------|
| `ReceiptEnvelope` | Constitutional public receipt envelope |
| `ReceiptMetaV1` | Receipt metadata header |
| `ReceiptType` | Receipt classification discriminator (7 variants) |
| `BoundaryOrigin` | Boundary provenance discriminator (6 variants) |
| `ProjectsToReceiptEnvelope` | Canonical projection trait |

### Context

| Type | Purpose |
|------|---------|
| `RBHInvariant` | Identity continuity constraint (Receipt-Boundary Handshake) |

### MRI Domain

| Type | Purpose |
|------|---------|
| `MriBatchPayload` | Batch-aware MRI invariant payload |
| `GradientCouplingPayload` | Gradient coupling diagnostic payload |
| `ReductionProvenance` | Reduction pipeline provenance |
| `ReductionMode` | Batch reduction strategy |
| `ReductionAxis` | Tensor axis discriminator |
| `TokenReduction` | Token aggregation method |
| `BatchReduction` | Batch aggregation method |

### Associated Constants

| Constant | Value |
|----------|-------|
| `DigestBytes::BYTE_LEN` | 32 |
| `DigestBytes::HEX_LEN` | 64 |
| `SchemaVersion::V1` | Spec version 1 (payload-only commitment) |
| `SchemaVersion::V2` | Spec version 2 (full-envelope commitment) |
| `SchemaVersion::digest_algorithm()` | `"BLAKE3"` |
| `SchemaVersion::canonicalization()` | `"JCS"` |

## Usage

```rust
use vertrule_schemas::DigestBytes;

// Parse a hex digest
let digest = DigestBytes::from_hex(
    "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
)?;

// Strict validation: rejects uppercase, wrong length, non-hex
assert!(DigestBytes::from_hex("A1B2...").is_err());
```

JCS canonicalization is provided by [`vr-jcs`](https://crates.io/crates/vr-jcs)
(a direct dependency of this crate, but not re-exported):

```rust
use vr_jcs::to_canon_string;
use serde_json::json;

let canon = to_canon_string(&json!({"z": 1, "a": 2}))?;
assert_eq!(canon, r#"{"a":2,"z":1}"#);
```

## Build

```bash
cargo build
cargo test
cargo clippy -- -D warnings
```

## License

Apache-2.0
