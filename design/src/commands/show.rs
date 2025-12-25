//! Show command implementation

use anyhow::{bail, Result};
use colored::*;
use design::index::DocumentIndex;

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
