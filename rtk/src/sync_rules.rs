use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Scans the directory and syncs root rule files to subprojects
pub fn run(root: &Path) -> Result<()> {
    let root_cursor = root.join(".cursor").join("rules");
    let root_agents = root.join(".agents").join("rules");

    if !root_cursor.exists() && !root_agents.exists() {
        println!("No rules found at root .cursor/rules or .agents/rules to sync.");
        return Ok(());
    }

    println!("Syncing rules from root to subprojects...");
    
    let entries = fs::read_dir(root)
        .context("failed to read root directory for syncing rules")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Skip common system/dependency folders
            if dir_name.starts_with('.')
                || dir_name == "node_modules"
                || dir_name == "target"
                || dir_name == "scratch"
            {
                continue;
            }

            // Check if this subdirectory is a project (has Cargo.toml, package.json, etc.)
            if is_project_directory(&path) {
                println!("  -> Syncing to project: {}", dir_name);
                
                if root_cursor.exists() {
                    let sub_cursor = path.join(".cursor").join("rules");
                    sync_rule_dir(&root_cursor, &sub_cursor)?;
                }

                if root_agents.exists() {
                    let sub_agents = path.join(".agents").join("rules");
                    sync_rule_dir(&root_agents, &sub_agents)?;
                }
            }
        }
    }

    println!("✅ Rules synced successfully.");
    Ok(())
}

fn is_project_directory(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join("pyproject.toml").exists()
        || path.join("requirements.txt").exists()
        || path.join(".git").exists()
}

fn sync_rule_dir(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)
        .with_context(|| format!("failed to create directory: {}", dest.display()))?;

    // Copy all .mdc files from source to destination
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        
        if src_path.is_file() {
            if let Some(ext) = src_path.extension() {
                if ext == "mdc" {
                    let file_name = src_path.file_name().unwrap();
                    let dest_path = dest.join(file_name);
                    fs::copy(&src_path, &dest_path)
                        .with_context(|| format!("failed to copy rule to {}", dest_path.display()))?;
                }
            }
        }
    }
    Ok(())
}
