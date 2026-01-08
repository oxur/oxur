//! Interactive REPL mode implementation
//!
//! Provides the default REPL experience with in-memory client/server.

use crate::repl::runner::{ReplClientAdapter, ReplRunner};
use crate::repl::stats::{parse_stats_command_with_resources, show_subprocess_stats};
use crate::repl::terminal::ReplTerminal;
use anyhow::{Context, Result};
use async_trait::async_trait;
use oxur_cli::config::ReplConfig;
use oxur_repl::protocol::{MessageId, Operation, ReplMode, Request, Response, SessionId};
use oxur_repl::server::{MessageHandler, SessionManager};
use oxur_repl::transport::{inprocess_channel, InProcessClient, InProcessServer, Transport};
use std::sync::Arc;

/// In-process client adapter for interactive mode
///
/// Handles the in-process channel communication and manual server-side
/// request routing, plus stats command handling.
struct InProcessAdapter {
    client: InProcessClient,
    server: InProcessServer,
    handler: MessageHandler,
    session_manager: Arc<SessionManager>,
    session_id: SessionId,
}

impl InProcessAdapter {
    fn new(
        client: InProcessClient,
        server: InProcessServer,
        handler: MessageHandler,
        session_manager: Arc<SessionManager>,
        session_id: SessionId,
    ) -> Self {
        Self { client, server, handler, session_manager, session_id }
    }
}

#[async_trait]
impl ReplClientAdapter for InProcessAdapter {
    async fn send_eval(&mut self, request: Request) -> Result<()> {
        // Send request to our side of the channel
        self.client.send_request(&request).await.context("Failed to send request")?;

        // Process request on server side (in-process routing)
        let response = self.handler.handle(request).await;

        // Send response back through the channel
        self.server.send_response(&response).await.context("Failed to send response")?;

        Ok(())
    }

    async fn recv_response(&mut self) -> Result<Response> {
        self.client.recv_response().await.context("Failed to receive response")
    }

    async fn close(&mut self) -> Result<()> {
        // No explicit close needed for in-process channels
        Ok(())
    }

    async fn handle_special_command(&mut self, input: &str, color_enabled: bool) -> Option<String> {
        // Handle stats commands
        if !input.starts_with("(stats") {
            return None;
        }

        // Handle server stats - not available in interactive mode (no dedicated server)
        if input == "(stats server)" {
            return Some(
                "Server stats not available in interactive mode.\n\
                 Use 'oxur repl serve' and connect with 'oxur repl connect' for server mode."
                    .to_string(),
            );
        }

        // Handle subprocess stats
        if input == "(stats subprocess)" {
            return match self.session_manager.get_subprocess_stats(&self.session_id) {
                Ok(Some(snapshot)) => Some(show_subprocess_stats(&snapshot, color_enabled)),
                Ok(None) => Some("Subprocess not running".to_string()),
                Err(e) => Some(format!("Failed to get subprocess stats: {}", e)),
            };
        }

        // Handle other stats commands that use the stats collector
        match self.session_manager.get_stats_collector(&self.session_id) {
            Ok(stats_collector) => {
                let (dir_stats, cache_stats) = self
                    .session_manager
                    .get_resource_stats(&self.session_id)
                    .unwrap_or((None, None));

                let collector = stats_collector.lock().unwrap();
                parse_stats_command_with_resources(
                    input,
                    &collector,
                    dir_stats.as_ref(),
                    cache_stats.as_ref(),
                    color_enabled,
                )
            }
            Err(e) => Some(format!("Failed to get stats: {}", e)),
        }
    }
}

/// Run the interactive REPL mode
///
/// Creates an in-process server and client connected via channels,
/// providing the fastest possible REPL experience with:
/// - Zero serialization overhead
/// - Line editing via rustyline
/// - Command history persistence
/// - Ctrl-C interrupt handling
/// - Ctrl-D exit handling
pub async fn run(config: ReplConfig) -> Result<()> {
    // Create in-process transport pair
    let (client, server_transport) = inprocess_channel();

    // Create session manager and message handler
    let session_manager = Arc::new(SessionManager::new());
    let handler = MessageHandler::new((*session_manager).clone());

    // Generate unique session ID
    let session_id = SessionId::new(format!("interactive-{}", std::process::id()));

    // Create session
    let create_req = Request {
        id: MessageId::new(1),
        session_id: session_id.clone(),
        operation: Operation::CreateSession { mode: ReplMode::Lisp },
    };

    // Process create request directly (no channel needed for setup)
    let _response = handler.handle(create_req).await;

    // Create terminal interface with configuration
    let terminal = ReplTerminal::with_config(config.terminal, config.history)
        .context("Failed to create terminal")?;

    // Create adapter
    let mut adapter = InProcessAdapter::new(
        client,
        server_transport,
        handler,
        session_manager,
        session_id.clone(),
    );

    // Create runner and run the REPL loop
    let mut runner = ReplRunner::new(terminal, session_id);
    runner.print_banner();
    runner.run(&mut adapter).await?;
    runner.finish(&mut adapter).await?;

    Ok(())
}
