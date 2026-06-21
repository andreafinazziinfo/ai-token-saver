# Limitations & Constraints

RTK is designed to maximize savings, but it has constraints that developers should keep in mind.

## 1. Context Truncation Risk

RTK uses filters to drop passing test names, compile check progress lines, and long lines of context.
- **Limitation**: If a test framework prints critical warnings inside a passing block that doesn't trigger standard error keywords, RTK might filter it out.
- **Workaround**: Configure the `developer` or `balanced` output profile in `.rtk.json` to keep more verbose logging if troubleshooting complex environment setup.

## 2. AST Parsing Boundaries

The `rtk-index` crate relies on tree-sitter to parse syntax structures:
- **Limitation**: Dynamic imports, metaprogramming structures (like macros in Rust, decorator-heavy class overrides in Python, or JavaScript `eval`), and reflection cannot be statically resolved.
- **Workaround**: Complement impact analysis with global search when working with high levels of runtime metaprogramming.

## 3. SQLite Database Locking

The project-local `.rtk/*.db` SQLite databases utilize WAL mode or standard locking:
- **Limitation**: Executing multiple commands in parallel can occasionally trigger database lock contentions if multiple agent instances write simultaneously.
- **Workaround**: Use the built-in static synchronization locks in test targets, or serial execution wrappers in agent mesh scripts.
