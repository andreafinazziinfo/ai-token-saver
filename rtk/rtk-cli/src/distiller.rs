use regex::Regex;
use std::sync::LazyLock;

const DEFAULT_LINE_LIMIT: usize = 80;
const HEAD_LINES: usize = 15;
const TAIL_LINES: usize = 15;

/// Generic log distiller.
/// Compresses large output blocks by keeping first N and last M lines,
/// while scanning the middle section to preserve any lines containing error indicators.
#[allow(clippy::needless_range_loop)]
pub fn distill(input: &str, max_lines: Option<usize>) -> String {
    let limit = max_lines.unwrap_or(DEFAULT_LINE_LIMIT);
    let raw_lines: Vec<&str> = input.lines().collect();
    let total_lines = raw_lines.len();

    if total_lines <= limit {
        return input.to_string();
    }

    let head_lines = std::cmp::min(HEAD_LINES, limit / 2);
    let tail_lines = std::cmp::min(TAIL_LINES, limit.saturating_sub(head_lines));

    // Match standard error keywords in logs (case-insensitive)
    static ERROR_KEYWORD: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(error|panic|failed|exception|fatal|critical|severe|warning)\b").unwrap()
    });

    // Match common compiler/tool diagnostic markers
    static DIAGNOSTIC_LINE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(error|warning|note|info|err|warn):\s+|^\[(ERROR|WARN|FATAL|SEVERE)\]")
            .unwrap()
    });

    let mut out = String::with_capacity(input.len() / 4);
    let mut collapsed_count = 0;

    // Flush any collapsed lines buffer
    let flush_collapsed = |count: &mut usize, buf: &mut String| {
        if *count > 0 {
            buf.push_str(&format!("... [{count} lines collapsed] ...\n"));
            *count = 0;
        }
    };

    // 1. Output the header/context lines
    for i in 0..head_lines {
        if i < total_lines {
            out.push_str(raw_lines[i]);
            out.push('\n');
        }
    }

    // 2. Process middle lines
    let middle_start = head_lines;
    let middle_end = total_lines.saturating_sub(tail_lines);

    if middle_start < middle_end {
        for i in middle_start..middle_end {
            let line = raw_lines[i];
            let trimmed = line.trim();

            let is_error = ERROR_KEYWORD.is_match(trimmed) || DIAGNOSTIC_LINE.is_match(trimmed);

            if is_error {
                flush_collapsed(&mut collapsed_count, &mut out);
                out.push_str(line);
                out.push('\n');
            } else {
                collapsed_count += 1;
            }
        }
    }
    flush_collapsed(&mut collapsed_count, &mut out);

    // 3. Output the tail/result lines
    let actual_tail_start = std::cmp::max(middle_end, head_lines);
    for i in actual_tail_start..total_lines {
        out.push_str(raw_lines[i]);
        out.push('\n');
    }

    out
}

/// Analyze git diff, estimate token counts and API costs across top models,
/// and print a structured token savings table.
pub fn run_estimate() -> anyhow::Result<()> {
    let output = std::process::Command::new("git")
        .args(["diff", "--no-color"])
        .output()
        .map_err(|e| anyhow::anyhow!("failed to execute git diff: {e}"))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git diff failed: {err_msg}"));
    }

    let diff_content = String::from_utf8_lossy(&output.stdout);
    if diff_content.trim().is_empty() {
        println!("==================================================");
        println!("           RTK PR TOKEN & COST ESTIMATOR          ");
        println!("==================================================");
        println!("No changes detected. Git diff is empty.");
        println!("==================================================");
        return Ok(());
    }

    let mut files_count = 0;
    for line in diff_content.lines() {
        if line.starts_with("diff --git ") {
            files_count += 1;
        }
    }

    let raw_chars = diff_content.len();
    let original_tokens = rtk_db::tracking::count_tokens(&diff_content);
    let filtered_content = rtk_filters::git_diff::filter(&diff_content);
    let filtered_tokens = rtk_db::tracking::count_tokens(&filtered_content);

    let saved_tokens = original_tokens - filtered_tokens;
    let savings_pct = if original_tokens > 0 {
        (saved_tokens as f64 / original_tokens as f64) * 100.0
    } else {
        0.0
    };

    println!("==================================================");
    println!("           RTK PR TOKEN & COST ESTIMATOR          ");
    println!("==================================================");
    println!("Active Git Diff:");
    println!("  Files Changed:        {}", files_count);
    println!("  Raw Characters:       {}", raw_chars);
    println!("  Estimated Raw Tokens: {}", original_tokens);
    println!(
        "  Filtered Tokens:      {} ({:.1}% saved)",
        filtered_tokens, savings_pct
    );
    println!();
    println!("Cost & Savings Projection (Input Tokens):");
    println!("------------------------------------------------------------------------------------------------");
    println!(
        "{:<24} | {:>11} | {:>11} | {:>18} | {:>9} | {:>9} | {:>10}",
        "Model Name",
        "Orig Tokens",
        "Filt Tokens",
        "Saved Tokens",
        "Orig Cost",
        "Filt Cost",
        "Saved Cost"
    );
    println!("------------------------------------------------------------------------------------------------");

    let models = vec![
        "claude-4.8-opus",
        "claude-4.6-sonnet",
        "gpt-5.5",
        "gpt-5.4",
        "gemini-3.5-flash",
    ];

    for model_id in models {
        let price = rtk_db::pricing::get_merged_price(model_id);
        let display_name = price
            .as_ref()
            .map(|p| p.display_name.as_str())
            .unwrap_or(model_id);

        let orig_cost = rtk_db::pricing::calculate_cost(original_tokens, model_id, false);
        let filt_cost = rtk_db::pricing::calculate_cost(filtered_tokens, model_id, false);
        let saved_cost = orig_cost - filt_cost;

        println!(
            "{:<24} | {:>11} | {:>11} | {:>11} ({:>.1}%) | ${:>8.4} | ${:>8.4} | ${:>9.4}",
            display_name,
            original_tokens,
            filtered_tokens,
            saved_tokens,
            savings_pct,
            orig_cost,
            filt_cost,
            saved_cost
        );
    }
    println!("------------------------------------------------------------------------------------------------");
    println!(
        "✅ Running this git diff through RTK filters saves you {:.1}% in tokens & API costs!",
        savings_pct
    );
    println!("==================================================");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distill_under_limit() {
        let input = "line1\nline2\nline3\n";
        assert_eq!(distill(input, Some(10)), input);
    }

    #[test]
    fn test_distill_collapses_middle() {
        let mut input = String::new();
        for i in 1..=50 {
            input.push_str(&format!("line {i}\n"));
        }
        // Limit 20. Head 10, Tail 10. Total 50 lines.
        // Middle is from line 11 to line 40 (30 lines)
        let out = distill(&input, Some(20));
        assert!(out.contains("line 1\n"));
        assert!(out.contains("line 10\n"));
        assert!(out.contains("... [30 lines collapsed] ...\n"));
        assert!(out.contains("line 41\n"));
        assert!(out.contains("line 50\n"));
        assert!(!out.contains("line 25\n"));
    }

    #[test]
    fn test_distill_preserves_errors_in_middle() {
        let mut input = String::new();
        for i in 1..=50 {
            if i == 25 {
                input.push_str("Error: something failed in execution\n");
            } else {
                input.push_str(&format!("normal log line {i}\n"));
            }
        }
        let out = distill(&input, Some(20));
        assert!(out.contains("Error: something failed in execution\n"));
        // The 30 middle lines (11..40) should be split into:
        // normal lines 11..24 (14 lines collapsed)
        // Error line 25 (kept)
        // normal lines 26..40 (15 lines collapsed)
        assert!(out.contains("... [14 lines collapsed] ...\n"));
        assert!(out.contains("... [15 lines collapsed] ...\n"));
    }
}
