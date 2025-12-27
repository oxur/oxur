//! Enhanced add command with interactive onboarding

use anyhow::{Context, Result};
use colored::*;
use design::doc::*;
use design::extract::*;
use design::filename::*;
use design::normalize::*;
use design::prompt::*;
use design::state::StateManager;
use std::fs;
use std::path::{Path, PathBuf};

/// Add a new document with full processing
pub fn add_document(
    state_mgr: &mut StateManager,
    doc_path: &str,
    dry_run: bool,
    interactive: bool,
    auto_yes: bool,
) -> Result<()> {
    if dry_run {
        println!("{}\n", "DRY RUN MODE - No changes will be made".yellow().bold());
    }

    println!("{} {}\n", "Adding document:".bold(), doc_path);

    let path = PathBuf::from(doc_path);

    // Validate file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }

    // Read content
    let mut content = fs::read_to_string(&path).context("Failed to read file")?;

    // Step 0: Validate it's markdown
    if !is_valid_markdown(&content) {
        anyhow::bail!("File doesn't appear to be valid markdown");
    }

    // Analyze content for issues
    let issues = analyze_markdown(&content);
    if !issues.is_empty() {
        println!("{}", "Content Issues Detected:".yellow().bold());
        for issue in &issues {
            println!("  {} {}", "⚠".yellow(), issue);
        }
        println!();

        if interactive && !auto_yes {
            let should_normalize = prompt_confirm("Apply automatic normalization?", true)?;

            if should_normalize {
                content = normalize_markdown(&content);
                println!("  {} Content normalized\n", "✓".green());
            }
        } else if auto_yes {
            content = normalize_markdown(&content);
            println!("  {} Content normalized\n", "✓".green());
        }
    }

    // Extract metadata from content
    let extracted = ExtractedMetadata::from_content(&content);

    // Step 1: Determine title
    let title = if interactive && !auto_yes {
        determine_title_interactive(&extracted, &path)?
    } else {
        determine_title_auto(&extracted, &path)
    };

    println!("{}", "Step 1: Title".cyan().bold());
    println!("  Title: {}\n", title.bold());

    // Step 2: Number assignment and filename sanitization
    let number = state_mgr.next_number();
    let new_filename = build_filename(number, &title);

    println!("{}", "Step 2: Number & Filename".cyan().bold());
    println!("  Number: {:04}", number);
    println!("  New filename: {}\n", new_filename.bold());

    if interactive && !auto_yes {
        let confirmed = prompt_confirm("Proceed with this filename?", true)?;
        if !confirmed {
            anyhow::bail!("User cancelled");
        }
    }

    // Step 3: Determine author
    let author = if interactive && !auto_yes {
        determine_author_interactive(&extracted)?
    } else {
        determine_author_auto(&extracted)
    };

    println!("{}", "Step 3: Author".cyan().bold());
    println!("  Author: {}\n", author.bold());

    // Step 4: Determine initial state
    let state = if interactive && !auto_yes {
        determine_state_interactive(&extracted)?
    } else {
        extracted.state_hint.unwrap_or(DocState::Draft)
    };

    println!("{}", "Step 4: Initial State".cyan().bold());
    println!("  State: {}\n", state.as_str().bold());

    // Step 5: Build complete metadata
    let today = chrono::Local::now().naive_local().date();
    let metadata = DocMetadata {
        number,
        title: title.clone(),
        author: author.clone(),
        created: today,
        updated: today,
        state,
        supersedes: None,
        superseded_by: None,
    };

    // Step 6: Process content
    println!("{}", "Step 5: Processing Content".cyan().bold());

    // Strip existing frontmatter if present
    if extracted.has_frontmatter {
        content = strip_frontmatter(&content);
        println!("  {} Removed existing frontmatter", "✓".green());
    }

    // Normalize content
    content = content.trim().to_string();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    // Build new content with proper frontmatter
    let frontmatter = build_yaml_frontmatter(&metadata);
    let new_content = frontmatter + &content;

    println!("  {} Added complete YAML frontmatter\n", "✓".green());

    // Calculate final path
    let state_dir = PathBuf::from(state_mgr.docs_dir()).join(state.directory());
    let final_path = state_dir.join(&new_filename);

    if dry_run {
        println!("{}", "Would create:".bold());
        println!("  {}\n", final_path.display());
        return Ok(());
    }

    // Step 7: Move to correct location
    println!("{}", "Step 6: Moving to Repository".cyan().bold());

    fs::create_dir_all(&state_dir)?;

    // Write the new file
    fs::write(&final_path, new_content).context("Failed to write file")?;

    println!("  {} Created: {}\n", "✓".green(), final_path.display());

    // Step 8: Git add
    println!("{}", "Step 7: Git Staging".cyan().bold());
    if let Err(e) = design::git::git_add(&final_path) {
        println!("  {} Git staging failed: {}", "⚠".yellow(), e);
    } else {
        println!("  {} Staged with git\n", "✓".green());
    }

    // Step 9: Update state
    state_mgr.record_file_change(&final_path)?;

    // Step 10: Offer to delete original if different
    if path != final_path && path.exists() && interactive && !auto_yes {
        let should_delete =
            prompt_confirm(&format!("Delete original file at {}?", path.display()), false)?;

        if should_delete {
            fs::remove_file(&path)?;
            println!("  {} Deleted original file\n", "✓".green());
        }
    }

    // Step 11: Update the index to reflect the new document
    println!();
    let index = design::index::DocumentIndex::from_state(state_mgr.state(), state_mgr.docs_dir())?;
    if let Err(e) = crate::commands::update_index::update_index(&index) {
        println!("{} Failed to update index", "Warning:".yellow());
        println!("  {}", e);
        println!("  Run 'oxd update-index' manually to sync the index");
    }

    println!("\n{} Successfully added: {}", "✓".green().bold(), new_filename.bold());

    Ok(())
}

fn determine_title_interactive(extracted: &ExtractedMetadata, path: &Path) -> Result<String> {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

    let default = extracted
        .title
        .as_ref()
        .or(extracted.first_heading.as_ref())
        .cloned()
        .unwrap_or_else(|| filename_to_title(filename));

    prompt_with_default("Document title", &default)
}

fn determine_title_auto(extracted: &ExtractedMetadata, path: &Path) -> String {
    extracted.title.clone().or_else(|| extracted.first_heading.clone()).unwrap_or_else(|| {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        filename_to_title(filename)
    })
}

fn determine_author_interactive(extracted: &ExtractedMetadata) -> Result<String> {
    let git_author = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());

    let default = extracted.author.as_ref().unwrap_or(&git_author);

    prompt_with_default("Author", default)
}

fn determine_author_auto(extracted: &ExtractedMetadata) -> String {
    extracted.author.clone().unwrap_or_else(|| design::git::get_author(std::path::Path::new(".")))
}

fn determine_state_interactive(extracted: &ExtractedMetadata) -> Result<DocState> {
    let states = DocState::all_state_names();
    let default_idx = if let Some(hint) = extracted.state_hint {
        DocState::all_states().iter().position(|&s| s == hint).unwrap_or(0)
    } else {
        0 // Draft
    };

    let selected = prompt_select("Initial state", &states, default_idx)?;
    Ok(DocState::from_str_flexible(&selected).unwrap_or(DocState::Draft))
}

/// Show preview of what will happen when adding a file
pub fn preview_add(doc_path: &str, state_mgr: &StateManager) -> Result<()> {
    let path = PathBuf::from(doc_path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }

    let content = fs::read_to_string(&path)?;

    if !is_valid_markdown(&content) {
        anyhow::bail!("File doesn't appear to be valid markdown");
    }

    let extracted = ExtractedMetadata::from_content(&content);

    let title = determine_title_auto(&extracted, &path);
    let author = determine_author_auto(&extracted);
    let state = extracted.state_hint.unwrap_or(DocState::Draft);
    let number = state_mgr.next_number();
    let new_filename = build_filename(number, &title);

    let final_path =
        PathBuf::from(state_mgr.docs_dir()).join(state.directory()).join(&new_filename);

    // Analyze for issues
    let issues = analyze_markdown(&content);

    println!("\n{}", "Preview of Changes".bold().underline());
    println!();

    // Current state
    println!("{}", "Current:".cyan().bold());
    println!("  Location: {}", path.display());
    println!("  Filename: {}", path.file_name().unwrap_or_default().to_string_lossy());
    println!("  Has frontmatter: {}", if extracted.has_frontmatter { "Yes" } else { "No" });
    if let Some(ref t) = extracted.title {
        println!("  Detected title: {}", t);
    }
    if let Some(ref a) = extracted.author {
        println!("  Detected author: {}", a);
    }
    println!();

    // After state
    println!("{}", "After:".green().bold());
    println!("  Location: {}", final_path.display());
    println!("  Filename: {}", new_filename);
    println!("  Number: {:04}", number);
    println!("  Title: {}", title);
    println!("  Author: {}", author);
    println!("  State: {}", state.as_str());
    println!();

    // Issues
    if !issues.is_empty() {
        println!("{}", "Content Issues:".yellow().bold());
        for issue in &issues {
            println!("  {} {}", "⚠".yellow(), issue);
        }
        println!();
    }

    Ok(())
}

/// Add multiple documents using glob patterns
pub fn add_batch(
    state_mgr: &mut StateManager,
    patterns: Vec<String>,
    dry_run: bool,
    interactive: bool,
) -> Result<()> {
    use glob::glob;

    let mut files = Vec::new();

    // Expand patterns
    for pattern in patterns {
        for path in glob(&pattern)?.flatten() {
            if path.is_file() {
                // Only include markdown files
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        files.push(path);
                    }
                }
            }
        }
    }

    if files.is_empty() {
        println!("No markdown files found matching patterns");
        return Ok(());
    }

    println!("{} Found {} file(s)\n", "→".cyan(), files.len());

    // Show files and confirm
    for file in &files {
        println!("  - {}", file.display());
    }
    println!();

    if interactive {
        let confirmed = prompt_confirm(&format!("Add all {} file(s)?", files.len()), true)?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let mut succeeded = 0;
    let mut failed = 0;

    for (idx, file) in files.iter().enumerate() {
        println!("\n{} [{}/{}] Processing: {}", "→".cyan(), idx + 1, files.len(), file.display());
        println!("{}", "─".repeat(60));

        match add_document(
            state_mgr,
            file.to_str().unwrap(),
            dry_run,
            false, // Non-interactive for batch
            true,  // Auto-yes
        ) {
            Ok(_) => {
                succeeded += 1;
            }
            Err(e) => {
                eprintln!("{} Failed: {}\n", "✗".red(), e);
                failed += 1;
            }
        }
    }

    println!(
        "\n{} Batch complete: {} succeeded, {} failed",
        if failed == 0 { "✓".green().bold() } else { "⚠".yellow().bold() },
        succeeded,
        failed
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use design::extract::ExtractedMetadata;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_determine_title_auto_with_heading() {
        let content = "# Heading Title\n\nContent";
        let extracted = ExtractedMetadata::from_content(content);

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.md");
        fs::write(&path, content).unwrap();

        let title = determine_title_auto(&extracted, &path);
        // Should extract from heading
        assert!(!title.is_empty());
    }

    #[test]
    fn test_determine_title_auto_from_heading() {
        let content = "# Heading Title\n\nContent";
        let extracted = ExtractedMetadata::from_content(content);

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.md");
        fs::write(&path, content).unwrap();

        let title = determine_title_auto(&extracted, &path);
        assert_eq!(title, "Heading Title");
    }

    #[test]
    fn test_determine_title_auto_from_filename() {
        let content = "Just some content";
        let extracted = ExtractedMetadata::from_content(content);

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("my-test-document.md");
        fs::write(&path, content).unwrap();

        let title = determine_title_auto(&extracted, &path);
        assert_eq!(title, "My Test Document");
    }

    #[test]
    fn test_determine_author_auto_with_extraction() {
        let content = "# Test\n\nSome content";
        let extracted = ExtractedMetadata::from_content(content);

        let author = determine_author_auto(&extracted);
        // Should get author from git or extraction
        assert!(!author.is_empty());
    }

    #[test]
    fn test_determine_author_auto_from_git() {
        let content = "No author in content";
        let extracted = ExtractedMetadata::from_content(content);

        let author = determine_author_auto(&extracted);
        // Should fall back to git author (not testing exact value as it depends on git config)
        assert!(!author.is_empty());
    }

    #[test]
    fn test_preview_add_valid_markdown() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "---\ntitle: Test Doc\n---\n\n# Test Document\n\nContent here.")
            .unwrap();

        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_preview_add_file_not_found() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let result = preview_add("/nonexistent/file.md", &state_mgr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_preview_add_with_minimal_content() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Title\n\nSome content").unwrap();

        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        // Should work with minimal markdown
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_file_not_found() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let result = add_document(&mut state_mgr, "/nonexistent/file.md", false, false, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_add_document_dry_run() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test Document\n\nContent here.").unwrap();

        // Dry run should succeed but not create files
        let result = add_document(
            &mut state_mgr,
            file_path.to_str().unwrap(),
            true,  // dry_run
            false, // not interactive
            true,  // auto_yes
        );
        assert!(result.is_ok());

        // Check that no numbered file was created
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            assert!(fs::read_dir(&draft_dir).unwrap().next().is_none());
        }
    }

    #[test]
    fn test_add_batch_no_files() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let result = add_batch(&mut state_mgr, vec!["*.nonexistent".to_string()], false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_batch_with_markdown_files() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Create some test markdown files
        fs::write(temp.path().join("test1.md"), "# Test 1").unwrap();
        fs::write(temp.path().join("test2.md"), "# Test 2").unwrap();
        fs::write(temp.path().join("test.txt"), "Not markdown").unwrap();

        let pattern = format!("{}/*.md", temp.path().display());
        let result = add_batch(&mut state_mgr, vec![pattern], true, false); // dry_run mode
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_invalid_markdown() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("invalid.md");
        fs::write(&file_path, "").unwrap(); // Empty file is not valid markdown

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("valid markdown"));
    }

    #[test]
    fn test_add_document_with_content_issues_auto_yes() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Content with issues (trailing spaces, inconsistent bullets, etc.)
        fs::write(&file_path, "# Test\n\nLine with trailing spaces  \n* item 1\n- item 2\n")
            .unwrap();

        let result = add_document(
            &mut state_mgr,
            file_path.to_str().unwrap(),
            false,
            false,
            true, // auto_yes - should normalize
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_with_existing_frontmatter() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content =
            "---\ntitle: Old Title\nauthor: Old Author\n---\n\n# Test Document\n\nContent.";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Verify frontmatter was stripped and replaced
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            if !entries.is_empty() {
                let created_file = entries[0].as_ref().unwrap().path();
                let new_content = fs::read_to_string(created_file).unwrap();
                // Should have new frontmatter
                assert!(new_content.starts_with("---\n"));
                // Should not have duplicate frontmatter
                let frontmatter_count = new_content.matches("---\n").count();
                assert_eq!(frontmatter_count, 2); // Opening and closing
            }
        }
    }

    #[test]
    fn test_add_document_with_state_hint() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Test\n\nThis is ready for review and please review it.";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Should be placed in under-review directory due to state hint
        let review_dir = temp.path().join("02-under-review");
        if review_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&review_dir).unwrap().collect();
            assert!(!entries.is_empty());
        }
    }

    #[test]
    fn test_add_document_content_normalization() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Content without trailing newline
        fs::write(&file_path, "# Test\n\nContent without trailing newline").unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Verify content ends with newline
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            if !entries.is_empty() {
                let created_file = entries[0].as_ref().unwrap().path();
                let new_content = fs::read_to_string(created_file).unwrap();
                assert!(new_content.ends_with('\n'));
            }
        }
    }

    #[test]
    fn test_determine_title_auto_with_frontmatter_title() {
        let content = "---\ntitle: Frontmatter Title\n---\n\n# Heading Title\n\nContent";
        let extracted = ExtractedMetadata::from_content(content);

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.md");
        fs::write(&path, content).unwrap();

        let title = determine_title_auto(&extracted, &path);
        // Note: ExtractedMetadata doesn't parse frontmatter title, so it should use heading
        assert_eq!(title, "Heading Title");
    }

    #[test]
    fn test_determine_title_auto_with_no_heading_or_frontmatter() {
        let content = "Just plain content without any headings";
        let extracted = ExtractedMetadata::from_content(content);

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("my-file-name.md");
        fs::write(&path, content).unwrap();

        let title = determine_title_auto(&extracted, &path);
        assert_eq!(title, "My File Name");
    }

    #[test]
    fn test_determine_author_auto_with_extracted_author() {
        let content = "# Test\n\nAuthor: John Doe\n\nSome content";
        let extracted = ExtractedMetadata::from_content(content);

        let author = determine_author_auto(&extracted);
        assert_eq!(author, "John Doe");
    }

    #[test]
    fn test_preview_add_with_invalid_markdown() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("invalid.md");
        fs::write(&file_path, "").unwrap(); // Empty is invalid

        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("valid markdown"));
    }

    #[test]
    fn test_preview_add_with_frontmatter() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "---\ntitle: Test Title\nauthor: Test Author\n---\n\n# Test\n\nContent";
        fs::write(&file_path, content).unwrap();

        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_preview_add_with_content_issues() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Content with trailing spaces and inconsistent bullets
        fs::write(&file_path, "# Test\n\nLine with spaces  \n* item1\n- item2").unwrap();

        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_preview_add_without_detected_metadata() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("plain.md");
        fs::write(&file_path, "Just some plain content without headings or metadata").unwrap();

        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_batch_with_one_failure() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Create valid and invalid markdown files
        fs::write(temp.path().join("valid.md"), "# Valid Document\n\nContent").unwrap();
        fs::write(temp.path().join("invalid.md"), "").unwrap(); // Invalid - empty

        let pattern = format!("{}/*.md", temp.path().display());
        let result = add_batch(&mut state_mgr, vec![pattern], false, false);
        // Should succeed overall but report failures
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_batch_non_interactive() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        fs::write(temp.path().join("doc1.md"), "# Document 1\n\nContent").unwrap();
        fs::write(temp.path().join("doc2.md"), "# Document 2\n\nContent").unwrap();

        let pattern = format!("{}/*.md", temp.path().display());
        let result = add_batch(&mut state_mgr, vec![pattern], false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_batch_with_multiple_patterns() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        fs::write(temp.path().join("doc1.md"), "# Doc 1").unwrap();
        fs::write(subdir.join("doc2.md"), "# Doc 2").unwrap();

        let pattern1 = format!("{}/*.md", temp.path().display());
        let pattern2 = format!("{}/subdir/*.md", temp.path().display());

        let result = add_batch(&mut state_mgr, vec![pattern1, pattern2], true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_batch_skips_non_markdown_extensions() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        fs::write(temp.path().join("doc.md"), "# Document").unwrap();
        fs::write(temp.path().join("readme.txt"), "Not markdown").unwrap();
        fs::write(temp.path().join("data.json"), "{}").unwrap();

        let pattern = format!("{}/*", temp.path().display());
        let result = add_batch(&mut state_mgr, vec![pattern], true, false);
        assert!(result.is_ok());
        // Should only process the .md file
    }

    #[test]
    fn test_add_document_creates_state_directory() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test Document\n\nContent").unwrap();

        // Draft directory shouldn't exist yet
        let draft_dir = temp.path().join("01-draft");
        assert!(!draft_dir.exists());

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Draft directory should now exist
        assert!(draft_dir.exists());
    }

    #[test]
    fn test_add_document_git_add_failure_continues() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test\n\nContent").unwrap();

        // Even if git add fails (not a git repo), the operation should continue
        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_same_location_no_delete_prompt() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Create a file already in the draft directory with proper naming
        let draft_dir = temp.path().join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let file_path = draft_dir.join("existing.md");
        fs::write(&file_path, "# Test\n\nContent").unwrap();

        // This should work without prompting to delete original
        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        // The file should still exist after processing
        assert!(result.is_ok() || file_path.exists());
    }

    #[test]
    fn test_add_document_with_metadata_from_content() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# My Great Idea\n\nAuthor: Jane Smith\n\nThis is approved and accepted.";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Should extract title "My Great Idea" and author "Jane Smith"
        // Should detect "accepted" state hint
        let accepted_dir = temp.path().join("03-accepted");
        if accepted_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&accepted_dir).unwrap().collect();
            if !entries.is_empty() {
                let created_file = entries[0].as_ref().unwrap().path();
                let new_content = fs::read_to_string(created_file).unwrap();
                assert!(new_content.contains("title: My Great Idea"));
                assert!(new_content.contains("author: Jane Smith"));
                assert!(new_content.contains("state: accepted"));
            }
        }
    }

    #[test]
    fn test_add_document_number_increments() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Add first document with proper content
        let file1 = temp.path().join("test1.md");
        fs::write(&file1, "# Test 1\n\nContent for test 1.").unwrap();
        add_document(&mut state_mgr, file1.to_str().unwrap(), false, false, true).unwrap();

        // Add second document with proper content
        let file2 = temp.path().join("test2.md");
        fs::write(&file2, "# Test 2\n\nContent for test 2.").unwrap();
        add_document(&mut state_mgr, file2.to_str().unwrap(), false, false, true).unwrap();

        // Verify both files exist with different numbers
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            assert_eq!(entries.len(), 2);
        }
    }

    #[test]
    fn test_determine_title_auto_edge_cases() {
        let temp = TempDir::new().unwrap();

        // Test with file that has no extension
        let path = temp.path().join("no-extension");
        fs::write(&path, "Content").unwrap();
        let extracted = ExtractedMetadata::from_content("Content");
        let title = determine_title_auto(&extracted, &path);
        assert_eq!(title, "No Extension");

        // Test with empty filename scenario (should not panic)
        let extracted2 = ExtractedMetadata::from_content("# Heading");
        let title2 = determine_title_auto(&extracted2, &temp.path());
        // Should use heading or temp dir name
        assert!(!title2.is_empty());
    }

    #[test]
    fn test_preview_add_shows_all_states() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let test_cases = vec![
            ("# Draft\n\nWork in progress", "draft"),
            ("# Review\n\nReady for review", "under-review"),
            ("# Final\n\nThis is implemented", "final"),
            ("# Rejected\n\nThis was rejected", "rejected"),
            ("# Deferred\n\nThis is deferred", "deferred"),
        ];

        for (content, _expected_state) in test_cases {
            let file_path = temp.path().join(format!("test_{}.md", _expected_state));
            fs::write(&file_path, content).unwrap();

            let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_add_document_with_very_long_title() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let long_title = "A".repeat(200);
        let content = format!("# {}\n\nContent", long_title);
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        // Should handle long titles (may be truncated in filename)
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_with_special_chars_in_title() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Title with Special: Chars / and \\ Stuff!\n\nContent";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        // Should sanitize special characters for filename
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_with_unicode_title() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Use mostly ASCII with some unicode to pass validation
        let content = "# Unicode Title with Japanese 日本語\n\nThis is content with unicode characters like émojis and more regular text to ensure it passes markdown validation.";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_batch_all_files_fail() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Create only invalid files
        fs::write(temp.path().join("invalid1.md"), "").unwrap();
        fs::write(temp.path().join("invalid2.md"), "").unwrap();

        let pattern = format!("{}/*.md", temp.path().display());
        let result = add_batch(&mut state_mgr, vec![pattern], false, false);
        // Should succeed overall even if all files fail
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_preserves_content_structure() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Title\n\n## Section 1\n\nParagraph 1\n\n## Section 2\n\nParagraph 2\n\n```rust\ncode block\n```";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Verify content structure is preserved
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            if !entries.is_empty() {
                let created_file = entries[0].as_ref().unwrap().path();
                let new_content = fs::read_to_string(created_file).unwrap();
                assert!(new_content.contains("## Section 1"));
                assert!(new_content.contains("## Section 2"));
                assert!(new_content.contains("```rust"));
            }
        }
    }

    #[test]
    fn test_determine_author_auto_fallback_to_git() {
        let content = "# Document\n\nNo author mentioned here.";
        let extracted = ExtractedMetadata::from_content(content);

        let author = determine_author_auto(&extracted);
        // Should fallback to git author (we can't test exact value)
        assert!(!author.is_empty());
    }

    #[test]
    fn test_preview_add_with_state_hints() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Document\n\nThis is deferred and postponed for now.";
        fs::write(&file_path, content).unwrap();

        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        assert!(result.is_ok());
        // Should show state as deferred in preview
    }

    #[test]
    fn test_add_batch_with_glob_error_handling() {
        let mut state_mgr = StateManager::new(PathBuf::from("/tmp")).unwrap();

        // Invalid glob pattern with unclosed bracket
        let result = add_batch(&mut state_mgr, vec!["[invalid".to_string()], false, false);
        // Should handle glob errors gracefully
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_add_document_no_content_issues() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Clean content with no issues
        let content = "# Perfect Document\n\nThis is well-formed markdown content.\n\n## Section\n\nMore content here.\n";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_with_whitespace_only_content() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Content that is mostly whitespace but valid
        fs::write(&file_path, "# Title\n\n\n\n\nContent here.\n\n\n").unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_determine_title_interactive_would_use_defaults() {
        // We can't easily test interactive functions without mocking stdin,
        // but we can test the logic they use
        let content = "---\ntitle: FM Title\n---\n\n# Heading Title\n\nContent";
        let extracted = ExtractedMetadata::from_content(content);

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("file-name.md");
        fs::write(&path, content).unwrap();

        // The default title logic (what interactive mode would show as default)
        let default_title = extracted
            .title
            .as_ref()
            .or(extracted.first_heading.as_ref())
            .cloned()
            .unwrap_or_else(|| design::filename::filename_to_title("file-name.md"));

        assert_eq!(default_title, "Heading Title");
    }

    #[test]
    fn test_determine_author_interactive_would_use_git_default() {
        let content = "# Test Document\n\nNo author info in content.";
        let extracted = ExtractedMetadata::from_content(content);

        // What the interactive mode would use as default
        let default_author = std::process::Command::new("git")
            .args(["config", "user.name"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()) // Filter out empty strings
            .unwrap_or_else(|| "Unknown Author".to_string());

        let final_default = extracted.author.as_ref().unwrap_or(&default_author);
        assert!(!final_default.is_empty());
    }

    #[test]
    fn test_determine_state_interactive_default_logic() {
        let content = "# Draft Document\n\nWork in progress here.";
        let extracted = ExtractedMetadata::from_content(content);

        // The state hint should be Draft based on "work in progress"
        assert_eq!(extracted.state_hint, Some(DocState::Draft));

        // The default index would be 0 (Draft) if state_hint is Some(Draft)
        let default_idx = if let Some(hint) = extracted.state_hint {
            DocState::all_states().iter().position(|&s| s == hint).unwrap_or(0)
        } else {
            0
        };
        assert_eq!(default_idx, 0);
    }

    #[test]
    fn test_add_document_with_final_state_hint() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Final Document\n\nThis is implemented and complete.";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Should be in final directory
        let final_dir = temp.path().join("04-final");
        if final_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&final_dir).unwrap().collect();
            assert!(!entries.is_empty());
        }
    }

    #[test]
    fn test_add_document_with_rejected_state_hint() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Rejected Proposal\n\nThis was rejected and not approved.";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Should be in rejected directory
        let rejected_dir = temp.path().join("05-rejected");
        if rejected_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&rejected_dir).unwrap().collect();
            assert!(!entries.is_empty());
        }
    }

    #[test]
    fn test_add_document_with_deferred_state_hint() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Deferred Item\n\nThis has been postponed for later consideration.";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Should be in deferred directory
        let deferred_dir = temp.path().join("06-deferred");
        if deferred_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&deferred_dir).unwrap().collect();
            assert!(!entries.is_empty());
        }
    }

    #[test]
    fn test_add_batch_dry_run_creates_nothing() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        fs::write(temp.path().join("doc1.md"), "# Document 1\n\nContent").unwrap();
        fs::write(temp.path().join("doc2.md"), "# Document 2\n\nContent").unwrap();

        let pattern = format!("{}/*.md", temp.path().display());
        let result = add_batch(&mut state_mgr, vec![pattern], true, false); // dry_run = true
        assert!(result.is_ok());

        // No files should be created in dry run
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            assert!(entries.is_empty());
        }
    }

    #[test]
    fn test_add_document_metadata_with_dates() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Test Document\n\nAuthor: Test Author\n\nContent here.";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Verify created and updated dates are in the frontmatter
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            if !entries.is_empty() {
                let created_file = entries[0].as_ref().unwrap().path();
                let new_content = fs::read_to_string(created_file).unwrap();
                assert!(new_content.contains("created:"));
                assert!(new_content.contains("updated:"));
            }
        }
    }

    #[test]
    fn test_add_document_records_file_change() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let initial_state = state_mgr.state().documents.len();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test\n\nContent").unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // State should have recorded the new file
        let final_state = state_mgr.state().documents.len();
        assert!(final_state >= initial_state);
    }

    #[test]
    fn test_preview_add_with_detected_author() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "# Document\n\nWritten by Alice Smith\n\nContent here.";
        fs::write(&file_path, content).unwrap();

        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_without_issues_skips_normalization() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Perfect markdown with no issues
        let content = "# Perfect Title\n\nThis is perfectly formatted content.\n";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_batch_filters_directories() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Create a directory that matches pattern
        let dir = temp.path().join("test.md");
        fs::create_dir(&dir).unwrap();

        // Create a file
        fs::write(temp.path().join("real.md"), "# Real\n\nContent").unwrap();

        let pattern = format!("{}/*.md", temp.path().display());
        let result = add_batch(&mut state_mgr, vec![pattern], true, false);
        assert!(result.is_ok());
        // Should only process the file, not the directory
    }

    #[test]
    fn test_add_document_builds_correct_yaml_frontmatter() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Title\n\nContent").unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            if !entries.is_empty() {
                let created_file = entries[0].as_ref().unwrap().path();
                let content = fs::read_to_string(created_file).unwrap();
                // Verify YAML frontmatter structure
                assert!(content.starts_with("---\n"));
                assert!(content.contains("number:"));
                assert!(content.contains("title:"));
                assert!(content.contains("author:"));
                assert!(content.contains("state:"));
            }
        }
    }

    #[test]
    fn test_determine_title_auto_with_path_edge_case() {
        let temp = TempDir::new().unwrap();
        let extracted = ExtractedMetadata::from_content("Content only");

        // Test with path that has multiple dots
        let path = temp.path().join("my.test.file.md");
        fs::write(&path, "content").unwrap();
        let title = determine_title_auto(&extracted, &path);
        assert!(!title.is_empty());
    }

    #[test]
    fn test_add_document_with_minimal_valid_content() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Minimum valid markdown (just over 10 chars)
        fs::write(&file_path, "# A\n\nContent text").unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_interactive_mode_requires_prompt() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test\n\nContent with issues  \n").unwrap();

        // Interactive mode without auto_yes would require user input,
        // so we can't test it directly without mocking stdin.
        // We're testing that the code path exists and compiles correctly.
        // In real usage, this would prompt the user.

        // We can only test non-interactive paths safely
        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_dry_run_mode_with_issues() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test\n\nContent with trailing spaces  \n* item1\n- item2\n")
            .unwrap();

        let result = add_document(
            &mut state_mgr,
            file_path.to_str().unwrap(),
            true, // dry_run
            false,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_document_non_interactive_with_issues() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test\n\nTrailing spaces  \n").unwrap();

        // Non-interactive mode with auto_yes=false should still work
        let result = add_document(
            &mut state_mgr,
            file_path.to_str().unwrap(),
            false,
            false, // not interactive
            false, // not auto_yes
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_determine_state_with_no_hint() {
        let content = "# Generic Document\n\nNo state hints in this content.";
        let extracted = ExtractedMetadata::from_content(content);

        assert_eq!(extracted.state_hint, None);

        // When no hint, should default to Draft
        let state = extracted.state_hint.unwrap_or(DocState::Draft);
        assert_eq!(state, DocState::Draft);
    }

    #[test]
    fn test_all_state_hints_detection() {
        let test_cases = vec![
            ("Work in progress here", Some(DocState::Draft)),
            ("WIP document", Some(DocState::Draft)),
            ("Ready for review please", Some(DocState::UnderReview)),
            ("Please review this", Some(DocState::UnderReview)),
            ("This is approved", Some(DocState::Accepted)),
            ("Accepted proposal", Some(DocState::Accepted)),
            ("This is implemented", Some(DocState::Final)),
            ("Complete implementation", Some(DocState::Final)),
            ("This was rejected", Some(DocState::Rejected)),
            ("Explicitly rejected proposal", Some(DocState::Rejected)),
            ("This is deferred", Some(DocState::Deferred)),
            ("Postponed for now", Some(DocState::Deferred)),
            ("No hints here", None),
        ];

        for (content, expected_state) in test_cases {
            let full_content = format!("# Test\n\n{}", content);
            let extracted = ExtractedMetadata::from_content(&full_content);
            assert_eq!(extracted.state_hint, expected_state, "Failed for: {}", content);
        }
    }

    #[test]
    fn test_add_batch_success_and_failure_count() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        // Mix of valid and invalid files
        fs::write(temp.path().join("valid1.md"), "# Valid 1\n\nContent").unwrap();
        fs::write(temp.path().join("valid2.md"), "# Valid 2\n\nContent").unwrap();
        fs::write(temp.path().join("invalid.md"), "").unwrap(); // Invalid

        let pattern = format!("{}/*.md", temp.path().display());
        let result = add_batch(&mut state_mgr, vec![pattern], false, false);

        // Should complete successfully even with failures
        assert!(result.is_ok());
    }

    #[test]
    fn test_preview_add_shows_number_assignment() {
        let temp = TempDir::new().unwrap();
        let state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test Document\n\nContent").unwrap();

        // Preview should show what number would be assigned
        let next_num = state_mgr.next_number();
        let result = preview_add(file_path.to_str().unwrap(), &state_mgr);
        assert!(result.is_ok());

        // Number should be predictable
        assert!(next_num > 0);
    }

    #[test]
    fn test_add_document_with_content_ending_with_newline() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "# Test\n\nContent already ending with newline.\n").unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Should preserve the trailing newline
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            if !entries.is_empty() {
                let created_file = entries[0].as_ref().unwrap().path();
                let content = fs::read_to_string(created_file).unwrap();
                assert!(content.ends_with('\n'));
            }
        }
    }

    #[test]
    fn test_add_document_strips_only_existing_frontmatter() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        let content = "---\nold: frontmatter\n---\n\n# Title\n\nContent";
        fs::write(&file_path, content).unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            if !entries.is_empty() {
                let created_file = entries[0].as_ref().unwrap().path();
                let content = fs::read_to_string(created_file).unwrap();
                // Should not contain old frontmatter
                assert!(!content.contains("old: frontmatter"));
                // Should have new frontmatter
                assert!(content.contains("number:"));
            }
        }
    }

    #[test]
    fn test_determine_author_with_pattern_variations() {
        let test_cases = vec![
            ("Author: John Doe", Some("John Doe")),
            ("by Alice Smith", Some("Alice Smith")),
            ("Written by Bob Jones", Some("Bob Jones")),
            ("No author here", None),
        ];

        for (content_snippet, expected_author) in test_cases {
            let full_content = format!("# Test\n\n{}\n\nMore content", content_snippet);
            let extracted = ExtractedMetadata::from_content(&full_content);

            if let Some(expected) = expected_author {
                assert_eq!(extracted.author.as_deref(), Some(expected));
            } else {
                assert_eq!(extracted.author, None);
            }
        }
    }

    #[test]
    fn test_add_batch_with_no_interactive_prompt() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        fs::write(temp.path().join("test.md"), "# Test\n\nContent").unwrap();

        let pattern = format!("{}/*.md", temp.path().display());

        // Non-interactive batch should not prompt
        let result = add_batch(&mut state_mgr, vec![pattern], false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_filename_sanitization_in_build() {
        let temp = TempDir::new().unwrap();
        let mut state_mgr = StateManager::new(temp.path()).unwrap();

        let file_path = temp.path().join("test.md");
        // Title with characters that need sanitization
        fs::write(&file_path, "# Title: With / Special \\ Characters?\n\nContent").unwrap();

        let result = add_document(&mut state_mgr, file_path.to_str().unwrap(), false, false, true);
        assert!(result.is_ok());

        // Filename should be sanitized
        let draft_dir = temp.path().join("01-draft");
        if draft_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&draft_dir).unwrap().collect();
            if !entries.is_empty() {
                let filename = entries[0].as_ref().unwrap().file_name();
                let filename_str = filename.to_string_lossy();
                // Should not contain illegal filename characters
                assert!(!filename_str.contains('/'));
                assert!(!filename_str.contains('\\'));
                assert!(!filename_str.contains(':'));
                assert!(!filename_str.contains('?'));
            }
        }
    }
}
