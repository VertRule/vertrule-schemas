//! MGS (Math Governance Substrate) public nouns.
//!
//! Domain-specific types for MGS receipt payloads and governance events.
//! These types are receipt-visible: they affect the meaning of
//! formalization evidence and are consumed by external verifiers.
//!
//! # Placement rule
//!
//! Only constitutional nouns belong here:
//! - Evidence posture (complete vs budget-hit)
//! - Status transition events (governance history)
//! - Certificate summary commitments (digest-bearing, not full body)
//!
//! Internal formalization metadata (registry, index, Lean sketches,
//! `SpecId`, `DomainNs`, `EpistemicReach`) stays in `mgs-specs`.
//! Full certificate bodies stay in `mgs-cert`.

mod evidence;
mod transition;

pub use evidence::{CertificateKind, CertificateSummary, SearchPosture};
pub use transition::{StatusTransition, TransitionJustification};
