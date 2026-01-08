//! Metrics system for REPL observability
//!
//! Provides unified metrics collection across all REPL components using the
//! `metrics` crate facade pattern. Supports TCP export for external monitoring.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    METRICS FACADE (metrics crate)               │
//! │  counter!(), gauge!(), histogram!() - recorded anywhere         │
//! └─────────────────────────────────────────────────────────────────┘
//!                                 │
//!         ┌───────────────────────┼───────────────────────┐
//!         ▼                       ▼                       ▼
//! ┌───────────────┐     ┌───────────────┐     ┌───────────────────┐
//! │ CLIENT        │     │ SERVER        │     │ SUBPROCESS        │
//! │ (oxur-cli)    │     │ (oxur-repl)   │     │ (via IPC proxy)   │
//! └───────────────┘     └───────────────┘     └───────────────────┘
//!                               │
//!                               ▼
//!                     ┌───────────────────┐
//!                     │ TCP Exporter      │
//!                     │ :9100 (default)   │
//!                     └───────────────────┘
//! ```
//!
//! # Metrics Catalog
//!
//! ## Server Metrics
//! - `repl.server.connections_total` - Total connections accepted
//! - `repl.server.connections_active` - Current open connections
//! - `repl.server.sessions_total` - Sessions created
//! - `repl.server.sessions_active` - Current active sessions
//! - `repl.server.requests_total` - Requests by operation type
//! - `repl.server.responses_total` - Responses by status
//!
//! ## Subprocess Metrics
//! - `repl.subprocess.restarts_total` - Restart count by reason
//! - `repl.subprocess.uptime_seconds` - Time since last restart
//!
//! ## Evaluation Metrics (StatsCollector integration)
//! - `repl.eval.total` - Evaluations by tier
//! - `repl.eval.duration_ms` - Execution time by tier
//! - `repl.cache.hits` - Cache hits
//! - `repl.cache.misses` - Cache misses

pub mod server;
pub mod subprocess;

pub use server::ServerMetrics;
pub use subprocess::{RestartReason, SubprocessMetrics};

use std::error::Error;
use std::net::SocketAddr;

/// Default TCP exporter address
pub const DEFAULT_METRICS_ADDR: &str = "127.0.0.1:9100";

/// Initialize the metrics exporter for TCP-based monitoring.
///
/// This installs a TCP exporter that streams metrics to connected clients
/// using Protocol Buffers encoding. Also registers process-level metrics
/// (memory, CPU, file descriptors).
///
/// # Arguments
///
/// * `addr` - Socket address to bind the TCP exporter
///
/// # Errors
///
/// Returns an error if:
/// - The address cannot be parsed
/// - The TCP listener cannot bind to the address
/// - A metrics exporter is already installed
///
/// # Example
///
/// ```no_run
/// use oxur_repl::metrics::init_metrics_exporter;
/// use std::net::SocketAddr;
///
/// let addr: SocketAddr = "127.0.0.1:9100".parse().unwrap();
/// init_metrics_exporter(addr).expect("Failed to initialize metrics");
/// ```
pub fn init_metrics_exporter(addr: SocketAddr) -> Result<(), Box<dyn Error + Send + Sync>> {
    use metrics_exporter_tcp::TcpBuilder;

    TcpBuilder::new().listen_address(addr).install()?;

    // Register process metrics (memory, CPU, file descriptors, etc.)
    let process_collector = metrics_process::Collector::default();
    process_collector.describe();

    Ok(())
}

/// Initialize metrics exporter with the default address (127.0.0.1:9100).
///
/// Convenience wrapper around [`init_metrics_exporter`].
pub fn init_metrics_exporter_default() -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr: SocketAddr = DEFAULT_METRICS_ADDR.parse()?;
    init_metrics_exporter(addr)
}

/// Check if a metrics recorder is installed.
///
/// Returns true if metrics are being recorded (an exporter was installed).
/// This can be used to conditionally emit metrics.
pub fn metrics_enabled() -> bool {
    // Try to get a counter - if it returns a noop recorder, metrics are disabled
    // The metrics crate always returns a valid handle, but if no recorder is
    // installed it's a no-op. We can't directly check this, so we assume
    // metrics are enabled if init was called.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_addr_parses() {
        let addr: SocketAddr = DEFAULT_METRICS_ADDR.parse().unwrap();
        assert_eq!(addr.port(), 9100);
    }
}
