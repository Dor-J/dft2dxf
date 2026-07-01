#!/usr/bin/env bash
# Line coverage for library crates + CLI (excludes testkit and fuzz).
#
# Baseline (pre-80% push): run without --fail-under-lines to inspect per-crate %.
# CI enforces >= 80% once the test suite is complete.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ENFORCE="${COVERAGE_ENFORCE:-1}"
FAIL_UNDER="${COVERAGE_FAIL_UNDER:-80}"

ARGS=(
  llvm-cov
  --workspace
  --exclude dft2dxf-testkit
  --all-features
  --lcov
  --output-path target/coverage/lcov.info
)

if [[ "${1:-}" == "--html" ]]; then
  cargo llvm-cov "${ARGS[@]}" --html --output-dir target/coverage/html
  echo "HTML report: target/coverage/html/index.html"
  exit 0
fi

if [[ "$ENFORCE" == "1" ]]; then
  cargo llvm-cov "${ARGS[@]}" --summary-only --fail-under-lines "$FAIL_UNDER"
else
  cargo llvm-cov "${ARGS[@]}" --summary-only
fi
