//! Terminal interface for REPL interaction
//!
//! Provides line editing, command history, and terminal handling
//! using rustyline.

use anyhow::{Context, Result};
use oxur_cli::config::{paths, EditMode, HistoryConfig, TerminalConfig};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Config, DefaultEditor, Editor};
use std::path::PathBuf;

/// REPL terminal interface with line editing and history
pub struct ReplTerminal {
    editor: Editor<(), DefaultHistory>,
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
    /// Returns error if rustyline initialization fails.
    pub fn with_config(
        terminal_config: TerminalConfig,
        history_config: HistoryConfig,
    ) -> Result<Self> {
        let rustyline_edit_mode = match terminal_config.edit_mode {
            EditMode::Emacs => rustyline::EditMode::Emacs,
            EditMode::Vi => rustyline::EditMode::Vi,
        };

        let config = Config::builder()
            .edit_mode(rustyline_edit_mode)
            .auto_add_history(history_config.enabled)
            .build();

        let mut editor =
            DefaultEditor::with_config(config).context("Failed to create terminal editor")?;

        // Determine history file path
        let history_path = history_config.path.unwrap_or_else(paths::default_history_path);

        // Load existing history if present and history is enabled
        if history_config.enabled && history_path.exists() {
            // Ignore errors loading history - not critical
            let _ = editor.load_history(&history_path);
        }

        Ok(Self { editor, history_path, terminal_config })
    }

    /// Read a line of input from the user
    ///
    /// Returns:
    /// - `Ok(Some(line))` - User entered a line
    /// - `Ok(None)` - User pressed Ctrl-C (interrupt)
    /// - `Err(ReadlineError::Eof)` - User pressed Ctrl-D (exit)
    /// - `Err(...)` - Other error
    pub fn read_line(&mut self, prompt: &str) -> Result<Option<String>, ReadlineError> {
        match self.editor.readline(prompt) {
            Ok(line) => Ok(Some(line)),
            Err(ReadlineError::Interrupted) => Ok(None), // Ctrl-C
            Err(ReadlineError::Eof) => Err(ReadlineError::Eof), // Ctrl-D
            Err(e) => Err(e),
        }
    }

    /// Read a line using the default prompt
    pub fn read_line_default(&mut self) -> Result<Option<String>, ReadlineError> {
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
    /// Creates the history directory if it doesn't exist.
    pub fn save_history(&mut self) -> Result<()> {
        // Create history directory if needed
        if let Some(parent) = self.history_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create history directory")?;
        }

        self.editor.save_history(&self.history_path).context("Failed to save command history")?;

        Ok(())
    }

    /// Add a line to history manually (in case auto_add_history is disabled)
    #[allow(dead_code)]
    pub fn add_history(&mut self, line: &str) -> Result<()> {
        self.editor.add_history_entry(line).context("Failed to add history entry")?;
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

    /// Print the welcome banner
    pub fn print_banner(&self) {
        if let Some(ref banner) = self.terminal_config.banner {
            println!("{}", banner);
        } else {
            // Default banner
            println!("Oxur REPL v{}", env!("CARGO_PKG_VERSION"));
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
        let config = TerminalConfig::builder().prompt("test> ").color(true).build();
        assert!(config.formatted_prompt().contains("\x1b[32m"));
        assert!(config.formatted_prompt().contains("test> "));
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
}
