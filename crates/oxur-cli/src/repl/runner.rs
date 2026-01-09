//! Shared REPL loop runner
//!
//! Extracts common REPL loop logic used by both interactive and connect modes.
//! Provides `ReplRunner` struct and `ReplClientAdapter` trait for client abstraction.

use crate::repl::help::HelpSystem;
use crate::repl::terminal::ReplTerminal;
use anyhow::Result;
use async_trait::async_trait;
use oxur_repl::protocol::{
    MessageId, Operation, OperationResult, ReplMode, Request, Response, SessionId,
};
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
    ///
    /// Made async to support protocol-level stats requests in remote mode.
    async fn handle_special_command(
        &mut self,
        _input: &str,
        _color_enabled: bool,
    ) -> Option<String> {
        None
    }

    /// Record a usage metric for the given command type
    ///
    /// Default implementation does nothing (no-op).
    /// Interactive mode will override this to track command frequency.
    fn record_usage(&mut self, _command_type: oxur_repl::metrics::CommandType) {}

    /// Create a new session
    ///
    /// Returns the new session ID on success.
    /// Default implementation returns an error.
    async fn create_session(&mut self, _name: Option<String>) -> Result<SessionId> {
        Err(anyhow::anyhow!("Session creation not supported in this mode"))
    }

    /// Switch to a different session
    ///
    /// Returns Ok if the session exists and switch was successful.
    /// Default implementation returns an error.
    async fn switch_session(&mut self, _session_id: SessionId) -> Result<()> {
        Err(anyhow::anyhow!("Session switching not supported in this mode"))
    }

    /// Get current session ID
    ///
    /// Returns the currently active session ID.
    fn current_session(&self) -> &SessionId;

    /// Close a session
    ///
    /// Closes the specified session. If None, closes current session.
    /// Default implementation returns an error.
    async fn close_session(&mut self, _session_id: Option<SessionId>) -> Result<()> {
        Err(anyhow::anyhow!("Session closing not supported in this mode"))
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
    metadata: Option<oxur_repl::metadata::SystemMetadata>,
}

impl ReplRunner {
    /// Create a new REPL runner
    pub fn new(terminal: ReplTerminal, session_id: SessionId) -> Self {
        Self { terminal, session_id, msg_counter: AtomicU64::new(1), metadata: None }
    }

    /// Get the next message ID
    fn next_message_id(&self) -> MessageId {
        MessageId::new(self.msg_counter.fetch_add(1, Ordering::SeqCst))
    }

    /// Get a reference to the terminal
    pub fn terminal(&self) -> &ReplTerminal {
        &self.terminal
    }

    /// Print the welcome banner with system metadata
    pub fn print_banner(&mut self, metadata: &oxur_repl::metadata::SystemMetadata) {
        self.metadata = Some(metadata.clone());
        self.terminal.print_banner(metadata);
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
                Err(e) => {
                    // Check if it's an EOF error (Ctrl-D)
                    if e.to_string().contains("EOF") {
                        break;
                    }
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
                client.record_usage(oxur_repl::metrics::CommandType::Help);
                self.terminal.print_help(&help_output);
                continue;
            }

            // Check for clear command
            if trimmed == "(clear)" {
                client.record_usage(oxur_repl::metrics::CommandType::Clear);
                if let Err(e) = self.terminal.clear_screen() {
                    self.terminal.print_error(&format!("Failed to clear screen: {}", e));
                }
                continue;
            }

            // Check for banner command
            if trimmed == "(banner)" {
                client.record_usage(oxur_repl::metrics::CommandType::Banner);
                if let Some(ref metadata) = self.metadata {
                    self.terminal.print_banner(metadata);
                } else {
                    self.terminal.print_error("No metadata available");
                }
                continue;
            }

            // Check for session management commands
            if let Some(output) = self.handle_session_command(trimmed, client).await {
                self.terminal.print_help(&output);
                continue;
            }

            // Check for adapter-specific special commands (e.g., stats)
            if let Some(output) = client.handle_special_command(trimmed, color_enabled).await {
                self.terminal.print_help(&output);
                continue;
            }

            // Track eval command
            client.record_usage(oxur_repl::metrics::CommandType::Eval);

            // Create eval request
            let eval_req = Request {
                id: self.next_message_id(),
                session_id: self.session_id.clone(),
                operation: Operation::Eval { code: trimmed.to_string(), mode: ReplMode::Lisp },
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

    /// Handle session management commands
    ///
    /// Returns Some(output) if the command was handled, None otherwise.
    async fn handle_session_command<C: ReplClientAdapter>(
        &mut self,
        input: &str,
        client: &mut C,
    ) -> Option<String> {
        // (current-session)
        if input == "(current-session)" {
            let session_id = client.current_session();
            return Some(format!("Current session: {}", session_id));
        }

        // (new-session)
        if input == "(new-session)" {
            match client.create_session(None).await {
                Ok(new_id) => {
                    self.session_id = new_id.clone();
                    return Some(format!("Created and switched to new session: {}", new_id));
                }
                Err(e) => return Some(format!("Failed to create session: {}", e)),
            }
        }

        // (new-session "name")
        if input.starts_with("(new-session ") && input.ends_with(')') {
            let name_part = &input[13..input.len() - 1].trim();
            // Remove quotes if present
            let name = if name_part.starts_with('"') && name_part.ends_with('"') {
                &name_part[1..name_part.len() - 1]
            } else {
                name_part
            };

            match client.create_session(Some(name.to_string())).await {
                Ok(new_id) => {
                    self.session_id = new_id.clone();
                    return Some(format!(
                        "Created and switched to new session: {} ({})",
                        name, new_id
                    ));
                }
                Err(e) => return Some(format!("Failed to create session: {}", e)),
            }
        }

        // (switch-session <id>)
        if input.starts_with("(switch-session ") && input.ends_with(')') {
            let id_part = &input[16..input.len() - 1].trim();
            let session_id = SessionId::new(*id_part);

            match client.switch_session(session_id.clone()).await {
                Ok(()) => {
                    self.session_id = session_id.clone();
                    return Some(format!("Switched to session: {}", session_id));
                }
                Err(e) => return Some(format!("Failed to switch session: {}", e)),
            }
        }

        // (close-session)
        if input == "(close-session)" {
            match client.close_session(None).await {
                Ok(()) => return Some("Closed current session".to_string()),
                Err(e) => return Some(format!("Failed to close session: {}", e)),
            }
        }

        // (close-session <id>)
        if input.starts_with("(close-session ") && input.ends_with(')') {
            let id_part = &input[15..input.len() - 1].trim();
            let session_id = SessionId::new(*id_part);

            match client.close_session(Some(session_id.clone())).await {
                Ok(()) => return Some(format!("Closed session: {}", session_id)),
                Err(e) => return Some(format!("Failed to close session: {}", e)),
            }
        }

        None
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
            Some(format!("Unknown help topic: {}. Try (help) for available topics.", topic))
        });
    }

    None
}
