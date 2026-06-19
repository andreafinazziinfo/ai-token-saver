use serde::Deserialize;
use std::path::Path;

/// Represents a declarative command plugin definition from `plugins.toml`.
#[derive(Debug, Deserialize, Clone)]
pub struct Plugin {
    /// Unique name of the plugin.
    pub name: String,
    /// Executable binary name (e.g. "ruff").
    pub bin: String,
    /// Optional subcommand argument prefix to match (e.g. ["check"]).
    pub args: Option<Vec<String>>,
    /// Output capture mode: "stdout", "stderr", "combined", or "distill".
    pub filter_mode: Option<String>,
    /// List of line prefixes that should be dropped.
    pub drop_prefixes: Option<Vec<String>>,
    /// List of line prefixes that must be kept (along with standard error/warning lines).
    pub keep_prefixes: Option<Vec<String>>,
}

/// Dynamic list of plugins loaded from TOML configuration.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct PluginsConfig {
    /// List of registered plugins.
    pub plugins: Vec<Plugin>,
}

/// Load and merge plugins from global (`~/.config/rtk/plugins.toml`) and local (`./plugins.toml`) locations.
pub fn load_plugins() -> PluginsConfig {
    let mut config = PluginsConfig::default();

    // 1. Try to load from user global home folder: ~/.config/rtk/plugins.toml
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let global_path = Path::new(&home).join(".config/rtk/plugins.toml");
        if let Ok(content) = std::fs::read_to_string(&global_path) {
            if let Ok(c) = toml::from_str::<PluginsConfig>(&content) {
                config.plugins.extend(c.plugins);
            }
        }
    }

    // 2. Try to load from local directory: ./plugins.toml
    let local_path = Path::new("plugins.toml");
    if let Ok(content) = std::fs::read_to_string(local_path) {
        if let Ok(c) = toml::from_str::<PluginsConfig>(&content) {
            // Local plugins override or append to global plugins
            for p in c.plugins {
                if let Some(pos) = config.plugins.iter().position(|x| x.name == p.name) {
                    config.plugins[pos] = p; // Override
                } else {
                    config.plugins.push(p); // Append
                }
            }
        }
    }

    config
}

/// Apply declarative line-based filtering rules to raw command output.
pub fn filter_plugin(input: &str, plugin: &Plugin) -> String {
    let mut out = String::with_capacity(input.len());
    for line in input.lines() {
        let mut keep = true;

        if let Some(ref drop) = plugin.drop_prefixes {
            if drop
                .iter()
                .any(|prefix| line.trim_start().starts_with(prefix))
            {
                keep = false;
            }
        }

        if let Some(ref k_pref) = plugin.keep_prefixes {
            let matches_keep = k_pref
                .iter()
                .any(|prefix| line.trim_start().starts_with(prefix));
            let has_error = line.to_lowercase().contains("error")
                || line.to_lowercase().contains("warning")
                || line.to_lowercase().contains("failed");
            if !matches_keep && !has_error {
                keep = false;
            }
        }

        if keep {
            out.push_str(line);
            out.push('\n');
        }
    }

    if out.trim().is_empty() {
        input.to_string()
    } else {
        out
    }
}
