//! Replace command - replaces a document while preserving its ID
//!
//! Moves the old document to dustbin/overwritten and installs the new one with the same number

use anyhow::{bail, Context, Result};
use colored::Colorize;
use design::doc::{build_yaml_frontmatter, DesignDoc, DocMetadata, DocState};
use design::filename::slugify;
use design::git;
use design::state::StateManager;
use std::path::PathBuf;
use uuid::Uuid;

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
        let doc = state_mgr
            .state()
            .all()
            .into_iter()
            .find(|d| d.path.contains(search_path))
            .ok_or_else(|| anyhow::anyhow!("Document '{}' not found", old_id_or_path))?;
        doc.metadata.number
    };

    let old_doc = state_mgr
        .state()
        .get(old_number)
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

    let new_content = std::fs::read_to_string(&new_path).context("Failed to read new file")?;

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
    std::fs::create_dir_all(&dustbin_dir).context("Failed to create dustbin directory")?;

    let uuid = Uuid::new_v4();
    let uuid_short = uuid.to_string().split('-').next().unwrap().to_string();

    let old_filename = old_path.file_name().context("Invalid old file path")?.to_string_lossy();
    let new_dustbin_name = format!("{}-{}", old_filename.trim_end_matches(".md"), uuid_short);
    let dustbin_path = dustbin_dir.join(format!("{}.md", new_dustbin_name));

    println!("{}", "Moving old version to dustbin...".cyan().bold());

    if old_path.exists() {
        git::git_mv(&old_path, &dustbin_path).context("Failed to move old file to dustbin")?;
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
    let new_filename = format!("{:04}-{}.md", old_number, slugify(&merged_metadata.title));

    // Place in draft directory initially
    let new_dir = state_mgr.docs_dir().join("01-draft");
    std::fs::create_dir_all(&new_dir)?;
    let new_location = new_dir.join(&new_filename);

    // Create content with merged frontmatter
    let new_content_with_frontmatter =
        format!("{}\n{}", build_yaml_frontmatter(&merged_metadata), new_doc.content.trim());

    std::fs::write(&new_location, new_content_with_frontmatter)
        .context("Failed to write new document")?;
    println!("  ✓ Created: {}", new_location.display().to_string().green());

    // Stage with git
    git::git_add(&new_location).context("Failed to stage new file")?;
    println!("  ✓ Staged with git");

    // Update state tracking for new document
    state_mgr.record_file_change(&new_location)?;
    println!("  ✓ Updated state tracking");

    // Update the index to reflect the replacement
    println!();
    let index = design::index::DocumentIndex::from_state(state_mgr.state(), state_mgr.docs_dir())
        .context("Failed to create index")?;
    if let Err(e) = crate::commands::update_index::update_index(&index) {
        println!("{} {}", "Warning:".yellow(), "Failed to update index");
        println!("  {}", e);
        println!("  Run 'oxd update-index' manually to sync the index");
    }

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
    let title = design::doc::extract_title_from_content(
        content,
        path.file_name().and_then(|n| n.to_str()).unwrap_or("untitled.md"),
    );

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use design::state::DocumentRecord;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn setup_git_repo(temp_dir: &std::path::Path) {
        use std::process::Command;
        Command::new("git").args(["init"]).current_dir(temp_dir).output().ok();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(temp_dir)
            .output()
            .ok();
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir)
            .output()
            .ok();
    }

    fn create_test_doc(temp: &TempDir, num: u32, title: &str, with_frontmatter: bool) -> PathBuf {
        let content = if with_frontmatter {
            format!(
                "---\nnumber: {}\ntitle: \"{}\"\nauthor: \"Test Author\"\ncreated: 2024-01-01\nupdated: 2024-01-01\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nTest content",
                num, title
            )
        } else {
            format!("# {}\n\nTest content without frontmatter", title)
        };

        let path = temp.path().join(format!("new-doc-{}.md", num));
        fs::write(&path, content).unwrap();
        path
    }

    fn create_test_state_manager(temp: &TempDir) -> StateManager {
        let docs_dir = temp.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        setup_git_repo(temp.path());

        let mut state_mgr = StateManager::new(&docs_dir).unwrap();

        // Add test document
        let meta = DocMetadata {
            number: 1,
            title: "Old Doc".to_string(),
            author: "Old Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Active,
            supersedes: None,
            superseded_by: None,
        };

        let doc_path = docs_dir.join("05-active/0001-old-doc.md");
        fs::create_dir_all(doc_path.parent().unwrap()).unwrap();
        let content = format!(
            "---\n{}\n---\n\nOld content",
            serde_yaml::to_string(&meta).unwrap().trim_start_matches("---\n")
        );
        fs::write(&doc_path, content).unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        state_mgr.state_mut().upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "05-active/0001-old-doc.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        state_mgr
    }

    #[test]
    #[serial]
    fn test_replace_by_number() {
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut state_mgr = create_test_state_manager(&temp);
        let new_doc = create_test_doc(&temp, 2, "New Doc", true);

        let result = execute(&mut state_mgr, "1", new_doc.to_str().unwrap());
        assert!(result.is_ok());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_replace_by_path() {
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut state_mgr = create_test_state_manager(&temp);
        let new_doc = create_test_doc(&temp, 2, "New Doc", true);

        let result = execute(&mut state_mgr, "old-doc", new_doc.to_str().unwrap());
        assert!(result.is_ok());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_replace_old_not_found() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = create_test_state_manager(&temp);
        let new_doc = create_test_doc(&temp, 2, "New Doc", true);

        let result = execute(&mut state_mgr, "999", new_doc.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_replace_new_file_not_found() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = create_test_state_manager(&temp);

        let result = execute(&mut state_mgr, "1", "/nonexistent/file.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    #[serial]
    fn test_replace_without_frontmatter() {
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut state_mgr = create_test_state_manager(&temp);
        let new_doc = create_test_doc(&temp, 2, "New Doc", false);

        let result = execute(&mut state_mgr, "1", new_doc.to_str().unwrap());
        assert!(result.is_ok());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_replace_preserves_number_and_created() {
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut state_mgr = create_test_state_manager(&temp);
        let old_doc = state_mgr.state().get(1).unwrap();
        let old_created = old_doc.metadata.created;

        let new_doc = create_test_doc(&temp, 2, "New Doc", true);
        execute(&mut state_mgr, "1", new_doc.to_str().unwrap()).unwrap();

        let updated_doc = state_mgr.state().get(1).unwrap();
        assert_eq!(updated_doc.metadata.number, 1);
        assert_eq!(updated_doc.metadata.created, old_created);
        assert_eq!(updated_doc.metadata.title, "New Doc");

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_replace_creates_dustbin_overwritten() {
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut state_mgr = create_test_state_manager(&temp);
        let new_doc = create_test_doc(&temp, 2, "New Doc", true);

        execute(&mut state_mgr, "1", new_doc.to_str().unwrap()).unwrap();

        let dustbin = temp.path().join("docs/.dustbin/overwritten");
        assert!(dustbin.exists());
        let files: Vec<_> = fs::read_dir(&dustbin).unwrap().collect();
        assert_eq!(files.len(), 1);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial]
    fn test_replace_new_doc_in_draft() {
        let temp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut state_mgr = create_test_state_manager(&temp);
        let new_doc = create_test_doc(&temp, 2, "New Doc", true);

        execute(&mut state_mgr, "1", new_doc.to_str().unwrap()).unwrap();

        let draft_dir = temp.path().join("docs/01-draft");
        assert!(draft_dir.exists());
        let new_file = draft_dir.join("0001-new-doc.md");
        assert!(new_file.exists());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_merge_metadata_preserves_number() {
        let old_meta = DocMetadata {
            number: 42,
            title: "Old".to_string(),
            author: "Old Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Active,
            supersedes: Some(10),
            superseded_by: None,
        };

        let new_meta = DocMetadata {
            number: 999,
            title: "New".to_string(),
            author: "New Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: Some(50),
        };

        let merged = merge_metadata(&old_meta, &new_meta, 42);

        assert_eq!(merged.number, 42);
        assert_eq!(merged.created, old_meta.created);
        assert_eq!(merged.title, "New");
        assert_eq!(merged.author, "New Author");
        assert_eq!(merged.state, DocState::Draft);
    }

    #[test]
    fn test_merge_metadata_falls_back_to_old() {
        let old_meta = DocMetadata {
            number: 42,
            title: "Old".to_string(),
            author: "Old Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Active,
            supersedes: None,
            superseded_by: None,
        };

        let new_meta = DocMetadata {
            number: 999,
            title: "Untitled Document".to_string(),
            author: "Unknown Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };

        let merged = merge_metadata(&old_meta, &new_meta, 42);

        assert_eq!(merged.title, "Old");
        assert_eq!(merged.author, "Old Author");
    }
}
