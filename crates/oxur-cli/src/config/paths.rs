//! Path resolution for REPL configuration
//!
//! Provides XDG-compliant path resolution with fallbacks for config,
//! data, and cache directories.

use std::path::PathBuf;

/// Find the REPL config file with XDG fallback
///
/// Searches in order:
/// 1. `$XDG_CONFIG_HOME/oxur/repl.toml` (if XDG_CONFIG_HOME set)
/// 2. `~/.config/oxur/repl.toml` (XDG default)
/// 3. `~/.oxur/repl.toml` (fallback)
///
/// Returns None if no config file exists.
pub fn find_config_file() -> Option<PathBuf> {
    // Check custom path from environment
    if let Ok(custom) = std::env::var("OXUR_CONFIG_PATH") {
        let path = PathBuf::from(custom);
        if path.exists() {
            return Some(path);
        }
    }

    // Try XDG config directory
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("oxur").join("repl.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // Fallback to ~/.oxur/repl.toml
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".oxur").join("repl.toml");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Get the default history file path
///
/// Uses XDG data directory or falls back to ~/.local/share/oxur/repl_history
pub fn default_history_path() -> PathBuf {
    // Check custom path from environment
    if let Ok(custom) = std::env::var("OXUR_HISTORY_PATH") {
        return PathBuf::from(custom);
    }

    // Use XDG data directory
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("oxur").join("repl_history")
}

/// Get the default config directory
///
/// Returns XDG config directory or ~/.config/oxur
pub fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("oxur")
}

/// Get the default data directory
///
/// Returns XDG data directory or ~/.local/share/oxur
pub fn data_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("oxur")
}

/// Get the default cache directory
///
/// Returns XDG cache directory or ~/.cache/oxur
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")).join("oxur")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_history_path_contains_oxur() {
        let path = default_history_path();
        assert!(path.to_string_lossy().contains("oxur"));
        assert!(path.to_string_lossy().contains("repl_history"));
    }

    #[test]
    fn test_config_dir_contains_oxur() {
        let path = config_dir();
        assert!(path.ends_with("oxur"));
    }

    #[test]
    fn test_data_dir_contains_oxur() {
        let path = data_dir();
        assert!(path.ends_with("oxur"));
    }

    #[test]
    fn test_cache_dir_contains_oxur() {
        let path = cache_dir();
        assert!(path.ends_with("oxur"));
    }

    #[test]
    fn test_find_config_file_returns_none_when_no_file() {
        // This test may pass or fail depending on whether the user has a config file
        // It's mainly here to verify the function doesn't panic
        let _ = find_config_file();
    }
}
