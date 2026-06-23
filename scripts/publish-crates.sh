#!/usr/bin/env bash
# Publish all RTK workspace crates to crates.io (dependency order).
# Prereq: cargo login <token from https://crates.io/settings/tokens>
set -euo pipefail
source "${HOME}/.cargo/env" 2>/dev/null || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/rtk/Cargo.toml"

# Leaf crates: no rtk workspace deps — safe to `cargo package` before anything is on crates.io.
LEAF_PACKAGES=(
  rtk-context-db
  rtk-context-filters
  rtk-context-index
)

DEP_PACKAGES=(
  rtk-context-pack
  rtk-context-mcp
  rtk-context-engine
)

PACKAGES=("${LEAF_PACKAGES[@]}" "${DEP_PACKAGES[@]}")

DRY="${DRY:-0}"
SKIP_TESTS="${SKIP_TESTS:-0}"
if [ "${1:-}" = "--dry-run" ]; then
  DRY=1
  shift
fi

cd "$ROOT/rtk"
cargo fmt --check
if [ "$SKIP_TESTS" != "1" ]; then
  cargo test --workspace
fi

publish_one() {
  local pkg="$1"
  local attempt
  for attempt in 1 2 3 4 5; do
    if cargo publish --manifest-path "$MANIFEST" -p "$pkg" "$@"; then
      return 0
    fi
    if [ "$attempt" -lt 5 ]; then
      echo "retry $pkg in 60s (crates.io index may lag)..."
      sleep 60
    fi
  done
  return 1
}

for pkg in "${PACKAGES[@]}"; do
  echo "== publish $pkg =="
  if [ "$DRY" = "1" ]; then
    case " $pkg " in
      *" rtk-context-pack "*|*" rtk-context-mcp "*|*" rtk-context-engine "*)
        echo "skip dry-run: $pkg needs upstream rtk crates on crates.io first"
        echo "  → run without --dry-run after cargo login"
        continue
        ;;
    esac
    cargo package --manifest-path "$MANIFEST" -p "$pkg" --allow-dirty "$@"
    continue
  fi
  publish_one "$pkg" "$@"
  echo "waiting for index..."
  sleep 45
done

echo "Done. Verify: https://crates.io/crates/rtk-context-engine"
