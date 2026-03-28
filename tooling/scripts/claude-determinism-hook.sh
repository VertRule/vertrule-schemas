#!/usr/bin/env bash
# Claude Code PostToolUse hook: enforce determinism invariants
# Runs after Write/Edit on Rust files within this repo.
# Non-blocking — emits warnings via Claude hook JSON protocol.

set -euo pipefail

JSON_INPUT=$(cat)

FILE=$(echo "$JSON_INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE" ]; then
    exit 0
fi

if [ ! -f "$FILE" ] || [[ "$FILE" != *.rs ]]; then
    exit 0
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if [[ "$FILE" != "$REPO_ROOT"/* ]]; then
    exit 0
fi

# Skip test files for float/unwrap checks
IS_TEST=0
if [[ "$FILE" == *_tests.rs ]] || [[ "$FILE" == */tests/*.rs ]]; then
    IS_TEST=1
fi

VIOLATIONS=""

# Strip comments (approximate)
CONTENT=$(grep -vE '^\s*//' "$FILE" 2>/dev/null || true)

# Check 1: HashMap/HashSet (all files)
if echo "$CONTENT" | grep -qE '\bHashMap\b|\bHashSet\b'; then
    VIOLATIONS="${VIOLATIONS}
- HashMap/HashSet detected — use BTreeMap/BTreeSet for deterministic iteration"
fi

# Check 2: Wall-clock time (all files)
if echo "$CONTENT" | grep -qE 'SystemTime::now|Instant::now|chrono::(Utc|Local)::now'; then
    VIOLATIONS="${VIOLATIONS}
- Wall-clock time detected — inject via Clock trait"
fi

# Check 3: Unseeded randomness (all files)
if echo "$CONTENT" | grep -qE '\bthread_rng\b|\bOsRng\b|\brandom\(\)'; then
    VIOLATIONS="${VIOLATIONS}
- Unseeded randomness detected — use seeded RNG"
fi

# Production-only checks
if [ $IS_TEST -eq 0 ]; then
    # Check 4: f32/f64 types
    if echo "$CONTENT" | grep -qE '\bf32\b|\bf64\b'; then
        VIOLATIONS="${VIOLATIONS}
- f32/f64 type detected — floats are forbidden in schema types"
    fi

    # Check 5: Float literals
    if grep -vE '^\s*//|version\s*=' "$FILE" 2>/dev/null | grep -qP '\b\d+\.\d+\b(?!\.)'; then
        VIOLATIONS="${VIOLATIONS}
- Float literal detected — floats are forbidden in the receipt spine"
    fi

    # Check 6: unwrap/expect
    if echo "$CONTENT" | grep -qE '\.unwrap\(\)|\.expect\('; then
        VIOLATIONS="${VIOLATIONS}
- unwrap()/expect() detected — use ? or let...else"
    fi
fi

if [ -n "$VIOLATIONS" ]; then
    WARNING_TEXT="Determinism violation in ${FILE}
${VIOLATIONS}

vertrule-schemas is a constitutional schema crate.
All code must be deterministic by construction."

    jq -n --arg context "$WARNING_TEXT" '{
        hookSpecificOutput: {
            hookEventName: "PostToolUse",
            additionalContext: $context
        }
    }'
fi

exit 0
