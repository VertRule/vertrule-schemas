# vertrule-schemas Public Surface (v0.1)

Canonical schema types for the VertRule receipt system.
This crate contains **nouns only** — wire shapes, validated scalar
types, and associated constants. No construction logic, no hashing,
no validation judgments.

## Governing Rule

| Crate | Allowed role |
|-------|-------------|
| `vr-jcs` | Canonicalization primitive |
| `vertrule-schemas` | Nouns / wire shapes / validated scalar types |
| `vertrule-verifier` | Judgment over public artifacts |

## Stable Root Exports

```rust
// Wire shapes
pub struct ReceiptEnvelope { .. }   // Pure data, no methods
pub struct ReceiptMetaV1 { .. }

// Discriminators
pub enum ReceiptType { .. }
pub enum BoundaryOrigin { .. }

// Validated scalars
pub struct DigestBytes { .. }       // BYTE_LEN = 32, HEX_LEN = 64
pub struct IJsonUInt { .. }
pub struct CanonicalPayload { .. }
pub struct PolicyId { .. }
pub struct SchemaId { .. }

// Version tag (carries identity triple)
pub struct SchemaVersion { .. }     // V1, V2, digest_algorithm(), canonicalization()

// Context constraint
pub struct RBHInvariant { .. }

// Projection trait
pub trait ProjectsToReceiptEnvelope { .. }

// Error
pub enum DefinitionError { .. }
```

## Scoped Exports (not root-exported)

Available via submodule path for sibling crates:

```rust
// Receipt commitment (behavior tied to schema contract)
vertrule_schemas::receipts::compute_event_hash(&ReceiptEnvelope) -> Result<DigestBytes, JcsError>
```

## Not Exported from This Crate

The following live in their respective crates, not here:

| Symbol | Home | Rationale |
|--------|------|-----------|
| JCS functions (`to_canon_bytes`, etc.) | `vr-jcs` | Canonicalization execution |
| Receipt construction | Producer crate | Construction is a procedure |
| Envelope integrity validation | `vertrule-verifier` | Judgment over nouns |
