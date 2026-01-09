//! Evaluation metrics for REPL performance tracking
//!
//! Provides [`EvalMetrics`] for tracking evaluation timing and cache performance
//! with both local tracking (for `(stats)` display with percentiles) and
//! `metrics` crate facade integration (for external monitoring).

use metrics::{counter, histogram};
use std::collections::VecDeque;
use std::time::Duration;

// Re-export ExecutionTier from eval (canonical definition is in eval::context)
pub use crate::eval::ExecutionTier;

/// Maximum timing samples to keep per tier (for percentile calculation)
const MAX_SAMPLES: usize = 1000;

/// Evaluation metrics collector.
///
/// Tracks execution timing samples and cache metrics per session.
/// Maintains local state for percentile calculation while also emitting
/// to the `metrics` crate facade for external monitoring.
///
/// Memory-bounded by MAX_SAMPLES limit per tier (~24KB total).
///
/// # Usage
///
/// ```
/// use oxur_repl::metrics::{EvalMetrics, ExecutionTier};
/// use std::time::Duration;
///
/// let mut metrics = EvalMetrics::new("session-1");
///
/// // Record an evaluation
/// metrics.record(ExecutionTier::Calculator, false, Duration::from_millis(1));
///
/// // Get percentiles for display
/// if let Some(p) = metrics.percentiles(ExecutionTier::Calculator) {
///     println!("p50: {:.2}ms", p.p50);
/// }
///
/// // Get cache stats
/// let cache = metrics.cache_stats();
/// println!("Hit rate: {:.1}%", cache.hit_rate);
/// ```
#[derive(Debug, Clone)]
pub struct EvalMetrics {
    /// Session identifier
    session_id: String,

    /// Tier 1 (Calculator) timing samples
    tier1_samples: VecDeque<Duration>,

    /// Tier 2 (CachedLoaded) timing samples
    tier2_samples: VecDeque<Duration>,

    /// Tier 3 (JustInTime) timing samples
    tier3_samples: VecDeque<Duration>,

    /// Cache hit count
    cache_hits: u64,

    /// Cache miss count
    cache_misses: u64,

    /// Total evaluations
    total_evals: u64,

    /// Parse error count
    parse_errors: u64,

    /// Compile error count
    compile_errors: u64,

    /// Runtime error count
    runtime_errors: u64,
}

impl EvalMetrics {
    /// Create a new metrics collector for a session.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            tier1_samples: VecDeque::with_capacity(MAX_SAMPLES),
            tier2_samples: VecDeque::with_capacity(MAX_SAMPLES),
            tier3_samples: VecDeque::with_capacity(MAX_SAMPLES),
            cache_hits: 0,
            cache_misses: 0,
            total_evals: 0,
            parse_errors: 0,
            compile_errors: 0,
            runtime_errors: 0,
        }
    }

    /// Record an evaluation result.
    ///
    /// Adds a timing sample to the appropriate tier and updates cache metrics.
    /// Uses a circular buffer pattern - oldest samples are evicted when MAX_SAMPLES is reached.
    ///
    /// Also emits metrics via the `metrics` crate facade:
    /// - `repl.eval.total` (counter, labeled by tier)
    /// - `repl.eval.duration_ms` (histogram, labeled by tier)
    /// - `repl.cache.hits` / `repl.cache.misses` (counters)
    pub fn record(&mut self, tier: ExecutionTier, cached: bool, duration: Duration) {
        self.total_evals += 1;

        if cached {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }

        // Add sample to appropriate tier, evicting oldest if at capacity
        let samples = match tier {
            ExecutionTier::Calculator => &mut self.tier1_samples,
            ExecutionTier::CachedLoaded => &mut self.tier2_samples,
            ExecutionTier::JustInTime => &mut self.tier3_samples,
        };

        if samples.len() >= MAX_SAMPLES {
            samples.pop_front();
        }
        samples.push_back(duration);

        // Emit metrics via facade
        let tier_label = tier.as_label();

        counter!("repl.eval.total", "tier" => tier_label).increment(1);
        histogram!("repl.eval.duration_ms", "tier" => tier_label)
            .record(duration.as_millis() as f64);

        if cached {
            counter!("repl.cache.hits").increment(1);
        } else {
            counter!("repl.cache.misses").increment(1);
        }
    }

    /// Calculate percentiles for a given tier.
    ///
    /// Returns None if no samples have been recorded for this tier.
    pub fn percentiles(&self, tier: ExecutionTier) -> Option<Percentiles> {
        let samples = match tier {
            ExecutionTier::Calculator => &self.tier1_samples,
            ExecutionTier::CachedLoaded => &self.tier2_samples,
            ExecutionTier::JustInTime => &self.tier3_samples,
        };

        if samples.is_empty() {
            return None;
        }

        // Convert to sorted vec of milliseconds
        let mut sorted: Vec<f64> = samples.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Some(Percentiles {
            p50: percentile(&sorted, 50.0),
            p95: percentile(&sorted, 95.0),
            p99: percentile(&sorted, 99.0),
            min: sorted[0],
            max: sorted[sorted.len() - 1],
            count: sorted.len(),
        })
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.cache_hits,
            misses: self.cache_misses,
            hit_rate: if self.total_evals > 0 {
                (self.cache_hits as f64 / self.total_evals as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Get total evaluation count.
    pub fn total_evaluations(&self) -> u64 {
        self.total_evals
    }

    /// Get session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Record a parse error.
    pub fn record_parse_error(&mut self) {
        self.parse_errors += 1;
        counter!("repl.errors.total", "type" => "parse").increment(1);
    }

    /// Record a compile error.
    pub fn record_compile_error(&mut self) {
        self.compile_errors += 1;
        counter!("repl.errors.total", "type" => "compile").increment(1);
    }

    /// Record a runtime error.
    pub fn record_runtime_error(&mut self) {
        self.runtime_errors += 1;
        counter!("repl.errors.total", "type" => "runtime").increment(1);
    }

    /// Get total error count.
    pub fn total_errors(&self) -> u64 {
        self.parse_errors + self.compile_errors + self.runtime_errors
    }

    /// Get error rate as percentage.
    pub fn error_rate(&self) -> f64 {
        let total = self.total_evals + self.total_errors();
        if total > 0 {
            (self.total_errors() as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Percentile statistics for a tier.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Percentiles {
    /// Median (50th percentile)
    pub p50: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Number of samples
    pub count: usize,
}

/// Cache statistics.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Hit rate as percentage (0-100)
    pub hit_rate: f64,
}

/// Snapshot of session statistics for protocol transport.
///
/// Contains all session-level evaluation metrics in a serializable form.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SessionStatsSnapshot {
    /// Session identifier
    pub session_id: String,
    /// Total evaluations performed
    pub total_evaluations: u64,
    /// Cache statistics
    pub cache: CacheStats,
    /// Tier 1 (Calculator) percentiles, if any samples exist
    pub tier1_percentiles: Option<Percentiles>,
    /// Tier 2 (CachedLoaded) percentiles, if any samples exist
    pub tier2_percentiles: Option<Percentiles>,
    /// Tier 3 (JustInTime) percentiles, if any samples exist
    pub tier3_percentiles: Option<Percentiles>,
    /// Parse error count
    pub parse_errors: u64,
    /// Compile error count
    pub compile_errors: u64,
    /// Runtime error count
    pub runtime_errors: u64,
    /// Average evaluation time across all tiers (milliseconds)
    pub average_eval_time_ms: f64,
}

impl EvalMetrics {
    /// Create a snapshot of current metrics for protocol transport.
    pub fn snapshot(&self) -> SessionStatsSnapshot {
        // Calculate average eval time across all tiers
        let all_samples: Vec<f64> = self
            .tier1_samples
            .iter()
            .chain(self.tier2_samples.iter())
            .chain(self.tier3_samples.iter())
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();

        let average_eval_time_ms = if !all_samples.is_empty() {
            all_samples.iter().sum::<f64>() / all_samples.len() as f64
        } else {
            0.0
        };

        SessionStatsSnapshot {
            session_id: self.session_id.clone(),
            total_evaluations: self.total_evals,
            cache: self.cache_stats(),
            tier1_percentiles: self.percentiles(ExecutionTier::Calculator),
            tier2_percentiles: self.percentiles(ExecutionTier::CachedLoaded),
            tier3_percentiles: self.percentiles(ExecutionTier::JustInTime),
            parse_errors: self.parse_errors,
            compile_errors: self.compile_errors,
            runtime_errors: self.runtime_errors,
            average_eval_time_ms,
        }
    }
}

/// Calculate percentile using linear interpolation.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }

    let index = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;

    if lower == upper {
        sorted[lower]
    } else {
        let fraction = index - lower as f64;
        sorted[lower] * (1.0 - fraction) + sorted[upper] * fraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_metrics_creation() {
        let metrics = EvalMetrics::new("test-session");
        assert_eq!(metrics.total_evaluations(), 0);
        assert_eq!(metrics.session_id(), "test-session");
    }

    #[test]
    fn test_record_tier1() {
        let mut metrics = EvalMetrics::new("test");
        metrics.record(ExecutionTier::Calculator, false, Duration::from_millis(1));

        assert_eq!(metrics.total_evaluations(), 1);
        assert_eq!(metrics.cache_stats().misses, 1);

        let p = metrics.percentiles(ExecutionTier::Calculator).unwrap();
        assert_eq!(p.count, 1);
        assert!((p.p50 - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_percentile_calculation() {
        let mut metrics = EvalMetrics::new("test");

        // Add samples: 1, 2, 3, 4, 5 ms
        for i in 1..=5 {
            metrics.record(ExecutionTier::Calculator, false, Duration::from_millis(i));
        }

        let p = metrics.percentiles(ExecutionTier::Calculator).unwrap();
        assert_eq!(p.count, 5);
        assert!((p.p50 - 3.0).abs() < 0.1); // Median should be 3
        assert!((p.min - 1.0).abs() < 0.1);
        assert!((p.max - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_sample_limit() {
        let mut metrics = EvalMetrics::new("test");

        // Add MAX_SAMPLES + 100 samples
        for i in 0..(MAX_SAMPLES + 100) {
            metrics.record(ExecutionTier::Calculator, false, Duration::from_millis(i as u64));
        }

        let p = metrics.percentiles(ExecutionTier::Calculator).unwrap();
        assert_eq!(p.count, MAX_SAMPLES); // Should cap at MAX_SAMPLES
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut metrics = EvalMetrics::new("test");

        // 7 hits, 3 misses
        for _ in 0..7 {
            metrics.record(ExecutionTier::CachedLoaded, true, Duration::from_millis(2));
        }
        for _ in 0..3 {
            metrics.record(ExecutionTier::JustInTime, false, Duration::from_millis(50));
        }

        let cache = metrics.cache_stats();
        assert_eq!(cache.hits, 7);
        assert_eq!(cache.misses, 3);
        assert!((cache.hit_rate - 70.0).abs() < 0.1);
    }

    #[test]
    fn test_percentile_single_value() {
        let mut metrics = EvalMetrics::new("test");
        metrics.record(ExecutionTier::Calculator, false, Duration::from_millis(5));

        let p = metrics.percentiles(ExecutionTier::Calculator).unwrap();
        assert_eq!(p.count, 1);
        assert!((p.p50 - 5.0).abs() < 0.1);
        assert!((p.p95 - 5.0).abs() < 0.1);
        assert!((p.p99 - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_empty_percentiles() {
        let metrics = EvalMetrics::new("test");
        assert!(metrics.percentiles(ExecutionTier::Calculator).is_none());
    }

    #[test]
    fn test_percentile_function() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&data, 0.0) - 1.0).abs() < 0.1);
        assert!((percentile(&data, 50.0) - 3.0).abs() < 0.1);
        assert!((percentile(&data, 100.0) - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_execution_tier_labels() {
        assert_eq!(ExecutionTier::Calculator.as_label(), "calculator");
        assert_eq!(ExecutionTier::CachedLoaded.as_label(), "cached");
        assert_eq!(ExecutionTier::JustInTime.as_label(), "jit");
    }

    #[test]
    fn test_execution_tier_display_names() {
        assert_eq!(ExecutionTier::Calculator.display_name(), "Calculator");
        assert_eq!(ExecutionTier::CachedLoaded.display_name(), "Cached");
        assert_eq!(ExecutionTier::JustInTime.display_name(), "JIT");
    }
}
