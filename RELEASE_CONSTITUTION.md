# vertrule-schemas Release Constitution

Version: draft-1
Date: 2026-03-27

## Purpose

This document defines the contract scope of `vertrule-schemas` as a public
standard substrate. It gates the refactoring and publication decision.

The test is: can a third party clone `vr-jcs`, `vertrule-schemas`, and
`vertrule-verifier`, build everything with no private runtime dependency,
and verify receipts?

---

## Section 1: What vertrule-schemas Owns

### Constitutional Primitives

| Type | Role | Invariant |
|------|------|-----------|
| `DigestBytes` | 32-byte cryptographic digest | Hex-validated, ordered, hashable |
| `SchemaVersion` | Identity triple binding | `(spec_version, canonicalization, commitment_primitive)` |
| `SchemaId` | Schema identifier | Grammar: `vr.<domain>.<name>@<major>.<minor>` |
| `IJsonUInt` | Interoperable integer | `[0, 2^53 - 1]`, round-trips through any JSON parser |
| `CanonicalPayload` | Float-guarded JSON value | Rejects floats at all nesting depths |
| `RBHInvariant` | Identity continuity constraint | Three `DigestBytes` binding parent→policy→receipt |

### Public Contract Types

| Type | Role | Invariant |
|------|------|-----------|
| `ReceiptEnvelope` | **The** public receipt envelope | Single canonical root for all proof-bearing receipts |
| `ReceiptType` | Top-level discriminator | `#[non_exhaustive]` enum: Event, Llm, Mri, Governance, Adapter, Projection, Training |
| `BoundaryOrigin` | Boundary provenance | `#[non_exhaustive]` enum: Engine, Adapter, Numeric, Governance, Model, Training |

### Validation Types

| Type | Role |
|------|------|
| `PolicyId` | Opaque policy identifier, grammar-validated |
| `DefinitionError` | Construction-time validation errors |

### Projection Law

Every proof-bearing receipt across all VertRule repositories MUST project
to `vertrule_schemas::ReceiptEnvelope`:

```
∀ r ∈ proof-bearing receipts: project(r) → ReceiptEnvelope
```

Internal receipt representations may differ, but the public projection
target is this one type. No competing envelope roots.

---

## Section 2: What vertrule-schemas Does Not Own

### Canonicalization (owned by vr-jcs)

The `jcs` module currently in vertrule-schemas MUST be replaced by a
dependency on `vr-jcs`. Canonicalization is algorithmic infrastructure,
not a domain noun.

| Current | Target |
|---------|--------|
| `vertrule_schemas::jcs::to_canon_bytes()` | `vr_jcs::to_canon_bytes()` |
| `vertrule_schemas::jcs::to_canon_string()` | `vr_jcs::to_canon_string()` |
| `vertrule_schemas::jcs::JcsError` | `vr_jcs::JcsError` |
| Internal `zmij` dependency | Moves to vr-jcs |

vertrule-schemas re-exports from vr-jcs for backward compatibility
during migration. The re-export is marked deprecated and removed in
the next major version.

### Runtime DTOs

These do not belong in the public contract substrate:

| Type | Disposition |
|------|-------------|
| `ReceiptMetaV1` | Move to runtime-private crate or remove from public API |

### Types That Must Not Enter

| Category | Examples | Reason |
|----------|----------|--------|
| CLI/MCP request shapes | PathBuf arguments, tool request types | Transport-specific |
| Archive/package manifests | Bundle formats, manifest schemas | Packaging-specific |
| Runtime policy structs | Execution policies, streaming policies | Implementation-specific |
| Float-bearing boundary DTOs | Unless explicit encoding semantics defined | Nondeterminism risk |
| Private runtime envelopes | `UnifiedReceiptEnvelope`, internal bridges | Competing roots |

---

## Section 3: Structural Rules

### Rule 1: One Envelope Root

There is exactly one canonical public receipt envelope:
`vertrule_schemas::ReceiptEnvelope`.

**Current violation**: `vertrule-runtime/crates/vr-verifier/src/envelope.rs`
defines a competing local `ReceiptEnvelope` with weaker typing
(`envelope_version: u32`, `receipt_type: String`, `payload: Value`).

**Required correction**: The runtime vr-verifier MUST consume
`vertrule_schemas::ReceiptEnvelope` or be retired in favor of the
standalone `vertrule-verifier`.

### Rule 2: One Digest/ID Root

Cross-repo identity types live in vertrule-schemas:

- `DigestBytes` — the canonical digest representation
- `SchemaId` — the canonical schema identifier
- `SchemaVersion` — the canonical version tag

No other crate may define competing public types for these roles.

### Rule 3: Verifier Closure

`vertrule-verifier` depends only on:

- `vertrule-schemas` (contract types)
- `vr-jcs` (canonicalization, via vertrule-schemas re-export or direct)
- Cryptographic primitives (`blake3`, `ed25519-dalek`)
- Serialization (`serde`, `serde_json`)

No runtime crate. No adapter crate. No application crate.

This is already true for the standalone verifier. It must remain true.

### Rule 4: Extension Direction

Public-but-non-constitutional schemas extend vertrule-schemas:

```
public-extension → vertrule-schemas
```

Never the reverse. vertrule-schemas has zero dependencies on extension
or runtime crates.

### Rule 5: Float Policy

No public type in vertrule-schemas exposes `f32` or `f64` fields.

`CanonicalPayload` enforces this at construction. The crate-level lints
deny float arithmetic. This is already true and must remain true.

If a future public schema requires numeric precision, it must define
explicit deterministic encoding (canonical decimal string, fixed-point
integer, or bit-level IEEE representation as hex/bytes).

### Rule 6: Generated Consumers

Website fixtures, demo JSON, and any non-Rust consumer MUST be generated
from the canonical verifier pipeline or consumed through the WASM
verifier. No hand-maintained copies.

---

## Dependency Graph (Target State)

```
vr-jcs                          ← zero vr-* deps, only serde + serde_json
  ↑
vertrule-schemas                ← depends on vr-jcs, serde, serde_json, hex, thiserror
  ↑
vertrule-verifier               ← depends on vertrule-schemas, blake3, ed25519-dalek
  ↑
[website WASM]                  ← vertrule-verifier compiled to wasm32
[conformance test suite]        ← vertrule-verifier + test vectors
```

No arrow points downward. No runtime crate appears.

---

## Migration Checklist

- [ ] Upgrade vr-jcs to full RFC 8785 (UTF-16 sorting, noncharacter rejection, duplicate key rejection)
- [ ] Move `zmij` dependency from vertrule-schemas to vr-jcs
- [ ] Replace vertrule-schemas `jcs` module with vr-jcs dependency + deprecated re-export
- [ ] Remove or relocate `ReceiptMetaV1` from public API
- [ ] Retire runtime vr-verifier competing ReceiptEnvelope
- [ ] Publish conformance test vectors alongside vertrule-verifier
- [ ] Build WASM verifier target
- [ ] Generate website fixtures from verifier pipeline

---

## Falsifiable Exit Criteria

A stranger can:

1. `cargo add vr-jcs vertrule-schemas` — no private deps pulled
2. Construct a `ReceiptEnvelope`, canonicalize its payload, compute its digest
3. `cargo add vertrule-verifier` — verify the receipt
4. Reject a tampered receipt (modified payload, wrong digest)
5. Reject malformed canonicalization (non-JCS ordering)
6. Reject schema/version mismatch
7. Get identical verdicts from CLI verifier and website WASM verifier
8. Run the full conformance test suite with `cargo test`

If any step fails, the publication is not ready.
