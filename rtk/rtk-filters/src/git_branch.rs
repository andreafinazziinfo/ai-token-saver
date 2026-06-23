/// Filter `git branch -v` output — one compact line per branch.
pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len() / 2);

    for line in input.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 {
            let (marker, branch, hash_idx) = if parts[0] == "*" {
                ("*", parts[1], 2usize)
            } else {
                (" ", parts[0], 1usize)
            };
            let hash = parts[hash_idx];
            let hash_short = &hash[..hash.len().min(7)];
            let msg = parts[(hash_idx + 1)..].join(" ");
            out.push_str(marker);
            out.push(' ');
            out.push_str(branch);
            out.push(' ');
            out.push_str(hash_short);
            out.push(' ');
            out.push_str(&msg);
            out.push('\n');
        } else {
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

    const SAMPLE: &str = concat!(
        "  main                  abc123def4567890abcdef1234567890abcd  feat: first\n",
        "* develop               def456abc7890123def456abc7890123def4  fix: second\n",
    );

    #[test]
    fn compacts_branch_lines() {
        let out = filter(SAMPLE);
        assert!(out.contains("* develop def456a fix: second"));
        assert!(out.contains("  main abc123d feat: first"));
    }

    #[test]
    fn token_savings() {
        let out = filter(SAMPLE);
        let orig = SAMPLE.len().div_ceil(4);
        let filt = out.len().div_ceil(4);
        let savings = 1.0 - filt as f64 / orig as f64;
        assert!(
            savings >= 0.40,
            "git branch -v filter: expected ≥40% savings, got {:.1}%",
            savings * 100.0
        );
    }
}
