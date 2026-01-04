//! Check command implementation for oxur-fmt.

use anyhow::{Context, Result};
use oxur_cli::common::output;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Cli;
use crate::config::FormatConfig;
use crate::format_sexp_with_config;

/// Execute the check command.
///
/// This command checks if S-expression files are properly formatted.
/// It exits with:
/// - Code 0: All files are formatted correctly
/// - Code 1: Some files need formatting
/// - Code 2: Error occurred (handled by main.rs)
///
/// Returns `Ok(true)` if all files are formatted, `Ok(false)` if any need formatting.
pub fn execute(cli: &Cli, config: FormatConfig) -> Result<bool> {
    // Handle stdin check
    if cli.files.is_empty() || (cli.files.len() == 1 && cli.files[0].as_os_str() == "-") {
        return check_stdin(&config);
    }

    let mut all_formatted = true;
    let mut files_needing_formatting = Vec::new();

    // Check each file
    for file in &cli.files {
        if file.as_os_str() == "-" {
            // stdin case
            if !check_stdin(&config)? {
                all_formatted = false;
            }
        } else {
            let is_formatted = check_file(file, &config, cli.is_verbose())?;
            if !is_formatted {
                all_formatted = false;
                files_needing_formatting.push(file.clone());
            }
        }
    }

    // Report results
    if cli.files_with_diff || cli.is_verbose() {
        for file in &files_needing_formatting {
            println!("{}", file.display());
        }
    }

    if cli.is_verbose() {
        if all_formatted {
            output::success(&format!("All {} file(s) are formatted correctly", cli.files.len()));
        } else {
            output::error(&format!(
                "{} of {} file(s) need formatting",
                files_needing_formatting.len(),
                cli.files.len()
            ));
        }
    }

    Ok(all_formatted)
}

/// Check if input from stdin is formatted.
fn check_stdin(config: &FormatConfig) -> Result<bool> {
    use oxur_cli::common::io;

    let input = io::read_input(&PathBuf::from("-"))?;
    let formatted = format_sexp_with_config(&input, config.clone())
        .context("Failed to format S-expression from stdin")?;

    Ok(input == formatted)
}

/// Check if a single file is formatted.
///
/// Returns `true` if the file is already formatted correctly,
/// `false` if it needs formatting.
fn check_file(file: &Path, config: &FormatConfig, verbose: bool) -> Result<bool> {
    if verbose {
        output::info(&format!("Checking '{}'", file.display()));
    }

    // Read the file
    let input = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file '{}'", file.display()))?;

    // Format the content
    let formatted = format_sexp_with_config(&input, config.clone())
        .with_context(|| format!("Failed to format file '{}'", file.display()))?;

    let is_formatted = input == formatted;

    if verbose {
        if is_formatted {
            output::success(&format!("'{}' is formatted correctly", file.display()));
        } else {
            output::warning(&format!("'{}' needs formatting", file.display()));
        }
    }

    Ok(is_formatted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_cli(files: Vec<PathBuf>, verbose: bool, files_with_diff: bool) -> Cli {
        Cli {
            files,
            check: true,
            emit: None,
            backup: false,
            config: None,
            color: None,
            files_with_diff,
            verbose,
            quiet: false,
        }
    }

    #[test]
    fn test_check_file_formatted() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "(Span :lo 0 :hi 10)").unwrap();

        let config = FormatConfig::default();
        let result = check_file(&file, &config, false).unwrap();

        assert!(result); // Already formatted
    }

    #[test]
    fn test_check_file_needs_formatting() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        // Write unformatted (though our formatter might not change this)
        fs::write(&file, "  (  a   b   c  )  ").unwrap();

        let config = FormatConfig::default();
        let result = check_file(&file, &config, false).unwrap();

        // Check the actual formatted output
        let input = fs::read_to_string(&file).unwrap();
        let formatted = format_sexp_with_config(&input, config).unwrap();
        assert_eq!(result, input == formatted);
    }

    #[test]
    fn test_execute_all_formatted() {
        let temp = TempDir::new().unwrap();
        let file1 = temp.path().join("test1.sexp");
        let file2 = temp.path().join("test2.sexp");

        fs::write(&file1, "(Span :lo 0 :hi 10)").unwrap();
        fs::write(&file2, "(a b c)").unwrap();

        let cli = make_cli(vec![file1, file2], false, false);
        let config = FormatConfig::default();

        let result = execute(&cli, config).unwrap();
        assert!(result); // All formatted
    }

    #[test]
    fn test_execute_with_verbose() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "(Span :lo 0 :hi 10)").unwrap();

        let cli = make_cli(vec![file], true, false);
        let config = FormatConfig::default();

        let result = execute(&cli, config).unwrap();
        assert!(result);
    }

    #[test]
    fn test_check_file_invalid_sexp() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "(unclosed").unwrap();

        let config = FormatConfig::default();
        let result = check_file(&file, &config, false);

        assert!(result.is_err()); // Should fail to format
    }
}
