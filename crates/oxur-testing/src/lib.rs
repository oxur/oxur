//! Shared testing infrastructure for Oxur crates
//!
//! This crate provides common test utilities used across oxur-ast, oxur-lang,
//! and oxur-repl for:
//!
//! - Working with test data files
//! - Extracting expected outputs from comments
//! - Test synchronization via `serial_test`
//!
//! # Test Data Organization
//!
//! Test data files are typically organized as:
//! ```text
//! test-data/
//! ├── examples/
//! │   ├── simple/
//! │   ├── intermediate/
//! │   └── complex/
//! ├── fixtures/       (for oxur-ast)
//! ├── edge-cases/     (for oxur-lang)
//! └── error-cases/    (for oxur-lang)
//! ```
//!
//! # Expected Output Format
//!
//! Test files can include expected outputs in comments:
//! ```lisp
//! (+ 1 2)
//! ;; Expected: 3
//! ```
//!
//! Or for errors:
//! ```lisp
//! (/ 10 0)
//! ;; Expected: Runtime error: division by zero
//! ```

use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

// Re-export serial_test for convenience
//
// Use #[serial_test::serial(env)] attribute on tests that modify environment variables
// to prevent race conditions when tests run in parallel.
//
// Example:
//   #[test]
//   #[serial_test::serial(env)]
//   fn test_with_env_var() {
//       std::env::set_var("MY_VAR", "value");
//       // test code
//   }
pub use serial_test;

/// Error types for test utilities
#[derive(Debug, Error)]
pub enum TestError {
    #[error("Failed to read test file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Test data directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("No expected output found in test file")]
    NoExpectedOutput,

    #[error("Invalid expected output format: {0}")]
    InvalidExpectedFormat(String),
}

pub type Result<T> = std::result::Result<T, TestError>;

/// Represents a test data file with its content and metadata
#[derive(Debug, Clone)]
pub struct TestFile {
    /// Relative path from test-data directory (e.g., "examples/simple/arithmetic.oxur")
    pub relative_path: String,

    /// Full absolute path to the file
    pub absolute_path: PathBuf,

    /// File content
    pub content: String,

    /// Extracted expected outputs (if any)
    pub expected_outputs: Vec<ExpectedOutput>,
}

/// Represents an expected output from a test file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedOutput {
    /// The expected value or error message
    pub value: String,

    /// Whether this is an error expectation
    pub is_error: bool,

    /// Line number where the expectation was found
    pub line_number: usize,
}

/// Load a test data file from a crate's test-data directory
///
/// # Arguments
///
/// * `crate_manifest_dir` - The CARGO_MANIFEST_DIR environment variable value
/// * `relative_path` - Path relative to test-data/ (e.g., "examples/simple/hello.rs")
///
/// # Example
///
/// ```rust,ignore
/// use oxur_testing::load_test_file;
///
/// let test_file = load_test_file(
///     env!("CARGO_MANIFEST_DIR"),
///     "examples/simple/arithmetic.oxur"
/// ).expect("Failed to load test file");
///
/// assert!(!test_file.content.is_empty());
/// ```
pub fn load_test_file(
    crate_manifest_dir: &str,
    relative_path: impl AsRef<Path>,
) -> Result<TestFile> {
    let relative_path = relative_path.as_ref();
    let absolute_path = PathBuf::from(crate_manifest_dir).join("test-data").join(relative_path);

    if !absolute_path.exists() {
        return Err(TestError::DirectoryNotFound(absolute_path));
    }

    let content = std::fs::read_to_string(&absolute_path)?;
    let expected_outputs = extract_expected_outputs(&content);

    Ok(TestFile {
        relative_path: relative_path.to_string_lossy().to_string(),
        absolute_path,
        content,
        expected_outputs,
    })
}

/// Discover all test files matching a pattern in the test-data directory
///
/// # Arguments
///
/// * `crate_manifest_dir` - The CARGO_MANIFEST_DIR environment variable value
/// * `pattern` - Glob pattern for file extension (e.g., "*.oxur", "*.rs", "*.sexp")
///
/// # Example
///
/// ```rust,ignore
/// use oxur_testing::discover_test_files;
///
/// let oxur_files = discover_test_files(
///     env!("CARGO_MANIFEST_DIR"),
///     "*.oxur"
/// ).expect("Failed to discover test files");
///
/// for file in oxur_files {
///     println!("Found: {}", file.relative_path);
/// }
/// ```
pub fn discover_test_files(crate_manifest_dir: &str, pattern: &str) -> Result<Vec<TestFile>> {
    let test_data_dir = PathBuf::from(crate_manifest_dir).join("test-data");

    if !test_data_dir.exists() {
        return Err(TestError::DirectoryNotFound(test_data_dir));
    }

    let mut test_files = Vec::new();

    for entry in WalkDir::new(&test_data_dir).follow_links(true).into_iter().filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Check if file matches pattern
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if matches_pattern(filename, pattern) {
                    let relative_path =
                        path.strip_prefix(&test_data_dir).unwrap().to_string_lossy().to_string();

                    let content = std::fs::read_to_string(path)?;
                    let expected_outputs = extract_expected_outputs(&content);

                    test_files.push(TestFile {
                        relative_path,
                        absolute_path: path.to_path_buf(),
                        content,
                        expected_outputs,
                    });
                }
            }
        }
    }

    Ok(test_files)
}

/// Extract expected outputs from test file content
///
/// Looks for comments in the format:
/// - `;; Expected: <value>` (Lisp-style)
/// - `// Expected: <value>` (Rust-style)
///
/// Error expectations are detected by keywords: "error", "Error", "ERROR"
fn extract_expected_outputs(content: &str) -> Vec<ExpectedOutput> {
    let mut outputs = Vec::new();

    for (line_number, line) in content.lines().enumerate() {
        let line = line.trim();

        // Check for Lisp-style comment
        if let Some(rest) = line.strip_prefix(";;") {
            if let Some(expected) = parse_expected_line(rest) {
                outputs.push(ExpectedOutput {
                    value: expected.value,
                    is_error: expected.is_error,
                    line_number: line_number + 1, // 1-based
                });
            }
        }

        // Check for Rust-style comment
        if let Some(rest) = line.strip_prefix("//") {
            if let Some(expected) = parse_expected_line(rest) {
                outputs.push(ExpectedOutput {
                    value: expected.value,
                    is_error: expected.is_error,
                    line_number: line_number + 1, // 1-based
                });
            }
        }
    }

    outputs
}

/// Parse an expected output from a comment line
fn parse_expected_line(comment: &str) -> Option<ExpectedOutput> {
    let comment = comment.trim();

    if let Some(rest) = comment.strip_prefix("Expected:") {
        let value = rest.trim().to_string();
        let is_error = value.to_lowercase().contains("error");

        Some(ExpectedOutput {
            value,
            is_error,
            line_number: 0, // Will be overwritten by caller
        })
    } else {
        None
    }
}

/// Simple glob pattern matching (supports * wildcard)
fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some(extension) = pattern.strip_prefix("*.") {
        return filename.ends_with(&format!(".{}", extension));
    }

    filename == pattern
}

/// Helper to get the manifest directory at compile time
///
/// # Example
///
/// ```rust,ignore
/// use oxur_testing::{manifest_dir, load_test_file};
///
/// let test_file = load_test_file(
///     manifest_dir!(),
///     "examples/simple/arithmetic.oxur"
/// ).expect("Failed to load");
/// ```
#[macro_export]
macro_rules! manifest_dir {
    () => {
        env!("CARGO_MANIFEST_DIR")
    };
}

/// Helper macro to load a test file with less boilerplate
///
/// # Example
///
/// ```rust,ignore
/// use oxur_testing::test_file;
///
/// let file = test_file!("examples/simple/arithmetic.oxur");
/// assert!(!file.content.is_empty());
/// ```
#[macro_export]
macro_rules! test_file {
    ($path:expr) => {
        $crate::load_test_file(env!("CARGO_MANIFEST_DIR"), $path)
            .unwrap_or_else(|e| panic!("Failed to load test file {}: {}", $path, e))
    };
}

/// Helper macro to discover test files with less boilerplate
///
/// # Example
///
/// ```rust,ignore
/// use oxur_testing::discover_tests;
///
/// let files = discover_tests!("*.oxur");
/// for file in files {
///     println!("Testing: {}", file.relative_path);
/// }
/// ```
#[macro_export]
macro_rules! discover_tests {
    ($pattern:expr) => {
        $crate::discover_test_files(env!("CARGO_MANIFEST_DIR"), $pattern)
            .unwrap_or_else(|e| panic!("Failed to discover test files: {}", e))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expected_line_value() {
        let result = parse_expected_line("Expected: 42");
        assert!(result.is_some());

        let expected = result.unwrap();
        assert_eq!(expected.value, "42");
        assert!(!expected.is_error);
    }

    #[test]
    fn test_parse_expected_line_error() {
        let result = parse_expected_line("Expected: Runtime error: division by zero");
        assert!(result.is_some());

        let expected = result.unwrap();
        assert_eq!(expected.value, "Runtime error: division by zero");
        assert!(expected.is_error);
    }

    #[test]
    fn test_parse_expected_line_no_match() {
        let result = parse_expected_line("This is just a comment");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_expected_outputs_lisp() {
        let content = r#"
(+ 1 2)
;; Expected: 3

(- 10 5)
;; Expected: 5
        "#;

        let outputs = extract_expected_outputs(content);
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].value, "3");
        assert_eq!(outputs[1].value, "5");
    }

    #[test]
    fn test_extract_expected_outputs_rust() {
        let content = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
// Expected: compiles successfully

fn divide(a: i32, b: i32) -> i32 {
    a / b
}
// Expected: Runtime error: division by zero
        "#;

        let outputs = extract_expected_outputs(content);
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].value, "compiles successfully");
        assert!(!outputs[0].is_error);
        assert_eq!(outputs[1].value, "Runtime error: division by zero");
        assert!(outputs[1].is_error);
    }

    #[test]
    fn test_matches_pattern_extension() {
        assert!(matches_pattern("test.oxur", "*.oxur"));
        assert!(matches_pattern("example.rs", "*.rs"));
        assert!(!matches_pattern("test.oxur", "*.rs"));
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("test.txt", "test.txt"));
        assert!(!matches_pattern("test.txt", "other.txt"));
    }

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("anything.txt", "*"));
        assert!(matches_pattern("example.rs", "*"));
    }
}
