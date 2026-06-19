# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
*   Local compliance audit logger for Data Loss Prevention (DLP) redactions, recording timestamps, context/source tools, and secure cryptographic hashes of redacted secrets to `~/.config/rtk/audit.log`.
*   Troubleshooting guidelines for shell profiles, WSL pathways, and database locks in `README.md`.
*   Safety disclaimers and bypass warning documentation for transparent CLI aliases.

### Changed
*   Updated benchmark engine and cost savings projections to use real-world state-of-the-art model pricing (Claude 3.5 Sonnet, Claude 3 Opus, GPT-4o, Gemini 1.5 Pro/Flash).

---

## [0.1.0] - 2026-06-19

### Added
*   Initial release of RTK (Runtime Token Toolkit).
*   15 input virtualization CLI wrappers for noisy tool outputs (`git`, `cargo`, `pytest`, `docker`, `npm`, `yarn`, `pnpm`, `composer`, `terraform`, `dotnet`, `gradle`, `go_test`, `ls`).
*   `rtk think` reasoning offloader and persistent project memory commands (`rtk memory`).
*   `rtk pack` Tree-Sitter AST context packaging engine with minification and signature skeletal structures.
*   Data Loss Prevention (DLP) engine with regex pattern matching and Shannon-entropy scanner.
*   `rtk init` rule system bootstrapping for Claude Code, Cursor, and Windsurf AI profiles (*Caveman* and *Ponytail* response rules).
