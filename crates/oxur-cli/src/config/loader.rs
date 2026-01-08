//! Configuration loader with layered resolution
//!
//! Loads configuration from multiple sources with precedence:
//! 1. Defaults (lowest)
//! 2. Config file
//! 3. Environment variables
//! 4. CLI arguments (highest)

use super::{paths, ReplConfig};
use anyhow::{Context, Result};

/// Configuration loader with builder-style API
pub struct ConfigLoader {
    config: ReplConfig,
}

impl ConfigLoader {
    /// Create a new config loader with defaults
    pub fn new() -> Self {
        Self { config: ReplConfig::default() }
    }

    /// Load configuration from file if it exists
    ///
    /// Uses XDG paths with fallback (see [`paths::find_config_file`])
    pub fn with_file(mut self) -> Result<Self> {
        if let Some(path) = paths::find_config_file() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;

            let file_config: ReplConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

            self.config.merge(file_config);
        }
        Ok(self)
    }

    /// Apply environment variable overrides
    ///
    /// Supported variables:
    /// - `OXUR_REPL_PROMPT` - Primary prompt
    /// - `OXUR_REPL_CONTINUATION_PROMPT` - Continuation prompt
    /// - `OXUR_REPL_BANNER` - Custom banner
    /// - `OXUR_REPL_COLOR` - Enable/disable colors (true/false/1/0)
    /// - `OXUR_REPL_EDIT_MODE` - Editing mode (emacs/vi)
    /// - `OXUR_HISTORY_ENABLED` - Enable/disable history
    /// - `OXUR_HISTORY_MAX_SIZE` - Maximum history entries
    pub fn with_env(mut self) -> Self {
        // Terminal config from environment
        if let Ok(prompt) = std::env::var("OXUR_REPL_PROMPT") {
            self.config.terminal.prompt = prompt;
        }

        if let Ok(cont_prompt) = std::env::var("OXUR_REPL_CONTINUATION_PROMPT") {
            self.config.terminal.continuation_prompt = cont_prompt;
        }

        if let Ok(banner) = std::env::var("OXUR_REPL_BANNER") {
            self.config.terminal.banner = Some(banner);
        }

        if let Ok(val) = std::env::var("OXUR_REPL_COLOR") {
            self.config.terminal.color_enabled = parse_bool(&val);
        }

        if let Ok(mode) = std::env::var("OXUR_REPL_EDIT_MODE") {
            if let Some(edit_mode) = parse_edit_mode(&mode) {
                self.config.terminal.edit_mode = edit_mode;
            }
        }

        // History config from environment
        if let Ok(val) = std::env::var("OXUR_HISTORY_ENABLED") {
            self.config.history.enabled = parse_bool(&val);
        }

        if let Ok(val) = std::env::var("OXUR_HISTORY_MAX_SIZE") {
            if let Ok(size) = val.parse() {
                self.config.history.max_size = Some(size);
            }
        }

        self
    }

    /// Apply CLI argument overrides
    ///
    /// This is the final layer with highest precedence.
    pub fn with_cli_overrides(mut self, no_color: bool) -> Self {
        if no_color {
            self.config.terminal.color_enabled = false;
        }
        self
    }

    /// Build the final configuration
    pub fn build(self) -> ReplConfig {
        self.config
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a boolean from various string representations
fn parse_bool(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
}

/// Parse edit mode from string
fn parse_edit_mode(s: &str) -> Option<super::EditMode> {
    match s.to_lowercase().as_str() {
        "emacs" => Some(super::EditMode::Emacs),
        "vi" | "vim" => Some(super::EditMode::Vi),
        _ => None,
    }
}

/// Load complete configuration with all layers
///
/// Convenience function that applies all configuration layers:
/// 1. Defaults
/// 2. Config file (if exists)
/// 3. Environment variables
/// 4. CLI arguments
pub fn load_config(no_color: bool) -> Result<ReplConfig> {
    ConfigLoader::new().with_file()?.with_env().with_cli_overrides(no_color).build().pipe(Ok)
}

/// Extension trait for pipe operator
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bool_true() {
        assert!(parse_bool("true"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("on"));
    }

    #[test]
    fn test_parse_bool_false() {
        assert!(!parse_bool("false"));
        assert!(!parse_bool("FALSE"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("off"));
        assert!(!parse_bool(""));
        assert!(!parse_bool("invalid"));
    }

    #[test]
    fn test_parse_edit_mode() {
        assert_eq!(parse_edit_mode("emacs"), Some(super::super::EditMode::Emacs));
        assert_eq!(parse_edit_mode("EMACS"), Some(super::super::EditMode::Emacs));
        assert_eq!(parse_edit_mode("vi"), Some(super::super::EditMode::Vi));
        assert_eq!(parse_edit_mode("vim"), Some(super::super::EditMode::Vi));
        assert_eq!(parse_edit_mode("invalid"), None);
    }

    #[test]
    fn test_loader_defaults() {
        let config = ConfigLoader::new().build();
        assert_eq!(config.terminal.prompt, "oxur> ");
        assert!(config.terminal.color_enabled);
        assert!(config.history.enabled);
    }

    #[test]
    fn test_loader_with_cli_overrides() {
        let config = ConfigLoader::new().with_cli_overrides(true).build();
        assert!(!config.terminal.color_enabled);
    }

    #[test]
    fn test_loader_chain() {
        // Test that the builder chain works without errors
        let result = ConfigLoader::new()
            .with_file() // May or may not find a file
            .map(|l| l.with_env())
            .map(|l| l.with_cli_overrides(false))
            .map(|l| l.build());

        assert!(result.is_ok());
    }
}
