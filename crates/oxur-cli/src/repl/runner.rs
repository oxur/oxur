//! Shared REPL loop runner
//!
//! Extracts common REPL loop logic used by both interactive and connect modes.
//! Provides `ReplRunner` struct and `ReplClientAdapter` trait for client abstraction.

use crate::repl::help::HelpSystem;
use crate::repl::terminal::ReplTerminal;
use anyhow::Result;
use async_trait::async_trait;
use oxur_repl::protocol::{MessageId, Operation, OperationResult, ReplMode, Request, Response, SessionId};
use rustyline::error::ReadlineError;
use std::sync::atomic::{AtomicU64, Ordering};

/// Trait for REPL client adapters
///
/// Abstracts the transport layer for different REPL client modes:
/// - `InProcessAdapter`: Channel-based for interactive mode
/// - `TcpAdapter`: TCP transport for connect mode
#[async_trait]
pub trait ReplClientAdapter: Send {
    /// Send an eval request to the server
    async fn send_eval(&mut self, request: Request) -> Result<()>;

    /// Receive a response from the server
    async fn recv_response(&mut self) -> Result<Response>;

    /// Close the client connection
    async fn close(&mut self) -> Result<()>;

    /// Handle special commands (e.g., stats in interactive mode)
    ///
    /// Returns `Some(output)` if the command was handled, `None` to continue normal eval.
    /// Default implementation returns `None` (no special handling).
    fn handle_special_command(&mut self, _input: &str, _color_enabled: bool) -> Option<String> {
        None
    }
}

/// Shared REPL loop runner
///
/// Encapsulates the main REPL loop logic shared between interactive and connect modes.
/// Uses a `ReplClientAdapter` for transport-specific behavior.
pub struct ReplRunner {
    terminal: ReplTerminal,
    session_id: SessionId,
    msg_counter: AtomicU64,
}

impl ReplRunner {
    /// Create a new REPL runner
    pub fn new(terminal: ReplTerminal, session_id: SessionId) -> Self {
        Self {
            terminal,
            session_id,
            msg_counter: AtomicU64::new(1),
        }
    }

    /// Get the next message ID
    fn next_message_id(&self) -> MessageId {
        MessageId::new(self.msg_counter.fetch_add(1, Ordering::SeqCst))
    }

    /// Get a reference to the terminal
    pub fn terminal(&self) -> &ReplTerminal {
        &self.terminal
    }

    /// Print the welcome banner
    pub fn print_banner(&self) {
        self.terminal.print_banner();
    }

    /// Run the main REPL loop
    ///
    /// Processes user input, sends requests to the server via the adapter,
    /// and displays results. Handles help commands, quit commands, and
    /// special adapter-specific commands.
    pub async fn run<C: ReplClientAdapter>(&mut self, client: &mut C) -> Result<()> {
        loop {
            // Read input from user
            let line = match self.terminal.read_line_default() {
                Ok(Some(line)) => line,
                Ok(None) => {
                    // Ctrl-C - just print newline and continue
                    println!();
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D - exit
                    break;
                }
                Err(e) => {
                    self.terminal.print_error(&format!("Input error: {}", e));
                    break;
                }
            };

            // Skip empty lines
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check for quit commands
            if Self::is_quit_command(trimmed) {
                break;
            }

            // Check for help commands
            let color_enabled = self.terminal.config().color_enabled;
            if let Some(help_output) = parse_help_command(trimmed, color_enabled) {
                self.terminal.print_help(&help_output);
                continue;
            }

            // Check for adapter-specific special commands (e.g., stats)
            if let Some(output) = client.handle_special_command(trimmed, color_enabled) {
                self.terminal.print_help(&output);
                continue;
            }

            // Create eval request
            let eval_req = Request {
                id: self.next_message_id(),
                session_id: self.session_id.clone(),
                operation: Operation::Eval {
                    code: trimmed.to_string(),
                    mode: ReplMode::Lisp,
                },
            };

            // Send request to server
            if let Err(e) = client.send_eval(eval_req).await {
                self.terminal.print_error(&format!("Failed to send request: {}", e));
                continue;
            }

            // Receive response
            let response = match client.recv_response().await {
                Ok(r) => r,
                Err(e) => {
                    self.terminal.print_error(&format!("Failed to receive response: {}", e));
                    continue;
                }
            };

            // Display result
            self.display_result(&response.result);
        }

        Ok(())
    }

    /// Finish the REPL session
    ///
    /// Saves history, prints goodbye, and closes the session on the server.
    pub async fn finish<C: ReplClientAdapter>(&mut self, client: &mut C) -> Result<()> {
        // Save history before exit
        if let Err(e) = self.terminal.save_history() {
            eprintln!("Warning: Failed to save command history: {}", e);
        }

        self.terminal.print_goodbye();

        // Close session
        let close_req = Request {
            id: self.next_message_id(),
            session_id: self.session_id.clone(),
            operation: Operation::Close,
        };

        let _ = client.send_eval(close_req).await;
        let _ = client.recv_response().await;
        let _ = client.close().await;

        Ok(())
    }

    /// Check if input is a quit command
    fn is_quit_command(input: &str) -> bool {
        matches!(input, "(quit)" | "(q)" | "(exit)")
    }

    /// Display an operation result
    fn display_result(&self, result: &OperationResult) {
        match result {
            OperationResult::Success { value, stdout, stderr, .. } => {
                // Print stdout if any
                if let Some(out) = stdout {
                    if !out.is_empty() {
                        self.terminal.print_output(out);
                    }
                }

                // Print return value if any
                if let Some(val) = value {
                    if !val.is_empty() {
                        self.terminal.print_result(val);
                    }
                }

                // Print stderr if any
                if let Some(err) = stderr {
                    if !err.is_empty() {
                        eprintln!("{}", err);
                    }
                }
            }
            OperationResult::Error { error, stdout, stderr } => {
                // Print any stdout before the error
                if let Some(out) = stdout {
                    if !out.is_empty() {
                        self.terminal.print_output(out);
                    }
                }

                // Print the error message
                self.terminal.print_error(&error.message);

                // Print stderr if any
                if let Some(err) = stderr {
                    if !err.is_empty() {
                        eprintln!("{}", err);
                    }
                }
            }
            OperationResult::Sessions { .. } | OperationResult::HistoryEntries { .. } => {
                // These don't produce output in interactive eval mode
            }
            _ => {
                // Handle any future OperationResult variants
            }
        }
    }
}

/// Parse help commands and return formatted help output
///
/// Recognizes:
/// - `(help)` - Returns overview help
/// - `(help <topic>)` - Returns topic-specific help or error message
///
/// Returns `None` if input is not a help command.
pub fn parse_help_command(input: &str, color_enabled: bool) -> Option<String> {
    let help_system = HelpSystem::new(color_enabled);

    if input == "(help)" {
        return Some(help_system.show_overview());
    }

    // Parse (help <topic>)
    if input.starts_with("(help ") && input.ends_with(')') {
        let topic = &input[6..input.len() - 1].trim();
        return help_system.show_topic(topic).or_else(|| {
            Some(format!(
                "Unknown help topic: {}. Try (help) for available topics.",
                topic
            ))
        });
    }

    None
}
