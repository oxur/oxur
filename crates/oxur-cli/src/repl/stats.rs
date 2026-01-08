//! Statistics display and formatting for REPL sessions
//!
//! Provides display functions for REPL statistics using OxurTable formatting.
//! Core data collection is in oxur-repl; this module handles presentation only.

use oxur_cli::table::{OxurTable, Tabled};
use oxur_repl::cache::CacheStats as ArtifactCacheStats;
use oxur_repl::eval::{get_resource_stats, ExecutionTier, StatsCollector};
use oxur_repl::session::DirStats;

// Display functions

/// Show session summary (default stats view)
pub fn show_session_summary(collector: &StatsCollector, color_enabled: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&header("Session Statistics", color_enabled));
    output.push('\n');

    // Overall summary
    output.push_str(&section("SUMMARY", color_enabled));
    output.push_str(&format!("Total Evaluations: {}\n", collector.total_evaluations()));

    let cache = collector.cache_stats();
    output.push_str(&format!(
        "Cache Hit Rate: {:.1}% ({} hits, {} misses)\n\n",
        cache.hit_rate, cache.hits, cache.misses
    ));

    // Execution tiers table
    output.push_str(&section("EXECUTION TIERS", color_enabled));

    #[derive(Tabled)]
    struct TierMetric {
        #[tabled(rename = "Tier")]
        tier: String,
        #[tabled(rename = "Count")]
        count: String,
        #[tabled(rename = "P50 (ms)")]
        p50: String,
        #[tabled(rename = "P95 (ms)")]
        p95: String,
        #[tabled(rename = "P99 (ms)")]
        p99: String,
    }

    let mut metrics = Vec::new();

    // Tier 1
    if let Some(p) = collector.percentiles(ExecutionTier::Calculator) {
        metrics.push(TierMetric {
            tier: "Calculator".to_string(),
            count: p.count.to_string(),
            p50: format!("{:.2}", p.p50),
            p95: format!("{:.2}", p.p95),
            p99: format!("{:.2}", p.p99),
        });
    }

    // Tier 2
    if let Some(p) = collector.percentiles(ExecutionTier::CachedLoaded) {
        metrics.push(TierMetric {
            tier: "Cached".to_string(),
            count: p.count.to_string(),
            p50: format!("{:.2}", p.p50),
            p95: format!("{:.2}", p.p95),
            p99: format!("{:.2}", p.p99),
        });
    }

    // Tier 3
    if let Some(p) = collector.percentiles(ExecutionTier::JustInTime) {
        metrics.push(TierMetric {
            tier: "JIT".to_string(),
            count: p.count.to_string(),
            p50: format!("{:.2}", p.p50),
            p95: format!("{:.2}", p.p95),
            p99: format!("{:.2}", p.p99),
        });
    }

    if !metrics.is_empty() {
        output.push_str(&OxurTable::new(metrics).render());
        output.push('\n');
    } else {
        output.push_str("No execution data yet.\n\n");
    }

    output
}

/// Show detailed execution breakdown
pub fn show_execution_details(collector: &StatsCollector, color_enabled: bool) -> String {
    let mut output = String::new();

    output.push_str(&header("Execution Statistics", color_enabled));
    output.push('\n');

    for tier in [
        ExecutionTier::Calculator,
        ExecutionTier::CachedLoaded,
        ExecutionTier::JustInTime,
    ] {
        if let Some(p) = collector.percentiles(tier) {
            output.push_str(&section(&tier_name(tier), color_enabled));

            #[derive(Tabled)]
            struct Metric {
                #[tabled(rename = "Metric")]
                metric: String,
                #[tabled(rename = "Value (ms)")]
                value: String,
            }

            let metrics = vec![
                Metric { metric: "Count".to_string(), value: p.count.to_string() },
                Metric { metric: "Min".to_string(), value: format!("{:.2}", p.min) },
                Metric { metric: "p50 (median)".to_string(), value: format!("{:.2}", p.p50) },
                Metric { metric: "p95".to_string(), value: format!("{:.2}", p.p95) },
                Metric { metric: "p99".to_string(), value: format!("{:.2}", p.p99) },
                Metric { metric: "Max".to_string(), value: format!("{:.2}", p.max) },
            ];

            output.push_str(&OxurTable::new(metrics).render());
            output.push('\n');
        }
    }

    output
}

/// Show cache statistics
pub fn show_cache_stats(collector: &StatsCollector, color_enabled: bool) -> String {
    let mut output = String::new();

    output.push_str(&header("Cache Statistics", color_enabled));
    output.push('\n');

    // Evaluation cache
    output.push_str(&section("EVALUATION CACHE", color_enabled));
    let cache = collector.cache_stats();

    #[derive(Tabled)]
    struct CacheMetric {
        #[tabled(rename = "Metric")]
        metric: String,
        #[tabled(rename = "Value")]
        value: String,
    }

    let metrics = vec![
        CacheMetric { metric: "Hits".to_string(), value: cache.hits.to_string() },
        CacheMetric { metric: "Misses".to_string(), value: cache.misses.to_string() },
        CacheMetric { metric: "Hit Rate".to_string(), value: format!("{:.1}%", cache.hit_rate) },
    ];

    output.push_str(&OxurTable::new(metrics).render());
    output.push('\n');

    output
}

/// Show resource usage statistics
pub fn show_resource_stats(
    dir_stats: Option<&DirStats>,
    cache_stats: Option<&ArtifactCacheStats>,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    output.push_str(&header("Resource Usage", color_enabled));
    output.push('\n');

    // Memory section
    output.push_str(&section("MEMORY", color_enabled));
    if let Some(resource_stats) = get_resource_stats() {
        #[derive(Tabled)]
        struct MemoryMetric {
            #[tabled(rename = "Metric")]
            metric: String,
            #[tabled(rename = "Value")]
            value: String,
        }

        let metrics = vec![
            MemoryMetric {
                metric: "Process RSS".to_string(),
                value: format_bytes(resource_stats.process_memory_bytes),
            },
            MemoryMetric {
                metric: "Virtual Memory".to_string(),
                value: format_bytes(resource_stats.virtual_memory_bytes),
            },
            MemoryMetric {
                metric: "Process ID".to_string(),
                value: resource_stats.pid.to_string(),
            },
        ];

        output.push_str(&OxurTable::new(metrics).render());
        output.push('\n');
    } else {
        output.push_str("Memory stats unavailable\n\n");
    }

    // Session directory section
    output.push_str(&section("SESSION DIRECTORY", color_enabled));
    if let Some(dir_stats) = dir_stats {
        let location_type = if dir_stats.is_tmpfs { " (tmpfs)" } else { "" };

        #[derive(Tabled)]
        struct DirMetric {
            #[tabled(rename = "Metric")]
            metric: String,
            #[tabled(rename = "Value")]
            value: String,
        }

        let metrics = vec![
            DirMetric {
                metric: "Location".to_string(),
                value: format!("{}{}", dir_stats.path.display(), location_type),
            },
            DirMetric {
                metric: "Files".to_string(),
                value: dir_stats.file_count.to_string(),
            },
            DirMetric {
                metric: "Disk Usage".to_string(),
                value: format_bytes(dir_stats.total_bytes),
            },
        ];

        output.push_str(&OxurTable::new(metrics).render());
        output.push('\n');
    } else {
        output.push_str("Session directory not initialized\n\n");
    }

    // Artifact cache section
    output.push_str(&section("ARTIFACT CACHE (Global)", color_enabled));
    if let Some(cache_stats) = cache_stats {
        #[derive(Tabled)]
        struct ArtifactMetric {
            #[tabled(rename = "Metric")]
            metric: String,
            #[tabled(rename = "Value")]
            value: String,
        }

        let age_seconds = if cache_stats.entry_count > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now.saturating_sub(cache_stats.oldest_entry_secs)
        } else {
            0
        };

        let metrics = vec![
            ArtifactMetric {
                metric: "Entries".to_string(),
                value: cache_stats.entry_count.to_string(),
            },
            ArtifactMetric {
                metric: "Total Size".to_string(),
                value: format_bytes(cache_stats.total_size_bytes),
            },
            ArtifactMetric {
                metric: "Oldest Entry".to_string(),
                value: if cache_stats.entry_count > 0 {
                    format_duration(age_seconds)
                } else {
                    "N/A".to_string()
                },
            },
            ArtifactMetric {
                metric: "Cache Directory".to_string(),
                value: cache_stats.cache_dir.display().to_string(),
            },
        ];

        output.push_str(&OxurTable::new(metrics).render());
        output.push('\n');
    } else {
        output.push_str("Artifact cache not initialized\n\n");
    }

    output
}

/// Parse stats commands
///
/// Recognizes:
/// - `(stats)` - Session summary
/// - `(stats execution)` - Detailed tier breakdown
/// - `(stats cache)` - Cache metrics
pub fn parse_stats_command(
    input: &str,
    collector: &StatsCollector,
    color_enabled: bool,
) -> Option<String> {
    if input == "(stats)" {
        return Some(show_session_summary(collector, color_enabled));
    }

    if input == "(stats execution)" {
        return Some(show_execution_details(collector, color_enabled));
    }

    if input == "(stats cache)" {
        return Some(show_cache_stats(collector, color_enabled));
    }

    None
}

/// Parse stats command with resource stats
///
/// Extended version that handles `(stats resources)` command
pub fn parse_stats_command_with_resources(
    input: &str,
    collector: &StatsCollector,
    dir_stats: Option<&DirStats>,
    cache_stats: Option<&ArtifactCacheStats>,
    color_enabled: bool,
) -> Option<String> {
    if input == "(stats resources)" {
        return Some(show_resource_stats(dir_stats, cache_stats, color_enabled));
    }

    // Fall back to regular stats commands
    parse_stats_command(input, collector, color_enabled)
}

// Helper functions

fn header(text: &str, color_enabled: bool) -> String {
    if color_enabled {
        format!("\x1b[1;36m{}\x1b[0m\n{}\n", text, "═".repeat(text.len()))
    } else {
        format!("{}\n{}\n", text, "=".repeat(text.len()))
    }
}

fn section(title: &str, color_enabled: bool) -> String {
    if color_enabled {
        format!("\x1b[1;36m{}\x1b[0m\n{}\n", title, "─".repeat(title.len()))
    } else {
        format!("{}\n{}\n", title, "-".repeat(title.len()))
    }
}

fn tier_name(tier: ExecutionTier) -> String {
    match tier {
        ExecutionTier::Calculator => "TIER 1: CALCULATOR (~1ms)".to_string(),
        ExecutionTier::CachedLoaded => "TIER 2: CACHED LOADED (~1-5ms)".to_string(),
        ExecutionTier::JustInTime => "TIER 3: JUST-IN-TIME (~50-300ms)".to_string(),
        _ => "UNKNOWN TIER".to_string(), // Handle future variants
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

fn format_duration(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;

    if seconds >= DAY {
        format!("{} days ago", seconds / DAY)
    } else if seconds >= HOUR {
        format!("{} hours ago", seconds / HOUR)
    } else if seconds >= MINUTE {
        format!("{} minutes ago", seconds / MINUTE)
    } else {
        format!("{} seconds ago", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_display_session_summary() {
        let mut collector = StatsCollector::new("test");
        collector.record(ExecutionTier::Calculator, false, Duration::from_millis(1));
        collector.record(ExecutionTier::CachedLoaded, true, Duration::from_millis(2));

        let output = show_session_summary(&collector, false);

        assert!(output.contains("Session Statistics"));
        assert!(output.contains("Total Evaluations: 2"));
        assert!(output.contains("Cache Hit Rate"));
    }

    #[test]
    fn test_display_execution_details() {
        let mut collector = StatsCollector::new("test");
        collector.record(ExecutionTier::Calculator, false, Duration::from_millis(1));
        collector.record(ExecutionTier::Calculator, false, Duration::from_millis(2));

        let output = show_execution_details(&collector, false);

        assert!(output.contains("Execution Statistics"));
        assert!(output.contains("TIER 1: CALCULATOR"));
    }

    #[test]
    fn test_display_cache_stats() {
        let mut collector = StatsCollector::new("test");
        collector.record(ExecutionTier::CachedLoaded, true, Duration::from_millis(2));

        let output = show_cache_stats(&collector, false);

        assert!(output.contains("Cache Statistics"));
        assert!(output.contains("EVALUATION CACHE"));
    }

    #[test]
    fn test_parse_stats_command_summary() {
        let collector = StatsCollector::new("test");

        let result = parse_stats_command("(stats)", &collector, false);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Session Statistics"));
    }

    #[test]
    fn test_parse_stats_command_execution() {
        let collector = StatsCollector::new("test");

        let result = parse_stats_command("(stats execution)", &collector, false);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Execution Statistics"));
    }

    #[test]
    fn test_parse_stats_command_cache() {
        let collector = StatsCollector::new("test");

        let result = parse_stats_command("(stats cache)", &collector, false);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Cache Statistics"));
    }

    #[test]
    fn test_parse_stats_command_invalid() {
        let collector = StatsCollector::new("test");

        let result = parse_stats_command("(stats invalid)", &collector, false);
        assert!(result.is_none());

        let result = parse_stats_command("(not-stats)", &collector, false);
        assert!(result.is_none());
    }
}
