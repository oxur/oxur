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
}
