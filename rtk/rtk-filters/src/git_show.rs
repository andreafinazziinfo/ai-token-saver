/// Filter `git show` output — keep subject + patch, drop Author/Date noise.
pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len() / 3);
    let mut in_diff = false;

    for line in input.lines() {
        if line.starts_with("diff --git")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
            || line.starts_with("index ")
        {
            in_diff = true;
        }

        if in_diff {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if line.starts_with("Author:")
            || line.starts_with("Date:")
            || line.starts_with("Merge:")
            || line.starts_with("Merge-tag:")
        {
            continue;
        }

        if let Some(stripped) = line.strip_prefix("commit ") {
            if let Some(hash) = stripped.split_whitespace().next() {
                if hash.len() >= 7 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    out.push_str(&hash[..7]);
                    out.push('\n');
                }
            }
            continue;
        }

        let trimmed = line.trim();
        if !trimmed.is_empty() {
            out.push_str(trimmed);
            out.push('\n');
        }
    }

    if out.is_empty() && !input.is_empty() {
        return input.to_string();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_tokens(s: &str) -> usize {
        s.len().div_ceil(4)
    }

    const SAMPLE: &str = concat!(
        "commit abc123def456abc123def456abc123def456abc1\n",
        "Author: Dev <dev@example.com>\n",
        "Date:   Mon Jan 1 00:00:00 2026 +0000\n",
        "\n",
        "    feat: add filter\n",
        "\n",
        "diff --git a/src/lib.rs b/src/lib.rs\n",
        "index 1111111..2222222 100644\n",
        "--- a/src/lib.rs\n",
        "+++ b/src/lib.rs\n",
        "@@ -1,3 +1,4 @@\n",
        "+fn new() {}\n",
    );

    #[test]
    fn keeps_diff_drops_author() {
        let out = filter(SAMPLE);
        assert!(out.contains("abc123d"));
        assert!(out.contains("feat: add filter"));
        assert!(out.contains("diff --git"));
        assert!(out.contains("+fn new()"));
        assert!(!out.contains("Author:"));
        assert!(!out.contains("Date:"));
    }

    #[test]
    fn token_savings() {
        let out = filter(SAMPLE);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(SAMPLE) as f64;
        assert!(
            savings >= 0.40,
            "git show filter: expected ≥40% savings, got {:.1}%",
            savings * 100.0
        );
    }
}
