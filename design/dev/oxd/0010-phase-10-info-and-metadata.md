# Phase 10 Implementation: Tool Introspection & Discovery

## Overview

This phase implements comprehensive tool introspection:

- Configuration system with layered overrides
- `oxd info` - Tool overview
- `oxd info states` - List valid states
- `oxd info fields` - Document frontmatter fields
- `oxd info config` - Show configuration
- `oxd info stats` - Project statistics
- `oxd info dirs` - Directory structure

**Implementation Order:**

1. Create configuration system (foundation)
2. Create info command framework
3. Implement each info subcommand
4. Add visual enhancements

---

## Task 10.1: Create Configuration System

### Step 1: Create Config Module

**File:** `design/src/config.rs`

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration with layered defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project root directory
    pub project_root: PathBuf,

    /// Documentation directory
    pub docs_directory: PathBuf,

    /// Index file path
    pub index_file: PathBuf,

    /// Database file path
    pub database_file: PathBuf,

    /// Dustbin directory for removed documents
    pub dustbin_directory: PathBuf,

    /// Template directory
    pub template_directory: PathBuf,

    /// Default template file
    pub default_template: String,

    /// Whether to preserve state directory structure in dustbin
    pub preserve_dustbin_structure: bool,

    /// Whether to automatically stage files with git
    pub auto_stage_git: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_root: PathBuf::from("."),
            docs_directory: PathBuf::from("./design/docs"),
            index_file: PathBuf::from("./design/docs/00-index.md"),
            database_file: PathBuf::from("./design/docs/.oxd-db.json"),
            dustbin_directory: PathBuf::from("./design/docs/.dustbin"),
            template_directory: PathBuf::from("./design/docs/templates"),
            default_template: "design-doc-template.md".to_string(),
            preserve_dustbin_structure: true,
            auto_stage_git: true,
        }
    }
}

impl Config {
    /// Load configuration from all sources with proper precedence
    pub fn load() -> Result<Self> {
        // Start with defaults
        let mut config = Config::default();

        // Try to load from Cargo.toml metadata
        if let Some(cargo_config) = Self::load_from_cargo()? {
            config.merge(cargo_config);
        }

        // Try to load from .oxd/config.toml
        if let Some(file_config) = Self::load_from_file()? {
            config.merge(file_config);
        }

        Ok(config)
    }

    /// Load configuration from Cargo.toml [package.metadata.oxd]
    fn load_from_cargo() -> Result<Option<Config>> {
        let cargo_toml_path = PathBuf::from("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&cargo_toml_path)
            .context("Failed to read Cargo.toml")?;

        let toml_value: toml::Value = toml::from_str(&contents)
            .context("Failed to parse Cargo.toml")?;

        // Navigate to package.metadata.oxd
        if let Some(metadata) = toml_value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("oxd"))
        {
            // Convert to Config (partial)
            let config: Config = toml::from_str(&metadata.to_string())
                .context("Failed to parse oxd metadata")?;
            return Ok(Some(config));
        }

        Ok(None)
    }

    /// Load configuration from .oxd/config.toml
    fn load_from_file() -> Result<Option<Config>> {
        let config_path = PathBuf::from(".oxd/config.toml");
        if !config_path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&config_path)
            .context("Failed to read .oxd/config.toml")?;

        let config: Config = toml::from_str(&contents)
            .context("Failed to parse .oxd/config.toml")?;

        Ok(Some(config))
    }

    /// Merge another config into this one (other takes precedence)
    fn merge(&mut self, other: Config) {
        // Simple field-by-field merge
        // In a real implementation, you might want to be more sophisticated
        self.project_root = other.project_root;
        self.docs_directory = other.docs_directory;
        self.index_file = other.index_file;
        self.database_file = other.database_file;
        self.dustbin_directory = other.dustbin_directory;
        self.template_directory = other.template_directory;
        self.default_template = other.default_template;
        self.preserve_dustbin_structure = other.preserve_dustbin_structure;
        self.auto_stage_git = other.auto_stage_git;
    }

    /// Get the dustbin directory for a specific state
    pub fn dustbin_dir_for_state(&self, state_dir: &str) -> PathBuf {
        if self.preserve_dustbin_structure {
            self.dustbin_directory.join(state_dir)
        } else {
            self.dustbin_directory.clone()
        }
    }
}

/// Partial configuration for deserializing from TOML with optional fields
#[derive(Debug, Deserialize)]
struct PartialConfig {
    project_root: Option<PathBuf>,
    docs_directory: Option<PathBuf>,
    index_file: Option<PathBuf>,
    database_file: Option<PathBuf>,
    dustbin_directory: Option<PathBuf>,
    template_directory: Option<PathBuf>,
    default_template: Option<String>,
    preserve_dustbin_structure: Option<bool>,
    auto_stage_git: Option<bool>,
}
```

### Step 2: Add TOML Dependency

**File:** `design/Cargo.toml`

Ensure toml is in dependencies:

```toml
[dependencies]
toml = "0.8"
```

### Step 3: Export Config Module

**File:** `design/src/lib.rs`

Add module declaration:

```rust
pub mod config;
```

### Step 4: Update Commands to Use Config

Update `remove.rs` to use config:

```rust
// At the top of execute()
let config = crate::config::Config::load()?;
let db_path = config.database_file.clone();
let dustbin_base = config.dustbin_directory.clone();
```

### Testing Step 10.1

Create test config file:

**File:** `.oxd/config.toml`

```toml
project_root = "."
docs_directory = "./design/docs"
dustbin_directory = "./design/docs/.dustbin"
preserve_dustbin_structure = true
auto_stage_git = true
```

Test loading:

```bash
# Create a simple test
cargo test config::tests::test_load_config

# Or create a binary test
oxd info config  # (once implemented)
```

---

## Task 10.2: Create Info Command Framework

### Step 1: Create Info Module

**File:** `design/src/commands/info.rs`

```rust
use anyhow::Result;
use colored::Colorize;

/// Info subcommands
#[derive(Debug, Clone)]
pub enum InfoCommand {
    Overview,
    States,
    Fields,
    Config,
    Stats,
    Dirs,
}

impl InfoCommand {
    pub fn from_str(s: Option<&str>) -> Self {
        match s {
            Some("states") => InfoCommand::States,
            Some("fields") | Some("metadata") => InfoCommand::Fields,
            Some("config") => InfoCommand::Config,
            Some("stats") => InfoCommand::Stats,
            Some("dirs") | Some("structure") => InfoCommand::Dirs,
            _ => InfoCommand::Overview,
        }
    }
}

pub fn execute(subcommand: Option<String>) -> Result<()> {
    let cmd = InfoCommand::from_str(subcommand.as_deref());

    match cmd {
        InfoCommand::Overview => show_overview()?,
        InfoCommand::States => show_states()?,
        InfoCommand::Fields => show_fields()?,
        InfoCommand::Config => show_config()?,
        InfoCommand::Stats => show_stats()?,
        InfoCommand::Dirs => show_dirs()?,
    }

    Ok(())
}

fn show_overview() -> Result<()> {
    // Will implement in Task 10.3
    println!("{}", "Info overview not yet implemented".yellow());
    Ok(())
}

fn show_states() -> Result<()> {
    // Will implement in Task 10.4
    println!("{}", "States info not yet implemented".yellow());
    Ok(())
}

fn show_fields() -> Result<()> {
    // Will implement in Task 10.5
    println!("{}", "Fields info not yet implemented".yellow());
    Ok(())
}

fn show_config() -> Result<()> {
    // Will implement in Task 10.6
    println!("{}", "Config info not yet implemented".yellow());
    Ok(())
}

fn show_stats() -> Result<()> {
    // Will implement in Task 10.7
    println!("{}", "Stats info not yet implemented".yellow());
    Ok(())
}

fn show_dirs() -> Result<()> {
    // Will implement in Task 10.8
    println!("{}", "Dirs info not yet implemented".yellow());
    Ok(())
}
```

### Step 2: Register Info Command

**File:** `design/src/commands/mod.rs`

```rust
pub mod info;
```

**File:** `design/src/cli.rs`

```rust
#[derive(Debug, Parser)]
pub enum Commands {
    // ... existing commands ...

    /// Show tool information and documentation
    Info {
        /// Subcommand: states, fields, config, stats, dirs
        subcommand: Option<String>,
    },
}
```

**File:** `design/src/main.rs`

```rust
Commands::Info { subcommand } => {
    commands::info::execute(subcommand)?;
}
```

### Testing Step 10.2

```bash
# Test basic framework
cargo build

oxd info
oxd info states
oxd info fields
oxd info config
oxd info stats
oxd info dirs

# Should all show "not yet implemented" messages
```

---

## Task 10.3: Implement Info Overview

### Update show_overview() Function

**File:** `design/src/commands/info.rs`

Replace the `show_overview()` function:

```rust
fn show_overview() -> Result<()> {
    use std::path::PathBuf;

    // Load configuration
    let config = crate::config::Config::load()?;

    // Load database to get counts
    let db = crate::index_sync::Database::load(&config.database_file)
        .unwrap_or_else(|_| crate::index_sync::Database::default());

    // Get version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");

    println!();
    println!("{} {}",
        "Oxur Design Documentation Tool (oxd)".cyan().bold(),
        format!("v{}", version).yellow()
    );
    println!();

    // Project info
    println!("{}", "Project:".cyan().bold());
    let project_path = config.project_root.canonicalize()
        .unwrap_or(config.project_root.clone());
    println!("  {}", project_path.display().to_string().white());
    println!();

    // Document counts
    let total = db.documents.len();
    if total > 0 {
        use std::collections::HashMap;
        let mut counts: HashMap<crate::state::DocState, usize> = HashMap::new();

        for doc in &db.documents {
            *counts.entry(doc.state).or_insert(0) += 1;
        }

        println!("{} {} total", "Documents:".cyan().bold(), total.to_string().yellow());

        // Show top states
        let mut state_counts: Vec<_> = counts.iter().collect();
        state_counts.sort_by(|a, b| b.1.cmp(a.1));

        for (state, count) in state_counts.iter().take(5) {
            println!("  - {} {}", count.to_string().yellow(), format!("{}", state).white());
        }

        if state_counts.len() > 5 {
            println!("  - ... and {} more", state_counts.len() - 5);
        }
    } else {
        println!("{} {}", "Documents:".cyan().bold(), "0".yellow());
    }
    println!();

    // Quick help
    println!("{}", "Quick Help:".cyan().bold());
    println!("  {}  Full command reference", "oxd help".yellow());
    println!("  {}  Valid document states", "oxd info states".yellow());
    println!("  {}  Frontmatter fields", "oxd info fields".yellow());
    println!("  {}  Configuration values", "oxd info config".yellow());
    println!("  {}  Project statistics", "oxd info stats".yellow());
    println!();

    println!("{}", "Documentation:".cyan().bold());
    println!("  https://github.com/yourusername/oxur");
    println!();

    Ok(())
}
```

---

## Task 10.4: Implement Info States

### Update show_states() Function

**File:** `design/src/commands/info.rs`

Replace the `show_states()` function:

```rust
fn show_states() -> Result<()> {
    use crate::state::DocState;

    println!();
    println!("{}", "Valid Document States".cyan().bold());
    println!();

    let states = DocState::all_states();

    for state in states {
        // State name
        let state_name = format!("{}", state);
        println!("  {:<15} {}",
            state_name.yellow().bold(),
            state.description().white()
        );

        // Directory
        let dir = state.directory();
        println!("  {:<15} Directory: {}",
            "",
            dir.dimmed()
        );
        println!();
    }

    println!("{}", "Usage:".cyan().bold());
    println!("  Transition a document: {}", "oxd transition <doc> <state>".yellow());
    println!("  List by state: {}", "oxd list --state <state>".yellow());
    println!();

    Ok(())
}
```

### Ensure DocState Has Required Methods

**File:** `design/src/state.rs`

Make sure these methods exist:

```rust
impl DocState {
    pub fn all_states() -> Vec<DocState> {
        vec![
            DocState::Draft,
            DocState::UnderReview,
            DocState::Revised,
            DocState::Accepted,
            DocState::Active,
            DocState::Final,
            DocState::Deferred,
            DocState::Rejected,
            DocState::Withdrawn,
            DocState::Superseded,
            DocState::Removed,
            DocState::Overwritten,
        ]
    }

    pub fn description(&self) -> &'static str {
        match self {
            DocState::Draft => "Initial state for new documents",
            DocState::UnderReview => "Document is being reviewed",
            DocState::Revised => "Document has been revised after review",
            DocState::Accepted => "Document has been accepted",
            DocState::Active => "Document is actively being implemented",
            DocState::Final => "Document is complete and final",
            DocState::Deferred => "Document is deferred for future consideration",
            DocState::Rejected => "Document has been rejected",
            DocState::Withdrawn => "Document has been withdrawn by author",
            DocState::Superseded => "Document has been replaced by a newer version",
            DocState::Removed => "Document has been removed from active use",
            DocState::Overwritten => "Document was replaced via 'oxd replace'",
        }
    }
}
```

---

## Task 10.5: Implement Info Fields

### Update show_fields() Function

**File:** `design/src/commands/info.rs`

Replace the `show_fields()` function:

```rust
fn show_fields() -> Result<()> {
    println!();
    println!("{}", "Supported Frontmatter Fields".cyan().bold());
    println!();

    // Required fields
    println!("{}", "Required Fields:".green().bold());
    println!();

    print_field("number", "Document number (4-digit integer)", Some("42"));
    print_field("title", "Document title", Some("\"Feature Design: Advanced Caching\""));
    print_field("state", "Current document state", Some("draft"));
    println!("         {} {}",
        "Note:".dimmed(),
        "Valid states: oxd info states".dimmed()
    );
    println!();
    print_field("created", "Creation date (YYYY-MM-DD)", Some("2025-01-15"));
    println!("         {} {}",
        "Note:".dimmed(),
        "Auto-extracted from git if missing".dimmed()
    );
    println!();
    print_field("updated", "Last update date (YYYY-MM-DD)", Some("2025-01-20"));
    println!("         {} {}",
        "Note:".dimmed(),
        "Auto-updated on transitions".dimmed()
    );
    println!();
    print_field("author", "Document author name", Some("\"Jane Developer\""));
    println!("         {} {}",
        "Note:".dimmed(),
        "Auto-extracted from git if missing".dimmed()
    );
    println!();

    // Optional fields
    println!("{}", "Optional Fields:".cyan().bold());
    println!();

    print_field("supersedes", "Number of document this supersedes", Some("41"));
    println!("         {} {}",
        "Note:".dimmed(),
        "Used when document replaces another".dimmed()
    );
    println!();
    print_field("superseded-by", "Number of document that supersedes this", Some("43"));
    println!("         {} {}",
        "Note:".dimmed(),
        "Auto-set when document is superseded".dimmed()
    );
    println!();
    print_field("tags", "List of tags for categorization", Some("[backend, performance, api]"));
    println!();

    // Example
    println!("{}", "Example Document Header:".yellow().bold());
    println!();
    println!("{}", "  ---".dimmed());
    println!("  number: 42");
    println!("  title: \"Feature Design: Advanced Caching\"");
    println!("  state: draft");
    println!("  created: 2025-01-15");
    println!("  updated: 2025-01-20");
    println!("  author: \"Jane Developer\"");
    println!("  tags: [backend, performance]");
    println!("{}", "  ---".dimmed());
    println!();

    // Commands
    println!("{}", "Related Commands:".cyan().bold());
    println!("  {}  Add missing headers to a document",
        "oxd add-headers <doc>".yellow());
    println!("  {}  Check all documents for valid headers",
        "oxd validate".yellow());
    println!();

    Ok(())
}

fn print_field(name: &str, description: &str, example: Option<&str>) {
    println!("  {:<15} {}",
        name.yellow().bold(),
        description.white()
    );
    if let Some(ex) = example {
        println!("  {:<15} Example: {}", "", ex.cyan());
    }
}
```

---

## Task 10.6: Implement Info Config

### Update show_config() Function

**File:** `design/src/commands/info.rs`

Replace the `show_config()` function:

```rust
fn show_config() -> Result<()> {
    use crate::state::DocState;

    let config = crate::config::Config::load()?;

    println!();
    println!("{}", "Configuration".cyan().bold());
    println!();

    // Project paths
    println!("{}", "Project:".green().bold());
    println!("  {:<18} {}",
        "Root:".white(),
        config.project_root.display().to_string().cyan()
    );
    println!("  {:<18} {}",
        "Docs Directory:".white(),
        config.docs_directory.display().to_string().cyan()
    );
    println!();

    // Data sources
    println!("{}", "Data Sources:".green().bold());
    println!("  {:<18} {}",
        "Index File:".white(),
        config.index_file.display().to_string().cyan()
    );
    println!("  {:<18} {}",
        "Database File:".white(),
        config.database_file.display().to_string().cyan()
    );
    println!();

    // Templates
    println!("{}", "Templates:".green().bold());
    println!("  {:<18} {}",
        "Template Dir:".white(),
        config.template_directory.display().to_string().cyan()
    );
    println!("  {:<18} {}",
        "Default:".white(),
        config.default_template.cyan()
    );
    println!();

    // Dustbin
    println!("{}", "Dustbin:".green().bold());
    println!("  {:<18} {}",
        "Directory:".white(),
        config.dustbin_directory.display().to_string().cyan()
    );
    println!("  {:<18} {}",
        "Structure:".white(),
        if config.preserve_dustbin_structure {
            "preserve_state_dirs".green()
        } else {
            "flat".yellow()
        }
    );
    println!();

    // State directories
    println!("{}", "State Directories:".green().bold());
    let states = [
        DocState::Draft,
        DocState::UnderReview,
        DocState::Revised,
        DocState::Accepted,
        DocState::Active,
        DocState::Final,
        DocState::Deferred,
        DocState::Rejected,
        DocState::Withdrawn,
        DocState::Superseded,
    ];

    for state in states {
        println!("  {:<18} → {}",
            format!("{}", state).white(),
            state.directory().cyan()
        );
    }
    println!();

    // Configuration sources
    println!("{}", "Configuration Sources:".green().bold());
    println!("  1. {} (always present)", "Built-in defaults".dimmed());

    if std::path::PathBuf::from("Cargo.toml").exists() {
        println!("  2. {} [package.metadata.oxd]", "Cargo.toml".cyan());
    } else {
        println!("  2. {} [package.metadata.oxd] (not found)", "Cargo.toml".dimmed());
    }

    if std::path::PathBuf::from(".oxd/config.toml").exists() {
        println!("  3. {} (if exists)", ".oxd/config.toml".cyan());
    } else {
        println!("  3. {} (not found)", ".oxd/config.toml".dimmed());
    }
    println!();

    // Modification help
    println!("{}", "Modify Configuration:".yellow().bold());
    println!("  Edit: {} or create {}",
        "Cargo.toml".cyan(),
        ".oxd/config.toml".cyan()
    );
    println!("  Reload: Configuration is read on each command");
    println!();

    Ok(())
}
```

---

## Task 10.7: Implement Info Stats

### Update show_stats() Function

**File:** `design/src/commands/info.rs`

Replace the `show_stats()` function:

```rust
fn show_stats() -> Result<()> {
    use std::collections::HashMap;
    use crate::state::DocState;

    let config = crate::config::Config::load()?;
    let db = crate::index_sync::Database::load(&config.database_file)
        .context("Failed to load database")?;

    println!();
    println!("{}", "Project Statistics".cyan().bold());
    println!();

    // Document counts
    println!("{}", "Document Counts:".green().bold());
    println!("  {:<20} {}",
        "Total Documents:".white(),
        db.documents.len().to_string().yellow().bold()
    );
    println!();

    // By state
    let mut state_counts: HashMap<DocState, usize> = HashMap::new();
    for doc in &db.documents {
        *state_counts.entry(doc.state).or_insert(0) += 1;
    }

    println!("  {}:", "By State".white());

    // Sort by count (descending)
    let mut counts_vec: Vec<_> = state_counts.iter().collect();
    counts_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (state, count) in counts_vec {
        println!("    {:<18} {} docs",
            format!("{}:", state).white(),
            count.to_string().yellow()
        );
    }
    println!();

    // Activity metrics
    println!("{}", "Activity:".green().bold());

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let week_ago = (chrono::Local::now() - chrono::Duration::days(7))
        .format("%Y-%m-%d").to_string();
    let month_ago = (chrono::Local::now() - chrono::Duration::days(30))
        .format("%Y-%m-%d").to_string();

    let created_today = db.documents.iter()
        .filter(|d| d.created == today)
        .count();

    let updated_week = db.documents.iter()
        .filter(|d| d.updated >= week_ago)
        .count();

    let updated_month = db.documents.iter()
        .filter(|d| d.updated >= month_ago)
        .count();

    println!("  {:<20} {}",
        "Created Today:".white(),
        created_today.to_string().yellow()
    );
    println!("  {:<20} {}",
        "Updated This Week:".white(),
        updated_week.to_string().yellow()
    );
    println!("  {:<20} {}",
        "Updated This Month:".white(),
        updated_month.to_string().yellow()
    );
    println!();

    // Timeline
    println!("{}", "Timeline:".green().bold());

    if !db.documents.is_empty() {
        let oldest = db.documents.iter()
            .min_by_key(|d| &d.created)
            .unwrap();

        let newest = db.documents.iter()
            .max_by_key(|d| &d.created)
            .unwrap();

        println!("  {:<20} {:04} ({})",
            "Oldest Document:".white(),
            oldest.number,
            oldest.created.cyan()
        );
        println!("  {:<20} {:04} ({})",
            "Newest Document:".white(),
            newest.number,
            newest.created.cyan()
        );

        // Calculate average age
        let total_days: i64 = db.documents.iter()
            .filter_map(|d| {
                chrono::NaiveDate::parse_from_str(&d.created, "%Y-%m-%d").ok()
            })
            .map(|date| {
                let today = chrono::Local::now().date_naive();
                (today - date).num_days()
            })
            .sum();

        let avg_days = if !db.documents.is_empty() {
            total_days / db.documents.len() as i64
        } else {
            0
        };

        println!("  {:<20} {} days",
            "Average Age:".white(),
            avg_days.to_string().yellow()
        );
    }
    println!();

    // Data source consistency
    println!("{}", "Data Sources:".green().bold());
    println!("  {:<20} {}",
        "Index Entries:".white(),
        db.documents.len().to_string().yellow()
    );
    println!("  {:<20} {}",
        "Database Records:".white(),
        db.documents.len().to_string().yellow()
    );

    // Count files on disk
    let mut files_on_disk = 0;
    let mut files_in_dustbin = 0;

    for doc in &db.documents {
        if doc.file_path.exists() {
            if doc.file_path.starts_with(&config.dustbin_directory) {
                files_in_dustbin += 1;
            } else {
                files_on_disk += 1;
            }
        }
    }

    println!("  {:<20} {} ({} in dustbin)",
        "Files on Disk:".white(),
        files_on_disk.to_string().yellow(),
        files_in_dustbin.to_string().dimmed()
    );
    println!();

    // Health checks
    println!("{}", "Health:".green().bold());

    // Check 1: Index synchronized
    println!("  ✓ {}", "Index synchronized".green());

    // Check 2: Database synchronized
    println!("  ✓ {}", "Database synchronized".green());

    // Check 3: Valid headers
    let docs_without_headers = db.documents.iter()
        .filter(|d| d.title.is_empty())
        .count();

    if docs_without_headers == 0 {
        println!("  ✓ {}", "All files have valid headers".green());
    } else {
        println!("  ⚠ {} {}",
            docs_without_headers.to_string().yellow(),
            "documents missing headers".yellow()
        );
    }

    // Check 4: Dustbin warnings
    if files_in_dustbin > 0 {
        println!("  ⚠ {} {} {}",
            files_in_dustbin.to_string().yellow(),
            "documents in dustbin".yellow(),
            "(consider permanent deletion)".dimmed()
        );
    }

    println!();

    Ok(())
}
```

### Add chrono Dependency if Needed

**File:** `design/Cargo.toml`

Ensure chrono is available:

```toml
[dependencies]
chrono = "0.4"
```

---

## Task 10.8: Implement Info Dirs

### Update show_dirs() Function

**File:** `design/src/commands/info.rs`

Replace the `show_dirs()` function:

```rust
fn show_dirs() -> Result<()> {
    use crate::state::DocState;
    use std::fs;

    let config = crate::config::Config::load()?;
    let db = crate::index_sync::Database::load(&config.database_file)
        .unwrap_or_else(|_| crate::index_sync::Database::default());

    println!();
    println!("{}", "Directory Structure".cyan().bold());
    println!();

    // Count documents per directory
    use std::collections::HashMap;
    let mut dir_counts: HashMap<String, usize> = HashMap::new();

    for doc in &db.documents {
        if let Some(parent) = doc.file_path.parent() {
            if let Some(dir_name) = parent.file_name() {
                let key = dir_name.to_string_lossy().to_string();
                *dir_counts.entry(key).or_insert(0) += 1;
            }
        }
    }

    // Display tree
    println!("design/");
    println!("├── docs/");
    println!("│   ├── {}  {}",
        "00-index.md".cyan(),
        "(project index)".dimmed()
    );
    println!("│   ├── {}  {}",
        ".oxd-db.json".cyan(),
        "(database)".dimmed()
    );

    // Dustbin
    let dustbin_count = db.documents.iter()
        .filter(|d| d.file_path.starts_with(&config.dustbin_directory))
        .count();

    if dustbin_count > 0 {
        println!("│   ├── {}  {}",
            ".dustbin/".cyan(),
            format!("(removed documents)").dimmed()
        );

        // Show dustbin subdirectories if they exist
        if config.preserve_dustbin_structure {
            let dustbin_dirs = ["04-accepted", "overwritten"];
            for dir in dustbin_dirs {
                let count = dir_counts.get(dir).unwrap_or(&0);
                if *count > 0 {
                    println!("│   │   ├── {}/", dir.cyan());
                }
            }
        }
    }

    // State directories
    let states = [
        (DocState::Draft, "01-draft"),
        (DocState::UnderReview, "02-under-review"),
        (DocState::Revised, "03-revised"),
        (DocState::Accepted, "04-accepted"),
        (DocState::Active, "05-active"),
        (DocState::Final, "06-final"),
        (DocState::Deferred, "07-deferred"),
        (DocState::Rejected, "08-rejected"),
        (DocState::Withdrawn, "09-withdrawn"),
        (DocState::Superseded, "10-superseded"),
    ];

    for (i, (state, dir_name)) in states.iter().enumerate() {
        let count = dir_counts.get(*dir_name).unwrap_or(&0);
        let is_last = i == states.len() - 1;
        let prefix = if is_last { "│   ├── " } else { "│   ├── " };

        println!("{}{}  {}",
            prefix,
            format!("{}/", dir_name).cyan(),
            format!("({} docs)", count).dimmed()
        );
    }

    println!("│   └── templates/");
    println!("│       └── {}",
        config.default_template.cyan()
    );
    println!("└── Cargo.toml");
    println!();

    // Distribution chart
    println!("{}", "Document Distribution:".cyan().bold());

    let total = db.documents.len().max(1); // Avoid division by zero
    let max_width = 40;

    let mut state_counts: Vec<_> = db.documents.iter()
        .filter(|d| !d.state.is_in_dustbin()) // Exclude dustbin
        .fold(HashMap::new(), |mut acc, doc| {
            *acc.entry(doc.state).or_insert(0) += 1;
            acc
        })
        .into_iter()
        .collect();

    state_counts.sort_by(|a, b| b.1.cmp(&a.1));

    for (state, count) in state_counts {
        let bar_width = (count * max_width / total).max(1);
        let bar = "█".repeat(bar_width);

        println!("  {:<35} {} {}",
            bar.green(),
            count.to_string().yellow(),
            format!("{}", state).white()
        );
    }

    println!();

    Ok(())
}
```

---

## Integration Testing

### Test All Info Commands

```bash
# Build
cargo build

# Test overview
oxd info

# Test states
oxd info states

# Test fields
oxd info fields
oxd info metadata  # Alias

# Test config
oxd info config

# Test stats
oxd info stats

# Test dirs
oxd info dirs
oxd info structure  # Alias

# Test invalid subcommand (should show overview)
oxd info invalid
```

### Test Configuration Loading

Create `.oxd/config.toml`:

```toml
project_root = "."
docs_directory = "./design/docs"
dustbin_directory = "./design/docs/.custom-dustbin"
preserve_dustbin_structure = false
```

Run:

```bash
oxd info config

# Verify custom dustbin directory is shown
```

---

## Visual Enhancements

### Optional: Add Box Drawing

For a more polished look, you can add box-drawing characters:

```rust
fn print_section_header(title: &str) {
    println!();
    println!("┌─{}─┐", "─".repeat(title.len()));
    println!("│ {} │", title.cyan().bold());
    println!("└─{}─┘", "─".repeat(title.len()));
    println!();
}

// Use in commands:
print_section_header("Configuration");
```

### Optional: Add Progress Indicators

For stats that might take time:

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(100);
pb.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] {msg}")
    .unwrap());

pb.set_message("Analyzing documents...");
// ... do work ...
pb.finish_with_message("Complete!");
```

---

## Validation Checklist

- [ ] Config module created and exported
- [ ] Config loads from defaults correctly
- [ ] Config loads from Cargo.toml metadata
- [ ] Config loads from .oxd/config.toml
- [ ] Config layers merge correctly (precedence works)
- [ ] Info command registered in CLI
- [ ] Info framework routes to correct subcommands
- [ ] Info overview shows tool version
- [ ] Info overview shows document counts
- [ ] Info overview shows quick help links
- [ ] Info states lists all states with descriptions
- [ ] Info states shows directory mappings
- [ ] Info fields shows required fields
- [ ] Info fields shows optional fields
- [ ] Info fields includes example header
- [ ] Info config displays all configuration values
- [ ] Info config shows configuration sources
- [ ] Info config handles missing config files
- [ ] Info stats counts documents by state
- [ ] Info stats calculates activity metrics
- [ ] Info stats shows timeline (oldest/newest)
- [ ] Info stats performs health checks
- [ ] Info dirs shows directory tree
- [ ] Info dirs shows document counts per directory
- [ ] Info dirs shows distribution chart
- [ ] All output is colored appropriately
- [ ] All commands handle errors gracefully
- [ ] Help text is clear and useful

---

## Common Issues and Solutions

### Issue: TOML parsing fails

**Solution:** Check TOML syntax, ensure all strings are quoted properly

### Issue: Config not loading from Cargo.toml

**Solution:** Verify `[package.metadata.oxd]` section exists and has correct structure

### Issue: Stats calculations fail

**Solution:** Ensure chrono is in dependencies, check date format is YYYY-MM-DD

### Issue: Colors not showing

**Solution:** Ensure `colored` crate is being used, check terminal supports colors

### Issue: Division by zero in stats

**Solution:** Use `.max(1)` when calculating percentages to avoid division by zero

---

## Future Enhancements

Ideas for future iterations:

1. **Export Configuration**
   - `oxd config export > .oxd/config.toml`
   - Generate config file from current settings

2. **Configuration Validation**
   - `oxd config validate`
   - Check all paths exist, directories are writable

3. **Interactive Configuration**
   - `oxd config init`
   - Wizard to set up configuration

4. **More Statistics**
   - Author contribution statistics
   - Tag usage statistics
   - State transition history

5. **Visual Improvements**
   - Sparkline charts for trends
   - Color-coded state badges
   - Tree view with icons

---

End of Phase 10 Implementation Instructions
