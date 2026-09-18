//! Payload-schema identities admitted for V2 receipts (ADR-056 §3.3, §4).
//!
//! `schema_digest` on a [`ReceiptEnvelopeV2`](crate::ReceiptEnvelopeV2)
//! identifies **only** the schema governing `payload` (R6). For a schema
//! named by a `SchemaId` label that identity is the ADR-054 `SchemaLabel`
//! class identity
//! `derive_key("vertrule.identity.schema-label.v1", UTF8(label))`.
//!
//! The identities are carried here as frozen constitutional hex (ADR-054
//! §7 vectors), not re-derived at runtime, until the sealed
//! `DomainSeparatedLabel` slot lands (boundary G1). A vector that fails
//! after a rename is a byte change and is refused.

use crate::DigestBytes;

/// Identity of a payload-governing schema label (ADR-054 `SchemaLabel` class).
///
/// The identity is `derive_key("vertrule.identity.schema-label.v1",
/// UTF8(label))`, carried as frozen constitutional hex until the sealed
/// `DomainSeparatedLabel` slot lands (boundary G1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PayloadSchemaV2 {
    label: &'static str,
    identity: [u8; 32],
}

impl PayloadSchemaV2 {
    /// `vr.surface.decision@0.1` — the governing schema of a
    /// `vr.governance.decision` payload (`DecisionPayload`).
    pub const VR_SURFACE_DECISION_0_1: Self = Self {
        label: "vr.surface.decision@0.1",
        // 48b92179bcf7f3663e761ef7c87e1378dd53c9d6d9f3a763bef9756b6151f460
        identity: [
            0x48, 0xb9, 0x21, 0x79, 0xbc, 0xf7, 0xf3, 0x66, 0x3e, 0x76, 0x1e, 0xf7, 0xc8, 0x7e,
            0x13, 0x78, 0xdd, 0x53, 0xc9, 0xd6, 0xd9, 0xf3, 0xa7, 0x63, 0xbe, 0xf9, 0x75, 0x6b,
            0x61, 0x51, 0xf4, 0x60,
        ],
    };

    /// `vr.runtime_port.submit_outcome@0.1` — the governing schema of a
    /// `vr.runtime_port.submit_outcome` payload
    /// (`RuntimePortSubmitOutcomePayload`).
    pub const VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1: Self = Self {
        label: "vr.runtime_port.submit_outcome@0.1",
        // ffc6e9a7710bc35c196819607a1f269480efe36bf85f7cd64c6ed7d10e554b49
        identity: [
            0xff, 0xc6, 0xe9, 0xa7, 0x71, 0x0b, 0xc3, 0x5c, 0x19, 0x68, 0x19, 0x60, 0x7a, 0x1f,
            0x26, 0x94, 0x80, 0xef, 0xe3, 0x6b, 0xf8, 0x5f, 0x7c, 0xd6, 0x4c, 0x6e, 0xd7, 0xd1,
            0x0e, 0x55, 0x4b, 0x49,
        ],
    };

    /// The `SchemaId`-grammar label this identity names.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }

    /// The frozen `SchemaLabel` class identity as wire-format
    /// [`DigestBytes`].
    #[must_use]
    pub const fn identity(&self) -> DigestBytes {
        DigestBytes::from_array(self.identity)
    }
}

#[cfg(test)]
#[path = "payload_schema_v2_tests.rs"]
mod payload_schema_v2_tests;
