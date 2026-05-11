# vertrule-schemas v0.4.1 Release Notes

Additive release over v0.4.0. Adds semantic digest role newtypes that
distinguish digest commitments at compile time across adapter,
runtime-port, verifier, gateway, and receipt-store boundaries.

## What changed in v0.4.1

- **Semantic digest role newtypes** — `PolicyDigest`, `SchemaDigest`,
  `ContextDigest`, `ReceiptDigest`, `PayloadDigest`, and
  `ContentIdentityDigest` wrap `DigestBytes` with `#[serde(transparent)]`
  so wire bytes are identical to a bare digest while compile-time
  types prevent silent substitution between trust roles. Each
  exposes `::new(DigestBytes)`, `.bytes()`, `.to_hex()`, a custom
  prefix-truncated `Debug`, and `From<DigestBytes>`. Live in
  `src/common/semantic_digests.rs`; re-exported through
  `vertrule_schemas::*` and `vertrule_schemas::common::*`.
- **Internal governance metadata excluded from the published crate** —
  `Cargo.toml [package].exclude` drops `.claude/**`, `.github/**`,
  `.vr/**`, `operator_private/**`, `tooling/**`, `justfile`,
  `PUBLIC_SURFACE.md`, `RELEASE.md`, and all `*.capability.yaml`
  sidecars from the tarball.
- **Adapter governance internals expanded** — `src/governance/adapter.rs`
  and its test suite gained implementation/test coverage; public
  surface (`AdapterOriginId`, `AdapterReference`, and the six
  convenience constructors) is unchanged from v0.4.0. Additive only;
  no breaking removals.
- **`vr-jcs` dependency refreshed to `0.4.1`** — `Cargo.lock` now
  resolves the freshly-published `vr-jcs 0.4.1` (RFC 8785
  canonicalization); the `^0.4` constraint is unchanged.

## Public surface change classification

Additive. No `pub` items removed or renamed from v0.4.0. The
existing v0.4.0 root re-exports continue to compile unchanged.

## Receipt-digest stability

Semantic digest types are wire-transparent: a `ReceiptDigest` and a
bare `DigestBytes` carrying the same 32 bytes serialize to the same
canonical hex string, so existing receipts and stored digests remain
byte-stable across the v0.4.0 → v0.4.1 boundary.

---

# vertrule-schemas v0.3.0 Release Notes

Completes the `AdapterOrigin` → `AdapterOriginId` migration. Adapter
identity is now a single validated newtype; the legacy enum is gone.

## Changes from 0.2.4

- **`AdapterOrigin` enum removed.** All six former variants (`Jira`,
  `LangChain`, `ServiceNow`, `Salesforce`, `Slack`, `Custom(String)`)
  collapse to a single validated newtype `AdapterOriginId`.
- **`AdapterReference.adapter_origin` retyped** from `AdapterOrigin`
  to `AdapterOriginId` (`src/governance/adapter.rs`).
- **`GovernanceScope.adapter_origin` retyped** from `AdapterOrigin`
  to `AdapterOriginId` (`src/governance/scope.rs`).
- **Root re-exports updated**: `AdapterOrigin` removed from `lib.rs`
  and `governance/mod.rs`; `AdapterOriginId` added in its place.
- **Convenience constructors** on `AdapterOriginId` for the six
  stable identifiers: `::jira()`, `::lang_chain()`, `::service_now()`,
  `::salesforce()`, `::slack()`, `::webhook()`. Arbitrary
  grammar-valid identifiers go through `AdapterOriginId::new()`.
- **Obsolete migration-capture example removed**:
  `examples/capture_adapter_origin_fixtures.rs` deleted — its purpose
  (pre-migration byte-stability captures) is satisfied by the frozen
  fixtures at `fixtures/adapter_origin_migration/`.

## Breaking changes

All are wire-identical for unit variants (`"jira"`, `"lang_chain"`,
`"service_now"`, `"salesforce"`, `"slack"`) under both the old enum
serialization and the new `#[serde(transparent)]` newtype — same bytes,
same digests. The sole non-compatible form is:

- **`AdapterOrigin::Custom(String)` serialization → `AdapterOriginId`
  bare string.** Old wire form `{"custom":"x"}` is no longer accepted;
  the new form is `"x"`. Call sites must be updated to use
  `AdapterOriginId::new("x".to_string())?`. The value must satisfy the
  newtype grammar `[a-z][a-z0-9_]{0,63}` — arbitrary unvalidated
  strings are no longer admissible.
- **Compile-time**: callers that matched on `AdapterOrigin` variants
  (`match origin { AdapterOrigin::Jira => ..., ... }`) must switch to
  string-level dispatch: `match origin.as_str() { "jira" => ..., other
  => ... }`.
- **Root re-export gone**: `use vertrule_schemas::AdapterOrigin;` no
  longer compiles. Use `AdapterOriginId`.

## Receipt-digest stability

Receipts minted before this release whose `adapter_origin` serialized
as a unit variant (`"jira"`, `"lang_chain"`, etc.) remain
byte-identical under the new type — canonical bytes, `context_digest`,
`event_hash` are all preserved. No receipt-chain break for existing
Jira / LangChain / ServiceNow / Salesforce / Slack data.

Receipts that encoded the legacy `{"custom":"x"}` form are
**incompatible** and will fail to deserialize under 0.3.0. No such
receipts are known in production stores; a pre-publish scan of
`products/vertrule-gateway/gateway-store/` found zero occurrences.

## Boundary rule

| Crate | Allowed role |
|-------|-------------|
| `vr-jcs` | Canonicalization primitive |
| `vertrule-schemas` | Wire shapes, validated scalars, commitment support |
| `vertrule-verifier` | Judgment over public artifacts |

---

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
