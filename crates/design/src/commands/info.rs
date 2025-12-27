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

        println!("{} {} total", "Documents:".cyan().bold(), total.to_string().yellow());

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
        println!("  {:<15} {}", state_name.yellow().bold(), state.description().white());

        // Directory
        let dir = state.directory();
        println!("  {:<15} Directory: {}", "", dir.dimmed());
        println!();
    }

    println!("{}", "Usage:".cyan().bold());
    println!("  Transition a document: {}", "oxd transition <doc> <state>".yellow());
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
    print_field("title", "Document title", Some("\"Feature Design: Advanced Caching\""));
    print_field("state", "Current document state", Some("draft"));
    println!("         {} {}", "Note:".dimmed(), "Valid states: oxd info states".dimmed());
    println!();
    print_field("created", "Creation date (YYYY-MM-DD)", Some("2025-01-15"));
    println!("         {} {}", "Note:".dimmed(), "Auto-extracted from git if missing".dimmed());
    println!();
    print_field("updated", "Last update date (YYYY-MM-DD)", Some("2025-01-20"));
    println!("         {} {}", "Note:".dimmed(), "Auto-updated on transitions".dimmed());
    println!();
    print_field("author", "Document author name", Some("\"Jane Developer\""));
    println!("         {} {}", "Note:".dimmed(), "Auto-extracted from git if missing".dimmed());
    println!();

    // Optional fields
    println!("{}", "Optional Fields:".cyan().bold());
    println!();

    print_field("supersedes", "Number of document this supersedes", Some("41"));
    println!("         {} {}", "Note:".dimmed(), "Used when document replaces another".dimmed());
    println!();
    print_field("superseded-by", "Number of document that supersedes this", Some("43"));
    println!("         {} {}", "Note:".dimmed(), "Auto-set when document is superseded".dimmed());
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
    println!("  {}  Add missing headers to a document", "oxd add-headers <doc>".yellow());
    println!("  {}  Check all documents for valid headers", "oxd validate".yellow());
    println!();

    Ok(())
}

fn print_field(name: &str, description: &str, example: Option<&str>) {
    println!("  {:<15} {}", name.yellow().bold(), description.white());
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
    println!("  {:<18} {}", "Root:".white(), config.project_root.display().to_string().cyan());
    println!(
        "  {:<18} {}",
        "Docs Directory:".white(),
        config.docs_directory.display().to_string().cyan()
    );
    println!();

    // Data sources
    println!("{}", "Data Sources:".green().bold());
    println!("  {:<18} {}", "State File:".white(), config.state_file.display().to_string().cyan());
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
        if config.auto_stage_git { "enabled".green() } else { "disabled".yellow() }
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
        println!("  {:<18} → {}", state.as_str().white(), state.directory().cyan());
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
    use design::doc::DocState;
    use std::collections::HashMap;

    let all_docs = state_mgr.state().all();

    println!();
    println!("{}", "Project Statistics".cyan().bold());
    println!();

    // Document counts
    println!("{}", "Document Counts:".green().bold());
    println!("  {:<20} {}", "Total Documents:".white(), all_docs.len().to_string().yellow().bold());
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
    let in_dustbin = all_docs.iter().filter(|d| d.metadata.state.is_in_dustbin()).count();

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
    println!("│   └── {}  {}", "state.json".cyan(), "(document state)".dimmed());

    // Dustbin
    let dustbin_count = all_docs.iter().filter(|d| d.metadata.state.is_in_dustbin()).count();

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test StateManager with a temporary directory
    fn setup_test_state_manager() -> (TempDir, StateManager) {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path();

        // Create necessary directory structure
        fs::create_dir_all(docs_dir.join(".oxd")).unwrap();
        fs::create_dir_all(docs_dir.join("01-draft")).unwrap();

        // Initialize git repo (required for StateManager)
        std::process::Command::new("git").args(["init"]).current_dir(docs_dir).output().unwrap();

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(docs_dir)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(docs_dir)
            .output()
            .unwrap();

        let state_mgr = StateManager::new(docs_dir).unwrap();

        (temp, state_mgr)
    }

    #[test]
    fn test_info_command_from_str_states() {
        let cmd = InfoCommand::from_str(Some("states"));
        assert!(matches!(cmd, InfoCommand::States));
    }

    #[test]
    fn test_info_command_from_str_fields() {
        let cmd = InfoCommand::from_str(Some("fields"));
        assert!(matches!(cmd, InfoCommand::Fields));
    }

    #[test]
    fn test_info_command_from_str_metadata_alias() {
        let cmd = InfoCommand::from_str(Some("metadata"));
        assert!(matches!(cmd, InfoCommand::Fields));
    }

    #[test]
    fn test_info_command_from_str_config() {
        let cmd = InfoCommand::from_str(Some("config"));
        assert!(matches!(cmd, InfoCommand::Config));
    }

    #[test]
    fn test_info_command_from_str_stats() {
        let cmd = InfoCommand::from_str(Some("stats"));
        assert!(matches!(cmd, InfoCommand::Stats));
    }

    #[test]
    fn test_info_command_from_str_dirs() {
        let cmd = InfoCommand::from_str(Some("dirs"));
        assert!(matches!(cmd, InfoCommand::Dirs));
    }

    #[test]
    fn test_info_command_from_str_structure_alias() {
        let cmd = InfoCommand::from_str(Some("structure"));
        assert!(matches!(cmd, InfoCommand::Dirs));
    }

    #[test]
    fn test_info_command_from_str_none_defaults_to_overview() {
        let cmd = InfoCommand::from_str(None);
        assert!(matches!(cmd, InfoCommand::Overview));
    }

    #[test]
    fn test_info_command_from_str_unknown_defaults_to_overview() {
        let cmd = InfoCommand::from_str(Some("unknown"));
        assert!(matches!(cmd, InfoCommand::Overview));
    }

    #[test]
    fn test_execute_overview() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = execute(None, &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_states() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = execute(Some("states".to_string()), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_fields() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = execute(Some("fields".to_string()), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_config() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = execute(Some("config".to_string()), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_stats() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = execute(Some("stats".to_string()), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_dirs() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = execute(Some("dirs".to_string()), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_overview_executes() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = show_overview(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_states_executes() {
        let result = show_states();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_fields_executes() {
        let result = show_fields();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_config_executes() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = show_config(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_stats_executes() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = show_stats(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_dirs_executes() {
        let (_temp, state_mgr) = setup_test_state_manager();
        let result = show_dirs(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_stats_with_documents() {
        let (temp, mut state_mgr) = setup_test_state_manager();

        // Create a test document
        let doc_path = temp.path().join("01-draft/0001-test.md");
        fs::write(
            &doc_path,
            r#"---
number: 1
title: Test Document
state: Draft
created: 2024-01-01
updated: 2024-01-01
author: Test Author
---

# Test Document
"#,
        )
        .unwrap();

        // Scan to pick up the document
        state_mgr.quick_scan().unwrap();

        // Should execute without error and show stats
        let result = show_stats(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_dirs_with_documents() {
        let (temp, mut state_mgr) = setup_test_state_manager();

        // Create a test document
        let doc_path = temp.path().join("01-draft/0001-test.md");
        fs::write(
            &doc_path,
            r#"---
number: 1
title: Test Document
state: Draft
created: 2024-01-01
updated: 2024-01-01
author: Test Author
---

# Test Document
"#,
        )
        .unwrap();

        // Scan to pick up the document
        state_mgr.quick_scan().unwrap();

        // Should execute without error and show directory structure
        let result = show_dirs(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_overview_with_documents() {
        let (temp, mut state_mgr) = setup_test_state_manager();

        // Create multiple test documents in different states
        fs::create_dir_all(temp.path().join("02-under-review")).unwrap();

        let doc1 = temp.path().join("01-draft/0001-first.md");
        fs::write(
            &doc1,
            r#"---
number: 1
title: First Document
state: Draft
created: 2024-01-01
updated: 2024-01-01
author: Test Author
---

# First Document
"#,
        )
        .unwrap();

        let doc2 = temp.path().join("02-under-review/0002-second.md");
        fs::write(
            &doc2,
            r#"---
number: 2
title: Second Document
state: Under Review
created: 2024-01-02
updated: 2024-01-02
author: Test Author
---

# Second Document
"#,
        )
        .unwrap();

        // Scan to pick up the documents
        state_mgr.quick_scan().unwrap();

        // Should execute without error and show overview with counts
        let result = show_overview(&state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_command_clone() {
        let cmd = InfoCommand::Overview;
        let cloned = cmd.clone();
        assert!(matches!(cloned, InfoCommand::Overview));
    }

    #[test]
    fn test_info_command_debug() {
        let cmd = InfoCommand::States;
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("States"));
    }
}
