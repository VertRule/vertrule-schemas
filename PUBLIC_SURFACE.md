# vertrule-schemas Public Surface (current)

Canonical schema types for the VertRule receipt system.
This crate contains wire shapes, validated scalar types, and associated
constants. Receipt commitment construction lives in
`vr-receipt-identity`; verification judgments live in `vertrule-verifier`.

## Governing Rule

| Crate | Allowed role |
|-------|-------------|
| `vr-jcs` | Canonicalization primitive |
| `vertrule-schemas` | Nouns / wire shapes / validated scalar types |
| `vr-receipt-identity` | Receipt projection and commitment construction |
| `vertrule-verifier` | Judgment over public artifacts |

## Stable Root Exports

```rust
// Wire shapes
pub struct ReceiptEnvelope { .. }   // Pure data, no methods

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
pub struct SchemaVersion { .. }     // V1, digest_algorithm(), canonicalization()

// Context constraint
pub struct RBHInvariant { .. }

// Projection trait
pub trait ProjectsToReceiptEnvelope { .. }

// Error
pub enum DefinitionError { .. }

// MRI domain types (batch-aware receipt payloads)
pub struct MriBatchPayload { .. }
pub struct GradientCouplingPayload { .. }
pub struct ReductionProvenance { .. }
pub enum ReductionMode { .. }
pub enum ReductionAxis { .. }
pub enum TokenReduction { .. }
pub enum BatchReduction { .. }

// Semantic digest role newtypes — same bytes, distinct trust roles
// (#[serde(transparent)] over DigestBytes; each provides
// ::new(DigestBytes), .bytes(), .to_hex(), From<DigestBytes>)
pub struct PolicyDigest { .. }
pub struct SchemaDigest { .. }
pub struct ContextDigest { .. }
pub struct ReceiptDigest { .. }
pub struct PayloadDigest { .. }
pub struct ContentIdentityDigest { .. }

// Passive policy-evaluation input carriers. Commitment producers remain
// downstream in vr-policy-substrate.
pub const GOVERNANCE_INPUT_FORMAT: &str;
pub enum EvaluationInputKindV1 { .. }
pub enum GovernanceOperationV1 { .. }
pub enum GovernanceSystemStatusV1 { .. }
pub enum GovernancePolicyStatusV1 { .. }
pub struct LinkedPolicyStateV1 { .. }
pub struct GovernanceSystemSubjectV1 { .. }
pub struct GovernanceEvaluationInputV1 { .. }
pub enum GovernanceInputError { .. }
```

## Not Exported from This Crate

The following live in their respective crates, not here:

| Symbol | Home | Rationale |
|--------|------|-----------|
| JCS functions (`to_canon_bytes`, etc.) | `vr-jcs` | Canonicalization execution |
| Receipt commitment (`compute_event_hash`) | `vr-receipt-identity` | Receipt-identity law |
| Governance evaluation-input commitment | `vr-policy-substrate` | ADR-052 named digest law |
| Receipt construction | Producer crate | Construction is a procedure |
| Envelope integrity validation | `vertrule-verifier` | Judgment over nouns |
