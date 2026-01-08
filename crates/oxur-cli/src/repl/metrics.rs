//! Client-side metrics for REPL operations
//!
//! Provides [`MetricsClientAdapter`] wrapper that adds metrics collection
//! around any [`ReplClientAdapter`] implementation.

#[cfg(feature = "binary")]
use crate::repl::runner::ReplClientAdapter;
#[cfg(feature = "binary")]
use anyhow::Result;
#[cfg(feature = "binary")]
use async_trait::async_trait;
#[cfg(feature = "binary")]
use metrics::{counter, histogram};
#[cfg(feature = "binary")]
use oxur_repl::protocol::{OperationResult, Request, Response};
#[cfg(feature = "binary")]
use std::time::Instant;

/// Wrapper that adds metrics collection to any [`ReplClientAdapter`].
///
/// Records:
/// - `repl.client.requests_total` - Requests sent (labeled by operation)
/// - `repl.client.responses_total` - Responses received (labeled by status)
/// - `repl.client.response_latency_ms` - Round-trip latency histogram
///
/// # Example
///
/// ```no_run
/// use oxur_cli::repl::metrics::MetricsClientAdapter;
///
/// // Wrap an existing adapter
/// // let inner_adapter = TcpAdapter::new(...);
/// // let metrics_adapter = MetricsClientAdapter::new(inner_adapter);
/// // runner.run(&mut metrics_adapter).await?;
/// ```
#[cfg(feature = "binary")]
#[derive(Debug)]
#[allow(dead_code)] // Optional infrastructure - not yet wired into connect.rs
pub struct MetricsClientAdapter<C: ReplClientAdapter> {
    inner: C,
    request_start: Option<Instant>,
}

#[cfg(feature = "binary")]
#[allow(dead_code)] // Optional infrastructure - not yet wired into connect.rs
impl<C: ReplClientAdapter> MetricsClientAdapter<C> {
    /// Create a new metrics-enabled adapter wrapping an inner adapter.
    pub fn new(inner: C) -> Self {
        Self { inner, request_start: None }
    }

    /// Unwrap and return the inner adapter.
    pub fn into_inner(self) -> C {
        self.inner
    }
}

#[cfg(feature = "binary")]
#[async_trait]
impl<C: ReplClientAdapter> ReplClientAdapter for MetricsClientAdapter<C> {
    async fn send_eval(&mut self, request: Request) -> Result<()> {
        // Record request start time
        self.request_start = Some(Instant::now());

        // Extract operation name for metrics label
        let operation = match &request.operation {
            oxur_repl::protocol::Operation::CreateSession { .. } => "create_session",
            oxur_repl::protocol::Operation::Clone { .. } => "clone",
            oxur_repl::protocol::Operation::Eval { .. } => "eval",
            oxur_repl::protocol::Operation::Close => "close",
            oxur_repl::protocol::Operation::LsSessions => "ls_sessions",
            oxur_repl::protocol::Operation::LoadFile { .. } => "load_file",
            oxur_repl::protocol::Operation::Interrupt => "interrupt",
            oxur_repl::protocol::Operation::Describe { .. } => "describe",
            oxur_repl::protocol::Operation::History { .. } => "history",
            oxur_repl::protocol::Operation::ClearOutput => "clear_output",
            _ => "unknown",
        };

        counter!("repl.client.requests_total", "operation" => operation).increment(1);

        // Delegate to inner adapter
        self.inner.send_eval(request).await
    }

    async fn recv_response(&mut self) -> Result<Response> {
        let response = self.inner.recv_response().await?;

        // Record latency if we have a start time
        if let Some(start) = self.request_start.take() {
            let latency_ms = start.elapsed().as_millis() as f64;
            histogram!("repl.client.response_latency_ms").record(latency_ms);
        }

        // Record response status
        let status = match &response.result {
            OperationResult::Success { .. } => "success",
            OperationResult::Error { .. } => "error",
            OperationResult::Sessions { .. } => "success",
            OperationResult::HistoryEntries { .. } => "success",
            _ => "unknown",
        };
        counter!("repl.client.responses_total", "status" => status).increment(1);

        Ok(response)
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }

    fn handle_special_command(&mut self, input: &str, color_enabled: bool) -> Option<String> {
        self.inner.handle_special_command(input, color_enabled)
    }
}

#[cfg(test)]
#[cfg(feature = "binary")]
mod tests {
    // Unit tests would require mocking ReplClientAdapter
    // Integration tests are more appropriate for this wrapper
}
