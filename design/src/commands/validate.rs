//! Validate command implementation

use anyhow::Result;
use colored::*;
use design::index::DocumentIndex;

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
