//! Usage metrics for tracking REPL command patterns
//!
//! Provides [`UsageMetrics`] for tracking command frequency and usage patterns
//! to understand how users interact with the REPL.

use metrics::counter;
use std::collections::VecDeque;

/// Maximum command history to keep (for recent activity analysis)
const MAX_HISTORY: usize = 100;

/// Usage metrics collector.
///
/// Tracks command frequency and patterns to understand REPL usage.
/// Maintains local state for analysis while also emitting to the
/// `metrics` crate facade for external monitoring.
///
/// # Usage
///
/// ```
/// use oxur_repl::metrics::UsageMetrics;
///
/// let mut metrics = UsageMetrics::new("session-1");
///
/// // Record commands
/// metrics.record_eval();
/// metrics.record_help();
/// metrics.record_stats();
///
/// // Get statistics
/// let snapshot = metrics.snapshot();
/// println!("Total commands: {}", snapshot.total_commands);
/// ```
#[derive(Debug, Clone)]
pub struct UsageMetrics {
    /// Session identifier
    session_id: String,

    /// Eval command count
    eval_count: u64,

    /// Help command count
    help_count: u64,

    /// Stats command count
    stats_count: u64,

    /// Info command count
    info_count: u64,

    /// Sessions command count
    sessions_count: u64,

    /// Clear command count
    clear_count: u64,

    /// Banner command count
    banner_count: u64,

    /// Recent command history (for pattern analysis)
    recent_commands: VecDeque<CommandType>,
}

/// Command type for history tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    Eval,
    Help,
    Stats,
    Info,
    Sessions,
    Clear,
    Banner,
}

impl CommandType {
    /// Convert to label string for metrics
    pub fn as_label(&self) -> &'static str {
        match self {
            CommandType::Eval => "eval",
            CommandType::Help => "help",
            CommandType::Stats => "stats",
            CommandType::Info => "info",
            CommandType::Sessions => "sessions",
            CommandType::Clear => "clear",
            CommandType::Banner => "banner",
        }
    }
}

impl UsageMetrics {
    /// Create a new usage metrics collector.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            eval_count: 0,
            help_count: 0,
            stats_count: 0,
            info_count: 0,
            sessions_count: 0,
            clear_count: 0,
            banner_count: 0,
            recent_commands: VecDeque::with_capacity(MAX_HISTORY),
        }
    }

    /// Record a command execution.
    fn record_command(&mut self, cmd_type: CommandType) {
        // Update count
        match cmd_type {
            CommandType::Eval => self.eval_count += 1,
            CommandType::Help => self.help_count += 1,
            CommandType::Stats => self.stats_count += 1,
            CommandType::Info => self.info_count += 1,
            CommandType::Sessions => self.sessions_count += 1,
            CommandType::Clear => self.clear_count += 1,
            CommandType::Banner => self.banner_count += 1,
        }

        // Add to recent history
        if self.recent_commands.len() >= MAX_HISTORY {
            self.recent_commands.pop_front();
        }
        self.recent_commands.push_back(cmd_type);

        // Emit metrics via facade
        counter!("repl.commands.total", "type" => cmd_type.as_label()).increment(1);
    }

    /// Record an eval command.
    pub fn record_eval(&mut self) {
        self.record_command(CommandType::Eval);
    }

    /// Record a help command.
    pub fn record_help(&mut self) {
        self.record_command(CommandType::Help);
    }

    /// Record a stats command.
    pub fn record_stats(&mut self) {
        self.record_command(CommandType::Stats);
    }

    /// Record an info command.
    pub fn record_info(&mut self) {
        self.record_command(CommandType::Info);
    }

    /// Record a sessions command.
    pub fn record_sessions(&mut self) {
        self.record_command(CommandType::Sessions);
    }

    /// Record a clear command.
    pub fn record_clear(&mut self) {
        self.record_command(CommandType::Clear);
    }

    /// Record a banner command.
    pub fn record_banner(&mut self) {
        self.record_command(CommandType::Banner);
    }

    /// Get total command count.
    pub fn total_commands(&self) -> u64 {
        self.eval_count
            + self.help_count
            + self.stats_count
            + self.info_count
            + self.sessions_count
            + self.clear_count
            + self.banner_count
    }

    /// Get eval percentage.
    pub fn eval_percentage(&self) -> f64 {
        let total = self.total_commands();
        if total > 0 {
            (self.eval_count as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Create a snapshot for display.
    pub fn snapshot(&self) -> UsageMetricsSnapshot {
        UsageMetricsSnapshot {
            session_id: self.session_id.clone(),
            eval_count: self.eval_count,
            help_count: self.help_count,
            stats_count: self.stats_count,
            info_count: self.info_count,
            sessions_count: self.sessions_count,
            clear_count: self.clear_count,
            banner_count: self.banner_count,
            total_commands: self.total_commands(),
        }
    }
}

/// Snapshot of usage metrics for display.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UsageMetricsSnapshot {
    /// Session identifier
    pub session_id: String,
    /// Eval command count
    pub eval_count: u64,
    /// Help command count
    pub help_count: u64,
    /// Stats command count
    pub stats_count: u64,
    /// Info command count
    pub info_count: u64,
    /// Sessions command count
    pub sessions_count: u64,
    /// Clear command count
    pub clear_count: u64,
    /// Banner command count
    pub banner_count: u64,
    /// Total command count
    pub total_commands: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_metrics_creation() {
        let metrics = UsageMetrics::new("test-session");
        assert_eq!(metrics.session_id(), "test-session");
        assert_eq!(metrics.total_commands(), 0);
    }

    #[test]
    fn test_record_commands() {
        let mut metrics = UsageMetrics::new("test");

        metrics.record_eval();
        metrics.record_eval();
        metrics.record_help();
        metrics.record_stats();

        assert_eq!(metrics.eval_count, 2);
        assert_eq!(metrics.help_count, 1);
        assert_eq!(metrics.stats_count, 1);
        assert_eq!(metrics.total_commands(), 4);
    }

    #[test]
    fn test_eval_percentage() {
        let mut metrics = UsageMetrics::new("test");

        metrics.record_eval();
        metrics.record_eval();
        metrics.record_eval();
        metrics.record_help();

        assert_eq!(metrics.eval_percentage(), 75.0);
    }

    #[test]
    fn test_snapshot() {
        let mut metrics = UsageMetrics::new("test");

        metrics.record_eval();
        metrics.record_help();
        metrics.record_stats();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.eval_count, 1);
        assert_eq!(snapshot.help_count, 1);
        assert_eq!(snapshot.stats_count, 1);
        assert_eq!(snapshot.total_commands, 3);
    }

    #[test]
    fn test_command_history() {
        let mut metrics = UsageMetrics::new("test");

        for _ in 0..150 {
            metrics.record_eval();
        }

        // Should cap at MAX_HISTORY
        assert_eq!(metrics.recent_commands.len(), MAX_HISTORY);
    }
}
