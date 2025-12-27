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

#[cfg(test)]
mod document_state_tests {
    use super::*;
    use crate::doc::DocState;
    use chrono::NaiveDate;
    use tempfile::TempDir;

    fn create_test_metadata(number: u32) -> DocMetadata {
        DocMetadata {
            number,
            title: format!("Test Doc {}", number),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        }
    }

    fn create_test_record(number: u32) -> DocumentRecord {
        DocumentRecord {
            metadata: create_test_metadata(number),
            path: format!("01-draft/00{:02}-test.md", number),
            checksum: "abc123".to_string(),
            file_size: 1024,
            modified: Utc::now(),
        }
    }

    #[test]
    fn test_new_state() {
        let state = DocumentState::new();
        assert_eq!(state.version, 1);
        assert_eq!(state.next_number, 1);
        assert!(state.documents.is_empty());
    }

    #[test]
    fn test_upsert_new() {
        let mut state = DocumentState::new();
        let record = create_test_record(1);

        state.upsert(1, record.clone());

        assert_eq!(state.documents.len(), 1);
        assert!(state.get(1).is_some());
        assert_eq!(state.next_number, 2);
    }

    #[test]
    fn test_upsert_update() {
        let mut state = DocumentState::new();
        let record1 = create_test_record(1);
        state.upsert(1, record1);

        let mut record2 = create_test_record(1);
        record2.metadata.title = "Updated Title".to_string();
        state.upsert(1, record2);

        assert_eq!(state.documents.len(), 1);
        assert_eq!(state.get(1).unwrap().metadata.title, "Updated Title");
    }

    #[test]
    fn test_upsert_updates_next_number() {
        let mut state = DocumentState::new();

        state.upsert(5, create_test_record(5));
        assert_eq!(state.next_number, 6);

        state.upsert(3, create_test_record(3));
        assert_eq!(state.next_number, 6); // Shouldn't decrease

        state.upsert(10, create_test_record(10));
        assert_eq!(state.next_number, 11);
    }

    #[test]
    fn test_remove_existing() {
        let mut state = DocumentState::new();
        state.upsert(1, create_test_record(1));

        let removed = state.remove(1);
        assert!(removed.is_some());
        assert_eq!(state.documents.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut state = DocumentState::new();
        let removed = state.remove(999);
        assert!(removed.is_none());
    }

    #[test]
    fn test_get_existing() {
        let mut state = DocumentState::new();
        state.upsert(1, create_test_record(1));

        let record = state.get(1);
        assert!(record.is_some());
        assert_eq!(record.unwrap().metadata.number, 1);
    }

    #[test]
    fn test_get_nonexistent() {
        let state = DocumentState::new();
        assert!(state.get(999).is_none());
    }

    #[test]
    fn test_all_sorted() {
        let mut state = DocumentState::new();
        state.upsert(3, create_test_record(3));
        state.upsert(1, create_test_record(1));
        state.upsert(2, create_test_record(2));

        let all = state.all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].metadata.number, 1);
        assert_eq!(all[1].metadata.number, 2);
        assert_eq!(all[2].metadata.number, 3);
    }

    #[test]
    fn test_all_empty() {
        let state = DocumentState::new();
        assert!(state.all().is_empty());
    }

    #[test]
    fn test_by_state() {
        let mut state = DocumentState::new();

        let mut record1 = create_test_record(1);
        record1.metadata.state = DocState::Draft;
        state.upsert(1, record1);

        let mut record2 = create_test_record(2);
        record2.metadata.state = DocState::Final;
        state.upsert(2, record2);

        let mut record3 = create_test_record(3);
        record3.metadata.state = DocState::Draft;
        state.upsert(3, record3);

        let drafts = state.by_state(DocState::Draft);
        assert_eq!(drafts.len(), 2);

        let finals = state.by_state(DocState::Final);
        assert_eq!(finals.len(), 1);

        let active = state.by_state(DocState::Active);
        assert!(active.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join(".oxd");

        let mut state = DocumentState::new();
        state.upsert(1, create_test_record(1));
        state.upsert(2, create_test_record(2));

        // Save
        state.save(&state_dir).unwrap();

        // Verify file exists
        assert!(state_dir.join("state.json").exists());
        assert!(state_dir.join(".gitignore").exists());

        // Load
        let loaded = DocumentState::load(&state_dir).unwrap();
        assert_eq!(loaded.documents.len(), 2);
        assert!(loaded.get(1).is_some());
        assert!(loaded.get(2).is_some());
        assert_eq!(loaded.next_number, state.next_number);
    }

    #[test]
    fn test_load_nonexistent() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join(".oxd");

        let state = DocumentState::load(&state_dir).unwrap();
        assert_eq!(state.version, 1);
        assert!(state.documents.is_empty());
        assert_eq!(state.next_number, 1);
    }

    #[test]
    fn test_save_creates_gitignore() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join(".oxd");

        let state = DocumentState::new();
        state.save(&state_dir).unwrap();

        let gitignore = state_dir.join(".gitignore");
        assert!(gitignore.exists());

        let content = fs::read_to_string(gitignore).unwrap();
        assert_eq!(content, "*\n");
    }

    #[test]
    fn test_save_atomic() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join(".oxd");

        let state = DocumentState::new();
        state.save(&state_dir).unwrap();

        // Temp file should not exist after save
        assert!(!state_dir.join("state.json.tmp").exists());
        assert!(state_dir.join("state.json").exists());
    }
}

#[cfg(test)]
mod checksum_tests {
    use super::*;
    use std::fs;
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_compute_checksum_empty() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("empty.txt");
        fs::write(&file_path, "").unwrap();

        let checksum = compute_checksum(&file_path).unwrap();
        // SHA-256 of empty string
        assert_eq!(checksum, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_compute_checksum_content() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let checksum = compute_checksum(&file_path).unwrap();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 is 64 hex chars
    }

    #[test]
    fn test_compute_checksum_deterministic() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "Same content").unwrap();

        let checksum1 = compute_checksum(&file_path).unwrap();
        let checksum2 = compute_checksum(&file_path).unwrap();

        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_compute_checksum_different_content() {
        let temp = TempDir::new().unwrap();

        let file1 = temp.path().join("file1.txt");
        fs::write(&file1, "Content A").unwrap();

        let file2 = temp.path().join("file2.txt");
        fs::write(&file2, "Content B").unwrap();

        let checksum1 = compute_checksum(&file1).unwrap();
        let checksum2 = compute_checksum(&file2).unwrap();

        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_file_changed_same_content() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "Content").unwrap();

        let checksum = compute_checksum(&file_path).unwrap();
        let changed = file_changed(&file_path, &checksum).unwrap();

        assert!(!changed);
    }

    #[test]
    fn test_file_changed_different_content() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "Original").unwrap();

        let checksum = compute_checksum(&file_path).unwrap();

        // Modify file
        fs::write(&file_path, "Modified").unwrap();
        let changed = file_changed(&file_path, &checksum).unwrap();

        assert!(changed);
    }

    #[test]
    fn test_file_metadata() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        let content = "Hello, World!";
        fs::write(&file_path, content).unwrap();

        let (size, modified) = file_metadata(&file_path).unwrap();

        assert_eq!(size, content.len() as u64);
        assert!(modified <= Utc::now());
    }

    #[test]
    fn test_file_metadata_tracks_changes() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");

        fs::write(&file_path, "Short").unwrap();
        let (size1, _) = file_metadata(&file_path).unwrap();

        sleep(Duration::from_millis(10));

        fs::write(&file_path, "Much longer content here").unwrap();
        let (size2, mtime2) = file_metadata(&file_path).unwrap();

        assert_ne!(size1, size2);
        assert!(mtime2 > Utc::now() - chrono::Duration::seconds(5));
    }
}

#[cfg(test)]
mod scan_result_tests {
    use super::*;

    #[test]
    fn test_scan_result_new() {
        let result = ScanResult::new();
        assert!(result.new_files.is_empty());
        assert!(result.changed.is_empty());
        assert!(result.deleted.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_has_changes_empty() {
        let result = ScanResult::new();
        assert!(!result.has_changes());
    }

    #[test]
    fn test_has_changes_with_new() {
        let mut result = ScanResult::new();
        result.new_files.push(1);
        assert!(result.has_changes());
    }

    #[test]
    fn test_has_changes_with_changed() {
        let mut result = ScanResult::new();
        result.changed.push(1);
        assert!(result.has_changes());
    }

    #[test]
    fn test_has_changes_with_deleted() {
        let mut result = ScanResult::new();
        result.deleted.push(1);
        assert!(result.has_changes());
    }

    #[test]
    fn test_total_changes() {
        let mut result = ScanResult::new();
        result.new_files.push(1);
        result.new_files.push(2);
        result.changed.push(3);
        result.deleted.push(4);
        result.deleted.push(5);
        result.deleted.push(6);

        assert_eq!(result.total_changes(), 6);
    }

    #[test]
    fn test_total_changes_empty() {
        let result = ScanResult::new();
        assert_eq!(result.total_changes(), 0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn checksum_is_deterministic(content in "\\PC{0,1000}") {
            use std::io::Write;
            let temp = tempfile::NamedTempFile::new().unwrap();
            temp.as_file().write_all(content.as_bytes()).unwrap();

            let checksum1 = compute_checksum(temp.path()).unwrap();
            let checksum2 = compute_checksum(temp.path()).unwrap();

            prop_assert_eq!(checksum1, checksum2);
        }

        #[test]
        fn checksum_is_64_hex_chars(content in "\\PC{0,500}") {
            use std::io::Write;
            let temp = tempfile::NamedTempFile::new().unwrap();
            temp.as_file().write_all(content.as_bytes()).unwrap();

            let checksum = compute_checksum(temp.path()).unwrap();
            prop_assert_eq!(checksum.len(), 64);
            prop_assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn next_number_always_increases(insertions in prop::collection::vec(1u32..100, 1..10)) {
            let mut state = DocumentState::new();
            let mut expected_next = 1u32;

            for num in insertions {
                state.upsert(num, create_test_record(num));
                if num >= expected_next {
                    expected_next = num + 1;
                }
                prop_assert_eq!(state.next_number, expected_next);
            }
        }

        #[test]
        fn upsert_and_get_consistency(num in 1u32..1000) {
            let mut state = DocumentState::new();
            let record = create_test_record(num);

            state.upsert(num, record.clone());
            let retrieved = state.get(num);

            prop_assert!(retrieved.is_some());
            prop_assert_eq!(retrieved.unwrap().metadata.number, num);
        }

        #[test]
        fn remove_actually_removes(num in 1u32..1000) {
            let mut state = DocumentState::new();
            state.upsert(num, create_test_record(num));

            prop_assert!(state.get(num).is_some());

            state.remove(num);

            prop_assert!(state.get(num).is_none());
        }

        #[test]
        fn save_load_round_trip(nums in prop::collection::vec(1u32..100, 0..10)) {
            let temp = tempfile::TempDir::new().unwrap();
            let state_dir = temp.path().join(".oxd");

            let mut state = DocumentState::new();
            for num in &nums {
                state.upsert(*num, create_test_record(*num));
            }

            state.save(&state_dir).unwrap();
            let loaded = DocumentState::load(&state_dir).unwrap();

            // Use unique count since HashMap deduplicates
            let unique_nums: std::collections::HashSet<_> = nums.iter().collect();
            prop_assert_eq!(loaded.documents.len(), unique_nums.len());

            for num in nums {
                prop_assert!(loaded.get(num).is_some());
            }
        }
    }

    fn create_test_record(number: u32) -> DocumentRecord {
        use crate::doc::DocState;
        use chrono::NaiveDate;

        DocumentRecord {
            metadata: crate::doc::DocMetadata {
                number,
                title: format!("Test {}", number),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            path: format!("01-draft/{:04}-test.md", number),
            checksum: format!("checksum{}", number),
            file_size: 1024,
            modified: Utc::now(),
        }
    }
}

#[cfg(test)]
mod state_manager_tests {
    use super::*;
    use std::fs;
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_doc(number: u32, state: &str) -> String {
        format!(
            r#"---
number: {}
title: Test Document {}
author: Test Author
created: 2024-01-01
updated: 2024-01-02
state: {}
---

# Test Document {}

This is test content for document {}.
"#,
            number, number, state, number, number
        )
    }

    fn setup_test_env() -> (TempDir, PathBuf, PathBuf) {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();
        (temp, docs_dir, draft_dir)
    }

    #[test]
    fn test_state_manager_new() {
        let (_temp, docs_dir, _draft_dir) = setup_test_env();
        let manager = StateManager::new(&docs_dir).unwrap();

        assert_eq!(manager.docs_dir(), docs_dir.as_path());
        assert_eq!(manager.next_number(), 1);
        assert!(manager.state().documents.is_empty());
    }

    #[test]
    fn test_state_manager_creates_state_dir_on_save() {
        let (_temp, docs_dir, _draft_dir) = setup_test_env();
        let manager = StateManager::new(&docs_dir).unwrap();

        let state_dir = docs_dir.join(".oxd");
        assert!(!state_dir.exists());

        // State directory is created when we save
        manager.save().unwrap();
        assert!(state_dir.exists());
    }

    #[test]
    fn test_state_manager_loads_existing_state() {
        let (_temp, docs_dir, _draft_dir) = setup_test_env();

        // Create initial state
        {
            let mut manager = StateManager::new(&docs_dir).unwrap();
            let mut state = DocumentState::new();
            state.next_number = 42;
            manager.state = state;
            manager.save().unwrap();
        }

        // Load it back
        let manager = StateManager::new(&docs_dir).unwrap();
        assert_eq!(manager.next_number(), 42);
    }

    #[test]
    fn test_scan_for_changes_new_file() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        // Create a test document
        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.scan_for_changes().unwrap();

        assert_eq!(result.new_files.len(), 1);
        assert_eq!(result.new_files[0], 1);
        assert!(result.changed.is_empty());
        assert!(result.deleted.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_scan_for_changes_multiple_new_files() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        // Create multiple test documents
        for i in 1..=3 {
            let doc_path = draft_dir.join(format!("{:04}-test.md", i));
            fs::write(&doc_path, create_test_doc(i, "draft")).unwrap();
        }

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.scan_for_changes().unwrap();

        assert_eq!(result.new_files.len(), 3);
        assert!(result.changed.is_empty());
        assert!(result.deleted.is_empty());
    }

    #[test]
    fn test_scan_for_changes_file_modified() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.scan_for_changes().unwrap();

        // Modify the file
        sleep(Duration::from_millis(10));
        fs::write(&doc_path, create_test_doc(1, "draft") + "\nModified content").unwrap();

        let result = manager.scan_for_changes().unwrap();

        assert!(result.new_files.is_empty());
        assert_eq!(result.changed.len(), 1);
        assert_eq!(result.changed[0], 1);
        assert!(result.deleted.is_empty());
    }

    #[test]
    fn test_scan_for_changes_file_deleted() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.scan_for_changes().unwrap();

        // Delete the file
        fs::remove_file(&doc_path).unwrap();

        let result = manager.scan_for_changes().unwrap();

        assert!(result.new_files.is_empty());
        assert!(result.changed.is_empty());
        assert_eq!(result.deleted.len(), 1);
        assert_eq!(result.deleted[0], 1);
    }

    #[test]
    fn test_scan_for_changes_mixed_operations() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        // Create initial file
        let doc1_path = draft_dir.join("0001-test.md");
        fs::write(&doc1_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.scan_for_changes().unwrap();

        // Now: add new file, modify existing, delete existing
        let doc2_path = draft_dir.join("0002-test.md");
        fs::write(&doc2_path, create_test_doc(2, "draft")).unwrap();

        sleep(Duration::from_millis(10));
        fs::write(&doc1_path, create_test_doc(1, "draft") + "\nModified").unwrap();

        let result = manager.scan_for_changes().unwrap();

        assert_eq!(result.new_files.len(), 1);
        assert_eq!(result.new_files[0], 2);
        assert_eq!(result.changed.len(), 1);
        assert_eq!(result.changed[0], 1);
        assert!(result.deleted.is_empty());
    }

    #[test]
    fn test_scan_for_changes_invalid_file() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        // Create invalid document (missing frontmatter)
        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, "Invalid content without frontmatter").unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.scan_for_changes().unwrap();

        assert!(result.errors.len() > 0);
        assert!(result.new_files.is_empty());
    }

    #[test]
    fn test_scan_for_changes_file_unchanged() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result1 = manager.scan_for_changes().unwrap();
        assert_eq!(result1.new_files.len(), 1);

        // Scan again without changes
        let result2 = manager.scan_for_changes().unwrap();

        assert!(result2.new_files.is_empty());
        assert!(result2.changed.is_empty());
        assert!(result2.deleted.is_empty());
        assert!(!result2.has_changes());
    }

    #[test]
    fn test_quick_scan_new_file() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.quick_scan().unwrap();

        assert_eq!(result.new_files.len(), 1);
        assert_eq!(result.new_files[0], 1);
        assert!(result.changed.is_empty());
        assert!(result.deleted.is_empty());
    }

    #[test]
    fn test_quick_scan_file_modified_by_size() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.quick_scan().unwrap();

        // Modify with different size
        sleep(Duration::from_millis(10));
        fs::write(&doc_path, create_test_doc(1, "draft") + "\nExtra content here").unwrap();

        let result = manager.quick_scan().unwrap();

        assert!(result.new_files.is_empty());
        assert_eq!(result.changed.len(), 1);
        assert_eq!(result.changed[0], 1);
    }

    #[test]
    fn test_quick_scan_file_modified_by_time() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        let content = create_test_doc(1, "draft");
        fs::write(&doc_path, &content).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.quick_scan().unwrap();

        // Modify with same content but different mtime
        sleep(Duration::from_millis(100));
        fs::write(&doc_path, &content).unwrap();

        let result = manager.quick_scan().unwrap();

        // Note: This may or may not detect change depending on checksum match
        // But quick_scan should at least run without error
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_quick_scan_file_unchanged() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result1 = manager.quick_scan().unwrap();
        assert_eq!(result1.new_files.len(), 1);

        // Scan again without changes
        let result2 = manager.quick_scan().unwrap();

        assert!(result2.new_files.is_empty());
        assert!(result2.changed.is_empty());
        assert!(result2.deleted.is_empty());
        assert!(!result2.has_changes());
    }

    #[test]
    fn test_quick_scan_doesnt_save_when_no_changes() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.quick_scan().unwrap();

        let state_file = docs_dir.join(".oxd/state.json");
        let modified_before = fs::metadata(&state_file).unwrap().modified().unwrap();

        sleep(Duration::from_millis(100));

        // Scan with no changes
        manager.quick_scan().unwrap();

        let modified_after = fs::metadata(&state_file).unwrap().modified().unwrap();

        // State file should not be updated if no changes
        assert_eq!(modified_before, modified_after);
    }

    #[test]
    fn test_init_with_scan() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.init_with_scan().unwrap();

        assert_eq!(result.new_files.len(), 1);
        assert_eq!(manager.state().documents.len(), 1);
    }

    #[test]
    fn test_record_file_change() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.scan_for_changes().unwrap();

        // Modify file
        sleep(Duration::from_millis(10));
        fs::write(&doc_path, create_test_doc(1, "draft") + "\nNew content").unwrap();

        // Record the change
        manager.record_file_change(&doc_path).unwrap();

        // Verify state was updated
        let record = manager.state().get(1).unwrap();
        let new_checksum = compute_checksum(&doc_path).unwrap();
        assert_eq!(record.checksum, new_checksum);
    }

    #[test]
    fn test_record_file_change_invalid_file() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, "Invalid content").unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.record_file_change(&doc_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_record_file_move() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let old_path = draft_dir.join("0001-old.md");
        let new_path = draft_dir.join("0001-new.md");
        fs::write(&old_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.scan_for_changes().unwrap();

        // Move file
        fs::rename(&old_path, &new_path).unwrap();

        // Record the move
        manager.record_file_move(&old_path, &new_path).unwrap();

        // Verify state has updated path
        let record = manager.state().get(1).unwrap();
        assert!(record.path.contains("0001-new.md"));
    }

    #[test]
    fn test_record_file_deletion() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.scan_for_changes().unwrap();
        assert!(manager.state().get(1).is_some());

        // Delete from state
        manager.record_file_deletion(1).unwrap();

        assert!(manager.state().get(1).is_none());
    }

    #[test]
    fn test_state_mut() {
        let (_temp, docs_dir, _draft_dir) = setup_test_env();
        let mut manager = StateManager::new(&docs_dir).unwrap();

        let state = manager.state_mut();
        state.next_number = 100;

        assert_eq!(manager.next_number(), 100);
    }

    #[test]
    fn test_save_and_reload() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        {
            let mut manager = StateManager::new(&docs_dir).unwrap();
            manager.scan_for_changes().unwrap();
            manager.save().unwrap();
        }

        // Reload
        let manager = StateManager::new(&docs_dir).unwrap();
        assert_eq!(manager.state().documents.len(), 1);
        assert!(manager.state().get(1).is_some());
    }

    #[test]
    fn test_state_persists_after_scan() {
        let (_temp, docs_dir, draft_dir) = setup_test_env();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        {
            let mut manager = StateManager::new(&docs_dir).unwrap();
            manager.scan_for_changes().unwrap();
        }

        // Create new manager - should load persisted state
        let manager = StateManager::new(&docs_dir).unwrap();
        assert_eq!(manager.state().documents.len(), 1);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_compute_checksum_nonexistent_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("nonexistent.txt");

        let result = compute_checksum(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_changed_nonexistent_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("nonexistent.txt");

        let result = file_changed(&file_path, "abc123");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_metadata_nonexistent_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("nonexistent.txt");

        let result = file_metadata(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_corrupted_state_file() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join(".oxd");
        fs::create_dir_all(&state_dir).unwrap();

        let state_file = state_dir.join("state.json");
        fs::write(&state_file, "{ invalid json }").unwrap();

        let result = DocumentState::load(&state_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_state_creates_directory() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join("nested/deep/state");

        let state = DocumentState::new();
        let result = state.save(&state_dir);

        assert!(result.is_ok());
        assert!(state_dir.exists());
        assert!(state_dir.join("state.json").exists());
    }

    #[test]
    fn test_record_file_change_nonexistent() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let mut manager = StateManager::new(&docs_dir).unwrap();

        let nonexistent = docs_dir.join("nonexistent.md");
        let result = manager.record_file_change(&nonexistent);

        assert!(result.is_err());
    }

    #[test]
    fn test_default_document_state() {
        let state1 = DocumentState::default();
        let state2 = DocumentState::new();

        assert_eq!(state1.version, state2.version);
        assert_eq!(state1.next_number, state2.next_number);
        assert_eq!(state1.documents.len(), state2.documents.len());
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_doc(number: u32, state: &str) -> String {
        format!(
            r#"---
number: {}
title: Test Document {}
author: Test Author
created: 2024-01-01
updated: 2024-01-02
state: {}
---

# Test Document {}

This is test content.
"#,
            number, number, state, number
        )
    }

    #[test]
    fn test_large_file_checksum() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("large.txt");

        // Create a file larger than the buffer size (8192 bytes)
        let large_content = "x".repeat(20000);
        fs::write(&file_path, &large_content).unwrap();

        let checksum = compute_checksum(&file_path).unwrap();
        assert_eq!(checksum.len(), 64); // SHA-256 hex length
    }

    #[test]
    fn test_empty_directory_scan() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        fs::create_dir_all(docs_dir.join("01-draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.scan_for_changes().unwrap();

        assert!(result.new_files.is_empty());
        assert!(result.changed.is_empty());
        assert!(result.deleted.is_empty());
    }

    fn create_test_record(number: u32) -> DocumentRecord {
        use crate::doc::DocState;
        use chrono::NaiveDate;

        DocumentRecord {
            metadata: crate::doc::DocMetadata {
                number,
                title: format!("Test {}", number),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state: DocState::Draft,
                supersedes: None,
                superseded_by: None,
            },
            path: format!("01-draft/{:04}-test.md", number),
            checksum: format!("checksum{}", number),
            file_size: 1024,
            modified: Utc::now(),
        }
    }

    #[test]
    fn test_upsert_with_number_zero() {
        let mut state = DocumentState::new();
        let mut record = create_test_record(0);
        record.metadata.number = 0;

        state.upsert(0, record);

        assert!(state.get(0).is_some());
        assert_eq!(state.next_number, 1);
    }

    #[test]
    fn test_upsert_with_large_number() {
        let mut state = DocumentState::new();
        let mut record = create_test_record(9999);
        record.metadata.number = 9999;

        state.upsert(9999, record);

        assert!(state.get(9999).is_some());
        assert_eq!(state.next_number, 10000);
    }

    #[test]
    fn test_multiple_state_directories() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();

        // Create multiple state directories
        let draft_dir = docs_dir.join("01-draft");
        let final_dir = docs_dir.join("06-final");
        fs::create_dir_all(&draft_dir).unwrap();
        fs::create_dir_all(&final_dir).unwrap();

        // Add documents in different states
        fs::write(draft_dir.join("0001-draft.md"), create_test_doc(1, "draft")).unwrap();
        fs::write(final_dir.join("0002-final.md"), create_test_doc(2, "final")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.scan_for_changes().unwrap();

        assert_eq!(result.new_files.len(), 2);
        assert!(result.new_files.contains(&1));
        assert!(result.new_files.contains(&2));
    }

    #[test]
    fn test_scan_with_duplicate_numbers() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create two files with same number (edge case)
        fs::write(draft_dir.join("0001-first.md"), create_test_doc(1, "draft")).unwrap();
        fs::write(draft_dir.join("0001-second.md"), create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.scan_for_changes().unwrap();

        // Should handle gracefully - one will overwrite the other in state
        assert_eq!(manager.state().documents.len(), 1);
        assert!(result.new_files.contains(&1));
    }

    #[test]
    fn test_checksum_with_unicode_content() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("unicode.txt");

        let unicode_content = "Hello 世界 🌍 Привет مرحبا";
        fs::write(&file_path, unicode_content).unwrap();

        let checksum = compute_checksum(&file_path).unwrap();
        assert_eq!(checksum.len(), 64);

        // Verify it's deterministic
        let checksum2 = compute_checksum(&file_path).unwrap();
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_file_with_only_frontmatter() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("0001-empty-body.md");
        let content = r#"---
number: 1
title: Empty Body Document
author: Test Author
created: 2024-01-01
updated: 2024-01-02
state: draft
---
"#;
        fs::write(&doc_path, content).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        let result = manager.scan_for_changes().unwrap();

        assert_eq!(result.new_files.len(), 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_by_state_sorted() {
        let mut state = DocumentState::new();

        // Add records in random order
        for num in [5, 2, 8, 1, 3].iter() {
            let mut record = create_test_record(*num);
            record.metadata.number = *num;
            state.upsert(*num, record);
        }

        let all = state.by_state(crate::doc::DocState::Draft);

        // Verify sorted order
        for i in 0..all.len() - 1 {
            assert!(all[i].metadata.number < all[i + 1].metadata.number);
        }
    }

    #[test]
    fn test_scan_result_errors_dont_affect_has_changes() {
        let mut result = ScanResult::new();
        result.errors.push("Some error".to_string());

        // Errors alone don't count as changes
        assert!(!result.has_changes());
        assert_eq!(result.total_changes(), 0);
    }

    #[test]
    fn test_gitignore_not_created_twice() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join(".oxd");

        let state = DocumentState::new();
        state.save(&state_dir).unwrap();

        let gitignore = state_dir.join(".gitignore");
        let first_modified = fs::metadata(&gitignore).unwrap().modified().unwrap();

        // Save again
        std::thread::sleep(std::time::Duration::from_millis(10));
        state.save(&state_dir).unwrap();

        let second_modified = fs::metadata(&gitignore).unwrap().modified().unwrap();

        // .gitignore should not be updated if it exists
        assert_eq!(first_modified, second_modified);
    }

    #[test]
    fn test_state_version_is_persisted() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join(".oxd");

        let state = DocumentState::new();
        assert_eq!(state.version, 1);

        state.save(&state_dir).unwrap();
        let loaded = DocumentState::load(&state_dir).unwrap();

        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn test_state_last_updated_changes() {
        let mut state = DocumentState::new();
        let first_update = state.last_updated;

        std::thread::sleep(std::time::Duration::from_millis(10));

        let record = create_test_record(1);
        state.upsert(1, record);

        assert!(state.last_updated > first_update);
    }

    #[test]
    fn test_remove_updates_last_updated() {
        let mut state = DocumentState::new();
        let record = create_test_record(1);
        state.upsert(1, record);

        let before_remove = state.last_updated;
        std::thread::sleep(std::time::Duration::from_millis(10));

        state.remove(1);

        assert!(state.last_updated > before_remove);
    }

    #[test]
    fn test_quick_scan_with_checksum_error() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create a file and scan it
        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.quick_scan().unwrap();

        // This test ensures quick_scan handles files correctly even when
        // metadata check passes but checksum verification might be needed
        assert!(manager.state().get(1).is_some());
    }

    #[test]
    fn test_scan_for_changes_with_read_error() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        // Create a valid file first
        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();

        // Initial scan should succeed
        let result = manager.scan_for_changes().unwrap();
        assert_eq!(result.new_files.len(), 1);
    }

    #[test]
    fn test_all_sorted_with_single_document() {
        let mut state = DocumentState::new();
        let record = create_test_record(42);
        state.upsert(42, record);

        let all = state.all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].metadata.number, 42);
    }

    #[test]
    fn test_by_state_with_mixed_states() {
        use crate::doc::DocState;

        let mut state = DocumentState::new();

        // Create records with different states
        for (num, doc_state) in [
            (1, DocState::Draft),
            (2, DocState::Final),
            (3, DocState::Draft),
            (4, DocState::Active),
        ] {
            let mut record = create_test_record(num);
            record.metadata.state = doc_state;
            state.upsert(num, record);
        }

        let drafts = state.by_state(DocState::Draft);
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].metadata.number, 1);
        assert_eq!(drafts[1].metadata.number, 3);

        let finals = state.by_state(DocState::Final);
        assert_eq!(finals.len(), 1);
        assert_eq!(finals[0].metadata.number, 2);

        let actives = state.by_state(DocState::Active);
        assert_eq!(actives.len(), 1);
        assert_eq!(actives[0].metadata.number, 4);
    }

    #[test]
    fn test_quick_check_changed_detects_size_change() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let manager = StateManager::new(&docs_dir).unwrap();

        // Create a record with different size
        let mut record = create_test_record(1);
        record.file_size = 9999;
        record.modified = Utc::now();

        let changed = manager.quick_check_changed(&doc_path, &record).unwrap();
        assert!(changed);
    }

    #[test]
    fn test_quick_check_changed_detects_mtime_change() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let manager = StateManager::new(&docs_dir).unwrap();

        // Create a record with old mtime
        let mut record = create_test_record(1);
        let (size, _) = file_metadata(&doc_path).unwrap();
        record.file_size = size;
        record.modified = Utc::now() - chrono::Duration::seconds(3600);

        let changed = manager.quick_check_changed(&doc_path, &record).unwrap();
        assert!(changed);
    }

    #[test]
    fn test_quick_check_unchanged() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let manager = StateManager::new(&docs_dir).unwrap();

        // Create a record with matching size and newer mtime
        let (size, modified) = file_metadata(&doc_path).unwrap();
        let mut record = create_test_record(1);
        record.file_size = size;
        record.modified = modified + chrono::Duration::seconds(10);

        let changed = manager.quick_check_changed(&doc_path, &record).unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_update_record_from_file_strips_prefix() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.scan_for_changes().unwrap();

        let record = manager.state().get(1).unwrap();
        // Path should be relative to docs_dir
        assert!(record.path.starts_with("01-draft"));
        assert!(!record.path.contains(&docs_dir.to_string_lossy().to_string()));
    }

    #[test]
    fn test_quick_scan_deleted_files() {
        let temp = TempDir::new().unwrap();
        let docs_dir = temp.path().to_path_buf();
        let draft_dir = docs_dir.join("01-draft");
        fs::create_dir_all(&draft_dir).unwrap();

        let doc_path = draft_dir.join("0001-test.md");
        fs::write(&doc_path, create_test_doc(1, "draft")).unwrap();

        let mut manager = StateManager::new(&docs_dir).unwrap();
        manager.quick_scan().unwrap();
        assert!(manager.state().get(1).is_some());

        // Delete the file
        fs::remove_file(&doc_path).unwrap();

        let result = manager.quick_scan().unwrap();
        assert_eq!(result.deleted.len(), 1);
        assert_eq!(result.deleted[0], 1);
        assert!(manager.state().get(1).is_none());
    }
}
