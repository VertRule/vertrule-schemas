# vertrule-schemas v0.2.1 Release Notes

Rigor and hardening follow-up to v0.2.0. No new public types.

## Changes from 0.2.0

- **Integrated dormant JCS compliance tests**: `src/jcs_tests.rs` was
  orphaned and never compiled. Now included in the active test tree
  (+14 RFC 8785 tests).
- **Normalized `CanonicalPayload` constructor errors**:
  `CanonicalPayload::new()` returns `Result<Self, DefinitionError>`
  instead of `Result<Self, String>`.
- **Hardened public type evolution posture**: `ReceiptEnvelope` and all
  MRI public types (`MriBatchPayload`, `GradientCouplingPayload`,
  `ReductionProvenance`, `ReductionMode`, `ReductionAxis`,
  `TokenReduction`, `BatchReduction`) are now `#[non_exhaustive]`.
- **MRI schema fields use `SchemaId`**: `MriBatchPayload::schema` and
  `GradientCouplingPayload::schema` are now validated `SchemaId` instead
  of bare `String`. Wire format changes from `mri2.*` to `vr.mri.*`.
- **MRI doc invariants clarified**: length constraints on per-example
  vectors are documented as producer obligations, not type guarantees.
- **`compute_event_hash` hardened**: silent `if let` replaced with
  explicit `let...else` failure path.
- **Frozen known-answer test**: `compute_event_hash` output is pinned
  against a specific hex digest.
- **Unknown-field rejection test**: proves `deny_unknown_fields` on
  `ReceiptEnvelope`.
- **Algorithm marker ownership documented**: `digest_algorithm` and
  `canonicalization` fields are explicitly documented as verifier-validated.
- **Unused error variants documented**: `MarkerMismatch` and
  `IntegrityViolation` are documented as downstream contract types.
- **`SchemaId` doctest cleaned**: `.unwrap()` replaced with `?`.
- **Public docs tightened**: removed overclaims about "nouns only / no
  hashing" — crate now accurately describes itself as providing
  protocol-scoped commitment support.

## Breaking changes

- `CanonicalPayload::new()` returns `DefinitionError` instead of `String`.
- `ReceiptEnvelope` is `#[non_exhaustive]` — external struct literal
  construction no longer compiles. Use deserialization or a builder.
- `MriBatchPayload::schema` and `GradientCouplingPayload::schema` are
  `SchemaId` instead of `String`. Wire identifiers change from
  `mri2.batch_invariant@0.1` to `vr.mri.batch_invariant@0.1`.
- MRI enums and structs are `#[non_exhaustive]`.

## Boundary rule

| Crate | Allowed role |
|-------|-------------|
| `vr-jcs` | Canonicalization primitive |
| `vertrule-schemas` | Wire shapes, validated scalars, commitment support |
| `vertrule-verifier` | Judgment over public artifacts |
