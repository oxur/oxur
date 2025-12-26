//! CLI argument parsing

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "oxd")]
#[command(about = "Oxur Design Documentation Manager", long_about = None)]
#[command(after_help = "Use 'oxd <command> --help' for more information about a command.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to docs directory (defaults to ./docs)
    #[arg(short, long, default_value = "docs")]
    pub docs_dir: String,
}

#[derive(Subcommand)]
pub enum DebugCommands {
    /// Show state information
    State {
        /// Document number (optional, shows all if omitted)
        number: Option<u32>,

        /// Output format (json, table, summary)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Show checksums and dirty files
    Checksums {
        /// Show all files, not just dirty ones
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show repository statistics
    Stats,

    /// Show diff between state and filesystem
    Diff,

    /// Show orphaned entries
    Orphans,

    /// Verify a specific document
    Verify {
        /// Document number
        number: u32,
    },
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all design documents
    #[command(visible_alias = "ls")]
    List {
        /// Filter by state (draft, under-review, revised, accepted, active, final, deferred, rejected, withdrawn, superseded)
        #[arg(short, long)]
        state: Option<String>,

        /// Show full details
        #[arg(short, long)]
        verbose: bool,

        /// Show only removed documents
        #[arg(long)]
        removed: bool,
    },

    /// Show a specific document
    Show {
        /// Document number
        number: u32,

        /// Show only metadata
        #[arg(short, long)]
        metadata_only: bool,
    },

    /// Create a new design document
    New {
        /// Document title
        title: String,

        /// Author name (defaults to git config user.name)
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Validate all documents
    #[command(visible_alias = "check")]
    Validate {
        /// Fix issues automatically where possible
        #[arg(short, long)]
        fix: bool,
    },

    /// Generate the index file (00-index.md)
    #[command(visible_alias = "gen-index")]
    Index {
        /// Output format (markdown or json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },

    /// Add or update YAML frontmatter headers
    #[command(visible_alias = "headers")]
    AddHeaders {
        /// Path to document
        path: String,
    },

    /// Transition document to a new state
    #[command(visible_alias = "mv")]
    Transition {
        /// Path to document
        path: String,

        /// New state (draft, under-review, revised, accepted, active, final, deferred, rejected, withdrawn, superseded)
        state: String,
    },

    /// Move document to directory matching its state header
    #[command(visible_alias = "sync")]
    SyncLocation {
        /// Path to document
        path: String,
    },

    /// Synchronize the index with documents on filesystem
    #[command(visible_alias = "sync-index")]
    UpdateIndex,

    /// Add a new document with full processing
    Add {
        /// Path to document file
        path: String,

        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,

        /// Interactive mode (prompt for metadata)
        #[arg(short, long)]
        interactive: bool,

        /// Auto-yes to prompts (non-interactive with defaults)
        #[arg(short = 'y', long)]
        yes: bool,

        /// Show preview without making changes
        #[arg(long)]
        preview: bool,
    },

    /// Add multiple documents (supports glob patterns)
    AddBatch {
        /// File patterns (e.g., *.md, ~/docs/*.md)
        patterns: Vec<String>,

        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,

        /// Interactive mode (confirm before adding)
        #[arg(short, long)]
        interactive: bool,
    },

    /// Scan filesystem and update document state
    #[command(visible_alias = "rescan")]
    Scan {
        /// Fix issues automatically where possible
        #[arg(short, long)]
        fix: bool,

        /// Show detailed validation output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Debug and introspection commands
    #[command(subcommand)]
    Debug(DebugCommands),

    /// Search documents
    #[command(visible_alias = "grep")]
    Search {
        /// Search query
        query: String,

        /// Filter by state
        #[arg(short, long)]
        state: Option<String>,

        /// Search only in metadata (YAML frontmatter)
        #[arg(short, long)]
        metadata: bool,

        /// Case-sensitive search
        #[arg(short = 'I', long)]
        case_sensitive: bool,
    },

    /// Remove a document (moves to dustbin)
    #[command(visible_alias = "rm")]
    Remove {
        /// Document number or filename
        doc: String,
    },

    /// Replace a document while preserving its ID
    Replace {
        /// Document number or filename to replace
        old: String,

        /// New document file path
        new: String,
    },
}
