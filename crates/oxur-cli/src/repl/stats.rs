//! Statistics display and formatting for REPL sessions
//!
//! Provides display functions for REPL statistics using OxurTable formatting.
//! Core data collection is in oxur-repl; this module handles presentation only.

use oxur_cli::table::{OxurTable, Tabled};
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
}
