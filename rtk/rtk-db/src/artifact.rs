use anyhow::{Context, Result};
use rusqlite::{params, Connection};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Artifact {
    pub id: String,
    pub r#type: String,
    pub content: String,
    pub metadata_json: Option<String>,
    pub created_at: String,
}

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
        "CREATE TABLE IF NOT EXISTS artifacts (
            id            TEXT PRIMARY KEY,
            type          TEXT NOT NULL,
            content       TEXT NOT NULL,
            metadata_json TEXT,
            created_at    TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    ).context("create artifacts table")?;
    
    Ok(conn)
}

pub fn artifact_add(id: &str, r#type: &str, content: &str, metadata: Option<&str>) -> Result<()> {
    let conn = open_db()?;
    conn.execute(
        "INSERT OR REPLACE INTO artifacts (id, type, content, metadata_json, created_at) \
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![id, r#type, content, metadata],
    )?;
    Ok(())
}

pub fn artifact_list() -> Result<Vec<Artifact>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare("SELECT id, type, content, metadata_json, created_at FROM artifacts ORDER BY created_at DESC, id DESC")?;
    let rows = stmt.query_map([], |r| {
        Ok(Artifact {
            id: r.get(0)?,
            r#type: r.get(1)?,
            content: r.get(2)?,
            metadata_json: r.get(3)?,
            created_at: r.get(4)?,
        })
    })?;
    
    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn artifact_get(id: &str) -> Result<Artifact> {
    let conn = open_db()?;
    let mut stmt = conn.prepare("SELECT id, type, content, metadata_json, created_at FROM artifacts WHERE id = ?1")?;
    let art = stmt.query_row(params![id], |r| {
        Ok(Artifact {
            id: r.get(0)?,
            r#type: r.get(1)?,
            content: r.get(2)?,
            metadata_json: r.get(3)?,
            created_at: r.get(4)?,
        })
    })?;
    Ok(art)
}

pub fn artifact_gc() -> Result<usize> {
    let conn = open_db()?;
    let deleted = conn.execute(
        "DELETE FROM artifacts WHERE created_at < datetime('now', '-30 days')",
        [],
    )?;
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_artifact_lifecycle() {
        let _lock = crate::tracking::DB_TEST_LOCK.lock().unwrap();
        let tmp = env::temp_dir().join(format!("rtk_test_artifact_{}.db", std::process::id()));
        env::set_var("RTK_DB_PATH", &tmp);

        // Add artifact
        artifact_add("art-1", "reasoning", "First reasoning trace", Some("{\"model\":\"claude\"}")).unwrap();
        artifact_add("art-2", "summary", "A short summary", None).unwrap();

        // List
        let list = artifact_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "art-2");
        assert_eq!(list[1].id, "art-1");

        // Get
        let single = artifact_get("art-1").unwrap();
        assert_eq!(single.content, "First reasoning trace");
        assert_eq!(single.metadata_json, Some("{\"model\":\"claude\"}".to_string()));

        // GC old
        // Manually insert an old artifact
        let conn = open_db().unwrap();
        conn.execute(
            "INSERT INTO artifacts (id, type, content, metadata_json, created_at) \
             VALUES ('old-art', 'cli-log', 'logs...', NULL, datetime('now', '-32 days'))",
            [],
        ).unwrap();

        let count_before = artifact_list().unwrap().len();
        assert_eq!(count_before, 3);

        let purged = artifact_gc().unwrap();
        assert_eq!(purged, 1);

        let count_after = artifact_list().unwrap().len();
        assert_eq!(count_after, 2);

        std::fs::remove_file(&tmp).ok();
        env::remove_var("RTK_DB_PATH");
    }
}
