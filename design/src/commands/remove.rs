//! Remove command - moves documents to dustbin
//!
//! Moves documents to .dustbin directory while preserving git history

use anyhow::{Context, Result};
use colored::Colorize;
use design::doc::DocState;
use design::git;
use design::state::StateManager;
use uuid::Uuid;

/// Execute the remove command
pub fn execute(state_mgr: &mut StateManager, doc_id_or_path: &str) -> Result<()> {
    println!("{}", "Removing document...".cyan().bold());
    println!();

    // Step 1: Find document by number or path
    let doc_number = if let Ok(num) = doc_id_or_path.parse::<u32>() {
        num
    } else {
        // Try to find by path
        let search_path = doc_id_or_path;
        let doc = state_mgr
            .state()
            .all()
            .into_iter()
            .find(|d| d.path.contains(search_path))
            .ok_or_else(|| anyhow::anyhow!("Document '{}' not found", doc_id_or_path))?;
        doc.metadata.number
    };

    // Get document record
    let doc = state_mgr
        .state()
        .get(doc_number)
        .ok_or_else(|| anyhow::anyhow!("Document {} not found", doc_number))?;

    let doc_title = doc.metadata.title.clone();
    let current_state = doc.metadata.state;
    let current_path = state_mgr.docs_dir().join(&doc.path);

    println!("  Document: {} - {}", format!("{:04}", doc_number).yellow(), doc_title.white());
    println!("  Current state: {}", format!("{}", current_state.as_str()).cyan());
    println!();

    // Check if already removed
    if current_state == DocState::Removed {
        println!("{}", "⚠ Document is already removed".yellow());
        println!("  Location: {}", current_path.display());
        return Ok(());
    }

    // Step 2: Prepare dustbin directory
    let dustbin_base = state_mgr.docs_dir().join(".dustbin");
    let state_subdir = current_state.directory();

    // Place in subdirectory based on original state (unless already in dustbin)
    let dustbin_dir = if current_state.is_in_dustbin() {
        dustbin_base.clone()
    } else {
        dustbin_base.join(state_subdir)
    };

    std::fs::create_dir_all(&dustbin_dir).context("Failed to create dustbin directory")?;
    println!("  ✓ Dustbin ready: {}", dustbin_dir.display().to_string().green());

    // Step 3: Generate unique filename with UUID
    let filename = current_path.file_name().context("Invalid file path")?.to_string_lossy();

    let uuid = Uuid::new_v4();
    let uuid_short = uuid.to_string().split('-').next().unwrap().to_string();

    let new_filename = if let Some(stem) = current_path.file_stem() {
        let stem_str = stem.to_string_lossy();
        format!("{}-{}.md", stem_str, uuid_short)
    } else {
        format!("{}-{}", filename.trim_end_matches(".md"), uuid_short)
    };

    let dustbin_path = dustbin_dir.join(&new_filename);
    println!("  ✓ Generated unique name: {}", new_filename.yellow());

    // Step 4: Move file using git
    if current_path.exists() {
        git::git_mv(&current_path, &dustbin_path).context("Failed to move file with git")?;
        println!("  ✓ Moved to dustbin: {}", dustbin_path.display().to_string().green());
    } else {
        println!("  {} File not found on disk: {}", "⚠".yellow(), current_path.display());
    }

    // Step 5: Update state - mark as removed and update path

    // Read and update the document
    if let Ok(content) = std::fs::read_to_string(&dustbin_path) {
        if let Ok(mut parsed_doc) = design::doc::DesignDoc::parse(&content, dustbin_path.clone()) {
            parsed_doc.metadata.state = DocState::Removed;
            parsed_doc.metadata.updated = chrono::Local::now().naive_local().date();

            // Write back with updated frontmatter
            let new_content =
                design::doc::build_yaml_frontmatter(&parsed_doc.metadata) + &parsed_doc.content;
            std::fs::write(&dustbin_path, new_content)
                .context("Failed to update document frontmatter")?;
        }
    }

    // Update state manager
    state_mgr.record_file_move(&current_path, &dustbin_path)?;
    println!("  ✓ Updated state tracking");

    println!();
    println!("{}", "Document removed successfully!".green().bold());
    println!("  Location: {}", dustbin_path.display().to_string().cyan());
    println!();
    println!("To view removed documents: {}", "oxd list --removed".yellow());

    Ok(())
}
