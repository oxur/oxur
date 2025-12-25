//! CLI argument parsing

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "oxd")]
#[command(about = "Oxur Design Documentation Manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to docs directory (defaults to ./docs)
    #[arg(short, long, default_value = "docs")]
    pub docs_dir: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all design documents
    List {
        /// Filter by state (draft, under-review, final, superseded)
        #[arg(short, long)]
        state: Option<String>,

        /// Show full details
        #[arg(short, long)]
        verbose: bool,
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
    Validate {
        /// Fix issues automatically where possible
        #[arg(short, long)]
        fix: bool,
    },

    /// Generate the index file (00-index.md)
    Index {
        /// Output format (markdown or json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
}
