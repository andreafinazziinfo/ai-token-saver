use std::fs;
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ProfileSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_line_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minify_json: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_only: Option<bool>,
}

impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            max_line_length: None,
            remove_comments: Some(false),
            minify_json: Some(false),
            json_only: Some(false),
        }
    }
}

/// The configuration structure loaded from global/local JSON files.
/// Includes a list of denied shell commands, custom Data Loss Prevention patterns, and savings profiles.
#[derive(Debug, Clone)]
pub struct UserConfig {
    /// Command regex patterns that should be denied/blocked.
    pub denied_commands: Vec<String>,
    /// Custom Data Loss Prevention regex patterns to scrub sensitive data.
    pub custom_dlp_patterns: Vec<String>,
    
    // P1 savings profiles
    pub output_profiles: std::collections::HashMap<String, ProfileSettings>,
    pub default_profile: String,
    pub overrides: std::collections::HashMap<String, String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        let mut output_profiles = std::collections::HashMap::new();
        output_profiles.insert(
            "strict".to_string(),
            ProfileSettings {
                max_line_length: Some(80),
                remove_comments: Some(true),
                minify_json: Some(true),
                json_only: Some(false),
            },
        );
        output_profiles.insert(
            "balanced".to_string(),
            ProfileSettings {
                max_line_length: Some(100),
                remove_comments: Some(true),
                minify_json: Some(false),
                json_only: Some(false),
            },
        );
        output_profiles.insert(
            "developer".to_string(),
            ProfileSettings {
                max_line_length: Some(120),
                remove_comments: Some(false),
                minify_json: Some(false),
                json_only: Some(false),
            },
        );
        output_profiles.insert(
            "audit".to_string(),
            ProfileSettings {
                max_line_length: None,
                remove_comments: Some(false),
                minify_json: Some(false),
                json_only: Some(false),
            },
        );
        output_profiles.insert(
            "json-only".to_string(),
            ProfileSettings {
                max_line_length: None,
                remove_comments: Some(false),
                minify_json: Some(false),
                json_only: Some(true),
            },
        );

        Self {
            denied_commands: vec![
                "git push.*--force".to_string(),
                "git reset --hard".to_string(),
            ],
            custom_dlp_patterns: Vec::new(),
            output_profiles,
            default_profile: "strict".to_string(),
            overrides: std::collections::HashMap::new(),
        }
    }
}

impl UserConfig {
    /// Load the configuration from global (`~/.config/rtk/config.json`) and local (`./.rtk.json`) paths.
    /// Merge settings prioritizing local definitions.
    pub fn load() -> Self {
        let mut config = UserConfig::default();

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

    /// Merge config keys from a JSON string representation.
    pub fn merge_from_str(&mut self, content: &str) -> Result<(), serde_json::Error> {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(arr) = val.get("denied_commands").and_then(|v| v.as_array()) {
                self.denied_commands.clear();
                self.denied_commands.extend(
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .filter(|s| regex::Regex::new(s).is_ok())
                        .map(String::from),
                );
            }
            if let Some(arr) = val
                .pointer("/dlp/custom_patterns")
                .and_then(|v| v.as_array())
            {
                self.custom_dlp_patterns.clear();
                self.custom_dlp_patterns.extend(
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .filter(|s| regex::Regex::new(s).is_ok())
                        .map(String::from),
                );
            }
            
            // P1 profiles
            if let Some(profiles_val) = val.get("output_profiles").and_then(|v| v.as_object()) {
                for (k, v_obj) in profiles_val {
                    if let Ok(settings) = serde_json::from_value::<ProfileSettings>(v_obj.clone()) {
                        self.output_profiles.insert(k.clone(), settings);
                    }
                }
            }
            
            if let Some(def) = val.get("default_profile").and_then(|v| v.as_str()) {
                self.default_profile = def.to_string();
            }
            
            if let Some(overrides_val) = val.get("overrides").and_then(|v| v.as_object()) {
                for (k, v_str) in overrides_val {
                    if let Some(profile_name) = v_str.as_str() {
                        self.overrides.insert(k.clone(), profile_name.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Retrieve the savings profile settings for a specific command using configured overrides or fallback to default.
    pub fn get_profile_for_cmd(&self, cmd: &str) -> ProfileSettings {
        for (pattern, profile_name) in &self.overrides {
            if cmd.contains(pattern) {
                if let Some(settings) = self.output_profiles.get(profile_name) {
                    return settings.clone();
                }
            }
        }
        
        if let Some(settings) = self.output_profiles.get(&self.default_profile) {
            settings.clone()
        } else {
            ProfileSettings::default()
        }
    }
}

/// Get the loaded UserConfig (global and local configuration merged).
pub fn get_config() -> UserConfig {
    UserConfig::load()
}

/// Create a default config.json file in `~/.config/rtk/` if it does not already exist.
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

/// Display the current active merged configuration as pretty-printed JSON.
pub fn config_show() -> anyhow::Result<()> {
    let config = get_config();
    let merged_json = serde_json::json!({
        "denied_commands": config.denied_commands,
        "dlp": {
            "custom_patterns": config.custom_dlp_patterns
        },
        "output_profiles": config.output_profiles,
        "default_profile": config.default_profile,
        "overrides": config.overrides
    });
    println!("{}", serde_json::to_string_pretty(&merged_json)?);
    Ok(())
}

/// Append a regex pattern to the list of denied commands in the global config.
pub fn config_deny_add(pattern: &str) -> anyhow::Result<()> {
    regex::Regex::new(pattern).map_err(|e| anyhow::anyhow!("invalid regex pattern: {e}"))?;
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

/// Append a custom Data Loss Prevention regex pattern to the global config.
pub fn config_dlp_add(pattern: &str) -> anyhow::Result<()> {
    regex::Regex::new(pattern).map_err(|e| anyhow::anyhow!("invalid regex pattern: {e}"))?;
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

/// Set the default active savings profile in global config.
pub fn config_profile_set(name: &str) -> anyhow::Result<()> {
    let valid_profiles = ["strict", "balanced", "developer", "audit", "json-only"];
    if !valid_profiles.contains(&name) {
        return Err(anyhow::anyhow!(
            "Invalid profile '{}'. Supported profiles: {}",
            name,
            valid_profiles.join(", ")
        ));
    }
    modify_config(|obj| {
        obj.insert("default_profile".to_string(), serde_json::Value::String(name.to_string()));
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

    #[test]
    fn test_profile_merging() {
        let mut config = UserConfig::default();
        let json = r#"{
            "default_profile": "developer",
            "overrides": {
                "git diff": "strict",
                "cargo test": "developer"
            },
            "output_profiles": {
                "custom": {
                    "max_line_length": 50,
                    "remove_comments": true,
                    "minify_json": true
                }
            }
        }"#;
        config.merge_from_str(json).unwrap();
        assert_eq!(config.default_profile, "developer");
        assert_eq!(config.overrides.get("git diff").unwrap(), "strict");
        assert_eq!(config.overrides.get("cargo test").unwrap(), "developer");
        
        let custom_profile = config.output_profiles.get("custom").unwrap();
        assert_eq!(custom_profile.max_line_length, Some(50));
        assert_eq!(custom_profile.remove_comments, Some(true));
        assert_eq!(custom_profile.minify_json, Some(true));

        // Test get_profile_for_cmd
        let p_git = config.get_profile_for_cmd("git diff --name-only");
        assert_eq!(p_git.max_line_length, Some(80)); // from default strict

        let p_cargo = config.get_profile_for_cmd("cargo test --all");
        assert_eq!(p_cargo.max_line_length, Some(120)); // from default developer

        let p_fallback = config.get_profile_for_cmd("npm run dev");
        assert_eq!(p_fallback.max_line_length, Some(120)); // fallbacks to default_profile which is developer
    }
}
