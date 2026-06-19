# Dynamic Plugins (plugins.toml)

As codebases evolve, you may need to wrap and filter commands for tools that are not built into RTK by default (such as `ruff`, `uv`, `make`, or internal custom scripts). RTK provides a dynamic plugin system that allows you to configure command filters using simple TOML configuration files.

---

## Configuration Paths

RTK searches for dynamic plugins in two locations and merges them:
1. **Global Plugins**: `~/.config/rtk/plugins.toml`
2. **Local Plugins**: `./plugins.toml` (in your current workspace directory)

---

## Writing a Plugin Filter

Each plugin configuration specifies the target binary name (`bin`) and lists of line prefixes to either drop or explicitly keep.

### TOML Format

```toml
[[plugin]]
bin = "ruff"
drop_prefixes = ["ℹ️", "DEBUG", "Analyzing"]
keep_prefixes = ["Error:", "Failure:"]
```

### Configuration Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `bin` | `String` (Required) | The command binary name (e.g. `"ruff"`, `"mypy"`, `"npm"`). If the intercepted command starts with this binary name, the filter is applied. |
| `drop_prefixes` | `Array of Strings` | Any line in the command's stdout or stderr that begins with one of these prefixes will be deleted. |
| `keep_prefixes` | `Array of Strings` | If specified, only lines starting with these prefixes will be preserved (unless they also match a `drop_prefixes` rule). If empty, all lines that are not dropped are kept. |

---

## Example Scenario: Wrapping `ruff` linting

Suppose `ruff check` outputs:
```text
Analyzing 42 files...
ℹ️ 12 files parsed successfully
DEBUG: Line length check complete
Error: src/main.rs:12:8: Unused import `std::collections::HashMap`
Found 1 lint error.
```

With the following configuration:
```toml
[[plugin]]
bin = "ruff"
drop_prefixes = ["ℹ️", "DEBUG", "Analyzing"]
keep_prefixes = ["Error:", "Found"]
```

RTK processes the output, strips out the noisy `Analyzing`, `ℹ️`, and `DEBUG` lines, and returns only:
```text
Error: src/main.rs:12:8: Unused import `std::collections::HashMap`
Found 1 lint error.
```
This reduces the tokens from ~45 to ~12 (a **73%** reduction), preventing the LLM from reading irrelevant status logs.

---

## Executing Plugin Commands

Once configured in a `plugins.toml` file, run the tool using `rtk plugin`:

```bash
rtk plugin -- ruff check .
```

If your shell integration is loaded and the command is aliased, running `ruff check .` will automatically trigger RTK's plugin wrapper under the hood.
