# Installation & Setup

Setting up RTK takes just a few steps. Follow this guide to build the binaries, install the CLI, and integrate RTK with your favorite terminal or AI environment.

---

## Prerequisites

* **Rust Toolchain**: You will need Cargo and Rust 1.75+. You can get Rust from [rustup.rs](https://rustup.rs/).
* **WSL/Linux/macOS Environment**: RTK is optimized for POSIX environments. Under Windows, run RTK inside Windows Subsystem for Linux (WSL).

---

## 1. Quick Install

Clone the repository and run the installation script:

```bash
git clone https://github.com/andreafinazziinfo/ai-token-saver.git
cd ai-token-saver
bash install.sh
```

The script will compile the release binary (`rtk`) and copy it to your path (typically `~/.cargo/bin/rtk`).

---

## 2. Shell Integration

RTK works by intercepting common development commands using shell aliases. 

### Automated Setup
To configure your shell automatically, initialize your desired profile:

```bash
rtk init --profile high
```

This command:
1. Generates and installs a block of aliases in your terminal configuration files (`~/.bashrc`, `~/.zshrc`, or `~/.profile`).
2. Configures default guardrail rules and workspace settings.

### Manual Shell Aliases
If you prefer to configure aliases manually, append the following to your `~/.bashrc` or `~/.zshrc`:

```bash
alias git="rtk git"
alias cargo="rtk cargo"
alias pytest="rtk pytest"
alias ls="rtk ls"
alias npm="rtk npm"
alias yarn="rtk yarn"
alias pnpm="rtk pnpm"
alias dotnet="rtk dotnet"
alias composer="rtk composer"
alias terraform="rtk terraform"
```

Once added, reload your shell:
```bash
source ~/.bashrc  # or source ~/.zshrc
```

---

## 3. IDE & AI CLI Hook Integrations

Integrating RTK directly into the AI toolchain ensures that commands typed by the AI are transparently rewritten without you needing to do it manually.

### For Claude Code (PreToolUse Hook)
Claude Code supports intercepting tool executions before they run. Add the PreToolUse hook configuration to your user settings (e.g., `~/.claude/settings.json` or `%USERPROFILE%\.gemini\antigravity\settings.json`):

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash /absolute/path/to/ai-token-saver/hooks/rtk-rewrite.sh",
            "timeout": 5000
          }
        ]
      }
    ]
  }
}
```

This hook uses the `rtk rewrite` engine. When Claude tries to run `cargo test`, RTK intercepts it and rewrites it to `rtk cargo test`. If Claude tries to run a denied command (e.g., `git push --force`), RTK blocks execution.

### For Cursor, Windsurf, or Aider
These tools execute commands in a subshell that respects your user's shell configurations. Having the aliases in `~/.bashrc` or `~/.zshrc` is sufficient to activate RTK automatically when these agents run commands.
