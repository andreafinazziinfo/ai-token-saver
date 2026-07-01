use regex::Regex;
use std::sync::LazyLock;

/// Filter `gh pr checks` output.
///
/// Each check is a tab-separated row `name \t status \t elapsed \t URL`. The
/// per-job URL is long (~80 chars) and rarely needed inline — the agent wants
/// the check name, status and duration.
///
/// Strategy:
///   - Drop the trailing GitHub URL field from each row.
///   - Pass through unchanged if no URL-bearing rows are present (so other `gh`
///     output routed here is never mangled).
pub fn filter(input: &str) -> String {
    static URL_FIELD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\thttps?://\S+").unwrap());

    let mut changed = false;
    let mut out = String::with_capacity(input.len());
    for line in input.lines() {
        if URL_FIELD.is_match(line) {
            let cleaned = URL_FIELD.replace_all(line, "");
            out.push_str(cleaned.trim_end());
            out.push('\n');
            changed = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    if !changed {
        return input.to_string(); // no check URLs — passthrough
    }
    let trimmed = out.trim_end();
    if trimmed.is_empty() {
        return input.to_string();
    }
    format!("{trimmed}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drops_urls_keeps_status() {
        let input = include_str!("../tests/fixtures/gh_pr_checks.txt");
        let out = filter(input);
        assert!(out.contains("Analyze (actions)\tpass\t42s"));
        assert!(out.contains("Build and Test (windows-latest)\tpass\t2m16s"));
        assert!(!out.contains("https://"), "URL leaked");
        assert!(!out.contains("/job/"), "job URL leaked");
    }

    #[test]
    fn test_non_check_passthrough() {
        let input = "Some other gh output\nwithout urls\n";
        assert_eq!(filter(input), input);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(filter(""), "");
    }

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().map(str::len).sum::<usize>().max(1)
    }

    #[test]
    fn token_savings_checks() {
        let input = include_str!("../tests/fixtures/gh_pr_checks.txt");
        let out = filter(input);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(input) as f64;
        assert!(
            savings >= 0.40,
            "gh filter: expected ≥40% savings, got {:.1}%",
            savings * 100.0
        );
    }
}
