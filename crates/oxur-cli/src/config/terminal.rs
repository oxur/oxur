//! Terminal configuration for the REPL
//!
//! Provides configuration for terminal appearance including prompts,
//! colors, banners, and editing mode.

use serde::{Deserialize, Serialize};

/// Default ASCII art banner for the REPL
///
/// Embedded from oxur-repl/assets/banners/banner0.1.0.txt at compile time.
/// Contains ANSI color codes and ASCII art.
///
/// Users can override this banner via:
/// - Config file: `[terminal]` section, `banner` field
/// - Environment variable: `OXUR_REPL_BANNER`
const DEFAULT_BANNER: &str = include_str!("../../../oxur-repl/assets/banners/banner0.1.0.txt");

/// Terminal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    /// Custom login banner (None uses default)
    pub banner: Option<String>,
    /// Primary prompt string
    pub prompt: String,
    /// Continuation prompt for multi-line input
    pub continuation_prompt: String,
    /// Whether ANSI colors are enabled
    pub color_enabled: bool,
    /// Line editing mode
    pub edit_mode: EditMode,
}

/// Line editing mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EditMode {
    /// Emacs-style keybindings (default)
    #[default]
    Emacs,
    /// Vi-style keybindings
    Vi,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            banner: Some(DEFAULT_BANNER.to_string()),
            prompt: "oxur> ".to_string(),
            continuation_prompt: "....> ".to_string(),
            color_enabled: true,
            edit_mode: EditMode::Emacs,
        }
    }
}

impl TerminalConfig {
    /// Create a new builder for TerminalConfig
    pub fn builder() -> TerminalConfigBuilder {
        TerminalConfigBuilder::new()
    }

    /// Get the prompt with optional color formatting
    ///
    /// When colors are enabled and the prompt starts with "oxur",
    /// "oxur" is colored orange and the rest (e.g., "> ") is bright green.
    pub fn formatted_prompt(&self) -> String {
        if self.color_enabled {
            // Check if prompt starts with "oxur" for special coloring
            if self.prompt.starts_with("oxur") {
                let rest = &self.prompt[4..]; // Everything after "oxur"
                // Orange for "oxur", bright green for "> "
                format!("\x1b[33moxur\x1b[0m\x1b[92m{}\x1b[0m", rest)
            } else {
                format!("\x1b[32m{}\x1b[0m", self.prompt)
            }
        } else {
            self.prompt.clone()
        }
    }

    /// Get the continuation prompt with optional color formatting
    pub fn formatted_continuation_prompt(&self) -> String {
        if self.color_enabled {
            format!("\x1b[32m{}\x1b[0m", self.continuation_prompt)
        } else {
            self.continuation_prompt.clone()
        }
    }

    /// Merge another config into this one (other takes precedence for Some values)
    pub fn merge(&mut self, other: TerminalConfig) {
        if other.banner.is_some() {
            self.banner = other.banner;
        }
        // For non-Option fields, we need partial config pattern
        // For now, other always overrides if explicitly set in file
        self.prompt = other.prompt;
        self.continuation_prompt = other.continuation_prompt;
        self.color_enabled = other.color_enabled;
        self.edit_mode = other.edit_mode;
    }
}

/// Builder for TerminalConfig
#[derive(Debug, Clone)]
pub struct TerminalConfigBuilder {
    config: TerminalConfig,
}

impl TerminalConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self { config: TerminalConfig::default() }
    }

    /// Set a custom login banner
    pub fn banner(mut self, banner: impl Into<String>) -> Self {
        self.config.banner = Some(banner.into());
        self
    }

    /// Set the primary prompt
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.prompt = prompt.into();
        self
    }

    /// Set the continuation prompt for multi-line input
    pub fn continuation_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.continuation_prompt = prompt.into();
        self
    }

    /// Enable or disable ANSI colors
    pub fn color(mut self, enabled: bool) -> Self {
        self.config.color_enabled = enabled;
        self
    }

    /// Set the line editing mode
    pub fn edit_mode(mut self, mode: EditMode) -> Self {
        self.config.edit_mode = mode;
        self
    }

    /// Build the TerminalConfig
    pub fn build(self) -> TerminalConfig {
        self.config
    }
}

impl Default for TerminalConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TerminalConfig::default();
        assert_eq!(config.prompt, "oxur> ");
        assert_eq!(config.continuation_prompt, "....> ");
        assert!(config.color_enabled);
        assert_eq!(config.edit_mode, EditMode::Emacs);
        assert!(config.banner.is_some());
    }

    #[test]
    fn test_builder() {
        let config = TerminalConfig::builder()
            .banner("Welcome!")
            .prompt("λ> ")
            .continuation_prompt("  | ")
            .color(false)
            .edit_mode(EditMode::Vi)
            .build();

        assert_eq!(config.banner, Some("Welcome!".to_string()));
        assert_eq!(config.prompt, "λ> ");
        assert_eq!(config.continuation_prompt, "  | ");
        assert!(!config.color_enabled);
        assert_eq!(config.edit_mode, EditMode::Vi);
    }

    #[test]
    fn test_formatted_prompt_with_color() {
        // Non-oxur prompt uses standard green
        let config = TerminalConfig::builder().prompt("test> ").color(true).build();
        assert!(config.formatted_prompt().contains("\x1b[32m"));
        assert!(config.formatted_prompt().contains("test> "));
    }

    #[test]
    fn test_formatted_prompt_oxur_colors() {
        // oxur prompt uses orange for "oxur" and bright green for "> "
        let config = TerminalConfig::builder().prompt("oxur> ").color(true).build();
        let prompt = config.formatted_prompt();
        assert!(prompt.contains("\x1b[33m")); // Orange
        assert!(prompt.contains("\x1b[92m")); // Bright green
        assert!(prompt.contains("oxur"));
        assert!(prompt.contains("> "));
    }

    #[test]
    fn test_formatted_prompt_without_color() {
        let config = TerminalConfig::builder().prompt("test> ").color(false).build();
        assert_eq!(config.formatted_prompt(), "test> ");
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = TerminalConfig::builder()
            .banner("Test Banner")
            .prompt(">>> ")
            .edit_mode(EditMode::Vi)
            .build();

        let toml = toml::to_string(&config).unwrap();
        let parsed: TerminalConfig = toml::from_str(&toml).unwrap();

        assert_eq!(config.banner, parsed.banner);
        assert_eq!(config.prompt, parsed.prompt);
        assert_eq!(config.edit_mode, parsed.edit_mode);
    }

    #[test]
    fn test_edit_mode_serde() {
        // Test via a wrapper struct since TOML requires key-value pairs
        #[derive(Debug, serde::Deserialize)]
        struct Wrapper {
            mode: EditMode,
        }

        let emacs: Wrapper = toml::from_str("mode = \"emacs\"").unwrap();
        let vi: Wrapper = toml::from_str("mode = \"vi\"").unwrap();

        assert_eq!(emacs.mode, EditMode::Emacs);
        assert_eq!(vi.mode, EditMode::Vi);
    }

    #[test]
    fn test_default_banner_embedded() {
        let config = TerminalConfig::default();
        let banner = config.banner.expect("Default config should have banner");

        // Verify banner contains expected elements
        assert!(banner.contains("Welcome to"));
        assert!(banner.contains("oxur:"));
        assert!(banner.contains("http://oxur.li/"));
        assert!(banner.contains("(help)"));
        assert!(banner.contains("(quit)"));

        // Verify banner is non-empty and reasonable size
        assert!(banner.len() > 1000);
        assert!(banner.len() < 10000);
    }
}
