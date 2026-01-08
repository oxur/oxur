//! Statistics collection for REPL evaluation
//!
//! Provides timing and cache metrics collection with percentile calculation.
//! This module focuses on data collection only; display formatting is in oxur-cli.

use super::ExecutionTier;
use std::collections::VecDeque;
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

/// Maximum timing samples to keep per tier (for percentile calculation)
const MAX_SAMPLES: usize = 1000;

/// Statistics collector for REPL execution
///
/// Tracks execution timing samples and cache metrics per session.
/// Memory-bounded by MAX_SAMPLES limit per tier (~24KB total).
///
/// # Thread Safety
///
/// Designed to be wrapped in Arc<Mutex<>> for shared access from EvalContext.
#[derive(Debug, Clone)]
pub struct StatsCollector {
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
}

impl StatsCollector {
    /// Create a new stats collector for a session
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            tier1_samples: VecDeque::with_capacity(MAX_SAMPLES),
            tier2_samples: VecDeque::with_capacity(MAX_SAMPLES),
            tier3_samples: VecDeque::with_capacity(MAX_SAMPLES),
            cache_hits: 0,
            cache_misses: 0,
            total_evals: 0,
        }
    }

    /// Record an evaluation result
    ///
    /// Adds a timing sample to the appropriate tier and updates cache metrics.
    /// Uses a circular buffer pattern - oldest samples are evicted when MAX_SAMPLES is reached.
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
    }

    /// Calculate percentiles for a given tier
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

    /// Get cache statistics
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

    /// Get total evaluation count
    pub fn total_evaluations(&self) -> u64 {
        self.total_evals
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

/// Percentile statistics
#[derive(Debug, Clone)]
pub struct Percentiles {
    pub p50: f64, // Median
    pub p95: f64,
    pub p99: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64, // Percentage
}

/// Calculate percentile using linear interpolation
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

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// Process memory (RSS - Resident Set Size) in bytes
    pub process_memory_bytes: u64,

    /// Virtual memory size in bytes
    pub virtual_memory_bytes: u64,

    /// Process ID
    pub pid: u32,
}

/// Get current process resource usage
pub fn get_resource_stats() -> Option<ResourceStats> {
    let pid = std::process::id();
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new().with_memory()),
    );

    system.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());

    let sysinfo_pid = Pid::from_u32(pid);
    let process = system.process(sysinfo_pid)?;

    Some(ResourceStats {
        process_memory_bytes: process.memory(),
        virtual_memory_bytes: process.virtual_memory(),
        pid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_collector_creation() {
        let collector = StatsCollector::new("test-session");
        assert_eq!(collector.total_evaluations(), 0);
        assert_eq!(collector.session_id(), "test-session");
    }

    #[test]
    fn test_record_tier1() {
        let mut collector = StatsCollector::new("test");
        collector.record(ExecutionTier::Calculator, false, Duration::from_millis(1));

        assert_eq!(collector.total_evaluations(), 1);
        assert_eq!(collector.cache_stats().misses, 1);

        let p = collector.percentiles(ExecutionTier::Calculator).unwrap();
        assert_eq!(p.count, 1);
        assert!((p.p50 - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_percentile_calculation() {
        let mut collector = StatsCollector::new("test");

        // Add samples: 1, 2, 3, 4, 5 ms
        for i in 1..=5 {
            collector.record(ExecutionTier::Calculator, false, Duration::from_millis(i));
        }

        let p = collector.percentiles(ExecutionTier::Calculator).unwrap();
        assert_eq!(p.count, 5);
        assert!((p.p50 - 3.0).abs() < 0.1); // Median should be 3
        assert!((p.min - 1.0).abs() < 0.1);
        assert!((p.max - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_sample_limit() {
        let mut collector = StatsCollector::new("test");

        // Add MAX_SAMPLES + 100 samples
        for i in 0..(MAX_SAMPLES + 100) {
            collector.record(ExecutionTier::Calculator, false, Duration::from_millis(i as u64));
        }

        let p = collector.percentiles(ExecutionTier::Calculator).unwrap();
        assert_eq!(p.count, MAX_SAMPLES); // Should cap at MAX_SAMPLES
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut collector = StatsCollector::new("test");

        // 7 hits, 3 misses
        for _ in 0..7 {
            collector.record(ExecutionTier::CachedLoaded, true, Duration::from_millis(2));
        }
        for _ in 0..3 {
            collector.record(ExecutionTier::JustInTime, false, Duration::from_millis(50));
        }

        let cache = collector.cache_stats();
        assert_eq!(cache.hits, 7);
        assert_eq!(cache.misses, 3);
        assert!((cache.hit_rate - 70.0).abs() < 0.1);
    }

    #[test]
    fn test_percentile_single_value() {
        let mut collector = StatsCollector::new("test");
        collector.record(ExecutionTier::Calculator, false, Duration::from_millis(5));

        let p = collector.percentiles(ExecutionTier::Calculator).unwrap();
        assert_eq!(p.count, 1);
        assert!((p.p50 - 5.0).abs() < 0.1);
        assert!((p.p95 - 5.0).abs() < 0.1);
        assert!((p.p99 - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_empty_percentiles() {
        let collector = StatsCollector::new("test");
        assert!(collector.percentiles(ExecutionTier::Calculator).is_none());
    }

    #[test]
    fn test_percentile_function() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&data, 0.0) - 1.0).abs() < 0.1);
        assert!((percentile(&data, 50.0) - 3.0).abs() < 0.1);
        assert!((percentile(&data, 100.0) - 5.0).abs() < 0.1);
    }
}
