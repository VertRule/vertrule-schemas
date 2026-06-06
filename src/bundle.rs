//! Shared bundle-vocabulary schema types.
//!
//! [`BundleMode`] is shared schema vocabulary describing a bundle's storage and
//! execution-availability semantics. It is used by **both** the pack-index and
//! verification-manifest surfaces, so it is owned here — the lowest shared
//! schema concept — rather than under `pack` or `manifest`. Transport/schema
//! only: no behavior, no authority.

use serde::{Deserialize, Serialize};

/// How model weights are handled in a bundle, determining verification strategy.
///
/// Wire form is `snake_case`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BundleMode {
    /// Weights included in the package; fully replayable.
    #[default]
    ReplayableIncluded,
    /// Weights external; verifier must supply them.
    ReplayableExternal,
    /// No replay possible; verify receipts plus provider attestation.
    AttestedExternal,
}
