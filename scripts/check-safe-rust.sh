#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PATTERN='use std::any::Any|use std::ffi::c_void|\bunsafe\b|\*const\b|\*mut\b'
SEARCH_PATHS=(crates fuzz)

if command -v rg >/dev/null 2>&1; then
  if rg -n --glob '*.rs' "$PATTERN" "${SEARCH_PATHS[@]}"; then
    echo
    echo 'error: forbidden Rust constructs found (see CONTRIBUTING.md safe Rust policy)'
    exit 1
  fi
else
  matches=()
  while IFS= read -r -d '' file; do
    if grep -nE "$PATTERN" "$file"; then
      matches+=("$file")
    fi
  done < <(find "${SEARCH_PATHS[@]}" -name '*.rs' -print0)

  if ((${#matches[@]} > 0)); then
    echo
    echo 'error: forbidden Rust constructs found (see CONTRIBUTING.md safe Rust policy)'
    exit 1
  fi
fi

echo 'safe Rust policy: OK'
