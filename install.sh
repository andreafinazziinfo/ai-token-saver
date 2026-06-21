#!/usr/bin/env bash
# Installation script for AI Efficiency Toolkit
set -euo pipefail

cd "$(dirname "$0")"

echo "🚀 Starting AI Efficiency Toolkit Installation..."

# 1. Check for Rust / Cargo
if ! command -v cargo &>/dev/null; then
    echo "❌ Cargo is not installed. Please install Rust (https://rustup.rs/) and try again."
    exit 1
fi

# 2. Build RTK
echo "🏗 Building and installing rtk CLI..."
cd rtk
cargo build --release
cargo install --path . --force
cd ..

# 3. Inform user of setup instructions
echo ""
echo "=========================================================="
echo "🎉 RTK CLI installed successfully!"
echo "=========================================================="
echo "To activate transparent terminal filtering in Claude Code, "
echo "add the PreToolUse hook to your .claude/settings.json:"
echo ""
echo '  "hooks": {'
echo '    "PreToolUse": ['
echo '      {'
echo '        "matcher": "Bash",'
echo '        "hooks": ['
echo '          {'
echo '            "type": "command",'
echo '            "command": "bash /path/to/rust-context-engine/hooks/rtk-rewrite.sh",'
echo '            "timeout": 5000'
echo '          }'
echo '        ]'
echo '      }'
echo '    ]'
echo '  }'
echo ""
echo "To synchronize Cursor rules across subfolders, run:"
echo "  rtk sync-rules"
echo "=========================================================="
