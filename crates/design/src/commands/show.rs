//! Show command implementation

use anyhow::{bail, Result};
use design::index::DocumentIndex;
use oxur_table::TableStyleConfig;
use std::env;
use tabled::{builder::Builder, Tabled};

/// Minimal struct for type parameter when building tables with Builder
/// (Not used for actual data - Builder uses plain strings)
#[derive(Tabled)]
struct DocumentInfoRow {
    field: String,
    content: String,
}

/// Get the relative path from the current working directory to the document
fn get_relative_path(doc_path: &std::path::Path) -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    match doc_path.strip_prefix(&current_dir) {
        Ok(rel_path) => rel_path.to_string_lossy().to_string(),
        Err(_) => doc_path.to_string_lossy().to_string(),
    }
}

pub fn show_document(index: &DocumentIndex, number: u32, _metadata_only: bool) -> Result<()> {
    let doc = match index.get(number) {
        Some(d) => d,
        None => bail!("Document {:04} not found", number),
    };

    // Build table with Builder (plain text only)
    let mut builder = Builder::default();

    // Row 0: Title - PLAIN TEXT (formatting applied later)
    builder.push_record(["DOCUMENT", "INFORMATION", ""]);

    // Row 1: Header - PLAIN TEXT
    builder.push_record(["Field", "Content", ""]);

    // Data rows - PLAIN TEXT with space prefix (no ANSI codes, formatting applied later)

    // Number (0-padded to 4 digits)
    builder.push_record([" Number", &format!(" {:04}", doc.metadata.number), ""]);

    // Title
    builder.push_record([" Title", &format!(" {} ", doc.metadata.title), ""]);

    // Author
    builder.push_record([" Author", &format!(" {}", doc.metadata.author), ""]);

    // State
    builder.push_record([" State", &format!(" {}", doc.metadata.state.as_str()), ""]);

    // Created
    builder.push_record([" Created ", &format!(" {}", doc.metadata.created), ""]);

    // Updated
    builder.push_record([" Updated ", &format!(" {}", doc.metadata.updated), ""]);

    // Path (relative to current working directory)
    let rel_path = get_relative_path(&doc.path);
    builder.push_record([" Path", &format!(" {} ", rel_path), ""]);

    // Supersedes (only if not null)
    if let Some(supersedes) = doc.metadata.supersedes {
        builder.push_record([" Supersedes", &format!(" {:04}", supersedes), ""]);
    }

    // Superseded By (only if not null)
    if let Some(superseded_by) = doc.metadata.superseded_by {
        builder.push_record([" Superseded By", &format!(" {:04}", superseded_by), ""]);
    }

    // Empty footer row
    builder.push_record(["", "", ""]);

    // Build the table structure (width calculation happens here with plain text)
    let mut table = builder.build();

    // Apply the theme (background colors, padding, justification)
    let config = TableStyleConfig::default();
    config.apply_to_table::<DocumentInfoRow>(&mut table);

    // Print with blank lines for spacing
    println!();
    println!("{}", table);
    println!();

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

    fn create_test_index_with_docs() -> DocumentIndex {
        let temp = TempDir::new().unwrap();
        let mut state = DocumentState::new();

        // Doc 1: Basic document
        let meta1 = DocMetadata {
            number: 1,
            title: "First Document".to_string(),
            author: "Alice".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        state.upsert(
            1,
            DocumentRecord {
                metadata: meta1,
                path: "0001-first.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: chrono::Utc::now(),
            },
        );

        // Doc 2: With supersedes
        let meta2 = DocMetadata {
            number: 2,
            title: "Second Document".to_string(),
            author: "Bob".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            state: DocState::Active,
            supersedes: Some(1),
            superseded_by: None,
        };
        state.upsert(
            2,
            DocumentRecord {
                metadata: meta2,
                path: "0002-second.md".to_string(),
                checksum: "def456".to_string(),
                file_size: 150,
                modified: chrono::Utc::now(),
            },
        );

        // Doc 3: With superseded_by
        let meta3 = DocMetadata {
            number: 3,
            title: "Third Document".to_string(),
            author: "Charlie".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            state: DocState::Superseded,
            supersedes: None,
            superseded_by: Some(4),
        };
        state.upsert(
            3,
            DocumentRecord {
                metadata: meta3,
                path: "0003-third.md".to_string(),
                checksum: "ghi789".to_string(),
                file_size: 200,
                modified: chrono::Utc::now(),
            },
        );

        DocumentIndex::from_state(&state, temp.path()).unwrap()
    }

    #[test]
    fn test_show_existing_document() {
        let index = create_test_index_with_docs();

        let result = show_document(&index, 1, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_nonexistent_document() {
        let index = create_test_index_with_docs();

        let result = show_document(&index, 9999, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_show_metadata_only() {
        let index = create_test_index_with_docs();

        // The metadata_only parameter is now unused but the function should still work
        let result = show_document(&index, 1, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_document_with_supersedes() {
        let index = create_test_index_with_docs();

        // Doc 2 supersedes Doc 1
        let result = show_document(&index, 2, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_document_with_superseded_by() {
        let index = create_test_index_with_docs();

        // Doc 3 is superseded by Doc 4
        let result = show_document(&index, 3, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_all_documents() {
        let index = create_test_index_with_docs();

        // Test showing each document
        for num in [1, 2, 3] {
            let result = show_document(&index, num, false);
            assert!(result.is_ok(), "Failed to show document {}", num);
        }
    }

    #[test]
    fn test_show_empty_index() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        let result = show_document(&index, 1, false);
        assert!(result.is_err());
    }
}
