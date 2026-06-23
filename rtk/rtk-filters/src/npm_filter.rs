use regex::Regex;
use std::sync::LazyLock;

const MAX_ADDED_SHOWN: usize = 5;
const MAX_WARN_SHOWN: usize = 3;

/// Filter verbose `npm` / `pnpm` install output.
pub fn filter(input: &str) -> String {
    static ADDED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^added .+").unwrap());
    static NPM_WARN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^npm warn ").unwrap());
    static PNPM_PROGRESS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^Progress: resolved \d+").unwrap());

    filter_pkg_manager(input, &ADDED, &NPM_WARN, &PNPM_PROGRESS)
}

/// Filter verbose `yarn` install output.
pub fn filter_yarn(input: &str) -> String {
    static YARN_PKG: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(info|warning) .+@(file:|npm:|https?:)").unwrap());
    static YARN_WARN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^warning .+").unwrap());
    static YARN_STEP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\[\d+/\d+\]").unwrap());

    let mut out = String::with_capacity(input.len() / 3);
    let mut step_hidden = 0;
    let mut pkg_hidden = 0;
    let mut warn_hidden = 0;
    let mut pkg_shown = 0;
    let mut warn_shown = 0;

    let flush = |out: &mut String,
                 step_hidden: &mut usize,
                 pkg_hidden: &mut usize,
                 warn_hidden: &mut usize| {
        if *step_hidden > 0 {
            out.push_str(&format!("... {step_hidden} yarn steps collapsed ...\n"));
            *step_hidden = 0;
        }
        if *pkg_hidden > 0 {
            out.push_str(&format!("... {pkg_hidden} package lines collapsed ...\n"));
            *pkg_hidden = 0;
        }
        if *warn_hidden > 0 {
            out.push_str(&format!("... {warn_hidden} warnings collapsed ...\n"));
            *warn_hidden = 0;
        }
    };

    for line in input.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with("error ") || t.contains("ERR!") {
            flush(
                &mut out,
                &mut step_hidden,
                &mut pkg_hidden,
                &mut warn_hidden,
            );
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if YARN_STEP.is_match(t) {
            step_hidden += 1;
            continue;
        }
        if YARN_WARN.is_match(t) {
            if warn_shown < MAX_WARN_SHOWN {
                flush(
                    &mut out,
                    &mut step_hidden,
                    &mut pkg_hidden,
                    &mut warn_hidden,
                );
                out.push_str(line);
                out.push('\n');
                warn_shown += 1;
            } else {
                warn_hidden += 1;
            }
            continue;
        }
        if YARN_PKG.is_match(t) {
            if pkg_shown < MAX_ADDED_SHOWN {
                flush(
                    &mut out,
                    &mut step_hidden,
                    &mut pkg_hidden,
                    &mut warn_hidden,
                );
                out.push_str(line);
                out.push('\n');
                pkg_shown += 1;
            } else {
                pkg_hidden += 1;
            }
            continue;
        }
        flush(
            &mut out,
            &mut step_hidden,
            &mut pkg_hidden,
            &mut warn_hidden,
        );
        out.push_str(line);
        out.push('\n');
    }
    flush(
        &mut out,
        &mut step_hidden,
        &mut pkg_hidden,
        &mut warn_hidden,
    );
    finalize(out, input)
}

fn filter_pkg_manager(input: &str, added: &Regex, warn: &Regex, extra_noise: &Regex) -> String {
    let mut out = String::with_capacity(input.len() / 3);
    let mut added_hidden = 0;
    let mut warn_hidden = 0;
    let mut added_shown = 0;
    let mut warn_shown = 0;

    let flush = |out: &mut String, added_hidden: &mut usize, warn_hidden: &mut usize| {
        if *added_hidden > 0 {
            out.push_str(&format!(
                "... {} packages added (collapsed) ...\n",
                *added_hidden
            ));
            *added_hidden = 0;
        }
        if *warn_hidden > 0 {
            out.push_str(&format!(
                "... {} deprecation warnings collapsed ...\n",
                *warn_hidden
            ));
            *warn_hidden = 0;
        }
    };

    for line in input.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with("npm ERR!") || extra_noise.is_match(t) {
            if extra_noise.is_match(t) {
                continue;
            }
            flush(&mut out, &mut added_hidden, &mut warn_hidden);
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if warn.is_match(t) {
            if warn_shown < MAX_WARN_SHOWN {
                flush(&mut out, &mut added_hidden, &mut warn_hidden);
                out.push_str(line);
                out.push('\n');
                warn_shown += 1;
            } else {
                warn_hidden += 1;
            }
            continue;
        }
        if added.is_match(t) {
            if added_shown < MAX_ADDED_SHOWN {
                flush(&mut out, &mut added_hidden, &mut warn_hidden);
                out.push_str(line);
                out.push('\n');
                added_shown += 1;
            } else {
                added_hidden += 1;
            }
            continue;
        }
        flush(&mut out, &mut added_hidden, &mut warn_hidden);
        out.push_str(line);
        out.push('\n');
    }
    flush(&mut out, &mut added_hidden, &mut warn_hidden);
    finalize(out, input)
}

fn finalize(out: String, input: &str) -> String {
    let trimmed = out.trim_end();
    if trimmed.is_empty() && !input.trim().is_empty() {
        input.to_string()
    } else {
        format!("{trimmed}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().map(str::len).sum::<usize>().max(1)
    }

    #[test]
    fn collapses_added_and_warn_spam() {
        let input = include_str!("../tests/fixtures/npm_install_verbose.txt");
        let out = filter(input);
        assert!(out.contains("added express@"));
        assert!(out.contains("packages added (collapsed)"));
        assert!(out.contains("deprecation warnings collapsed"));
        assert!(out.contains("moderate severity vulnerabilities"));
        assert!(!out.contains("added webpack@"));
    }

    #[test]
    fn token_savings_npm_install() {
        let input = include_str!("../tests/fixtures/npm_install_verbose.txt");
        let out = filter(input);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(input) as f64;
        assert!(
            savings >= 0.40,
            "npm filter: expected ≥40% savings, got {:.1}%",
            savings * 100.0
        );
    }

    #[test]
    fn yarn_collapses_steps() {
        let input = concat!(
            "[1/4] Resolving packages...\n",
            "[2/4] Fetching packages...\n",
            "[3/4] Linking dependencies...\n",
            "[4/4] Building fresh packages...\n",
            "warning package-lock.json found\n",
            "Done in 4.12s.\n",
        );
        let out = filter_yarn(input);
        assert!(out.contains("yarn steps collapsed"));
        assert!(out.contains("Done in 4.12s."));
    }
}
