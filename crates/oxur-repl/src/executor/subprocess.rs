//! Subprocess-based code execution with isolation
//!
//! Manages a subprocess for executing compiled user code with isolation,
//! panic catching, and Ctrl-C interruption support.
//!
//! Based on ODD-0026 Section 3.2 (SubprocessExecutor)

use crate::metrics::{RestartReason, SubprocessMetrics};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use thiserror::Error;

/// Subprocess execution errors
#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Failed to spawn subprocess: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("Subprocess not running")]
    NotRunning,

    #[error("Failed to send command to subprocess: {0}")]
    CommandFailed(String),

    #[error("Failed to read response from subprocess: {0}")]
    ResponseFailed(String),

    #[error("Library load failed: {0}")]
    LoadFailed(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Runtime error in subprocess: {0}")]
    RuntimeError(String),

    #[error("Panic in subprocess: {0}")]
    Panic(String),
}

pub type Result<T> = std::result::Result<T, ExecutorError>;

/// Execution response from subprocess
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionResult {
    /// Execution completed successfully
    Success { output: String },

    /// Runtime error occurred
    RuntimeError { message: String },

    /// Panic occurred
    Panic { location: String, message: String },
}

/// Subprocess executor
///
/// Manages a child process for executing compiled code with isolation.
/// The subprocess binary implements the IPC protocol for loading libraries
/// and executing functions.
///
/// # Why Subprocess? (ODD-0038 Decision 3)
///
/// - Rust threads cannot be interrupted (no pthread_cancel)
/// - Ctrl-C support requires process isolation
/// - Crashes in user code don't corrupt REPL state
/// - evcxr evidence: Subprocess from day one, never changed
///
/// # IPC Protocol (ODD-0038 Decision 3a)
///
/// Commands (stdin):
/// - `LOAD <path>` - Load dynamic library
/// - `RUN <cache_key>` - Execute function from loaded library
///
/// Responses (stdout):
/// - `LOADED` - Library loaded successfully
/// - `OXUR_EXECUTION_COMPLETE` - Execution finished
/// - `OXUR_RUNTIME_ERROR <msg>` - Runtime error
/// - `OXUR_PANIC_LOCATION <info>` - Panic with location
///
/// # Status
///
/// **Phase 1 STUB:** Manages subprocess lifecycle but doesn't execute yet.
/// Full IPC implementation coming in Phase 2.
///
/// # Examples
///
/// ```no_run
/// use oxur_repl::executor::SubprocessExecutor;
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create executor (spawns subprocess)
/// let mut executor = SubprocessExecutor::new()?;
///
/// // Load a library (Phase 2)
/// // executor.load_library(&PathBuf::from("path/to/lib.so"))?;
///
/// // Execute code (Phase 2)
/// // let result = executor.execute("cache_key_123")?;
///
/// // Shutdown
/// executor.shutdown()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SubprocessExecutor {
    /// Child process handle
    child: Option<Child>,

    /// Stdin handle for sending commands
    stdin: Option<ChildStdin>,

    /// Stdout handle for reading responses
    stdout: Option<BufReader<ChildStdout>>,

    /// Set of loaded library cache keys
    loaded_libraries: HashSet<String>,

    /// Path to subprocess binary
    subprocess_binary: PathBuf,

    /// Metrics for tracking subprocess lifecycle
    metrics: SubprocessMetrics,
}

impl SubprocessExecutor {
    /// Create a new subprocess executor
    ///
    /// Spawns the subprocess binary and establishes IPC connection.
    ///
    /// # Errors
    ///
    /// Returns error if subprocess cannot be spawned
    pub fn new() -> Result<Self> {
        let subprocess_binary = Self::find_subprocess_binary()?;

        let mut executor = Self {
            child: None,
            stdin: None,
            stdout: None,
            loaded_libraries: HashSet::new(),
            subprocess_binary,
            metrics: SubprocessMetrics::new(),
        };

        executor.spawn()?;
        Ok(executor)
    }

    /// Find the subprocess binary
    ///
    /// Looks for `oxur-repl-subprocess` in:
    /// 1. Same directory as current executable
    /// 2. PATH
    /// 3. Target directory (for development)
    fn find_subprocess_binary() -> Result<PathBuf> {
        // Try same directory as current executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let subprocess = parent.join("oxur-repl-subprocess");
                if subprocess.exists() {
                    return Ok(subprocess);
                }
            }
        }

        // Try PATH
        if let Ok(path) = which::which("oxur-repl-subprocess") {
            return Ok(path);
        }

        // Try target directory (development) - local crate
        let target_debug = PathBuf::from("target/debug/oxur-repl-subprocess");
        if target_debug.exists() {
            return Ok(target_debug);
        }

        let target_release = PathBuf::from("target/release/oxur-repl-subprocess");
        if target_release.exists() {
            return Ok(target_release);
        }

        // Try workspace root target directory (for workspace layout)
        let workspace_debug = PathBuf::from("../../target/debug/oxur-repl-subprocess");
        if workspace_debug.exists() {
            return Ok(workspace_debug);
        }

        let workspace_release = PathBuf::from("../../target/release/oxur-repl-subprocess");
        if workspace_release.exists() {
            return Ok(workspace_release);
        }

        // Try ../../../target for deeper nesting (e.g., running from tests/)
        let deep_workspace_debug = PathBuf::from("../../../target/debug/oxur-repl-subprocess");
        if deep_workspace_debug.exists() {
            return Ok(deep_workspace_debug);
        }

        // Fallback: assume it's in PATH
        Ok(PathBuf::from("oxur-repl-subprocess"))
    }

    /// Spawn the subprocess
    fn spawn(&mut self) -> Result<()> {
        let mut child = Command::new(&self.subprocess_binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Inherit stderr for debugging
            .spawn()?;

        let stdin = child.stdin.take().expect("Failed to take stdin");
        let stdout = child.stdout.take().expect("Failed to take stdout");

        self.child = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(BufReader::new(stdout));

        // Record process start for metrics
        self.metrics.process_started();

        Ok(())
    }

    /// Check if a library is already loaded
    pub fn is_loaded(&self, cache_key: impl AsRef<str>) -> bool {
        self.loaded_libraries.contains(cache_key.as_ref())
    }

    /// Load a dynamic library into the subprocess
    ///
    /// Sends a LOAD command via IPC and waits for confirmation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to dynamic library (.so/.dylib/.dll)
    /// * `cache_key` - Cache key for tracking loaded state
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Subprocess is not running
    /// - LOAD command fails to send
    /// - Library fails to load in subprocess
    /// - Response parsing fails
    pub fn load_library(&mut self, path: &Path, cache_key: impl AsRef<str>) -> Result<()> {
        let cache_key = cache_key.as_ref();

        // Check if already loaded
        if self.is_loaded(cache_key) {
            return Ok(());
        }

        // Send LOAD command: LOAD <cache_key> <path>
        let command = format!("LOAD {} {}\n", cache_key, path.display());
        self.send_command(&command)?;

        // Wait for response, skipping empty lines
        let response = loop {
            let line = self.read_line()?;
            if !line.trim().is_empty() {
                break line;
            }
        };

        if response == "LOADED" {
            // Mark as loaded
            self.loaded_libraries.insert(cache_key.to_string());
            Ok(())
        } else if let Some(error) = response.strip_prefix("LOAD_ERROR ") {
            Err(ExecutorError::LoadFailed(error.to_string()))
        } else {
            Err(ExecutorError::LoadFailed(format!("Unexpected response: {}", response)))
        }
    }

    /// Execute compiled code from a loaded library
    ///
    /// Sends a RUN command via IPC and parses the execution result.
    ///
    /// # Arguments
    ///
    /// * `cache_key` - Cache key identifying the function to execute
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Subprocess is not running
    /// - RUN command fails to send
    /// - Response parsing fails
    ///
    /// # Returns
    ///
    /// Returns ExecutionResult with success, runtime error, or panic information
    pub fn execute(&mut self, cache_key: impl AsRef<str>) -> Result<ExecutionResult> {
        let cache_key = cache_key.as_ref();

        // Send RUN command
        let command = format!("RUN {}\n", cache_key);
        self.send_command(&command)?;

        // Read responses (may include OXUR_RESULT before OXUR_EXECUTION_COMPLETE)
        let mut result_value = String::new();

        loop {
            let response = self.read_line()?;

            // Skip empty lines (can happen with subprocess stdout flushing)
            if response.trim().is_empty() {
                continue;
            }

            match response.as_str() {
                "OXUR_EXECUTION_COMPLETE" => {
                    return Ok(ExecutionResult::Success { output: result_value });
                }
                line if line.starts_with("OXUR_RESULT ") => {
                    result_value = line.strip_prefix("OXUR_RESULT ").unwrap_or("").to_string();
                    // Continue reading to get OXUR_EXECUTION_COMPLETE
                }
                line if line.starts_with("OXUR_RUNTIME_ERROR ") => {
                    let message =
                        line.strip_prefix("OXUR_RUNTIME_ERROR ").unwrap_or("").to_string();
                    return Ok(ExecutionResult::RuntimeError { message });
                }
                line if line.starts_with("OXUR_PANIC_LOCATION ") => {
                    let location =
                        line.strip_prefix("OXUR_PANIC_LOCATION ").unwrap_or("unknown").to_string();

                    // Read next line for panic message
                    let message_line = self.read_line()?;
                    let message = message_line
                        .strip_prefix("OXUR_PANIC_MESSAGE ")
                        .unwrap_or("Unknown panic")
                        .to_string();

                    return Ok(ExecutionResult::Panic { location, message });
                }
                _ => {
                    return Err(ExecutorError::ExecutionFailed(format!(
                        "Unexpected response: {}",
                        response
                    )));
                }
            }
        }
    }

    /// Send a command to the subprocess
    fn send_command(&mut self, command: &str) -> Result<()> {
        let stdin = self.stdin.as_mut().ok_or(ExecutorError::NotRunning)?;

        stdin
            .write_all(command.as_bytes())
            .map_err(|e| ExecutorError::CommandFailed(e.to_string()))?;

        stdin.flush().map_err(|e| ExecutorError::CommandFailed(format!("flush failed: {}", e)))?;

        Ok(())
    }

    /// Read a response line from subprocess
    fn read_line(&mut self) -> Result<String> {
        let stdout = self.stdout.as_mut().ok_or(ExecutorError::NotRunning)?;

        let mut line = String::new();
        stdout.read_line(&mut line).map_err(|e| ExecutorError::ResponseFailed(e.to_string()))?;

        Ok(line.trim().to_string())
    }

    /// Shutdown the subprocess
    ///
    /// Closes stdin and waits for subprocess to exit.
    ///
    /// Returns the exit status if the subprocess was running, None otherwise.
    pub fn shutdown(&mut self) -> Result<Option<ExitStatus>> {
        // Close stdin to signal subprocess to exit
        self.stdin.take();

        // Wait for child to exit and capture exit status
        let exit_status =
            if let Some(mut child) = self.child.take() { child.wait().ok() } else { None };

        self.stdout.take();
        self.loaded_libraries.clear();

        Ok(exit_status)
    }

    /// Restart the subprocess
    ///
    /// Useful for recovering from crashes or clearing state.
    /// Records the restart reason in metrics based on exit status.
    ///
    /// Returns the restart reason determined from exit status.
    pub fn restart(&mut self) -> Result<RestartReason> {
        let exit_status = self.shutdown()?;
        let reason =
            exit_status.map(RestartReason::from_exit_status).unwrap_or(RestartReason::Unknown);

        self.metrics.record_restart(reason);
        self.spawn()?;
        Ok(reason)
    }

    /// Restart the subprocess due to user request
    ///
    /// Records the restart as user-requested in metrics.
    pub fn restart_user_requested(&mut self) -> Result<()> {
        let _ = self.shutdown()?;
        self.metrics.record_restart(RestartReason::UserRequested);
        self.spawn()?;
        Ok(())
    }

    /// Get count of loaded libraries
    pub fn loaded_count(&self) -> usize {
        self.loaded_libraries.len()
    }

    /// Check if subprocess is running
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    /// Get the current subprocess uptime in seconds
    pub fn uptime_seconds(&self) -> f64 {
        self.metrics.uptime_seconds()
    }

    /// Get the total restart count
    pub fn restart_count(&self) -> u64 {
        self.metrics.restart_count()
    }

    /// Update the uptime gauge in metrics
    ///
    /// Should be called periodically to keep the metrics current.
    pub fn update_metrics(&self) {
        self.metrics.update_uptime_gauge();
    }

    /// Get a reference to the subprocess metrics
    pub fn metrics(&self) -> &SubprocessMetrics {
        &self.metrics
    }
}

impl Drop for SubprocessExecutor {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        // This will spawn the actual subprocess binary
        let executor = SubprocessExecutor::new();

        // Should succeed if subprocess binary exists
        match executor {
            Ok(exec) => {
                assert!(exec.is_running());
            }
            Err(e) => {
                // Binary not found is OK in test environment
                eprintln!(
                    "Note: Subprocess binary not found (expected in some test environments): {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_executor_is_loaded() {
        if let Ok(executor) = SubprocessExecutor::new() {
            // Initially, no libraries should be loaded
            assert!(!executor.is_loaded("test_key"));
            assert!(!executor.is_loaded("another_key"));

            // Note: We can't test successful loading without a real compiled library.
            // That's covered by integration tests in tests/compilation_integration.rs
        }
    }

    #[test]
    fn test_executor_loaded_count() {
        if let Ok(executor) = SubprocessExecutor::new() {
            // Initially, no libraries should be loaded
            assert_eq!(executor.loaded_count(), 0);

            // Note: We can't test incrementing count without real compiled libraries.
            // That's covered by integration tests in tests/compilation_integration.rs
        }
    }

    #[test]
    fn test_executor_shutdown() {
        if let Ok(mut executor) = SubprocessExecutor::new() {
            assert!(executor.is_running());

            executor.shutdown().expect("Shutdown failed");
            assert!(!executor.is_running());
        }
    }

    #[test]
    fn test_executor_restart() {
        if let Ok(mut executor) = SubprocessExecutor::new() {
            // Verify executor starts with no loaded libraries
            assert_eq!(executor.loaded_count(), 0);
            assert!(executor.is_running());
            assert_eq!(executor.restart_count(), 0);

            // Restart should succeed and return a reason
            let reason = executor.restart().expect("Restart failed");
            // First restart from clean process should be CleanShutdown
            assert!(reason.is_clean(), "Expected clean restart, got {:?}", reason);

            // After restart, state should still be clean but restart count incremented
            assert_eq!(executor.loaded_count(), 0);
            assert!(executor.is_running());
            assert_eq!(executor.restart_count(), 1);
        }
    }

    #[test]
    fn test_executor_restart_user_requested() {
        if let Ok(mut executor) = SubprocessExecutor::new() {
            assert_eq!(executor.restart_count(), 0);

            // User-requested restart
            executor.restart_user_requested().expect("Restart failed");

            assert!(executor.is_running());
            assert_eq!(executor.restart_count(), 1);
        }
    }

    #[test]
    fn test_executor_uptime() {
        if let Ok(executor) = SubprocessExecutor::new() {
            // Uptime should be non-negative
            let uptime = executor.uptime_seconds();
            assert!(uptime >= 0.0, "Uptime should be non-negative");
        }
    }

    #[test]
    fn test_executor_execute_unloaded() {
        if let Ok(mut executor) = SubprocessExecutor::new() {
            // Trying to execute code that hasn't been loaded should fail gracefully
            // The subprocess will return an error since the cache key doesn't exist
            let result = executor.execute("nonexistent_key");

            // This should either fail or return an error result
            // The exact behavior depends on the subprocess implementation
            match result {
                Ok(ExecutionResult::RuntimeError { .. }) => {
                    // Expected: subprocess reports runtime error
                }
                Err(_) => {
                    // Also acceptable: execution fails
                }
                Ok(ExecutionResult::Success { .. }) => {
                    panic!("Should not succeed when executing unloaded code");
                }
                Ok(ExecutionResult::Panic { .. }) => {
                    // Also acceptable: subprocess panics
                }
            }
        }
    }
}
