use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration with layered defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project root directory
    pub project_root: PathBuf,

    /// Documentation directory
    pub docs_directory: PathBuf,

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
            state_file: PathBuf::from("./design/docs/.oxd/state.json"),
            dustbin_directory: PathBuf::from("./design/docs/.dustbin"),
            preserve_dustbin_structure: true,
            auto_stage_git: true,
        }
    }
}

impl Config {
    /// Load configuration from all sources with proper precedence
    pub fn load(docs_dir: Option<&str>) -> Result<Self> {
        // Start with defaults
        let mut config = Config::default();

        // Override docs_directory if provided
        if let Some(dir) = docs_dir {
            let path = PathBuf::from(dir);
            config.docs_directory = path.clone();
            config.state_file = path.join(".oxd/state.json");
            config.dustbin_directory = path.join(".dustbin");
        }

        // Try to load from .oxd/config.toml (takes precedence)
        if let Some(file_config) = Self::load_from_file(&config.docs_directory)? {
            config.merge(file_config);
        }

        Ok(config)
    }

    /// Load configuration from .oxd/config.toml
    fn load_from_file(docs_dir: &PathBuf) -> Result<Option<PartialConfig>> {
        let config_path = docs_dir.join(".oxd/config.toml");
        if !config_path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&config_path)
            .context("Failed to read .oxd/config.toml")?;

        let config: PartialConfig = toml::from_str(&contents)
            .context("Failed to parse .oxd/config.toml")?;

        Ok(Some(config))
    }

    /// Merge partial config into this one (partial takes precedence for specified fields)
    fn merge(&mut self, other: PartialConfig) {
        if let Some(val) = other.project_root {
            self.project_root = val;
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
        assert_eq!(config.state_file, PathBuf::from("./design/docs/.oxd/state.json"));
        assert!(config.preserve_dustbin_structure);
        assert!(config.auto_stage_git);
    }

    #[test]
    fn test_load_with_docs_dir() {
        let config = Config::load(Some("/custom/docs")).unwrap();
        assert_eq!(config.docs_directory, PathBuf::from("/custom/docs"));
        assert_eq!(config.state_file, PathBuf::from("/custom/docs/.oxd/state.json"));
        assert_eq!(config.dustbin_directory, PathBuf::from("/custom/docs/.dustbin"));
    }

    #[test]
    fn test_load_from_file() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();

        // Create .oxd directory and config file
        fs::create_dir_all(docs_dir.join(".oxd")).unwrap();
        fs::write(
            docs_dir.join(".oxd/config.toml"),
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
}
