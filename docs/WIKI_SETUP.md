# GitHub Wiki setup (optional)

The **source of truth** for docs is `docs/` in the repo (versioned with releases).  
Use the Wiki as a **friendly entry point** that links to those files — do not duplicate long content.

## Enable Wiki

1. GitHub repo → **Settings** → **Features** → check **Wikis**
2. Tab **Wiki** → **Create the first page** → paste [WIKI_HOME.md](./WIKI_HOME.md)
3. Sidebar (Edit): add links to key pages below

## Recommended wiki pages (short)

| Wiki page | Content |
|-----------|---------|
| **Home** | Paste `WIKI_HOME.md` |
| **Quickstart** | Link only: `https://github.com/.../blob/main/docs/QUICKSTART.md` |
| **Community** | Link: Discussions + `docs/COMMUNITY.md` |
| **FAQ** | 5–10 Q&A from Discussions over time |

## Sync strategy

- **Do not** maintain two copies of QUICKSTART/USER in Wiki.
- Update Wiki **Home** only on major releases (v2.4, etc.).
- User questions → **Discussions** (not Wiki edits).

## Optional: clone wiki locally

```bash
git clone https://github.com/andreafinazziinfo/rust-context-engine.wiki.git
# edit Home.md, push
```

Requires wiki write access on the repo.

## Wiki vs mdBook

This repo has `docs/book.toml` for [mdBook](https://rust-lang.github.io/mdBook/) (`mdbook serve` in `docs/`).  
Pick one long-form doc channel:

- **mdBook** — developer docs from repo CI (future)
- **Wiki** — lightweight landing for GitHub users

For v2.3.1: **Discussions + `docs/`** is enough; Wiki is optional polish.
