use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct UserConfig {
    pub denied_commands: Vec<String>,
    pub custom_dlp_patterns: Vec<String>,
}

impl UserConfig {
    pub fn load() -> Self {
        let mut config = UserConfig {
            denied_commands: Vec::new(),
            custom_dlp_patterns: Vec::new(),
        };

        // 1. Try to load from user global home folder: ~/.config/rtk/config.json
        if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
            let global_path = Path::new(&home).join(".config/rtk/config.json");
            if let Ok(content) = fs::read_to_string(&global_path) {
                let _ = config.merge_from_str(&content);
            }
        }

        // 2. Try to load from local directory: ./.rtk.json
        let local_path = Path::new(".rtk.json");
        if let Ok(content) = fs::read_to_string(local_path) {
            let _ = config.merge_from_str(&content);
        }

        config
    }

    pub fn merge_from_str(&mut self, content: &str) -> Result<(), serde_json::Error> {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(arr) = val.get("denied_commands").and_then(|v| v.as_array()) {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        self.denied_commands.push(s.to_string());
                    }
                }
            }
            if let Some(dlp_obj) = val.get("dlp") {
                if let Some(arr) = dlp_obj.get("custom_patterns").and_then(|v| v.as_array()) {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            self.custom_dlp_patterns.push(s.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG: UserConfig = UserConfig::load();
}

pub fn create_default_config() -> Result<(), std::io::Error> {
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let config_dir = Path::new(&home).join(".config/rtk");
        let config_path = config_dir.join("config.json");
        if !config_path.exists() {
            fs::create_dir_all(&config_dir)?;
            let default_json = r#"{
  "denied_commands": [
    "git push.*--force",
    "git reset --hard"
  ],
  "dlp": {
    "custom_patterns": [
      "MY_PROJECT_SECRET_[a-zA-Z0-9]{12}"
    ]
  }
}"#;
            fs::write(config_path, default_json)?;
        }
    }
    Ok(())
}

fn global_config_path() -> Option<std::path::PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|home| Path::new(&home).join(".config/rtk/config.json"))
}

fn modify_config<F>(f: F) -> anyhow::Result<()>
where
    F: FnOnce(&mut serde_json::Map<String, serde_json::Value>),
{
    create_default_config().map_err(|e| anyhow::anyhow!("failed to create default config: {e}"))?;

    let path = global_config_path()
        .ok_or_else(|| anyhow::anyhow!("could not determine global config directory"))?;

    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("failed to read global config: {e}"))?;

    let mut val: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse global config JSON: {e}"))?;

    if let Some(obj) = val.as_object_mut() {
        f(obj);
    } else {
        let mut obj = serde_json::Map::new();
        f(&mut obj);
        val = serde_json::Value::Object(obj);
    }

    let updated = serde_json::to_string_pretty(&val)
        .map_err(|e| anyhow::anyhow!("failed to serialize updated config: {e}"))?;

    fs::write(&path, updated).map_err(|e| anyhow::anyhow!("failed to write global config: {e}"))?;

    Ok(())
}

pub fn config_show() -> anyhow::Result<()> {
    let merged_json = serde_json::json!({
        "denied_commands": CONFIG.denied_commands,
        "dlp": {
            "custom_patterns": CONFIG.custom_dlp_patterns
        }
    });
    println!("{}", serde_json::to_string_pretty(&merged_json)?);
    Ok(())
}

pub fn config_deny_add(pattern: &str) -> anyhow::Result<()> {
    modify_config(|obj| {
        let denied = obj
            .entry("denied_commands")
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        if let Some(arr) = denied.as_array_mut() {
            let item = serde_json::Value::String(pattern.to_string());
            if !arr.contains(&item) {
                arr.push(item);
            }
        }
    })
}

pub fn config_dlp_add(pattern: &str) -> anyhow::Result<()> {
    modify_config(|obj| {
        let dlp = obj
            .entry("dlp")
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if let Some(dlp_obj) = dlp.as_object_mut() {
            let custom_patterns = dlp_obj
                .entry("custom_patterns")
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            if let Some(arr) = custom_patterns.as_array_mut() {
                let item = serde_json::Value::String(pattern.to_string());
                if !arr.contains(&item) {
                    arr.push(item);
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_from_str() {
        let mut config = UserConfig::default();
        let json = r#"{
            "denied_commands": ["rm -rf", "git push --force"],
            "dlp": {
                "custom_patterns": ["SECRET_[0-9]+"]
            }
        }"#;
        config.merge_from_str(json).unwrap();
        assert_eq!(config.denied_commands.len(), 2);
        assert_eq!(config.denied_commands[0], "rm -rf");
        assert_eq!(config.denied_commands[1], "git push --force");
        assert_eq!(config.custom_dlp_patterns.len(), 1);
        assert_eq!(config.custom_dlp_patterns[0], "SECRET_[0-9]+");
    }

    #[test]
    fn test_merge_missing_keys() {
        let mut config = UserConfig::default();
        let json = r#"{
            "denied_commands": ["rm -rf"]
        }"#;
        config.merge_from_str(json).unwrap();
        assert_eq!(config.denied_commands.len(), 1);
        assert_eq!(config.custom_dlp_patterns.len(), 0);
    }

    #[test]
    fn test_modify_config() {
        let temp_dir =
            std::env::temp_dir().join(format!("rtk_config_modify_test_{}", rand_suffix()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Temporarily override HOME and USERPROFILE env vars
        let original_home = std::env::var_os("HOME");
        let original_userprofile = std::env::var_os("USERPROFILE");
        std::env::set_var("HOME", &temp_dir);
        std::env::set_var("USERPROFILE", &temp_dir);

        // Add to deny
        config_deny_add("forbidden_cmd").unwrap();
        // Add to dlp
        config_dlp_add("PAT_[a-z]+").unwrap();

        // Read file and check
        let path = temp_dir.join(".config/rtk/config.json");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        let val: serde_json::Value = serde_json::from_str(&content).unwrap();

        let denied = val["denied_commands"].as_array().unwrap();
        assert!(denied.contains(&serde_json::Value::String("forbidden_cmd".to_string())));
        assert!(denied.contains(&serde_json::Value::String("git reset --hard".to_string()))); // from default template

        let custom_patterns = val["dlp"]["custom_patterns"].as_array().unwrap();
        assert!(custom_patterns.contains(&serde_json::Value::String("PAT_[a-z]+".to_string())));

        // Test config_show does not error
        assert!(config_show().is_ok());

        // Restore env vars
        if let Some(h) = original_home {
            std::env::set_var("HOME", h);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(up) = original_userprofile {
            std::env::set_var("USERPROFILE", up);
        } else {
            std::env::remove_var("USERPROFILE");
        }

        fs::remove_dir_all(temp_dir).unwrap();
    }

    fn rand_suffix() -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }
}
