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
