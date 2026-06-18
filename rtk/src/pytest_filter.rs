use lazy_static::lazy_static;
use regex::Regex;

/// Filter `pytest` output.
///
/// Strategy:
///   - Drop preamble details (platform, plugins, rootdir, configfile).
///   - Strip warnings block (`=== WARNINGS ===` to next header) and count warnings.
///   - Retain test execution progress (e.g. `tests/test_foo.py .F [100%]`).
///   - Retain failures block (`=== FAILURES ===`).
///   - Retain short test summary and final outcome line.
///   - Fallback: return input unchanged if filter produces empty output.
pub fn filter(input: &str) -> String {
    lazy_static! {
        static ref HEADER_LINE: Regex = Regex::new(r"^={3,}\s+(.+)\s+={3,}$").unwrap();
        static ref PREAMBLE_LINE: Regex =
            Regex::new(r"^(platform|plugins|rootdir|configfile|collected|plugins)[:\s]").unwrap();
    }

    let mut out = String::with_capacity(input.len() / 3);
    let mut in_warnings = false;
    let mut warning_count = 0;
    let mut in_preamble = true;

    for line in input.lines() {
        let trimmed = line.trim();

        // Check for section headers
        if let Some(caps) = HEADER_LINE.captures(trimmed) {
            let section_title = caps[1].trim();
            if !section_title.contains("test session starts") {
                in_preamble = false;
            }

            if section_title == "WARNINGS" || section_title.starts_with("warnings summary") {
                in_warnings = true;
                continue;
            } else {
                if in_warnings {
                    // Left warnings section, output a summary
                    if warning_count > 0 {
                        out.push_str(&format!("=== {warning_count} warnings collapsed (run with -W ignore to suppress) ===\n\n"));
                    }
                    in_warnings = false;
                }
            }
        }

        // If in warnings section, count warning blocks and skip printing
        if in_warnings {
            // Warnings usually start with a file location or warning description line
            if trimmed.contains("Warning") || trimmed.contains("DeprecationWarning") {
                warning_count += 1;
            }
            continue;
        }

        // Drop preamble lines at start of test run
        if in_preamble && (PREAMBLE_LINE.is_match(trimmed) || trimmed.is_empty()) {
            continue;
        }

        // We passed the preamble
        if in_preamble && !trimmed.is_empty() && !HEADER_LINE.is_match(trimmed) {
            in_preamble = false;
        }

        out.push_str(line);
        out.push('\n');
    }

    // If warnings was the last section and we didn't close it:
    if in_warnings && warning_count > 0 {
        out.push_str(&format!(
            "=== {warning_count} warnings collapsed (run with -W ignore to suppress) ===\n"
        ));
    }

    let trimmed = out.trim_end();
    if trimmed.is_empty() && !input.trim().is_empty() {
        return input.to_string(); // fallback
    }
    format!("{trimmed}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pytest_clean() {
        let input = concat!(
            "============================= test session starts =============================\n",
            "platform linux -- Python 3.10.12, pytest-7.4.3, pluggy-1.3.0\n",
            "rootdir: /path/to/project\n",
            "collected 3 items\n",
            "\n",
            "tests/test_foo.py ...                                                    [100%]\n",
            "============================== 3 passed in 0.12s ==============================\n",
        );
        let out = filter(input);
        assert!(!out.contains("platform linux"), "preamble leaked");
        assert!(!out.contains("rootdir:"), "rootdir leaked");
        assert!(
            out.contains("tests/test_foo.py ..."),
            "test progress missing"
        );
        assert!(out.contains("3 passed in 0.12s"), "test summary missing");
    }

    #[test]
    fn test_pytest_warnings_collapsed() {
        let input = concat!(
            "============================= test session starts =============================\n",
            "collected 1 item\n",
            "\n",
            "tests/test_foo.py .                                                      [100%]\n",
            "=================================== WARNINGS ===================================\n",
            "tests/test_foo.py::test_bar\n",
            "  /path/to/lib.py:12: DeprecationWarning: some warning text\n",
            "============================== 1 passed, 1 warning in 0.12s ====================\n",
        );
        let out = filter(input);
        assert!(!out.contains("DeprecationWarning:"), "warning block leaked");
        assert!(
            out.contains("1 warnings collapsed"),
            "warnings collapse summary missing"
        );
        assert!(out.contains("1 passed, 1 warning"), "outcome line missing");
    }

    #[test]
    fn test_pytest_failures_kept() {
        let input = concat!(
            "collected 2 items\n",
            "tests/test_foo.py .F                                                     [100%]\n",
            "=================================== FAILURES ===================================\n",
            "__________________________________ test_fail ___________________________________\n",
            "def test_fail():\n",
            ">       assert False\n",
            "E       assert False\n",
            "tests/test_foo.py:10: AssertionError\n",
            "=========================== short test summary info ============================\n",
            "FAILED tests/test_foo.py::test_fail - assert False\n",
            "========================= 1 failed, 1 passed in 0.15s ==========================\n",
        );
        let out = filter(input);
        assert!(out.contains("=== FAILURES ==="), "failures header missing");
        assert!(out.contains("AssertionError"), "assertion details missing");
        assert!(
            out.contains("FAILED tests/test_foo.py::test_fail"),
            "summary info missing"
        );
    }
}
