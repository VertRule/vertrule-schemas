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

# Validate .vr/ governance surface (structural + semantic)
governance-check:
    cargo run --bin validate-governance-surface --features governance-check

# Full local verification: lint + unit + vectors + determinism + governance
verify-local: lint test test-vectors test-determinism governance-check
