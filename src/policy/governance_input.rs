//! `vr.policy.governance-input@0.1` — recorded governance facts for policy evaluation.
//!
//! This is a passive, facts-only carrier. It does not evaluate policy and it
//! deliberately has no commitment constructor; ADR-052 assigns the named
//! digest law to `vr-policy-substrate`.

use serde::{Deserialize, Deserializer, Serialize};

/// Governance evaluation-input format identity frozen by ADR-052.
pub const GOVERNANCE_INPUT_FORMAT: &str = "vr.policy.governance-input@0.1";

/// Closed evaluation-input family discriminator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationInputKindV1 {
    /// Existing `vr.policy.input@0.1` claim-evidence carrier.
    #[default]
    PolicyInputV0_1,
    /// Facts-only `vr.policy.governance-input@0.1` carrier.
    GovernanceInputV0_1,
}

impl EvaluationInputKindV1 {
    /// Return the one format identity governed by this kind.
    #[must_use]
    pub const fn format(self) -> &'static str {
        match self {
            Self::PolicyInputV0_1 => super::INPUT_FORMAT,
            Self::GovernanceInputV0_1 => GOVERNANCE_INPUT_FORMAT,
        }
    }
}

/// Governed operation whose facts are carried by the input.
///
/// This names the operation; it does not decide whether the operation is
/// allowed. New operations require an explicit schema revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceOperationV1 {
    /// Request authorization for an AI-system promotion transition.
    SystemPromote,
}

/// Workspace AI-system status vocabulary observed by ADR-052.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceSystemStatusV1 {
    /// System is active.
    Active,
    /// System is undergoing review.
    UnderReview,
    /// System has recorded approved status.
    Approved,
    /// System has recorded restricted status.
    Restricted,
    /// System has been retired.
    Retired,
}

/// Workspace policy status vocabulary observed by ADR-052.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernancePolicyStatusV1 {
    /// Policy remains a draft.
    Draft,
    /// Policy has recorded active status.
    Active,
    /// Policy has been retired.
    Retired,
}

/// One linked policy identity and its recorded state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinkedPolicyStateV1 {
    policy_id: String,
    policy_status: GovernancePolicyStatusV1,
}

impl LinkedPolicyStateV1 {
    /// Construct one linked-policy fact.
    ///
    /// # Errors
    ///
    /// Returns [`GovernanceInputError::InvalidIdentifier`] when `policy_id`
    /// is empty or contains surrounding whitespace.
    pub fn new(
        policy_id: String,
        policy_status: GovernancePolicyStatusV1,
    ) -> Result<Self, GovernanceInputError> {
        validate_identifier("policy_id", &policy_id)?;
        Ok(Self {
            policy_id,
            policy_status,
        })
    }

    /// Return the canonical Workspace policy identity.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        &self.policy_id
    }

    /// Return the recorded policy status.
    #[must_use]
    pub const fn policy_status(&self) -> GovernancePolicyStatusV1 {
        self.policy_status
    }
}

/// Canonical AI-system subject and its linked-policy facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GovernanceSystemSubjectV1 {
    system_id: String,
    system_status: GovernanceSystemStatusV1,
    linked_policies: Vec<LinkedPolicyStateV1>,
}

impl GovernanceSystemSubjectV1 {
    /// Construct a subject, sorting policy links by canonical policy identity.
    ///
    /// # Errors
    ///
    /// Returns [`GovernanceInputError::InvalidIdentifier`] for an invalid
    /// system identity, or [`GovernanceInputError::DuplicatePolicyId`] when
    /// more than one link carries the same policy identity.
    pub fn new(
        system_id: String,
        system_status: GovernanceSystemStatusV1,
        mut linked_policies: Vec<LinkedPolicyStateV1>,
    ) -> Result<Self, GovernanceInputError> {
        validate_identifier("system_id", &system_id)?;
        linked_policies.sort_by(|left, right| left.policy_id.cmp(&right.policy_id));
        for adjacent in linked_policies.windows(2) {
            if adjacent[0].policy_id == adjacent[1].policy_id {
                return Err(GovernanceInputError::DuplicatePolicyId(
                    adjacent[0].policy_id.clone(),
                ));
            }
        }
        Ok(Self {
            system_id,
            system_status,
            linked_policies,
        })
    }

    /// Return the canonical Workspace system identity.
    #[must_use]
    pub fn system_id(&self) -> &str {
        &self.system_id
    }

    /// Return the recorded system status.
    #[must_use]
    pub const fn system_status(&self) -> GovernanceSystemStatusV1 {
        self.system_status
    }

    /// Return policy links in canonical policy-identity order.
    #[must_use]
    pub fn linked_policies(&self) -> &[LinkedPolicyStateV1] {
        &self.linked_policies
    }
}

/// Facts-only governance evaluation input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GovernanceEvaluationInputV1 {
    input_format: String,
    operation: GovernanceOperationV1,
    subject: GovernanceSystemSubjectV1,
}

impl GovernanceEvaluationInputV1 {
    /// Construct a governance evaluation input under the one permitted format.
    #[must_use]
    pub fn new(operation: GovernanceOperationV1, subject: GovernanceSystemSubjectV1) -> Self {
        Self {
            input_format: GOVERNANCE_INPUT_FORMAT.to_string(),
            operation,
            subject,
        }
    }

    /// Return the committed format identity.
    #[must_use]
    pub fn input_format(&self) -> &str {
        &self.input_format
    }

    /// Return the governed operation.
    #[must_use]
    pub const fn operation(&self) -> GovernanceOperationV1 {
        self.operation
    }

    /// Return the canonical facts-only subject.
    #[must_use]
    pub const fn subject(&self) -> &GovernanceSystemSubjectV1 {
        &self.subject
    }

    /// Serialize to RFC 8785 JCS bytes.
    ///
    /// # Errors
    ///
    /// Returns [`GovernanceInputError::Canonicalization`] if serialization or
    /// canonicalization fails.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, GovernanceInputError> {
        let json = serde_json::to_vec(self)
            .map_err(|error| GovernanceInputError::Canonicalization(error.to_string()))?;
        vr_jcs::to_canon_bytes_from_slice(&json)
            .map_err(|error| GovernanceInputError::Canonicalization(error.to_string()))
    }

    /// Parse an externally supplied canonical governance-input document.
    ///
    /// Parsing rejects duplicate JSON members, unknown fields, invalid enum
    /// tokens, wrong format identity, duplicate policy identities and
    /// noncanonical byte encodings.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic [`GovernanceInputError`].
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, GovernanceInputError> {
        let value = vr_jcs::strict_parse::parse_json_value_no_duplicates(bytes)
            .map_err(|error| GovernanceInputError::Parse(error.to_string()))?;
        let parsed: Self = serde_json::from_value(value)
            .map_err(|error| GovernanceInputError::Parse(error.to_string()))?;
        let canonical = parsed.to_canonical_bytes()?;
        if canonical != bytes {
            return Err(GovernanceInputError::NonCanonicalEncoding);
        }
        Ok(parsed)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkedPolicyStateWire {
    policy_id: String,
    policy_status: GovernancePolicyStatusV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GovernanceSystemSubjectWire {
    system_id: String,
    system_status: GovernanceSystemStatusV1,
    linked_policies: Vec<LinkedPolicyStateWire>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GovernanceEvaluationInputWire {
    input_format: String,
    operation: GovernanceOperationV1,
    subject: GovernanceSystemSubjectWire,
}

impl<'de> Deserialize<'de> for GovernanceEvaluationInputV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = GovernanceEvaluationInputWire::deserialize(deserializer)?;
        if wire.input_format != GOVERNANCE_INPUT_FORMAT {
            return Err(serde::de::Error::custom(format!(
                "wrong governance input format: expected {GOVERNANCE_INPUT_FORMAT}, got {}",
                wire.input_format
            )));
        }
        let links = wire
            .subject
            .linked_policies
            .into_iter()
            .map(|link| LinkedPolicyStateV1::new(link.policy_id, link.policy_status))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)?;
        let subject = GovernanceSystemSubjectV1::new(
            wire.subject.system_id,
            wire.subject.system_status,
            links,
        )
        .map_err(serde::de::Error::custom)?;
        Ok(Self::new(wire.operation, subject))
    }
}

fn validate_identifier(field: &'static str, value: &str) -> Result<(), GovernanceInputError> {
    if value.is_empty() || value.trim() != value {
        return Err(GovernanceInputError::InvalidIdentifier { field });
    }
    Ok(())
}

/// Governance-input construction, parsing and canonicalization failures.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GovernanceInputError {
    /// A canonical identity was empty or contained surrounding whitespace.
    #[error("{field} must be non-empty and contain no surrounding whitespace")]
    InvalidIdentifier {
        /// Invalid identity field.
        field: &'static str,
    },
    /// The same linked policy identity appeared more than once.
    #[error("duplicate linked policy identity: {0}")]
    DuplicatePolicyId(String),
    /// Typed serialization or JCS canonicalization failed.
    #[error("governance input canonicalization failed: {0}")]
    Canonicalization(String),
    /// Strict parsing or typed deserialization failed.
    #[error("governance input parsing failed: {0}")]
    Parse(String),
    /// Supplied bytes encoded a valid document but were not its canonical JCS.
    #[error("governance input bytes are not canonical JCS")]
    NonCanonicalEncoding,
}

#[cfg(test)]
#[path = "governance_input_tests.rs"]
mod tests;
