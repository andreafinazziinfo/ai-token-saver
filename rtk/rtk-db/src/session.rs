use anyhow::{Context, Result};
use rusqlite::{params, Connection};

fn open_db() -> Result<Connection> {
    let path = if let Ok(p) = std::env::var("RTK_DB_PATH") {
        std::path::PathBuf::from(p)
    } else {
        let rtk_dir = std::env::current_dir()?.join(".rtk");
        if !rtk_dir.exists() {
            std::fs::create_dir_all(&rtk_dir).with_context(|| format!("create {}", rtk_dir.display()))?;
        }
        rtk_dir.join("rtk.db")
    };
    
    let conn = Connection::open(&path).with_context(|| format!("open db {}", path.display()))?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_state (
            project_path  TEXT NOT NULL,
            key           TEXT NOT NULL,
            val           TEXT NOT NULL,
            updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (project_path, key)
        )",
        [],
    ).context("create session_state table")?;
    
    Ok(conn)
}

fn get_project_path() -> String {
    std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn session_init() -> Result<()> {
    let conn = open_db()?;
    let pwd = get_project_path();
    
    let defaults = [
        ("decisions", "[]"),
        ("active_tasks", "[]"),
        ("context_files", "[]"),
        ("warnings", "[]"),
    ];

    for (k, v) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO session_state (project_path, key, val, updated_at) VALUES (?1, ?2, ?3, datetime('now'))",
            params![pwd, k, v],
        )?;
    }
    Ok(())
}

pub fn session_get() -> Result<String> {
    let conn = open_db()?;
    let pwd = get_project_path();
    
    let mut stmt = conn.prepare(
        "SELECT key, val FROM session_state WHERE project_path = ?1"
    )?;
    
    let mut map = serde_json::Map::new();
    let rows = stmt.query_map(params![pwd], |r| {
        let k: String = r.get(0)?;
        let v: String = r.get(1)?;
        Ok((k, v))
    })?;

    for row in rows {
        let (k, v) = row?;
        let parsed_val: serde_json::Value = serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v));
        map.insert(k, parsed_val);
    }

    let json_val = serde_json::Value::Object(map);
    Ok(serde_json::to_string_pretty(&json_val)?)
}

pub fn session_update(key: &str, value: &str) -> Result<()> {
    let conn = open_db()?;
    let pwd = get_project_path();
    
    let formatted_val = if serde_json::from_str::<serde_json::Value>(value).is_ok() {
        value.to_string()
    } else {
        serde_json::to_string(value)?
    };

    conn.execute(
        "INSERT INTO session_state (project_path, key, val, updated_at) VALUES (?1, ?2, ?3, datetime('now')) \
         ON CONFLICT(project_path, key) DO UPDATE SET val = ?3, updated_at = datetime('now')",
        params![pwd, key, formatted_val],
    )?;
    Ok(())
}

pub fn session_export() -> Result<String> {
    let conn = open_db()?;
    let pwd = get_project_path();
    
    let mut stmt = conn.prepare(
        "SELECT key, val FROM session_state WHERE project_path = ?1"
    )?;
    
    let rows = stmt.query_map(params![pwd], |r| {
        let k: String = r.get(0)?;
        let v: String = r.get(1)?;
        Ok((k, v))
    })?;

    let mut decisions = Vec::new();
    let mut active_tasks = Vec::new();
    let mut context_files = Vec::new();
    let mut warnings = Vec::new();

    for row in rows {
        let (k, v) = row?;
        let parsed: serde_json::Value = serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v));
        match k.as_str() {
            "decisions" => {
                if let Some(arr) = parsed.as_array() {
                    decisions = arr.iter().filter_map(|x| x.as_str()).map(String::from).collect();
                }
            }
            "active_tasks" => {
                if let Some(arr) = parsed.as_array() {
                    active_tasks = arr.iter().filter_map(|x| x.as_str()).map(String::from).collect();
                }
            }
            "context_files" => {
                if let Some(arr) = parsed.as_array() {
                    context_files = arr.iter().filter_map(|x| x.as_str()).map(String::from).collect();
                }
            }
            "warnings" => {
                if let Some(arr) = parsed.as_array() {
                    warnings = arr.iter().filter_map(|x| x.as_str()).map(String::from).collect();
                }
            }
            _ => {}
        }
    }

    let mut output = String::new();
    output.push_str("# 📋 RTK Agent Session Handoff State\n\n");
    output.push_str(&format!("*Project Path:* `{}`\n\n", pwd));
    
    output.push_str("## 🎯 Active Tasks\n");
    if active_tasks.is_empty() {
        output.push_str("- None\n");
    } else {
        for t in active_tasks {
            output.push_str(&format!("- [ ] {}\n", t));
        }
    }
    output.push_str("\n");

    output.push_str("## 💡 Decisions & Consensus\n");
    if decisions.is_empty() {
        output.push_str("- None\n");
    } else {
        for d in decisions {
            output.push_str(&format!("- {}\n", d));
        }
    }
    output.push_str("\n");

    output.push_str("## 📂 Context Files\n");
    if context_files.is_empty() {
        output.push_str("- None\n");
    } else {
        for f in context_files {
            output.push_str(&format!("- `{}`\n", f));
        }
    }
    output.push_str("\n");

    output.push_str("## ⚠️ Warnings & Guardrails\n");
    if warnings.is_empty() {
        output.push_str("- None\n");
    } else {
        for w in warnings {
            output.push_str(&format!("- {}\n", w));
        }
    }
    output.push_str("\n");

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_session_state_lifecycle() {
        let _lock = crate::tracking::DB_TEST_LOCK.lock().unwrap();
        let tmp = env::temp_dir().join(format!("rtk_test_session_{}.db", std::process::id()));
        env::set_var("RTK_DB_PATH", &tmp);

        crate::tracking::open_db().unwrap();

        session_init().unwrap();
        
        let state_json = session_get().unwrap();
        assert!(state_json.contains("decisions"));
        assert!(state_json.contains("active_tasks"));

        session_update("decisions", "[\"Decided to rename crate\"]").unwrap();
        session_update("active_tasks", "[\"Rename crate\", \"Update dependents\"]").unwrap();

        let state_updated = session_get().unwrap();
        assert!(state_updated.contains("Decided to rename crate"));

        let handoff_doc = session_export().unwrap();
        assert!(handoff_doc.contains("Decided to rename crate"));
        assert!(handoff_doc.contains("Active Tasks"));

        std::fs::remove_file(&tmp).ok();
        env::remove_var("RTK_DB_PATH");
    }
}
