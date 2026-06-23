#!/usr/bin/env bash
# TST-3: minimal end-to-end smoke — rewrite, filter, show-log
set -euo pipefail
source "${HOME}/.cargo/env" 2>/dev/null || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RTK="${RTK:-$ROOT/rtk/target/debug/rtk}"

if [ ! -x "$RTK" ]; then
  cargo build --manifest-path "$ROOT/rtk/Cargo.toml" -p rtk-context-engine -q
fi

echo "== rewrite =="
rewritten="$("$RTK" rewrite "git status")"
[ "$rewritten" = "rtk git status" ]

echo "== deny chained segment =="
set +e
"$RTK" rewrite "git status && git push origin main --force" >/dev/null 2>&1
deny_code=$?
set -e
[ "$deny_code" = "2" ] || {
  echo "expected exit 2 on chained force push, got $deny_code" >&2
  exit 1
}

echo "== filter + show-log =="
SMOKE_DIR="$(mktemp -d)"
trap 'rm -rf "$SMOKE_DIR"' EXIT
DB="$SMOKE_DIR/rtk.db"

git -C "$SMOKE_DIR" init -q
git -C "$SMOKE_DIR" config user.email "smoke@rtk.local"
git -C "$SMOKE_DIR" config user.name "RTK Smoke"

OUT="$("$RTK" git log --oneline -30 2>&1)" || true
if [ -z "$OUT" ]; then
  OUT="$("$RTK" git status 2>&1)" || true
fi

if echo "$OUT" | grep -q 'rtk show-log'; then
  LOG_ID="$(echo "$OUT" | sed -n 's/.*rtk show-log \([0-9][0-9]*\).*/\1/p' | tail -1)"
  [ -n "$LOG_ID" ]
  "$RTK" show-log "$LOG_ID" >/dev/null
  echo "show-log $LOG_ID OK"
else
  RTK_DB_PATH="$DB" "$RTK" git status >/dev/null 2>&1 || true
  [ -f "$DB" ]
  echo "filter OK (no compression marker in this repo snapshot)"
fi

echo "e2e smoke passed"
