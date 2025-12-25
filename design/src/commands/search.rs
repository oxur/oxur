//! Search command implementation

use anyhow::Result;
use colored::*;
use design::doc::DocState;
use design::state::StateManager;
use regex::Regex;
use std::process::Command;

/// Search documents using git grep
pub fn search(
    state_mgr: &StateManager,
    query: &str,
    state_filter: Option<String>,
    metadata_only: bool,
    case_sensitive: bool,
) -> Result<()> {
    println!("{} Searching for: {}\n", "→".cyan(), query.bold());

    // Build git grep command
    let mut cmd = Command::new("git");
    cmd.arg("grep");

    // Options
    cmd.arg("-n"); // Show line numbers
    cmd.arg("--color=never"); // We'll colorize ourselves

    if !case_sensitive {
        cmd.arg("-i"); // Case insensitive
    }

    // Pattern
    cmd.arg(query);

    // Paths to search
    if let Some(state_str) = &state_filter {
        // Filter by state directory
        if let Some(state) = DocState::from_str_flexible(state_str) {
            let state_dir = format!("{}/{}", state_mgr.docs_dir().display(), state.directory());
            cmd.arg("--");
            cmd.arg(format!("{}/*.md", state_dir));
        } else {
            anyhow::bail!("Invalid state: {}", state_str);
        }
    } else {
        // Search all docs
        cmd.arg("--");
        cmd.arg(format!("{}/**/*.md", state_mgr.docs_dir().display()));
    }

    // Execute search
    let output = cmd.output()?;

    if output.status.success() || !output.stdout.is_empty() {
        let results = String::from_utf8_lossy(&output.stdout);

        // Parse and enhance results
        let match_count = display_results(&results, state_mgr, metadata_only, query)?;

        if match_count > 0 {
            println!("\n{} {} matches found", "✓".green(), match_count);
        } else {
            println!("{} No matches found", "→".cyan());
        }
    } else {
        println!("{} No matches found", "→".cyan());
    }

    Ok(())
}

fn display_results(
    results: &str,
    state_mgr: &StateManager,
    metadata_only: bool,
    query: &str,
) -> Result<usize> {
    // Pattern to extract: path:line:content
    let re = Regex::new(r"^([^:]+):(\d+):(.*)$").unwrap();

    let mut current_file = String::new();
    let mut match_count = 0;

    for line in results.lines() {
        if let Some(caps) = re.captures(line) {
            let path = caps.get(1).unwrap().as_str();
            let line_num = caps.get(2).unwrap().as_str();
            let content = caps.get(3).unwrap().as_str();

            // Extract document number from path
            let doc_number = extract_number_from_path(path);

            // Check if in YAML frontmatter (lines < ~15 usually)
            let line_num_val = line_num.parse::<usize>().unwrap_or(999);
            let is_metadata = line_num_val < 15;

            // Skip if metadata_only and not in metadata
            if metadata_only && !is_metadata {
                continue;
            }

            match_count += 1;

            // Print file header on change
            if path != current_file {
                println!();

                // Try to get document title from state
                if let Some(num) = doc_number {
                    if let Some(record) = state_mgr.state().get(num) {
                        println!(
                            "{} {:04} - {} ({})",
                            "→".cyan(),
                            num,
                            record.metadata.title.bold(),
                            record.metadata.state.as_str().dimmed()
                        );
                    } else {
                        println!("{} {}", "→".cyan(), path.bold());
                    }
                } else {
                    println!("{} {}", "→".cyan(), path.bold());
                }

                current_file = path.to_string();
            }

            // Highlight the matched text in content
            let highlighted = highlight_match(content, query);

            // Print the match
            println!("  {}:{}", line_num.dimmed(), highlighted);
        }
    }

    Ok(match_count)
}

fn extract_number_from_path(path: &str) -> Option<u32> {
    let re = Regex::new(r"(\d{4})-").unwrap();
    re.captures(path).and_then(|caps| caps.get(1)).and_then(|m| m.as_str().parse().ok())
}

fn highlight_match(content: &str, query: &str) -> String {
    // Simple case-insensitive highlighting
    let lower_content = content.to_lowercase();
    let lower_query = query.to_lowercase();

    if let Some(pos) = lower_content.find(&lower_query) {
        let before = &content[..pos];
        let matched = &content[pos..pos + query.len()];
        let after = &content[pos + query.len()..];

        format!("{}{}{}", before, matched.red().bold(), after)
    } else {
        content.to_string()
    }
}

/// Search options for advanced searches
#[derive(Default)]
#[allow(dead_code)]
pub struct SearchOptions {
    pub state: Option<String>,
    pub metadata_only: bool,
    pub case_sensitive: bool,
    #[allow(dead_code)]
    pub context_lines: usize,
    #[allow(dead_code)]
    pub regex: bool,
}

/// Search with more advanced options
#[allow(dead_code)]
pub fn search_advanced(
    state_mgr: &StateManager,
    query: &str,
    options: SearchOptions,
) -> Result<()> {
    search(state_mgr, query, options.state, options.metadata_only, options.case_sensitive)
}
