/// Filter `cargo test` output.
///
/// Strategy:
/// - Drop: "test <name> ... ok" lines (passing tests — pure noise).
/// - Drop: build preamble (Compiling, Checking, Finished, Running, Locking, Downloading, Downloaded, Adding).
/// - Keep: everything else — "running N tests", FAILED lines, failures section, summaries, errors, ignored, doc-test output.
/// - Fallback: return input unchanged if filter produces empty output.
///
/// Safety: a passing test line must match EXACTLY "test <…> ... ok".
/// The " ... ok" suffix is required, so stdout inside a failing test that
/// happens to start with "test " is not dropped.
pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len() / 5);

    for line in input.lines() {
        // Drop build preamble (cargo progress lines, always on stderr)
        if line.starts_with("   Compiling ")
            || line.starts_with("    Checking ")
            || line.starts_with("    Finished ")
            || line.starts_with("     Running ")
            || line.starts_with("   Locking ")
            || line.starts_with("      Adding ")
            || line.starts_with("  Downloading ")
            || line.starts_with("  Downloaded ")
            || line.starts_with("   Downloaded ")
            || line.starts_with(" Downloading ")
        {
            continue;
        }

        // Drop passing and ignored test lines.
        // Requires " ... ok" / " ... ignored" suffix — tighter than just " ok".
        // The "test result:" summary already captures totals for both.
        if line.trim_start().starts_with("test ")
            && (line.ends_with(" ... ok") || line.ends_with(" ... ignored"))
        {
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    // Trim excessive trailing blank lines to one
    let trimmed = out.trim_end();
    if trimmed.is_empty() && !input.trim().is_empty() {
        return input.to_string(); // fallback: never blank the user
    }
    format!("{trimmed}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_tokens(s: &str) -> usize {
        (s.len() as f64 / 4.0).ceil() as usize
    }

    #[test]
    fn test_snapshot_cargo_test_passing() {
        use insta::assert_snapshot;
        let input = include_str!("../tests/fixtures/cargo_test_passing.txt");
        assert_snapshot!(filter(input));
    }

    #[test]
    fn drops_passing_tests_and_preamble() {
        let input = concat!(
            "   Compiling rtk v0.1.0 (/path)\n",
            "    Finished `test` profile in 1.00s\n",
            "     Running unittests src/main.rs\n",
            "\n",
            "running 3 tests\n",
            "test foo::bar ... ok\n",
            "test foo::baz ... ok\n",
            "test foo::qux ... ok\n",
            "\n",
            "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured\n",
        );
        let out = filter(input);
        assert!(!out.contains("Compiling"), "Compiling leaked");
        assert!(!out.contains("Finished"), "Finished leaked");
        assert!(!out.contains("Running"), "Running leaked");
        assert!(!out.contains("test foo::bar"), "passing test line leaked");
        assert!(out.contains("running 3 tests"), "'running N tests' missing");
        assert!(out.contains("test result: ok"), "test result missing");
    }

    #[test]
    fn keeps_failed_tests() {
        let input = concat!(
            "running 2 tests\n",
            "test foo::pass ... ok\n",
            "test foo::fail ... FAILED\n",
            "\n",
            "failures:\n",
            "\n",
            "---- foo::fail stdout ----\n",
            "thread 'foo::fail' panicked at 'assertion failed'\n",
            "\n",
            "failures:\n",
            "    foo::fail\n",
            "\n",
            "test result: FAILED. 1 passed; 1 failed; 0 ignored\n",
        );
        let out = filter(input);
        assert!(!out.contains("test foo::pass"), "passing test leaked");
        assert!(
            out.contains("test foo::fail ... FAILED"),
            "FAILED test missing"
        );
        assert!(out.contains("failures:"), "failures section missing");
        assert!(out.contains("assertion failed"), "panic message missing");
        assert!(
            out.contains("test result: FAILED"),
            "result summary missing"
        );
    }

    #[test]
    fn drops_ignored_tests_keeps_summary() {
        // Individual "... ignored" lines are dropped; the count is preserved in the result summary.
        let input = "running 2 tests\ntest foo::bar ... ok\ntest foo::skip ... ignored\n\ntest result: ok. 1 passed; 0 failed; 1 ignored\n";
        let out = filter(input);
        assert!(
            !out.contains("test foo::skip"),
            "ignored test line should be dropped"
        );
        assert!(
            out.contains("1 ignored"),
            "ignored count in result summary must be kept"
        );
    }

    #[test]
    fn false_positive_safety() {
        // A debug line inside a failure dump that starts with "test " but lacks " ... ok"
        let input = concat!(
            "failures:\n",
            "\n",
            "---- foo::bar stdout ----\n",
            "test value was ok but got wrong result\n",
            "\n",
            "test result: FAILED. 0 passed; 1 failed\n",
        );
        let out = filter(input);
        assert!(
            out.contains("test value was ok but got wrong result"),
            "false-positive drop: debug line inside failure dump"
        );
    }

    #[test]
    fn token_savings_passing_suite() {
        let input = include_str!("../tests/fixtures/cargo_test_passing.txt");
        let out = filter(input);
        let orig = count_tokens(input);
        let filt = count_tokens(&out);
        let savings = 1.0 - filt as f64 / orig as f64;
        assert!(
            savings >= 0.60,
            "cargo test filter: expected ≥60% savings, got {:.1}%",
            savings * 100.0
        );
    }
}
