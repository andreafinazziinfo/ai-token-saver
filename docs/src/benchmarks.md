# Compression Benchmarks

This page documents the empirical token savings achieved by RTK's compression filters on real-world outputs.

## Filter Effectiveness Summary

The table below summarizes average token reductions for common development tasks:

| Command | Raw Output Size (Tokens) | Filtered Size (Tokens) | Token Savings (%) | Key Retention |
|---------|-------------------------|------------------------|-------------------|---------------|
| `git status` | 150 - 400 | 40 - 60 | **75% - 85%** | Modified, untracked, conflicts |
| `git diff` | 1,000 - 15,000 | 250 - 2,500 | **65% - 80%** | Added/removed changes, signature context |
| `cargo build` | 2,000 - 8,000 | 300 - 1,200 | **80% - 90%** | Compilation warnings & errors |
| `cargo test` | 1,500 - 10,000 | 200 - 1,000 | **85% - 95%** | Failed tests, summaries, stacktraces |
| `pytest` | 800 - 5,000 | 150 - 800 | **80% - 90%** | Failed test outputs & traceback |

## Real-World Case Study: Workspace Test Suite

Using the `rtk benchmark` tool on the RTK Rust project containing 118 tests:

- **Raw Output Size**: 2,459 tokens (incremental test suite execution).
- **RTK Filtered Output**: 548 tokens.
- **Total Savings**: **77.7%**
- **Financial Savings**: Saved **$0.0057** on input context for a single run using Claude Sonnet 4.6 pricing. Over 100 runs per day, this saves **$0.57/developer/day**.

## Reproducibility

Benchmarks can be replicated locally by running:

```bash
rtk benchmark export --format json --output benchmarks_report.json
```
