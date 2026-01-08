//! Server-side metrics for connection and session tracking
//!
//! Provides the [`ServerMetrics`] struct for recording server-level metrics
//! via the `metrics` crate facade.

use metrics::{counter, gauge};

/// Server-side metrics recorder.
///
/// Records metrics for:
/// - Connection lifecycle (accepted, closed, active count)
/// - Session lifecycle (created, closed, active count)
/// - Request/response counts by type and status
///
/// # Usage
///
/// ```
/// use oxur_repl::metrics::ServerMetrics;
///
/// let metrics = ServerMetrics::new();
///
/// // On new connection
/// metrics.connection_accepted();
///
/// // On connection close
/// metrics.connection_closed();
///
/// // On session operations
/// metrics.session_created();
/// metrics.request_received("eval");
/// metrics.response_sent("success");
/// metrics.session_closed();
/// ```
#[derive(Debug, Clone, Default)]
pub struct ServerMetrics {
    // Stateless - all state is in the metrics backend
}

impl ServerMetrics {
    /// Create a new ServerMetrics instance.
    ///
    /// This is a lightweight operation - the actual metrics are stored
    /// in the global metrics registry.
    pub fn new() -> Self {
        Self {}
    }

    /// Record a new connection being accepted.
    ///
    /// Increments `repl.server.connections_total` counter and
    /// `repl.server.connections_active` gauge.
    pub fn connection_accepted(&self) {
        counter!("repl.server.connections_total").increment(1);
        gauge!("repl.server.connections_active").increment(1.0);
    }

    /// Record a connection being closed.
    ///
    /// Decrements `repl.server.connections_active` gauge.
    pub fn connection_closed(&self) {
        gauge!("repl.server.connections_active").decrement(1.0);
    }

    /// Record a new session being created.
    ///
    /// Increments `repl.server.sessions_total` counter and
    /// `repl.server.sessions_active` gauge.
    pub fn session_created(&self) {
        counter!("repl.server.sessions_total").increment(1);
        gauge!("repl.server.sessions_active").increment(1.0);
    }

    /// Record a session being closed.
    ///
    /// Decrements `repl.server.sessions_active` gauge.
    pub fn session_closed(&self) {
        gauge!("repl.server.sessions_active").decrement(1.0);
    }

    /// Record a request being received.
    ///
    /// Increments `repl.server.requests_total` counter with operation label.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation type (e.g., "eval", "create_session", "close")
    pub fn request_received(&self, operation: &'static str) {
        counter!("repl.server.requests_total", "operation" => operation).increment(1);
    }

    /// Record a response being sent.
    ///
    /// Increments `repl.server.responses_total` counter with status label.
    ///
    /// # Arguments
    ///
    /// * `status` - The response status (e.g., "success", "error")
    pub fn response_sent(&self, status: &'static str) {
        counter!("repl.server.responses_total", "status" => status).increment(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_metrics_creation() {
        // Metrics should be creatable without a recorder installed
        let metrics = ServerMetrics::new();

        // These should not panic even without a recorder
        metrics.connection_accepted();
        metrics.connection_closed();
        metrics.session_created();
        metrics.session_closed();
        metrics.request_received("eval");
        metrics.response_sent("success");
    }

    #[test]
    fn test_server_metrics_clone() {
        let metrics = ServerMetrics::new();
        let _cloned = metrics.clone();
    }
}
