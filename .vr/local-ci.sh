#!/usr/bin/env bash
set -euo pipefail

# Local CI gate for vertrule-schemas.
# Runs the same checks as .github/workflows/ci.yml.
# Invoked by the governance pre-push hook.

cd "$(git rev-parse --show-toplevel)"

echo "local-ci: format check"
cargo fmt --check

echo "local-ci: clippy"
cargo clippy --all-targets -- -D warnings

echo "local-ci: tests"
cargo test --quiet

echo "local-ci: test vectors"
cargo test --test test_vector_validation --quiet

echo "local-ci: determinism tests"
cargo test --test determinism_tests --quiet

echo "local-ci: governance surface"
cargo run --bin validate-governance-surface --features governance-check

echo "local-ci: all checks passed"
