//! Semantic digest newtypes — same bytes, distinct trust roles.
//!
//! Every type in this module wraps [`DigestBytes`] but names a specific
//! commitment role: policy, schema, context, receipt, payload, or
//! content identity. The newtypes prevent silent substitution at port,
//! adapter, verifier, and receipt-store boundaries.
//!
//! Wrong digest role must fail at compile time. A `PolicyDigest` cannot
//! be passed where a `ContextDigest` is required, even though both are
//! 32 bytes underneath.
//!
//! These types are byte containers with serde-transparent wire format
//! equal to `DigestBytes` (lowercase hex). They do not encode their
//! derivation law: `ContextDigest::new(d)` does not assert what fields
//! `d` was computed over. Derivation lives in the producer crate; the
//! type only marks the result.
//!
//! ## Doctrine
//!
//! See `plans/governance/runtime-port-architecture.md` and the
//! recommendation that semantic digest types live in the constitutional
//! schemas crate (Option A) so adapters, runtime-port, verifier,
//! gateway, and receipt stores share a single vocabulary.

use serde::{Deserialize, Serialize};

use crate::DigestBytes;

macro_rules! semantic_digest {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(DigestBytes);

        impl $name {
            /// Wrap a precomputed digest. Construction is intentionally
            /// direct — the caller is responsible for having computed
            /// the digest over the canonical bytes for this role.
            #[must_use]
            pub const fn new(digest: DigestBytes) -> Self {
                Self(digest)
            }

            /// Borrow the wrapped digest bytes.
            #[must_use]
            pub const fn bytes(&self) -> &DigestBytes {
                &self.0
            }

            /// Encode the inner digest as a 64-character lowercase hex
            /// string. Mirrors [`DigestBytes::to_hex`].
            #[must_use]
            pub fn to_hex(&self) -> String {
                self.0.to_hex()
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let hex = self.0.to_hex();
                let prefix = &hex[..hex.len().min(8)];
                write!(f, concat!(stringify!($name), "({}…)"), prefix)
            }
        }

        impl From<DigestBytes> for $name {
            fn from(d: DigestBytes) -> Self {
                Self(d)
            }
        }
    };
}

semantic_digest!(
    PolicyDigest,
    "Digest of the policy pack under which a submission or receipt is evaluated."
);
semantic_digest!(
    SchemaDigest,
    "Digest of the schema/profile a payload conforms to."
);
semantic_digest!(
    ContextDigest,
    "Digest of the static governance context a sealed handle refers to."
);
semantic_digest!(
    ReceiptDigest,
    "Digest of an already-existing canonical receipt envelope."
);
semantic_digest!(
    PayloadDigest,
    "Digest of canonical payload bytes (BLAKE3 over the canonical encoding)."
);
semantic_digest!(
    ContentIdentityDigest,
    "Digest of canonicalized retrieved content (for retrieval-class evidence)."
);
