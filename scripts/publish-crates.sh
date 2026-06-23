#!/usr/bin/env bash
# Publish all RTK workspace crates to crates.io (dependency order).
# Prereq: cargo login <token from https://crates.io/settings/tokens>
set -euo pipefail
source "${HOME}/.cargo/env" 2>/dev/null || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/rtk/Cargo.toml"

PACKAGES=(
  rtk-context-db
  rtk-context-filters
  rtk-context-index
  rtk-context-pack
  rtk-context-mcp
  rtk-context-engine
)

DRY="${DRY:-0}"
if [ "${1:-}" = "--dry-run" ]; then
  DRY=1
  shift
fi

cd "$ROOT/rtk"
cargo fmt --check
cargo test --workspace

for pkg in "${PACKAGES[@]}"; do
  echo "== publish $pkg =="
  if [ "$DRY" = "1" ]; then
    # package only: publish --dry-run fails if upstream 2.3.x not on crates.io yet
    cargo package --manifest-path "$MANIFEST" -p "$pkg" --allow-dirty "$@"
    continue
  fi
  cargo publish --manifest-path "$MANIFEST" -p "$pkg" "$@"
  echo "waiting for index..."
  sleep 45
done

echo "Done. Verify: https://crates.io/crates/rtk-context-engine"
