# VertRule-Schemas v1 Publishing Review Checklist

Repository: `vertrule-schemas`  
Date: 2026-03-19  
Purpose: truth-and-verification gate for whether `vertrule-schemas` is ready to publish as the public contract for receipts, envelopes, and verifier-facing artifacts.

This is not a generic schema-release checklist. It is the VertRule-specific publication gate, adapted for the sealed-context model, adapter receipts, semantic-graph/state-digest posture, and dual-persistence verifier model.

## Current Audit Status

### Release Blockers

- [ ] Fix JCS key ordering to match RFC 8785 UTF-16 code unit ordering.
  Current issue: `src/jcs.rs` sorts Rust `String`s directly, which diverges from RFC ordering for some valid Unicode keys.
  Minimum exit criteria:
  - canonicalization sorts object keys by UTF-16 code units
  - regression tests cover known mismatch cases such as `"\u{10000}"` vs `"\u{E000}"`
  - docs stop claiming RFC 8785 compliance until the implementation actually matches it

- [ ] Harden schema strictness at the serde and JSON Schema boundaries.
  Current issue: `ReceiptEnvelope` and `ReceiptMetaV1` accept unknown fields, and `.vr/receipts/governance/schema/GovernanceReceipt.schema.json` does not close objects with `additionalProperties: false`.
  Minimum exit criteria:
  - add `#[serde(deny_unknown_fields)]` to public receipt structs where appropriate
  - add schema-level `additionalProperties: false` to top-level and nested object definitions that are meant to be closed
  - add regression tests proving unknown fields are rejected

- [ ] Restore the advertised lint gate for all targets.
  Current issue: `cargo clippy --all-targets -- -D warnings` fails in `src/canonical_payload_tests.rs`.
  Known failures:
  - `clippy::approx_constant` on `3.14`
  - `clippy::map_identity` on `map_err(|e| e)`
  Minimum exit criteria:
  - `cargo fmt --check`
  - `cargo test -q`
  - `cargo clippy --all-targets -- -D warnings`
  all pass locally

- [ ] Add real CI for release-readiness checks.
  Current issue: the only GitHub Actions workflow enforces attribution text; it does not run format, test, or clippy gates.
  Minimum exit criteria:
  - CI runs `cargo fmt --check`
  - CI runs tests
  - CI runs `cargo clippy --all-targets -- -D warnings`

### Open Contract Decisions

- [ ] Decide whether enum inputs must be canonical on ingest.
  Current issue: `ReceiptType` and `BoundaryOrigin` accept uppercase input and normalize it to lowercase on serialization.
  Decision required:
  - reject non-canonical casing and preserve byte stability, or
  - explicitly document that parsing is lenient and serialization canonicalizes
  Recommendation: reject non-canonical casing for public wire-format types.

- [ ] Resolve release metadata before publication.
  Current issue: `.vr/governance/manifest.toml` still marks `extraction_ready = false`.
  Minimum exit criteria:
  - update the manifest if the crate is truly publication-ready, or
  - document why public visibility does not imply extraction readiness

### Publication Hygiene

- [ ] Move ignore rules for local/private artifacts into shared repo config or move the artifacts out of the worktree.
  Current issue:
  - `.DS_Store` is currently untracked and not ignored by the repo
  - `operator_private/` is only excluded via `.git/info/exclude`
  - `.claude/settings.local.json` is ignored via machine-local global gitignore, not shared repo config
  Minimum exit criteria:
  - repo-level ignore rules cover expected local junk
  - private review material cannot be accidentally committed from a clean clone

- [ ] Do a final publication scrub for non-public files and naming drift.
  Notes:
  - current remote is `git@github.com:VertRule/vertrule-definitions.git` while the crate name is `vertrule-schemas`
  - confirm public-facing naming is intentional before publication

### Validation Snapshot

- [x] `cargo fmt --check`
- [x] `cargo test -q`
- [x] `cargo nextest run`
- [ ] `cargo clippy --all-targets -- -D warnings`

## Publishing Review Gate

### 1. Problem and release boundary are explicit

- [ ] The repo states, in the first screenful, that these schemas define the portable receipt/verifier contract for governed computation.
- [ ] It is explicit that the schemas exist to support:
  - deterministic receipt generation
  - replay and integrity verification
  - public and private dual-persistence boundaries
  - offline verification without re-executing nondeterministic work
- [ ] The repo clearly distinguishes what is in scope for v1:
  - receipt envelopes
  - core receipt payloads
  - chain and linkage objects
  - context identity and policy identity references
  - adapter receipt bindings
  - verifier-visible verdict and result objects
- [ ] The repo clearly states what is not in scope for v1:
  - runtime implementation details
  - storage engine specifics
  - transport protocols
  - every future receipt family
  - every experiment-specific schema
- [ ] The scope is narrow enough that an outside implementer can tell what they may safely depend on.

### 2. Core design principles are written down

- [ ] The repo defines the governing principles explicitly:
  - determinism first
  - canonicalization before hashing
  - one field, one meaning
  - receipts are evidence, not prose
  - verifier-facing surfaces must be stable
  - no silent projection or semantic collapse
- [ ] Required vs optional vs extension fields are unambiguous.
- [ ] The repo states that schema evolution must preserve verifier clarity and digest stability expectations where promised.
- [ ] The repo states whether schemas are execution-facing, storage-facing, verifier-facing, or interchange-facing.

### 3. Canonical object model is actually settled

- [ ] The top-level objects are stable enough for a `v1.0.0` tag.
- [ ] The repository has a coherent model for:
  - `ReceiptEnvelope`
  - payload or body object
  - schema identifier
  - digest-bearing fields
  - lineage, predecessor, or chain linkage
  - context, policy, or execution identity references
  - adapter evidence bindings
  - validation and verdict objects
  - failure, denial, or policy-block objects where applicable
- [ ] The relationship between core receipts and adapter receipts is explicit.
- [ ] The model for semantic-state proof is represented honestly:
  - whether the schema carries a state-transition digest
  - whether MSS or TMS support is in payload or only referred to
  - whether the verifier is expected to reconstruct or only validate digests

### 4. Envelope semantics are frozen enough for outsiders

- [ ] `ReceiptEnvelope` semantics are fully documented.
- [ ] The repo states exactly:
  - what the envelope wraps
  - what is inside the payload
  - what is hashed
  - what is signed
  - what is merely metadata
  - what fields are verifier-critical
- [ ] There is no ambiguity between:
  - payload value
  - canonical payload form
  - digest of payload
  - digest of envelope
  - signature target
- [ ] If `CanonicalPayload` is now the contract instead of raw JSON-like payloads, that is reflected everywhere, including examples, verifier docs, and tests.

### 5. Canonicalization and hashing rules are brutally explicit

- [ ] The repo states the canonical serialization scheme used before hashing.
- [ ] The digest algorithm is named explicitly.
- [ ] The byte domain of every digest is defined:
  - raw payload bytes
  - canonical JSON bytes
  - envelope bytes
  - domain-separated tagged bytes
- [ ] Field ordering rules are documented.
- [ ] Unicode and string normalization expectations are documented.
- [ ] There is zero room for two independent implementations to hash different byte sequences from the same logical object.
- [ ] If JCS or equivalent canonical ordering is required, the repo says so plainly and tests it.

### 6. Determinism claims are schema-supported, not hand-waved

- [ ] Every schema field that affects replay or verification is deterministic by construction.
- [ ] The repo bans ambiguous types in digest-critical surfaces:
  - unordered maps without canonicalization
  - floats where stable decimal strings or bit-wrapped forms are required
  - timestamps without clearly defined semantics
  - nullable fields whose omission, null, or empty behavior is not fixed
- [ ] If timestamps exist, the repo says whether they are:
  - observational metadata only
  - excluded from identity digests
  - included in a canonical identity
- [ ] If logical clocks or counters are expected instead of wall-clock time, that is stated.

### 7. Field-level semantics are complete

- [ ] For each public schema field, the docs provide:
  - name
  - type
  - meaning
  - whether required
  - whether nullable
  - canonical example
  - whether it participates in digest identity
  - whether it participates in signature verification
  - verifier interpretation rules
  - migration risk if changed

### 8. Schema identity and versioning are first-class

- [ ] Every schema has a stable identifier.
- [ ] The repository documents how schema IDs are formed.
- [ ] Major and minor version policy is explicit.
- [ ] Breaking vs non-breaking changes are defined in verifier terms, not marketing terms.
- [ ] The repo states whether:
  - adding an optional field is non-breaking
  - tightening constraints is breaking
  - changing canonicalization is always breaking
  - renaming fields is breaking
  - changing digest participation is absolutely breaking
- [ ] A deprecation policy exists before release.

### 9. Receipt lineage and chain linkage are defined precisely

- [ ] The repository specifies how predecessor linkage works.
- [ ] Ordered subreceipt lists are defined where applicable.
- [ ] The difference between any public ladder levels is spelled out:
  - block receipt
  - projection receipt
  - layer receipt
  - decode-step receipt
  - other ladder levels
- [ ] The repo documents whether ordering is semantically meaningful.
- [ ] If ordering is meaningful, it is stable and tested.

### 10. Verifier contract is publishable on its own

- [ ] An external implementer can write a verifier from the repo alone.
- [ ] The repo explains:
  - what a verifier must check
  - what a verifier may ignore
  - what constitutes pass or fail
  - what constitutes malformed vs invalid
  - what data is required for offline verification
- [ ] There is a minimal end-to-end verification example.
- [ ] There is at least one bad receipt example showing why validation fails.

### 11. Context identity is explicit and non-magical

- [ ] The schemas define how context identity is represented:
  - context digest
  - policy digest
  - schema digest
  - authority-set or environment references if exposed
- [ ] The docs clearly distinguish:
  - parameters bound into context identity
  - dynamic session parameters excluded from it
- [ ] The repo avoids implying that context is a vague blob.

### 12. Adapter-bound nondeterminism is represented honestly

- [ ] If adapter receipts are public surfaces in v1, the repo defines:
  - adapter class or kind
  - canonical input representation
  - canonical output representation
  - content identity or digest
  - binding relationship to core receipts
- [ ] The docs make clear that adapter receipts freeze nondeterministic interactions into deterministic evidence for the core system.
- [ ] The schemas do not leak raw nondeterministic state unless that is deliberate and documented.

### 13. Failure semantics are first-class

- [ ] Policy denial, invalid receipt, malformed envelope, digest mismatch, signature failure, missing subreceipt, and schema-version mismatch all have defined representations or defined verifier behaviors.
- [ ] The repo does not force downstream consumers to reverse-engineer errors from absence.
- [ ] If there is a verifier result schema, it includes machine-usable failure classes.

### 14. Examples are real and strategically chosen

- [ ] The examples cover:
  - minimal valid envelope
  - fully populated valid receipt
  - receipt with adapter binding
  - chained receipt or predecessor linkage
  - policy-denied or blocked operation receipt
  - malformed example
  - invalid-digest example
  - version-mismatch example
  - verifier result example
- [ ] Every example validates against the schemas and aligns with digest and signature rules.

### 15. Negative tests are serious

- [ ] CI validates positive examples.
- [ ] CI rejects negative fixtures.
- [ ] Regression tests exist for previously discovered breakages.
- [ ] There is a test specifically guarding the payload and envelope type boundary that caused the `CanonicalPayload` breakage in the verifier context.
- [ ] There are golden tests for:
  - canonical serialization
  - digest stability
  - example fixture validity
  - cross-language byte identity if claimed

### 16. Schema and reference split is clean

- [ ] The repo distinguishes clearly between:
  - normative schema files
  - human-readable explanations
  - examples
  - generated artifacts
- [ ] Generated files, if any, are reproducible.
- [ ] There is no documentation drift between schema definitions and prose.
- [ ] There are no duplicate competing definitions of the same object in separate files.

### 17. Repository structure is release-grade

- [ ] The repo has predictable directories, such as:
  - `schemas/`
  - `examples/`
  - `tests/`
  - `docs/`
  - `changes/` or `CHANGELOG.md`
- [ ] There are no scratch notes.
- [ ] There are no internal TODOs that undermine v1 confidence.
- [ ] There is no draft language left on files intended to be treated as stable.

### 18. Licensing and patent posture are not muddy

- [ ] A license file exists.
- [ ] The repo states the intended adoption posture.
- [ ] If there are patent-sensitive aspects around the broader VertRule system, the schema repo avoids accidental ambiguity.
- [ ] Users can tell whether they can implement the verifier and schema surfaces freely.
- [ ] Documentation and examples licensing is clear if separate.

### 19. Privacy and disclosure boundaries are documented

- [ ] The repo states that receipt examples are synthetic.
- [ ] The repo states whether payloads may contain sensitive data, digests only, or references.
- [ ] Redaction guidance exists if receipts may contain customer, system, or user data.
- [ ] Public-log vs private-ledger expectations are documented, matching VertRule's dual-persistence architecture.

### 20. Security posture is sane

- [ ] There are no secrets in examples.
- [ ] There is no hidden signing material in the repo.
- [ ] Signature examples use clearly fake keys.
- [ ] The docs warn that schema validity is not the same as truth.
- [ ] The docs warn that canonical-looking receipts from untrusted producers still require digest, signature, and policy verification.

### 21. Contributor and governance basics exist

- [ ] `CONTRIBUTING` exists.
- [ ] `CODE_OF_CONDUCT` exists.
- [ ] Issue templates exist for:
  - schema bug
  - schema clarification
  - breaking-change proposal
  - new receipt family request
  - example request
- [ ] Maintainer decision rules are written down.
- [ ] The repo says how a proposed schema change becomes accepted.

### 22. Boring names test passes

- [ ] Names are stable, unsurprising, and not overfit to internal jargon unless essential.
- [ ] Similar objects are clearly distinguished:
  - envelope vs payload
  - digest vs hash pointer
  - context ID vs run ID
  - schema ID vs receipt type
  - verifier result vs runtime receipt
- [ ] Abbreviations are minimized or centrally defined.

### 23. Public docs answer the real outsider questions

- [ ] The README answers, quickly:
  - what this repo is
  - why it exists
  - what problem it solves that plain logs or ad hoc JSON do not
  - what the minimal object in the system is
  - how to validate an example
  - how to write a verifier
  - what is stable in v1 and what is not
  - why not just use generic JSON Schema with arbitrary events

### 24. Release packaging is tight

- [ ] The first tag is small enough to defend.
- [ ] Release notes state:
  - exactly what v1 includes
  - exactly what it does not include
  - known limitations
  - migration expectations for v1.1 and later
- [ ] CI is green.
- [ ] Badges are accurate.
- [ ] No claim exceeds what the repo actually proves.

### 25. First-version discipline check

- [ ] `v1` solves the core public contract, not the entire ecosystem.
- [ ] The repo does not try to publish every future receipt family.
- [ ] The repo does not smuggle runtime architecture promises into schema guarantees.
- [ ] At least one meaningful thing is explicitly deferred to `v2`.
- [ ] The release is opinionated enough that two independent implementers would converge.

## VertRule-Specific Minimum Gate For Publish

- [ ] `ReceiptEnvelope` semantics are frozen and reflected consistently across schema, docs, examples, and verifier tests.
- [ ] Canonical serialization and digest rules are explicitly documented and golden-tested.
- [ ] Adapter receipt binding semantics are documented if they are in public scope.
- [ ] Context, policy, and schema identity fields are clearly separated and defined.
- [ ] A minimal offline verifier can be implemented from repo docs alone.
- [ ] Positive and negative fixtures both exist and run in CI.
- [ ] The `CanonicalPayload` boundary is resolved everywhere, not just in one crate.
- [ ] The README explains the repo in under 30 seconds.
- [ ] License and contribution rules exist.
- [ ] Known limitations are written down.

## Strong Recommendation

For `vertrule-schemas v1`, ship only:

- the tight normative schemas
- a schema reference doc
- a handful of excellent fixtures
- verifier-oriented validation notes
- a brutally honest README

Do not ship a standards-body fantasy. Ship a small, sharp contract.

## Review Notes

- This checklist incorporates the VertRule-specific publishing gate context on 2026-03-19.
- Treat the JCS ordering issue, permissive schema boundary, missing validation CI, and unresolved `CanonicalPayload` boundary discipline as pre-publication blockers.
