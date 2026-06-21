# 📊 RTK Efficiency & Token Savings Audit

Generated on: 2026-06-22 00:58:10 (local time)

## 📈 Summary Statistics

| Metric | Value |
| :--- | :--- |
| **Total Commands Intercepted** | 48 |
| **Original Tokens** | 32523 |
| **Filtered Tokens** | 10899 |
| **Tokens Saved** | 21624 (66.5%) |
| **Estimated API Cost Saved (Dynamic)** | $0.0649 USD |
| **Estimated Developer Hours Saved** | 0.30 hrs |

## 💰 Cost Savings Projection by Model

This table projects what would have been saved under different LLM pricing models for the same volume of saved tokens (21624 tokens):

| Model | Input Price / MTok | Estimated Savings |
| :--- | ---: | ---: |
| **Claude Opus 4.8** | $5.00 | $0.1081 |
| **Claude Sonnet 4.6** | $3.00 | $0.0649 |
| **GPT-5.5** | $5.00 | $0.1081 |
| **GPT-5.4** | $2.50 | $0.0541 |
| **Gemini 3.1 Pro Preview** | $2.00 | $0.0432 |
| **Gemini 3.5 Flash** | $1.50 | $0.0324 |

> [!NOTE]
> Savings calculations are based on input token reductions. Wait-time savings are calculated at a conservative rate of 22.8 seconds of developer waiting time saved per command.

## 🗃️ Command Breakdown

| Command | Invocations | Original Tokens | Filtered Tokens | Tokens Saved | Savings % |
| :--- | ---: | ---: | ---: | ---: | ---: |
| `git status` | 26 | 4626 | 3513 | 1113 | 24.1% |
| `cargo test` | 15 | 25614 | 6085 | 19529 | 76.2% |
| `cargo check` | 6 | 2156 | 1282 | 874 | 40.5% |
| `cargo build` | 1 | 127 | 19 | 108 | 85.0% |
