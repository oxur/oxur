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

#[cfg(test)]
mod tests {
    use super::*;
    use design::doc::{DocMetadata, DocState};
    use design::index::DocumentIndex;
    use design::state::{DocumentRecord, DocumentState};
    use chrono::NaiveDate;
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
