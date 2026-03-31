# vertrule-schemas v0.2.0 Release Notes

Canonical schema types for the VertRule receipt system. This crate is
**nouns only** — wire shapes, validated scalar types, and associated
constants.

## What ships

- `ReceiptEnvelope` — pure data struct, no methods
- `ReceiptType`, `BoundaryOrigin` — receipt discriminators
- `DigestBytes` — 32-byte digest with strict hex serde
- `IJsonUInt` — non-negative integer safe for I-JSON round-trip
- `CanonicalPayload` — float-guarded JSON payload
- `SchemaVersion` — version tag carrying identity triple (V1)
- `PolicyId`, `SchemaId` — opaque identifiers
- `RBHInvariant` — identity continuity constraint
- `ProjectsToReceiptEnvelope` — projection trait
- `DefinitionError` — validation errors
- `MriBatchPayload` — batch-aware MRI receipt payload with per-example vectors
- `GradientCouplingPayload` — gradient coupling profile payload
- `ReductionProvenance`, `ReductionMode`, `ReductionAxis`,
  `TokenReduction`, `BatchReduction` — reduction semantics for MRI payloads

## Changes from 0.1.x

- **Single schema version**: `SchemaVersion::V1` is the only supported
  version, with full-envelope commitment (`BLAKE3(JCS(envelope \ {event_hash}))`).
  The legacy payload-only commitment model and `SchemaVersion::V2` have been removed.
- **`ReceiptMetaV1` removed**: no longer part of the public surface.
- **Migration doc removed**: no migration path exists or is needed.
- **Package boundary cleaned**: internal files (`.claude/`, `.vr/`, `tooling/`,
  process docs) are excluded from the published crate.

## Boundary rule

| Crate | Allowed role |
|-------|-------------|
| `vr-jcs` | Canonicalization primitive |
| `vertrule-schemas` | Nouns / wire shapes / validated scalar types |
| `vertrule-verifier` | Judgment over public artifacts |
