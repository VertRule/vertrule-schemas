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
