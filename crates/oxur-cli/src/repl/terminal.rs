//! Terminal interface for REPL interaction
//!
//! Provides line editing, command history, and terminal handling
//! using reedline.

use anyhow::{Context, Result};
use oxur_cli::config::{paths, EditMode, HistoryConfig, TerminalConfig};
use reedline::{
    default_emacs_keybindings, default_vi_insert_keybindings, default_vi_normal_keybindings,
    DefaultPrompt, Emacs, FileBackedHistory, Reedline, Signal, Vi,
};
use std::path::PathBuf;

use crate::repl::sexp_highlighter::SExpHighlighter;

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
        // Convert edit mode to reedline's EditMode trait object
        let edit_mode: Box<dyn reedline::EditMode> = match terminal_config.edit_mode {
            EditMode::Emacs => Box::new(Emacs::new(default_emacs_keybindings())),
            EditMode::Vi => {
                Box::new(Vi::new(default_vi_insert_keybindings(), default_vi_normal_keybindings()))
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

        // Build reedline editor with syntax highlighting
        let editor = Reedline::create()
            .with_history(history)
            .with_edit_mode(edit_mode)
            .with_highlighter(Box::new(SExpHighlighter::new(terminal_config.color_enabled)));

        Ok(Self { editor, history_path, terminal_config })
    }

    /// Read a line of input from the user
    ///
    /// Returns:
    /// - `Ok(Some(line))` - User entered a line
    /// - `Ok(None)` - User pressed Ctrl-C (interrupt)
    /// - `Err(_)` - User pressed Ctrl-D (exit) or other error
    pub fn read_line(&mut self, _prompt: &str) -> Result<Option<String>> {
        // Note: DefaultPrompt doesn't support custom prompt strings in reedline.
        // For custom prompts, we'll need to implement the Prompt trait (Phase 3).
        let default_prompt = DefaultPrompt::default();

        match self.editor.read_line(&default_prompt) {
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
    pub fn print_help(&self, content: &str) {
        println!("{}", content);
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
}
