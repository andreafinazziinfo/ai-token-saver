//! Filter `git status` output.
//!
//! Strategy:
//! - Keep: branch line, upstream tracking line, file entries (labelled by section).
//! - Remove: section headers, hint lines, blank lines, and "nothing added" noise.
//! - Compact: "nothing to commit, working tree clean" -> "clean".
//! - File entries: prefixed with their section (`staged:`, `unstaged:`, `untracked:`, `conflict:`) so context is preserved.
//! - Fallback: return input unchanged if filter produces empty output.

/// Maximum untracked file entries to enumerate before collapsing to a summary.
const MAX_UNTRACKED: usize = 10;

/// Filter the `git status` output to eliminate hint lines and group/limit file statuses.
pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len() / 3);
    // "staged" | "unstaged" | "untracked" | "conflict" | ""
    let mut section: &str = "";
    let mut untracked_shown: usize = 0;
    let mut untracked_total: usize = 0;

    for line in input.lines() {
        let trimmed = line.trim();

        // ── Drop ────────────────────────────────────────────────────────────
        if trimmed.is_empty()
            || trimmed.starts_with("(use ")
            || trimmed.starts_with("nothing added")
            || trimmed.starts_with("no changes added")
        {
            continue;
        }

        // ── Section transitions (no output) ─────────────────────────────────
        if trimmed.starts_with("Changes to be committed") {
            section = "staged";
            continue;
        }
        if trimmed.starts_with("Changes not staged for commit") {
            section = "unstaged";
            continue;
        }
        if trimmed.starts_with("Untracked files") {
            section = "untracked";
            continue;
        }
        if trimmed.starts_with("Unmerged paths") {
            section = "conflict";
            continue;
        }

        // ── "Clean" shorthand ────────────────────────────────────────────────
        if trimmed == "nothing to commit, working tree clean" {
            out.push_str("clean\n");
            continue;
        }

        // ── Tab-indented file entries ────────────────────────────────────────
        if line.starts_with('\t') && !section.is_empty() {
            if section == "untracked" {
                untracked_total += 1;
                if untracked_shown < MAX_UNTRACKED {
                    out.push_str("untracked: ");
                    out.push_str(trimmed);
                    out.push('\n');
                    untracked_shown += 1;
                }
                // excess untracked entries are counted but not printed;
                // the summary is flushed at the end
            } else {
                out.push_str(section);
                out.push_str(": ");
                out.push_str(trimmed); // keeps "modified: foo.rs" or just "foo.rs"
                out.push('\n');
            }
            continue;
        }

        // ── Everything else: keep (branch, HEAD detached, upstream status,
        //    rebase-in-progress notices, interactive-rebase state, etc.) ──────
        out.push_str(trimmed);
        out.push('\n');
    }

    // Flush untracked overflow summary
    let overflow = untracked_total.saturating_sub(untracked_shown);
    if overflow > 0 {
        out.push_str(&format!("... and {overflow} more untracked\n"));
    }

    if out.trim().is_empty() && !input.trim().is_empty() {
        return input.to_string(); // fallback: never blank the user
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_tokens(s: &str) -> usize {
        (s.len() as f64 / 4.0).ceil() as usize
    }

    // ── Core behaviour ───────────────────────────────────────────────────────

    #[test]
    fn clean_repo() {
        let input = "On branch main\nnothing to commit, working tree clean\n";
        let out = filter(input);
        assert!(out.contains("On branch main"), "branch missing");
        assert!(out.contains("clean"), "'clean' missing");
        assert!(!out.contains("working tree"), "verbose phrase leaked");
    }

    #[test]
    fn no_hint_lines() {
        let input = "On branch main\nChanges not staged for commit:\n  (use \"git add <file>...\" to update what will be committed)\n\tmodified:   src/foo.rs\n";
        let out = filter(input);
        assert!(!out.contains("(use"), "hint line leaked");
        assert!(
            out.contains("unstaged: modified:   src/foo.rs"),
            "file entry missing"
        );
    }

    #[test]
    fn sections_labelled() {
        let input = concat!(
            "On branch main\n",
            "Changes to be committed:\n",
            "  (use \"git restore --staged...\" to unstage)\n",
            "\tnew file:   src/new.rs\n",
            "Changes not staged for commit:\n",
            "  (use \"git add...\")\n",
            "\tmodified:   Cargo.toml\n",
            "Untracked files:\n",
            "  (use \"git add...\")\n",
            "\tsrc/scratch.rs\n",
        );
        let out = filter(input);
        assert!(
            out.contains("staged: new file:   src/new.rs"),
            "staged label missing"
        );
        assert!(
            out.contains("unstaged: modified:   Cargo.toml"),
            "unstaged label missing"
        );
        assert!(
            out.contains("untracked: src/scratch.rs"),
            "untracked label missing"
        );
    }

    #[test]
    fn conflict_labelled() {
        let input = "On branch main\nUnmerged paths:\n  (use \"git add...\")\n\tboth modified:   src/lib.rs\n";
        let out = filter(input);
        assert!(
            out.contains("conflict: both modified:   src/lib.rs"),
            "conflict label missing"
        );
    }

    #[test]
    fn token_savings_dirty() {
        let input = include_str!("../tests/fixtures/git_status_dirty.txt");
        let out = filter(input);
        let orig = count_tokens(input);
        let filt = count_tokens(&out);
        let savings = 1.0 - filt as f64 / orig as f64;
        assert!(
            savings >= 0.50,
            "git status filter: expected ≥50% savings, got {:.1}%",
            savings * 100.0
        );
    }

    #[test]
    fn untracked_cap() {
        // Build a status with 15 untracked files
        let mut input = "On branch main\nUntracked files:\n  (use \"git add...\")\n".to_string();
        for i in 0..15 {
            input.push_str(&format!("\tfile{i}.rs\n"));
        }
        let out = filter(&input);
        let untracked_lines: Vec<&str> = out
            .lines()
            .filter(|l| l.starts_with("untracked: "))
            .collect();
        assert_eq!(
            untracked_lines.len(),
            MAX_UNTRACKED,
            "should show exactly MAX_UNTRACKED entries"
        );
        assert!(
            out.contains("... and 5 more untracked"),
            "overflow summary missing"
        );
    }

    #[test]
    fn fallback_on_empty_output() {
        let weird = "some unrecognised git output\nwith no known patterns\n";
        let out = filter(weird);
        // Should not return empty — fallback kicks in
        assert!(!out.is_empty());
    }
}
