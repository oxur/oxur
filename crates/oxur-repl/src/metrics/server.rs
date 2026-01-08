//! Server-side metrics for connection and session tracking
//!
//! Provides the [`ServerMetrics`] struct for recording server-level metrics
//! with both local tracking (for `(stats)` display) and `metrics` crate facade
//! integration (for external monitoring).

use metrics::{counter, gauge};
use std::sync::atomic::{AtomicU64, Ordering};

/// Server-side metrics recorder.
///
/// Records metrics for:
/// - Connection lifecycle (accepted, closed, active count)
/// - Session lifecycle (created, closed, active count)
/// - Request/response counts by type and status
///
/// Maintains local state for `(stats server)` display while also emitting
/// to the `metrics` crate facade for external monitoring.
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
///
/// // Query local state for display
/// let snapshot = metrics.snapshot();
/// println!("Total connections: {}", snapshot.connections_total);
/// ```
#[derive(Debug, Default)]
pub struct ServerMetrics {
    // Connection tracking
    connections_total: AtomicU64,
    connections_active: AtomicU64,

    // Session tracking
    sessions_total: AtomicU64,
    sessions_active: AtomicU64,

    // Request/response tracking (simplified - detailed by label in metrics facade)
    requests_total: AtomicU64,
    responses_total: AtomicU64,
    responses_success: AtomicU64,
    responses_error: AtomicU64,
}

/// Snapshot of server metrics for display
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ServerMetricsSnapshot {
    pub connections_total: u64,
    pub connections_active: u64,
    pub sessions_total: u64,
    pub sessions_active: u64,
    pub requests_total: u64,
    pub responses_total: u64,
    pub responses_success: u64,
    pub responses_error: u64,
}

impl ServerMetrics {
    /// Create a new ServerMetrics instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new connection being accepted.
    ///
    /// Increments `repl.server.connections_total` counter and
    /// `repl.server.connections_active` gauge.
    pub fn connection_accepted(&self) {
        self.connections_total.fetch_add(1, Ordering::Relaxed);
        self.connections_active.fetch_add(1, Ordering::Relaxed);

        counter!("repl.server.connections_total").increment(1);
        gauge!("repl.server.connections_active")
            .set(self.connections_active.load(Ordering::Relaxed) as f64);
    }

    /// Record a connection being closed.
    ///
    /// Decrements `repl.server.connections_active` gauge.
    pub fn connection_closed(&self) {
        self.connections_active.fetch_sub(1, Ordering::Relaxed);

        gauge!("repl.server.connections_active")
            .set(self.connections_active.load(Ordering::Relaxed) as f64);
    }

    /// Record a new session being created.
    ///
    /// Increments `repl.server.sessions_total` counter and
    /// `repl.server.sessions_active` gauge.
    pub fn session_created(&self) {
        self.sessions_total.fetch_add(1, Ordering::Relaxed);
        self.sessions_active.fetch_add(1, Ordering::Relaxed);

        counter!("repl.server.sessions_total").increment(1);
        gauge!("repl.server.sessions_active")
            .set(self.sessions_active.load(Ordering::Relaxed) as f64);
    }

    /// Record a session being closed.
    ///
    /// Decrements `repl.server.sessions_active` gauge.
    pub fn session_closed(&self) {
        self.sessions_active.fetch_sub(1, Ordering::Relaxed);

        gauge!("repl.server.sessions_active")
            .set(self.sessions_active.load(Ordering::Relaxed) as f64);
    }

    /// Record a request being received.
    ///
    /// Increments `repl.server.requests_total` counter with operation label.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation type (e.g., "eval", "create_session", "close")
    pub fn request_received(&self, operation: &'static str) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);

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
        self.responses_total.fetch_add(1, Ordering::Relaxed);

        match status {
            "success" => {
                self.responses_success.fetch_add(1, Ordering::Relaxed);
            }
            "error" => {
                self.responses_error.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }

        counter!("repl.server.responses_total", "status" => status).increment(1);
    }

    /// Get a snapshot of current metrics for display.
    ///
    /// Returns a point-in-time snapshot of all server metrics.
    pub fn snapshot(&self) -> ServerMetricsSnapshot {
        ServerMetricsSnapshot {
            connections_total: self.connections_total.load(Ordering::Relaxed),
            connections_active: self.connections_active.load(Ordering::Relaxed),
            sessions_total: self.sessions_total.load(Ordering::Relaxed),
            sessions_active: self.sessions_active.load(Ordering::Relaxed),
            requests_total: self.requests_total.load(Ordering::Relaxed),
            responses_total: self.responses_total.load(Ordering::Relaxed),
            responses_success: self.responses_success.load(Ordering::Relaxed),
            responses_error: self.responses_error.load(Ordering::Relaxed),
        }
    }

    /// Get total connections count.
    pub fn connections_total(&self) -> u64 {
        self.connections_total.load(Ordering::Relaxed)
    }

    /// Get active connections count.
    pub fn connections_active(&self) -> u64 {
        self.connections_active.load(Ordering::Relaxed)
    }

    /// Get total sessions count.
    pub fn sessions_total(&self) -> u64 {
        self.sessions_total.load(Ordering::Relaxed)
    }

    /// Get active sessions count.
    pub fn sessions_active(&self) -> u64 {
        self.sessions_active.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_metrics_creation() {
        let metrics = ServerMetrics::new();

        // Should start at zero
        assert_eq!(metrics.connections_total(), 0);
        assert_eq!(metrics.connections_active(), 0);
        assert_eq!(metrics.sessions_total(), 0);
        assert_eq!(metrics.sessions_active(), 0);
    }

    #[test]
    fn test_connection_tracking() {
        let metrics = ServerMetrics::new();

        metrics.connection_accepted();
        assert_eq!(metrics.connections_total(), 1);
        assert_eq!(metrics.connections_active(), 1);

        metrics.connection_accepted();
        assert_eq!(metrics.connections_total(), 2);
        assert_eq!(metrics.connections_active(), 2);

        metrics.connection_closed();
        assert_eq!(metrics.connections_total(), 2);
        assert_eq!(metrics.connections_active(), 1);
    }

    #[test]
    fn test_session_tracking() {
        let metrics = ServerMetrics::new();

        metrics.session_created();
        assert_eq!(metrics.sessions_total(), 1);
        assert_eq!(metrics.sessions_active(), 1);

        metrics.session_closed();
        assert_eq!(metrics.sessions_total(), 1);
        assert_eq!(metrics.sessions_active(), 0);
    }

    #[test]
    fn test_request_response_tracking() {
        let metrics = ServerMetrics::new();

        metrics.request_received("eval");
        metrics.request_received("eval");
        metrics.request_received("close");

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.requests_total, 3);

        metrics.response_sent("success");
        metrics.response_sent("success");
        metrics.response_sent("error");

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.responses_total, 3);
        assert_eq!(snapshot.responses_success, 2);
        assert_eq!(snapshot.responses_error, 1);
    }

    #[test]
    fn test_snapshot() {
        let metrics = ServerMetrics::new();

        metrics.connection_accepted();
        metrics.session_created();
        metrics.request_received("eval");
        metrics.response_sent("success");

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.connections_total, 1);
        assert_eq!(snapshot.connections_active, 1);
        assert_eq!(snapshot.sessions_total, 1);
        assert_eq!(snapshot.sessions_active, 1);
        assert_eq!(snapshot.requests_total, 1);
        assert_eq!(snapshot.responses_total, 1);
        assert_eq!(snapshot.responses_success, 1);
        assert_eq!(snapshot.responses_error, 0);
    }
}
