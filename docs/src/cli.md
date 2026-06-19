# CLI Reference

RTK provides subcommands for input virtualization, semantic memory, configuration, directory packaging, and system auditing.

---

## 1. Audit Subcommand

### `rtk audit`
Analyses your project invocation logs and prints developer productivity metrics, token savings, and cost reductions.
* **Stdout**: Prints a formatted ASCII table showing savings.
* **File Output**: Writes a comprehensive report to `rtk-audit.md`.

```bash
rtk audit
```

---

## 2. Directory Packaging

### `rtk pack [path]`
Aggregates workspace source files into a single structured XML envelope for AI consumption.
* `--strip`: Strips comments and empty lines from source code.
* `--skeleton`: Uses Tree-Sitter parsing to collapse function/method bodies, returning only API signatures.
* `--limit <tokens>`: Fails with an error if the packaged result exceeds the specified token threshold.

```bash
rtk pack src/ --strip --skeleton --limit 20000
```

---

## 3. Semantic Memory

### `rtk memory set <key> <value>`
Saves a variable into the local SQLite memory vault.
```bash
rtk memory set db_port 5432
```

### `rtk memory get <key>`
Retrieves a saved variable.
```bash
rtk memory get db_port
```

### `rtk memory list`
Lists all active key-value pairs stored in the project's memory.
```bash
rtk memory list
```

### `rtk memory search <query>`
Runs a full-text FTS5 search on your memory vault.
```bash
rtk memory search "database connection"
```

---

## 4. Reasoning Management

### `rtk think`
Reads from standard input (`stdin`) and saves your reasoning steps to SQLite. Rather than dumping long thought processes to the LLM, the model uses this command to keep context lightweight.
```bash
echo "Refactoring main engine. Step 1: rename variables. Step 2: run tests." | rtk think
```

---

## 5. Administration & Optimization

### `rtk init`
Sets up user aliases and default guardrail rules.
* `--profile <low|medium|high|max>`: Installs AI assistant rules based on the desired strictness profile.

```bash
rtk init --profile high
```

### `rtk show-log <id>`
Retrieves the raw uncompressed output of a previously filtered command.
```bash
rtk show-log 42
```

### `rtk gc`
Runs a garbage collection cycle on the SQLite database, purging logs older than 7 days and running SQLite's `VACUUM` to reclaim disk space.
```bash
rtk gc
```

### `rtk status` / `rtk stats` / `rtk dashboard`
Displays performance telemetry and active configurations.
```bash
rtk status
```
