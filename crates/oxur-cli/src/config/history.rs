//! History configuration for the REPL
//!
//! Provides configuration for command history including storage path,
//! maximum size, and enabling/disabling history.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// History configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryConfig {
    /// Whether command history is enabled
    pub enabled: bool,
    /// Maximum number of history entries to keep
    pub max_size: Option<usize>,
    /// Custom path for history file (None uses XDG default)
    pub path: Option<PathBuf>,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self { enabled: true, max_size: Some(10000), path: None }
    }
}

impl HistoryConfig {
    /// Create a new builder for HistoryConfig
    pub fn builder() -> HistoryConfigBuilder {
        HistoryConfigBuilder::new()
    }

    /// Merge another config into this one
    pub fn merge(&mut self, other: HistoryConfig) {
        self.enabled = other.enabled;
        if other.max_size.is_some() {
            self.max_size = other.max_size;
        }
        if other.path.is_some() {
            self.path = other.path;
        }
    }
}

/// Builder for HistoryConfig
#[derive(Debug, Clone)]
pub struct HistoryConfigBuilder {
    config: HistoryConfig,
}

impl HistoryConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self { config: HistoryConfig::default() }
    }

    /// Enable or disable history
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set maximum history size
    pub fn max_size(mut self, size: usize) -> Self {
        self.config.max_size = Some(size);
        self
    }

    /// Set custom history file path
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.path = Some(path.into());
        self
    }

    /// Build the HistoryConfig
    pub fn build(self) -> HistoryConfig {
        self.config
    }
}

impl Default for HistoryConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HistoryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_size, Some(10000));
        assert!(config.path.is_none());
    }

    #[test]
    fn test_builder() {
        let config =
            HistoryConfig::builder().enabled(false).max_size(5000).path("/custom/path").build();

        assert!(!config.enabled);
        assert_eq!(config.max_size, Some(5000));
        assert_eq!(config.path, Some(PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = HistoryConfig::builder().max_size(1000).path("/test/history").build();

        let toml = toml::to_string(&config).unwrap();
        let parsed: HistoryConfig = toml::from_str(&toml).unwrap();

        assert_eq!(config.enabled, parsed.enabled);
        assert_eq!(config.max_size, parsed.max_size);
        assert_eq!(config.path, parsed.path);
    }

    #[test]
    fn test_merge_preserves_none_values() {
        let mut base = HistoryConfig::builder().max_size(5000).path("/base/path").build();

        let other = HistoryConfig { enabled: false, max_size: None, path: None };

        base.merge(other);

        assert!(!base.enabled);
        assert_eq!(base.max_size, Some(5000)); // Preserved
        assert_eq!(base.path, Some(PathBuf::from("/base/path"))); // Preserved
    }

    #[test]
    fn test_merge_overrides_some_values() {
        let mut base = HistoryConfig::builder().max_size(5000).build();

        let other = HistoryConfig::builder().max_size(1000).path("/new/path").build();

        base.merge(other);

        assert_eq!(base.max_size, Some(1000));
        assert_eq!(base.path, Some(PathBuf::from("/new/path")));
    }
}
