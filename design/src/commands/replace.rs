//! Replace command - replaces a document while preserving its ID
//!
//! Moves the old document to dustbin/overwritten and installs the new one with the same number

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use uuid::Uuid;
use design::doc::{DocState, DesignDoc, DocMetadata, build_yaml_frontmatter};
use design::filename::slugify;
use design::git;
use design::state::StateManager;

/// Execute the replace command
pub fn execute(
    state_mgr: &mut StateManager,
    old_id_or_path: &str,
    new_file_path: &str,
) -> Result<()> {
    println!("{}", "Replacing document...".cyan().bold());
    println!();

    // Step 1: Find old document
    let old_number = if let Ok(num) = old_id_or_path.parse::<u32>() {
        num
    } else {
        let search_path = old_id_or_path;
        let doc = state_mgr.state().all()
            .into_iter()
            .find(|d| d.path.contains(search_path))
            .ok_or_else(|| anyhow::anyhow!("Document '{}' not found", old_id_or_path))?;
        doc.metadata.number
    };

    let old_doc = state_mgr.state().get(old_number)
        .ok_or_else(|| anyhow::anyhow!("Document {} not found", old_number))?;

    let old_title = old_doc.metadata.title.clone();
    let old_state = old_doc.metadata.state;
    let old_path = state_mgr.docs_dir().join(&old_doc.path);

    println!("{}", "Old Document:".cyan().bold());
    println!("  Number: {}", format!("{:04}", old_number).yellow());
    println!("  Title: {}", old_title.white());
    println!("  State: {}", old_state.as_str().cyan());
    println!();

    // Step 2: Load and validate new document
    let new_path = PathBuf::from(new_file_path);
    if !new_path.exists() {
        bail!("New file not found: {}", new_file_path);
    }

    let new_content = std::fs::read_to_string(&new_path)
        .context("Failed to read new file")?;

    println!("{}", "New Document:".cyan().bold());
    println!("  File: {}", new_path.display().to_string().white());

    // Parse new document (may or may not have frontmatter)
    let new_doc = if new_content.trim_start().starts_with("---") {
        match DesignDoc::parse(&new_content, new_path.clone()) {
            Ok(doc) => doc,
            Err(_) => {
                // Partial/broken frontmatter - extract what we can
                println!("  {} Invalid frontmatter, will merge metadata", "⚠".yellow());
                extract_basic_doc(&new_content, &new_path)?
            }
        }
    } else {
        // No frontmatter
        extract_basic_doc(&new_content, &new_path)?
    };

    // Step 3: Merge metadata (preserve critical fields from old)
    let merged_metadata = merge_metadata(&old_doc.metadata, &new_doc.metadata, old_number);

    println!();
    println!("{}", "Merged Metadata:".cyan().bold());
    println!("  ✓ Preserved number: {}", format!("{:04}", merged_metadata.number).yellow());
    println!("  ✓ Preserved created: {}", merged_metadata.created.to_string().green());
    println!("  ✓ New title: {}", merged_metadata.title.white());
    println!("  ✓ New author: {}", merged_metadata.author.white());
    println!();

    // Step 4: Move old document to dustbin as "overwritten"
    let dustbin_dir = state_mgr.docs_dir().join(".dustbin/overwritten");
    std::fs::create_dir_all(&dustbin_dir)
        .context("Failed to create dustbin directory")?;

    let uuid = Uuid::new_v4();
    let uuid_short = uuid.to_string().split('-').next().unwrap().to_string();

    let old_filename = old_path.file_name()
        .context("Invalid old file path")?
        .to_string_lossy();
    let new_dustbin_name = format!("{}-{}",
        old_filename.trim_end_matches(".md"),
        uuid_short
    );
    let dustbin_path = dustbin_dir.join(format!("{}.md", new_dustbin_name));

    println!("{}", "Moving old version to dustbin...".cyan().bold());

    if old_path.exists() {
        git::git_mv(&old_path, &dustbin_path)
            .context("Failed to move old file to dustbin")?;
        println!("  ✓ Moved to: {}", dustbin_path.display().to_string().green());

        // Update old document's frontmatter to mark as overwritten
        if let Ok(content) = std::fs::read_to_string(&dustbin_path) {
            if let Ok(mut doc) = DesignDoc::parse(&content, dustbin_path.clone()) {
                doc.metadata.state = DocState::Overwritten;
                doc.metadata.updated = chrono::Local::now().naive_local().date();
                let updated_content = build_yaml_frontmatter(&doc.metadata) + &doc.content;
                std::fs::write(&dustbin_path, updated_content).ok();
            }
        }
    }

    // Update state tracking for old document
    state_mgr.record_file_move(&old_path, &dustbin_path)?;

    // Step 5: Install new document
    println!();
    println!("{}", "Installing new version...".cyan().bold());

    // Generate new filename based on old document's number
    let new_filename = format!("{:04}-{}.md",
        old_number,
        slugify(&merged_metadata.title)
    );

    // Place in draft directory initially
    let new_dir = state_mgr.docs_dir().join("01-draft");
    std::fs::create_dir_all(&new_dir)?;
    let new_location = new_dir.join(&new_filename);

    // Create content with merged frontmatter
    let new_content_with_frontmatter = format!(
        "{}\n{}",
        build_yaml_frontmatter(&merged_metadata),
        new_doc.content.trim()
    );

    std::fs::write(&new_location, new_content_with_frontmatter)
        .context("Failed to write new document")?;
    println!("  ✓ Created: {}", new_location.display().to_string().green());

    // Stage with git
    git::git_add(&new_location)
        .context("Failed to stage new file")?;
    println!("  ✓ Staged with git");

    // Update state tracking for new document
    state_mgr.record_file_change(&new_location)?;
    println!("  ✓ Updated state tracking");

    println!();
    println!("{}", "Document replaced successfully!".green().bold());
    println!("  Old version: {}", dustbin_path.display().to_string().yellow());
    println!("  New version: {}", new_location.display().to_string().green());
    println!();
    println!("To view: {}", format!("oxd show {}", old_number).yellow());

    Ok(())
}

/// Extract basic document info when frontmatter is missing or invalid
fn extract_basic_doc(content: &str, path: &PathBuf) -> Result<DesignDoc> {
    let title = design::doc::extract_title_from_content(content,
        path.file_name().and_then(|n| n.to_str()).unwrap_or("untitled.md"));

    let author = git::get_author(path);
    let created = chrono::Local::now().naive_local().date();
    let updated = created;

    Ok(DesignDoc {
        metadata: DocMetadata {
            number: 0, // Will be overridden
            title,
            author,
            created,
            updated,
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        },
        content: content.to_string(),
        path: path.clone(),
    })
}

/// Merge metadata from old and new documents
fn merge_metadata(
    old_meta: &DocMetadata,
    new_meta: &DocMetadata,
    preserve_number: u32,
) -> DocMetadata {
    // Always preserve from old document
    let number = preserve_number;
    let created = old_meta.created;

    // Use new document's values if present, otherwise fall back to old
    let title = if !new_meta.title.is_empty() && new_meta.title != "Untitled Document" {
        new_meta.title.clone()
    } else {
        old_meta.title.clone()
    };

    let author = if !new_meta.author.is_empty() && new_meta.author != "Unknown Author" {
        new_meta.author.clone()
    } else {
        old_meta.author.clone()
    };

    let updated = chrono::Local::now().naive_local().date();

    // For optional fields, prefer new but keep old if new is missing
    let supersedes = new_meta.supersedes.or(old_meta.supersedes);
    let superseded_by = new_meta.superseded_by.or(old_meta.superseded_by);

    DocMetadata {
        number,
        title,
        author,
        created,
        updated,
        state: DocState::Draft, // New version always starts as draft
        supersedes,
        superseded_by,
    }
}
