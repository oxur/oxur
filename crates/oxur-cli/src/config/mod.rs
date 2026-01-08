//! Configuration module for the Oxur REPL CLI
//!
//! Provides a layered configuration system with support for:
//! - Default values
//! - TOML config file (~/.config/oxur/repl.toml)
//! - Environment variables (OXUR_REPL_*)
//! - CLI argument overrides
//!
//! # Example
//!
//! ```no_run
//! use oxur_cli::config::{ReplConfig, TerminalConfig};
//!
//! // Load with all layers
//! let config = ReplConfig::load(false).unwrap();
//!
//! // Or build programmatically
//! let config = TerminalConfig::builder()
//!     .prompt("λ> ")
//!     .color(true)
//!     .build();
//! ```

mod history;
mod loader;
pub mod paths;
mod terminal;

pub use history::{HistoryConfig, HistoryConfigBuilder};
pub use loader::{load_config, ConfigLoader};
pub use terminal::{EditMode, TerminalConfig, TerminalConfigBuilder};

use serde::{Deserialize, Serialize};

/// Top-level REPL configuration
///
/// Contains all configuration sections for the REPL.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ReplConfig {
    /// Terminal appearance and behavior
    pub terminal: TerminalConfig,
    /// Command history settings
    pub history: HistoryConfig,
}

impl ReplConfig {
    /// Load configuration with all layers applied
    ///
    /// Loads configuration in order of precedence:
    /// 1. Defaults (lowest)
    /// 2. Config file (if exists)
    /// 3. Environment variables
    /// 4. CLI arguments (highest)
    ///
    /// # Arguments
    ///
    /// * `no_color` - CLI flag to disable colors
    ///
    /// # Errors
    ///
    /// Returns error if config file exists but cannot be read or parsed.
    pub fn load(no_color: bool) -> anyhow::Result<Self> {
        load_config(no_color)
    }

    /// Create a new builder
    pub fn builder() -> ReplConfigBuilder {
        ReplConfigBuilder::new()
    }

    /// Merge another config into this one
    ///
    /// Values from `other` override values in `self`.
    pub fn merge(&mut self, other: ReplConfig) {
        self.terminal.merge(other.terminal);
        self.history.merge(other.history);
    }
}

/// Builder for ReplConfig
#[derive(Debug, Clone, Default)]
pub struct ReplConfigBuilder {
    config: ReplConfig,
}

impl ReplConfigBuilder {
    /// Create a new builder with defaults
    pub fn new() -> Self {
        Self { config: ReplConfig::default() }
    }

    /// Set terminal configuration
    pub fn terminal(mut self, config: TerminalConfig) -> Self {
        self.config.terminal = config;
        self
    }

    /// Set history configuration
    pub fn history(mut self, config: HistoryConfig) -> Self {
        self.config.history = config;
        self
    }

    /// Build the ReplConfig
    pub fn build(self) -> ReplConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_config_default() {
        let config = ReplConfig::default();
        assert_eq!(config.terminal.prompt, "oxur> ");
        assert!(config.history.enabled);
    }

    #[test]
    fn test_repl_config_builder() {
        let terminal = TerminalConfig::builder().prompt(">>> ").build();

        let history = HistoryConfig::builder().max_size(500).build();

        let config = ReplConfig::builder().terminal(terminal).history(history).build();

        assert_eq!(config.terminal.prompt, ">>> ");
        assert_eq!(config.history.max_size, Some(500));
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = ReplConfig::builder()
            .terminal(TerminalConfig::builder().prompt("test> ").build())
            .build();

        let toml = toml::to_string(&config).unwrap();
        let parsed: ReplConfig = toml::from_str(&toml).unwrap();

        assert_eq!(config.terminal.prompt, parsed.terminal.prompt);
    }

    #[test]
    fn test_merge() {
        let mut base = ReplConfig::default();
        let other = ReplConfig::builder()
            .terminal(TerminalConfig::builder().prompt("merged> ").build())
            .build();

        base.merge(other);
        assert_eq!(base.terminal.prompt, "merged> ");
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
[terminal]
prompt = "λ> "
continuation_prompt = "  | "
color_enabled = true
edit_mode = "vi"

[history]
enabled = true
max_size = 5000
"#;

        let config: ReplConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.terminal.prompt, "λ> ");
        assert_eq!(config.terminal.continuation_prompt, "  | ");
        assert_eq!(config.terminal.edit_mode, EditMode::Vi);
        assert_eq!(config.history.max_size, Some(5000));
    }
}
