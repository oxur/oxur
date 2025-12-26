use anyhow::Result;
use colored::Colorize;
use design::state::StateManager;

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

pub fn execute(subcommand: Option<String>, state_mgr: &StateManager) -> Result<()> {
    let cmd = InfoCommand::from_str(subcommand.as_deref());

    match cmd {
        InfoCommand::Overview => show_overview(state_mgr)?,
        InfoCommand::States => show_states()?,
        InfoCommand::Fields => show_fields()?,
        InfoCommand::Config => show_config(state_mgr)?,
        InfoCommand::Stats => show_stats(state_mgr)?,
        InfoCommand::Dirs => show_dirs(state_mgr)?,
    }

    Ok(())
}

fn show_overview(state_mgr: &StateManager) -> Result<()> {
    // Get version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");

    println!();
    println!(
        "{} {}",
        "Oxur Design Documentation Tool (oxd)".cyan().bold(),
        format!("v{}", version).yellow()
    );
    println!();

    // Project info
    println!("{}", "Project:".cyan().bold());
    let docs_dir = state_mgr.docs_dir();
    println!("  {}", docs_dir.display().to_string().white());
    println!();

    // Document counts
    let all_docs = state_mgr.state().all();
    let total = all_docs.len();

    if total > 0 {
        use std::collections::HashMap;
        let mut counts: HashMap<design::DocState, usize> = HashMap::new();

        for doc in all_docs {
            *counts.entry(doc.metadata.state).or_insert(0) += 1;
        }

        println!(
            "{} {} total",
            "Documents:".cyan().bold(),
            total.to_string().yellow()
        );

        // Show top states
        let mut state_counts: Vec<_> = counts.iter().collect();
        state_counts.sort_by(|a, b| b.1.cmp(a.1));

        for (state, count) in state_counts.iter().take(5) {
            println!("  - {} {}", count.to_string().yellow(), state.as_str().white());
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
    println!("  https://github.com/oxur/oxur");
    println!();

    Ok(())
}

fn show_states() -> Result<()> {
    use design::doc::DocState;

    println!();
    println!("{}", "Valid Document States".cyan().bold());
    println!();

    let states = DocState::all_states();

    for state in states {
        // State name
        let state_name = state.as_str();
        println!(
            "  {:<15} {}",
            state_name.yellow().bold(),
            state.description().white()
        );

        // Directory
        let dir = state.directory();
        println!("  {:<15} Directory: {}", "", dir.dimmed());
        println!();
    }

    println!("{}", "Usage:".cyan().bold());
    println!(
        "  Transition a document: {}",
        "oxd transition <doc> <state>".yellow()
    );
    println!("  List by state: {}", "oxd list --state <state>".yellow());
    println!();

    Ok(())
}

fn show_fields() -> Result<()> {
    println!();
    println!("{}", "Supported Frontmatter Fields".cyan().bold());
    println!();

    // Required fields
    println!("{}", "Required Fields:".green().bold());
    println!();

    print_field("number", "Document number (4-digit integer)", Some("42"));
    print_field(
        "title",
        "Document title",
        Some("\"Feature Design: Advanced Caching\""),
    );
    print_field("state", "Current document state", Some("draft"));
    println!(
        "         {} {}",
        "Note:".dimmed(),
        "Valid states: oxd info states".dimmed()
    );
    println!();
    print_field("created", "Creation date (YYYY-MM-DD)", Some("2025-01-15"));
    println!(
        "         {} {}",
        "Note:".dimmed(),
        "Auto-extracted from git if missing".dimmed()
    );
    println!();
    print_field("updated", "Last update date (YYYY-MM-DD)", Some("2025-01-20"));
    println!(
        "         {} {}",
        "Note:".dimmed(),
        "Auto-updated on transitions".dimmed()
    );
    println!();
    print_field("author", "Document author name", Some("\"Jane Developer\""));
    println!(
        "         {} {}",
        "Note:".dimmed(),
        "Auto-extracted from git if missing".dimmed()
    );
    println!();

    // Optional fields
    println!("{}", "Optional Fields:".cyan().bold());
    println!();

    print_field(
        "supersedes",
        "Number of document this supersedes",
        Some("41"),
    );
    println!(
        "         {} {}",
        "Note:".dimmed(),
        "Used when document replaces another".dimmed()
    );
    println!();
    print_field(
        "superseded-by",
        "Number of document that supersedes this",
        Some("43"),
    );
    println!(
        "         {} {}",
        "Note:".dimmed(),
        "Auto-set when document is superseded".dimmed()
    );
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
    println!("{}", "  ---".dimmed());
    println!();

    // Commands
    println!("{}", "Related Commands:".cyan().bold());
    println!(
        "  {}  Add missing headers to a document",
        "oxd add-headers <doc>".yellow()
    );
    println!(
        "  {}  Check all documents for valid headers",
        "oxd validate".yellow()
    );
    println!();

    Ok(())
}

fn print_field(name: &str, description: &str, example: Option<&str>) {
    println!(
        "  {:<15} {}",
        name.yellow().bold(),
        description.white()
    );
    if let Some(ex) = example {
        println!("  {:<15} Example: {}", "", ex.cyan());
    }
}

fn show_config(state_mgr: &StateManager) -> Result<()> {
    use design::config::Config;
    use design::doc::DocState;

    let config = Config::load(Some(state_mgr.docs_dir().to_str().unwrap()))?;

    println!();
    println!("{}", "Configuration".cyan().bold());
    println!();

    // Project paths
    println!("{}", "Project:".green().bold());
    println!(
        "  {:<18} {}",
        "Root:".white(),
        config.project_root.display().to_string().cyan()
    );
    println!(
        "  {:<18} {}",
        "Docs Directory:".white(),
        config.docs_directory.display().to_string().cyan()
    );
    println!();

    // Data sources
    println!("{}", "Data Sources:".green().bold());
    println!(
        "  {:<18} {}",
        "State File:".white(),
        config.state_file.display().to_string().cyan()
    );
    println!();

    // Dustbin
    println!("{}", "Dustbin:".green().bold());
    println!(
        "  {:<18} {}",
        "Directory:".white(),
        config.dustbin_directory.display().to_string().cyan()
    );
    println!(
        "  {:<18} {}",
        "Structure:".white(),
        if config.preserve_dustbin_structure {
            "preserve_state_dirs".green()
        } else {
            "flat".yellow()
        }
    );
    println!();

    // Git integration
    println!("{}", "Git Integration:".green().bold());
    println!(
        "  {:<18} {}",
        "Auto-stage:".white(),
        if config.auto_stage_git {
            "enabled".green()
        } else {
            "disabled".yellow()
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
        println!(
            "  {:<18} → {}",
            state.as_str().white(),
            state.directory().cyan()
        );
    }
    println!();

    // Configuration sources
    println!("{}", "Configuration Sources:".green().bold());
    println!("  1. {} (always present)", "Built-in defaults".dimmed());

    if std::path::PathBuf::from(".oxd/config.toml").exists() {
        println!("  2. {} (if exists)", ".oxd/config.toml".cyan());
    } else {
        println!("  2. {} (not found)", ".oxd/config.toml".dimmed());
    }
    println!();

    // Modification help
    println!("{}", "Modify Configuration:".yellow().bold());
    println!("  Create: {}", ".oxd/config.toml".cyan());
    println!("  Reload: Configuration is read on each command");
    println!();

    Ok(())
}

fn show_stats(state_mgr: &StateManager) -> Result<()> {
    use std::collections::HashMap;
    use design::doc::DocState;

    let all_docs = state_mgr.state().all();

    println!();
    println!("{}", "Project Statistics".cyan().bold());
    println!();

    // Document counts
    println!("{}", "Document Counts:".green().bold());
    println!(
        "  {:<20} {}",
        "Total Documents:".white(),
        all_docs.len().to_string().yellow().bold()
    );
    println!();

    // By state
    let mut state_counts: HashMap<DocState, usize> = HashMap::new();
    for doc in &all_docs {
        *state_counts.entry(doc.metadata.state).or_insert(0) += 1;
    }

    println!("  {}:", "By State".white());

    // Sort by count (descending)
    let mut counts_vec: Vec<_> = state_counts.iter().collect();
    counts_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (state, count) in counts_vec {
        println!(
            "    {:<18} {} docs",
            format!("{}:", state.as_str()).white(),
            count.to_string().yellow()
        );
    }
    println!();

    // Activity metrics (if we have documents)
    if !all_docs.is_empty() {
        println!("{}", "Timeline:".green().bold());

        let oldest = all_docs.iter().min_by_key(|d| &d.metadata.created).unwrap();

        let newest = all_docs.iter().max_by_key(|d| &d.metadata.created).unwrap();

        println!(
            "  {:<20} {:04} ({})",
            "Oldest Document:".white(),
            oldest.metadata.number,
            oldest.metadata.created.to_string().cyan()
        );
        println!(
            "  {:<20} {:04} ({})",
            "Newest Document:".white(),
            newest.metadata.number,
            newest.metadata.created.to_string().cyan()
        );
        println!();
    }

    // Health checks
    println!("{}", "Health:".green().bold());

    // Check for documents without proper metadata
    let docs_with_placeholder = all_docs
        .iter()
        .filter(|d| d.metadata.title == "Untitled Document" || d.metadata.title.is_empty())
        .count();

    if docs_with_placeholder == 0 {
        println!("  ✓ {}", "All documents have titles".green());
    } else {
        println!(
            "  ⚠ {} {}",
            docs_with_placeholder.to_string().yellow(),
            "documents need titles".yellow()
        );
    }

    // Check dustbin
    let in_dustbin = all_docs
        .iter()
        .filter(|d| d.metadata.state.is_in_dustbin())
        .count();

    if in_dustbin > 0 {
        println!(
            "  ⚠ {} {} {}",
            in_dustbin.to_string().yellow(),
            "documents in dustbin".yellow(),
            "(consider permanent deletion)".dimmed()
        );
    } else {
        println!("  ✓ {}", "No documents in dustbin".green());
    }

    println!();

    Ok(())
}

fn show_dirs(state_mgr: &StateManager) -> Result<()> {
    use design::doc::DocState;
    use std::collections::HashMap;

    let all_docs = state_mgr.state().all();

    println!();
    println!("{}", "Directory Structure".cyan().bold());
    println!();

    // Count documents per state
    let mut state_counts: HashMap<DocState, usize> = HashMap::new();
    for doc in &all_docs {
        *state_counts.entry(doc.metadata.state).or_insert(0) += 1;
    }

    // Display tree
    let docs_dir = state_mgr.docs_dir();
    println!("{}/", docs_dir.file_name().unwrap().to_string_lossy());
    println!("├── {}  {}", ".oxd/".cyan(), "(state tracking)".dimmed());
    println!(
        "│   └── {}  {}",
        "state.json".cyan(),
        "(document state)".dimmed()
    );

    // Dustbin
    let dustbin_count = all_docs
        .iter()
        .filter(|d| d.metadata.state.is_in_dustbin())
        .count();

    if dustbin_count > 0 {
        println!(
            "├── {}  {}",
            ".dustbin/".cyan(),
            format!("({} removed docs)", dustbin_count).dimmed()
        );
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
        let count = state_counts.get(state).unwrap_or(&0);
        let is_last = i == states.len() - 1 && dustbin_count == 0;
        let prefix = if is_last { "└── " } else { "├── " };

        println!(
            "{}{}  {}",
            prefix,
            format!("{}/", dir_name).cyan(),
            format!("({} docs)", count).dimmed()
        );
    }

    if dustbin_count == 0 {
        // No final entry needed
    }

    println!();

    // Distribution chart
    if !all_docs.is_empty() {
        println!("{}", "Document Distribution:".cyan().bold());

        let total = all_docs.len().max(1);
        let max_width = 40;

        let mut active_state_counts: Vec<_> = all_docs
            .iter()
            .filter(|d| !d.metadata.state.is_in_dustbin())
            .fold(HashMap::new(), |mut acc, doc| {
                *acc.entry(doc.metadata.state).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .collect();

        active_state_counts.sort_by(|a, b| b.1.cmp(&a.1));

        for (state, count) in active_state_counts {
            let bar_width = (count * max_width / total).max(1);
            let bar = "█".repeat(bar_width);

            println!(
                "  {:<35} {} {}",
                bar.green(),
                count.to_string().yellow(),
                state.as_str().white()
            );
        }

        println!();
    }

    Ok(())
}
