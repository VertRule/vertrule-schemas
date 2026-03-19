# `vertrule-schemas`

Canonical wire-format types for VertRule receipt envelopes.

This crate defines the public verification contract: the shape of receipt
envelopes, digest types, receipt classifications, and boundary origins that
any verifier implementation must understand.

## Design Constraints

- Zero `vr-*` runtime dependencies
- Zero unsafe code
- No floating-point arithmetic
- Deterministic serialization (serde + strict hex)

## Public Types

| Type | Purpose |
|------|---------|
| `DigestBytes` | 32-byte cryptographic digest with strict lowercase hex serde |
| `ReceiptEnvelope` | Constitutional public receipt envelope |
| `ReceiptMetaV1` | Receipt metadata header |
| `ReceiptType` | Receipt classification discriminator (7 variants) |
| `BoundaryOrigin` | Boundary provenance discriminator (6 variants) |
| `PolicyId` | Opaque policy identifier |
| `RBHInvariant` | Identity continuity constraint (Receipt-Boundary Handshake) |
| `SchemaVersion` | Schema version tag |
| `DefinitionError` | Validation error type |

### Constants

- `ENVELOPE_VERSION_1` — current envelope version
- `DIGEST_ALGORITHM` — `"BLAKE3"`
- `DIGEST_HEX_LEN` — 64
- `DIGEST_BYTE_LEN` — 32
- `CANONICALIZATION` — `"JCS"`

## Usage

```rust
use vertrule_schemas::{DigestBytes, ReceiptEnvelope, ReceiptType, BoundaryOrigin};

// Parse a hex digest
let digest = DigestBytes::from_hex(
    "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
)?;

// Strict validation: rejects uppercase, wrong length, non-hex
assert!(DigestBytes::from_hex("A1B2...").is_err());
```

## Build

```bash
cargo build
cargo test
cargo clippy -- -D warnings
```

## License

Apache-2.0
