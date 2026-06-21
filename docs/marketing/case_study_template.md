# RTK Case Study: [Team/Company Name]

This case study documents the efficiency gains, token savings, and cost reductions achieved by [Team/Company Name] after integrating **RTK (Rust Context Engine)** into their AI coding workflows.

## Context & Baseline

- **Team Size**: [Number of developers using AI agents]
- **AI Tools Used**: [e.g., Claude Code, Cursor, GitHub Copilot Workspace, custom agents]
- **Primary Codebase Size**: [e.g., 50k lines of Rust/TypeScript, monorepo]
- **Baseline Monthly API Spending**: [$ USD spent on AI APIs before RTK]
- **Main Pain Points**:
  - [e.g., High cost of long shell command outputs in context]
  - [e.g., Agent getting distracted by build warning walls of text]
  - [e.g., Slow agent response times due to large prompts]

## Implementation

RTK was deployed in [Number] days across the team. The integration involved:
1. **Shell Wrappers**: Aliasing `git`, `cargo`, `npm`, and `pytest` to their filtered RTK equivalents.
2. **MCP Integration**: Adding the `rtk` MCP server to Claude Desktop and Cursor configurations.
3. **Session-state Rules**: Utilizing `.rtk.json` output profiles (`strict` for CI, `balanced` for local development).

## Results & Impact

After [Number] weeks of using RTK, the team gathered the following metrics from `rtk audit` and `rtk stats`:

### 1. Token Reduction & Cost Savings

| Metric | Before RTK | After RTK | Saving % |
|---|---|---|---|
| **Average Prompt Token Count** | [e.g. 45,000] | [e.g. 12,000] | **[e.g. 73%]** |
| **Average Cost per Task** | [e.g. $0.65] | [e.g. $0.18] | **[e.g. 72%]** |
| **Total Monthly Spend** | [$ Baseline] | [$ Post-RTK] | **[Saving %]** |

### 2. Performance & Correctness
- **Response Latency**: Reduced from average [e.g. 15s] to [e.g. 5s] (a **[X]% speedup** due to prompt caching optimization).
- **Task Success Rate (Correctness)**: Improved by **[X]%** because the agent received clean, distraction-free code graphs instead of raw files and redundant compiler logs.

## Testimonials

> "[Quote from developer or tech lead on how RTK changed their daily coding flow with agents]"
>
> — *[Name], [Title]*

## Verified Savings Ledger

The savings claimed in this document are backed by RTK's local SQLite ledger, verified using `rtk stats --json` and `rtk benchmark export`.
