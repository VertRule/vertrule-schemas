//! Execution-context schema types.
//!
//! - [`RBHInvariant`] — Constitutional identity continuity constraint (RBH)

mod rbh_invariant;

pub use rbh_invariant::RBHInvariant;

#[cfg(test)]
#[path = "rbh_invariant_tests.rs"]
mod rbh_invariant_tests;
