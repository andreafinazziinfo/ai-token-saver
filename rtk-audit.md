# 📊 RTK Efficiency & Token Savings Audit

Generated on: 2026-06-20 00:40:37 (local time)

## 📈 Summary Statistics

| Metric | Value |
| :--- | :--- |
| **Total Commands Intercepted** | 198 |
| **Original Tokens** | 69996 |
| **Filtered Tokens** | 18763 |
| **Tokens Saved** | 51233 (73.2%) |
| **Estimated API Cost Saved (Sonnet)** | $0.1537 USD |
| **Estimated Developer Hours Saved** | 1.25 hrs |

## 💰 Cost Savings Projection by Model

This table projects what would have been saved under different LLM pricing models for the same volume of saved tokens (51233 tokens):

| Model | Input Price / MTok | Estimated Savings |
| :--- | ---: | ---: |
| **Claude 3 Opus** | $15.00 | $0.7685 |
| **Claude 3.5 Sonnet** | $3.00 | $0.1537 |
| **GPT-4o** | $2.50 | $0.1281 |
| **Gemini 1.5 Pro** | $1.25 | $0.0640 |
| **GPT-4o mini** | $0.15 | $0.0077 |
| **Gemini 1.5 Flash** | $0.075 | $0.0038 |

> [!NOTE]
> Savings calculations are based on input token reductions. Wait-time savings are calculated at a conservative rate of 22.8 seconds of developer waiting time saved per command.

## 🗃️ Command Breakdown

| Command | Invocations | Original Tokens | Filtered Tokens | Tokens Saved | Savings % |
| :--- | ---: | ---: | ---: | ---: | ---: |
| `git status` | 91 | 10850 | 5970 | 4880 | 45.0% |
| `git log` | 55 | 24726 | 9082 | 15644 | 63.3% |
| `git diff` | 26 | 5365 | 2335 | 3030 | 56.5% |
| `cargo test` | 24 | 29040 | 1368 | 27672 | 95.3% |
| `think` | 2 | 15 | 8 | 7 | 46.7% |
