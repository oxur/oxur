//! File I/O utilities for CLI tools
//!
//! Provides helpers for reading from stdin/file and writing to stdout/file,
//! which is a common pattern in CLI tools that support piping.

use anyhow::Result;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// Read input from stdin (if path is "-") or from a file
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use oxur_term::common::io::read_input;
///
/// // Read from file
/// let content = read_input(&PathBuf::from("input.txt"))?;
///
/// // Read from stdin (if user passes "-")
/// let content = read_input(&PathBuf::from("-"))?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn read_input(path: &Path) -> Result<String> {
    if path.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

/// Write output to stdout (if path is None or "-") or to a file
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use oxur_term::common::io::write_output;
///
/// // Write to file
/// write_output("content", Some(&PathBuf::from("output.txt")))?;
///
/// // Write to stdout
/// write_output("content", None)?;
///
/// // Write to stdout (if user passes "-")
/// write_output("content", Some(&PathBuf::from("-")))?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn write_output(content: &str, path: Option<&Path>) -> Result<()> {
    match path {
        Some(p) if p.to_str() == Some("-") => {
            println!("{}", content);
            Ok(())
        }
        Some(p) => {
            fs::write(p, content)?;
            Ok(())
        }
        None => {
            println!("{}", content);
            Ok(())
        }
    }
}

/// Write output to stderr
///
/// Useful for progress messages and diagnostics that shouldn't be mixed with stdout.
///
/// # Examples
///
/// ```no_run
/// use oxur_term::common::io::write_stderr;
///
/// write_stderr("Processing file...")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn write_stderr(message: &str) -> Result<()> {
    writeln!(io::stderr(), "{}", message)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_read_input_from_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let content = read_input(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_read_input_nonexistent_file() {
        let path = PathBuf::from("nonexistent.txt");
        let result = read_input(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_output_to_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("output.txt");

        write_output("test content", Some(&file_path)).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_write_output_to_stdout() {
        // This test just verifies it doesn't panic
        let result = write_output("test", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_output_dash_means_stdout() {
        // Verify that "-" is treated as stdout
        let result = write_output("test", Some(&PathBuf::from("-")));
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_stderr() {
        let result = write_stderr("test message");
        assert!(result.is_ok());
    }
}
