//! Output Capture for REPL Execution
//!
//! Captures stdout and stderr during code execution to return as part of
//! the evaluation result. This allows REPL clients to display all output
//! from executed code.
//!
//! Based on ODD-0026: Oxur REPL Evaluation Strategy

use std::sync::{Arc, Mutex};

/// Captured output from code execution
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapturedOutput {
    /// Standard output (stdout)
    pub stdout: String,

    /// Standard error (stderr)
    pub stderr: String,
}

impl CapturedOutput {
    /// Create a new empty captured output
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any output was captured
    pub fn is_empty(&self) -> bool {
        self.stdout.is_empty() && self.stderr.is_empty()
    }

    /// Get stdout as Option (None if empty)
    pub fn stdout_option(&self) -> Option<String> {
        if self.stdout.is_empty() {
            None
        } else {
            Some(self.stdout.clone())
        }
    }

    /// Get stderr as Option (None if empty)
    pub fn stderr_option(&self) -> Option<String> {
        if self.stderr.is_empty() {
            None
        } else {
            Some(self.stderr.clone())
        }
    }
}

/// Output capturer for a single execution
///
/// Provides a simple interface for capturing output during code execution.
/// Uses Arc<Mutex<String>> internally to allow sharing across threads.
pub struct OutputCapturer {
    /// Captured stdout
    stdout: Arc<Mutex<String>>,

    /// Captured stderr
    stderr: Arc<Mutex<String>>,
}

impl OutputCapturer {
    /// Create a new output capturer
    pub fn new() -> Self {
        Self {
            stdout: Arc::new(Mutex::new(String::new())),
            stderr: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Capture stdout
    ///
    /// Appends the given string to the captured stdout.
    pub fn capture_stdout(&self, output: &str) {
        if let Ok(mut stdout) = self.stdout.lock() {
            stdout.push_str(output);
        }
    }

    /// Capture stderr
    ///
    /// Appends the given string to the captured stderr.
    pub fn capture_stderr(&self, output: &str) {
        if let Ok(mut stderr) = self.stderr.lock() {
            stderr.push_str(output);
        }
    }

    /// Get the captured output
    pub fn get_output(&self) -> CapturedOutput {
        let stdout = self.stdout.lock().map(|s| s.clone()).unwrap_or_default();

        let stderr = self.stderr.lock().map(|s| s.clone()).unwrap_or_default();

        CapturedOutput { stdout, stderr }
    }

    /// Clear all captured output
    pub fn clear(&self) {
        if let Ok(mut stdout) = self.stdout.lock() {
            stdout.clear();
        }
        if let Ok(mut stderr) = self.stderr.lock() {
            stderr.clear();
        }
    }

    /// Execute a function and capture its output
    ///
    /// This is a helper for testing and will be used when we integrate
    /// with actual code execution.
    ///
    /// # Example
    ///
    /// ```
    /// use oxur_repl::eval::output_capture::OutputCapturer;
    ///
    /// let capturer = OutputCapturer::new();
    ///
    /// let result = capturer.with_capture(|| {
    ///     // Simulate some output
    ///     capturer.capture_stdout("Hello, ");
    ///     capturer.capture_stdout("world!\n");
    ///     capturer.capture_stderr("Warning: test\n");
    ///     42
    /// });
    ///
    /// assert_eq!(result, 42);
    /// let output = capturer.get_output();
    /// assert_eq!(output.stdout, "Hello, world!\n");
    /// assert_eq!(output.stderr, "Warning: test\n");
    /// ```
    pub fn with_capture<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Clear previous output
        self.clear();

        // Execute and return the result
        f()
    }
}

impl Default for OutputCapturer {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulate compiled code execution with output
///
/// This is a placeholder for when we actually execute compiled code.
/// It demonstrates how output capture will work with real execution.
pub fn simulate_execution(code: &str, capturer: &OutputCapturer) -> String {
    // Simulate different types of code execution

    // Check for eprintln! (check this first since it contains "println")
    if code.contains("eprintln!(\"") {
        if let Some(start) = code.find("eprintln!(\"") {
            if let Some(end) = code[start..].find("\")") {
                let msg = &code[start + 11..start + end];
                capturer.capture_stderr(&format!("{}\n", msg));
            }
        }
    }

    // Also check for println! (use more specific match to avoid matching eprintln)
    if let Some(start) = code.find("println!(\"") {
        // Make sure this isn't part of "eprintln"
        let is_println = if start > 0 { !code[..start].ends_with('e') } else { true };

        if is_println {
            if let Some(end) = code[start..].find("\")") {
                let msg = &code[start + 10..start + end];
                capturer.capture_stdout(&format!("{}\n", msg));
            }
        }
    }

    // Return a placeholder execution result
    format!("executed: {}", code.chars().take(50).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captured_output_new() {
        let output = CapturedOutput::new();
        assert!(output.is_empty());
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "");
    }

    #[test]
    fn test_captured_output_is_empty() {
        let mut output = CapturedOutput::new();
        assert!(output.is_empty());

        output.stdout = "test".to_string();
        assert!(!output.is_empty());

        output.stdout.clear();
        output.stderr = "error".to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_captured_output_options() {
        let output = CapturedOutput { stdout: "output".to_string(), stderr: "".to_string() };

        assert_eq!(output.stdout_option(), Some("output".to_string()));
        assert_eq!(output.stderr_option(), None);
    }

    #[test]
    fn test_output_capturer_new() {
        let capturer = OutputCapturer::new();
        let output = capturer.get_output();
        assert!(output.is_empty());
    }

    #[test]
    fn test_capture_stdout() {
        let capturer = OutputCapturer::new();

        capturer.capture_stdout("Hello, ");
        capturer.capture_stdout("world!\n");

        let output = capturer.get_output();
        assert_eq!(output.stdout, "Hello, world!\n");
        assert_eq!(output.stderr, "");
    }

    #[test]
    fn test_capture_stderr() {
        let capturer = OutputCapturer::new();

        capturer.capture_stderr("Error: ");
        capturer.capture_stderr("something went wrong\n");

        let output = capturer.get_output();
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "Error: something went wrong\n");
    }

    #[test]
    fn test_capture_both() {
        let capturer = OutputCapturer::new();

        capturer.capture_stdout("Output line 1\n");
        capturer.capture_stderr("Warning: test\n");
        capturer.capture_stdout("Output line 2\n");

        let output = capturer.get_output();
        assert_eq!(output.stdout, "Output line 1\nOutput line 2\n");
        assert_eq!(output.stderr, "Warning: test\n");
    }

    #[test]
    fn test_clear() {
        let capturer = OutputCapturer::new();

        capturer.capture_stdout("test");
        capturer.capture_stderr("error");

        assert!(!capturer.get_output().is_empty());

        capturer.clear();

        let output = capturer.get_output();
        assert!(output.is_empty());
    }

    #[test]
    fn test_with_capture() {
        let capturer = OutputCapturer::new();

        let result = capturer.with_capture(|| {
            capturer.capture_stdout("captured\n");
            42
        });

        assert_eq!(result, 42);
        assert_eq!(capturer.get_output().stdout, "captured\n");
    }

    #[test]
    fn test_with_capture_clears_previous() {
        let capturer = OutputCapturer::new();

        capturer.capture_stdout("old output\n");

        capturer.with_capture(|| {
            capturer.capture_stdout("new output\n");
        });

        let output = capturer.get_output();
        assert_eq!(output.stdout, "new output\n");
    }

    #[test]
    fn test_simulate_execution_println() {
        let capturer = OutputCapturer::new();

        let code = r#"println!("Hello from Oxur")"#;
        simulate_execution(code, &capturer);

        let output = capturer.get_output();
        assert_eq!(output.stdout, "Hello from Oxur\n");
        assert_eq!(output.stderr, "");
    }

    #[test]
    fn test_simulate_execution_eprintln() {
        let capturer = OutputCapturer::new();

        let code = r#"eprintln!("Error message")"#;
        simulate_execution(code, &capturer);

        let output = capturer.get_output();
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "Error message\n");
    }

    #[test]
    fn test_simulate_execution_both() {
        let capturer = OutputCapturer::new();

        let code = r#"println!("output"); eprintln!("error")"#;
        simulate_execution(code, &capturer);

        let output = capturer.get_output();
        assert_eq!(output.stdout, "output\n");
        assert_eq!(output.stderr, "error\n");
    }

    #[test]
    fn test_simulate_execution_no_output() {
        let capturer = OutputCapturer::new();

        let code = "let x = 42;";
        let result = simulate_execution(code, &capturer);

        assert!(result.contains("executed"));
        let output = capturer.get_output();
        assert!(output.is_empty());
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let capturer = Arc::new(OutputCapturer::new());
        let mut handles = vec![];

        for i in 0..10 {
            let capturer_clone = Arc::clone(&capturer);
            let handle = thread::spawn(move || {
                capturer_clone.capture_stdout(&format!("Thread {}\n", i));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let output = capturer.get_output();
        // Should have captured output from all 10 threads
        assert_eq!(output.stdout.lines().count(), 10);
    }
}
