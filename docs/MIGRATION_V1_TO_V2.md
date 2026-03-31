# Migration: Envelope Version V1 → V2

## What Changed

**V1** (payload-only commitment):
```
event_hash = BLAKE3(JCS(payload))
```

**V2** (full-envelope commitment):
```
event_hash = BLAKE3(JCS(envelope \ {event_hash}))
```

V2 commits every trust-bearing field: `envelope_version`, `receipt_type`,
`context_digest`, `schema_digest`, `policy_digest`, `logical_time`,
`parent_id`, `boundary_origin`, `digest_algorithm`, `canonicalization`,
and `payload`.

## Why

V1 only committed the payload. Mutating metadata fields (`policy_digest`,
`context_digest`, `schema_digest`, `logical_time`, `receipt_type`,
`boundary_origin`, `parent_id`) without recomputing `event_hash` was
undetectable by the verifier.

V2 closes this gap. Any field mutation without recomputing the commitment
fails verification.

## Backward Compatibility

- V1 envelopes continue to verify correctly — the verifier checks
  `envelope_version` and applies the matching commitment scheme.
- V2 envelopes are rejected by verifiers that only support V1.
- Mixed V1/V2 chains are permitted (the verifier checks each envelope
  independently).

## How to Migrate Emitters

### Step 1: Set `envelope_version` to 2

```rust
envelope.envelope_version = SchemaVersion::V2;
```

### Step 2: Use `compute_event_hash` after all fields are set

```rust
use vertrule_schemas::receipts::compute_event_hash;

// Build envelope with all fields populated...
let mut envelope = ReceiptEnvelope {
    envelope_version: SchemaVersion::V2,
    receipt_type: ReceiptType::Governance,
    context_digest,
    schema_digest,
    policy_digest,
    logical_time,
    event_hash: DigestBytes::from_array([0u8; 32]), // placeholder
    parent_id,
    boundary_origin: Some(BoundaryOrigin::Engine),
    digest_algorithm: None,
    canonicalization: None,
    payload,
};

// Compute event_hash LAST — it covers all other fields
envelope.event_hash = compute_event_hash(&envelope)?;
```

### Step 3: Update signature computation

For signed receipts, the signature domain also changes for V2.
`compute_receipt_digest` in the verifier is version-aware:

- V1: `BLAKE3(domain_prefix || JCS(payload))`
- V2: `BLAKE3(domain_prefix || JCS(envelope \ {event_hash}))`

Signing code must use the same scheme. Update `vertrule-crypto` or
equivalent signing code to match.

## Projection Contract

`ProjectsToReceiptEnvelope` implementations should produce V2 envelopes.
The projection must:

1. Populate all fields
2. Call `compute_event_hash()` as the final step
3. Never mutate any field after computing `event_hash`

## Verification Behavior

| Envelope Version | Commitment Scope | Verifier Support |
|-----------------|------------------|------------------|
| V1 | payload only | Supported (backward compat) |
| V2 | all fields except `event_hash` | Supported |
| Other | — | Rejected (`UnsupportedVersion`) |

## V1 Retirement Timeline

V1 remains supported indefinitely for verification. New envelope emission
should use V2. A future major version of vertrule-schemas may mark V1
as deprecated for emission while keeping verification support.

## Test Coverage

### vertrule-schemas (commitment)
- V1 payload-only hash matches
- V2 full-envelope hash matches
- V2 deterministic
- V1/V2 produce different hashes for same content
- Tamper detection for every trust-bearing field (8 tests)

### vertrule-verifier (integration)
- V2 valid envelope passes verification
- V2 tamper tests: receipt_type, context_digest, schema_digest,
  policy_digest, logical_time, parent_id, boundary_origin, payload,
  removal of boundary_origin (10 tests)
- V1 backward compat: metadata tampering passes V1 (documents the
  weakness V2 fixes)
- All existing V1 test vectors still pass
