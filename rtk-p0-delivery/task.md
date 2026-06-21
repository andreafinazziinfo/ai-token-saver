# P0 — Foundation & Data Layer (0-30 giorni)

## Phase 0.0: Crate Restructuring
- [x] Rename `rtk-memory` → `rtk-db` (directory, Cargo.toml, all `use` statements)
- [x] Update workspace `Cargo.toml` members
- [x] Update all dependents (`rtk-cli/Cargo.toml`, `main.rs`, `dashboard.rs`, `dotnet.rs`)
- [x] Run `cargo test` — verify 0 regressions

## Phase 0.1: Pricing Registry & Cost Calculator
- [x] Create `data/model_pricing.json` (all current LLM models + pricing)
- [x] Create `rtk-db/src/pricing.rs` (load, query, calculate)
- [x] Refactor `tracking.rs` — replace hardcoded `$3.00/MTok` with pricing module
- [x] Refactor `run_audit()` — use shared cost calculator
- [x] Refactor `dashboard.rs` `/api/stats` — use shared cost calculator
- [x] Run `cargo test` — verify 0 regressions

## Phase 0.2: Benchmark Export (JSON/CSV)
- [x] Create `rtk-cli/src/benchmark.rs` (export_json, export_csv)
- [x] Add `rtk benchmark export` subcommand to `main.rs`
- [x] Verify: `rtk benchmark export --format json --output bench.json`

## Phase 0.3: `rtk doctor`
- [x] Create `rtk-cli/src/doctor.rs` (health checks)
- [x] Add `rtk doctor` subcommand to `main.rs`
- [x] Verify: `rtk doctor` shows green checklist

## Phase 0.4: `rtk session-state`
- [x] Create `rtk-db/src/session.rs` (SQLite table + CRUD)
- [x] Add `rtk session-state init|get|update|export` to `main.rs`
- [x] Verify: `rtk session-state init && rtk session-state get`

## Phase 0.5: `rtk agents`
- [x] Create `rtk-cli/src/agents.rs` (init, doctor, compact)
- [x] Add `rtk agents init|doctor|compact` to `main.rs`
- [x] Verify: `rtk agents init --template solo-dev`

## Phase 0.6: `rtk think` Enhancements
- [x] Add `think_inspect()` and `think_gc()` to `think.rs`
- [x] Expand `Think` subcommand with `inspect` and `gc`
- [x] Verify: `rtk think inspect`

## Final P0 Validation
- [x] `cargo test` — all tests pass (concurrency test lock added)
- [x] `cargo build --release` — optimized size with LTO, opt-level=z (result: 8.9MB due to tree-sitter grammars and embedded SQLite)
- [x] `rtk doctor` — all checks green
- [x] GitNexus `detect-changes` — verified scope and processes
