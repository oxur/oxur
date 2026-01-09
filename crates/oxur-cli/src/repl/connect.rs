//! Connect mode REPL client implementation
//!
//! Provides REPL client that connects to a remote server via TCP.

use crate::repl::runner::{ReplClientAdapter, ReplRunner};
use crate::repl::stats::{
    show_cache_from_snapshot, show_execution_from_snapshot, show_server_stats,
    show_session_summary_from_snapshot, show_subprocess_stats,
};
use crate::repl::terminal::ReplTerminal;
use anyhow::{Context, Result};
use async_trait::async_trait;
use oxur_cli::config::ReplConfig;
use oxur_repl::protocol::{
    MessageId, Operation, OperationResult, ReplMode, Request, Response, SessionId,
};
use oxur_repl::transport::{TcpTransport, Transport};
use std::sync::atomic::{AtomicU64, Ordering};

/// TCP client adapter for connect mode
///
/// Wraps TcpTransport and provides stats command handling via protocol operations.
struct TcpAdapter {
    transport: TcpTransport,
    session_id: SessionId,
    msg_counter: AtomicU64,
}

impl TcpAdapter {
    fn new(transport: TcpTransport, session_id: SessionId) -> Self {
        Self { transport, session_id, msg_counter: AtomicU64::new(1000) }
    }

    /// Get the next message ID for internal protocol requests
    fn next_message_id(&self) -> MessageId {
        MessageId::new(self.msg_counter.fetch_add(1, Ordering::SeqCst))
    }

    /// Send a stats request and receive the response
    async fn request_stats(&mut self, operation: Operation) -> Result<OperationResult> {
        let request =
            Request { id: self.next_message_id(), session_id: self.session_id.clone(), operation };
        self.transport.send_request(&request).await?;
        let response = self.transport.recv_response().await?;
        Ok(response.result)
    }
}

#[async_trait]
impl ReplClientAdapter for TcpAdapter {
    async fn send_eval(&mut self, request: Request) -> Result<()> {
        self.transport.send_request(&request).await.context("Failed to send request")
    }

    async fn recv_response(&mut self) -> Result<Response> {
        self.transport.recv_response().await.context("Failed to receive response")
    }

    async fn close(&mut self) -> Result<()> {
        self.transport.close().await.context("Failed to close connection")
    }

    async fn handle_special_command(&mut self, input: &str, color_enabled: bool) -> Option<String> {
        // Handle stats commands via protocol
        if !input.starts_with("(stats") {
            return None;
        }

        match input {
            "(stats server)" => match self.request_stats(Operation::GetServerStats).await {
                Ok(OperationResult::ServerStats { snapshot }) => {
                    Some(show_server_stats(&snapshot, color_enabled))
                }
                Ok(_) => Some("Unexpected response from server".to_string()),
                Err(e) => Some(format!("Failed to get server stats: {}", e)),
            },
            "(stats subprocess)" => match self.request_stats(Operation::GetSubprocessStats).await {
                Ok(OperationResult::SubprocessStats { snapshot }) => {
                    Some(show_subprocess_stats(&snapshot, color_enabled))
                }
                Ok(_) => Some("Unexpected response from server".to_string()),
                Err(e) => Some(format!("Failed to get subprocess stats: {}", e)),
            },
            "(stats)" => match self.request_stats(Operation::GetSessionStats).await {
                Ok(OperationResult::SessionStats { snapshot }) => {
                    Some(show_session_summary_from_snapshot(&snapshot, color_enabled))
                }
                Ok(_) => Some("Unexpected response from server".to_string()),
                Err(e) => Some(format!("Failed to get session stats: {}", e)),
            },
            "(stats execution)" => match self.request_stats(Operation::GetSessionStats).await {
                Ok(OperationResult::SessionStats { snapshot }) => {
                    Some(show_execution_from_snapshot(&snapshot, color_enabled))
                }
                Ok(_) => Some("Unexpected response from server".to_string()),
                Err(e) => Some(format!("Failed to get session stats: {}", e)),
            },
            "(stats cache)" => match self.request_stats(Operation::GetSessionStats).await {
                Ok(OperationResult::SessionStats { snapshot }) => {
                    Some(show_cache_from_snapshot(&snapshot, color_enabled))
                }
                Ok(_) => Some("Unexpected response from server".to_string()),
                Err(e) => Some(format!("Failed to get session stats: {}", e)),
            },
            "(stats resources)" => {
                // Resources require local filesystem access, not available remotely
                Some("Resource stats not available in remote mode (requires local filesystem access)".to_string())
            }
            _ => {
                // Unknown stats subcommand
                Some(format!(
                    "Unknown stats command: {}. Try (help stats) for available commands.",
                    input
                ))
            }
        }
    }
}

/// Run the connect mode REPL
///
/// Connects to an existing REPL server and provides terminal interface
/// for sending commands and receiving results.
pub async fn run(addr: &str, config: ReplConfig) -> Result<()> {
    // Capture system metadata at startup
    let system_metadata = oxur_repl::metadata::SystemMetadata::capture();

    // Connect to server
    let transport = TcpTransport::connect(addr)
        .await
        .context(format!("Failed to connect to REPL server at {}", addr))?;

    // Generate unique session ID
    let session_id = SessionId::new(format!("connect-{}", std::process::id()));

    // Create adapter with session_id for stats protocol support
    let mut adapter = TcpAdapter::new(transport, session_id.clone());

    // Create session on server
    let create_req = Request {
        id: adapter.next_message_id(),
        session_id: session_id.clone(),
        operation: Operation::CreateSession { mode: ReplMode::Lisp },
    };

    adapter.send_eval(create_req).await.context("Failed to send create session request")?;
    let _response = adapter.recv_response().await.context("Failed to receive create response")?;

    // Create terminal interface with configuration
    let terminal = ReplTerminal::with_config(config.terminal, config.history)
        .context("Failed to create terminal")?;

    // Create runner
    let mut runner = ReplRunner::new(terminal, session_id);

    // Print welcome banner and connection info
    runner.print_banner(&system_metadata);
    print_connection_info(addr, runner.terminal().config().color_enabled);

    // Run the REPL loop
    runner.run(&mut adapter).await?;
    runner.finish(&mut adapter).await?;

    Ok(())
}

/// Print connection info message
fn print_connection_info(addr: &str, color_enabled: bool) {
    if color_enabled {
        println!("\x1b[36mConnected to {}\x1b[0m\n", addr);
    } else {
        println!("Connected to {}\n", addr);
    }
}
