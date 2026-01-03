//! New document command implementation

use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use design::doc::DocState;
use design::git;
use design::index::DocumentIndex;
use std::fs;
use std::path::PathBuf;

pub fn new_document(
    index: &DocumentIndex,
    title: String,
    author: Option<String>,
    component: Option<String>,
    tags: Vec<String>,
) -> Result<()> {
    let number = index.next_number();
    let author = author.unwrap_or_else(|| git::get_author("."));

    let today = Local::now().naive_local().date();

    // Apply defaults: component = "All", tags = ["change-me"]
    let component = component.unwrap_or_else(|| "All".to_string());
    let tags = if tags.is_empty() { vec!["change-me".to_string()] } else { tags };

    // Format component YAML (always present now)
    let component_yaml = format!("component: {}\n", component);

    // Format tags YAML (always present now)
    let tags_yaml = format!("tags: [{}]\n", tags.join(", "));

    let template = format!(
        r#"---
number: {}
title: "{}"
author: "{}"
{}{}created: {}
updated: {}
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# {}

## Overview

*Brief description of what this document covers*

## Background

*Context and motivation for this design*

## Proposal

*Detailed description of the proposed design*

## Alternatives Considered

*What other approaches were considered and why were they rejected?*

## Implementation Plan

*Steps needed to implement this design*

## Open Questions

*Unresolved questions that need discussion*

## Success Criteria

*How will we know this design is successful?*
"#,
        number, title, author, component_yaml, tags_yaml, today, today, title
    );

    let filename = format!(
        "{:04}-{}.md",
        number,
        title
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
    );

    // Use the new directory naming scheme
    let docs_dir = PathBuf::from(index.docs_dir()).join(DocState::Draft.directory());
    fs::create_dir_all(&docs_dir)?;

    let path = docs_dir.join(&filename);
    fs::write(&path, template)?;

    println!("Created new design document:");
    println!("  Number: {:04}", number);
    println!("  Title: {}", title);
    println!("  File: {}", path.display());

    // Update the index to reflect the new document
    println!();
    if let Err(e) = crate::commands::update_index::update_index(index) {
        println!("{} Failed to update index", "Warning:".yellow());
        println!("  {}", e);
        println!("  Run 'oxd update-index' manually to sync the index");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use design::doc::{DocMetadata, DocState};
    use design::index::DocumentIndex;
    use design::state::{DocumentRecord, DocumentState};
    use tempfile::TempDir;

    fn create_test_index() -> (DocumentIndex, TempDir) {
        let temp = TempDir::new().unwrap();
        let mut state = DocumentState::new();

        // Add one existing document
        let meta = DocMetadata {
            number: 1,
            title: "Existing Doc".to_string(),
            author: "Test Author".to_string(),
            component: None,
            tags: Vec::new(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
            version: "1.0".to_string(),
        };
        state.upsert(
            1,
            DocumentRecord {
                metadata: meta,
                path: "0001-existing.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        let index = DocumentIndex::from_state(&state, temp.path()).unwrap();
        (index, temp)
    }

    #[test]
    fn test_new_document_with_provided_author() {
        let (index, _temp) = create_test_index();

        let result = new_document(
            &index,
            "Test Document".to_string(),
            Some("Alice".to_string()),
            None,
            Vec::new(),
        );
        assert!(result.is_ok());

        // Verify file was created in Draft directory
        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        assert!(draft_dir.exists());

        let expected_file = draft_dir.join("0002-test-document.md");
        assert!(expected_file.exists());

        // Verify content (including defaults)
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("number: 2"));
        assert!(content.contains("title: \"Test Document\""));
        assert!(content.contains("author: \"Alice\""));
        assert!(content.contains("component: All"));
        assert!(content.contains("tags: [change-me]"));
        assert!(content.contains("state: Draft"));
        assert!(content.contains("# Test Document"));
    }

    #[test]
    fn test_new_document_with_default_author() {
        let (index, _temp) = create_test_index();

        let result = new_document(&index, "Another Doc".to_string(), None, None, Vec::new());
        assert!(result.is_ok());

        // Verify file was created
        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        let expected_file = draft_dir.join("0002-another-doc.md");
        assert!(expected_file.exists());
    }

    #[test]
    fn test_filename_sanitization() {
        let (index, _temp) = create_test_index();

        // Title with spaces and special characters
        let result = new_document(
            &index,
            "Test: Document & More!".to_string(),
            Some("Test".to_string()),
            None,
            Vec::new(),
        );
        assert!(result.is_ok());

        // Verify sanitized filename (special chars removed, spaces become dashes)
        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        // ": " and "& !" are removed leaving "Test Document  More" which becomes "test-document--more"
        let expected_file = draft_dir.join("0002-test-document--more.md");
        assert!(expected_file.exists());
    }

    #[test]
    fn test_next_number_calculation() {
        let (index, _temp) = create_test_index();

        // Should get number 2 since we have 1 existing doc
        let result = new_document(
            &index,
            "Doc Title".to_string(),
            Some("Author".to_string()),
            None,
            Vec::new(),
        );
        assert!(result.is_ok());

        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        let expected_file = draft_dir.join("0002-doc-title.md");
        assert!(expected_file.exists());

        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("number: 2"));
    }

    #[test]
    fn test_creates_draft_directory() {
        let (index, _temp) = create_test_index();

        // Draft directory shouldn't exist yet
        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        assert!(!draft_dir.exists());

        let result = new_document(
            &index,
            "New Doc".to_string(),
            Some("Author".to_string()),
            None,
            Vec::new(),
        );
        assert!(result.is_ok());

        // Now it should exist
        assert!(draft_dir.exists());
        assert!(draft_dir.is_dir());
    }

    #[test]
    fn test_template_structure() {
        let (index, _temp) = create_test_index();

        let result = new_document(
            &index,
            "Template Test".to_string(),
            Some("Alice".to_string()),
            None,
            Vec::new(),
        );
        assert!(result.is_ok());

        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        let file = draft_dir.join("0002-template-test.md");
        let content = fs::read_to_string(&file).unwrap();

        // Verify all expected sections
        assert!(content.contains("## Overview"));
        assert!(content.contains("## Background"));
        assert!(content.contains("## Proposal"));
        assert!(content.contains("## Alternatives Considered"));
        assert!(content.contains("## Implementation Plan"));
        assert!(content.contains("## Open Questions"));
        assert!(content.contains("## Success Criteria"));
    }

    #[test]
    fn test_empty_index() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        // First document should be number 1
        let result = new_document(
            &index,
            "First Doc".to_string(),
            Some("Author".to_string()),
            None,
            Vec::new(),
        );
        assert!(result.is_ok());

        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        let expected_file = draft_dir.join("0001-first-doc.md");
        assert!(expected_file.exists());

        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("number: 1"));
    }

    #[test]
    fn test_title_with_unicode() {
        let (index, _temp) = create_test_index();

        // Title with unicode characters (is_alphanumeric includes unicode letters)
        let result = new_document(
            &index,
            "Test café".to_string(),
            Some("Author".to_string()),
            None,
            Vec::new(),
        );
        assert!(result.is_ok());

        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        // Unicode 'é' is preserved because is_alphanumeric() returns true for it
        let expected_file = draft_dir.join("0002-test-café.md");
        assert!(expected_file.exists());
    }

    #[test]
    fn test_new_document_with_custom_component_and_tags() {
        let (index, _temp) = create_test_index();

        let result = new_document(
            &index,
            "Custom Doc".to_string(),
            Some("Author".to_string()),
            Some("AST".to_string()),
            vec!["Phase-1".to_string(), "Testing".to_string()],
        );
        assert!(result.is_ok());

        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        let expected_file = draft_dir.join("0002-custom-doc.md");
        assert!(expected_file.exists());

        // Verify custom values are used
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("component: AST"));
        assert!(content.contains("tags: [Phase-1, Testing]"));
    }

    #[test]
    fn test_new_document_applies_defaults() {
        let (index, _temp) = create_test_index();

        // Don't provide component or tags
        let result = new_document(
            &index,
            "Default Values".to_string(),
            Some("Author".to_string()),
            None,
            Vec::new(),
        );
        assert!(result.is_ok());

        let draft_dir = PathBuf::from(index.docs_dir()).join("01-draft");
        let expected_file = draft_dir.join("0002-default-values.md");
        assert!(expected_file.exists());

        // Verify defaults are applied
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("component: All"));
        assert!(content.contains("tags: [change-me]"));
    }
}
