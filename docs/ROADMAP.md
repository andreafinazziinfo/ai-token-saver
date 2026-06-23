# RTK Roadmap

Public snapshot for **v2.3.0**. Execution plan (what to do now): **[PLAN_NOW.md](./PLAN_NOW.md)**. Sprint audit (archived): [archive/IMPROVEMENT_PLAN_9PLUS.md](./archive/IMPROVEMENT_PLAN_9PLUS.md).

## Shipped (release quality)

- Command filters + DLP redaction (15+ wrappers)
- `rtk pack` with `--strip`, `--skeleton`, `--limit`
- SQLite memory, artifacts, session state
- Tree-sitter code graph (`symbols`, `refs`, `impact`, lazy index)
- MCP server (8 tools, version parity with CLI)
- CI matrix (Linux / Windows / macOS), release smoke, savings regression gate
- Homebrew formula (`rtk.rb`) pinned v2.3.0 + sha256; fmt pre-push hook

## Adesso (Fase A — see PLAN_NOW.md) ✅

Completed v2.3.0+ — see [QUICKSTART.md](./QUICKSTART.md).

## Subito dopo (Fase B)

| Item |
|------|
| Native npm/yarn filters (`NEXT-1`) |
| Targeted tests dlp/rewrite/pipeline (`NEXT-2`) |
| Golden ls/npm (`NEXT-3`) |
| Split user vs contributor docs (`NEXT-4`) |
| Windows prebuilt quickstart (`NEXT-6`) |

## Won't do (unless product direction changes)

- Embeddings in **default** binary (+15MB ONNX) — stays optional `--features embeddings`
- Filesystem index watcher — manual `rtk index run` after large refactors
- Chasing scorecard 9.5 everywhere without user-facing benefit

## Contributing

```bash
bash scripts/setup-githooks.sh   # once per clone
bash scripts/dev-gate.sh         # fmt + clippy + test before PR
```
