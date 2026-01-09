//! Terminal interface for REPL interaction
//!
//! Provides line editing, command history, and terminal handling
//! using reedline.

use anyhow::{Context, Result};
use oxur_cli::config::{paths, EditMode, HistoryConfig, TerminalConfig};
use reedline::{
    default_emacs_keybindings, default_vi_insert_keybindings, default_vi_normal_keybindings,
    ColumnarMenu, Emacs, FileBackedHistory, KeyCode, KeyModifiers, Keybindings, MenuBuilder,
    Reedline, ReedlineEvent, ReedlineMenu, Signal, Vi,
};
use std::path::PathBuf;

use crate::repl::completer::OxurCompleter;
use crate::repl::oxur_prompt::OxurPrompt;
use crate::repl::pager;
use crate::repl::sexp_highlighter::SExpHighlighter;
use crate::repl::sexp_validator::SExpValidator;

/// Extract version info from version string (without tool name)
///
/// Converts "rustc 1.75.0 (hash)" to "1.75.0 (hash)", or "cargo 1.75.0 (date)" to "1.75.0 (date)"
/// Keeps the full version string but removes the tool name prefix.
fn format_version(version_str: &str) -> String {
    // Split by whitespace and skip the first element (tool name)
    // e.g., "rustc 1.75.0 (82e1608df 2023-12-21)" -> "1.75.0 (82e1608df 2023-12-21)"
    let parts: Vec<&str> = version_str.split_whitespace().collect();
    if parts.len() > 1 {
        parts[1..].join(" ")
    } else {
        version_str.to_string()
    }
}

/// Substitute version placeholders in banner text
///
/// Replaces template placeholders with actual version information:
/// - `N.N.N` → Oxur version (e.g., "0.1.0")
/// - `M.M.M` → Rust version info (e.g., "1.75.0 (82e1608df 2023-12-21)")
/// - `L.L.L` → Cargo version info (e.g., "1.75.0 (1d8b05cdd 2023-11-20)")
fn substitute_banner_versions(
    banner: &str,
    metadata: &oxur_repl::metadata::SystemMetadata,
) -> String {
    banner
        .replace("N.N.N", &metadata.oxur_version)
        .replace("M.M.M", &format_version(&metadata.rust_version))
        .replace("L.L.L", &format_version(&metadata.cargo_version))
}

/// Add Tab keybinding for completion menu
fn add_completion_keybinding(keybindings: &mut Keybindings) {
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
}

/// REPL terminal interface with line editing and history
pub struct ReplTerminal {
    editor: Reedline,
    #[allow(dead_code)] // Kept for API compatibility and future use
    history_path: PathBuf,
    terminal_config: TerminalConfig,
}

impl ReplTerminal {
    /// Create a new REPL terminal with configuration
    ///
    /// # Arguments
    ///
    /// * `terminal_config` - Terminal appearance configuration
    /// * `history_config` - Command history configuration
    ///
    /// # Errors
    ///
    /// Returns error if reedline initialization fails.
    pub fn with_config(
        terminal_config: TerminalConfig,
        history_config: HistoryConfig,
    ) -> Result<Self> {
        // Convert edit mode to reedline's EditMode trait object with Tab completion
        let edit_mode: Box<dyn reedline::EditMode> = match terminal_config.edit_mode {
            EditMode::Emacs => {
                let mut keybindings = default_emacs_keybindings();
                add_completion_keybinding(&mut keybindings);
                Box::new(Emacs::new(keybindings))
            }
            EditMode::Vi => {
                let mut insert_keybindings = default_vi_insert_keybindings();
                let mut normal_keybindings = default_vi_normal_keybindings();
                add_completion_keybinding(&mut insert_keybindings);
                add_completion_keybinding(&mut normal_keybindings);
                Box::new(Vi::new(insert_keybindings, normal_keybindings))
            }
        };

        // Determine history file path
        let history_path = history_config.path.unwrap_or_else(paths::default_history_path);

        // Create history backend
        // Note: FileBackedHistory is used for both enabled and disabled cases.
        // When disabled, we use a temp path that won't persist between sessions.
        let history_path_for_backend = if history_config.enabled {
            history_path.clone()
        } else {
            // Use a temporary path that won't be loaded or saved
            std::env::temp_dir().join("oxur-repl-temp-history")
        };

        let history = Box::new(
            FileBackedHistory::with_file(
                history_config.max_size.unwrap_or(10000),
                history_path_for_backend,
            )
            .context("Failed to create history backend")?,
        );

        // Create completion menu
        let completion_menu = ColumnarMenu::default()
            .with_name("completion_menu")
            .with_columns(4)
            .with_column_width(Some(20))
            .with_column_padding(2);

        // Build reedline editor with syntax highlighting, validation, and completion
        let editor = Reedline::create()
            .with_history(history)
            .with_edit_mode(edit_mode)
            .with_highlighter(Box::new(SExpHighlighter::new(terminal_config.color_enabled)))
            .with_validator(Box::new(SExpValidator::new()))
            .with_completer(Box::new(OxurCompleter::new()))
            .with_menu(ReedlineMenu::EngineCompleter(Box::new(completion_menu)));

        Ok(Self { editor, history_path, terminal_config })
    }

    /// Read a line of input from the user
    ///
    /// Returns:
    /// - `Ok(Some(line))` - User entered a line
    /// - `Ok(None)` - User pressed Ctrl-C (interrupt)
    /// - `Err(_)` - User pressed Ctrl-D (exit) or other error
    pub fn read_line(&mut self, prompt: &str) -> Result<Option<String>> {
        let oxur_prompt = OxurPrompt::new(
            prompt.to_string(),
            self.terminal_config.formatted_continuation_prompt(),
        );

        match self.editor.read_line(&oxur_prompt) {
            Ok(Signal::Success(line)) => Ok(Some(line)),
            Ok(Signal::CtrlC) => Ok(None),
            Ok(Signal::CtrlD) => Err(anyhow::anyhow!("EOF")),
            Err(e) => Err(anyhow::anyhow!("Input error: {}", e)),
        }
    }

    /// Read a line using the default prompt
    pub fn read_line_default(&mut self) -> Result<Option<String>> {
        let prompt = self.prompt();
        self.read_line(&prompt)
    }

    /// Get the formatted prompt string
    pub fn prompt(&self) -> String {
        self.terminal_config.formatted_prompt()
    }

    /// Get the formatted continuation prompt for multi-line input
    #[allow(dead_code)]
    pub fn continuation_prompt(&self) -> String {
        self.terminal_config.formatted_continuation_prompt()
    }

    /// Save command history to disk
    ///
    /// With FileBackedHistory, history is automatically saved.
    /// This method is retained for API compatibility.
    pub fn save_history(&mut self) -> Result<()> {
        // FileBackedHistory auto-saves - this is a no-op
        Ok(())
    }

    /// Check if colors are enabled
    #[allow(dead_code)]
    pub fn color_enabled(&self) -> bool {
        self.terminal_config.color_enabled
    }

    /// Print an error message with appropriate formatting
    pub fn print_error(&self, msg: &str) {
        if self.terminal_config.color_enabled {
            eprintln!("\x1b[31mError:\x1b[0m {}", msg);
        } else {
            eprintln!("Error: {}", msg);
        }
    }

    /// Print a result value with appropriate formatting
    pub fn print_result(&self, value: &str) {
        if self.terminal_config.color_enabled {
            println!("\x1b[36m{}\x1b[0m", value);
        } else {
            println!("{}", value);
        }
    }

    /// Print output (stdout from evaluation)
    pub fn print_output(&self, output: &str) {
        print!("{}", output);
    }

    /// Print help content with appropriate formatting
    ///
    /// Automatically pages content if it exceeds terminal height.
    pub fn print_help(&self, content: &str) {
        if let Err(e) = pager::page_text(content) {
            // Fallback to direct print if paging fails
            eprintln!("Warning: Pager failed ({}), printing directly", e);
            println!("{}", content);
        }
    }

    /// Print the welcome banner with system metadata
    pub fn print_banner(&self, metadata: &oxur_repl::metadata::SystemMetadata) {
        if let Some(ref banner) = self.terminal_config.banner {
            // Custom banner with version substitution
            let banner_with_versions = substitute_banner_versions(banner, metadata);
            println!("{}", banner_with_versions);
        } else {
            // Default banner with version information
            if self.terminal_config.color_enabled {
                println!(
                    "\x1b[1mOxur REPL\x1b[0m v{} | \x1b[90mRust: {} | Cargo: {}\x1b[0m",
                    metadata.oxur_version,
                    format_version(&metadata.rust_version),
                    format_version(&metadata.cargo_version)
                );
            } else {
                println!(
                    "Oxur REPL v{} | Rust: {} | Cargo: {}",
                    metadata.oxur_version,
                    format_version(&metadata.rust_version),
                    format_version(&metadata.cargo_version)
                );
            }
            println!("Type (help) for assistance, Ctrl-D to exit.");
        }
        println!();
    }

    /// Print a goodbye message
    pub fn print_goodbye(&self) {
        println!();
        if self.terminal_config.color_enabled {
            println!("\x1b[33mGoodbye!\x1b[0m");
        } else {
            println!("Goodbye!");
        }
    }

    /// Get the terminal configuration
    pub fn config(&self) -> &TerminalConfig {
        &self.terminal_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_history_path() {
        let path = paths::default_history_path();
        assert!(path.ends_with("repl_history"));
    }

    #[test]
    fn test_terminal_config_prompt() {
        let config = TerminalConfig::builder().prompt("test> ").color(false).build();
        assert_eq!(config.formatted_prompt(), "test> ");
    }

    #[test]
    fn test_terminal_config_colored_prompt() {
        // Non-oxur prompt uses standard green
        let config = TerminalConfig::builder().prompt("test> ").color(true).build();
        assert!(config.formatted_prompt().contains("\x1b[32m"));
        assert!(config.formatted_prompt().contains("test> "));

        // oxur prompt uses special coloring (orange + bright green)
        let oxur_config = TerminalConfig::builder().prompt("oxur> ").color(true).build();
        assert!(oxur_config.formatted_prompt().contains("\x1b[33m")); // Orange
        assert!(oxur_config.formatted_prompt().contains("\x1b[92m")); // Bright green
    }

    #[test]
    fn test_continuation_prompt() {
        let config = TerminalConfig::builder().continuation_prompt("... ").color(false).build();
        assert_eq!(config.formatted_continuation_prompt(), "... ");
    }

    #[test]
    fn test_custom_banner() {
        let config = TerminalConfig::builder().banner("Custom Welcome!").build();
        assert_eq!(config.banner, Some("Custom Welcome!".to_string()));
    }

    #[test]
    fn test_format_version_rustc() {
        let version = "rustc 1.75.0 (82e1608df 2023-12-21)";
        assert_eq!(format_version(version), "1.75.0 (82e1608df 2023-12-21)");
    }

    #[test]
    fn test_format_version_cargo() {
        let version = "cargo 1.75.0 (1d8b05cdd 2023-11-20)";
        assert_eq!(format_version(version), "1.75.0 (1d8b05cdd 2023-11-20)");
    }

    #[test]
    fn test_format_version_unknown() {
        let version = "unknown";
        assert_eq!(format_version(version), "unknown");
    }

    #[test]
    fn test_substitute_banner_versions() {
        let banner = "oxur: N.N.N\nrustc: M.M.M\ncargo: L.L.L";
        let metadata = oxur_repl::metadata::SystemMetadata {
            oxur_version: "0.1.0".to_string(),
            rust_version: "rustc 1.75.0 (82e1608df 2023-12-21)".to_string(),
            cargo_version: "cargo 1.75.0 (1d8b05cdd 2023-11-20)".to_string(),
            os_name: "Test".to_string(),
            os_version: "1.0".to_string(),
            arch: "x86_64".to_string(),
            hostname: "test".to_string(),
            pid: 1234,
            cwd: std::path::PathBuf::from("/test"),
            started_at: std::time::SystemTime::now(),
        };

        let result = substitute_banner_versions(banner, &metadata);
        assert!(result.contains("oxur: 0.1.0"));
        assert!(result.contains("rustc: 1.75.0 (82e1608df 2023-12-21)"));
        assert!(result.contains("cargo: 1.75.0 (1d8b05cdd 2023-11-20)"));
        assert!(!result.contains("N.N.N"));
        assert!(!result.contains("M.M.M"));
        assert!(!result.contains("L.L.L"));
    }
}
