//! List command implementation

use anyhow::Result;
use colored::*;
use design::doc::DocState;
use design::index::DocumentIndex;
use design::theme;

pub fn list_documents(
    index: &DocumentIndex,
    state_filter: Option<String>,
    verbose: bool,
) -> Result<()> {
    let docs = if let Some(state_str) = state_filter {
        match DocState::from_str_flexible(&state_str) {
            Some(state) => index.by_state(state),
            None => {
                eprintln!("{} Unknown state: {}", "ERROR:".red().bold(), state_str);
                eprintln!("Valid states: {}", DocState::all_state_names().join(", "));
                return Ok(());
            }
        }
    } else {
        index.all()
    };

    println!("\n{}", "Design Documents".bold().underline());
    println!();

    for doc in &docs {
        let state = doc.metadata.state.as_str();

        if verbose {
            println!(
                "{} {} [{}]",
                theme::doc_number(doc.metadata.number),
                doc.metadata.title,
                theme::state_badge(state)
            );
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
            println!(
                "{} {} [{}]",
                theme::doc_number(doc.metadata.number),
                doc.metadata.title,
                theme::state_badge(state)
            );
        }
    }

    println!("\nTotal: {} documents\n", docs.len());
    Ok(())
}
