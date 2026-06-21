# From Token Saver to Rust Context Engine: The Future of Agentic Optimization

In the early days of AI coding assistants, the primary challenge was cost. Developers quickly realized that sending thousands of lines of logs, build errors, and raw directories to large language models quickly exhausted budgets. Naturally, the first generation of tools focused on one thing: saving tokens.

But the state of the art in 2026 has shifted. Large context windows are now common, but they have brought a new, more insidious problem: **context distraction**. When an LLM receives 100,000 tokens of noisy context, its reasoning degrades. It misses details, focuses on the wrong code, and returns lower-quality responses.

Today, we are proud to announce the rebranding of RTK from a simple "Token Saver" to a full **Rust Context Engine**.

## What is a Context Engine?

A context engine is a local runtime layer that mediates between your local environment (IDE, terminal, code files) and the AI coding agent. Instead of passing massive raw text streams, RTK filters, indexes, and layers context dynamically.

RTK does this through four core pillars:
1. **Source-level Filtering**: Condensing raw shell outputs (git diff, cargo build, pytest logs) by up to 90% while preserving critical errors and warnings.
2. **Structural Code Graph (rtk-index)**: Built on tree-sitter and petgraph, it lets your agent query codebase symbols, find references, and analyze blast radius BEFORE editing.
3. **Multi-tier Memory & Session Handoff**: Archiving project facts, managing session state files, and automating compaction policies.
4. **Tool Sparsity & MCP Server**: Exposing a minimal, highly optimized 7-tool surface via Model Context Protocol (MCP) to reduce agent confusion.

## Measured, Verifiable Savings

RTK is designed with a single source of truth: our database stores exact pricing, tokenizer estimates, and cost projection reports. Every command run under RTK writes a cryptographically signed receipt of saved tokens, allowing team leads and solo developers to check their exact ROI.

For example, a raw `git status` output of 4,000 tokens is condensed to less than 400 tokens, saving over 90% in costs and reducing agent response times by 3x due to less prompt processing latency.

## Getting Started

RTK is a single, zero-dependency Rust binary. Initialize it in your project root:

```bash
rtk init
rtk mcp install --client claude
```

Read more about our pricing methodology in our [documentation](https://github.com/Andrea/rust-context-engine/blob/main/docs/src/pricing.md). Welcome to the era of precise context engineering.
