#!/usr/bin/env bash
# CI-3: fail if filter token_savings regression tests fail
set -euo pipefail
source "${HOME}/.cargo/env" 2>/dev/null || true
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cargo test --manifest-path "$ROOT/rtk/Cargo.toml" -p rtk-context-filters token_savings
