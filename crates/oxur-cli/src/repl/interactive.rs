//! Interactive REPL mode implementation
//!
//! Provides the default REPL experience with in-memory client/server.

use crate::repl::info::show_system_info;
use crate::repl::runner::{ReplClientAdapter, ReplRunner};
use crate::repl::stats::{
    parse_stats_command_with_resources, show_all_stats, show_server_stats, show_subprocess_stats,
};
use crate::repl::terminal::ReplTerminal;
use anyhow::{Context, Result};
use async_trait::async_trait;
use oxur_cli::config::ReplConfig;
use oxur_repl::metadata::SystemMetadata;
use oxur_repl::metrics::{ClientMetrics, ServerMetrics};
use oxur_repl::protocol::{
    MessageId, Operation, OperationResult, ReplMode, Request, Response, SessionId,
};
use oxur_repl::server::{MessageHandler, SessionManager};
use oxur_repl::transport::{inprocess_channel, InProcessClient, InProcessServer, Transport};
use std::sync::Arc;

/// Metrics bundle for InProcessAdapter
struct AdapterMetrics {
    server: Arc<ServerMetrics>,
    client: Arc<ClientMetrics>,
}

/// In-process client adapter for interactive mode
///
/// Handles the in-process channel communication and manual server-side
/// request routing, plus stats and info command handling.
///
/// Tracks both server-side and client-side metrics for consistency
/// with server mode.
struct InProcessAdapter {
    client: InProcessClient,
    server: InProcessServer,
    handler: MessageHandler,
    session_manager: Arc<SessionManager>,
    session_id: SessionId,
    system_metadata: Arc<SystemMetadata>,
    metrics: AdapterMetrics,
}

impl InProcessAdapter {
    fn new(
        client: InProcessClient,
        server: InProcessServer,
        handler: MessageHandler,
        session_manager: Arc<SessionManager>,
        session_id: SessionId,
        system_metadata: Arc<SystemMetadata>,
        metrics: AdapterMetrics,
    ) -> Self {
        Self { client, server, handler, session_manager, session_id, system_metadata, metrics }
    }
}

#[async_trait]
impl ReplClientAdapter for InProcessAdapter {
    async fn send_eval(&mut self, request: Request) -> Result<()> {
        // Start client-side latency tracking
        let start_time = std::time::Instant::now();

        // Track server-side request metrics (matching server mode behavior)
        let operation_name = match &request.operation {
            Operation::CreateSession { .. } => "create_session",
            Operation::Clone { .. } => "clone",
            Operation::Eval { .. } => "eval",
            Operation::Close => "close",
            Operation::LsSessions => "ls_sessions",
            Operation::LoadFile { .. } => "load_file",
            Operation::Interrupt => "interrupt",
            Operation::Describe { .. } => "describe",
            Operation::History { .. } => "history",
            Operation::ClearOutput => "clear_output",
            Operation::GetServerStats => "get_server_stats",
            Operation::GetSessionStats => "get_session_stats",
            Operation::GetSubprocessStats => "get_subprocess_stats",
            Operation::GetSystemInfo => "get_system_info",
            _ => "unknown",
        };
        self.metrics.server.request_received(operation_name);

        // Track client-side request
        self.metrics.client.request_sent(operation_name);

        // Track session creation/close for session metrics
        match &request.operation {
            Operation::CreateSession { .. } => self.metrics.server.session_created(),
            Operation::Close => self.metrics.server.session_closed(),
            _ => {}
        }

        // Send request to our side of the channel
        self.client.send_request(&request).await.context("Failed to send request")?;

        // Process request on server side (in-process routing)
        let response = self.handler.handle(request).await;

        // Track server-side response metrics
        let status = match &response.result {
            OperationResult::Success { .. } => "success",
            OperationResult::Error { .. } => "error",
            OperationResult::Sessions { .. } => "success",
            OperationResult::HistoryEntries { .. } => "success",
            OperationResult::ServerStats { .. } => "success",
            OperationResult::SessionStats { .. } => "success",
            OperationResult::SubprocessStats { .. } => "success",
            OperationResult::SystemInfo { .. } => "success",
            _ => "unknown",
        };
        self.metrics.server.response_sent(status);

        // Track client-side response with latency
        let latency = start_time.elapsed();
        self.metrics.client.response_received(status, latency);

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
        // Handle (info) command
        if input == "(info)" {
            // Track usage metric
            if let Ok(usage_metrics) = self.session_manager.get_usage_metrics(&self.session_id) {
                if let Ok(mut metrics) = usage_metrics.lock() {
                    metrics.record_info();
                }
            }
            return Some(show_system_info(&self.system_metadata, color_enabled));
        }

        // Handle (sessions) command
        if input == "(sessions)" {
            // Track usage metric
            if let Ok(usage_metrics) = self.session_manager.get_usage_metrics(&self.session_id) {
                if let Ok(mut metrics) = usage_metrics.lock() {
                    metrics.record_sessions();
                }
            }
            return match self.session_manager.list() {
                Ok(sessions) => Some(crate::repl::stats::show_sessions(
                    &sessions,
                    &self.session_id,
                    color_enabled,
                )),
                Err(e) => Some(format!("Failed to list sessions: {}", e)),
            };
        }

        // Handle stats commands
        if !input.starts_with("(stats") {
            return None;
        }

        // Track stats command usage
        if let Ok(usage_metrics) = self.session_manager.get_usage_metrics(&self.session_id) {
            if let Ok(mut metrics) = usage_metrics.lock() {
                metrics.record_stats();
            }
        }

        // Handle server stats - available in interactive mode via local metrics
        if input == "(stats server)" {
            let snapshot = self.metrics.server.snapshot();
            return Some(show_server_stats(&snapshot, color_enabled));
        }

        // Handle client stats - available in interactive mode via local metrics
        if input == "(stats client)" {
            let snapshot = self.metrics.client.snapshot();
            return Some(crate::repl::stats::show_client_stats(&snapshot, color_enabled));
        }

        // Handle subprocess stats
        if input == "(stats subprocess)" {
            return match self.session_manager.get_subprocess_stats(&self.session_id) {
                Ok(Some(snapshot)) => Some(show_subprocess_stats(&snapshot, color_enabled)),
                Ok(None) => Some("Subprocess not running".to_string()),
                Err(e) => Some(format!("Failed to get subprocess stats: {}", e)),
            };
        }

        // Handle usage stats
        if input == "(stats usage)" {
            return match self.session_manager.get_usage_metrics(&self.session_id) {
                Ok(usage_metrics) => {
                    let metrics = usage_metrics.lock().unwrap();
                    let snapshot = metrics.snapshot();
                    Some(crate::repl::stats::show_usage_stats(&snapshot, color_enabled))
                }
                Err(e) => Some(format!("Failed to get usage metrics: {}", e)),
            };
        }

        // Handle comprehensive stats (all stats combined)
        if input == "(stats)" {
            return match self.session_manager.get_stats_collector(&self.session_id) {
                Ok(stats_collector) => {
                    let (dir_stats, cache_stats) = self
                        .session_manager
                        .get_resource_stats(&self.session_id)
                        .unwrap_or((None, None));

                    let collector = stats_collector.lock().unwrap();

                    // Gather all metrics
                    let server_snapshot = self.metrics.server.snapshot();
                    let client_snapshot = self.metrics.client.snapshot();
                    let subprocess_snapshot =
                        self.session_manager.get_subprocess_stats(&self.session_id).ok().flatten();
                    let usage_snapshot = self
                        .session_manager
                        .get_usage_metrics(&self.session_id)
                        .ok()
                        .and_then(|m| m.lock().ok().map(|metrics| metrics.snapshot()));

                    Some(show_all_stats(
                        &collector,
                        dir_stats.as_ref(),
                        cache_stats.as_ref(),
                        Some(&server_snapshot),
                        Some(&client_snapshot),
                        subprocess_snapshot.as_ref(),
                        usage_snapshot.as_ref(),
                        color_enabled,
                    ))
                }
                Err(e) => Some(format!("Failed to get stats: {}", e)),
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

    fn record_usage(&mut self, command_type: oxur_repl::metrics::CommandType) {
        if let Ok(usage_metrics) = self.session_manager.get_usage_metrics(&self.session_id) {
            if let Ok(mut metrics) = usage_metrics.lock() {
                match command_type {
                    oxur_repl::metrics::CommandType::Eval => metrics.record_eval(),
                    oxur_repl::metrics::CommandType::Help => metrics.record_help(),
                    oxur_repl::metrics::CommandType::Stats => metrics.record_stats(),
                    oxur_repl::metrics::CommandType::Info => metrics.record_info(),
                    oxur_repl::metrics::CommandType::Sessions => metrics.record_sessions(),
                    oxur_repl::metrics::CommandType::Clear => metrics.record_clear(),
                    oxur_repl::metrics::CommandType::Banner => metrics.record_banner(),
                }
            }
        }
    }

    async fn create_session(&mut self, name: Option<String>) -> Result<SessionId> {
        // Generate a new session ID
        let new_id = SessionId::new(format!("session-{}", uuid::Uuid::new_v4()));

        // Create the session via SessionManager
        self.session_manager
            .create(new_id.clone(), ReplMode::Lisp)
            .context("Failed to create new session")?;

        // Set the name if provided
        if let Some(session_name) = name {
            // Note: We'd need to add a set_name method to SessionManager
            // For now, we'll just note the limitation
            let _ = session_name; // Suppress unused warning
        }

        Ok(new_id)
    }

    async fn switch_session(&mut self, session_id: SessionId) -> Result<()> {
        // Verify the session exists
        self.session_manager.get_info(&session_id).context("Session not found")?;

        // Update the current session ID
        self.session_id = session_id;

        Ok(())
    }

    fn current_session(&self) -> &SessionId {
        &self.session_id
    }

    async fn close_session(&mut self, session_id: Option<SessionId>) -> Result<()> {
        let target_id = session_id.unwrap_or_else(|| self.session_id.clone());

        // Don't allow closing the current session
        if target_id == self.session_id {
            return Err(anyhow::anyhow!(
                "Cannot close current session. Switch to another session first."
            ));
        }

        // Close the session
        self.session_manager.close(&target_id).context("Failed to close session")?;

        Ok(())
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
    // Capture system metadata at startup
    let system_metadata = Arc::new(SystemMetadata::capture());

    // Create in-process transport pair
    let (client, server_transport) = inprocess_channel();

    // Create session manager
    let session_manager = Arc::new(SessionManager::new());

    // Create server metrics for tracking (same as server mode)
    let server_metrics = Arc::new(ServerMetrics::new());

    // Create client metrics for latency tracking
    let client_metrics = Arc::new(ClientMetrics::new());

    // Create message handler with both metrics and metadata
    let handler = MessageHandler::with_metrics_and_metadata(
        (*session_manager).clone(),
        server_metrics.clone(),
        system_metadata.clone(),
    );

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

    // Create adapter with metrics bundle
    let metrics = AdapterMetrics { server: server_metrics, client: client_metrics };
    let mut adapter = InProcessAdapter::new(
        client,
        server_transport,
        handler,
        session_manager,
        session_id.clone(),
        system_metadata.clone(),
        metrics,
    );

    // Create runner and run the REPL loop
    let mut runner = ReplRunner::new(terminal, session_id);
    runner.print_banner(&system_metadata);
    runner.run(&mut adapter).await?;
    runner.finish(&mut adapter).await?;

    Ok(())
}
