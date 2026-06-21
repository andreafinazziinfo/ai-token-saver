# Use Cases & Integration Scenarios

RTK integrates smoothly into different development environments, enabling automated token efficiency.

## 1. Local Agent Proxy (Cursor, Claude Code, OpenCode)

By installing RTK as a global shell proxy, local coding agents automatically benefit from compression:

- **Claude Code**: Set command aliases inside shell config (`alias git="rtk git"`, `alias cargo="rtk cargo"`).
- **Cursor**: Configure terminal command interception.
- **MCP Server (Planned)**: Minimal stdio-based tool routing for IDEs supporting MCP.

## 2. CI/CD Pipeline Cost Protection

Integrate RTK into your GitHub Actions, GitLab CI, or Jenkins pipelines to guard against high AI consumption costs.

```yaml
# GitHub Actions Example
steps:
  - name: Install RTK
    run: cargo install --path ./rtk
  - name: Run Build and Test with RTK Audit
    run: |
      rtk cargo build
      rtk cargo test
  - name: Generate Cost Savings Report
    run: rtk audit --output rtk-ci-report.md
```

## 3. Multi-Agent Swarm Orchestration

In large projects using multiple autonomous subagents, use project-local `.rtk/session_state.db` to coordinate agent state:

- **Handoffs**: Agent A writes its state (`rtk session-state update decisions '["Implemented component X"]'`).
- **Resuming**: Agent B reads state (`rtk session-state get`) to immediately catch up without reading historical logs.
