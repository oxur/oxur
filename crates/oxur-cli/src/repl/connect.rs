//! Connect mode REPL client implementation
//!
//! Provides REPL client that connects to a remote server via TCP.

use crate::repl::runner::{ReplClientAdapter, ReplRunner};
use crate::repl::terminal::ReplTerminal;
use anyhow::{Context, Result};
use async_trait::async_trait;
use oxur_cli::config::ReplConfig;
use oxur_repl::protocol::{MessageId, Operation, ReplMode, Request, Response, SessionId};
use oxur_repl::transport::{TcpTransport, Transport};
use std::sync::atomic::{AtomicU64, Ordering};

/// TCP client adapter for connect mode
///
/// Simple wrapper around TcpTransport for remote server connections.
struct TcpAdapter {
    transport: TcpTransport,
}

impl TcpAdapter {
    fn new(transport: TcpTransport) -> Self {
        Self { transport }
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

    // No special command handling for TCP client (stats not available remotely)
}

/// Run the connect mode REPL
///
/// Connects to an existing REPL server and provides terminal interface
/// for sending commands and receiving results.
pub async fn run(addr: &str, config: ReplConfig) -> Result<()> {
    // Connect to server
    let transport = TcpTransport::connect(addr)
        .await
        .context(format!("Failed to connect to REPL server at {}", addr))?;

    // Generate unique session ID
    let session_id = SessionId::new(format!("connect-{}", std::process::id()));

    // Message ID counter
    let msg_counter = AtomicU64::new(1);

    // Create session on server
    let create_req = Request {
        id: MessageId::new(msg_counter.fetch_add(1, Ordering::SeqCst)),
        session_id: session_id.clone(),
        operation: Operation::CreateSession { mode: ReplMode::Lisp },
    };

    let mut adapter = TcpAdapter::new(transport);
    adapter.send_eval(create_req).await.context("Failed to send create session request")?;
    let _response = adapter.recv_response().await.context("Failed to receive create response")?;

    // Create terminal interface with configuration
    let terminal = ReplTerminal::with_config(config.terminal, config.history)
        .context("Failed to create terminal")?;

    // Create runner
    let mut runner = ReplRunner::new(terminal, session_id);

    // Print welcome banner and connection info
    runner.print_banner();
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
