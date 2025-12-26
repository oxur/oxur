//! Index generation command implementation

use anyhow::Result;
use design::doc::DocState;
use design::index::DocumentIndex;
use std::fs;
use std::path::PathBuf;

/// Generate the index markdown or JSON
pub fn generate_index(index: &DocumentIndex, format: &str) -> Result<()> {
    match format {
        "markdown" | "md" => generate_markdown_index(index),
        "json" => generate_json_index(index),
        _ => {
            eprintln!("Unknown format: {}. Using markdown.", format);
            generate_markdown_index(index)
        }
    }
}

fn generate_markdown_index(index: &DocumentIndex) -> Result<()> {
    let mut content = String::new();

    // Header
    content.push_str("# Design Document Index\n\n");
    content.push_str("This index is automatically generated. Do not edit manually.\n\n");

    // Table section
    content.push_str("## All Documents by Number\n\n");
    content.push_str("| Number | Title | State | Updated |\n");
    content.push_str("|--------|-------|-------|----------|\n");

    let docs = index.all();

    for doc in &docs {
        content.push_str(&format!(
            "| {:04} | {} | {} | {} |\n",
            doc.metadata.number,
            doc.metadata.title,
            doc.metadata.state.as_str(),
            doc.metadata.updated
        ));
    }

    content.push('\n');

    // State sections
    content.push_str("## Documents by State\n");

    for state in DocState::all_states() {
        let state_docs = index.by_state(state);

        if !state_docs.is_empty() {
            content.push_str(&format!("\n### {}\n\n", state.as_str()));

            for doc in state_docs {
                let rel_path = doc.path.strip_prefix(index.docs_dir()).unwrap_or(&doc.path);
                let path_str = rel_path.to_string_lossy();

                content.push_str(&format!(
                    "- [{:04} - {}]({})\n",
                    doc.metadata.number, doc.metadata.title, path_str
                ));
            }
        }
    }

    // Write to file
    let index_path = PathBuf::from(index.docs_dir()).join("00-index.md");
    fs::write(&index_path, content)?;

    println!("Generated index at: {}", index_path.display());
    Ok(())
}

fn generate_json_index(index: &DocumentIndex) -> Result<()> {
    #[derive(serde::Serialize)]
    struct JsonDoc {
        number: u32,
        title: String,
        author: String,
        state: String,
        created: String,
        updated: String,
        path: String,
    }

    let docs: Vec<JsonDoc> = index
        .all()
        .iter()
        .map(|doc| {
            let rel_path = doc.path.strip_prefix(index.docs_dir()).unwrap_or(&doc.path);

            JsonDoc {
                number: doc.metadata.number,
                title: doc.metadata.title.clone(),
                author: doc.metadata.author.clone(),
                state: doc.metadata.state.as_str().to_string(),
                created: doc.metadata.created.to_string(),
                updated: doc.metadata.updated.to_string(),
                path: rel_path.to_string_lossy().to_string(),
            }
        })
        .collect();

    let json = serde_json::to_string_pretty(&docs)?;

    let index_path = PathBuf::from(index.docs_dir()).join("00-index.json");
    fs::write(&index_path, json)?;

    println!("Generated JSON index at: {}", index_path.display());
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

    fn create_test_index_with_docs() -> (DocumentIndex, TempDir) {
        let temp = TempDir::new().unwrap();
        let mut state = DocumentState::new();

        // Create documents in different states
        for (num, title, doc_state) in [
            (1, "First Doc", DocState::Draft),
            (2, "Second Doc", DocState::Final),
            (3, "Third Doc", DocState::Active),
            (4, "Fourth Doc", DocState::Draft),
        ] {
            let meta = DocMetadata {
                number: num,
                title: title.to_string(),
                author: "Test Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, num).unwrap(),
                state: doc_state,
                supersedes: None,
                superseded_by: None,
            };
            state.upsert(
                num,
                DocumentRecord {
                    metadata: meta,
                    path: format!("{:04}-test.md", num),
                    checksum: "abc123".to_string(),
                    file_size: 100,
                    modified: chrono::Utc::now(),
                },
            );
        }

        let index = DocumentIndex::from_state(&state, temp.path()).unwrap();
        (index, temp)
    }

    #[test]
    fn test_generate_markdown_index() {
        let (index, temp) = create_test_index_with_docs();

        let result = generate_index(&index, "markdown");
        assert!(result.is_ok());

        // Verify file was created
        let index_path = temp.path().join("00-index.md");
        assert!(index_path.exists());

        // Verify content structure
        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("# Design Document Index"));
        assert!(content.contains("## All Documents by Number"));
        assert!(content.contains("| Number | Title | State | Updated |"));
        assert!(content.contains("## Documents by State"));

        // Verify document entries
        assert!(content.contains("0001"));
        assert!(content.contains("First Doc"));
        assert!(content.contains("0002"));
        assert!(content.contains("Second Doc"));
    }

    #[test]
    fn test_generate_markdown_index_with_md_format() {
        let (index, temp) = create_test_index_with_docs();

        let result = generate_index(&index, "md");
        assert!(result.is_ok());

        let index_path = temp.path().join("00-index.md");
        assert!(index_path.exists());
    }

    #[test]
    fn test_generate_json_index() {
        let (index, temp) = create_test_index_with_docs();

        let result = generate_index(&index, "json");
        assert!(result.is_ok());

        // Verify file was created
        let index_path = temp.path().join("00-index.json");
        assert!(index_path.exists());

        // Verify JSON structure
        let content = fs::read_to_string(&index_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.is_array());
        let docs = json.as_array().unwrap();
        assert_eq!(docs.len(), 4);

        // Verify first document structure
        let first = &docs[0];
        assert_eq!(first["number"], 1);
        assert_eq!(first["title"], "First Doc");
        assert_eq!(first["author"], "Test Author");
        assert!(first.get("state").is_some());
        assert!(first.get("created").is_some());
        assert!(first.get("updated").is_some());
        assert!(first.get("path").is_some());
    }

    #[test]
    fn test_generate_index_unknown_format_defaults_to_markdown() {
        let (index, temp) = create_test_index_with_docs();

        let result = generate_index(&index, "unknown-format");
        assert!(result.is_ok());

        // Should default to markdown
        let index_path = temp.path().join("00-index.md");
        assert!(index_path.exists());
    }

    #[test]
    fn test_generate_markdown_includes_all_states() {
        let (index, temp) = create_test_index_with_docs();

        let result = generate_index(&index, "markdown");
        assert!(result.is_ok());

        let content = fs::read_to_string(temp.path().join("00-index.md")).unwrap();

        // Verify state sections exist
        assert!(content.contains("### Draft"));
        assert!(content.contains("### Final"));
        assert!(content.contains("### Active"));

        // Verify documents are listed under their states
        // Draft section should have docs 1 and 4
        let draft_section = content.split("### Draft").nth(1).unwrap();
        assert!(draft_section.contains("0001"));
        assert!(draft_section.contains("0004"));
    }

    #[test]
    fn test_generate_empty_index() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        let result = generate_index(&index, "markdown");
        assert!(result.is_ok());

        let index_path = temp.path().join("00-index.md");
        assert!(index_path.exists());

        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("# Design Document Index"));
        // Should have headers but no document entries
    }

    #[test]
    fn test_generate_json_empty_index() {
        let temp = TempDir::new().unwrap();
        let index = DocumentIndex::new(temp.path()).unwrap();

        let result = generate_index(&index, "json");
        assert!(result.is_ok());

        let index_path = temp.path().join("00-index.json");
        assert!(index_path.exists());

        let content = fs::read_to_string(&index_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_markdown_table_formatting() {
        let (index, temp) = create_test_index_with_docs();

        let result = generate_index(&index, "markdown");
        assert!(result.is_ok());

        let content = fs::read_to_string(temp.path().join("00-index.md")).unwrap();

        // Verify table has correct format
        assert!(content.contains("|--------|-------|-------|----------|"));

        // Verify document numbers are formatted correctly (4 digits)
        assert!(content.contains("| 0001 |"));
        assert!(content.contains("| 0002 |"));
        assert!(content.contains("| 0003 |"));
        assert!(content.contains("| 0004 |"));
    }
}
