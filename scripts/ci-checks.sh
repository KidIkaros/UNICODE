#!/usr/bin/env bash
# CI check suite for unicode16-blocks.
#
# Runs everything the GitHub Actions workflow runs, so the same command
# works locally and in CI with no drift:
#
#   ./scripts/ci-checks.sh
#
# Exit status is non-zero on the first failing section.
set -euo pipefail

echo "==> cargo build (no_std core, default, json features)"
cargo build --quiet --no-default-features --lib
cargo build --quiet
cargo build --quiet --features json

echo "==> cargo test (default + json features)"
cargo test --quiet
cargo test --quiet --features json

echo "==> clippy (all targets, all features, warnings denied)"
cargo clippy --quiet --all-targets --all-features -- -D warnings

echo "==> rustfmt"
cargo fmt --check

echo "==> layout samples: validate + round-trip"
BIN="target/debug/layout"
PAYLOAD="$(mktemp)"
trap 'rm -f "$PAYLOAD"' EXIT
head -c 4096 /dev/urandom > "$PAYLOAD"

for layout in layouts/*.json; do
    name="$(basename "$layout" .json)"
    cipher="$(mktemp)"
    back="$(mktemp)"
    trap 'rm -f "$PAYLOAD" "$cipher" "$back"' EXIT

    "$BIN" validate "$layout" > /dev/null
    "$BIN" encode "$layout" --in "$PAYLOAD" --out "$cipher" 2> /dev/null
    "$BIN" decode "$layout" --in "$cipher" --out "$back" 2> /dev/null
    cmp "$PAYLOAD" "$back"

    symbols=$(python3 -c "print(len(open('$cipher', encoding='utf-8').read()))")
    echo "    $name: valid, 4096 bytes -> $symbols symbols -> byte-identical"

    rm -f "$cipher" "$back"
done

echo "==> all checks passed"
