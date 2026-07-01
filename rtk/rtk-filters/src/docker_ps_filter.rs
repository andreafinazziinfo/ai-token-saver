use regex::Regex;
use std::sync::LazyLock;

/// Filter `docker ps` output.
///
/// The default table is very wide (often 200+ columns): CONTAINER ID, IMAGE,
/// COMMAND, CREATED, STATUS, PORTS, NAMES. For an agent the essential columns
/// are NAMES, IMAGE, STATUS and the published PORTS; the container id (a
/// truncated hash), the truncated COMMAND, and CREATED are noise.
///
/// Strategy:
///   - Keep `NAMES  IMAGE  STATUS  PORTS`, drop CONTAINER ID / COMMAND / CREATED.
///   - Pass through unchanged if the output is not the standard table (e.g. a
///     custom `--format`).
pub fn filter(input: &str) -> String {
    static COLS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}").unwrap());

    let mut lines = input.lines();
    match lines.next() {
        Some(h) if h.starts_with("CONTAINER ID") => {}
        _ => return input.to_string(), // not the standard table — passthrough
    }

    let mut out = String::from("NAMES  IMAGE  STATUS  PORTS\n");
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = COLS.split(line.trim_end()).collect();
        // 7 cols = ports present; 6 cols = ports column empty.
        let (image, status, ports, names) = match f.len() {
            7 => (f[1], f[4], f[5], f[6]),
            6 => (f[1], f[4], "", f[5]),
            _ => {
                out.push_str(line);
                out.push('\n');
                continue;
            }
        };
        if ports.is_empty() {
            out.push_str(&format!("{names}  {image}  {status}\n"));
        } else {
            out.push_str(&format!("{names}  {image}  {status}  {ports}\n"));
        }
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
    fn test_compacts_table() {
        let input = include_str!("../tests/fixtures/docker_ps.txt");
        let out = filter(input);
        assert_eq!(out.lines().next().unwrap(), "NAMES  IMAGE  STATUS  PORTS");
        assert!(out.contains("backend  myapp-backend  Up 6 hours  127.0.0.1:18110->8000/tcp"));
        // ports-less container keeps 3 columns.
        assert!(out.contains("cache  redis:7-alpine  Exited (0) 2 hours ago"));
        assert!(!out.contains("94d7ac8372d9"), "container id leaked");
        assert!(!out.contains("uvicorn app.main"), "COMMAND column leaked");
        assert!(!out.contains("hours ago  "), "CREATED column leaked");
    }

    #[test]
    fn test_custom_format_passthrough() {
        let input = "backend running\ndb running\n";
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
    fn token_savings_wide_table() {
        let input = include_str!("../tests/fixtures/docker_ps.txt");
        let out = filter(input);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(input) as f64;
        assert!(
            savings >= 0.30,
            "docker ps filter: expected ≥30% savings, got {:.1}%",
            savings * 100.0
        );
    }
}
