use regex::Regex;
use std::sync::LazyLock;

const MAX_LINE: usize = 80;
/// Hunks with more changed lines than this are collapsed to a summary.
const HUNK_COLLAPSE_THRESHOLD: usize = 8;
/// Number of lines shown before the summary in a collapsed hunk.
const HUNK_HEAD_LINES: usize = 3;

/// Filter `git diff` output.
///
/// Strategy:
///   - Drop: metadata headers (`index`, `---`, `+++`), context lines (space prefix)
///   - Compact: `diff --git` → `[filename]`, `@@ -L,N +L,N @@ …` → `@@ -L +L @@`
///   - Collapse: hunks with >8 changed lines show first 3 + `[…+N/-M more]`
///   - Truncate: lines > 80 chars get `…` suffix
///   - Fallback: return input unchanged if filter produces empty output
pub fn filter(input: &str) -> String {
    static DIFF_FILE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^diff --git a/.+ b/(.+)$").unwrap());
    static HUNK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@").unwrap());

    let mut out = String::with_capacity(input.len() / 4);
    let mut hunk_header = String::new();
    let mut hunk_lines: Vec<String> = Vec::new();

    let flush_hunk = |hdr: &str, lines: &[String], buf: &mut String| {
        if lines.is_empty() {
            return;
        }
        buf.push_str(hdr);
        buf.push('\n');
        if lines.len() <= HUNK_COLLAPSE_THRESHOLD {
            for l in lines {
                push_truncated(buf, l, MAX_LINE);
                buf.push('\n');
            }
        } else {
            for l in lines.iter().take(HUNK_HEAD_LINES) {
                push_truncated(buf, l, MAX_LINE);
                buf.push('\n');
            }
            let rest = &lines[HUNK_HEAD_LINES..];
            let adds = rest.iter().filter(|l| l.starts_with('+')).count();
            let dels = rest.iter().filter(|l| l.starts_with('-')).count();
            let mut summary = String::from("[…");
            if adds > 0 {
                summary.push_str(&format!("+{adds}"));
            }
            if dels > 0 {
                summary.push_str(&format!(" -{dels}"));
            }
            summary.push_str(" more lines]");
            buf.push_str(&summary);
            buf.push('\n');
        }
    };

    for line in input.lines() {
        if let Some(caps) = DIFF_FILE.captures(line) {
            flush_hunk(&hunk_header, &hunk_lines, &mut out);
            hunk_header.clear();
            hunk_lines.clear();
            out.push('[');
            out.push_str(&caps[1]);
            out.push_str("]\n");
        } else if let Some(caps) = HUNK.captures(line) {
            flush_hunk(&hunk_header, &hunk_lines, &mut out);
            hunk_lines.clear();
            hunk_header = format!("@@ -{} +{} @@", &caps[1], &caps[2]);
        } else if line.starts_with("index ")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
            || line.starts_with("deleted file mode")
            || line.starts_with("new file mode")
            || line.starts_with("\\ No newline")
        {
            // metadata noise — drop
        } else if line.starts_with(' ') {
            // context line — drop
        } else if line.starts_with('+') || line.starts_with('-') {
            hunk_lines.push(line.to_string());
        } else if !line.is_empty() {
            // rename/binary/mode lines — keep
            flush_hunk(&hunk_header, &hunk_lines, &mut out);
            hunk_header.clear();
            hunk_lines.clear();
            out.push_str(line);
            out.push('\n');
        }
    }
    flush_hunk(&hunk_header, &hunk_lines, &mut out);

    if out.is_empty() && !input.is_empty() {
        return input.to_string(); // fallback: never blank the user
    }
    out
}

fn push_truncated(buf: &mut String, s: &str, max: usize) {
    if s.len() <= max {
        buf.push_str(s);
    } else {
        let mut boundary = max.saturating_sub(1);
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        buf.push_str(&s[..boundary]);
        buf.push('…');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    fn count_tokens(s: &str) -> usize {
        (s.len() as f64 / 4.0).ceil() as usize
    }

    #[test]
    fn test_snapshot() {
        let input = include_str!("../tests/fixtures/git_diff_raw.txt");
        let output = filter(input);
        assert_snapshot!(output);
    }

    #[test]
    fn test_token_savings() {
        let input = include_str!("../tests/fixtures/git_diff_raw.txt");
        let output = filter(input);
        let savings = 1.0 - (count_tokens(&output) as f64 / count_tokens(input) as f64);
        assert!(
            savings >= 0.60,
            "git diff filter: expected ≥60% savings, got {:.1}%",
            savings * 100.0
        );
    }

    #[test]
    fn test_char_savings() {
        let input = include_str!("../tests/fixtures/git_diff_raw.txt");
        let output = filter(input);
        let savings = 1.0 - (output.len() as f64 / input.len() as f64);
        assert!(
            savings >= 0.60,
            "git diff filter: expected ≥60% char savings, got {:.1}%",
            savings * 100.0
        );
    }

    #[test]
    fn test_empty_input_fallback() {
        assert_eq!(filter(""), "");
    }

    #[test]
    fn test_no_context_lines() {
        let input = "diff --git a/foo.rs b/foo.rs\nindex abc..def\n--- a/foo.rs\n+++ b/foo.rs\n@@ -1,4 +1,4 @@\n fn main() {\n-    old();\n+    new();\n }\n";
        let output = filter(input);
        assert!(!output.contains("fn main()"), "context line leaked");
        assert!(output.contains("-    old();"));
        assert!(output.contains("+    new();"));
        assert!(output.contains("[foo.rs]"));
        assert!(output.contains("@@ -1 +1 @@"));
    }

    #[test]
    fn test_hunk_collapse() {
        // Hunk with 12 changed lines → should collapse to head + summary
        let adds: String = (0..12).map(|i| format!("+line {i}\n")).collect();
        let input = format!(
            "diff --git a/big.rs b/big.rs\nindex 0..1\n--- a/big.rs\n+++ b/big.rs\n@@ -1,12 +1,12 @@\n{adds}"
        );
        let output = filter(&input);
        assert!(output.contains("[…"), "missing collapse summary");
        assert!(output.contains("+line 0"), "first line missing");
        assert!(
            !output.contains("+line 11"),
            "last line should be collapsed"
        );
    }

    #[test]
    fn test_small_hunk_not_collapsed() {
        // Hunk with ≤8 lines → show all
        let changes: String = (0..5).map(|i| format!("+line {i}\n")).collect();
        let input = format!(
            "diff --git a/small.rs b/small.rs\nindex 0..1\n--- a/small.rs\n+++ b/small.rs\n@@ -1,5 +1,5 @@\n{changes}"
        );
        let output = filter(&input);
        assert!(!output.contains("[…"), "should not collapse small hunk");
        assert!(output.contains("+line 4"), "last line should be present");
    }

    #[test]
    fn test_long_line_truncated() {
        let long_plus = format!("+{}", "x".repeat(200));
        let input =
            format!("diff --git a/x b/x\nindex 0..1\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n{long_plus}\n");
        let output = filter(&input);
        let changed_line = output.lines().find(|l| l.starts_with('+')).unwrap();
        assert!(
            changed_line.len() <= MAX_LINE + 4,
            "line not truncated: {} chars",
            changed_line.len()
        );
    }

    #[test]
    fn test_fallback_on_empty_output() {
        let just_context = " line1\n line2\n line3\n";
        let output = filter(just_context);
        assert_eq!(output, just_context);
    }
}
