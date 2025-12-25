//! State management and persistence
//!
//! Provides a canonical source of truth for document state, with change detection
//! and atomic persistence.

use crate::doc::{DesignDoc, DocMetadata, DocState};
use crate::index_sync::get_docs_from_filesystem;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The canonical state of all documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    /// Schema version for migrations
    pub version: u32,

    /// When this state was last updated
    pub last_updated: DateTime<Utc>,

    /// All documents keyed by number
    pub documents: HashMap<u32, DocumentRecord>,

    /// Next available document number
    pub next_number: u32,
}

/// A single document's canonical record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecord {
    /// Document metadata
    pub metadata: DocMetadata,

    /// Relative path from docs_dir
    pub path: String,

    /// File checksum (for change detection)
    pub checksum: String,

    /// File size in bytes
    pub file_size: u64,

    /// Last modified time (from filesystem)
    pub modified: DateTime<Utc>,
}

impl Default for DocumentState {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentState {
    /// Create a new empty state
    pub fn new() -> Self {
        DocumentState {
            version: 1,
            last_updated: Utc::now(),
            documents: HashMap::new(),
            next_number: 1,
        }
    }

    /// Load state from disk
    pub fn load(state_dir: impl AsRef<Path>) -> Result<Self> {
        let state_file = state_dir.as_ref().join("state.json");

        if !state_file.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&state_file).context("Failed to read state file")?;

        let state: DocumentState =
            serde_json::from_str(&content).context("Failed to parse state file")?;

        Ok(state)
    }

    /// Save state to disk
    pub fn save(&self, state_dir: impl AsRef<Path>) -> Result<()> {
        let state_dir = state_dir.as_ref();
        fs::create_dir_all(state_dir).context("Failed to create state directory")?;

        // Create .gitignore if it doesn't exist
        let gitignore_path = state_dir.join(".gitignore");
        if !gitignore_path.exists() {
            fs::write(&gitignore_path, "*\n").context("Failed to create .gitignore")?;
        }

        let state_file = state_dir.join("state.json");
        let content = serde_json::to_string_pretty(self).context("Failed to serialize state")?;

        // Atomic write: write to temp file, then rename
        let temp_file = state_dir.join("state.json.tmp");
        fs::write(&temp_file, content).context("Failed to write temp state file")?;

        fs::rename(&temp_file, &state_file).context("Failed to rename state file")?;

        Ok(())
    }

    /// Add or update a document record
    pub fn upsert(&mut self, number: u32, record: DocumentRecord) {
        self.documents.insert(number, record);
        self.last_updated = Utc::now();

        // Update next_number if needed
        if number >= self.next_number {
            self.next_number = number + 1;
        }
    }

    /// Remove a document record
    pub fn remove(&mut self, number: u32) -> Option<DocumentRecord> {
        self.last_updated = Utc::now();
        self.documents.remove(&number)
    }

    /// Get a document record
    pub fn get(&self, number: u32) -> Option<&DocumentRecord> {
        self.documents.get(&number)
    }

    /// Get all documents sorted by number
    pub fn all(&self) -> Vec<&DocumentRecord> {
        let mut docs: Vec<_> = self.documents.values().collect();
        docs.sort_by_key(|d| d.metadata.number);
        docs
    }

    /// Get documents by state
    pub fn by_state(&self, state: DocState) -> Vec<&DocumentRecord> {
        let mut docs: Vec<_> =
            self.documents.values().filter(|d| d.metadata.state == state).collect();
        docs.sort_by_key(|d| d.metadata.number);
        docs
    }
}

// ============================================================================
// Checksum Utilities
// ============================================================================

/// Compute SHA-256 checksum of a file
pub fn compute_checksum(path: impl AsRef<Path>) -> Result<String> {
    let mut file = fs::File::open(path.as_ref()).context("Failed to open file for checksum")?;

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = file.read(&mut buffer).context("Failed to read file for checksum")?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Check if a file has changed based on checksum
pub fn file_changed(path: impl AsRef<Path>, expected_checksum: &str) -> Result<bool> {
    let actual = compute_checksum(path)?;
    Ok(actual != expected_checksum)
}

/// Get file metadata (size, mtime)
pub fn file_metadata(path: impl AsRef<Path>) -> Result<(u64, DateTime<Utc>)> {
    let metadata = fs::metadata(path.as_ref()).context("Failed to read file metadata")?;

    let size = metadata.len();
    let modified = metadata.modified().context("Failed to get modification time")?;

    let datetime = DateTime::<Utc>::from(modified);

    Ok((size, datetime))
}

// ============================================================================
// State Manager
// ============================================================================

/// State manager handles loading, updating, and persisting state
pub struct StateManager {
    state: DocumentState,
    docs_dir: PathBuf,
    state_dir: PathBuf,
}

impl StateManager {
    /// Initialize state manager
    pub fn new(docs_dir: impl AsRef<Path>) -> Result<Self> {
        let docs_dir = docs_dir.as_ref().to_path_buf();
        let state_dir = docs_dir.join(".oxd");

        // Load existing state or create new
        let state = DocumentState::load(&state_dir)?;

        Ok(StateManager { state, docs_dir, state_dir })
    }

    /// Initialize and scan for changes
    pub fn init_with_scan(&mut self) -> Result<ScanResult> {
        self.scan_for_changes()
    }

    /// Scan filesystem for changes
    pub fn scan_for_changes(&mut self) -> Result<ScanResult> {
        let mut result = ScanResult::new();

        // Get all docs from filesystem
        let filesystem_docs = get_docs_from_filesystem(&self.docs_dir)?;

        // Track which numbers we've seen on filesystem
        let mut seen_numbers = std::collections::HashSet::new();

        // Check each file
        for path in &filesystem_docs {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    result.errors.push(format!("Failed to read {}: {}", path.display(), e));
                    continue;
                }
            };

            // Parse document
            let doc = match DesignDoc::parse(&content, path.clone()) {
                Ok(d) => d,
                Err(e) => {
                    result.errors.push(format!("Failed to parse {}: {}", path.display(), e));
                    continue;
                }
            };

            let number = doc.metadata.number;
            seen_numbers.insert(number);

            // Check if we have a record
            if let Some(record) = self.state.get(number) {
                // Check if file changed
                match file_changed(path, &record.checksum) {
                    Ok(true) => {
                        result.changed.push(number);
                        self.update_record_from_file(&doc, path)?;
                    }
                    Ok(false) => {
                        // File unchanged
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to check checksum for {}: {}",
                            path.display(),
                            e
                        ));
                    }
                }
            } else {
                // New file not in state
                result.new_files.push(number);
                self.update_record_from_file(&doc, path)?;
            }
        }

        // Check for deleted files
        let state_numbers: Vec<u32> = self.state.documents.keys().copied().collect();
        for number in state_numbers {
            if !seen_numbers.contains(&number) {
                result.deleted.push(number);
            }
        }

        // Remove deleted files from state
        for number in &result.deleted {
            self.state.remove(*number);
        }

        // Save updated state
        self.save()?;

        Ok(result)
    }

    /// Quick scan using mtime/size checks before checksums
    pub fn quick_scan(&mut self) -> Result<ScanResult> {
        let mut result = ScanResult::new();

        // Get all docs from filesystem
        let filesystem_docs = get_docs_from_filesystem(&self.docs_dir)?;

        // Track which numbers we've seen on filesystem
        let mut seen_numbers = std::collections::HashSet::new();

        // Check each file
        for path in &filesystem_docs {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    result.errors.push(format!("Failed to read {}: {}", path.display(), e));
                    continue;
                }
            };

            // Parse document
            let doc = match DesignDoc::parse(&content, path.clone()) {
                Ok(d) => d,
                Err(e) => {
                    result.errors.push(format!("Failed to parse {}: {}", path.display(), e));
                    continue;
                }
            };

            let number = doc.metadata.number;
            seen_numbers.insert(number);

            // Check if we have a record
            if let Some(record) = self.state.get(number) {
                // Quick check first (mtime/size)
                if self.quick_check_changed(path, record)? {
                    // Verify with full checksum
                    if file_changed(path, &record.checksum)? {
                        result.changed.push(number);
                        self.update_record_from_file(&doc, path)?;
                    }
                }
            } else {
                // New file not in state
                result.new_files.push(number);
                self.update_record_from_file(&doc, path)?;
            }
        }

        // Check for deleted files
        let state_numbers: Vec<u32> = self.state.documents.keys().copied().collect();
        for number in state_numbers {
            if !seen_numbers.contains(&number) {
                result.deleted.push(number);
            }
        }

        // Remove deleted files from state
        for number in &result.deleted {
            self.state.remove(*number);
        }

        // Only save if there were changes
        if result.has_changes() {
            self.save()?;
        }

        Ok(result)
    }

    /// Quick check if file might have changed (without full checksum)
    fn quick_check_changed(&self, path: &Path, record: &DocumentRecord) -> Result<bool> {
        let (size, modified) = file_metadata(path)?;

        // Quick checks first
        if size != record.file_size {
            return Ok(true);
        }

        if modified > record.modified {
            return Ok(true);
        }

        // Size and mtime match, probably unchanged
        Ok(false)
    }

    /// Update a record from a file
    fn update_record_from_file(&mut self, doc: &DesignDoc, path: &Path) -> Result<()> {
        let checksum = compute_checksum(path)?;
        let (file_size, modified) = file_metadata(path)?;

        let rel_path =
            path.strip_prefix(&self.docs_dir).unwrap_or(path).to_string_lossy().to_string();

        let record = DocumentRecord {
            metadata: doc.metadata.clone(),
            path: rel_path,
            checksum,
            file_size,
            modified,
        };

        self.state.upsert(doc.metadata.number, record);
        Ok(())
    }

    /// Update state after modifying a file
    pub fn record_file_change(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        // Re-parse the file
        let content = std::fs::read_to_string(path).context("Failed to read modified file")?;

        let doc = DesignDoc::parse(&content, path.to_path_buf())
            .map_err(|e| anyhow::anyhow!("Failed to parse modified file: {}", e))?;

        // Update record
        self.update_record_from_file(&doc, path)?;

        // Save state
        self.save()?;

        Ok(())
    }

    /// Update state after moving a file
    pub fn record_file_move(
        &mut self,
        _old_path: impl AsRef<Path>,
        new_path: impl AsRef<Path>,
    ) -> Result<()> {
        // Just re-record the file at new location
        self.record_file_change(new_path)
    }

    /// Remove file from state
    pub fn record_file_deletion(&mut self, number: u32) -> Result<()> {
        self.state.remove(number);
        self.save()?;
        Ok(())
    }

    /// Get state reference
    pub fn state(&self) -> &DocumentState {
        &self.state
    }

    /// Get mutable state reference
    pub fn state_mut(&mut self) -> &mut DocumentState {
        &mut self.state
    }

    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        self.state.save(&self.state_dir)
    }

    /// Get docs directory
    pub fn docs_dir(&self) -> &Path {
        &self.docs_dir
    }

    /// Get next available document number
    pub fn next_number(&self) -> u32 {
        self.state.next_number
    }
}

/// Results of a filesystem scan
#[derive(Debug)]
pub struct ScanResult {
    pub new_files: Vec<u32>,
    pub changed: Vec<u32>,
    pub deleted: Vec<u32>,
    pub errors: Vec<String>,
}

impl ScanResult {
    fn new() -> Self {
        ScanResult {
            new_files: Vec::new(),
            changed: Vec::new(),
            deleted: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.new_files.is_empty() || !self.changed.is_empty() || !self.deleted.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.new_files.len() + self.changed.len() + self.deleted.len()
    }
}
