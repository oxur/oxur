//! Statistics display and formatting for REPL sessions
//!
//! Provides display functions for REPL statistics using OxurTable formatting.
//! Core data collection is in oxur-repl; this module handles presentation only.

use crate::table::{OxurTable, Tabled};
use oxur_repl::cache::CacheStats as ArtifactCacheStats;
use oxur_repl::eval::{get_resource_stats, EvalMetrics, ExecutionTier};
use oxur_repl::metrics::SessionStatsSnapshot;
use oxur_repl::session::DirStats;

// Display functions

/// Show comprehensive statistics (all stats combined)
#[allow(clippy::too_many_arguments)]
pub fn show_all_stats(
    collector: &EvalMetrics,
    dir_stats: Option<&DirStats>,
    cache_stats: Option<&ArtifactCacheStats>,
    server_snapshot: Option<&oxur_repl::metrics::ServerMetricsSnapshot>,
    client_snapshot: Option<&oxur_repl::metrics::ClientMetricsSnapshot>,
    subprocess_snapshot: Option<&oxur_repl::metrics::SubprocessMetricsSnapshot>,
    usage_snapshot: Option<&oxur_repl::metrics::UsageMetricsSnapshot>,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    // 1. Execution Statistics (full detailed view)
    output.push_str(&show_execution_details(collector, color_enabled));

    // 2. Cache Statistics (full view)
    output.push_str(&show_cache_stats(collector, color_enabled));

    // 3. Resource Usage (full view if available)
    if dir_stats.is_some() || cache_stats.is_some() {
        output.push('\n');
        output.push_str(&show_resource_stats(dir_stats, cache_stats, color_enabled));
    }

    // 4. Client Statistics (full view if available)
    if let Some(client) = client_snapshot {
        output.push_str(&show_client_stats(client, color_enabled));
    }

    // 5. Usage Statistics (full view if available)
    if let Some(usage) = usage_snapshot {
        output.push('\n');
        output.push_str(&show_usage_stats(usage, color_enabled));
    }

    // 6. Subprocess Statistics (full view if available)
    if let Some(subprocess) = subprocess_snapshot {
        output.push_str(&show_subprocess_stats(subprocess, color_enabled));
    }

    // 7. Server Statistics (full view if available)
    if let Some(server) = server_snapshot {
        output.push('\n');
        output.push_str(&show_server_stats(server, color_enabled));
    }

    output
}

/// Show detailed execution breakdown
pub fn show_execution_details(collector: &EvalMetrics, color_enabled: bool) -> String {
    let mut output = String::new();

    output.push_str(&header("Execution Statistics", color_enabled));
    output.push('\n');

    for tier in [ExecutionTier::Calculator, ExecutionTier::CachedLoaded, ExecutionTier::JustInTime]
    {
        if let Some(p) = collector.percentiles(tier) {
            #[derive(Tabled)]
            struct Metric {
                #[tabled(rename = "Metric")]
                metric: String,
                #[tabled(rename = "Value (ms) ")]
                value: String,
            }

            let metrics = vec![
                Metric { metric: " Count ".to_string(), value: format!(" {} ", p.count) },
                Metric { metric: " Min ".to_string(), value: format!(" {:.2} ", p.min) },
                Metric { metric: " p50 (median) ".to_string(), value: format!(" {:.2} ", p.p50) },
                Metric { metric: " p95 ".to_string(), value: format!(" {:.2} ", p.p95) },
                Metric { metric: " p99 ".to_string(), value: format!(" {:.2} ", p.p99) },
                Metric { metric: " Max ".to_string(), value: format!(" {:.2} ", p.max) },
            ];

            output.push_str(
                &OxurTable::new(metrics).with_title(tier_name(tier)).with_footer().render(),
            );
            output.push_str("\n\n");
        }
    }

    output
}

/// Show cache statistics
pub fn show_cache_stats(collector: &EvalMetrics, color_enabled: bool) -> String {
    let mut output = String::new();

    output.push_str(&header("Cache Statistics", color_enabled));
    output.push('\n');

    // Evaluation cache
    let cache = collector.cache_stats();

    #[derive(Tabled)]
    struct CacheMetric {
        #[tabled(rename = "Metric")]
        metric: String,
        #[tabled(rename = "Value ")]
        value: String,
    }

    let metrics = vec![
        CacheMetric { metric: " Hits ".to_string(), value: format!(" {} ", cache.hits) },
        CacheMetric { metric: " Misses ".to_string(), value: format!(" {} ", cache.misses) },
        CacheMetric {
            metric: " Hit Rate ".to_string(),
            value: format!(" {:.1}% ", cache.hit_rate),
        },
    ];

    output.push_str(&OxurTable::new(metrics).with_title("EVALUATION CACHE").with_footer().render());
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
    if let Some(resource_stats) = get_resource_stats() {
        #[derive(Tabled)]
        struct MemoryMetric {
            #[tabled(rename = "Metric")]
            metric: String,
            #[tabled(rename = "Value ")]
            value: String,
        }

        let metrics = vec![
            MemoryMetric {
                metric: " Process RSS ".to_string(),
                value: format!(" {} ", format_bytes(resource_stats.process_memory_bytes)),
            },
            MemoryMetric {
                metric: " Virtual Memory ".to_string(),
                value: format!(" {} ", format_bytes(resource_stats.virtual_memory_bytes)),
            },
            MemoryMetric {
                metric: " Process ID ".to_string(),
                value: format!(" {} ", resource_stats.pid),
            },
        ];

        output.push_str(&OxurTable::new(metrics).with_title("MEMORY").with_footer().render());
        output.push_str("\n\n");
    } else {
        output.push_str("Memory stats unavailable\n\n");
    }

    // Session directory section
    if let Some(dir_stats) = dir_stats {
        let location_type = if dir_stats.is_tmpfs { " (tmpfs)" } else { "" };

        #[derive(Tabled)]
        struct DirMetric {
            #[tabled(rename = "Metric")]
            metric: String,
            #[tabled(rename = "Value ")]
            value: String,
        }

        let metrics = vec![
            DirMetric {
                metric: " Location ".to_string(),
                value: format!(" {}{} ", dir_stats.path.display(), location_type),
            },
            DirMetric {
                metric: " Files ".to_string(),
                value: format!(" {} ", dir_stats.file_count),
            },
            DirMetric {
                metric: " Disk Usage ".to_string(),
                value: format!(" {} ", format_bytes(dir_stats.total_bytes)),
            },
        ];

        output.push_str(
            &OxurTable::new(metrics).with_title("SESSION DIRECTORY").with_footer().render(),
        );
        output.push_str("\n\n");
    } else {
        output.push_str("Session directory not initialized\n\n");
    }

    // Artifact cache section
    if let Some(cache_stats) = cache_stats {
        #[derive(Tabled)]
        struct ArtifactMetric {
            #[tabled(rename = "Metric")]
            metric: String,
            #[tabled(rename = "Value ")]
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
                metric: " Entries ".to_string(),
                value: format!(" {} ", cache_stats.entry_count),
            },
            ArtifactMetric {
                metric: " Total Size ".to_string(),
                value: format!(" {} ", format_bytes(cache_stats.total_size_bytes)),
            },
            ArtifactMetric {
                metric: " Oldest Entry ".to_string(),
                value: if cache_stats.entry_count > 0 {
                    format!(" {} ", format_duration(age_seconds))
                } else {
                    " N/A ".to_string()
                },
            },
            ArtifactMetric {
                metric: " Cache Directory ".to_string(),
                value: format!(" {} ", cache_stats.cache_dir.display()),
            },
        ];

        output.push_str(
            &OxurTable::new(metrics).with_title("ARTIFACT CACHE (Global)").with_footer().render(),
        );
        output.push_str("\n\n");
    } else {
        output.push_str("Artifact cache not initialized\n\n");
    }

    output
}

/// Show all sessions with their statistics
pub fn show_sessions(
    sessions: &[oxur_repl::server::SessionInfo],
    current_session_id: &oxur_repl::protocol::SessionId,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    output.push_str(&header("Sessions", color_enabled));
    output.push('\n');

    if sessions.is_empty() {
        output.push_str("No active sessions\n");
        return output;
    }

    #[derive(Tabled)]
    struct SessionRow {
        #[tabled(rename = "ID")]
        id: String,
        #[tabled(rename = "Name")]
        name: String,
        #[tabled(rename = "Active")]
        active: String,
        #[tabled(rename = "Evals")]
        evals: String,
        #[tabled(rename = "Last Active")]
        last_active: String,
    }

    let rows: Vec<SessionRow> = sessions
        .iter()
        .map(|s| {
            let is_current = s.id == *current_session_id;
            let active_marker = if is_current { " * " } else { " " };

            // Format last active time
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let elapsed_ms = now.saturating_sub(s.last_active_at);
            let last_active = if elapsed_ms < 60_000 {
                " just now ".to_string()
            } else if elapsed_ms < 3_600_000 {
                format!(" {} min ago ", elapsed_ms / 60_000)
            } else if elapsed_ms < 86_400_000 {
                format!(" {} hr ago ", elapsed_ms / 3_600_000)
            } else {
                format!(" {} days ago ", elapsed_ms / 86_400_000)
            };

            SessionRow {
                id: format!(" {} ", s.id),
                name: format!(" {} ", s.name.clone().unwrap_or_else(|| "-".to_string())),
                active: active_marker.to_string(),
                evals: format!(" {} ", s.eval_count),
                last_active,
            }
        })
        .collect();

    output.push_str(&OxurTable::new(rows).with_title("ACTIVE SESSIONS").with_footer().render());
    output.push('\n');

    output
}

/// Show usage metrics
pub fn show_usage_stats(
    usage_snapshot: &oxur_repl::metrics::UsageMetricsSnapshot,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    output.push_str(&header("Usage Statistics", color_enabled));
    output.push('\n');

    #[derive(Tabled)]
    struct CommandMetric {
        #[tabled(rename = "Command")]
        command: String,
        #[tabled(rename = "Count")]
        count: String,
        #[tabled(rename = "Percentage")]
        percentage: String,
    }

    let total = usage_snapshot.total_commands as f64;
    let calc_pct = |count: u64| {
        if total > 0.0 {
            format!(" {:.1}% ", (count as f64 / total) * 100.0)
        } else {
            " 0.0% ".to_string()
        }
    };

    let mut metrics = vec![
        CommandMetric {
            command: " Eval ".to_string(),
            count: format!(" {} ", usage_snapshot.eval_count),
            percentage: calc_pct(usage_snapshot.eval_count),
        },
        CommandMetric {
            command: " Help ".to_string(),
            count: format!(" {} ", usage_snapshot.help_count),
            percentage: calc_pct(usage_snapshot.help_count),
        },
        CommandMetric {
            command: " Stats ".to_string(),
            count: format!(" {} ", usage_snapshot.stats_count),
            percentage: calc_pct(usage_snapshot.stats_count),
        },
        CommandMetric {
            command: " Info ".to_string(),
            count: format!(" {} ", usage_snapshot.info_count),
            percentage: calc_pct(usage_snapshot.info_count),
        },
        CommandMetric {
            command: " Sessions ".to_string(),
            count: format!(" {} ", usage_snapshot.sessions_count),
            percentage: calc_pct(usage_snapshot.sessions_count),
        },
        CommandMetric {
            command: " Clear ".to_string(),
            count: format!(" {} ", usage_snapshot.clear_count),
            percentage: calc_pct(usage_snapshot.clear_count),
        },
        CommandMetric {
            command: " Banner ".to_string(),
            count: format!(" {} ", usage_snapshot.banner_count),
            percentage: calc_pct(usage_snapshot.banner_count),
        },
    ];

    // Add total as a footer-like row
    metrics.push(CommandMetric {
        command: " Total Commands: ".to_string(),
        count: format!(" {} ", usage_snapshot.total_commands),
        percentage: " ".to_string(),
    });

    output
        .push_str(&OxurTable::new(metrics).with_title("COMMAND FREQUENCY").with_footer().render());
    output.push_str("\n\n");

    output
}

/// Show client metrics
pub fn show_client_stats(
    client_snapshot: &oxur_repl::metrics::ClientMetricsSnapshot,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    output.push_str(&header("Client Statistics", color_enabled));
    output.push('\n');

    #[derive(Tabled)]
    struct RequestMetric {
        #[tabled(rename = "Metric")]
        metric: String,
        #[tabled(rename = "Value ")]
        value: String,
    }

    // Request/Response stats
    let metrics = vec![
        RequestMetric {
            metric: " Total Requests ".to_string(),
            value: format!(" {} ", client_snapshot.requests_total),
        },
        RequestMetric {
            metric: " Total Responses ".to_string(),
            value: format!(" {} ", client_snapshot.responses_total),
        },
        RequestMetric {
            metric: " Success Responses ".to_string(),
            value: format!(" {} ", client_snapshot.responses_success),
        },
        RequestMetric {
            metric: " Error Responses ".to_string(),
            value: format!(" {} ", client_snapshot.responses_error),
        },
    ];

    output.push_str(
        &OxurTable::new(metrics).with_title("REQUESTS & RESPONSES").with_footer().render(),
    );
    output.push_str("\n\n");

    #[derive(Tabled)]
    struct LatencyMetric {
        #[tabled(rename = "Metric")]
        metric: String,
        #[tabled(rename = "Value (ms)")]
        value: String,
    }

    // Latency stats
    let latency_metrics = vec![
        LatencyMetric {
            metric: " Average ".to_string(),
            value: format!(" {:.2} ", client_snapshot.average_latency_ms),
        },
        LatencyMetric {
            metric: " P50 ".to_string(),
            value: format!(" {:.2} ", client_snapshot.p50_latency_ms),
        },
        LatencyMetric {
            metric: " P95 ".to_string(),
            value: format!(" {:.2} ", client_snapshot.p95_latency_ms),
        },
        LatencyMetric {
            metric: " P99 ".to_string(),
            value: format!(" {:.2} ", client_snapshot.p99_latency_ms),
        },
        LatencyMetric {
            metric: " Min ".to_string(),
            value: format!(" {:.2} ", client_snapshot.min_latency_ms),
        },
        LatencyMetric {
            metric: " Max ".to_string(),
            value: format!(" {:.2} ", client_snapshot.max_latency_ms),
        },
    ];

    output.push_str(
        &OxurTable::new(latency_metrics).with_title("LATENCY DISTRIBUTION").with_footer().render(),
    );
    output.push('\n');

    output
}

/// Show server metrics
pub fn show_server_stats(
    server_snapshot: &oxur_repl::metrics::ServerMetricsSnapshot,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    output.push_str(&header("Server Statistics", color_enabled));
    output.push('\n');

    #[derive(Tabled)]
    struct ConnectionMetric {
        #[tabled(rename = "Metric")]
        metric: String,
        #[tabled(rename = "Value ")]
        value: String,
    }

    // Connection stats
    let metrics = vec![
        ConnectionMetric {
            metric: " Total Connections ".to_string(),
            value: format!(" {} ", server_snapshot.connections_total),
        },
        ConnectionMetric {
            metric: " Active Connections ".to_string(),
            value: format!(" {} ", server_snapshot.connections_active),
        },
    ];

    output.push_str(&OxurTable::new(metrics).with_title("CONNECTIONS").with_footer().render());
    output.push_str("\n\n");

    // Session stats
    let metrics = vec![
        ConnectionMetric {
            metric: " Total Sessions ".to_string(),
            value: format!(" {} ", server_snapshot.sessions_total),
        },
        ConnectionMetric {
            metric: " Active Sessions ".to_string(),
            value: format!(" {} ", server_snapshot.sessions_active),
        },
    ];

    output.push_str(&OxurTable::new(metrics).with_title("SESSIONS").with_footer().render());
    output.push_str("\n\n");

    // Request/Response stats
    let success_rate = if server_snapshot.responses_total > 0 {
        (server_snapshot.responses_success as f64 / server_snapshot.responses_total as f64) * 100.0
    } else {
        0.0
    };

    let metrics = vec![
        ConnectionMetric {
            metric: " Total Requests ".to_string(),
            value: format!(" {} ", server_snapshot.requests_total),
        },
        ConnectionMetric {
            metric: " Total Responses ".to_string(),
            value: format!(" {} ", server_snapshot.responses_total),
        },
        ConnectionMetric {
            metric: " Successful ".to_string(),
            value: format!(" {} ", server_snapshot.responses_success),
        },
        ConnectionMetric {
            metric: " Errors ".to_string(),
            value: format!(" {} ", server_snapshot.responses_error),
        },
        ConnectionMetric {
            metric: " Success Rate ".to_string(),
            value: format!(" {:.1}% ", success_rate),
        },
    ];

    output.push_str(
        &OxurTable::new(metrics).with_title("REQUESTS & RESPONSES").with_footer().render(),
    );
    output.push_str("\n\n");

    output
}

/// Show subprocess metrics
pub fn show_subprocess_stats(
    subprocess_snapshot: &oxur_repl::metrics::SubprocessMetricsSnapshot,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    output.push_str(&header("Subprocess Statistics", color_enabled));
    output.push('\n');

    #[derive(Tabled)]
    struct SubprocessMetric {
        #[tabled(rename = "Metric")]
        metric: String,
        #[tabled(rename = "Value ")]
        value: String,
    }

    let status = if subprocess_snapshot.is_running { "Running" } else { "Stopped" };
    let uptime = format_uptime_seconds(subprocess_snapshot.uptime_seconds);
    let last_reason = subprocess_snapshot
        .last_restart_reason
        .map(|r| r.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let metrics = vec![
        SubprocessMetric { metric: " Status ".to_string(), value: format!(" {} ", status) },
        SubprocessMetric { metric: " Uptime ".to_string(), value: format!(" {} ", uptime) },
        SubprocessMetric {
            metric: " Restart Count ".to_string(),
            value: format!(" {} ", subprocess_snapshot.restart_count),
        },
        SubprocessMetric {
            metric: " Last Restart Reason ".to_string(),
            value: format!(" {} ", last_reason),
        },
    ];

    output.push_str(&OxurTable::new(metrics).with_title("STATUS").with_footer().render());
    output.push_str("\n\n");

    output
}

// ============================================================================
// Snapshot-based display functions (for remote/protocol mode)
// ============================================================================

/// Show session summary from snapshot (for remote mode)
///
/// Uses the serialized SessionStatsSnapshot instead of direct EvalMetrics access.
pub fn show_session_summary_from_snapshot(
    snapshot: &SessionStatsSnapshot,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&header("Session Statistics", color_enabled));
    output.push('\n');

    // Overall summary
    output.push_str(&section("SUMMARY", color_enabled));
    output.push_str(&format!("Total Evaluations: {}\n", snapshot.total_evaluations));
    output.push_str(&format!(
        "Cache Hit Rate: {:.1}% ({} hits, {} misses)\n\n",
        snapshot.cache.hit_rate, snapshot.cache.hits, snapshot.cache.misses
    ));

    // Execution tiers table
    #[derive(Tabled)]
    struct TierMetric {
        #[tabled(rename = "Tier")]
        tier: String,
        #[tabled(rename = "Count ")]
        count: String,
        #[tabled(rename = "P50 (ms)")]
        p50: String,
        #[tabled(rename = "P95 (ms) ")]
        p95: String,
        #[tabled(rename = "P99 (ms) ")]
        p99: String,
    }

    let mut metrics = Vec::new();

    // Tier 1
    if let Some(ref p) = snapshot.tier1_percentiles {
        metrics.push(TierMetric {
            tier: " Calculator ".to_string(),
            count: format!(" {} ", p.count),
            p50: format!(" {:.2} ", p.p50),
            p95: format!(" {:.2} ", p.p95),
            p99: format!(" {:.2} ", p.p99),
        });
    }

    // Tier 2
    if let Some(ref p) = snapshot.tier2_percentiles {
        metrics.push(TierMetric {
            tier: " Cached ".to_string(),
            count: format!(" {} ", p.count),
            p50: format!(" {:.2} ", p.p50),
            p95: format!(" {:.2} ", p.p95),
            p99: format!(" {:.2} ", p.p99),
        });
    }

    // Tier 3
    if let Some(ref p) = snapshot.tier3_percentiles {
        metrics.push(TierMetric {
            tier: " JIT ".to_string(),
            count: format!(" {} ", p.count),
            p50: format!(" {:.2} ", p.p50),
            p95: format!(" {:.2} ", p.p95),
            p99: format!(" {:.2} ", p.p99),
        });
    }

    if !metrics.is_empty() {
        output.push_str(
            &OxurTable::new(metrics).with_title("EXECUTION TIERS").with_footer().render(),
        );
        output.push('\n');
    } else {
        output.push_str("No execution data yet.\n\n");
    }

    output
}

/// Show detailed execution breakdown from snapshot (for remote mode)
pub fn show_execution_from_snapshot(
    snapshot: &SessionStatsSnapshot,
    color_enabled: bool,
) -> String {
    let mut output = String::new();

    output.push_str(&header("Execution Statistics", color_enabled));
    output.push('\n');

    // Helper to display a tier's percentiles
    let display_tier =
        |output: &mut String, name: &str, percentiles: &Option<oxur_repl::metrics::Percentiles>| {
            if let Some(ref p) = percentiles {
                #[derive(Tabled)]
                struct Metric {
                    #[tabled(rename = "Metric")]
                    metric: String,
                    #[tabled(rename = "Value (ms)")]
                    value: String,
                }

                let metrics = vec![
                    Metric { metric: " Count ".to_string(), value: format!(" {} ", p.count) },
                    Metric { metric: " Min ".to_string(), value: format!(" {:.2} ", p.min) },
                    Metric {
                        metric: " p50 (median) ".to_string(),
                        value: format!(" {:.2} ", p.p50),
                    },
                    Metric { metric: " p95 ".to_string(), value: format!(" {:.2} ", p.p95) },
                    Metric { metric: " p99 ".to_string(), value: format!(" {:.2} ", p.p99) },
                    Metric { metric: " Max ".to_string(), value: format!(" {:.2} ", p.max) },
                ];

                output.push_str(&OxurTable::new(metrics).with_title(name).with_footer().render());
                output.push_str("\n\n");
            }
        };

    display_tier(&mut output, "TIER 1: CALCULATOR (~1ms)", &snapshot.tier1_percentiles);
    display_tier(&mut output, "TIER 2: CACHED LOADED (~1-5ms)", &snapshot.tier2_percentiles);
    display_tier(&mut output, "TIER 3: JUST-IN-TIME (~50-300ms)", &snapshot.tier3_percentiles);

    output
}

/// Show cache statistics from snapshot (for remote mode)
pub fn show_cache_from_snapshot(snapshot: &SessionStatsSnapshot, color_enabled: bool) -> String {
    let mut output = String::new();

    output.push_str(&header("Cache Statistics", color_enabled));
    output.push('\n');

    // Evaluation cache
    #[derive(Tabled)]
    struct CacheMetric {
        #[tabled(rename = "Metric")]
        metric: String,
        #[tabled(rename = "Value")]
        value: String,
    }

    let metrics = vec![
        CacheMetric { metric: " Hits ".to_string(), value: format!(" {} ", snapshot.cache.hits) },
        CacheMetric {
            metric: " Misses ".to_string(),
            value: format!(" {} ", snapshot.cache.misses),
        },
        CacheMetric {
            metric: " Hit Rate ".to_string(),
            value: format!(" {:.1}% ", snapshot.cache.hit_rate),
        },
    ];

    output.push_str(&OxurTable::new(metrics).with_title("EVALUATION CACHE").with_footer().render());
    output.push('\n');

    output
}

// ============================================================================
// Stats command parsing
// ============================================================================

/// Parse stats commands
///
/// Recognizes:
/// - `(stats)` - Session summary
/// - `(stats execution)` - Detailed tier breakdown
/// - `(stats cache)` - Cache metrics
pub fn parse_stats_command(
    input: &str,
    collector: &EvalMetrics,
    color_enabled: bool,
) -> Option<String> {
    if input == "(stats)" {
        return Some(show_all_stats(collector, None, None, None, None, None, None, color_enabled));
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
    collector: &EvalMetrics,
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

fn format_uptime_seconds(seconds: f64) -> String {
    let secs = seconds as u64;
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;

    if secs >= DAY {
        let days = secs / DAY;
        let hours = (secs % DAY) / HOUR;
        format!("{}d {}h", days, hours)
    } else if secs >= HOUR {
        let hours = secs / HOUR;
        let mins = (secs % HOUR) / MINUTE;
        format!("{}h {}m", hours, mins)
    } else if secs >= MINUTE {
        let mins = secs / MINUTE;
        let s = secs % MINUTE;
        format!("{}m {}s", mins, s)
    } else {
        format!("{:.1}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_display_session_summary() {
        let mut collector = EvalMetrics::new("test");
        collector.record(ExecutionTier::Calculator, false, Duration::from_millis(1));
        collector.record(ExecutionTier::CachedLoaded, true, Duration::from_millis(2));

        let output = show_all_stats(&collector, None, None, None, None, None, None, false);

        // Should contain all the major sections
        assert!(output.contains("Execution Statistics"));
        assert!(output.contains("Cache Statistics"));
        assert!(output.contains("TIER 1: CALCULATOR"));
        assert!(output.contains("EVALUATION CACHE"));
    }

    #[test]
    fn test_display_execution_details() {
        let mut collector = EvalMetrics::new("test");
        collector.record(ExecutionTier::Calculator, false, Duration::from_millis(1));
        collector.record(ExecutionTier::Calculator, false, Duration::from_millis(2));

        let output = show_execution_details(&collector, false);

        assert!(output.contains("Execution Statistics"));
        assert!(output.contains("TIER 1: CALCULATOR"));
    }

    #[test]
    fn test_display_cache_stats() {
        let mut collector = EvalMetrics::new("test");
        collector.record(ExecutionTier::CachedLoaded, true, Duration::from_millis(2));

        let output = show_cache_stats(&collector, false);

        assert!(output.contains("Cache Statistics"));
        assert!(output.contains("EVALUATION CACHE"));
    }

    #[test]
    fn test_parse_stats_command_summary() {
        let collector = EvalMetrics::new("test");

        let result = parse_stats_command("(stats)", &collector, false);
        assert!(result.is_some());
        // Should contain multiple sections
        let output = result.unwrap();
        assert!(output.contains("Execution Statistics"));
        assert!(output.contains("Cache Statistics"));
    }

    #[test]
    fn test_parse_stats_command_execution() {
        let collector = EvalMetrics::new("test");

        let result = parse_stats_command("(stats execution)", &collector, false);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Execution Statistics"));
    }

    #[test]
    fn test_parse_stats_command_cache() {
        let collector = EvalMetrics::new("test");

        let result = parse_stats_command("(stats cache)", &collector, false);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Cache Statistics"));
    }

    #[test]
    fn test_parse_stats_command_invalid() {
        let collector = EvalMetrics::new("test");

        let result = parse_stats_command("(stats invalid)", &collector, false);
        assert!(result.is_none());

        let result = parse_stats_command("(not-stats)", &collector, false);
        assert!(result.is_none());
    }

    // ===== Helper function tests =====

    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(0), "0 bytes");
        assert_eq!(format_bytes(100), "100 bytes");
        assert_eq!(format_bytes(1023), "1023 bytes");
    }

    #[test]
    fn test_format_bytes_kilobytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(2048), "2.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024 - 1), "1024.00 KB");
    }

    #[test]
    fn test_format_bytes_megabytes() {
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(2 * 1024 * 1024), "2.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 - 1), "1024.00 MB");
    }

    #[test]
    fn test_format_bytes_gigabytes() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2.00 GB");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(0), "0 seconds ago");
        assert_eq!(format_duration(30), "30 seconds ago");
        assert_eq!(format_duration(59), "59 seconds ago");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60), "1 minutes ago");
        assert_eq!(format_duration(120), "2 minutes ago");
        assert_eq!(format_duration(3599), "59 minutes ago");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600), "1 hours ago");
        assert_eq!(format_duration(7200), "2 hours ago");
        assert_eq!(format_duration(86399), "23 hours ago");
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(86400), "1 days ago");
        assert_eq!(format_duration(172800), "2 days ago");
        assert_eq!(format_duration(604800), "7 days ago");
    }

    #[test]
    fn test_format_uptime_seconds_subsecond() {
        assert_eq!(format_uptime_seconds(0.5), "0.5s");
        assert_eq!(format_uptime_seconds(30.5), "30.5s");
        assert_eq!(format_uptime_seconds(59.9), "59.9s");
    }

    #[test]
    fn test_format_uptime_seconds_minutes() {
        assert_eq!(format_uptime_seconds(60.0), "1m 0s");
        assert_eq!(format_uptime_seconds(90.0), "1m 30s");
        assert_eq!(format_uptime_seconds(3599.0), "59m 59s");
    }

    #[test]
    fn test_format_uptime_seconds_hours() {
        assert_eq!(format_uptime_seconds(3600.0), "1h 0m");
        assert_eq!(format_uptime_seconds(5400.0), "1h 30m");
        assert_eq!(format_uptime_seconds(86399.0), "23h 59m");
    }

    #[test]
    fn test_format_uptime_seconds_days() {
        assert_eq!(format_uptime_seconds(86400.0), "1d 0h");
        assert_eq!(format_uptime_seconds(129600.0), "1d 12h");
        assert_eq!(format_uptime_seconds(172800.0), "2d 0h");
    }

    #[test]
    fn test_header_with_color() {
        let result = header("Test", true);
        assert!(result.contains("Test"));
        assert!(result.contains("\x1b[1;36m")); // cyan color code
        assert!(result.contains("\x1b[0m")); // reset code
        assert!(result.contains("═")); // unicode equals
    }

    #[test]
    fn test_header_without_color() {
        let result = header("Test", false);
        assert!(result.contains("Test"));
        assert!(!result.contains("\x1b")); // no escape codes
        assert!(result.contains("=")); // ASCII equals
    }

    #[test]
    fn test_section_with_color() {
        let result = section("Section", true);
        assert!(result.contains("Section"));
        assert!(result.contains("\x1b[1;36m")); // cyan color code
        assert!(result.contains("─")); // unicode dash
    }

    #[test]
    fn test_section_without_color() {
        let result = section("Section", false);
        assert!(result.contains("Section"));
        assert!(!result.contains("\x1b")); // no escape codes
        assert!(result.contains("-")); // ASCII dash
    }

    #[test]
    fn test_tier_name_calculator() {
        let name = tier_name(ExecutionTier::Calculator);
        assert!(name.contains("CALCULATOR"));
        assert!(name.contains("TIER 1"));
    }

    #[test]
    fn test_tier_name_cached() {
        let name = tier_name(ExecutionTier::CachedLoaded);
        assert!(name.contains("CACHED"));
        assert!(name.contains("TIER 2"));
    }

    #[test]
    fn test_tier_name_jit() {
        let name = tier_name(ExecutionTier::JustInTime);
        assert!(name.contains("JUST-IN-TIME"));
        assert!(name.contains("TIER 3"));
    }

    // ===== Display function tests =====

    #[test]
    fn test_show_resource_stats_no_data() {
        let output = show_resource_stats(None, None, false);
        // When no data is provided, should show messages about unavailability
        assert!(output.contains("Resource Usage"));
        assert!(output.contains("Session directory not initialized"));
        assert!(output.contains("Artifact cache not initialized"));
    }

    #[test]
    fn test_show_sessions_empty() {
        let sessions: Vec<oxur_repl::server::SessionInfo> = vec![];
        let current = oxur_repl::protocol::SessionId::new("test");

        let output = show_sessions(&sessions, &current, false);
        assert!(output.contains("Sessions"));
        assert!(output.contains("No active sessions"));
    }

    #[test]
    fn test_show_usage_stats() {
        let usage = oxur_repl::metrics::UsageMetricsSnapshot {
            session_id: "test".to_string(),
            eval_count: 10,
            help_count: 5,
            stats_count: 3,
            info_count: 2,
            sessions_count: 1,
            clear_count: 0,
            banner_count: 1,
            total_commands: 22,
        };

        let output = show_usage_stats(&usage, false);
        assert!(output.contains("Usage Statistics"));
        assert!(output.contains("COMMAND FREQUENCY"));
        assert!(output.contains("Eval"));
        assert!(output.contains("10"));
    }

    #[test]
    fn test_show_client_stats() {
        let client = oxur_repl::metrics::ClientMetricsSnapshot {
            requests_total: 100,
            responses_total: 98,
            responses_success: 95,
            responses_error: 3,
            average_latency_ms: 10.5,
            p50_latency_ms: 8.0,
            p95_latency_ms: 25.0,
            p99_latency_ms: 50.0,
            min_latency_ms: 1.0,
            max_latency_ms: 100.0,
        };

        let output = show_client_stats(&client, false);
        assert!(output.contains("Client Statistics"));
        assert!(output.contains("REQUESTS & RESPONSES"));
        assert!(output.contains("LATENCY DISTRIBUTION"));
        assert!(output.contains("100")); // requests_total
    }

    #[test]
    fn test_show_server_stats() {
        let server = oxur_repl::metrics::ServerMetricsSnapshot {
            connections_total: 50,
            connections_active: 2,
            sessions_total: 30,
            sessions_active: 5,
            requests_total: 1000,
            responses_total: 998,
            responses_success: 990,
            responses_error: 8,
        };

        let output = show_server_stats(&server, false);
        assert!(output.contains("Server Statistics"));
        assert!(output.contains("CONNECTIONS"));
        assert!(output.contains("SESSIONS"));
        assert!(output.contains("Success Rate"));
    }

    #[test]
    fn test_show_subprocess_stats_running() {
        let subprocess = oxur_repl::metrics::SubprocessMetricsSnapshot {
            is_running: true,
            uptime_seconds: 3600.0,
            restart_count: 2,
            last_restart_reason: None,
        };

        let output = show_subprocess_stats(&subprocess, false);
        assert!(output.contains("Subprocess Statistics"));
        assert!(output.contains("Running"));
        assert!(output.contains("1h 0m")); // uptime
    }

    #[test]
    fn test_show_subprocess_stats_stopped() {
        let subprocess = oxur_repl::metrics::SubprocessMetricsSnapshot {
            is_running: false,
            uptime_seconds: 0.0,
            restart_count: 0,
            last_restart_reason: None,
        };

        let output = show_subprocess_stats(&subprocess, false);
        assert!(output.contains("Stopped"));
    }

    #[test]
    fn test_parse_stats_command_with_resources() {
        let collector = EvalMetrics::new("test");

        // Test resources command
        let result =
            parse_stats_command_with_resources("(stats resources)", &collector, None, None, false);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Resource Usage"));

        // Test fallback to regular stats
        let result = parse_stats_command_with_resources("(stats)", &collector, None, None, false);
        assert!(result.is_some());

        // Test invalid command
        let result =
            parse_stats_command_with_resources("(stats invalid)", &collector, None, None, false);
        assert!(result.is_none());
    }

    // ===== Snapshot-based display tests =====

    #[test]
    fn test_show_session_summary_from_snapshot_empty() {
        let snapshot = SessionStatsSnapshot {
            session_id: "test".to_string(),
            total_evaluations: 0,
            cache: oxur_repl::metrics::CacheStats { hits: 0, misses: 0, hit_rate: 0.0 },
            tier1_percentiles: None,
            tier2_percentiles: None,
            tier3_percentiles: None,
            parse_errors: 0,
            compile_errors: 0,
            runtime_errors: 0,
            average_eval_time_ms: 0.0,
        };

        let output = show_session_summary_from_snapshot(&snapshot, false);
        assert!(output.contains("Session Statistics"));
        assert!(output.contains("Total Evaluations: 0"));
        assert!(output.contains("No execution data yet"));
    }

    #[test]
    fn test_show_session_summary_from_snapshot_with_data() {
        let snapshot = SessionStatsSnapshot {
            session_id: "test".to_string(),
            total_evaluations: 100,
            cache: oxur_repl::metrics::CacheStats { hits: 80, misses: 20, hit_rate: 80.0 },
            tier1_percentiles: Some(oxur_repl::metrics::Percentiles {
                count: 50,
                min: 0.5,
                p50: 1.0,
                p95: 2.0,
                p99: 3.0,
                max: 5.0,
            }),
            tier2_percentiles: None,
            tier3_percentiles: None,
            parse_errors: 0,
            compile_errors: 0,
            runtime_errors: 0,
            average_eval_time_ms: 1.5,
        };

        let output = show_session_summary_from_snapshot(&snapshot, false);
        assert!(output.contains("Total Evaluations: 100"));
        assert!(output.contains("80.0%")); // hit rate
        assert!(output.contains("EXECUTION TIERS"));
        assert!(output.contains("Calculator"));
    }

    #[test]
    fn test_show_execution_from_snapshot() {
        let snapshot = SessionStatsSnapshot {
            session_id: "test".to_string(),
            total_evaluations: 10,
            cache: oxur_repl::metrics::CacheStats { hits: 5, misses: 5, hit_rate: 50.0 },
            tier1_percentiles: Some(oxur_repl::metrics::Percentiles {
                count: 5,
                min: 0.5,
                p50: 1.0,
                p95: 2.0,
                p99: 3.0,
                max: 5.0,
            }),
            tier2_percentiles: Some(oxur_repl::metrics::Percentiles {
                count: 3,
                min: 1.0,
                p50: 2.0,
                p95: 4.0,
                p99: 5.0,
                max: 8.0,
            }),
            tier3_percentiles: Some(oxur_repl::metrics::Percentiles {
                count: 2,
                min: 50.0,
                p50: 100.0,
                p95: 200.0,
                p99: 250.0,
                max: 300.0,
            }),
            parse_errors: 0,
            compile_errors: 0,
            runtime_errors: 0,
            average_eval_time_ms: 50.0,
        };

        let output = show_execution_from_snapshot(&snapshot, false);
        assert!(output.contains("Execution Statistics"));
        assert!(output.contains("TIER 1: CALCULATOR"));
        assert!(output.contains("TIER 2: CACHED LOADED"));
        assert!(output.contains("TIER 3: JUST-IN-TIME"));
    }

    #[test]
    fn test_show_cache_from_snapshot() {
        let snapshot = SessionStatsSnapshot {
            session_id: "test".to_string(),
            total_evaluations: 100,
            cache: oxur_repl::metrics::CacheStats { hits: 75, misses: 25, hit_rate: 75.0 },
            tier1_percentiles: None,
            tier2_percentiles: None,
            tier3_percentiles: None,
            parse_errors: 0,
            compile_errors: 0,
            runtime_errors: 0,
            average_eval_time_ms: 0.0,
        };

        let output = show_cache_from_snapshot(&snapshot, false);
        assert!(output.contains("Cache Statistics"));
        assert!(output.contains("EVALUATION CACHE"));
        assert!(output.contains("75")); // hits
        assert!(output.contains("25")); // misses
        assert!(output.contains("75.0%")); // hit rate
    }

    #[test]
    fn test_show_all_stats_with_optional_data() {
        let collector = EvalMetrics::new("test");

        // Test with all options as Some
        let server = oxur_repl::metrics::ServerMetricsSnapshot {
            connections_total: 10,
            connections_active: 1,
            sessions_total: 5,
            sessions_active: 1,
            requests_total: 100,
            responses_total: 100,
            responses_success: 99,
            responses_error: 1,
        };

        let client = oxur_repl::metrics::ClientMetricsSnapshot {
            requests_total: 50,
            responses_total: 50,
            responses_success: 49,
            responses_error: 1,
            average_latency_ms: 5.0,
            p50_latency_ms: 4.0,
            p95_latency_ms: 10.0,
            p99_latency_ms: 20.0,
            min_latency_ms: 1.0,
            max_latency_ms: 30.0,
        };

        let subprocess = oxur_repl::metrics::SubprocessMetricsSnapshot {
            is_running: true,
            uptime_seconds: 120.0,
            restart_count: 1,
            last_restart_reason: None,
        };

        let usage = oxur_repl::metrics::UsageMetricsSnapshot {
            session_id: "test".to_string(),
            eval_count: 25,
            help_count: 5,
            stats_count: 3,
            info_count: 2,
            sessions_count: 1,
            clear_count: 0,
            banner_count: 1,
            total_commands: 37,
        };

        let output = show_all_stats(
            &collector,
            None,
            None,
            Some(&server),
            Some(&client),
            Some(&subprocess),
            Some(&usage),
            false,
        );

        assert!(output.contains("Execution Statistics"));
        assert!(output.contains("Cache Statistics"));
        assert!(output.contains("Server Statistics"));
        assert!(output.contains("Client Statistics"));
        assert!(output.contains("Subprocess Statistics"));
        assert!(output.contains("Usage Statistics"));
    }

    // ===== Additional coverage tests =====

    #[test]
    fn test_show_sessions_with_data() {
        use oxur_repl::protocol::ReplMode;
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

        let sessions = vec![
            oxur_repl::server::SessionInfo {
                id: oxur_repl::protocol::SessionId::new("session-1"),
                name: Some("Main".to_string()),
                mode: ReplMode::Lisp,
                eval_count: 42,
                created_at: now - 3_600_000,  // 1 hour ago
                last_active_at: now - 30_000, // 30 seconds ago - "just now"
                timeout_ms: 300_000,
            },
            oxur_repl::server::SessionInfo {
                id: oxur_repl::protocol::SessionId::new("session-2"),
                name: None, // No name - tests the "-" fallback
                mode: ReplMode::Sexpr,
                eval_count: 10,
                created_at: now - 7_200_000,
                last_active_at: now - 120_000, // 2 min ago
                timeout_ms: 300_000,
            },
            oxur_repl::server::SessionInfo {
                id: oxur_repl::protocol::SessionId::new("session-3"),
                name: Some("Old".to_string()),
                mode: ReplMode::Lisp,
                eval_count: 5,
                created_at: now - 172_800_000,
                last_active_at: now - 7_200_000, // 2 hr ago
                timeout_ms: 300_000,
            },
            oxur_repl::server::SessionInfo {
                id: oxur_repl::protocol::SessionId::new("session-4"),
                name: Some("Very Old".to_string()),
                mode: ReplMode::Lisp,
                eval_count: 1,
                created_at: now - 259_200_000,
                last_active_at: now - 172_800_000, // 2 days ago
                timeout_ms: 300_000,
            },
        ];
        let current = oxur_repl::protocol::SessionId::new("session-1");

        let output = show_sessions(&sessions, &current, false);
        assert!(output.contains("Sessions"));
        assert!(output.contains("ACTIVE SESSIONS"));
        assert!(output.contains("session-1"));
        assert!(output.contains("Main"));
        assert!(output.contains("42")); // eval_count
        assert!(output.contains("just now")); // <1 min
        assert!(output.contains("min ago")); // 2 min
        assert!(output.contains("hr ago")); // 2 hr
        assert!(output.contains("days ago")); // 2 days
    }

    #[test]
    fn test_show_resource_stats_with_dir_stats() {
        use std::path::PathBuf;

        let dir_stats = oxur_repl::session::DirStats {
            file_count: 15,
            total_bytes: 102400,
            is_tmpfs: true,
            path: PathBuf::from("/tmp/oxur-session"),
        };

        let output = show_resource_stats(Some(&dir_stats), None, false);
        assert!(output.contains("Resource Usage"));
        assert!(output.contains("SESSION DIRECTORY"));
        assert!(output.contains("15")); // file_count
        assert!(output.contains("100.00 KB")); // total_bytes formatted
        assert!(output.contains("tmpfs")); // is_tmpfs flag
    }

    #[test]
    fn test_show_resource_stats_with_cache_stats() {
        use std::path::PathBuf;
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let cache_stats = oxur_repl::cache::CacheStats {
            entry_count: 25,
            total_size_bytes: 5242880,
            oldest_entry_secs: now - 3600, // 1 hour ago
            newest_entry_secs: now,
            cache_dir: PathBuf::from("/home/user/.cache/oxur"),
        };

        let output = show_resource_stats(None, Some(&cache_stats), false);
        assert!(output.contains("Resource Usage"));
        assert!(output.contains("ARTIFACT CACHE"));
        assert!(output.contains("25")); // entry_count
        assert!(output.contains("5.00 MB")); // total_size formatted
    }

    #[test]
    fn test_show_resource_stats_with_empty_cache() {
        use std::path::PathBuf;

        let cache_stats = oxur_repl::cache::CacheStats {
            entry_count: 0,
            total_size_bytes: 0,
            oldest_entry_secs: 0,
            newest_entry_secs: 0,
            cache_dir: PathBuf::from("/home/user/.cache/oxur"),
        };

        let output = show_resource_stats(None, Some(&cache_stats), false);
        assert!(output.contains("ARTIFACT CACHE"));
        assert!(output.contains("N/A")); // oldest entry shows N/A when empty
    }

    #[test]
    fn test_show_resource_stats_dir_not_tmpfs() {
        use std::path::PathBuf;

        let dir_stats = oxur_repl::session::DirStats {
            file_count: 5,
            total_bytes: 1024,
            is_tmpfs: false, // Not on tmpfs
            path: PathBuf::from("/var/oxur-session"),
        };

        let output = show_resource_stats(Some(&dir_stats), None, false);
        assert!(output.contains("SESSION DIRECTORY"));
        assert!(!output.contains("tmpfs")); // No tmpfs indicator
    }

    #[test]
    fn test_show_usage_stats_zero_commands() {
        let usage = oxur_repl::metrics::UsageMetricsSnapshot {
            session_id: "test".to_string(),
            eval_count: 0,
            help_count: 0,
            stats_count: 0,
            info_count: 0,
            sessions_count: 0,
            clear_count: 0,
            banner_count: 0,
            total_commands: 0,
        };

        let output = show_usage_stats(&usage, false);
        assert!(output.contains("Usage Statistics"));
        assert!(output.contains("0.0%")); // All percentages should be 0.0%
    }

    #[test]
    fn test_show_subprocess_stats_with_restart_reason() {
        let subprocess = oxur_repl::metrics::SubprocessMetricsSnapshot {
            is_running: true,
            uptime_seconds: 7200.0,
            restart_count: 3,
            last_restart_reason: Some(oxur_repl::metrics::RestartReason::UserRequested),
        };

        let output = show_subprocess_stats(&subprocess, false);
        assert!(output.contains("Subprocess Statistics"));
        assert!(output.contains("2h 0m")); // 2 hours
        assert!(output.contains("3")); // restart count
        assert!(output.contains("user requested")); // UserRequested displays
    }

    #[test]
    fn test_show_server_stats_zero_responses() {
        let server = oxur_repl::metrics::ServerMetricsSnapshot {
            connections_total: 5,
            connections_active: 0,
            sessions_total: 2,
            sessions_active: 0,
            requests_total: 10,
            responses_total: 0, // No responses yet
            responses_success: 0,
            responses_error: 0,
        };

        let output = show_server_stats(&server, false);
        assert!(output.contains("Server Statistics"));
        assert!(output.contains("0.0%")); // Success rate with no responses
    }

    #[test]
    fn test_show_session_summary_from_snapshot_with_tier2_and_tier3() {
        let snapshot = SessionStatsSnapshot {
            session_id: "test".to_string(),
            total_evaluations: 100,
            cache: oxur_repl::metrics::CacheStats { hits: 50, misses: 50, hit_rate: 50.0 },
            tier1_percentiles: None, // No tier1
            tier2_percentiles: Some(oxur_repl::metrics::Percentiles {
                count: 30,
                min: 1.0,
                p50: 5.0,
                p95: 10.0,
                p99: 15.0,
                max: 20.0,
            }),
            tier3_percentiles: Some(oxur_repl::metrics::Percentiles {
                count: 20,
                min: 50.0,
                p50: 100.0,
                p95: 200.0,
                p99: 300.0,
                max: 400.0,
            }),
            parse_errors: 5,
            compile_errors: 3,
            runtime_errors: 2,
            average_eval_time_ms: 25.0,
        };

        let output = show_session_summary_from_snapshot(&snapshot, false);
        assert!(output.contains("Cached")); // Tier 2
        assert!(output.contains("JIT")); // Tier 3
    }
}
