# .vr/ — Repository State Root

Canonical repo-local state directory for the `vertrule-schemas` repository.

Governed by Repo State Standard v1.

## Canonical Layout

```
.vr/
  README.md
  governance/
    bindings/
    overlays/
    manifest.toml
    known-nondeterminism.toml
  receipts/
    chain-manifest.json
    governance/
      schema/
  state/
  public/
  tmp/
```

## Invariants

1. Exactly one canonical state root: `.vr/`
2. Governance definitions and receipts are strictly separated.
3. No mutable operational files at `.vr/` root level.
4. All path resolution uses `.vr/` prefix, never bare `governance/`.

## Governance Status

| Property | Value |
|----------|-------|
| Governance tier | Constitutional schema layer |
| Determinism stage | 0 — structural |
| Receipt chain | Genesis — no governance receipts produced |
| Authority set | Development — keys derived from plaintext hashing, not managed custody |

### Active Policy Bindings

| Policy | Mode | Overlay |
|--------|------|---------|
| `determinism@0.1` | bind+overlay | `overlays/determinism@0.1.toml` (strict: `src/**/*.rs`, no exceptions) |
| `repo-boundary@0.1` | bind | No overlay needed |

### Binding Resolution

Policy and authority-set bindings reference external governance infrastructure
by BLAKE3 digest rather than by file path. This means:

- The **digest** field in each binding is the BLAKE3 hash of the canonical
  source file (policy.toml or authority-set YAML) in the VertRule shared
  governance infrastructure.
- The **overlay** (if any) is bundled in this repository under
  `.vr/governance/overlays/` and is fully inspectable from a fresh clone.
- The digest is an **anchor**, not a self-contained proof. To verify the
  binding against source material, you need access to the governance
  infrastructure. Without it, the digest serves as a tamper-evident seal:
  if the policy changes, the digest will no longer match.

### Governance Surface Validation

The following governance files are validated in CI by `validate-governance-surface`:

- `.vr/governance/manifest.toml` — version must match `Cargo.toml`, policy IDs
  must be consistent with `policy-set.json`
- `.vr/governance/bindings/authority-set.json` — required fields, valid BLAKE3
  digest format, no legacy `source` field, allowed grade values
- `.vr/governance/bindings/policy-set.json` — no duplicate policy IDs, no legacy
  `source` fields, overlay paths must resolve, valid digest format
- `.vr/receipts/chain-manifest.json` — valid chain status, genesis constraints
  enforced (receipt_count=0, no chain_root)

This validates local structure and semantics only. It does not verify upstream
governance sources or imply that conforming governance receipts exist.

Run locally: `just governance-check`

### What can be verified from a fresh clone

- Code builds and all tests pass: `cargo test`
- Determinism tests pass: `cargo test --test determinism_tests`
- Test vector validation passes: `cargo test --test test_vector_validation`
- Repo state structure is valid: `tooling/scripts/check-repo-state.sh`
- Governance surface is valid: `just governance-check`
- Policy binding digests in `manifest.toml` can be compared against known policy hashes
- No nondeterminism sources: `known-nondeterminism.toml` is empty

### What cannot be verified from a fresh clone

- No governance receipts exist to verify (chain-manifest.json is in genesis state)
- Authority set binding references external governance infrastructure
  (the digest is committed but the source material is not bundled)
- No signature-backed governance evidence has been produced for this repository
- `GovernanceReceipt.schema.json` defines the target receipt shape but no conforming
  receipts exist in this repository
