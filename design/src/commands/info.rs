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

fn show_stats(_state_mgr: &StateManager) -> Result<()> {
    // Will implement in Task 10.7
    println!("{}", "Stats info not yet implemented".yellow());
    Ok(())
}

fn show_dirs(_state_mgr: &StateManager) -> Result<()> {
    // Will implement in Task 10.8
    println!("{}", "Dirs info not yet implemented".yellow());
    Ok(())
}
