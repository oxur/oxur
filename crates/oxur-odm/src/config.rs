use anyhow::{Context, Result};
use confyg::searchpath::Finder;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

/// Application configuration with layered defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project root directory
    pub project_root: PathBuf,

    /// Documentation directory
    pub docs_directory: PathBuf,

    /// Development documents directory
    pub dev_directory: PathBuf,

    /// State file path
    pub state_file: PathBuf,

    /// Dustbin directory for removed documents
    pub dustbin_directory: PathBuf,

    /// Whether to preserve state directory structure in dustbin
    pub preserve_dustbin_structure: bool,

    /// Whether to automatically stage files with git
    pub auto_stage_git: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_root: PathBuf::from("."),
            docs_directory: PathBuf::from("./design/docs"),
            dev_directory: PathBuf::from("./docs/dev"),
            state_file: PathBuf::from("./design/docs/.odm/state.json"),
            dustbin_directory: PathBuf::from("./design/docs/.dustbin"),
            preserve_dustbin_structure: true,
            auto_stage_git: true,
        }
    }
}

impl Config {
    /// Load configuration with confyg search paths
    pub fn load(docs_dir: Option<&str>) -> Result<Self> {
        // Start with defaults
        let mut config = Config::default();

        // 1. Try to load from confyg search paths (odm.toml)
        if let Some(confyg_config) = Self::load_from_confyg()? {
            config.merge(confyg_config);
        }

        // 2. Try legacy .odmrc for backward compatibility (deprecated)
        if let Some(legacy_config) = Self::load_legacy_odmrc()? {
            oxur_cli::common::output::warning(".odmrc is deprecated, please migrate to odm.toml");
            config.merge(legacy_config);
        }

        // 3. Override docs_directory if CLI provided (highest priority)
        let cli_override = docs_dir.is_some();
        if let Some(dir) = docs_dir {
            let path = PathBuf::from(dir);
            config.docs_directory = path.clone();
            config.state_file = path.join(".odm/state.json");
            config.dustbin_directory = path.join(".dustbin");
        }

        // 4. Try to load from .odm/config.toml in docs directory
        // Note: if CLI provided docs_dir, we skip merging docs_directory from file
        if let Some(mut file_config) = Self::load_from_file(&config.docs_directory)? {
            if cli_override {
                // Don't let file config override CLI-provided docs_directory
                file_config.docs_directory = None;
            }
            config.merge(file_config);
        }

        Ok(config)
    }

    /// Load configuration using confyg's search path mechanism
    fn load_from_confyg() -> Result<Option<PartialConfig>> {
        // Build search paths
        let mut finder = Finder::new();

        // 1. Current directory (highest priority for local overrides)
        finder.add_path(".");

        // 2. Git repository root (if in a git repo)
        if let Some(repo_root) = crate::git::get_repo_root() {
            if let Some(path_str) = repo_root.to_str() {
                finder.add_path(path_str);
            }
        }

        // 3. User config directory (~/.config/odm/)
        if let Some(config_dir) = Self::get_user_config_dir() {
            if let Some(path_str) = config_dir.to_str() {
                finder.add_path(path_str);
            }
        }

        // 4. System config directory (optional)
        finder.add_path("/etc/odm");

        // Search for odm.toml
        match finder.find("odm.toml") {
            Ok(config_path) => {
                let contents = std::fs::read_to_string(&config_path)
                    .context(format!("Failed to read {:?}", config_path))?;

                let config: PartialConfig = toml::from_str(&contents)
                    .context(format!("Failed to parse {:?}", config_path))?;

                Ok(Some(config))
            }
            Err(_) => Ok(None), // No config file found, use defaults
        }
    }

    /// Get user config directory (~/.config/odm or platform equivalent)
    fn get_user_config_dir() -> Option<PathBuf> {
        // Use XDG_CONFIG_HOME if set, otherwise ~/.config
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            Some(PathBuf::from(xdg_config).join("odm"))
        } else if let Ok(home) = env::var("HOME") {
            Some(PathBuf::from(home).join(".config/odm"))
        } else {
            None
        }
    }

    /// Load legacy .odmrc from git root (backward compatibility)
    fn load_legacy_odmrc() -> Result<Option<PartialConfig>> {
        let Some(root) = crate::git::get_repo_root() else {
            return Ok(None);
        };

        let config_path = root.join(".odmrc");

        if !config_path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&config_path).context("Failed to read .odmrc")?;

        // .odmrc has a different format - just docs_dir
        #[derive(Debug, serde::Deserialize)]
        struct LegacyConfig {
            docs_dir: Option<String>,
        }

        let legacy: LegacyConfig = toml::from_str(&contents).context("Failed to parse .odmrc")?;

        Ok(legacy.docs_dir.map(|dir| PartialConfig {
            project_root: None,
            docs_directory: Some(PathBuf::from(dir)),
            dev_directory: None,
            dustbin_directory: None,
            preserve_dustbin_structure: None,
            auto_stage_git: None,
        }))
    }

    /// Load configuration from .odm/config.toml
    fn load_from_file(docs_dir: &Path) -> Result<Option<PartialConfig>> {
        let config_path = docs_dir.join(".odm/config.toml");
        if !config_path.exists() {
            return Ok(None);
        }

        let contents =
            std::fs::read_to_string(&config_path).context("Failed to read .odm/config.toml")?;

        let config: PartialConfig =
            toml::from_str(&contents).context("Failed to parse .odm/config.toml")?;

        Ok(Some(config))
    }

    /// Merge partial config into this one (partial takes precedence for specified fields)
    fn merge(&mut self, other: PartialConfig) {
        if let Some(val) = other.project_root {
            self.project_root = val;
        }
        if let Some(val) = other.docs_directory {
            self.docs_directory = val.clone();
            self.state_file = val.join(".odm/state.json");
            self.dustbin_directory = val.join(".dustbin");
        }
        if let Some(val) = other.dev_directory {
            self.dev_directory = val;
        }
        if let Some(val) = other.dustbin_directory {
            self.dustbin_directory = val;
        }
        if let Some(val) = other.preserve_dustbin_structure {
            self.preserve_dustbin_structure = val;
        }
        if let Some(val) = other.auto_stage_git {
            self.auto_stage_git = val;
        }
    }

    /// Get the dustbin directory for a specific state
    pub fn dustbin_dir_for_state(&self, state_dir: &str) -> PathBuf {
        if self.preserve_dustbin_structure {
            self.dustbin_directory.join(state_dir)
        } else {
            self.dustbin_directory.clone()
        }
    }
}

/// Partial configuration for deserializing from TOML with optional fields
#[derive(Debug, Deserialize)]
struct PartialConfig {
    project_root: Option<PathBuf>,
    docs_directory: Option<PathBuf>,
    dev_directory: Option<PathBuf>,
    dustbin_directory: Option<PathBuf>,
    preserve_dustbin_structure: Option<bool>,
    auto_stage_git: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.docs_directory, PathBuf::from("./design/docs"));
        assert_eq!(config.dev_directory, PathBuf::from("./docs/dev"));
        assert_eq!(config.state_file, PathBuf::from("./design/docs/.odm/state.json"));
        assert!(config.preserve_dustbin_structure);
        assert!(config.auto_stage_git);
    }

    #[test]
    fn test_load_with_docs_dir() {
        let config = Config::load(Some("/custom/docs")).unwrap();
        assert_eq!(config.docs_directory, PathBuf::from("/custom/docs"));
        assert_eq!(config.state_file, PathBuf::from("/custom/docs/.odm/state.json"));
        assert_eq!(config.dustbin_directory, PathBuf::from("/custom/docs/.dustbin"));
    }

    #[test]
    fn test_load_from_file() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();

        // Create .odm directory and config file
        fs::create_dir_all(docs_dir.join(".odm")).unwrap();
        fs::write(
            docs_dir.join(".odm/config.toml"),
            r#"
preserve_dustbin_structure = false
auto_stage_git = false
"#,
        )
        .unwrap();

        let config = Config::load(Some(docs_dir.to_str().unwrap())).unwrap();
        assert!(!config.preserve_dustbin_structure);
        assert!(!config.auto_stage_git);
    }

    #[test]
    fn test_dustbin_dir_for_state_preserved() {
        let config = Config {
            dustbin_directory: PathBuf::from("/dustbin"),
            preserve_dustbin_structure: true,
            ..Default::default()
        };

        let result = config.dustbin_dir_for_state("01-draft");
        assert_eq!(result, PathBuf::from("/dustbin/01-draft"));
    }

    #[test]
    fn test_dustbin_dir_for_state_flat() {
        let config = Config {
            dustbin_directory: PathBuf::from("/dustbin"),
            preserve_dustbin_structure: false,
            ..Default::default()
        };

        let result = config.dustbin_dir_for_state("01-draft");
        assert_eq!(result, PathBuf::from("/dustbin"));
    }

    #[test]
    fn test_partial_config_with_docs_directory() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();

        fs::create_dir_all(docs_dir.join(".odm")).unwrap();
        fs::write(
            docs_dir.join(".odm/config.toml"),
            r#"
docs_directory = "/custom/docs"
preserve_dustbin_structure = false
"#,
        )
        .unwrap();

        let config = Config::load(Some(docs_dir.to_str().unwrap())).unwrap();
        // docs_directory from .odm/config.toml should be merged
        // but then CLI override wins
        assert_eq!(config.docs_directory.to_str().unwrap(), docs_dir.to_str().unwrap());
        assert!(!config.preserve_dustbin_structure);
    }

    #[test]
    fn test_cli_override_beats_all() {
        // Verify CLI --docs-dir overrides everything
        let config = Config::load(Some("/cli/override")).unwrap();
        assert_eq!(config.docs_directory, PathBuf::from("/cli/override"));
        assert_eq!(config.state_file, PathBuf::from("/cli/override/.odm/state.json"));
        assert_eq!(config.dustbin_directory, PathBuf::from("/cli/override/.dustbin"));
    }

    #[test]
    fn test_get_user_config_dir_with_xdg() {
        // Save original env vars
        let original_xdg = env::var("XDG_CONFIG_HOME").ok();

        // Test with XDG_CONFIG_HOME set
        env::set_var("XDG_CONFIG_HOME", "/test/xdg");
        let result = Config::get_user_config_dir();
        assert_eq!(result, Some(PathBuf::from("/test/xdg/odm")));

        // Restore original
        match original_xdg {
            Some(val) => env::set_var("XDG_CONFIG_HOME", val),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    #[test]
    fn test_get_user_config_dir_with_home() {
        // Save original env vars
        let original_xdg = env::var("XDG_CONFIG_HOME").ok();
        let original_home = env::var("HOME").ok();

        // Test with HOME but no XDG_CONFIG_HOME
        env::remove_var("XDG_CONFIG_HOME");
        env::set_var("HOME", "/test/home");
        let result = Config::get_user_config_dir();
        assert_eq!(result, Some(PathBuf::from("/test/home/.config/odm")));

        // Restore original
        match original_xdg {
            Some(val) => env::set_var("XDG_CONFIG_HOME", val),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
        match original_home {
            Some(val) => env::set_var("HOME", val),
            None => env::remove_var("HOME"),
        }
    }

    #[test]
    fn test_merge_docs_directory_updates_related_paths() {
        let mut config = Config::default();

        let partial = PartialConfig {
            project_root: None,
            docs_directory: Some(PathBuf::from("/new/docs")),
            dev_directory: None,
            dustbin_directory: None,
            preserve_dustbin_structure: None,
            auto_stage_git: None,
        };

        config.merge(partial);

        // Verify docs_directory and related paths are updated
        assert_eq!(config.docs_directory, PathBuf::from("/new/docs"));
        assert_eq!(config.state_file, PathBuf::from("/new/docs/.odm/state.json"));
        assert_eq!(config.dustbin_directory, PathBuf::from("/new/docs/.dustbin"));
    }

    #[test]
    fn test_load_from_file_nonexistent() {
        let temp = TempDir::new().unwrap();
        let result = Config::load_from_file(temp.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_dev_directory_from_config() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();

        fs::create_dir_all(docs_dir.join(".odm")).unwrap();
        fs::write(
            docs_dir.join(".odm/config.toml"),
            r#"
dev_directory = "/custom/dev"
"#,
        )
        .unwrap();

        let config = Config::load(Some(docs_dir.to_str().unwrap())).unwrap();
        assert_eq!(config.dev_directory, PathBuf::from("/custom/dev"));
    }

    #[test]
    fn test_merge_dev_directory() {
        let mut config = Config::default();

        let partial = PartialConfig {
            project_root: None,
            docs_directory: None,
            dev_directory: Some(PathBuf::from("/new/dev")),
            dustbin_directory: None,
            preserve_dustbin_structure: None,
            auto_stage_git: None,
        };

        config.merge(partial);

        assert_eq!(config.dev_directory, PathBuf::from("/new/dev"));
    }
}
