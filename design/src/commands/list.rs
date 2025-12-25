//! List command implementation

use anyhow::Result;
use colored::*;
use design::doc::DocState;
use design::index::DocumentIndex;

pub fn list_documents(
    index: &DocumentIndex,
    state_filter: Option<String>,
    verbose: bool,
) -> Result<()> {
    let docs = if let Some(state_str) = state_filter {
        match DocState::from_str_flexible(&state_str) {
            Some(state) => index.by_state(state),
            None => {
                eprintln!(
                    "{} Unknown state: {}",
                    "ERROR:".red().bold(),
                    state_str
                );
                eprintln!(
                    "Valid states: {}",
                    DocState::all_state_names().join(", ")
                );
                return Ok(());
            }
        }
    } else {
        index.all()
    };

    println!("\n{}", "Design Documents".bold().underline());
    println!();

    for doc in &docs {
        let state_color = match doc.metadata.state {
            DocState::Draft => "yellow",
            DocState::UnderReview => "cyan",
            DocState::Revised => "magenta",
            DocState::Accepted => "green",
            DocState::Active => "bright green",
            DocState::Final => "green",
            DocState::Deferred => "blue",
            DocState::Rejected => "red",
            DocState::Withdrawn => "bright black",
            DocState::Superseded => "red",
        };

        let number = format!("{:04}", doc.metadata.number);
        let state = doc.metadata.state.as_str();

        if verbose {
            println!("{} {} [{}]", number.bold(), doc.metadata.title, state.color(state_color));
            println!("  Author: {}", doc.metadata.author);
            println!("  Created: {} | Updated: {}", doc.metadata.created, doc.metadata.updated);
            if let Some(supersedes) = doc.metadata.supersedes {
                println!("  Supersedes: {:04}", supersedes);
            }
            if let Some(superseded_by) = doc.metadata.superseded_by {
                println!("  Superseded by: {:04}", superseded_by);
            }
            println!();
        } else {
            println!("{} {} [{}]", number.bold(), doc.metadata.title, state.color(state_color));
        }
    }

    println!("\nTotal: {} documents\n", docs.len());
    Ok(())
}
