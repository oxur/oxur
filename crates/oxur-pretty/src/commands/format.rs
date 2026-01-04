//! Format command implementation for oxur-fmt.

use anyhow::{Context, Result};
use oxur_cli::common::{io, output};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::{Cli, EmitMode};
use crate::config::FormatConfig;
use crate::format_sexp_with_config;

/// Execute the format command.
///
/// This command formats S-expression files according to the provided configuration.
/// It supports multiple modes:
/// - In-place modification (default)
/// - Output to stdout
/// - Backup before modifying
pub fn execute(cli: &Cli, config: FormatConfig) -> Result<()> {
    let emit_mode = cli.effective_emit_mode();

    // Handle stdin -> stdout case
    if cli.files.is_empty() || (cli.files.len() == 1 && cli.files[0].as_os_str() == "-") {
        return format_stdin_to_stdout(&config);
    }

    // Format each file
    for file in &cli.files {
        if file.as_os_str() == "-" {
            format_stdin_to_stdout(&config)?;
        } else {
            format_file(file, emit_mode, cli.backup, &config, cli.is_verbose())?;
        }
    }

    Ok(())
}

/// Format from stdin to stdout.
fn format_stdin_to_stdout(config: &FormatConfig) -> Result<()> {
    let input = io::read_input(&PathBuf::from("-"))?;
    let formatted = format_sexp_with_config(&input, config.clone())
        .context("Failed to format S-expression from stdin")?;

    io::write_output(&formatted, None)?;
    Ok(())
}

/// Format a single file.
fn format_file(
    file: &Path,
    emit_mode: EmitMode,
    backup: bool,
    config: &FormatConfig,
    verbose: bool,
) -> Result<()> {
    if verbose {
        output::info(&format!("Formatting '{}'", file.display()));
    }

    // Read the file
    let input = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file '{}'", file.display()))?;

    // Format the content
    let formatted = format_sexp_with_config(&input, config.clone())
        .with_context(|| format!("Failed to format file '{}'", file.display()))?;

    match emit_mode {
        EmitMode::Stdout => {
            // Print to stdout
            io::write_output(&formatted, None)?;
        }
        EmitMode::Files => {
            // Only write if content changed
            if input != formatted {
                if backup {
                    create_backup(file)?;
                }

                atomic_write_file(file, &formatted)?;

                if verbose {
                    output::success(&format!("Formatted '{}'", file.display()));
                }
            } else if verbose {
                output::info(&format!("'{}' already formatted", file.display()));
            }
        }
    }

    Ok(())
}

/// Create a backup of a file with .bk extension.
fn create_backup(file: &Path) -> Result<()> {
    let backup_path = file.with_extension(
        file.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{}.bk", ext))
            .unwrap_or_else(|| "bk".to_string()),
    );

    fs::copy(file, &backup_path).with_context(|| {
        format!(
            "Failed to create backup '{}' -> '{}'",
            file.display(),
            backup_path.display()
        )
    })?;

    Ok(())
}

/// Atomically write content to a file.
///
/// This function:
/// 1. Writes to a temporary file
/// 2. Preserves the original file's permissions
/// 3. Atomically renames the temp file to replace the original
///
/// This ensures the original file is never left in a partially-written state.
fn atomic_write_file(file: &Path, content: &str) -> Result<()> {
    // Create temp file name
    let temp_path = file.with_extension("tmp");

    // Write to temp file
    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write to temp file '{}'", temp_path.display()))?;

    // Preserve permissions if original file exists
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = fs::metadata(file) {
            let permissions = metadata.permissions();
            fs::set_permissions(&temp_path, permissions).with_context(|| {
                format!(
                    "Failed to set permissions on temp file '{}'",
                    temp_path.display()
                )
            })?;
        }
    }

    // Atomic rename
    fs::rename(&temp_path, file).with_context(|| {
        format!(
            "Failed to rename '{}' to '{}'",
            temp_path.display(),
            file.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_format_file_stdout() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "(a b c)").unwrap();

        let config = FormatConfig::default();
        let result = format_file(&file, EmitMode::Stdout, false, &config, false);

        assert!(result.is_ok());
    }

    #[test]
    fn test_format_file_in_place() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "(Span :lo 0 :hi 10)").unwrap();

        let config = FormatConfig::default();
        format_file(&file, EmitMode::Files, false, &config, false).unwrap();

        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "(Span :lo 0 :hi 10)");
    }

    #[test]
    fn test_format_file_with_backup() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "(a b c)").unwrap();

        let config = FormatConfig::default();
        format_file(&file, EmitMode::Files, true, &config, false).unwrap();

        let backup = temp.path().join("test.sexp.bk");
        assert!(backup.exists());

        let backup_content = fs::read_to_string(&backup).unwrap();
        assert_eq!(backup_content, "(a b c)");
    }

    #[test]
    fn test_atomic_write_preserves_content() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "old content").unwrap();

        atomic_write_file(&file, "new content").unwrap();

        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "new content");

        // Temp file should be gone
        let temp_file = file.with_extension("tmp");
        assert!(!temp_file.exists());
    }

    #[cfg(unix)]
    #[test]
    fn test_atomic_write_preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "content").unwrap();

        // Set specific permissions
        let mut perms = fs::metadata(&file).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file, perms).unwrap();

        // Write new content
        atomic_write_file(&file, "new content").unwrap();

        // Check permissions are preserved
        let new_perms = fs::metadata(&file).unwrap().permissions();
        assert_eq!(new_perms.mode() & 0o777, 0o644);
    }

    #[test]
    fn test_create_backup() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.sexp");
        fs::write(&file, "content").unwrap();

        create_backup(&file).unwrap();

        let backup = temp.path().join("test.sexp.bk");
        assert!(backup.exists());

        let backup_content = fs::read_to_string(&backup).unwrap();
        assert_eq!(backup_content, "content");
    }
}
