#!/usr/bin/env bash
# Homebrew formula sanity — license, URLs, version parity with Cargo.toml
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FORMULA="$ROOT/rtk.rb"
PKG="$ROOT/rtk/rtk-cli/Cargo.toml"

grep -q 'license "Apache-2.0"' "$FORMULA" || {
  echo "homebrew smoke FAIL: rtk.rb license must be Apache-2.0" >&2
  exit 1
}

grep -q 'andreafinazziinfo/rust-context-engine' "$FORMULA" || {
  echo "homebrew smoke FAIL: rtk.rb homepage/url mismatch" >&2
  exit 1
}

for asset in rtk-macos-arm64.tar.gz rtk-macos-amd64.tar.gz rtk-linux-amd64.tar.gz; do
  grep -q "$asset" "$FORMULA" || {
    echo "homebrew smoke FAIL: missing release asset $asset in rtk.rb" >&2
    exit 1
  }
done

EXPECTED="$(grep '^version' "$PKG" | head -1 | sed 's/version = "//;s/"//' | tr -d ' ')"
echo "homebrew smoke OK (formula v$EXPECTED parity target)"
