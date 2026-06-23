/// Filter `cargo build` and `cargo check` stderr output.
///
/// IMPORTANT: Unlike `cargo test`, all build/check output is on **stderr**.
/// This filter is applied via `run_filtered_stderr`, not `run_filtered`.
///
/// Strategy:
/// - Drop: "   Compiling …", "    Checking …" (crate progress lines), download/lock progress lines.
/// - Keep: "    Finished …" (the primary result line), ALL warning/error blocks, and compile failure summaries.
/// - Fallback: return input unchanged if filter produces empty output.
///
/// Safety: warning and error lines never share the prefix patterns we drop,
/// so no diagnostics can be silently lost.
pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len() / 4);

    for line in input.lines() {
        // Drop crate-level progress lines only.
        // "    Finished " is intentionally NOT listed here — it is the
        // primary result for build/check (unlike cargo_test where it is noise).
        if line.starts_with("   Compiling ")
            || line.starts_with("    Checking ")
            || line.starts_with("   Locking ")
            || line.starts_with("      Adding ")
            || line.starts_with("  Downloading ")
            || line.starts_with("  Downloaded ")
            || line.starts_with("   Downloaded ")
            || line.starts_with(" Downloading ")
        {
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

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
    fn drops_compiling_and_checking_lines() {
        let input = concat!(
            "   Compiling anyhow v1.0.86\n",
            "    Checking rtk v0.1.0 (/path)\n",
            "  Downloading serde v1.0\n",
            "    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.06s\n",
        );
        let out = filter(input);
        assert!(!out.contains("Compiling"), "Compiling leaked");
        assert!(!out.contains("Checking"), "Checking leaked");
        assert!(!out.contains("Downloading"), "Downloading leaked");
        assert!(out.contains("Finished"), "Finished summary dropped");
    }

    #[test]
    fn test_snapshot_cargo_build_clean() {
        use insta::assert_snapshot;
        let input = include_str!("../tests/fixtures/cargo_build_clean.txt");
        assert_snapshot!(filter(input));
    }

    #[test]
    fn keeps_warnings() {
        let input = concat!(
            "   Compiling rtk v0.1.0 (/path)\n",
            "warning: unused variable `x`\n",
            "  --> src/main.rs:10:9\n",
            "   |\n",
            "10 |     let x = 5;\n",
            "   |         ^ help: prefix with `_`\n",
            "   |\n",
            "   = note: `#[warn(unused_variables)]` on by default\n",
            "\n",
            "warning: `rtk` (bin \"rtk\") generated 1 warning\n",
            "    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.50s\n",
        );
        let out = filter(input);
        assert!(!out.contains("Compiling"), "Compiling leaked");
        assert!(out.contains("warning: unused variable"), "warning dropped");
        assert!(out.contains("--> src/main.rs"), "source location dropped");
        assert!(
            out.contains("generated 1 warning"),
            "warning summary dropped"
        );
        assert!(out.contains("Finished"), "Finished dropped");
    }

    #[test]
    fn keeps_errors() {
        let input = concat!(
            "   Compiling rtk v0.1.0 (/path)\n",
            "error[E0308]: mismatched types\n",
            "  --> src/main.rs:5:5\n",
            "   |\n",
            "5  |     42\n",
            "   |     ^^ expected `()`, found integer\n",
            "\n",
            "error: could not compile `rtk` (bin \"rtk\") due to 1 previous error\n",
        );
        let out = filter(input);
        assert!(!out.contains("Compiling"), "Compiling leaked");
        assert!(out.contains("error[E0308]"), "error block dropped");
        assert!(
            out.contains("could not compile"),
            "compile error summary dropped"
        );
    }

    #[test]
    fn token_savings_clean_build() {
        let input = include_str!("../tests/fixtures/cargo_build_clean.txt");
        let out = filter(input);
        let orig = count_tokens(input);
        let filt = count_tokens(&out);
        let savings = 1.0 - filt as f64 / orig as f64;
        assert!(
            savings >= 0.60,
            "cargo build filter: expected ≥60% savings, got {:.1}%",
            savings * 100.0
        );
    }

    #[test]
    fn check_with_warnings_correctness() {
        // Savings are intentionally not tested here: they scale with the number of
        // crates compiled (can be 50-300 on real projects) relative to warning volume.
        // What matters is that preamble is gone and diagnostics are intact.
        let input = include_str!("../tests/fixtures/cargo_check_warnings.txt");
        let out = filter(input);
        assert!(!out.contains("   Compiling "), "Compiling line leaked");
        assert!(!out.contains("    Checking "), "Checking line leaked");
        assert!(
            out.contains("warning: unused variable"),
            "warning block dropped"
        );
        assert!(
            out.contains("warning: variable does not need to be mutable"),
            "second warning dropped"
        );
        assert!(
            out.contains("generated 2 warnings"),
            "warning summary dropped"
        );
        assert!(out.contains("Finished"), "Finished line dropped");
    }

    #[test]
    fn fallback_on_unrecognised_output() {
        let weird = "some unrecognised cargo output line\n";
        let out = filter(weird);
        assert_eq!(out, weird);
    }
}
