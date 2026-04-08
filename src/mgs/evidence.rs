//! Evidence posture and certificate summary nouns.
//!
//! These are public constitutional types consumed by verifiers and
//! governance surfaces across repos.

use crate::DigestBytes;
use serde::{Deserialize, Serialize};

/// Whether a bounded search completed or was cut short.
///
/// This is a constitutional noun: evidence admissibility across repos
/// depends on distinguishing complete from budget-hit searches.
///
/// A complete search is a proof over its declared domain.
/// A budget-hit search is evidence with explicit debt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SearchPosture {
    /// Search covered the entire declared domain.
    Complete,
    /// Search stopped due to budget exhaustion.
    BudgetHit,
}

/// Digest-bearing summary of a computational certificate.
///
/// Public surfaces bind to this summary without exposing the full
/// private certificate body. External verifiers can validate the
/// commitment chain without access to internal crate types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct CertificateSummary {
    /// What kind of certificate this summarizes.
    pub kind: CertificateKind,
    /// Evidence posture (complete vs budget-hit).
    pub posture: SearchPosture,
    /// BLAKE3 digest of the full certificate body.
    pub summary_digest: DigestBytes,
    /// Optional digest of the search policy that governed exploration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_policy_digest: Option<DigestBytes>,
    /// Optional digest of the domain bound that was searched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_bound_digest: Option<DigestBytes>,
}

/// What kind of certificate a summary represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CertificateKind {
    /// Constructive positive witness.
    Witness,
    /// Complete or budget-hit search over a finite domain.
    BoundedExhaustive,
    /// Falsification artifact (counterexample found).
    Counterexample,
}
