use regex::Regex;
use std::sync::LazyLock;

/// Filter `mypy` output.
///
/// With `--pretty` (and often in CI) mypy prints, per diagnostic, the
/// `file:line: error: ...` message, an indented source code-frame with a caret
/// line, and wraps long messages onto unprefixed continuation lines. For an
/// agent the actionable signal is the single reconstructed diagnostic line.
///
/// Strategy:
///   - Keep every `file:line[:col]: error|note:` diagnostic (notes included).
///   - Re-join wrapped message continuations onto their diagnostic line.
///   - Drop the indented source code-frame and caret lines.
///   - Keep the trailing summary (`Found N errors`, `Success: ...`).
///   - Fallback: return input unchanged if the filter produces empty output.
pub fn filter(input: &str) -> String {
    // Diagnostic line: `path:line: error: msg` or `path:line:col: note: msg`.
    static DIAG: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(\S+:\d+:(?:\d+:)?)\s+(error|note|warning):\s?(.*)$").unwrap()
    });
    // Final summary / status lines.
    static SUMMARY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(Found \d+ error|Success: no issues|No issues found)").unwrap()
    });

    // Caret line of a `--pretty` code-frame, e.g. `        ^~~~~`.
    static CARET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*\^[~^]*\s*$").unwrap());

    // `--pretty` frames are always a (source, caret) pair. Mark caret lines and
    // the source line immediately above them for dropping. This is robust to
    // mypy wrapping long messages onto space-prefixed continuation lines, which
    // an indentation heuristic would misclassify as frame source.
    let lines: Vec<&str> = input.lines().collect();
    let mut drop = vec![false; lines.len()];
    for (i, line) in lines.iter().enumerate() {
        if CARET.is_match(line) {
            drop[i] = true;
            if i > 0 {
                drop[i - 1] = true;
            }
        }
    }

    let mut out = String::with_capacity(input.len() / 2);
    let mut current: Option<String> = None;

    let flush = |current: &mut Option<String>, out: &mut String| {
        if let Some(line) = current.take() {
            out.push_str(&line);
            out.push('\n');
        }
    };

    for (i, line) in lines.iter().enumerate() {
        if drop[i] {
            continue;
        }

        if let Some(caps) = DIAG.captures(line) {
            flush(&mut current, &mut out);
            current = Some(format!("{} {}: {}", &caps[1], &caps[2], caps[3].trim()));
            continue;
        }

        if SUMMARY.is_match(line) {
            flush(&mut current, &mut out);
            out.push_str(line);
            out.push('\n');
            continue;
        }

        // A non-diagnostic, non-empty line is a wrapped message continuation
        // belonging to the current diagnostic (mypy wraps at word boundaries,
        // sometimes with a leading space).
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if let Some(cur) = current.as_mut() {
                cur.push(' ');
                cur.push_str(trimmed);
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    flush(&mut current, &mut out);

    let trimmed = out.trim_end();
    if trimmed.is_empty() {
        return input.to_string(); // fallback / empty passthrough
    }
    format!("{trimmed}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drops_frame_and_rejoins_wrapped() {
        let input = concat!(
            "svc.py:25: error: Argument 1 to \"greet\" has incompatible type \"int\"; expected\n",
            "\"str\"  [arg-type]\n",
            "        greet(42)\n",
            "              ^~\n",
            "Found 1 error in 1 file (checked 1 source file)\n",
        );
        let out = filter(input);
        assert_eq!(
            out.lines().next().unwrap(),
            "svc.py:25: error: Argument 1 to \"greet\" has incompatible type \"int\"; expected \"str\"  [arg-type]"
        );
        assert!(!out.contains("greet(42)"), "code-frame leaked");
        assert!(!out.contains("^~"), "caret leaked");
        assert!(out.contains("Found 1 error"), "summary missing");
    }

    #[test]
    fn test_keeps_notes() {
        let input = concat!(
            "over.py:13: error: No overload variant of \"f\" matches argument type \"list[int]\"\n",
            " [call-overload]\n",
            "        f([1, 2])\n",
            "        ^~~~~~~~~\n",
            "over.py:13: note: Possible overload variants:\n",
            "over.py:13: note:     def f(x: int) -> int\n",
            "over.py:15: note: Revealed type is \"int\"\n",
        );
        let out = filter(input);
        assert!(out.contains("[call-overload]"), "wrapped code missing");
        assert!(out.contains("note: Possible overload variants:"));
        assert!(out.contains("note: def f(x: int) -> int"));
        assert!(out.contains("note: Revealed type is \"int\""));
        assert!(!out.contains("f([1, 2])"), "code-frame leaked");
    }

    #[test]
    fn test_plain_output_passthrough() {
        // Non-pretty mypy is already one line per diagnostic: kept verbatim.
        let input = "svc.py:20: error: Function is missing a type annotation  [no-untyped-def]\nFound 1 error in 1 file (checked 1 source file)\n";
        let out = filter(input);
        assert!(out
            .contains("svc.py:20: error: Function is missing a type annotation  [no-untyped-def]"));
        assert!(out.contains("Found 1 error"));
    }

    #[test]
    fn test_success_line() {
        let input = "Success: no issues found in 3 source files\n";
        assert_eq!(
            filter(input).trim(),
            "Success: no issues found in 3 source files"
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(filter(""), "");
    }

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().map(str::len).sum::<usize>().max(1)
    }

    #[test]
    fn token_savings_pretty_run() {
        let input = include_str!("../tests/fixtures/mypy_pretty.txt");
        let out = filter(input);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(input) as f64;
        // mypy's default output is already one line per diagnostic, so there is
        // little to strip; the win is collapsing `--pretty` frames and re-joining
        // wrapped messages into clean single lines. This word-length metric
        // ignores whitespace (most of a frame), so it understates the real BPE
        // token saving — hence the modest floor.
        assert!(
            savings >= 0.15,
            "mypy filter: expected ≥15% savings on --pretty, got {:.1}%",
            savings * 100.0
        );
        // All diagnostics survive; frames do not.
        // Wrapped exactly at the `  [union-attr]` separator, so the rejoin
        // yields a single space before the code — content preserved regardless.
        assert!(out.contains("svc.py:27: error: Item \"None\" of \"Account | None\" has no attribute \"withdraw\" [union-attr]"));
        assert!(out.contains("over.py:13: note: Possible overload variants:"));
        assert!(out.contains("Found 7 errors"));
        assert!(!out.contains("acc.withdraw(10)"), "code-frame leaked");
    }
}
