set shell := ["bash", "-euo", "pipefail", "-c"]

# Default command - show available recipes
default:
    @just --list

# Lint (fmt + clippy)
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

# Run all tests
test:
    cargo nextest run

# Run fixture-driven vector/rejection tests (L1, L2)
test-vectors:
    cargo test --test test_vector_validation

# Run determinism proof tests (L1, L3)
test-determinism:
    cargo test --test determinism_tests

# Full local verification: lint + unit + vectors + determinism
verify-local: lint test test-vectors test-determinism
