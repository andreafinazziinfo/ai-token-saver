use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("RTK_DB_PATH") {
        return PathBuf::from(p);
    }
    // XDG_DATA_HOME / ~/.local/share — matches the status-line's first probe path
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".local/share")
        });
    base.join("rtk/rtk.db")
}

fn open_db() -> Result<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let conn = Connection::open(&path)
        .with_context(|| format!("open db {}", path.display()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tracking (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            cmd              TEXT    NOT NULL,
            original_tokens  INTEGER NOT NULL,
            filtered_tokens  INTEGER NOT NULL,
            timestamp        TEXT    NOT NULL DEFAULT (datetime('now')),
            raw_output       TEXT
        );",
    )
    .context("create tracking table")?;
    
    // Migration: ensure raw_output column exists if table was created previously without it
    let _ = conn.execute("ALTER TABLE tracking ADD COLUMN raw_output TEXT", []);
    
    Ok(conn)
}

// Approximate token count: whitespace-split word count.
// Matches the test helper in git_diff.rs and is consistent with status-line
// percentage math. Not exact model tokens.
fn count_tokens(text: &str) -> i64 {
    text.split_whitespace().count() as i64
}

/// Record one filtered execution. Returns the ID of the inserted row.
pub fn record(cmd: &str, original: &str, filtered: &str, raw_output: &str) -> Result<i64> {
    let orig = count_tokens(original);
    let filt = count_tokens(filtered);
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO tracking (cmd, original_tokens, filtered_tokens, raw_output) \
         VALUES (?1, ?2, ?3, ?4)",
        params![cmd, orig, filt, raw_output],
    )
    .context("insert tracking row")?;
    let log_id = conn.last_insert_rowid();
    Ok(log_id)
}

/// Retrieve raw log output from the database by log ID.
pub fn get_raw_log(id: i64) -> Result<String> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT raw_output FROM tracking WHERE id = ?1"
    )?;
    let raw_output: Option<String> = stmt.query_row(params![id], |r| r.get(0))?;
    raw_output.context("log not found or has no raw output")
}

/// Query tracking DB and print savings report.
pub fn print_stats() -> Result<()> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT COUNT(*), SUM(original_tokens), SUM(filtered_tokens) FROM tracking"
    )?;
    
    let (count, original, filtered): (i64, Option<i64>, Option<i64>) = stmt.query_row([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    })?;

    let original = original.unwrap_or(0);
    let filtered = filtered.unwrap_or(0);
    let saved = original - filtered;
    let savings_pct = if original > 0 {
        (saved as f64 / original as f64) * 100.0
    } else {
        0.0
    };

    // Claude 3.5 Sonnet pricing: $3.00 / million input tokens
    let cost_saved = (saved as f64 / 1_000_000.0) * 3.00;

    println!("========================================");
    println!("          RTK TOKEN SAVINGS STATS       ");
    println!("========================================");
    println!("Total Commands Run:       {}", count);
    println!("Original Tokens:          {}", original);
    println!("Filtered Tokens:          {}", filtered);
    println!("Tokens Saved:             {} ({:.1}%)", saved, savings_pct);
    println!("Estimated API Cost Saved: ${:.4} USD", cost_saved);
    println!("========================================");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn count_tokens_basic() {
        assert_eq!(count_tokens("hello world foo"), 3);
        assert_eq!(count_tokens(""), 0);
        assert_eq!(count_tokens("  lots   of   space  "), 3);
    }

    #[test]
    fn record_writes_row() {
        let tmp = env::temp_dir().join(format!("rtk_test_{}.db", std::process::id()));
        env::set_var("RTK_DB_PATH", &tmp);

        let original = "a b c d e f g h i j"; // 10 tokens
        let filtered = "a b c";               // 3 tokens
        let log_id = record("git diff", original, filtered, original).expect("record failed");

        let conn = Connection::open(&tmp).unwrap();
        let (orig, filt): (i64, i64) = conn
            .query_row(
                "SELECT original_tokens, filtered_tokens FROM tracking LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("query failed");

        assert_eq!(orig, 10);
        assert_eq!(filt, 3);

        let raw = get_raw_log(log_id).expect("get_raw_log failed");
        assert_eq!(raw, original);

        // Also test print_stats doesn't error
        print_stats().expect("print_stats failed");

        std::fs::remove_file(&tmp).ok();
        env::remove_var("RTK_DB_PATH");
    }
}
