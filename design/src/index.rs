//! Document index management

use crate::doc::{DesignDoc, DocState};
use crate::state::DocumentState;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Manages the collection of design documents
pub struct DocumentIndex {
    docs: HashMap<u32, DesignDoc>,
    docs_dir: PathBuf,
}

impl DocumentIndex {
    /// Create a new index from a documentation directory
    pub fn new(docs_dir: impl AsRef<Path>) -> Result<Self> {
        let docs_dir = docs_dir.as_ref().to_path_buf();
        let mut index = DocumentIndex { docs: HashMap::new(), docs_dir: docs_dir.clone() };

        index.scan()?;
        Ok(index)
    }

    /// Scan the docs directory and load all documents
    pub fn scan(&mut self) -> Result<()> {
        self.docs.clear();

        for entry in
            WalkDir::new(&self.docs_dir).follow_links(true).into_iter().filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            if let Some(ext) = entry.path().extension() {
                if ext != "md" {
                    continue;
                }
            } else {
                continue;
            }

            // Skip the index file
            if entry.file_name() == "00-index.md" {
                continue;
            }

            let content = fs::read_to_string(entry.path())
                .context(format!("Failed to read {:?}", entry.path()))?;

            match DesignDoc::parse(&content, entry.path().to_path_buf()) {
                Ok(doc) => {
                    self.docs.insert(doc.metadata.number, doc);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {:?}: {}", entry.path(), e);
                }
            }
        }

        Ok(())
    }

    /// Get a document by number
    pub fn get(&self, number: u32) -> Option<&DesignDoc> {
        self.docs.get(&number)
    }

    /// Get all documents
    pub fn all(&self) -> Vec<&DesignDoc> {
        let mut docs: Vec<_> = self.docs.values().collect();
        docs.sort_by_key(|d| d.metadata.number);
        docs
    }

    /// Get documents by state
    pub fn by_state(&self, state: DocState) -> Vec<&DesignDoc> {
        let mut docs: Vec<_> = self.docs.values().filter(|d| d.metadata.state == state).collect();
        docs.sort_by_key(|d| d.metadata.number);
        docs
    }

    /// Get the next available document number
    pub fn next_number(&self) -> u32 {
        self.docs.keys().max().map(|n| n + 1).unwrap_or(1)
    }

    /// Get the docs directory path
    pub fn docs_dir(&self) -> &Path {
        &self.docs_dir
    }

    /// Create index from state (for fast loading from cache)
    pub fn from_state(state: &DocumentState, docs_dir: impl AsRef<Path>) -> Result<Self> {
        let docs_dir = docs_dir.as_ref().to_path_buf();
        let mut docs = HashMap::new();

        for record in state.documents.values() {
            let doc = DesignDoc {
                metadata: record.metadata.clone(),
                content: String::new(), // Don't load content unless needed
                path: docs_dir.join(&record.path),
            };
            docs.insert(record.metadata.number, doc);
        }

        Ok(DocumentIndex { docs, docs_dir })
    }

    /// Get document with lazy content loading
    pub fn get_with_content(&self, number: u32) -> Option<DesignDoc> {
        let doc = self.docs.get(&number)?;

        // If content is empty, load it
        if doc.content.is_empty() {
            if let Ok(content) = std::fs::read_to_string(&doc.path) {
                // Parse to get just the body content (after frontmatter)
                if let Ok(parsed) = DesignDoc::parse(&content, doc.path.clone()) {
                    return Some(parsed);
                }
            }
        }

        Some(doc.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::DocMetadata;
    use crate::state::DocumentRecord;
    use chrono::{NaiveDate, Utc};
    use tempfile::TempDir;

    fn create_test_metadata(number: u32, state: DocState) -> DocMetadata {
        DocMetadata {
            number,
            title: format!("Test Doc {}", number),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state,
            supersedes: None,
            superseded_by: None,
        }
    }

    fn create_test_doc_content(metadata: &DocMetadata, body: &str) -> String {
        let frontmatter = format!(
            "number: {}\ntitle: \"{}\"\nauthor: \"{}\"\ncreated: {}\nupdated: {}\nstate: {}\n",
            metadata.number,
            metadata.title,
            metadata.author,
            metadata.created,
            metadata.updated,
            metadata.state.as_str()
        );
        format!("---\n{}---\n\n{}", frontmatter, body)
    }

    mod construction {
        use super::*;

        #[test]
        fn test_new_empty_directory() {
            let temp = TempDir::new().unwrap();
            let index = DocumentIndex::new(temp.path()).unwrap();

            assert_eq!(index.all().len(), 0);
            assert_eq!(index.docs_dir(), temp.path());
        }

        #[test]
        fn test_new_with_documents() {
            let temp = TempDir::new().unwrap();

            // Create a few test documents
            let meta1 = create_test_metadata(1, DocState::Draft);
            let content1 = create_test_doc_content(&meta1, "# Doc 1\n\nContent");
            fs::write(temp.path().join("0001-test.md"), content1).unwrap();

            let meta2 = create_test_metadata(2, DocState::Final);
            let content2 = create_test_doc_content(&meta2, "# Doc 2\n\nContent");
            fs::write(temp.path().join("0002-test.md"), content2).unwrap();

            let index = DocumentIndex::new(temp.path()).unwrap();

            assert_eq!(index.all().len(), 2);
            assert!(index.get(1).is_some());
            assert!(index.get(2).is_some());
        }

        #[test]
        fn test_scan_subdirectories() {
            let temp = TempDir::new().unwrap();

            // Create nested directory structure
            let draft_dir = temp.path().join("01-draft");
            fs::create_dir_all(&draft_dir).unwrap();

            let meta = create_test_metadata(5, DocState::Draft);
            let content = create_test_doc_content(&meta, "# Doc 5\n\nContent");
            fs::write(draft_dir.join("0005-nested.md"), content).unwrap();

            let index = DocumentIndex::new(temp.path()).unwrap();

            assert_eq!(index.all().len(), 1);
            assert!(index.get(5).is_some());
        }

        #[test]
        fn test_scan_skips_non_markdown() {
            let temp = TempDir::new().unwrap();

            // Create markdown file
            let meta = create_test_metadata(1, DocState::Draft);
            let content = create_test_doc_content(&meta, "# Doc 1\n\nContent");
            fs::write(temp.path().join("0001-test.md"), content).unwrap();

            // Create non-markdown files
            fs::write(temp.path().join("readme.txt"), "Not markdown").unwrap();
            fs::write(temp.path().join("data.json"), "{}").unwrap();
            fs::write(temp.path().join("noext"), "No extension").unwrap();

            let index = DocumentIndex::new(temp.path()).unwrap();

            // Should only load the .md file
            assert_eq!(index.all().len(), 1);
        }

        #[test]
        fn test_scan_skips_index_file() {
            let temp = TempDir::new().unwrap();

            // Create regular document
            let meta = create_test_metadata(1, DocState::Draft);
            let content = create_test_doc_content(&meta, "# Doc 1\n\nContent");
            fs::write(temp.path().join("0001-test.md"), content).unwrap();

            // Create index file (should be skipped)
            fs::write(temp.path().join("00-index.md"), "# Index\n\nShould be skipped").unwrap();

            let index = DocumentIndex::new(temp.path()).unwrap();

            // Should only find the regular document, not the index
            assert_eq!(index.all().len(), 1);
            assert!(index.get(1).is_some());
        }

        #[test]
        fn test_scan_handles_invalid_documents() {
            let temp = TempDir::new().unwrap();

            // Create valid document
            let meta = create_test_metadata(1, DocState::Draft);
            let content = create_test_doc_content(&meta, "# Doc 1\n\nContent");
            fs::write(temp.path().join("0001-valid.md"), content).unwrap();

            // Create invalid document (malformed frontmatter)
            fs::write(temp.path().join("0002-invalid.md"), "---\nbroken yaml:\n  -\n---\n# Bad")
                .unwrap();

            let index = DocumentIndex::new(temp.path()).unwrap();

            // Should load valid doc, skip invalid one with warning
            assert_eq!(index.all().len(), 1);
            assert!(index.get(1).is_some());
            assert!(index.get(2).is_none());
        }

        #[test]
        fn test_rescan_clears_previous() {
            let temp = TempDir::new().unwrap();

            // Create initial document
            let meta1 = create_test_metadata(1, DocState::Draft);
            let content1 = create_test_doc_content(&meta1, "# Doc 1\n\nContent");
            fs::write(temp.path().join("0001-test.md"), content1).unwrap();

            let mut index = DocumentIndex::new(temp.path()).unwrap();
            assert_eq!(index.all().len(), 1);

            // Remove file and add new one
            fs::remove_file(temp.path().join("0001-test.md")).unwrap();

            let meta2 = create_test_metadata(2, DocState::Final);
            let content2 = create_test_doc_content(&meta2, "# Doc 2\n\nContent");
            fs::write(temp.path().join("0002-new.md"), content2).unwrap();

            // Rescan
            index.scan().unwrap();

            // Should have new document, not old one
            assert_eq!(index.all().len(), 1);
            assert!(index.get(1).is_none());
            assert!(index.get(2).is_some());
        }
    }

    mod getters {
        use super::*;

        fn create_test_index() -> (TempDir, DocumentIndex) {
            let temp = TempDir::new().unwrap();

            // Create docs in different states
            for (num, state) in [
                (1, DocState::Draft),
                (2, DocState::Draft),
                (3, DocState::Final),
                (5, DocState::Accepted),
                (10, DocState::Draft),
            ] {
                let meta = create_test_metadata(num, state);
                let content = create_test_doc_content(&meta, &format!("# Doc {}\n\nContent", num));
                fs::write(temp.path().join(format!("{:04}-test.md", num)), content).unwrap();
            }

            let index = DocumentIndex::new(temp.path()).unwrap();
            (temp, index)
        }

        #[test]
        fn test_get_existing() {
            let (_temp, index) = create_test_index();

            let doc = index.get(3);
            assert!(doc.is_some());
            assert_eq!(doc.unwrap().metadata.number, 3);
        }

        #[test]
        fn test_get_missing() {
            let (_temp, index) = create_test_index();

            assert!(index.get(99).is_none());
        }

        #[test]
        fn test_all_sorted_by_number() {
            let (_temp, index) = create_test_index();

            let all = index.all();
            assert_eq!(all.len(), 5);

            // Should be sorted by number
            assert_eq!(all[0].metadata.number, 1);
            assert_eq!(all[1].metadata.number, 2);
            assert_eq!(all[2].metadata.number, 3);
            assert_eq!(all[3].metadata.number, 5);
            assert_eq!(all[4].metadata.number, 10);
        }

        #[test]
        fn test_by_state_draft() {
            let (_temp, index) = create_test_index();

            let drafts = index.by_state(DocState::Draft);
            assert_eq!(drafts.len(), 3);

            // Should be sorted by number
            assert_eq!(drafts[0].metadata.number, 1);
            assert_eq!(drafts[1].metadata.number, 2);
            assert_eq!(drafts[2].metadata.number, 10);
        }

        #[test]
        fn test_by_state_final() {
            let (_temp, index) = create_test_index();

            let finals = index.by_state(DocState::Final);
            assert_eq!(finals.len(), 1);
            assert_eq!(finals[0].metadata.number, 3);
        }

        #[test]
        fn test_by_state_empty() {
            let (_temp, index) = create_test_index();

            let rejected = index.by_state(DocState::Rejected);
            assert_eq!(rejected.len(), 0);
        }

        #[test]
        fn test_next_number_with_docs() {
            let (_temp, index) = create_test_index();

            // Highest number is 10
            assert_eq!(index.next_number(), 11);
        }

        #[test]
        fn test_next_number_empty() {
            let temp = TempDir::new().unwrap();
            let index = DocumentIndex::new(temp.path()).unwrap();

            assert_eq!(index.next_number(), 1);
        }

        #[test]
        fn test_docs_dir() {
            let temp = TempDir::new().unwrap();
            let index = DocumentIndex::new(temp.path()).unwrap();

            assert_eq!(index.docs_dir(), temp.path());
        }
    }

    mod from_state {
        use super::*;

        #[test]
        fn test_from_state_empty() {
            let temp = TempDir::new().unwrap();
            let state = DocumentState::new();

            let index = DocumentIndex::from_state(&state, temp.path()).unwrap();

            assert_eq!(index.all().len(), 0);
            assert_eq!(index.docs_dir(), temp.path());
        }

        #[test]
        fn test_from_state_with_records() {
            let temp = TempDir::new().unwrap();
            let mut state = DocumentState::new();

            // Add some state records
            let meta1 = create_test_metadata(1, DocState::Draft);
            let record1 = DocumentRecord {
                metadata: meta1,
                path: "01-draft/0001-test.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: Utc::now(),
            };
            state.upsert(1, record1);

            let meta2 = create_test_metadata(2, DocState::Final);
            let record2 = DocumentRecord {
                metadata: meta2,
                path: "06-final/0002-test.md".to_string(),
                checksum: "def456".to_string(),
                file_size: 200,
                modified: Utc::now(),
            };
            state.upsert(2, record2);

            let index = DocumentIndex::from_state(&state, temp.path()).unwrap();

            assert_eq!(index.all().len(), 2);
            assert!(index.get(1).is_some());
            assert!(index.get(2).is_some());
        }

        #[test]
        fn test_from_state_lazy_content() {
            let temp = TempDir::new().unwrap();
            let mut state = DocumentState::new();

            let meta = create_test_metadata(1, DocState::Draft);
            let record = DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: Utc::now(),
            };
            state.upsert(1, record);

            let index = DocumentIndex::from_state(&state, temp.path()).unwrap();

            let doc = index.get(1).unwrap();
            // Content should be empty (lazy loading)
            assert_eq!(doc.content, "");
        }

        #[test]
        fn test_from_state_correct_paths() {
            let temp = TempDir::new().unwrap();
            let mut state = DocumentState::new();

            let meta = create_test_metadata(1, DocState::Draft);
            let record = DocumentRecord {
                metadata: meta,
                path: "subdir/0001-test.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: Utc::now(),
            };
            state.upsert(1, record);

            let index = DocumentIndex::from_state(&state, temp.path()).unwrap();

            let doc = index.get(1).unwrap();
            assert_eq!(doc.path, temp.path().join("subdir/0001-test.md"));
        }
    }

    mod lazy_loading {
        use super::*;

        #[test]
        fn test_get_with_content_missing_doc() {
            let temp = TempDir::new().unwrap();
            let index = DocumentIndex::new(temp.path()).unwrap();

            assert!(index.get_with_content(99).is_none());
        }

        #[test]
        fn test_get_with_content_already_loaded() {
            let temp = TempDir::new().unwrap();

            let meta = create_test_metadata(1, DocState::Draft);
            let content = create_test_doc_content(&meta, "# Doc 1\n\nTest content here");
            fs::write(temp.path().join("0001-test.md"), &content).unwrap();

            let index = DocumentIndex::new(temp.path()).unwrap();

            let doc = index.get_with_content(1);
            assert!(doc.is_some());

            let doc = doc.unwrap();
            assert_eq!(doc.metadata.number, 1);
            // Content should be loaded
            assert!(doc.content.contains("Test content here"));
        }

        #[test]
        fn test_get_with_content_lazy_load() {
            let temp = TempDir::new().unwrap();
            let mut state = DocumentState::new();

            // Create actual file
            let meta = create_test_metadata(1, DocState::Draft);
            let content = create_test_doc_content(&meta, "# Doc 1\n\nLazy loaded content");
            fs::write(temp.path().join("0001-test.md"), &content).unwrap();

            // Add to state (will have empty content)
            let record = DocumentRecord {
                metadata: meta,
                path: "0001-test.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: Utc::now(),
            };
            state.upsert(1, record);

            let index = DocumentIndex::from_state(&state, temp.path()).unwrap();

            // Doc should exist but content empty
            let doc = index.get(1).unwrap();
            assert_eq!(doc.content, "");

            // Lazy load should fill content
            let doc_with_content = index.get_with_content(1);
            assert!(doc_with_content.is_some());

            let doc = doc_with_content.unwrap();
            assert!(doc.content.contains("Lazy loaded content"));
        }

        #[test]
        fn test_get_with_content_file_missing() {
            let temp = TempDir::new().unwrap();
            let mut state = DocumentState::new();

            // Add to state but DON'T create file
            let meta = create_test_metadata(1, DocState::Draft);
            let record = DocumentRecord {
                metadata: meta.clone(),
                path: "0001-missing.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: Utc::now(),
            };
            state.upsert(1, record);

            let index = DocumentIndex::from_state(&state, temp.path()).unwrap();

            // Should still return doc (with empty content) even if file doesn't exist
            let doc = index.get_with_content(1);
            assert!(doc.is_some());

            let doc = doc.unwrap();
            assert_eq!(doc.content, "");
            assert_eq!(doc.metadata.number, 1);
        }

        #[test]
        fn test_get_with_content_parse_error() {
            let temp = TempDir::new().unwrap();
            let mut state = DocumentState::new();

            // Create file with invalid content
            fs::write(temp.path().join("0001-bad.md"), "---\nbroken yaml\n  -\n---\nContent")
                .unwrap();

            let meta = create_test_metadata(1, DocState::Draft);
            let record = DocumentRecord {
                metadata: meta.clone(),
                path: "0001-bad.md".to_string(),
                checksum: "abc123".to_string(),
                file_size: 100,
                modified: Utc::now(),
            };
            state.upsert(1, record);

            let index = DocumentIndex::from_state(&state, temp.path()).unwrap();

            // Should return doc with empty content (parse failed)
            let doc = index.get_with_content(1);
            assert!(doc.is_some());

            let doc = doc.unwrap();
            assert_eq!(doc.content, "");
        }
    }
}
