# Limitations & Constraints

RTK is designed to maximize savings, but it has constraints that developers should keep in mind.

## 1. Context Truncation Risk

RTK uses filters to drop passing test names, compile check progress lines, and long lines of context.

- **Limitation**: If a test framework prints critical warnings inside a passing block that doesn't trigger standard error keywords, RTK might filter it out.
- **Workaround**: Configure the `developer` or `balanced` output profile in `.rtk.json` to keep more verbose logging if troubleshooting complex environment setup.

## 2. Shell Alias Bypass

RTK filters apply when commands run through `rtk` wrappers or the PreToolUse hook (`rtk rewrite`).

- **Limitation**: An agent that invokes `/usr/bin/git` or `command git` directly bypasses RTK aliases and filters.
- **Workaround**: Install the PreToolUse hook (`hooks/rtk-rewrite.sh`) and verify with `rtk doctor`.

## 3. Chained Commands and Guardrails

Since v2.3+, **deny rules** apply to each segment of chained commands (`&&`, `;`, `||`, `|`).

- **Limitation**: Rewrite and ask rules still **pass through** chained commands unchanged (exit code 1). Only deny rules run per segment.
- **Workaround**: Enable strict chained handling when available (`strict_chained` in config, planned). Prefer single commands in agent policies.

## 4. DLP / Secret Redaction

DLP runs on filter output and pack content. High-entropy tokens and known patterns (JWT, API keys, URIs) are redacted.

- **Limitation**: Short secrets, structured tokens with low entropy, or secrets split across lines may evade detection. Git commit hashes (40 hex chars) are intentionally preserved.
- **Workaround**: Add custom patterns via `rtk config dlp add`. Never rely on DLP as the only security control.

## 5. Code Index (Lazy Auto-Index)

Symbol search (`rtk symbols`, `refs`, `impact`, MCP `find_symbols`) triggers indexing when the index is empty or older than 24 hours.

- **Limitation**: First query after clone may take seconds on large repos. Stale index refreshes at most daily unless you run `rtk index run`.
- **Workaround**: Run `rtk index run` after large refactors; check freshness with `rtk index status`.

## 6. AST Parsing Boundaries

The `rtk-index` crate relies on tree-sitter to parse syntax structures:

- **Limitation**: Dynamic imports, metaprogramming (Rust macros, heavy Python decorators, `eval`), and reflection cannot be statically resolved.
- **Workaround**: Complement impact analysis with global search when working with high levels of runtime metaprogramming.

## 7. Memory Search (Default Build)

Project memory uses **SQLite FTS5** keyword search in the default release binary.

- **Limitation**: Hybrid vector / ONNX semantic search requires building with `--features embeddings`. README diagrams may show ONNX; that path is optional.
- **Workaround**: Use precise keywords in `rtk memory search`, or build with the `embeddings` feature for semantic recall.

## 8. SQLite Database Locking

The project-local `.rtk/*.db` SQLite databases utilize WAL mode with a busy timeout:

- **Limitation**: Many parallel writers (multiple agent instances) can still hit transient lock errors under heavy load.
- **Workaround**: Serialize memory writes in agent scripts, or set `RTK_DB_PATH` / `RTK_PROJECT_DB_PATH` per workspace.
