//! CLI argument parsing for oxur-fmt.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Format S-expression files
///
/// oxur-fmt formats S-expression files to make them more readable.
/// By default it reformats files in-place.
#[derive(Parser, Debug)]
#[command(name = "oxur-fmt")]
#[command(version)]
#[command(about = "Format S-expression files", long_about = None)]
#[command(after_help = "Use 'oxur-fmt --help' for more information.")]
pub struct Cli {
    /// Input files to format (use '-' for stdin)
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Run in 'check' mode. Exits with 0 if input is formatted correctly.
    /// Exits with 1 and prints a diff if formatting is required.
    #[arg(long)]
    pub check: bool,

    /// What data to emit and how
    #[arg(long, value_enum, value_name = "MODE")]
    pub emit: Option<EmitMode>,

    /// Backup any modified files
    #[arg(long)]
    pub backup: bool,

    /// Set options from command line. Format: key1=val1,key2=val2
    ///
    /// Example: --config max_width=120,tab_spaces=4
    #[arg(long, value_name = "key1=val1,key2=val2")]
    pub config: Option<String>,

    /// Use colored output (if supported)
    #[arg(long, value_enum, value_name = "MODE")]
    pub color: Option<ColorMode>,

    /// Prints the names of mismatched files that were formatted.
    /// Prints the names of files that would be formatted when used with `--check` mode.
    #[arg(short = 'l', long = "files-with-diff")]
    pub files_with_diff: bool,

    /// Print verbose output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Print less output
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

/// What data to emit and how
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EmitMode {
    /// Modify files in-place (default)
    Files,
    /// Print to stdout
    Stdout,
}

/// Color output mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorMode {
    /// Always use colors
    Always,
    /// Never use colors
    Never,
    /// Automatically detect
    Auto,
}

impl Cli {
    /// Parse command-line arguments from environment.
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Determine the effective emit mode.
    pub fn effective_emit_mode(&self) -> EmitMode {
        if let Some(mode) = self.emit {
            mode
        } else if self.files.is_empty() || self.files.iter().any(|f| f.as_os_str() == "-") {
            // If no files or stdin, default to stdout
            EmitMode::Stdout
        } else {
            // Otherwise default to files
            EmitMode::Files
        }
    }

    /// Check if verbose mode is enabled and quiet is not.
    pub fn is_verbose(&self) -> bool {
        self.verbose && !self.quiet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_mode_default_files() {
        let cli = Cli {
            files: vec![PathBuf::from("test.sexp")],
            check: false,
            emit: None,
            backup: false,
            config: None,
            color: None,
            files_with_diff: false,
            verbose: false,
            quiet: false,
        };

        assert_eq!(cli.effective_emit_mode(), EmitMode::Files);
    }

    #[test]
    fn test_emit_mode_default_stdout() {
        let cli = Cli {
            files: vec![],
            check: false,
            emit: None,
            backup: false,
            config: None,
            color: None,
            files_with_diff: false,
            verbose: false,
            quiet: false,
        };

        assert_eq!(cli.effective_emit_mode(), EmitMode::Stdout);
    }

    #[test]
    fn test_emit_mode_stdin() {
        let cli = Cli {
            files: vec![PathBuf::from("-")],
            check: false,
            emit: None,
            backup: false,
            config: None,
            color: None,
            files_with_diff: false,
            verbose: false,
            quiet: false,
        };

        assert_eq!(cli.effective_emit_mode(), EmitMode::Stdout);
    }

    #[test]
    fn test_emit_mode_explicit() {
        let cli = Cli {
            files: vec![PathBuf::from("test.sexp")],
            check: false,
            emit: Some(EmitMode::Stdout),
            backup: false,
            config: None,
            color: None,
            files_with_diff: false,
            verbose: false,
            quiet: false,
        };

        assert_eq!(cli.effective_emit_mode(), EmitMode::Stdout);
    }

    #[test]
    fn test_is_verbose() {
        let mut cli = Cli {
            files: vec![],
            check: false,
            emit: None,
            backup: false,
            config: None,
            color: None,
            files_with_diff: false,
            verbose: true,
            quiet: false,
        };

        assert!(cli.is_verbose());

        cli.quiet = true;
        assert!(!cli.is_verbose());
    }
}
