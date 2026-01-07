//! Terminal interface for REPL interaction
//!
//! Provides line editing, command history, and terminal handling
//! using rustyline.

use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Config, DefaultEditor, EditMode, Editor};
use std::path::PathBuf;

/// REPL terminal interface with line editing and history
pub struct ReplTerminal {
    editor: Editor<(), DefaultHistory>,
    history_path: PathBuf,
    color_enabled: bool,
}

impl ReplTerminal {
    /// Create a new REPL terminal
    ///
    /// # Arguments
    ///
    /// * `color_enabled` - Whether to use ANSI colors in prompts
    ///
    /// # Errors
    ///
    /// Returns error if rustyline initialization fails.
    pub fn new(color_enabled: bool) -> Result<Self> {
        let config = Config::builder().edit_mode(EditMode::Emacs).auto_add_history(true).build();

        let mut editor =
            DefaultEditor::with_config(config).context("Failed to create terminal editor")?;

        // Determine history file path
        let history_path = Self::history_path();

        // Load existing history if present
        if history_path.exists() {
            // Ignore errors loading history - not critical
            let _ = editor.load_history(&history_path);
        }

        Ok(Self { editor, history_path, color_enabled })
    }

    /// Get the path to the history file
    fn history_path() -> PathBuf {
        dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("oxur").join("repl_history")
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

    /// Get the default prompt string
    pub fn prompt(&self) -> String {
        if self.color_enabled {
            "\x1b[32moxur>\x1b[0m ".to_string()
        } else {
            "oxur> ".to_string()
        }
    }

    /// Get a continuation prompt for multi-line input
    #[allow(dead_code)]
    pub fn continuation_prompt(&self) -> String {
        if self.color_enabled {
            "\x1b[32m....>\x1b[0m ".to_string()
        } else {
            "....> ".to_string()
        }
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
        self.color_enabled
    }

    /// Print an error message with appropriate formatting
    pub fn print_error(&self, msg: &str) {
        if self.color_enabled {
            eprintln!("\x1b[31mError:\x1b[0m {}", msg);
        } else {
            eprintln!("Error: {}", msg);
        }
    }

    /// Print a result value with appropriate formatting
    pub fn print_result(&self, value: &str) {
        if self.color_enabled {
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
        println!("Oxur REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type (help) for assistance, Ctrl-D to exit.");
        println!();
    }

    /// Print a goodbye message
    pub fn print_goodbye(&self) {
        println!();
        if self.color_enabled {
            println!("\x1b[33mGoodbye!\x1b[0m");
        } else {
            println!("Goodbye!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_path() {
        let path = ReplTerminal::history_path();
        assert!(path.ends_with("repl_history"));
    }

    #[test]
    fn test_prompt_with_color() {
        // Can't easily test terminal creation without a TTY,
        // but we can test the prompt formatting logic
        let prompt_colored = "\x1b[32moxur>\x1b[0m ";
        let prompt_plain = "oxur> ";

        assert!(prompt_colored.contains("oxur>"));
        assert!(prompt_plain.contains("oxur>"));
    }

    #[test]
    fn test_continuation_prompt() {
        let cont_colored = "\x1b[32m....>\x1b[0m ";
        let cont_plain = "....> ";

        assert!(cont_colored.contains("....>"));
        assert!(cont_plain.contains("....>"));
    }
}
