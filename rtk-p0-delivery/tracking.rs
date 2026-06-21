use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub(crate) fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("RTK_DB_PATH") {
        return PathBuf::from(p);
    }

    // On Windows, prefer LOCALAPPDATA
    if cfg!(target_os = "windows") {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local_appdata).join("rtk").join("rtk.db");
        }
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

pub(crate) fn open_db() -> Result<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let conn = Connection::open(&path).with_context(|| format!("open db {}", path.display()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tracking (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            cmd              TEXT    NOT NULL,
            original_tokens  INTEGER NOT NULL,
            filtered_tokens  INTEGER NOT NULL,
            timestamp        TEXT    NOT NULL DEFAULT (datetime('now')),
            raw_output       TEXT
        );
        CREATE TABLE IF NOT EXISTS project_memory (
            key              TEXT    NOT NULL,
            val              TEXT    NOT NULL,
            project_path     TEXT    NOT NULL,
            PRIMARY KEY (key, project_path)
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS project_memory_fts USING fts5(
            key, 
            val, 
            project_path UNINDEXED, 
            content='project_memory', 
            content_rowid='rowid'
        );
        
        -- Triggers to keep FTS index in sync
        CREATE TRIGGER IF NOT EXISTS project_memory_ai AFTER INSERT ON project_memory BEGIN
          INSERT INTO project_memory_fts(rowid, key, val, project_path) VALUES (new.rowid, new.key, new.val, new.project_path);
        END;
        CREATE TRIGGER IF NOT EXISTS project_memory_ad AFTER DELETE ON project_memory BEGIN
          INSERT INTO project_memory_fts(project_memory_fts, rowid, key, val, project_path) VALUES('delete', old.rowid, old.key, old.val, old.project_path);
        END;
        CREATE TRIGGER IF NOT EXISTS project_memory_au AFTER UPDATE ON project_memory BEGIN
          INSERT INTO project_memory_fts(project_memory_fts, rowid, key, val, project_path) VALUES('delete', old.rowid, old.key, old.val, old.project_path);
          INSERT INTO project_memory_fts(rowid, key, val, project_path) VALUES (new.rowid, new.key, new.val, new.project_path);
        END;",
    )
    .context("create DB tables")?;

    // Migration: ensure raw_output column exists if table was created previously without it
    let _ = conn.execute("ALTER TABLE tracking ADD COLUMN raw_output TEXT", []);
    let _ = conn.execute("ALTER TABLE tracking ADD COLUMN model TEXT", []);
    let _ = conn.execute("ALTER TABLE tracking ADD COLUMN project TEXT", []);
    let _ = conn.execute("ALTER TABLE tracking ADD COLUMN branch TEXT", []);
    let _ = conn.execute("ALTER TABLE tracking ADD COLUMN duration_ms INTEGER", []);

    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS session_state (
            project_path  TEXT NOT NULL,
            key           TEXT NOT NULL,
            val           TEXT NOT NULL,
            updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (project_path, key)
        )",
        [],
    );

    Ok(conn)
}

fn get_git_branch() -> String {
    let mut dir = std::env::current_dir().ok();
    while let Some(path) = dir {
        let git_dir = path.join(".git");
        if git_dir.is_dir() {
            let head_path = git_dir.join("HEAD");
            if let Ok(content) = std::fs::read_to_string(head_path) {
                let trimmed = content.trim();
                if trimmed.starts_with("ref: refs/heads/") {
                    return trimmed["ref: refs/heads/".len()..].to_string();
                } else if !trimmed.is_empty() {
                    // Detached HEAD, return first 7 chars of hash
                    return trimmed.chars().take(7).collect();
                }
            }
        }
        dir = path.parent().map(|p| p.to_path_buf());
    }
    "detached".to_string()
}

/// Estimate the token count of a string slice.
/// Uses a simple heuristic of approximately 1 token per 4 characters.
pub fn count_tokens(text: &str) -> i64 {
    (text.len().div_ceil(4)) as i64
}

/// Returns a warning string if the output is dangerously large.
pub fn check_autonomy(text: &str) -> Option<&'static str> {
    if count_tokens(text) > 3000 {
        Some("\n[RTK AUTONOMY: Contesto saturo. L'output appena generato è enorme. Usa Profile MAX o sii molto sintetico nella prossima risposta per evitare di saturare la memoria.]")
    } else {
        None
    }
}

/// Record one filtered execution. Returns the ID of the inserted row.
pub fn record(
    cmd: &str,
    original: &str,
    filtered: &str,
    raw_output: &str,
    duration_ms: Option<i64>,
) -> Result<i64> {
    let orig = count_tokens(original);
    let filt = count_tokens(filtered);
    let conn = open_db()?;

    // Automatic DB garbage collection: purge logs older than 30 days during record calls
    let _ = conn.execute(
        "DELETE FROM tracking WHERE timestamp < datetime('now', '-30 days')",
        [],
    );

    let mut model_name = String::from("Unknown Model");
    for var in &[
        "CLAUDE_MODEL",
        "LLM_MODEL",
        "OPENAI_MODEL",
        "GEMINI_MODEL",
        "ANTHROPIC_MODEL",
        "GITHUB_MODEL",
    ] {
        if let Ok(m) = std::env::var(var) {
            model_name = m;
            break;
        }
    }

    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| String::from("Unknown Project"));

    let branch_name = get_git_branch();

    conn.execute(
        "INSERT INTO tracking (cmd, original_tokens, filtered_tokens, raw_output, model, project, branch, duration_ms) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            cmd,
            orig,
            filt,
            raw_output,
            model_name,
            project_name,
            branch_name,
            duration_ms
        ],
    )
    .context("insert tracking row")?;
    let log_id = conn.last_insert_rowid();
    Ok(log_id)
}

/// Force database garbage collection (manual TTL purging) and VACUUM.
/// Returns the number of purged rows.
pub fn gc() -> Result<usize> {
    let conn = open_db()?;
    let deleted = conn
        .execute(
            "DELETE FROM tracking WHERE timestamp < datetime('now', '-30 days')",
            [],
        )
        .context("execute GC delete query")?;

    // Shrink database file to reclaim deleted space
    let _ = conn.execute("VACUUM", []);

    Ok(deleted)
}

/// Retrieve raw log output from the database by log ID.
pub fn get_raw_log(id: i64) -> Result<String> {
    let conn = open_db()?;
    let mut stmt = conn.prepare("SELECT raw_output FROM tracking WHERE id = ?1")?;
    let raw_output: Option<String> = stmt.query_row(params![id], |r| r.get(0))?;
    raw_output.context("log not found or has no raw output")
}

/// Query tracking DB and print savings report.
pub fn print_stats() -> Result<()> {
    let conn = open_db()?;
    let mut stmt =
        conn.prepare("SELECT COUNT(*), SUM(original_tokens), SUM(filtered_tokens) FROM tracking")?;

    let (count, original, filtered): (i64, Option<i64>, Option<i64>) =
        stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;

    let original = original.unwrap_or(0);
    let filtered = filtered.unwrap_or(0);
    let saved = original - filtered;
    let savings_pct = if original > 0 {
        (saved as f64 / original as f64) * 100.0
    } else {
        0.0
    };

    // Calculate actual cost saved by summing savings for each command
    let mut stmt_saved = conn.prepare("SELECT original_tokens - filtered_tokens, COALESCE(model, '') FROM tracking")?;
    let rows = stmt_saved.query_map([], |r| {
        let saved_tokens: i64 = r.get(0)?;
        let model: String = r.get(1)?;
        Ok((saved_tokens, model))
    })?;

    let mut cost_saved = 0.0;
    for row in rows {
        let (tokens, model) = row?;
        cost_saved += crate::pricing::calculate_savings(tokens, &model);
    }

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

/// Fetch aggregate savings stats for the dashboard.
pub fn get_savings_data() -> Result<(i64, i64, i64, i64, f64)> {
    let conn = open_db()?;
    let mut stmt =
        conn.prepare("SELECT COUNT(*), SUM(original_tokens), SUM(filtered_tokens) FROM tracking")?;

    let (count, original, filtered): (i64, Option<i64>, Option<i64>) =
        stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;

    let original = original.unwrap_or(0);
    let filtered = filtered.unwrap_or(0);
    let saved = original - filtered;
    
    // Calculate actual cost saved by summing savings for each command
    let mut stmt_saved = conn.prepare("SELECT original_tokens - filtered_tokens, COALESCE(model, '') FROM tracking")?;
    let rows = stmt_saved.query_map([], |r| {
        let saved_tokens: i64 = r.get(0)?;
        let model: String = r.get(1)?;
        Ok((saved_tokens, model))
    })?;

    let mut cost_saved = 0.0;
    for row in rows {
        let (tokens, model) = row?;
        cost_saved += crate::pricing::calculate_savings(tokens, &model);
    }

    Ok((count, original, filtered, saved, cost_saved))
}

/// Fetch command breakdown statistics (name, invocations, saved tokens).
pub fn get_command_breakdown() -> Result<Vec<(String, i64, i64)>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT cmd, COUNT(*), SUM(original_tokens - filtered_tokens) FROM tracking GROUP BY cmd ORDER BY COUNT(*) DESC"
    )?;

    let rows = stmt.query_map([], |r| {
        let cmd: String = r.get(0)?;
        let count: i64 = r.get(1)?;
        let saved: Option<i64> = r.get(2)?;
        Ok((cmd, count, saved.unwrap_or(0)))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Fetch complete audit breakdown statistics (cmd, count, original, filtered, saved)
#[allow(clippy::type_complexity)]
pub fn get_audit_breakdown() -> Result<Vec<(String, i64, i64, i64, i64)>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT cmd, COUNT(*), SUM(original_tokens), SUM(filtered_tokens), SUM(original_tokens - filtered_tokens) \
         FROM tracking GROUP BY cmd ORDER BY COUNT(*) DESC"
    )?;

    let rows = stmt.query_map([], |r| {
        let cmd: String = r.get(0)?;
        let count: i64 = r.get(1)?;
        let original: Option<i64> = r.get(2)?;
        let filtered: Option<i64> = r.get(3)?;
        let saved: Option<i64> = r.get(4)?;
        Ok((
            cmd,
            count,
            original.unwrap_or(0),
            filtered.unwrap_or(0),
            saved.unwrap_or(0),
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Save a project memory key-value pair.
pub fn memory_set(key: &str, val: &str) -> Result<()> {
    let conn = open_db()?;
    let pwd = std::env::current_dir()?
        .to_string_lossy()
        .replace('\\', "/");
    conn.execute(
        "INSERT OR REPLACE INTO project_memory (key, val, project_path) VALUES (?1, ?2, ?3)",
        params![key, val, pwd],
    )
    .context("insert project memory")?;
    Ok(())
}

/// Retrieve a project memory value by key.
pub fn memory_get(key: &str) -> Result<String> {
    let conn = open_db()?;
    let pwd = std::env::current_dir()?
        .to_string_lossy()
        .replace('\\', "/");
    let mut stmt =
        conn.prepare("SELECT val FROM project_memory WHERE key = ?1 AND project_path = ?2")?;
    let val: String = stmt.query_row(params![key, pwd], |r| r.get(0))?;
    Ok(val)
}

/// List all memory key-value pairs for the current project.
pub fn memory_list() -> Result<Vec<(String, String)>> {
    let conn = open_db()?;
    let pwd = std::env::current_dir()?
        .to_string_lossy()
        .replace('\\', "/");
    let mut stmt = conn.prepare("SELECT key, val FROM project_memory WHERE project_path = ?1")?;
    let rows = stmt.query_map(params![pwd], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Search project memory using FTS5 semantic full-text search.
pub fn memory_search(query: &str) -> Result<Vec<(String, String)>> {
    let conn = open_db()?;
    let pwd = std::env::current_dir()?
        .to_string_lossy()
        .replace('\\', "/");

    // FTS syntax: wrap words with asterisks for fuzzy prefix matching
    let words: Vec<&str> = query.split_whitespace().collect();
    let fts_query = if words.len() > 1 {
        let mapped: Vec<String> = words.iter().map(|w| format!("{}*", w)).collect();
        mapped.join(" OR ")
    } else {
        format!("{}*", query)
    };

    let mut stmt = conn.prepare(
        "SELECT key, val FROM project_memory_fts 
         WHERE project_path = ?1 AND project_memory_fts MATCH ?2 
         ORDER BY rank LIMIT 5",
    )?;

    let rows = stmt.query_map(params![pwd, fts_query], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Generate a detailed token savings audit report.
/// Calculates aggregate statistics, projects costs across various models,
/// lists command breakdowns, prints a summary to stdout, and writes a Markdown report.
pub fn run_audit(output_path: &str) -> Result<()> {
    let (count, original, filtered, saved, cost_saved) = get_savings_data()?;
    let breakdown = get_audit_breakdown()?;

    let savings_pct = if original > 0 {
        (saved as f64 / original as f64) * 100.0
    } else {
        0.0
    };

    let hours_saved = (count as f64 * 22.8) / 3600.0;

    let opus_price = crate::pricing::get_model_price("claude-3-opus").map(|m| m.input_price_per_mtok).unwrap_or(15.0);
    let sonnet_price = crate::pricing::get_model_price("claude-3.5-sonnet").map(|m| m.input_price_per_mtok).unwrap_or(3.0);
    let gpt4o_price = crate::pricing::get_model_price("gpt-4o").map(|m| m.input_price_per_mtok).unwrap_or(2.50);
    let gemini_pro_price = crate::pricing::get_model_price("gemini-2.5-pro").map(|m| m.input_price_per_mtok).unwrap_or(1.25);
    let gpt4o_mini_price = crate::pricing::get_model_price("gpt-4o-mini").map(|m| m.input_price_per_mtok).unwrap_or(0.15);
    let gemini_flash_price = crate::pricing::get_model_price("gemini-2.5-flash").map(|m| m.input_price_per_mtok).unwrap_or(0.15);

    let opus_savings = crate::pricing::calculate_savings(saved, "claude-3-opus");
    let sonnet_savings = crate::pricing::calculate_savings(saved, "claude-3.5-sonnet");
    let gpt4o_savings = crate::pricing::calculate_savings(saved, "gpt-4o");
    let gemini_pro_savings = crate::pricing::calculate_savings(saved, "gemini-2.5-pro");
    let gpt4o_mini_savings = crate::pricing::calculate_savings(saved, "gpt-4o-mini");
    let gemini_flash_savings = crate::pricing::calculate_savings(saved, "gemini-2.5-flash");

    // Build command breakdown rows
    let mut rows_md = String::new();
    for (cmd, cnt, orig, filt, svd) in &breakdown {
        let pct = if *orig > 0 {
            (*svd as f64 / *orig as f64) * 100.0
        } else {
            0.0
        };
        rows_md.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {:.1}% |\n",
            cmd, cnt, orig, filt, svd, pct
        ));
    }

    let conn = open_db()?;
    let now: String = conn.query_row("SELECT datetime('now', 'localtime')", [], |r| r.get(0))?;

    let report_content = format!(
        "# 📊 RTK Efficiency & Token Savings Audit\n\n\
         Generated on: {} (local time)\n\n\
         ## 📈 Summary Statistics\n\n\
         | Metric | Value |\n\
         | :--- | :--- |\n\
         | **Total Commands Intercepted** | {} |\n\
         | **Original Tokens** | {} |\n\
         | **Filtered Tokens** | {} |\n\
         | **Tokens Saved** | {} ({:.1}%) |\n\
         | **Estimated API Cost Saved (Dynamic)** | ${:.4} USD |\n\
         | **Estimated Developer Hours Saved** | {:.2} hrs |\n\n\
         ## 💰 Cost Savings Projection by Model\n\n\
         This table projects what would have been saved under different LLM pricing models for the same volume of saved tokens ({} tokens):\n\n\
         | Model | Input Price / MTok | Estimated Savings |\n\
         | :--- | ---: | ---: |\n\
         | **Claude 3 Opus** | ${:.2} | ${:.4} |\n\
         | **Claude 3.5 Sonnet** | ${:.2} | ${:.4} |\n\
         | **GPT-4o** | ${:.2} | ${:.4} |\n\
         | **Gemini 2.5 Pro** | ${:.2} | ${:.4} |\n\
         | **GPT-4o mini** | ${:.2} | ${:.4} |\n\
         | **Gemini 2.5 Flash** | ${:.2} | ${:.4} |\n\n\
         > [!NOTE]\n\
         > Savings calculations are based on input token reductions. Wait-time savings are calculated at a conservative rate of 22.8 seconds of developer waiting time saved per command.\n\n\
         ## 🗃️ Command Breakdown\n\n\
         | Command | Invocations | Original Tokens | Filtered Tokens | Tokens Saved | Savings % |\n\
         | :--- | ---: | ---: | ---: | ---: | ---: |\n\
         {}",
        now, count, original, filtered, saved, savings_pct, cost_saved, hours_saved, saved,
        opus_price, opus_savings,
        sonnet_price, sonnet_savings,
        gpt4o_price, gpt4o_savings,
        gemini_pro_price, gemini_pro_savings,
        gpt4o_mini_price, gpt4o_mini_savings,
        gemini_flash_price, gemini_flash_savings,
        rows_md
    );

    // Print summary to stdout
    println!("==========================================================");
    println!("📊                RTK TOKEN SAVINGS AUDIT                ");
    println!("==========================================================");
    println!("Total Commands Intercepted:      {}", count);
    println!("Original Tokens:                 {}", original);
    println!("Filtered Tokens:                 {}", filtered);
    println!(
        "Tokens Saved:                    {} ({:.1}%)",
        saved, savings_pct
    );
    println!("Estimated API Cost Saved (USD):  ${:.4}", cost_saved);
    println!("Estimated Developer Hours Saved: {:.2} hrs", hours_saved);
    println!("----------------------------------------------------------");

    // Write report to file
    std::fs::write(output_path, report_content)
        .with_context(|| format!("failed to write audit report to {}", output_path))?;

    println!("Audit report successfully written to: {}", output_path);
    println!("==========================================================");

    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CommandLog {
    pub id: i64,
    pub cmd: String,
    pub original_tokens: i64,
    pub filtered_tokens: i64,
    pub timestamp: String,
    pub raw_output: Option<String>,
    pub model: Option<String>,
    pub project: Option<String>,
    pub branch: Option<String>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DailySavings {
    pub day: String,
    pub original: i64,
    pub filtered: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ModelSavings {
    pub model: String,
    pub invocations: i64,
    pub saved: i64,
}

/// Retrieve the most recent command logs from the database.
pub fn get_recent_logs(limit: usize) -> Result<Vec<CommandLog>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, cmd, original_tokens, filtered_tokens, timestamp, raw_output, model, project, branch, duration_ms \
         FROM tracking ORDER BY id DESC LIMIT ?1"
    )?;

    let rows = stmt.query_map(params![limit], |r| {
        Ok(CommandLog {
            id: r.get(0)?,
            cmd: r.get(1)?,
            original_tokens: r.get(2)?,
            filtered_tokens: r.get(3)?,
            timestamp: r.get(4)?,
            raw_output: r.get(5)?,
            model: r.get(6)?,
            project: r.get(7)?,
            branch: r.get(8)?,
            duration_ms: r.get(9)?,
        })
    })?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row?);
    }
    Ok(logs)
}

/// Retrieve time-series token usage and savings grouped by day.
pub fn get_daily_savings() -> Result<Vec<DailySavings>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT strftime('%Y-%m-%d', timestamp) AS day, SUM(original_tokens), SUM(filtered_tokens) \
         FROM tracking GROUP BY day ORDER BY day ASC"
    )?;

    let rows = stmt.query_map([], |r| {
        let day: String = r.get(0)?;
        let original: Option<i64> = r.get(1)?;
        let filtered: Option<i64> = r.get(2)?;
        Ok(DailySavings {
            day,
            original: original.unwrap_or(0),
            filtered: filtered.unwrap_or(0),
        })
    })?;

    let mut savings = Vec::new();
    for row in rows {
        savings.push(row?);
    }
    Ok(savings)
}

/// Retrieve model savings distribution.
pub fn get_model_savings() -> Result<Vec<ModelSavings>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT COALESCE(model, 'Unknown Model') AS md, COUNT(*), SUM(original_tokens - filtered_tokens) \
         FROM tracking GROUP BY md ORDER BY COUNT(*) DESC"
    )?;

    let rows = stmt.query_map([], |r| {
        let model: String = r.get(0)?;
        let invocations: i64 = r.get(1)?;
        let saved: Option<i64> = r.get(2)?;
        Ok(ModelSavings {
            model,
            invocations,
            saved: saved.unwrap_or(0),
        })
    })?;

    let mut model_stats = Vec::new();
    for row in rows {
        model_stats.push(row?);
    }
    Ok(model_stats)
}

#[cfg(test)]
pub(crate) static DB_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn count_tokens_basic() {
        assert_eq!(count_tokens("hello world foo"), 4);
        assert_eq!(count_tokens(""), 0);
        assert_eq!(count_tokens("  lots   of   space  "), 6);
    }

    #[test]
    fn record_writes_row() {
        let _lock = DB_TEST_LOCK.lock().unwrap();
        let tmp = env::temp_dir().join(format!("rtk_test_{}.db", std::process::id()));
        env::set_var("RTK_DB_PATH", &tmp);

        let original = "a b c d e f g h i j"; // 19 chars -> 5 tokens
        let filtered = "a b c"; // 5 chars -> 2 tokens
        let log_id = record("git diff", original, filtered, original, Some(150)).expect("record failed");

        let conn = Connection::open(&tmp).unwrap();
        let (orig, filt): (i64, i64) = conn
            .query_row(
                "SELECT original_tokens, filtered_tokens FROM tracking LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("query failed");

        assert_eq!(orig, 5);
        assert_eq!(filt, 2);

        let id_in_db: i64 = conn.query_row("SELECT id FROM tracking", [], |r| r.get(0)).unwrap();
        println!("DIAGNOSTIC: log_id returned = {}, id in DB = {}", log_id, id_in_db);

        let raw = get_raw_log(log_id).expect("get_raw_log failed");
        assert_eq!(raw, original);

        // Also test print_stats doesn't error
        print_stats().expect("print_stats failed");

        // Test project memory functions sequentially (prevents env var race condition)
        memory_set("port", "8080").unwrap();
        let val = memory_get("port").unwrap();
        assert_eq!(val, "8080");

        memory_set("host", "localhost").unwrap();
        let list = memory_list().unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&("port".to_string(), "8080".to_string())));
        assert!(list.contains(&("host".to_string(), "localhost".to_string())));

        // Test manual GC
        // 1. Insert an old record (older than 30 days)
        conn.execute(
            "INSERT INTO tracking (cmd, original_tokens, filtered_tokens, raw_output, timestamp) \
             VALUES ('old_cmd', 10, 2, 'old output', datetime('now', '-31 days'))",
            [],
        )
        .unwrap();

        // Check it is indeed inserted
        let total_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracking", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total_count, 2);

        // 2. Call gc()
        let purged = gc().expect("gc failed");
        assert_eq!(purged, 1);

        // Check it is deleted
        let total_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracking", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total_count, 1);

        // Test automatic GC during record()
        // 1. Insert an old record again
        conn.execute(
            "INSERT INTO tracking (cmd, original_tokens, filtered_tokens, raw_output, timestamp) \
             VALUES ('old_cmd_2', 10, 2, 'old output 2', datetime('now', '-32 days'))",
            [],
        )
        .unwrap();

        // 2. Call record() and verify it triggers automatic purge
        record("new_cmd", "foo bar", "foo", "foo bar", None).unwrap();

        // 3. Verify total rows is still 2 (the first recorded cmd, plus the new_cmd. The old_cmd_2 should be auto-purged)
        let total_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracking", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total_count, 2);

        // Test get_audit_breakdown
        let breakdown = get_audit_breakdown().unwrap();
        assert_eq!(breakdown.len(), 2);

        // Test run_audit writes to file successfully
        let audit_md_path = env::temp_dir().join(format!("rtk_audit_{}.md", std::process::id()));
        run_audit(&audit_md_path.to_string_lossy()).unwrap();
        assert!(audit_md_path.exists());
        let content = std::fs::read_to_string(&audit_md_path).unwrap();
        assert!(content.contains("# 📊 RTK Efficiency & Token Savings Audit"));
        assert!(content.contains("Summary Statistics"));
        assert!(content.contains("Cost Savings Projection by Model"));
        assert!(content.contains("Command Breakdown"));
        assert!(content.contains("git diff"));
        assert!(content.contains("new_cmd"));
        std::fs::remove_file(&audit_md_path).ok();

        std::fs::remove_file(&tmp).ok();
        env::remove_var("RTK_DB_PATH");
    }

    #[test]
    fn test_new_dashboard_queries() {
        let _lock = DB_TEST_LOCK.lock().unwrap();
        let tmp = env::temp_dir().join(format!("rtk_test_queries_{}.db", std::process::id()));
        env::set_var("RTK_DB_PATH", &tmp);

        record("git status", "untracked files...", "untracked", "raw log status", Some(42)).unwrap();
        record("cargo build", "compiling...", "ok", "raw log build", Some(120)).unwrap();

        let logs = get_recent_logs(10).unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].cmd, "cargo build");
        assert_eq!(logs[0].duration_ms, Some(120));
        assert_eq!(logs[1].cmd, "git status");
        assert_eq!(logs[1].duration_ms, Some(42));

        let daily = get_daily_savings().unwrap();
        assert!(!daily.is_empty());

        let models = get_model_savings().unwrap();
        assert!(!models.is_empty());

        std::fs::remove_file(&tmp).ok();
        env::remove_var("RTK_DB_PATH");
    }
}
