//! Metrics system for REPL observability
//!
//! Provides unified metrics collection across all REPL components using the
//! `metrics` crate facade pattern. Stats are exposed via the REPL protocol
//! (postcard-encoded) for internal monitoring and debugging.
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
//!                     │ REPL Protocol     │
//!                     │ (postcard-based)  │
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
//! ## Evaluation Metrics
//! - `repl.eval.total` - Evaluations by tier
//! - `repl.eval.duration_ms` - Execution time by tier
//! - `repl.cache.hits` - Cache hits
//! - `repl.cache.misses` - Cache misses

pub mod client;
pub mod eval;
pub mod server;
pub mod subprocess;
pub mod usage;

pub use client::{ClientMetrics, ClientMetricsSnapshot};
pub use eval::{CacheStats, EvalMetrics, ExecutionTier, Percentiles, SessionStatsSnapshot};
pub use server::{ServerMetrics, ServerMetricsSnapshot};
pub use subprocess::{RestartReason, SubprocessMetrics, SubprocessMetricsSnapshot};
pub use usage::{CommandType, UsageMetrics, UsageMetricsSnapshot};

/// Initialize process-level metrics collection.
///
/// Registers process metrics (memory, CPU, file descriptors, etc.) with the
/// metrics facade. These metrics are recorded in-memory and can be retrieved
/// via the REPL protocol using stats requests.
///
/// # Example
///
/// ```no_run
/// use oxur_repl::metrics::init_process_metrics;
///
/// init_process_metrics();
/// // Metrics now being collected, retrieve via protocol stats requests
/// ```
pub fn init_process_metrics() {
    // Register process metrics (memory, CPU, file descriptors, etc.)
    let process_collector = metrics_process::Collector::default();
    process_collector.describe();
}
