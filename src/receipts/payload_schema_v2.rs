//! Payload-schema labels admitted for V2 receipts (ADR-056 §3.3, §4;
//! ADR-057 D5).
//!
//! `schema_digest` on a [`ReceiptEnvelopeV2`](crate::ReceiptEnvelopeV2)
//! identifies **only** the schema governing `payload` (R6). For a schema
//! named by a `SchemaId` label that identity is the ADR-054 `SchemaLabel`
//! class identity
//! `derive_key("vertrule.identity.schema-label.v1", UTF8(label))`, formed
//! through the sealed `vr-identity` slot
//! ([`SchemaLabelIdentity`](vr_identity::digest::SchemaLabelIdentity)).
//!
//! This type carries the **label** only; it assigns the label its
//! payload-governance meaning (which shape it names, on which row) and
//! derives the identity through the owned path. The former frozen hex pins
//! live in the sibling tests as known-answer vectors (ADR-057 D5).

use vr_identity::digest::{DigestOf, SchemaLabelIdentity};
use vr_identity::{IdentityError, SchemaId};

use crate::DigestBytes;

/// A payload-governing schema label (ADR-054 `SchemaLabel` class).
///
/// The identity is
/// `derive_key("vertrule.identity.schema-label.v1", UTF8(label))`, derived
/// through [`SchemaLabelIdentity`] on demand — never carried as bytes here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PayloadSchemaV2 {
    label: &'static str,
}

impl PayloadSchemaV2 {
    /// `vr.surface.decision@0.1` — the governing schema of a
    /// `vr.governance.decision` payload (`DecisionPayload`).
    pub const VR_SURFACE_DECISION_0_1: Self = Self {
        label: "vr.surface.decision@0.1",
    };

    /// `vr.runtime_port.submit_outcome@0.1` — the governing schema of a
    /// `vr.runtime_port.submit_outcome` payload
    /// (`RuntimePortSubmitOutcomePayload`).
    pub const VR_RUNTIME_PORT_SUBMIT_OUTCOME_0_1: Self = Self {
        label: "vr.runtime_port.submit_outcome@0.1",
    };

    /// The `SchemaId`-grammar label this schema is named by.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }

    /// The `SchemaLabel` class identity of this label as wire-format
    /// [`DigestBytes`], derived through the sealed `vr-identity` slot.
    ///
    /// # Errors
    ///
    /// Returns [`IdentityError::InvalidSchemaId`] if the label is not an
    /// admitted `SchemaId` (a source defect for a frozen literal), or the
    /// formation's [`IdentityError::ConstructionFailed`] if the sealed slot
    /// refuses to form the identity.
    pub fn identity(&self) -> Result<DigestBytes, IdentityError> {
        let label = SchemaId::new(self.label.to_string())?;
        let identity = DigestOf::<SchemaLabelIdentity>::derive(&label)?;
        Ok(DigestBytes::from_array(identity.as_32_byte_array()?))
    }
}

#[cfg(test)]
#[path = "payload_schema_v2_tests.rs"]
mod payload_schema_v2_tests;
