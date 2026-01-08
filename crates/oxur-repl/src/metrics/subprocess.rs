//! Subprocess metrics for restart tracking and lifecycle monitoring
//!
//! Provides:
//! - [`RestartReason`]: Categorized restart reasons from exit status
//! - [`SubprocessMetrics`]: Metrics recorder for subprocess lifecycle

use metrics::{counter, gauge};
use std::process::ExitStatus;
use std::time::Instant;

/// Categorized reasons for subprocess restart.
///
/// Determined from the subprocess exit status, distinguishing between
/// clean exits, error exits, and signal-based terminations.
///
/// # Detection Methods
///
/// | Variant | Detection |
/// |---------|-----------|
/// | `UserRequested` | Explicit restart command |
/// | `CleanShutdown` | `exit(0)` |
/// | `ErrorExit(n)` | `exit(n)` where n ≠ 0 |
/// | `Segfault` | SIGSEGV (signal 11) |
/// | `Killed` | SIGKILL (signal 9), possibly OOM |
/// | `Aborted` | SIGABRT (signal 6) |
/// | `Terminated` | SIGTERM (signal 15) |
/// | `SignalOther(n)` | Other signal |
/// | `Unknown` | No exit code or signal available |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartReason {
    /// User explicitly requested restart via command
    UserRequested,
    /// Clean shutdown with exit code 0
    CleanShutdown,
    /// Error exit with non-zero code
    ErrorExit(i32),
    /// Segmentation fault (SIGSEGV, signal 11)
    Segfault,
    /// Killed (SIGKILL, signal 9) - possibly OOM
    Killed,
    /// Aborted (SIGABRT, signal 6) - typically assertion failure
    Aborted,
    /// Terminated (SIGTERM, signal 15) - graceful shutdown request
    Terminated,
    /// Other signal termination
    SignalOther(i32),
    /// Unknown reason (no exit code or signal)
    Unknown,
}

impl RestartReason {
    /// Determine restart reason from process exit status.
    ///
    /// On Unix, checks for signal termination first, then exit code.
    /// On other platforms, only exit code is available.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::process::Command;
    /// use oxur_repl::metrics::RestartReason;
    ///
    /// let mut child = Command::new("ls").spawn().unwrap();
    /// let status = child.wait().unwrap();
    /// let reason = RestartReason::from_exit_status(status);
    /// ```
    pub fn from_exit_status(status: ExitStatus) -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(signal) = status.signal() {
                return match signal {
                    6 => Self::Aborted,
                    9 => Self::Killed,
                    11 => Self::Segfault,
                    15 => Self::Terminated,
                    s => Self::SignalOther(s),
                };
            }
        }

        match status.code() {
            Some(0) => Self::CleanShutdown,
            Some(n) => Self::ErrorExit(n),
            None => Self::Unknown,
        }
    }

    /// Get the metrics label for this restart reason.
    ///
    /// Returns a static string suitable for use as a metrics label value.
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::UserRequested => "user_requested",
            Self::CleanShutdown => "clean_shutdown",
            Self::ErrorExit(_) => "error_exit",
            Self::Segfault => "segfault",
            Self::Killed => "killed",
            Self::Aborted => "aborted",
            Self::Terminated => "terminated",
            Self::SignalOther(_) => "signal_other",
            Self::Unknown => "unknown",
        }
    }

    /// Check if this is a signal-based termination.
    pub fn is_signal(&self) -> bool {
        matches!(
            self,
            Self::Segfault | Self::Killed | Self::Aborted | Self::Terminated | Self::SignalOther(_)
        )
    }

    /// Check if this is a clean shutdown.
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::CleanShutdown | Self::UserRequested)
    }
}

impl std::fmt::Display for RestartReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserRequested => write!(f, "user requested"),
            Self::CleanShutdown => write!(f, "clean shutdown"),
            Self::ErrorExit(code) => write!(f, "error exit (code {})", code),
            Self::Segfault => write!(f, "segmentation fault (SIGSEGV)"),
            Self::Killed => write!(f, "killed (SIGKILL)"),
            Self::Aborted => write!(f, "aborted (SIGABRT)"),
            Self::Terminated => write!(f, "terminated (SIGTERM)"),
            Self::SignalOther(sig) => write!(f, "signal {}", sig),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Subprocess metrics recorder.
///
/// Tracks subprocess lifecycle including:
/// - Restart counts by reason
/// - Current uptime
/// - Start time tracking
///
/// # Usage
///
/// ```
/// use oxur_repl::metrics::{SubprocessMetrics, RestartReason};
///
/// let mut metrics = SubprocessMetrics::new();
///
/// // Record process start
/// metrics.process_started();
///
/// // On subprocess death, record reason
/// metrics.record_restart(RestartReason::ErrorExit(1));
///
/// // Get current uptime
/// let uptime = metrics.uptime_seconds();
/// ```
#[derive(Debug)]
pub struct SubprocessMetrics {
    started_at: Option<Instant>,
    restart_count: u64,
}

impl SubprocessMetrics {
    /// Create a new SubprocessMetrics instance.
    pub fn new() -> Self {
        Self { started_at: None, restart_count: 0 }
    }

    /// Record that the subprocess has started.
    ///
    /// Resets the uptime tracking and updates the `repl.subprocess.uptime_seconds` gauge.
    pub fn process_started(&mut self) {
        self.started_at = Some(Instant::now());
        gauge!("repl.subprocess.uptime_seconds").set(0.0);
    }

    /// Record a subprocess restart with the given reason.
    ///
    /// Increments `repl.subprocess.restarts_total` counter with reason label.
    ///
    /// # Arguments
    ///
    /// * `reason` - The categorized reason for the restart
    pub fn record_restart(&mut self, reason: RestartReason) {
        self.restart_count += 1;
        counter!("repl.subprocess.restarts_total", "reason" => reason.as_label()).increment(1);
    }

    /// Get the current uptime in seconds.
    ///
    /// Returns 0.0 if the subprocess hasn't been started.
    pub fn uptime_seconds(&self) -> f64 {
        self.started_at.map(|start| start.elapsed().as_secs_f64()).unwrap_or(0.0)
    }

    /// Update the uptime gauge with the current value.
    ///
    /// Should be called periodically to keep the gauge current.
    pub fn update_uptime_gauge(&self) {
        gauge!("repl.subprocess.uptime_seconds").set(self.uptime_seconds());
    }

    /// Get the total restart count.
    pub fn restart_count(&self) -> u64 {
        self.restart_count
    }
}

impl Default for SubprocessMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restart_reason_labels() {
        assert_eq!(RestartReason::UserRequested.as_label(), "user_requested");
        assert_eq!(RestartReason::CleanShutdown.as_label(), "clean_shutdown");
        assert_eq!(RestartReason::ErrorExit(1).as_label(), "error_exit");
        assert_eq!(RestartReason::Segfault.as_label(), "segfault");
        assert_eq!(RestartReason::Killed.as_label(), "killed");
        assert_eq!(RestartReason::Aborted.as_label(), "aborted");
        assert_eq!(RestartReason::Terminated.as_label(), "terminated");
        assert_eq!(RestartReason::SignalOther(99).as_label(), "signal_other");
        assert_eq!(RestartReason::Unknown.as_label(), "unknown");
    }

    #[test]
    fn test_restart_reason_is_signal() {
        assert!(!RestartReason::UserRequested.is_signal());
        assert!(!RestartReason::CleanShutdown.is_signal());
        assert!(!RestartReason::ErrorExit(1).is_signal());
        assert!(RestartReason::Segfault.is_signal());
        assert!(RestartReason::Killed.is_signal());
        assert!(RestartReason::Aborted.is_signal());
        assert!(RestartReason::Terminated.is_signal());
        assert!(RestartReason::SignalOther(99).is_signal());
        assert!(!RestartReason::Unknown.is_signal());
    }

    #[test]
    fn test_restart_reason_is_clean() {
        assert!(RestartReason::UserRequested.is_clean());
        assert!(RestartReason::CleanShutdown.is_clean());
        assert!(!RestartReason::ErrorExit(1).is_clean());
        assert!(!RestartReason::Segfault.is_clean());
        assert!(!RestartReason::Unknown.is_clean());
    }

    #[test]
    fn test_restart_reason_display() {
        assert_eq!(format!("{}", RestartReason::UserRequested), "user requested");
        assert_eq!(format!("{}", RestartReason::CleanShutdown), "clean shutdown");
        assert_eq!(format!("{}", RestartReason::ErrorExit(42)), "error exit (code 42)");
        assert_eq!(format!("{}", RestartReason::Segfault), "segmentation fault (SIGSEGV)");
        assert_eq!(format!("{}", RestartReason::Killed), "killed (SIGKILL)");
    }

    #[test]
    fn test_subprocess_metrics_creation() {
        let metrics = SubprocessMetrics::new();
        assert_eq!(metrics.uptime_seconds(), 0.0);
        assert_eq!(metrics.restart_count(), 0);
    }

    #[test]
    fn test_subprocess_metrics_lifecycle() {
        let mut metrics = SubprocessMetrics::new();

        // Start process
        metrics.process_started();
        assert!(metrics.uptime_seconds() >= 0.0);

        // Record restart
        metrics.record_restart(RestartReason::ErrorExit(1));
        assert_eq!(metrics.restart_count(), 1);

        // Start again
        metrics.process_started();
        metrics.record_restart(RestartReason::Segfault);
        assert_eq!(metrics.restart_count(), 2);
    }
}
