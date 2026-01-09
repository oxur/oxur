//! Client-side metrics for request/response latency tracking
//!
//! Provides the [`ClientMetrics`] struct for recording client-level metrics
//! with both local tracking (for `(stats client)` display) and `metrics` crate
//! facade integration (for external monitoring).

use metrics::{counter, histogram};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

/// Maximum number of latency samples to keep for percentile calculation
const MAX_LATENCY_SAMPLES: usize = 1000;

/// Client-side metrics recorder.
///
/// Records metrics for:
/// - Request/response latencies (with P50, P95, P99 percentiles)
/// - Request counts by operation type
/// - Response counts by status
/// - Error tracking
///
/// Maintains local state for `(stats client)` display while also emitting
/// to the `metrics` crate facade for external monitoring.
///
/// # Usage
///
/// ```
/// use oxur_repl::metrics::ClientMetrics;
/// use std::time::Duration;
///
/// let metrics = ClientMetrics::new();
///
/// // Record request/response cycle
/// metrics.request_sent("eval");
/// metrics.response_received("success", Duration::from_millis(15));
///
/// // Query local state for display
/// let snapshot = metrics.snapshot();
/// println!("Average latency: {:.2}ms", snapshot.average_latency_ms);
/// ```
#[derive(Debug)]
pub struct ClientMetrics {
    // Request/response tracking
    requests_total: AtomicU64,
    responses_total: AtomicU64,
    responses_success: AtomicU64,
    responses_error: AtomicU64,

    // Latency tracking (protected by mutex for percentile calculation)
    latency_samples: Mutex<VecDeque<Duration>>,
}

/// Snapshot of client metrics for display
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClientMetricsSnapshot {
    pub requests_total: u64,
    pub responses_total: u64,
    pub responses_success: u64,
    pub responses_error: u64,
    pub average_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
}

impl Default for ClientMetrics {
    fn default() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            responses_total: AtomicU64::new(0),
            responses_success: AtomicU64::new(0),
            responses_error: AtomicU64::new(0),
            latency_samples: Mutex::new(VecDeque::with_capacity(MAX_LATENCY_SAMPLES)),
        }
    }
}

impl ClientMetrics {
    /// Create a new ClientMetrics instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a request being sent.
    ///
    /// Increments `repl.client.requests_total` counter with operation label.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation type (e.g., "eval", "create_session", "close")
    pub fn request_sent(&self, operation: &'static str) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        counter!("repl.client.requests_total", "operation" => operation).increment(1);
    }

    /// Record a response being received with latency.
    ///
    /// Increments `repl.client.responses_total` counter with status label
    /// and records latency to histogram.
    ///
    /// # Arguments
    ///
    /// * `status` - The response status (e.g., "success", "error")
    /// * `latency` - The round-trip latency for this request/response
    pub fn response_received(&self, status: &'static str, latency: Duration) {
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

        counter!("repl.client.responses_total", "status" => status).increment(1);
        histogram!("repl.client.latency_ms", "status" => status)
            .record(latency.as_secs_f64() * 1000.0);

        // Store sample for local percentile calculation
        let mut samples = self.latency_samples.lock().unwrap();
        if samples.len() >= MAX_LATENCY_SAMPLES {
            samples.pop_front();
        }
        samples.push_back(latency);
    }

    /// Calculate percentile from sorted samples.
    ///
    /// Returns the value at the given percentile (0.0 to 1.0).
    fn calculate_percentile(sorted_samples: &[f64], percentile: f64) -> f64 {
        if sorted_samples.is_empty() {
            return 0.0;
        }
        let index = ((sorted_samples.len() as f64) * percentile).floor() as usize;
        let index = index.min(sorted_samples.len() - 1);
        sorted_samples[index]
    }

    /// Get a snapshot of current metrics for display.
    ///
    /// Returns a point-in-time snapshot of all client metrics including
    /// latency percentiles.
    pub fn snapshot(&self) -> ClientMetricsSnapshot {
        let samples = self.latency_samples.lock().unwrap();
        let mut sorted_ms: Vec<f64> = samples.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        sorted_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let average_latency_ms = if !sorted_ms.is_empty() {
            sorted_ms.iter().sum::<f64>() / sorted_ms.len() as f64
        } else {
            0.0
        };

        let min_latency_ms = sorted_ms.first().copied().unwrap_or(0.0);
        let max_latency_ms = sorted_ms.last().copied().unwrap_or(0.0);

        ClientMetricsSnapshot {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            responses_total: self.responses_total.load(Ordering::Relaxed),
            responses_success: self.responses_success.load(Ordering::Relaxed),
            responses_error: self.responses_error.load(Ordering::Relaxed),
            average_latency_ms,
            p50_latency_ms: Self::calculate_percentile(&sorted_ms, 0.50),
            p95_latency_ms: Self::calculate_percentile(&sorted_ms, 0.95),
            p99_latency_ms: Self::calculate_percentile(&sorted_ms, 0.99),
            min_latency_ms,
            max_latency_ms,
        }
    }

    /// Get total requests count.
    pub fn requests_total(&self) -> u64 {
        self.requests_total.load(Ordering::Relaxed)
    }

    /// Get total responses count.
    pub fn responses_total(&self) -> u64 {
        self.responses_total.load(Ordering::Relaxed)
    }

    /// Get success responses count.
    pub fn responses_success(&self) -> u64 {
        self.responses_success.load(Ordering::Relaxed)
    }

    /// Get error responses count.
    pub fn responses_error(&self) -> u64 {
        self.responses_error.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_metrics_creation() {
        let metrics = ClientMetrics::new();

        // Should start at zero
        assert_eq!(metrics.requests_total(), 0);
        assert_eq!(metrics.responses_total(), 0);
        assert_eq!(metrics.responses_success(), 0);
        assert_eq!(metrics.responses_error(), 0);
    }

    #[test]
    fn test_request_tracking() {
        let metrics = ClientMetrics::new();

        metrics.request_sent("eval");
        metrics.request_sent("eval");
        metrics.request_sent("close");

        assert_eq!(metrics.requests_total(), 3);
    }

    #[test]
    fn test_response_tracking() {
        let metrics = ClientMetrics::new();

        metrics.response_received("success", Duration::from_millis(10));
        metrics.response_received("success", Duration::from_millis(20));
        metrics.response_received("error", Duration::from_millis(5));

        assert_eq!(metrics.responses_total(), 3);
        assert_eq!(metrics.responses_success(), 2);
        assert_eq!(metrics.responses_error(), 1);
    }

    #[test]
    fn test_latency_tracking() {
        let metrics = ClientMetrics::new();

        metrics.response_received("success", Duration::from_millis(10));
        metrics.response_received("success", Duration::from_millis(20));
        metrics.response_received("success", Duration::from_millis(30));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.average_latency_ms, 20.0);
        assert_eq!(snapshot.min_latency_ms, 10.0);
        assert_eq!(snapshot.max_latency_ms, 30.0);
    }

    #[test]
    fn test_percentiles() {
        let metrics = ClientMetrics::new();

        // Add 100 samples from 1ms to 100ms
        for i in 1..=100 {
            metrics.response_received("success", Duration::from_millis(i));
        }

        let snapshot = metrics.snapshot();
        assert!((snapshot.p50_latency_ms - 50.0).abs() < 2.0); // ~50ms
        assert!((snapshot.p95_latency_ms - 95.0).abs() < 2.0); // ~95ms
        assert!((snapshot.p99_latency_ms - 99.0).abs() < 2.0); // ~99ms
    }

    #[test]
    fn test_max_samples_limit() {
        let metrics = ClientMetrics::new();

        // Add more than MAX_LATENCY_SAMPLES
        for i in 1..=(MAX_LATENCY_SAMPLES + 100) {
            metrics.response_received("success", Duration::from_millis(i as u64));
        }

        let samples = metrics.latency_samples.lock().unwrap();
        assert_eq!(samples.len(), MAX_LATENCY_SAMPLES);
    }

    #[test]
    fn test_snapshot_with_no_data() {
        let metrics = ClientMetrics::new();
        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.requests_total, 0);
        assert_eq!(snapshot.responses_total, 0);
        assert_eq!(snapshot.average_latency_ms, 0.0);
        assert_eq!(snapshot.p50_latency_ms, 0.0);
        assert_eq!(snapshot.p95_latency_ms, 0.0);
        assert_eq!(snapshot.p99_latency_ms, 0.0);
    }
}
