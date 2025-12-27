# Oxur Workspace Setup Instructions for Claude Code

## Overview

Create a multi-crate Rust workspace for the Oxur project. The workspace will use Cargo's workspace feature to manage multiple related crates in a single repository. The first crate will be `design`, which contains all design documentation and provides a CLI tool for managing those documents.

## Project Structure

```
oxur/
├── Cargo.toml                 # Workspace root manifest
├── README.md                  # Project overview
├── LICENSE                    # License file (suggest MIT or Apache-2.0)
├── .gitignore                 # Git ignore patterns
├── .rustfmt.toml              # Rust formatting config (optional)
├── design/
│   ├── Cargo.toml             # Design crate manifest
│   ├── src/
│   │   ├── main.rs            # CLI entry point
│   │   ├── lib.rs             # Library exports
│   │   ├── doc.rs             # Document type definitions
│   │   ├── index.rs           # Document index management
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs        # List documents
│   │   │   ├── show.rs        # Show document content
│   │   │   ├── new.rs         # Create new document
│   │   │   └── validate.rs    # Validate document format
│   │   └── cli.rs             # CLI argument parsing
│   └── docs/
│       ├── 00-index.md        # Document index (like Zylisp)
│       ├── 01-drafts/
│       ├── 02-under-review/
│       ├── 03-final/
│       └── templates/
│           └── design-doc-template.md
└── .github/                   # GitHub workflows (optional)
    └── workflows/
        └── ci.yml
```

## Step-by-Step Instructions

### 1. Create the Root Workspace

Create `Cargo.toml` at the repository root with the following content:

```toml
[workspace]
resolver = "2"
members = [
    "design",
    # Future crates will be added here:
    # "rast",
    # "lang",
    # "repl",
    # "cli",
]

# Workspace-wide dependencies (shared versions)
[workspace.dependencies]
clap = { version = "4.5", features = ["derive", "cargo"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
walkdir = "2.5"
regex = "1.10"
colored = "2.1"

# Development dependencies shared across all crates
[workspace.dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.10"

# Package metadata shared by all crates
[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Duncan McGreggor <duncan@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/oxur/oxur"
```

### 2. Create Root README.md

```markdown
# Oxur

A Lisp dialect that compiles to Rust with 100% interoperability.

## Overview

Oxur is a Lisp that treats Rust as its compilation target and runtime. Drawing inspiration from Zetalisp, LFE, and Clojure, Oxur provides Lisp's expressiveness and metaprogramming power while leveraging Rust's type system, ownership model, and ecosystem.

## Project Status

**Early Development** - Currently in the design phase.

## Repository Structure

This is a Cargo workspace containing multiple related crates:

- **design/** - Design documentation and CLI tool for managing docs
- **rast/** *(planned)* - Rust AST ↔ S-expression conversion
- **lang/** *(planned)* - Oxur compiler (Stage 1)
- **repl/** *(planned)* - REPL server/client
- **cli/** *(planned)* - User-facing CLI tool

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p design

# Build with optimizations
cargo build --release
```

### Design Documentation CLI

```bash
# List all design documents
cargo run -p design -- list

# Show a specific document
cargo run -p design -- show 0001

# Create a new design document
cargo run -p design -- new "Document Title"

# Validate all documents
cargo run -p design -- validate
```

## Design Documents

All architectural decisions, specifications, and design discussions are documented in the `design/docs/` directory. Start with [00-index.md](design/docs/00-index.md).

## Contributing

*(To be added)*

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
```

### 3. Create .gitignore

```gitignore
# Rust
/target/
**/*.rs.bk
*.pdb
Cargo.lock  # Include this for applications, exclude for libraries

# IDEs
.vscode/
.idea/
*.swp
*.swo
*~
.DS_Store

# Documentation build artifacts
/design/docs/.build/

# CI/CD
.coverage/

# OS
Thumbs.db
```

### 4. Create the design/ Crate

#### design/Cargo.toml

```toml
[package]
name = "design"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Design documentation and management tools for Oxur"

# This is both a library and a binary
[[bin]]
name = "oxd"
path = "src/main.rs"

[lib]
name = "design"
path = "src/lib.rs"

[dependencies]
clap.workspace = true
serde.workspace = true
serde_json.workspace = true
toml.workspace = true
anyhow.workspace = true
thiserror.workspace = true
chrono.workspace = true
walkdir.workspace = true
regex.workspace = true
colored.workspace = true

[dev-dependencies]
assert_cmd.workspace = true
predicates.workspace = true
tempfile.workspace = true
```

#### design/src/lib.rs

```rust
//! Design documentation library for Oxur
//!
//! This library provides types and utilities for managing design documents
//! in the Oxur project.

pub mod doc;
pub mod index;

pub use doc::{DesignDoc, DocMetadata, DocState};
pub use index::DocumentIndex;

/// Re-export common error types
pub use anyhow::{Error, Result};
```

#### design/src/doc.rs

```rust
//! Design document types and parsing

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocError {
    #[error("Invalid document format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid date format: {0}")]
    InvalidDate(String),
}

/// Document state following the Zylisp pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocState {
    Draft,
    UnderReview,
    Final,
    Superseded,
}

impl DocState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocState::Draft => "Draft",
            DocState::UnderReview => "Under Review",
            DocState::Final => "Final",
            DocState::Superseded => "Superseded",
        }
    }
    
    pub fn directory(&self) -> &'static str {
        match self {
            DocState::Draft => "01-drafts",
            DocState::UnderReview => "02-under-review",
            DocState::Final => "03-final",
            DocState::Superseded => "04-superseded",
        }
    }
}

/// Metadata from the YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMetadata {
    pub number: u32,
    pub title: String,
    pub author: String,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    pub state: DocState,
    pub supersedes: Option<u32>,
    pub superseded_by: Option<u32>,
}

/// A complete design document
#[derive(Debug, Clone)]
pub struct DesignDoc {
    pub metadata: DocMetadata,
    pub content: String,
    pub path: PathBuf,
}

impl DesignDoc {
    /// Parse a design document from markdown content
    pub fn parse(content: &str, path: PathBuf) -> Result<Self, DocError> {
        // Look for YAML frontmatter between --- markers
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        
        if parts.len() < 3 {
            return Err(DocError::InvalidFormat(
                "Missing YAML frontmatter".to_string()
            ));
        }
        
        let frontmatter = parts[1].trim();
        let body = parts[2].trim();
        
        // Parse YAML frontmatter
        let metadata: DocMetadata = serde_yaml::from_str(frontmatter)
            .map_err(|e| DocError::InvalidFormat(format!("YAML parse error: {}", e)))?;
        
        Ok(DesignDoc {
            metadata,
            content: body.to_string(),
            path,
        })
    }
    
    /// Get the document filename based on number and state
    pub fn filename(&self) -> String {
        format!("{:04}-{}.md", 
            self.metadata.number,
            self.metadata.title
                .to_lowercase()
                .replace(" ", "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>()
        )
    }
}
```

#### design/src/index.rs

```rust
//! Document index management

use crate::doc::{DesignDoc, DocState};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Manages the collection of design documents
pub struct DocumentIndex {
    docs: HashMap<u32, DesignDoc>,
    docs_dir: PathBuf,
}

impl DocumentIndex {
    /// Create a new index from a documentation directory
    pub fn new(docs_dir: impl AsRef<Path>) -> Result<Self> {
        let docs_dir = docs_dir.as_ref().to_path_buf();
        let mut index = DocumentIndex {
            docs: HashMap::new(),
            docs_dir: docs_dir.clone(),
        };
        
        index.scan()?;
        Ok(index)
    }
    
    /// Scan the docs directory and load all documents
    pub fn scan(&mut self) -> Result<()> {
        self.docs.clear();
        
        for entry in WalkDir::new(&self.docs_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            
            if let Some(ext) = entry.path().extension() {
                if ext != "md" {
                    continue;
                }
            } else {
                continue;
            }
            
            // Skip the index file
            if entry.file_name() == "00-index.md" {
                continue;
            }
            
            let content = fs::read_to_string(entry.path())
                .context(format!("Failed to read {:?}", entry.path()))?;
            
            match DesignDoc::parse(&content, entry.path().to_path_buf()) {
                Ok(doc) => {
                    self.docs.insert(doc.metadata.number, doc);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {:?}: {}", entry.path(), e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a document by number
    pub fn get(&self, number: u32) -> Option<&DesignDoc> {
        self.docs.get(&number)
    }
    
    /// Get all documents
    pub fn all(&self) -> Vec<&DesignDoc> {
        let mut docs: Vec<_> = self.docs.values().collect();
        docs.sort_by_key(|d| d.metadata.number);
        docs
    }
    
    /// Get documents by state
    pub fn by_state(&self, state: DocState) -> Vec<&DesignDoc> {
        let mut docs: Vec<_> = self.docs.values()
            .filter(|d| d.metadata.state == state)
            .collect();
        docs.sort_by_key(|d| d.metadata.number);
        docs
    }
    
    /// Get the next available document number
    pub fn next_number(&self) -> u32 {
        self.docs.keys().max().map(|n| n + 1).unwrap_or(1)
    }
}
```

#### design/src/cli.rs

```rust
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
```

#### design/src/commands/mod.rs

```rust
//! Command implementations

pub mod list;
pub mod show;
pub mod new;
pub mod validate;

pub use list::list_documents;
pub use show::show_document;
pub use new::new_document;
pub use validate::validate_documents;
```

#### design/src/commands/list.rs

```rust
//! List command implementation

use crate::doc::DocState;
use crate::index::DocumentIndex;
use anyhow::Result;
use colored::*;

pub fn list_documents(index: &DocumentIndex, state_filter: Option<String>, verbose: bool) -> Result<()> {
    let docs = if let Some(state_str) = state_filter {
        let state = match state_str.to_lowercase().as_str() {
            "draft" => DocState::Draft,
            "under-review" | "review" => DocState::UnderReview,
            "final" => DocState::Final,
            "superseded" => DocState::Superseded,
            _ => {
                eprintln!("{}", format!("Unknown state: {}", state_str).red());
                return Ok(());
            }
        };
        index.by_state(state)
    } else {
        index.all()
    };
    
    println!("\n{}", "Design Documents".bold().underline());
    println!();
    
    for doc in docs {
        let state_color = match doc.metadata.state {
            DocState::Draft => "yellow",
            DocState::UnderReview => "cyan",
            DocState::Final => "green",
            DocState::Superseded => "red",
        };
        
        let number = format!("{:04}", doc.metadata.number);
        let state = doc.metadata.state.as_str();
        
        if verbose {
            println!("{} {} [{}]", 
                number.bold(), 
                doc.metadata.title,
                state.color(state_color)
            );
            println!("  Author: {}", doc.metadata.author);
            println!("  Created: {} | Updated: {}", 
                doc.metadata.created, 
                doc.metadata.updated
            );
            if let Some(supersedes) = doc.metadata.supersedes {
                println!("  Supersedes: {:04}", supersedes);
            }
            if let Some(superseded_by) = doc.metadata.superseded_by {
                println!("  Superseded by: {:04}", superseded_by);
            }
            println!();
        } else {
            println!("{} {} [{}]", 
                number.bold(), 
                doc.metadata.title,
                state.color(state_color)
            );
        }
    }
    
    println!("\nTotal: {} documents\n", docs.len());
    Ok(())
}
```

#### design/src/commands/show.rs

```rust
//! Show command implementation

use crate::index::DocumentIndex;
use anyhow::{bail, Result};
use colored::*;

pub fn show_document(index: &DocumentIndex, number: u32, metadata_only: bool) -> Result<()> {
    let doc = match index.get(number) {
        Some(d) => d,
        None => bail!("Document {:04} not found", number),
    };
    
    println!("\n{}", format!("Document {:04}", number).bold().underline());
    println!();
    println!("{}: {}", "Title".bold(), doc.metadata.title);
    println!("{}: {}", "Author".bold(), doc.metadata.author);
    println!("{}: {}", "State".bold(), doc.metadata.state.as_str());
    println!("{}: {}", "Created".bold(), doc.metadata.created);
    println!("{}: {}", "Updated".bold(), doc.metadata.updated);
    
    if let Some(supersedes) = doc.metadata.supersedes {
        println!("{}: {:04}", "Supersedes".bold(), supersedes);
    }
    if let Some(superseded_by) = doc.metadata.superseded_by {
        println!("{}: {:04}", "Superseded by".bold(), superseded_by);
    }
    
    if !metadata_only {
        println!("\n{}", "Content:".bold());
        println!("{}", "─".repeat(80));
        println!("{}", doc.content);
        println!("{}", "─".repeat(80));
    }
    
    println!();
    Ok(())
}
```

#### design/src/commands/new.rs

```rust
//! New document command implementation

use crate::index::DocumentIndex;
use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::PathBuf;

pub fn new_document(index: &DocumentIndex, title: String, author: Option<String>) -> Result<()> {
    let number = index.next_number();
    let author = author.unwrap_or_else(|| {
        // Try to get from git config
        std::process::Command::new("git")
            .args(&["config", "user.name"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string())
    });
    
    let today = Local::now().date_naive();
    
    let template = format!(
r#"---
number: {}
title: "{}"
author: {}
created: {}
updated: {}
state: Draft
supersedes: None
superseded-by: None
---

# {}

## Overview

*Brief description of what this document covers*

## Background

*Context and motivation for this design*

## Proposal

*Detailed description of the proposed design*

## Alternatives Considered

*What other approaches were considered and why were they rejected?*

## Implementation Plan

*Steps needed to implement this design*

## Open Questions

*Unresolved questions that need discussion*

## Success Criteria

*How will we know this design is successful?*
"#,
        number, title, author, today, today, title
    );
    
    let filename = format!(
        "{:04}-{}.md",
        number,
        title
            .to_lowercase()
            .replace(" ", "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
    );
    
    let docs_dir = PathBuf::from("design/docs/01-drafts");
    fs::create_dir_all(&docs_dir)?;
    
    let path = docs_dir.join(&filename);
    fs::write(&path, template)?;
    
    println!("Created new design document:");
    println!("  Number: {:04}", number);
    println!("  Title: {}", title);
    println!("  File: {}", path.display());
    
    Ok(())
}
```

#### design/src/commands/validate.rs

```rust
//! Validate command implementation

use crate::index::DocumentIndex;
use anyhow::Result;
use colored::*;

pub fn validate_documents(index: &DocumentIndex, _fix: bool) -> Result<()> {
    println!("\n{}", "Validating design documents...".bold());
    println!();
    
    let docs = index.all();
    let mut errors = 0;
    let mut warnings = 0;
    
    for doc in docs {
        // Check for duplicate numbers (index should prevent this, but verify)
        
        // Check that supersedes/superseded-by references exist
        if let Some(supersedes) = doc.metadata.supersedes {
            if index.get(supersedes).is_none() {
                println!(
                    "{} Document {:04} references non-existent document {:04}",
                    "ERROR:".red().bold(),
                    doc.metadata.number,
                    supersedes
                );
                errors += 1;
            }
        }
        
        if let Some(superseded_by) = doc.metadata.superseded_by {
            if index.get(superseded_by).is_none() {
                println!(
                    "{} Document {:04} references non-existent document {:04}",
                    "ERROR:".red().bold(),
                    doc.metadata.number,
                    superseded_by
                );
                errors += 1;
            }
        }
        
        // Check that created date <= updated date
        if doc.metadata.created > doc.metadata.updated {
            println!(
                "{} Document {:04}: created date ({}) is after updated date ({})",
                "WARNING:".yellow().bold(),
                doc.metadata.number,
                doc.metadata.created,
                doc.metadata.updated
            );
            warnings += 1;
        }
    }
    
    println!();
    if errors == 0 && warnings == 0 {
        println!("{} All documents valid!", "✓".green().bold());
    } else {
        println!("Found {} errors and {} warnings", errors, warnings);
    }
    println!();
    
    Ok(())
}
```

#### design/src/main.rs

```rust
//! Design documentation CLI tool

use anyhow::Result;
use clap::Parser;
use design::index::DocumentIndex;

mod cli;
mod commands;

use cli::{Cli, Commands};
use commands::*;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load the document index
    let index = DocumentIndex::new(&cli.docs_dir)?;
    
    // Execute the command
    match cli.command {
        Commands::List { state, verbose } => {
            list_documents(&index, state, verbose)?;
        }
        Commands::Show { number, metadata_only } => {
            show_document(&index, number, metadata_only)?;
        }
        Commands::New { title, author } => {
            new_document(&index, title, author)?;
        }
        Commands::Validate { fix } => {
            validate_documents(&index, fix)?;
        }
        Commands::Index { format } => {
            eprintln!("Index generation not yet implemented");
            eprintln!("Format: {}", format);
        }
    }
    
    Ok(())
}
```

### 5. Create Initial Documentation Structure

#### design/docs/00-index.md

```markdown
# Oxur Design Documents

This directory contains all design documents, architectural decisions, and technical specifications for the Oxur project.

## Document States

- **Draft** - Work in progress, open for discussion
- **Under Review** - Complete but awaiting approval
- **Final** - Approved and implemented
- **Superseded** - Replaced by a newer document

## Document Index

| Number | Title | State | Updated |
|--------|-------|-------|---------|
| 0001 | "Oxur: A Letter of Intent" | Draft | 2025-12-25 |

## Adding New Documents

Use the design CLI tool:

```bash
cargo run -p design -- new "Your Document Title"
```

This will create a new document in `01-drafts/` with the next available number.

## Document Template

All design documents follow a standard template with YAML frontmatter. See `templates/design-doc-template.md`.
```

#### design/docs/templates/design-doc-template.md

```markdown
---
number: XXXX
title: "Document Title"
author: Your Name
created: YYYY-MM-DD
updated: YYYY-MM-DD
state: Draft
supersedes: None
superseded-by: None
---

# Document Title

## Overview

*Brief description of what this document covers*

## Background

*Context and motivation for this design*

## Proposal

*Detailed description of the proposed design*

## Alternatives Considered

*What other approaches were considered and why were they rejected?*

## Implementation Plan

*Steps needed to implement this design*

## Open Questions

*Unresolved questions that need discussion*

## Success Criteria

*How will we know this design is successful?*
```

### 6. Create Initial Tests

#### design/tests/integration_tests.rs

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("oxd").unwrap();
    cmd.arg("list");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Design Documents"));
}

#[test]
fn test_show_nonexistent() {
    let mut cmd = Command::cargo_bin("oxd").unwrap();
    cmd.arg("show").arg("9999");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_validate_command() {
    let mut cmd = Command::cargo_bin("oxd").unwrap();
    cmd.arg("validate");
    
    cmd.assert()
        .success();
}
```

### 7. Add the Letter of Intent Document

Copy the previously created `oxur-001-letter-of-intent.md` to `design/docs/01-drafts/0001-oxur-letter-of-intent.md`.

### 8. CI/CD Setup (Optional but Recommended)

#### .github/workflows/ci.yml

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features --workspace
      
  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check
      
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-features --workspace -- -D warnings
```

### 9. Format Configuration (Optional)

#### .rustfmt.toml

```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
imports_granularity = "Crate"
```

## Verification Steps

After setup, verify everything works:

```bash
# 1. Build the workspace
cargo build

# 2. Run tests
cargo test

# 3. Try the design CLI
cargo run -p design -- list

# 4. Validate documents
cargo run -p design -- validate

# 5. Format code
cargo fmt --all

# 6. Run clippy
cargo clippy --all-features --workspace
```

## Success Criteria

- [ ] `cargo build` succeeds with no errors
- [ ] `cargo test` passes all tests
- [ ] `cargo run -p design -- list` shows the letter of intent document
- [ ] `cargo run -p design -- show 1` displays the document content
- [ ] `cargo run -p design -- validate` reports no errors
- [ ] Project structure matches the specified layout
- [ ] All necessary dependencies are added to Cargo.toml files
- [ ] CLI tool can create new documents
- [ ] Git repository is initialized with proper .gitignore

## Notes for Claude Code

- Use Rust edition 2021 throughout
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Add appropriate documentation comments (///) for public APIs
- Use `anyhow::Result` for error handling in applications
- Use `thiserror` for custom error types in libraries
- Keep the design CLI simple and focused - it's a tool for managing documents, not a complex application
- The document numbering should be zero-padded to 4 digits (0001, 0002, etc.)
- Follow the same organizational pattern as Zylisp (drafts, under-review, final, superseded)
- Make sure all paths are relative to the workspace root where appropriate
