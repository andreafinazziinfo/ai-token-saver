use lazy_static::lazy_static;
use regex::Regex;

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

    lazy_static! {
        // Match standard error keywords in logs (case-insensitive)
        static ref ERROR_KEYWORD: Regex = Regex::new(
            r"(?i)\b(error|panic|failed|exception|fatal|critical|severe|warning)\b"
        ).unwrap();

        // Match common compiler/tool diagnostic markers
        static ref DIAGNOSTIC_LINE: Regex = Regex::new(
            r"^(error|warning|note|info|err|warn):\s+|^\[(ERROR|WARN|FATAL|SEVERE)\]"
        ).unwrap();
    }

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
    for i in 0..HEAD_LINES {
        if i < total_lines {
            out.push_str(raw_lines[i]);
            out.push('\n');
        }
    }

    // 2. Process middle lines
    let middle_start = HEAD_LINES;
    let middle_end = total_lines.saturating_sub(TAIL_LINES);

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
    let actual_tail_start = std::cmp::max(middle_end, HEAD_LINES);
    for i in actual_tail_start..total_lines {
        out.push_str(raw_lines[i]);
        out.push('\n');
    }

    out
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
        // Limit 20. Head 15, Tail 15. Total 50 lines.
        // Middle is from line 16 to line 35 (20 lines)
        let out = distill(&input, Some(20));
        assert!(out.contains("line 1\n"));
        assert!(out.contains("line 15\n"));
        assert!(out.contains("... [20 lines collapsed] ...\n"));
        assert!(out.contains("line 36\n"));
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
        // The 20 middle lines should be split into:
        // normal lines 16..24 (9 lines collapsed)
        // Error line 25 (kept)
        // normal lines 26..35 (10 lines collapsed)
        assert!(out.contains("... [9 lines collapsed] ...\n"));
        assert!(out.contains("... [10 lines collapsed] ...\n"));
    }
}
