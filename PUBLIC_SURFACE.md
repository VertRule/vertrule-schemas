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
// `SchemaId` (`vr.<domain>.<name>@<major>.<minor>`) is identity-layer syntax
// owned by `vr-identity` (ADR-057); consume `vr_identity::SchemaId`. It is
// not re-exported here.

// Version tag (carries identity triple)
pub struct SchemaVersion { .. }     // V1, V2, digest_algorithm(), canonicalization()

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

## V2 Receipt Surface (ADR-056)

Passive nouns only; the formation law lives in `vr-receipt-identity`, every
V2 law in `vertrule-verifier`.

```rust
// Wire shape — closed (deny_unknown_fields), #[non_exhaustive] for construction only;
// producers mint through vr_receipt_identity::seal_receipt_v2.
pub struct ReceiptEnvelopeV2 {
    pub envelope_version: SchemaVersion,          // must be V2
    pub receipt_type: ReceiptTypeV2,
    pub schema_digest: DigestBytes,               // identity of the payload-governing schema (R6)
    pub context_digest: Option<DigestBytes>,      // absent = key omitted (R8); `null` is rejected
    pub policy_digest: Option<DigestBytes>,
    pub logical_time: u64,                        // canonical decimal string on the wire; input: that string or a bare number only
    pub parent_id: Option<DigestBytes>,           // same-type chain only
    pub payload: CanonicalPayload,
    pub receipt_digest: DigestBytes,              // BLAKE3("vertrule.receipt.v2\0" ‖ JCS(envelope \ {receipt_digest}))
}

// Closed semantic discriminator — NOT #[non_exhaustive]; a new label is a
// schemas release AND a vertrule-verifier registry row.
pub enum ReceiptTypeV2 { GovernanceDecision /* vr.governance.decision */,
                         RuntimePortSubmitOutcome /* vr.runtime_port.submit_outcome */ }
impl ReceiptTypeV2 { pub const ADMITTED: [Self; 2]; pub const fn label(self) -> &'static str; }

// Admitted payload-schema labels (ADR-054 SchemaLabel class; ADR-057 D5).
// The struct carries the label only; identity() derives
// derive_key("vertrule.identity.schema-label.v1", UTF8(label)) through
// vr_identity::digest::SchemaLabelIdentity. The former hex pins are KATs in
// tests, not surface.
pub struct PayloadSchemaV2 { .. }                 // label() -> &'static str,
                                                  // identity() -> Result<DigestBytes, vr_identity::IdentityError>
//   PayloadSchemaV2::VR_SURFACE_DECISION_0_1             vr.surface.decision@0.1
//   PayloadSchemaV2::VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1  vr.runtime_port.submit_outcome@0.1

// Passive payload shapes
pub struct RuntimePortSubmitOutcomePayload { .. } // closed; vr.runtime_port.submit_outcome@0.1
pub struct TransitionCommitment { .. }
pub enum RuntimePortCommandKind { Submit }
```

Dropped from V1 deliberately: `event_hash`, `event_hash_profile`, `boundary_origin`,
`digest_algorithm`, `canonicalization`. V1 (`ReceiptEnvelope`) is frozen and unchanged.

### `vr-receipt-identity` V2 surface (the ONE formation authority)

```rust
pub const RECEIPT_V2_DOMAIN_TAG: &[u8] = b"vertrule.receipt.v2\0";   // 20 bytes
pub enum ReceiptDigestV2Identity {}   // declare_digest_domain! { id: "receipt.envelope-v2", formation: TaggedJcsCanonicalJson }
pub struct ReceiptV2Draft { .. }      // producer facts, plain pub fields
pub fn compute_receipt_digest_v2(&ReceiptEnvelopeV2) -> Result<DigestBytes, ReceiptIdentityError>;
pub fn seal_receipt_v2(ReceiptV2Draft) -> Result<ReceiptEnvelopeV2, ReceiptIdentityError>;   // the only mint path
```

Golden: `vr-receipt-identity/test-vectors/receipt_v2_governance_decision_001.json`.

## Not Exported from This Crate

The following live in their respective crates, not here:

| Symbol | Home | Rationale |
|--------|------|-----------|
| JCS functions (`to_canon_bytes`, etc.) | `vr-jcs` | Canonicalization execution |
| Receipt commitment (`compute_event_hash`, `compute_receipt_digest_v2`, `seal_receipt_v2`) | `vr-receipt-identity` | Receipt-identity law |
| Governance evaluation-input commitment | `vr-policy-substrate` | ADR-052 named digest law |
| Receipt construction | Producer crate | Construction is a procedure |
| Envelope integrity validation | `vertrule-verifier` | Judgment over nouns |
