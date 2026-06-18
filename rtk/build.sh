#!/usr/bin/env bash
# One-shot: install Rust toolchain + build + install RTK binary.
# Run once from WSL after cloning.
set -euo pipefail

# Always run from the rtk/ directory regardless of caller CWD
cd "$(dirname "$0")"

# ── 1. Rust toolchain ─────────────────────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
  echo "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
  export PATH="$HOME/.cargo/bin:$PATH"
  # Persist for future shells
  if ! grep -q 'cargo/bin' "$HOME/.bashrc" 2>/dev/null; then
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.bashrc"
  fi
fi

# ── 2. System deps ────────────────────────────────────────────────────────────
if ! command -v jq &>/dev/null; then
  echo "Installing jq..."
  sudo apt-get update -qq && sudo apt-get install -y -qq jq
fi

# ── 3. Build + install RTK ────────────────────────────────────────────────────
echo "Building RTK..."
cargo build --release

echo "Installing RTK to ~/.cargo/bin/rtk..."
cargo install --path . --force

# ── 4. Verify ─────────────────────────────────────────────────────────────────
echo ""
echo "RTK version: $(rtk --version)"
echo "Location:    $(which rtk)"
echo ""
echo "Verify hook:"
echo '{"tool_input":{"command":"git status"}}' \
  | RTK_HOOK_AUDIT=0 bash "$(dirname "$0")/../.claude/hooks/rtk-rewrite.sh" \
  | python3 -m json.tool
echo ""
echo "✅ RTK installed. Restart Claude Code to activate the PreToolUse hook."
