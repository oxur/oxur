//! Add development documents with automatic numbering

use anyhow::{Context, Result};
use design::config::Config;
use design::extract::{is_valid_markdown, ExtractedMetadata};
use design::filename::build_filename;
use design::git;
use oxur_cli::common::output::{info, success, warning};
use std::fs;
use std::path::{Path, PathBuf};

/// Add a development document with automatic numbering
pub fn add_dev_document(
    config: &Config,
    doc_path: &str,
    subdir: Option<&str>,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        info("DRY RUN MODE - No changes will be made\n");
    }

    info(&format!("Adding development document: {}\n", doc_path));

    // Step 1: Validate source file exists
    let source_path = PathBuf::from(doc_path);
    if !source_path.exists() {
        anyhow::bail!("File not found: {}", doc_path);
    }

    // Step 2: Read and validate markdown content
    let content = fs::read_to_string(&source_path).context("Failed to read file")?;

    if !is_valid_markdown(&content) {
        anyhow::bail!("File doesn't appear to be valid markdown");
    }

    // Step 3: Extract title
    let extracted = ExtractedMetadata::from_content(&content);
    let title = determine_title_auto(&extracted, &source_path);

    info(&format!("Title: {}\n", title));

    // Step 4: Build target directory path
    let target_dir = build_target_directory(config, subdir)?;

    // Step 5: Find next number in target directory
    let number = find_next_dev_number(&target_dir)?;

    // Step 6: Build filename
    let filename = build_filename(number, &title);

    info(&format!("Number: {:04}", number));
    info(&format!("Filename: {}\n", filename));

    // Step 7: Build final target path
    let target_path = target_dir.join(&filename);

    // Step 8: Check for existing file
    if target_path.exists() && !force {
        anyhow::bail!(
            "File already exists: {}\nUse --force to overwrite",
            target_path.display()
        );
    }

    if target_path.exists() && force {
        warning(&format!("Overwriting existing file: {}", target_path.display()));
    }

    if dry_run {
        info(&format!("Would create: {}\n", target_path.display()));
        return Ok(());
    }

    // Step 9: Create directory
    fs::create_dir_all(&target_dir).context("Failed to create target directory")?;

    // Step 10: Copy file to target
    fs::copy(&source_path, &target_path).context("Failed to copy file")?;

    success(&format!("Created: {}", target_path.display()));

    // Step 11: Git add if auto_stage_git is enabled
    if config.auto_stage_git {
        if let Err(e) = git::git_add(&target_path) {
            warning(&format!("Git staging failed: {}", e));
        } else {
            info("Staged with git");
        }
    }

    success(&format!("\nSuccessfully added: {}", filename));

    Ok(())
}

/// Determine title automatically from extracted metadata or filename
fn determine_title_auto(extracted: &ExtractedMetadata, path: &Path) -> String {
    extracted.title.clone().or_else(|| extracted.first_heading.clone()).unwrap_or_else(|| {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        design::filename::filename_to_title(filename)
    })
}

/// Build target directory path from config and optional subdirectory
fn build_target_directory(config: &Config, subdir: Option<&str>) -> Result<PathBuf> {
    let mut path = config.dev_directory.clone();

    if let Some(sub) = subdir {
        // Sanitize subdirectory name (prevent path traversal)
        let sanitized = sub.replace(['/', '\\', '.'], "-");
        if sanitized.is_empty() {
            anyhow::bail!("Invalid subdirectory name");
        }
        path.push(sanitized);
    }

    Ok(path)
}

/// Find the next available number in a directory
fn find_next_dev_number(dir: &Path) -> Result<u32> {
    // If directory doesn't exist, start at 1
    if !dir.exists() {
        return Ok(1);
    }

    let mut max_number = 0;

    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if let Some(num) = extract_number_prefix(&filename_str) {
            if num > max_number {
                max_number = num;
            }
        }
    }

    Ok(max_number + 1)
}

/// Extract number prefix from filename (e.g., "0042-foo.md" -> Some(42))
fn extract_number_prefix(filename: &str) -> Option<u32> {
    // Look for pattern: nnnn-*.md where nnnn is 4 digits
    if filename.len() < 6 {
        return None;
    }

    // Check if it starts with 4 digits followed by a hyphen
    let prefix = &filename[..4];
    if !prefix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    if filename.chars().nth(4) != Some('-') {
        return None;
    }

    prefix.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_number_prefix_valid() {
        assert_eq!(extract_number_prefix("0001-test.md"), Some(1));
        assert_eq!(extract_number_prefix("0042-foo.md"), Some(42));
        assert_eq!(extract_number_prefix("9999-bar.md"), Some(9999));
    }

    #[test]
    fn test_extract_number_prefix_invalid() {
        assert_eq!(extract_number_prefix("test.md"), None);
        assert_eq!(extract_number_prefix("1-test.md"), None);
        assert_eq!(extract_number_prefix("001-test.md"), None);
        assert_eq!(extract_number_prefix("0001test.md"), None);
        assert_eq!(extract_number_prefix("abc-test.md"), None);
    }

    #[test]
    fn test_find_next_dev_number_empty_dir() {
        let temp = TempDir::new().unwrap();
        let result = find_next_dev_number(temp.path()).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_find_next_dev_number_nonexistent_dir() {
        let temp = TempDir::new().unwrap();
        let nonexistent = temp.path().join("nonexistent");
        let result = find_next_dev_number(&nonexistent).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_find_next_dev_number_with_files() {
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("0001-first.md"), "# First").unwrap();
        fs::write(temp.path().join("0002-second.md"), "# Second").unwrap();
        fs::write(temp.path().join("0005-fifth.md"), "# Fifth").unwrap();

        let result = find_next_dev_number(temp.path()).unwrap();
        assert_eq!(result, 6);
    }

    #[test]
    fn test_find_next_dev_number_ignores_non_numbered() {
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("0001-first.md"), "# First").unwrap();
        fs::write(temp.path().join("README.md"), "# README").unwrap();
        fs::write(temp.path().join("test.txt"), "test").unwrap();

        let result = find_next_dev_number(temp.path()).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_build_target_directory_without_subdir() {
        let config = Config {
            dev_directory: PathBuf::from("/dev"),
            ..Default::default()
        };

        let result = build_target_directory(&config, None).unwrap();
        assert_eq!(result, PathBuf::from("/dev"));
    }

    #[test]
    fn test_build_target_directory_with_subdir() {
        let config = Config {
            dev_directory: PathBuf::from("/dev"),
            ..Default::default()
        };

        let result = build_target_directory(&config, Some("planning")).unwrap();
        assert_eq!(result, PathBuf::from("/dev/planning"));
    }

    #[test]
    fn test_build_target_directory_sanitizes_subdir() {
        let config = Config {
            dev_directory: PathBuf::from("/dev"),
            ..Default::default()
        };

        let result = build_target_directory(&config, Some("../evil")).unwrap();
        assert_eq!(result, PathBuf::from("/dev/---evil"));
    }

    #[test]
    fn test_determine_title_auto_from_heading() {
        let content = "# Test Title\n\nContent";
        let extracted = ExtractedMetadata::from_content(content);

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("file.md");
        fs::write(&path, content).unwrap();

        let title = determine_title_auto(&extracted, &path);
        assert_eq!(title, "Test Title");
    }

    #[test]
    fn test_determine_title_auto_from_filename() {
        let content = "Just content";
        let extracted = ExtractedMetadata::from_content(content);

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("my-test-file.md");
        fs::write(&path, content).unwrap();

        let title = determine_title_auto(&extracted, &path);
        assert_eq!(title, "My Test File");
    }

    #[test]
    fn test_add_dev_document_basic() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        // Create source file
        let source = temp.path().join("source.md");
        fs::write(&source, "# Test Document\n\nContent here.").unwrap();

        let result = add_dev_document(&config, source.to_str().unwrap(), None, false, false);
        assert!(result.is_ok());

        // Verify target file was created
        let expected_file = temp.path().join("0001-test-document.md");
        assert!(expected_file.exists());

        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("# Test Document"));
    }

    #[test]
    fn test_add_dev_document_with_subdir() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        let source = temp.path().join("source.md");
        fs::write(&source, "# Test\n\nContent").unwrap();

        let result =
            add_dev_document(&config, source.to_str().unwrap(), Some("plans"), false, false);
        assert!(result.is_ok());

        let expected_file = temp.path().join("plans/0001-test.md");
        assert!(expected_file.exists());
    }

    #[test]
    fn test_add_dev_document_independent_numbering() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        // Add to root
        let source1 = temp.path().join("source1.md");
        fs::write(&source1, "# Root Doc\n\nContent").unwrap();
        add_dev_document(&config, source1.to_str().unwrap(), None, false, false).unwrap();

        // Add to subdir
        let source2 = temp.path().join("source2.md");
        fs::write(&source2, "# Subdir Doc\n\nContent").unwrap();
        add_dev_document(&config, source2.to_str().unwrap(), Some("sub"), false, false).unwrap();

        // Both should be numbered 0001
        assert!(temp.path().join("0001-root-doc.md").exists());
        assert!(temp.path().join("sub/0001-subdir-doc.md").exists());
    }

    #[test]
    fn test_add_dev_document_sequential_numbering() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        // Add first doc
        let source1 = temp.path().join("source1.md");
        fs::write(&source1, "# First\n\nContent").unwrap();
        add_dev_document(&config, source1.to_str().unwrap(), None, false, false).unwrap();

        // Add second doc
        let source2 = temp.path().join("source2.md");
        fs::write(&source2, "# Second\n\nContent").unwrap();
        add_dev_document(&config, source2.to_str().unwrap(), None, false, false).unwrap();

        assert!(temp.path().join("0001-first.md").exists());
        assert!(temp.path().join("0002-second.md").exists());
    }

    #[test]
    fn test_add_dev_document_dry_run() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        let source = temp.path().join("source.md");
        fs::write(&source, "# Test\n\nContent").unwrap();

        let result = add_dev_document(&config, source.to_str().unwrap(), None, false, true);
        assert!(result.is_ok());

        // No file should be created in dry run
        let expected_file = temp.path().join("0001-test.md");
        assert!(!expected_file.exists());
    }

    #[test]
    fn test_add_dev_document_duplicate_titles_get_different_numbers() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        // Add first file with title "Test"
        let source1 = temp.path().join("source1.md");
        fs::write(&source1, "# Test\n\nFirst content").unwrap();
        add_dev_document(&config, source1.to_str().unwrap(), None, false, false).unwrap();

        // Add second file with same title "Test" - should get different number
        let source2 = temp.path().join("source2.md");
        fs::write(&source2, "# Test\n\nSecond content").unwrap();
        let result = add_dev_document(&config, source2.to_str().unwrap(), None, false, false);
        assert!(result.is_ok());

        // Both files should exist with different numbers
        assert!(config.dev_directory.join("0001-test.md").exists());
        assert!(config.dev_directory.join("0002-test.md").exists());
    }

    // Note: Testing the --force flag is tricky because find_next_dev_number
    // always returns max+1, so files created via add_dev_document will never conflict.
    // The --force flag is mainly for rare race conditions or manual file creation.
    // In practice, duplicate titles simply get sequential numbers (0001, 0002, etc.)

    #[test]
    fn test_add_dev_document_invalid_markdown() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        let source = temp.path().join("source.md");
        fs::write(&source, "").unwrap(); // Empty is invalid

        let result = add_dev_document(&config, source.to_str().unwrap(), None, false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("valid markdown"));
    }

    #[test]
    fn test_add_dev_document_missing_file() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        let result = add_dev_document(&config, "/nonexistent/file.md", None, false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_add_dev_document_creates_directory() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().join("dev"),
            auto_stage_git: false,
            ..Default::default()
        };

        // Directory shouldn't exist yet
        assert!(!config.dev_directory.exists());

        let source = temp.path().join("source.md");
        fs::write(&source, "# Test\n\nContent").unwrap();

        let result = add_dev_document(&config, source.to_str().unwrap(), None, false, false);
        assert!(result.is_ok());

        // Directory should now exist
        assert!(config.dev_directory.exists());
    }

    #[test]
    fn test_add_dev_document_with_title_fallback() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            dev_directory: temp.path().to_path_buf(),
            auto_stage_git: false,
            ..Default::default()
        };

        // Content without heading
        let source = temp.path().join("my-awesome-doc.md");
        fs::write(&source, "Just some content without a heading.").unwrap();

        let result = add_dev_document(&config, source.to_str().unwrap(), None, false, false);
        assert!(result.is_ok());

        // Should use filename as title
        let expected_file = temp.path().join("0001-my-awesome-doc.md");
        assert!(expected_file.exists());
    }
}
