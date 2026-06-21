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

# 4. Prompt for PATH configuration
echo ""
echo "❓ Do you want to prepend ~/.rtk/bin to your PATH in ~/.bashrc / ~/.zshrc?"
echo "   This enables transparent CLI interception (interceptor wrappers for git, cargo, docker, etc.)."
read -p "   Enable wrappers? (y/N): " -n 1 -r REPLY
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    SHELL_PROFILE=""
    if [ -f "$HOME/.zshrc" ]; then
        SHELL_PROFILE="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        SHELL_PROFILE="$HOME/.bashrc"
    fi
    
    if [ -n "$SHELL_PROFILE" ]; then
        if ! grep -q '\.rtk/bin' "$SHELL_PROFILE"; then
            echo 'export PATH="$HOME/.rtk/bin:$PATH"' >> "$SHELL_PROFILE"
            echo "✅ Added ~/.rtk/bin to PATH in $SHELL_PROFILE"
            echo "👉 Please run: source $SHELL_PROFILE to apply changes."
        else
            echo "ℹ️  ~/.rtk/bin is already in your PATH inside $SHELL_PROFILE"
        fi
    else
        echo "⚠️  Could not find ~/.bashrc or ~/.zshrc. Please manually prepend ~/.rtk/bin to your PATH."
    fi
fi
echo "=========================================================="

