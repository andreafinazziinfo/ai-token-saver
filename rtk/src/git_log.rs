/// Filter `git log` output (default format).
///
/// Strategy:
///   - Reduce each commit to one line: `<7-char hash>  <subject>`
///   - Drop: Author, Date, Merge, commit body, blank lines
///   - Already-compact formats (--oneline, --format=…): produce no matching
///     headers → filter output is empty → fallback returns input unchanged
///   - Fallback: return input unchanged if filter produces empty output
pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len() / 5);
    let mut pending_hash: Option<&str> = None;

    for line in input.lines() {
        // "commit <hash>" — may optionally have refs after: "commit abc… (HEAD -> main)"
        if let Some(stripped) = line.strip_prefix("commit ") {
            let parts: Vec<&str> = stripped.split_whitespace().collect();
            if let Some(full_hash) = parts.first() {
                if full_hash.len() >= 7 && full_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    let end_idx = std::cmp::min(7, full_hash.len());
                    pending_hash = Some(&full_hash[..end_idx]);
                    continue;
                }
            }
        }

        // Header fields — skip
        if line.starts_with("Author:") || line.starts_with("Date:") || line.starts_with("Merge:") {
            continue;
        }

        if let Some(hash) = pending_hash {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                // blank line separating header from subject — keep waiting
                continue;
            }
            // First non-blank, non-header line after commit = subject
            out.push_str(hash);
            out.push_str("  ");
            out.push_str(trimmed);
            out.push('\n');
            pending_hash = None;
            // Remaining body lines fall through with pending_hash = None → ignored
        }
        // Lines after subject (body, blank separators): pending_hash is None → ignored
    }

    if out.is_empty() && !input.is_empty() {
        return input.to_string(); // fallback: --oneline / --format / empty repo
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_tokens(s: &str) -> usize {
        (s.len() as f64 / 4.0).ceil() as usize
    }

    const SAMPLE_LOG: &str = concat!(
        "commit abc123def456abc123def456abc123def456abc1\n",
        "Author: Dev <dev@example.com>\n",
        "Date:   Mon Jan 1 00:00:00 2026 +0000\n",
        "\n",
        "    feat: add tracking module\n",
        "\n",
        "    Longer body paragraph that should be dropped.\n",
        "\n",
        "commit 999888777666555444333222111000fffeeeddcb\n",
        "Author: Dev <dev@example.com>\n",
        "Date:   Sun Dec 31 23:00:00 2025 +0000\n",
        "\n",
        "    fix: correct DB path\n",
        "\n",
    );

    #[test]
    fn one_line_per_commit() {
        let out = filter(SAMPLE_LOG);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines.len(),
            2,
            "expected 2 lines for 2 commits, got: {:?}",
            lines
        );
    }

    #[test]
    fn short_hash_and_subject() {
        let out = filter(SAMPLE_LOG);
        assert!(out.contains("abc123d"), "short hash missing");
        assert!(out.contains("feat: add tracking module"), "subject missing");
        assert!(out.contains("9998887"), "second hash missing");
        assert!(
            out.contains("fix: correct DB path"),
            "second subject missing"
        );
    }

    #[test]
    fn no_author_or_date() {
        let out = filter(SAMPLE_LOG);
        assert!(!out.contains("Author:"), "Author leaked");
        assert!(!out.contains("Date:"), "Date leaked");
        assert!(!out.contains("Longer body"), "body leaked");
    }

    #[test]
    fn fallback_on_oneline_format() {
        // --oneline output has no "commit <40-hash>" lines → filter is empty → fallback
        let oneline = "abc1234 feat: add tracking module\n9998887 fix: correct path\n";
        let out = filter(oneline);
        assert_eq!(out, oneline, "--oneline should passthrough via fallback");
    }

    #[test]
    fn token_savings_real_log() {
        let input = include_str!("../tests/fixtures/git_log_raw.txt");
        let out = filter(input);
        let orig = count_tokens(input);
        let filt = count_tokens(&out);
        let savings = 1.0 - filt as f64 / orig as f64;
        assert!(
            savings >= 0.60,
            "git log filter: expected ≥60% savings, got {:.1}%",
            savings * 100.0
        );
    }
}
