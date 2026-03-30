# vertrule-schemas v0.1.0 Release Notes

Canonical schema types for the VertRule receipt system. This crate is
**nouns only** — wire shapes, validated scalar types, and associated
constants.

## What ships

- `ReceiptEnvelope` — pure data struct, no methods
- `ReceiptMetaV1`, `ReceiptType`, `BoundaryOrigin` — receipt discriminators
- `DigestBytes` — 32-byte digest with strict hex serde
- `IJsonUInt` — non-negative integer safe for I-JSON round-trip
- `CanonicalPayload` — float-guarded JSON payload
- `SchemaVersion` — version tag carrying identity triple (V1, V2)
- `PolicyId`, `SchemaId` — opaque identifiers
- `RBHInvariant` — identity continuity constraint
- `ProjectsToReceiptEnvelope` — projection trait
- `DefinitionError` — validation errors
- `MriBatchPayload` — batch-aware MRI receipt payload with per-example vectors
- `GradientCouplingPayload` — gradient coupling profile payload
- `ReductionProvenance`, `ReductionMode`, `ReductionAxis`,
  `TokenReduction`, `BatchReduction` — reduction semantics for MRI payloads

## Key decisions

- **Nouns-only boundary**: `vertrule-schemas` contains no construction
  logic, no hashing, no validation judgments. Construction belongs to
  producer crates; integrity validation belongs to `vertrule-verifier`.
- **JCS not re-exported**: consumers depend on `vr-jcs` directly.
  The internal `jcs` module is `pub(crate)`.
- **`compute_event_hash` scoped**: available at
  `vertrule_schemas::receipts::compute_event_hash`, not root-exported.
  It is receipt-commitment behavior tied to the schema version's
  identity triple.
- **`ReceiptEnvelope::new` removed**: no constructor method on the type.
  Producers build the struct directly and call `compute_event_hash`.
- **`validate_integrity` removed**: re-homed as
  `validate_receipt_envelope_integrity` in `vertrule-verifier`.

## Boundary rule

| Crate | Allowed role |
|-------|-------------|
| `vr-jcs` | Canonicalization primitive |
| `vertrule-schemas` | Nouns / wire shapes / validated scalar types |
| `vertrule-verifier` | Judgment over public artifacts |
