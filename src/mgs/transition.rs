//! Status transition nouns for governance history.
//!
//! A specification changing its proof mode — e.g., from `Conjecture`
//! to `PropertyTested` — is a verifiable governance event that
//! travels across repo boundaries.

use crate::DigestBytes;
use serde::{Deserialize, Serialize};

/// A recorded transition of a specification's verification status.
///
/// This is a public governance event. Status changes are verifiable
/// and must leave a machine-readable trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct StatusTransition {
    /// Identifier of the specification that changed.
    pub spec_id: String,
    /// Human-readable label for the previous status.
    pub from_status: String,
    /// Human-readable label for the new status.
    pub to_status: String,
    /// Why this transition happened.
    pub justification: TransitionJustification,
    /// BLAKE3 digest of this transition record.
    pub transition_digest: DigestBytes,
}

/// What justifies a status transition.
///
/// Public so that governance surfaces can reason over transition
/// causes without access to internal formalization types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum TransitionJustification {
    /// New evidence was produced (e.g., proptest file, certificate).
    NewEvidence {
        /// Digest of the new evidence artifact.
        evidence_digest: DigestBytes,
        /// Description of what was produced.
        description: String,
    },
    /// Existing evidence was invalidated (counterexample found).
    Falsified {
        /// Digest of the counterexample artifact.
        counterexample_digest: DigestBytes,
        /// What was falsified.
        description: String,
    },
    /// A dependency was resolved (e.g., extension became base truth).
    DependencyResolved {
        /// Identifier of the resolved dependency.
        dependency_id: String,
    },
}
