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
}
