use crate::tracking;
use anyhow::Result;
use std::io::{self, Read};

/// Record developer/agent reasoning/thought logs into the SQLite FTS5 database to preserve context
/// semantically without cluttering active chat windows.
pub fn run(content_args: Vec<String>) -> Result<()> {
    if content_args.len() == 1 {
        match content_args[0].as_str() {
            "inspect" => return inspect(),
            "gc" => return gc(),
            _ => {}
        }
    }

    let mut thought_content = String::new();

    // If arguments are provided, use them as the thought content
    if !content_args.is_empty() {
        thought_content = content_args.join(" ");
    } else {
        // Otherwise, read from standard input
        let mut stdin = io::stdin();
        stdin.read_to_string(&mut thought_content)?;
    }

    // Clean up excessive whitespace
    let cleaned_content = thought_content.trim().to_string();

    if cleaned_content.is_empty() {
        println!("[RTK] No thought content provided.");
        return Ok(());
    }

    // Record the thought in the SQLite FTS5 database for semantic search
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let key = format!("thought_{}", timestamp);
    let _ = tracking::memory_set(&key, &cleaned_content);

    // Also record in the logs for token savings calculation
    let _ = tracking::record(
        "think",
        &cleaned_content,
        "[Thought Stored]",
        &cleaned_content,
        None,
    );

    println!(
        "[RTK] Thought successfully offloaded to vector memory ({} bytes).",
        cleaned_content.len()
    );

    Ok(())
}

pub fn inspect() -> Result<()> {
    let conn = crate::tracking::open_db()?;
    let pwd = std::env::current_dir()?
        .to_string_lossy()
        .replace('\\', "/");
    
    let mut stmt = conn.prepare(
        "SELECT key, val, datetime(cast(substr(key, 9) as integer), 'unixepoch', 'localtime') AS dt \
         FROM project_memory \
         WHERE project_path = ?1 AND key LIKE 'thought_%' \
         ORDER BY key DESC"
    )?;
    
    let rows = stmt.query_map([pwd], |r| {
        let key: String = r.get(0)?;
        let val: String = r.get(1)?;
        let dt: Option<String> = r.get(2)?;
        Ok((key, val, dt.unwrap_or_else(|| "Unknown".to_string())))
    })?;
    
    println!("🧠 Offloaded thoughts in this project:");
    println!("========================================");
    
    let mut count = 0;
    for row in rows {
        let (_key, val, dt) = row?;
        println!("[{}]", dt);
        println!("{}", val);
        println!("----------------------------------------");
        count += 1;
    }
    
    if count == 0 {
        println!("No offloaded thoughts found.");
    } else {
        println!("Total thoughts: {}", count);
    }
    println!("========================================");
    Ok(())
}

pub fn gc() -> Result<()> {
    let conn = crate::tracking::open_db()?;
    let pwd = std::env::current_dir()?
        .to_string_lossy()
        .replace('\\', "/");
    
    let deleted = conn.execute(
        "DELETE FROM project_memory \
         WHERE project_path = ?1 AND key LIKE 'thought_%' \
         AND cast(substr(key, 9) as integer) < cast(strftime('%s', 'now', '-30 days') as integer)",
        [pwd],
    )?;
    
    println!("🗑️ Purged {} thought logs older than 30 days.", deleted);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_think_lifecycle() {
        let _lock = crate::tracking::DB_TEST_LOCK.lock().unwrap();
        let tmp = env::temp_dir().join(format!("rtk_test_think_{}.db", std::process::id()));
        env::set_var("RTK_DB_PATH", &tmp);

        crate::tracking::open_db().unwrap();

        // 1. Store a thought
        run(vec!["Test offloaded thought".to_string()]).unwrap();

        // 2. Inspect thoughts
        run(vec!["inspect".to_string()]).unwrap();

        // 3. Add an old thought manually (older than 30 days, e.g. 35 days ago = 35 * 24 * 3600 secs)
        let conn = crate::tracking::open_db().unwrap();
        let pwd = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let old_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (35 * 24 * 3600);
        let old_key = format!("thought_{}", old_time);
        
        conn.execute(
            "INSERT INTO project_memory (key, val, project_path) VALUES (?1, ?2, ?3)",
            [old_key, "Old thought that should be purged".to_string(), pwd],
        )
        .unwrap();

        // Verify it was inserted
        let count_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM project_memory WHERE key LIKE 'thought_%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count_before, 2);

        // 4. Run GC
        run(vec!["gc".to_string()]).unwrap();

        // Verify old thought was deleted, new one remains
        let count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM project_memory WHERE key LIKE 'thought_%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count_after, 1);

        std::fs::remove_file(&tmp).ok();
        env::remove_var("RTK_DB_PATH");
    }
}


