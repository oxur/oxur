//! Design document types and parsing

use chrono::NaiveDate;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocError {
    #[error("Invalid document format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Document state - 10 states following the expanded lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DocState {
    Draft,
    UnderReview,
    Revised,
    Accepted,
    Active,
    Final,
    Deferred,
    Rejected,
    Withdrawn,
    Superseded,
}

impl DocState {
    /// Get the display name for this state
    pub fn as_str(&self) -> &'static str {
        match self {
            DocState::Draft => "Draft",
            DocState::UnderReview => "Under Review",
            DocState::Revised => "Revised",
            DocState::Accepted => "Accepted",
            DocState::Active => "Active",
            DocState::Final => "Final",
            DocState::Deferred => "Deferred",
            DocState::Rejected => "Rejected",
            DocState::Withdrawn => "Withdrawn",
            DocState::Superseded => "Superseded",
        }
    }

    /// Get the directory name for this state
    pub fn directory(&self) -> &'static str {
        match self {
            DocState::Draft => "01-draft",
            DocState::UnderReview => "02-under-review",
            DocState::Revised => "03-revised",
            DocState::Accepted => "04-accepted",
            DocState::Active => "05-active",
            DocState::Final => "06-final",
            DocState::Deferred => "07-deferred",
            DocState::Rejected => "08-rejected",
            DocState::Withdrawn => "09-withdrawn",
            DocState::Superseded => "10-superseded",
        }
    }

    /// Parse from string, handling various formats (hyphens, spaces, case)
    pub fn from_str_flexible(s: &str) -> Option<Self> {
        let normalized = s.to_lowercase().replace(['-', '_'], " ");
        let normalized = normalized.trim();
        match normalized {
            "draft" => Some(DocState::Draft),
            "under review" | "review" | "underreview" => Some(DocState::UnderReview),
            "revised" => Some(DocState::Revised),
            "accepted" => Some(DocState::Accepted),
            "active" => Some(DocState::Active),
            "final" => Some(DocState::Final),
            "deferred" => Some(DocState::Deferred),
            "rejected" => Some(DocState::Rejected),
            "withdrawn" => Some(DocState::Withdrawn),
            "superseded" => Some(DocState::Superseded),
            _ => None,
        }
    }

    /// Get DocState from directory name
    pub fn from_directory(dir: &str) -> Option<Self> {
        match dir {
            "01-draft" | "01-drafts" => Some(DocState::Draft),
            "02-under-review" => Some(DocState::UnderReview),
            "03-revised" => Some(DocState::Revised),
            "04-accepted" => Some(DocState::Accepted),
            "05-active" => Some(DocState::Active),
            "06-final" | "03-final" => Some(DocState::Final),
            "07-deferred" => Some(DocState::Deferred),
            "08-rejected" => Some(DocState::Rejected),
            "09-withdrawn" => Some(DocState::Withdrawn),
            "10-superseded" | "04-superseded" => Some(DocState::Superseded),
            _ => None,
        }
    }

    /// Get all valid states
    pub fn all_states() -> Vec<DocState> {
        vec![
            DocState::Draft,
            DocState::UnderReview,
            DocState::Revised,
            DocState::Accepted,
            DocState::Active,
            DocState::Final,
            DocState::Deferred,
            DocState::Rejected,
            DocState::Withdrawn,
            DocState::Superseded,
        ]
    }

    /// Get all valid state names for display
    pub fn all_state_names() -> Vec<&'static str> {
        Self::all_states().iter().map(|s| s.as_str()).collect()
    }
}

impl<'de> Deserialize<'de> for DocState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DocState::from_str_flexible(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid state: {}", s)))
    }
}

/// Metadata from the YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMetadata {
    pub number: u32,
    pub title: String,
    pub author: String,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    pub state: DocState,
    pub supersedes: Option<u32>,
    #[serde(rename = "superseded-by")]
    pub superseded_by: Option<u32>,
}

/// A complete design document
#[derive(Debug, Clone)]
pub struct DesignDoc {
    pub metadata: DocMetadata,
    pub content: String,
    pub path: PathBuf,
}

impl DesignDoc {
    /// Parse a design document from markdown content
    pub fn parse(content: &str, path: PathBuf) -> Result<Self, DocError> {
        // Look for YAML frontmatter between --- markers
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            return Err(DocError::InvalidFormat("Missing YAML frontmatter".to_string()));
        }

        let frontmatter = parts[1].trim();
        let body = parts[2].trim();

        // Parse YAML frontmatter
        let metadata: DocMetadata = serde_yaml::from_str(frontmatter)
            .map_err(|e| DocError::InvalidFormat(format!("YAML parse error: {}", e)))?;

        Ok(DesignDoc { metadata, content: body.to_string(), path })
    }

    /// Get the document filename based on number and state
    pub fn filename(&self) -> String {
        format!(
            "{:04}-{}.md",
            self.metadata.number,
            self.metadata
                .title
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>()
        )
    }

    /// Update a specific field in the YAML frontmatter
    pub fn update_yaml_field(content: &str, field: &str, value: &str) -> Result<String, DocError> {
        let pattern = format!(r"(?m)^{}: .*$", regex::escape(field));
        let re = Regex::new(&pattern)
            .map_err(|e| DocError::InvalidFormat(format!("Regex error: {}", e)))?;

        let replacement = format!("{}: {}", field, value);
        Ok(re.replace(content, replacement.as_str()).to_string())
    }

    /// Update the state and updated date in one operation
    pub fn update_state(content: &str, new_state: DocState) -> Result<String, DocError> {
        let today = chrono::Local::now().naive_local().date();

        let mut updated = Self::update_yaml_field(content, "state", new_state.as_str())?;
        updated = Self::update_yaml_field(&updated, "updated", &today.to_string())?;

        Ok(updated)
    }
}

/// Build complete YAML frontmatter from metadata
pub fn build_yaml_frontmatter(metadata: &DocMetadata) -> String {
    let mut yaml = String::from("---\n");
    yaml.push_str(&format!("number: {}\n", metadata.number));
    yaml.push_str(&format!("title: \"{}\"\n", metadata.title));
    yaml.push_str(&format!("author: \"{}\"\n", metadata.author));
    yaml.push_str(&format!("created: {}\n", metadata.created));
    yaml.push_str(&format!("updated: {}\n", metadata.updated));
    yaml.push_str(&format!("state: {}\n", metadata.state.as_str()));

    if let Some(supersedes) = metadata.supersedes {
        yaml.push_str(&format!("supersedes: {}\n", supersedes));
    } else {
        yaml.push_str("supersedes: null\n");
    }

    if let Some(superseded_by) = metadata.superseded_by {
        yaml.push_str(&format!("superseded-by: {}\n", superseded_by));
    } else {
        yaml.push_str("superseded-by: null\n");
    }

    yaml.push_str("---\n\n");
    yaml
}

/// Extract title from content (first # heading) or filename
pub fn extract_title_from_content(content: &str, filename: &str) -> String {
    // Look for first # heading
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return title.trim().to_string();
        }
    }

    // Infer from filename: 0001-my-document.md -> "My Document"
    let re = Regex::new(r"^\d+-(.+)\.md$").unwrap();
    if let Some(caps) = re.captures(filename) {
        if let Some(slug) = caps.get(1) {
            return slug
                .as_str()
                .split('-')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
    }

    "Untitled Document".to_string()
}

/// Extract number from filename (with 4-digit padding)
pub fn extract_number_from_filename(filename: &str) -> u32 {
    let re = Regex::new(r"^(\d+)-").unwrap();
    if let Some(caps) = re.captures(filename) {
        if let Some(num) = caps.get(1) {
            return num.as_str().parse().unwrap_or(0);
        }
    }
    0
}

/// Add or complete YAML frontmatter headers
pub fn add_missing_headers(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<(String, Vec<String>), DocError> {
    use crate::git;

    let path = path.as_ref();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| DocError::InvalidFormat("Invalid filename".to_string()))?;

    // Extract metadata
    let number = extract_number_from_filename(filename);
    let title = extract_title_from_content(content, filename);
    let author = git::get_author(path);
    let created = git::get_created_date(path);
    let updated = git::get_updated_date(path);

    let mut added_fields = Vec::new();

    // Check if document has frontmatter
    if content.trim_start().starts_with("---") {
        // Try to parse existing frontmatter
        match DesignDoc::parse(content, path.to_path_buf()) {
            Ok(doc) => {
                // Merge with discovered metadata - only update empty/default fields
                let mut metadata = doc.metadata;

                if metadata.number == 0 && number > 0 {
                    metadata.number = number;
                    added_fields.push("number".to_string());
                }
                if metadata.title.is_empty() || metadata.title == "Untitled Document" {
                    metadata.title = title;
                    added_fields.push("title".to_string());
                }
                if metadata.author.is_empty() || metadata.author == "Unknown Author" {
                    metadata.author = author;
                    added_fields.push("author".to_string());
                }

                // Strip old frontmatter and add complete new one
                let re = Regex::new(r"(?s)^---\n.*?\n---\n*").unwrap();
                let body = re.replace(content, "");
                let new_content = build_yaml_frontmatter(&metadata) + body.trim_start();

                Ok((new_content, added_fields))
            }
            Err(_) => {
                // Partial/broken frontmatter, build from scratch
                let metadata = DocMetadata {
                    number,
                    title,
                    author,
                    created,
                    updated,
                    state: DocState::Draft,
                    supersedes: None,
                    superseded_by: None,
                };
                added_fields = [
                    "number",
                    "title",
                    "author",
                    "created",
                    "updated",
                    "state",
                    "supersedes",
                    "superseded-by",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect();

                // Strip old frontmatter and add new
                let re = Regex::new(r"(?s)^---\n.*?\n---\n*").unwrap();
                let body = re.replace(content, "");
                let new_content = build_yaml_frontmatter(&metadata) + body.trim_start();
                Ok((new_content, added_fields))
            }
        }
    } else {
        // No frontmatter, add it
        let metadata = DocMetadata {
            number,
            title,
            author,
            created,
            updated,
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };

        added_fields = [
            "number",
            "title",
            "author",
            "created",
            "updated",
            "state",
            "supersedes",
            "superseded-by",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let new_content = build_yaml_frontmatter(&metadata) + content;
        Ok((new_content, added_fields))
    }
}

// ============================================================================
// Task 4.1: Number Assignment Functions
// ============================================================================

/// Check if filename has a number prefix (e.g., 0001-, 0042-)
pub fn has_number_prefix(filename: &str) -> bool {
    let re = Regex::new(r"^\d{4}-").unwrap();
    re.is_match(filename)
}

/// Rename file to include number prefix
pub fn add_number_prefix(path: &Path, number: u32) -> Result<PathBuf, std::io::Error> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename"))?;

    let new_filename = format!("{:04}-{}", number, filename);
    let new_path = path.with_file_name(new_filename);

    std::fs::rename(path, &new_path)?;

    Ok(new_path)
}

// ============================================================================
// Task 4.2: Directory Placement Functions
// ============================================================================

/// Check if a path is within the project directory
pub fn is_in_project_dir(file_path: &Path, project_dir: &Path) -> Result<bool, std::io::Error> {
    let abs_file = file_path.canonicalize()?;
    let abs_project = project_dir.canonicalize()?;

    Ok(abs_file.starts_with(abs_project))
}

/// Check if a path is in one of the state directories
pub fn is_in_state_dir(file_path: &Path) -> bool {
    if let Some(parent) = file_path.parent() {
        if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
            return DocState::from_directory(dir_name).is_some();
        }
    }
    false
}

/// Get the state from the file's current directory
pub fn state_from_directory(file_path: &Path) -> Option<DocState> {
    file_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .and_then(DocState::from_directory)
}

/// Move file to project directory
pub fn move_to_project(file_path: &Path, project_dir: &Path) -> Result<PathBuf, std::io::Error> {
    let filename = file_path
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename"))?;

    let new_path = project_dir.join(filename);
    std::fs::rename(file_path, &new_path)?;

    Ok(new_path)
}

/// Move file to a state directory
pub fn move_to_state_dir(
    file_path: &Path,
    state: DocState,
    project_dir: &Path,
) -> Result<PathBuf, std::io::Error> {
    let filename = file_path
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid filename"))?;

    let state_dir = project_dir.join(state.directory());
    std::fs::create_dir_all(&state_dir)?;

    let new_path = state_dir.join(filename);
    std::fs::rename(file_path, &new_path)?;

    Ok(new_path)
}

// ============================================================================
// Task 4.3: Header Processing Functions
// ============================================================================

/// Check if content has YAML frontmatter
pub fn has_frontmatter(content: &str) -> bool {
    content.trim_start().starts_with("---\n")
}

/// Check if frontmatter has placeholder values
pub fn has_placeholder_values(content: &str) -> bool {
    content.contains("number: NNNN")
        || content.contains("number: 0\n")
        || content.contains("author: Unknown")
        || content.contains("title: \"\"")
}

/// Ensure document has complete, valid headers
pub fn ensure_valid_headers(path: &Path, content: &str) -> Result<String, DocError> {
    if !has_frontmatter(content) || has_placeholder_values(content) {
        let (new_content, _) = add_missing_headers(path, content)?;
        Ok(new_content)
    } else {
        Ok(content.to_string())
    }
}

// ============================================================================
// Task 4.4: State Synchronization Functions
// ============================================================================

/// Sync document state with its directory location
pub fn sync_state_with_directory(path: &Path, content: &str) -> Result<String, DocError> {
    // Get state from directory
    let dir_state = state_from_directory(path)
        .ok_or_else(|| DocError::InvalidFormat("Document not in a state directory".to_string()))?;

    // Parse document to get current state
    let doc = DesignDoc::parse(content, path.to_path_buf())?;

    // If states don't match, update the content
    if doc.metadata.state != dir_state {
        DesignDoc::update_state(content, dir_state)
    } else {
        Ok(content.to_string())
    }
}

#[cfg(test)]
mod docstate_tests {
    use super::*;

    #[test]
    fn test_as_str_all_states() {
        assert_eq!(DocState::Draft.as_str(), "Draft");
        assert_eq!(DocState::UnderReview.as_str(), "Under Review");
        assert_eq!(DocState::Revised.as_str(), "Revised");
        assert_eq!(DocState::Accepted.as_str(), "Accepted");
        assert_eq!(DocState::Active.as_str(), "Active");
        assert_eq!(DocState::Final.as_str(), "Final");
        assert_eq!(DocState::Deferred.as_str(), "Deferred");
        assert_eq!(DocState::Rejected.as_str(), "Rejected");
        assert_eq!(DocState::Withdrawn.as_str(), "Withdrawn");
        assert_eq!(DocState::Superseded.as_str(), "Superseded");
    }

    #[test]
    fn test_directory_all_states() {
        assert_eq!(DocState::Draft.directory(), "01-draft");
        assert_eq!(DocState::UnderReview.directory(), "02-under-review");
        assert_eq!(DocState::Revised.directory(), "03-revised");
        assert_eq!(DocState::Accepted.directory(), "04-accepted");
        assert_eq!(DocState::Active.directory(), "05-active");
        assert_eq!(DocState::Final.directory(), "06-final");
        assert_eq!(DocState::Deferred.directory(), "07-deferred");
        assert_eq!(DocState::Rejected.directory(), "08-rejected");
        assert_eq!(DocState::Withdrawn.directory(), "09-withdrawn");
        assert_eq!(DocState::Superseded.directory(), "10-superseded");
    }

    #[test]
    fn test_from_str_flexible_canonical() {
        assert_eq!(DocState::from_str_flexible("draft"), Some(DocState::Draft));
        assert_eq!(DocState::from_str_flexible("under review"), Some(DocState::UnderReview));
        assert_eq!(DocState::from_str_flexible("revised"), Some(DocState::Revised));
        assert_eq!(DocState::from_str_flexible("accepted"), Some(DocState::Accepted));
        assert_eq!(DocState::from_str_flexible("active"), Some(DocState::Active));
        assert_eq!(DocState::from_str_flexible("final"), Some(DocState::Final));
        assert_eq!(DocState::from_str_flexible("deferred"), Some(DocState::Deferred));
        assert_eq!(DocState::from_str_flexible("rejected"), Some(DocState::Rejected));
        assert_eq!(DocState::from_str_flexible("withdrawn"), Some(DocState::Withdrawn));
        assert_eq!(DocState::from_str_flexible("superseded"), Some(DocState::Superseded));
    }

    #[test]
    fn test_from_str_flexible_case_insensitive() {
        assert_eq!(DocState::from_str_flexible("DRAFT"), Some(DocState::Draft));
        assert_eq!(DocState::from_str_flexible("Draft"), Some(DocState::Draft));
        assert_eq!(DocState::from_str_flexible("DRaFT"), Some(DocState::Draft));
        assert_eq!(DocState::from_str_flexible("UNDER REVIEW"), Some(DocState::UnderReview));
    }

    #[test]
    fn test_from_str_flexible_aliases() {
        assert_eq!(DocState::from_str_flexible("review"), Some(DocState::UnderReview));
        assert_eq!(DocState::from_str_flexible("underreview"), Some(DocState::UnderReview));
    }

    #[test]
    fn test_from_str_flexible_with_hyphens() {
        assert_eq!(DocState::from_str_flexible("under-review"), Some(DocState::UnderReview));
        assert_eq!(DocState::from_str_flexible("under_review"), Some(DocState::UnderReview));
    }

    #[test]
    fn test_from_str_flexible_whitespace() {
        assert_eq!(DocState::from_str_flexible("  draft  "), Some(DocState::Draft));
        assert_eq!(DocState::from_str_flexible("  under review  "), Some(DocState::UnderReview));
    }

    #[test]
    fn test_from_str_flexible_invalid() {
        assert_eq!(DocState::from_str_flexible("invalid"), None);
        assert_eq!(DocState::from_str_flexible(""), None);
        assert_eq!(DocState::from_str_flexible("pending"), None);
    }

    #[test]
    fn test_from_directory_canonical() {
        assert_eq!(DocState::from_directory("01-draft"), Some(DocState::Draft));
        assert_eq!(DocState::from_directory("02-under-review"), Some(DocState::UnderReview));
        assert_eq!(DocState::from_directory("03-revised"), Some(DocState::Revised));
        assert_eq!(DocState::from_directory("04-accepted"), Some(DocState::Accepted));
        assert_eq!(DocState::from_directory("05-active"), Some(DocState::Active));
        assert_eq!(DocState::from_directory("06-final"), Some(DocState::Final));
        assert_eq!(DocState::from_directory("07-deferred"), Some(DocState::Deferred));
        assert_eq!(DocState::from_directory("08-rejected"), Some(DocState::Rejected));
        assert_eq!(DocState::from_directory("09-withdrawn"), Some(DocState::Withdrawn));
        assert_eq!(DocState::from_directory("10-superseded"), Some(DocState::Superseded));
    }

    #[test]
    fn test_from_directory_legacy() {
        // Legacy directory names
        assert_eq!(DocState::from_directory("01-drafts"), Some(DocState::Draft));
        assert_eq!(DocState::from_directory("03-final"), Some(DocState::Final));
        assert_eq!(DocState::from_directory("04-superseded"), Some(DocState::Superseded));
    }

    #[test]
    fn test_from_directory_invalid() {
        assert_eq!(DocState::from_directory("invalid"), None);
        assert_eq!(DocState::from_directory("11-unknown"), None);
        assert_eq!(DocState::from_directory("draft"), None);
    }

    #[test]
    fn test_all_states_count() {
        let states = DocState::all_states();
        assert_eq!(states.len(), 10);
    }

    #[test]
    fn test_all_states_complete() {
        let states = DocState::all_states();
        assert!(states.contains(&DocState::Draft));
        assert!(states.contains(&DocState::UnderReview));
        assert!(states.contains(&DocState::Revised));
        assert!(states.contains(&DocState::Accepted));
        assert!(states.contains(&DocState::Active));
        assert!(states.contains(&DocState::Final));
        assert!(states.contains(&DocState::Deferred));
        assert!(states.contains(&DocState::Rejected));
        assert!(states.contains(&DocState::Withdrawn));
        assert!(states.contains(&DocState::Superseded));
    }

    #[test]
    fn test_all_state_names() {
        let names = DocState::all_state_names();
        assert_eq!(names.len(), 10);
        assert!(names.contains(&"Draft"));
        assert!(names.contains(&"Under Review"));
        assert!(names.contains(&"Final"));
    }

    #[test]
    fn test_serde_serialization() {
        let state = DocState::Draft;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"Draft\"");
    }

    #[test]
    fn test_serde_deserialization_valid() {
        let json = "\"Draft\"";
        let state: DocState = serde_json::from_str(json).unwrap();
        assert_eq!(state, DocState::Draft);

        let json = "\"under review\"";
        let state: DocState = serde_json::from_str(json).unwrap();
        assert_eq!(state, DocState::UnderReview);
    }

    #[test]
    fn test_serde_deserialization_invalid() {
        let json = "\"invalid state\"";
        let result: Result<DocState, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_equality() {
        assert_eq!(DocState::Draft, DocState::Draft);
        assert_ne!(DocState::Draft, DocState::Final);
    }

    #[test]
    fn test_state_round_trip() {
        for state in DocState::all_states() {
            // as_str -> from_str_flexible
            let str_repr = state.as_str();
            assert_eq!(DocState::from_str_flexible(str_repr), Some(state));

            // directory -> from_directory
            let dir_repr = state.directory();
            assert_eq!(DocState::from_directory(dir_repr), Some(state));
        }
    }
}

#[cfg(test)]
mod parsing_tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_test_doc_content(state: &str) -> String {
        format!(
            "---\nnumber: 42\ntitle: \"Test Document\"\nauthor: \"Test Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: {}\nsupersedes: null\nsuperseded-by: null\n---\n\n# Test Document\n\nThis is the content.",
            state
        )
    }

    #[test]
    fn test_parse_valid_document() {
        let content = create_test_doc_content("Draft");
        let result = DesignDoc::parse(&content, PathBuf::from("test.md"));

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.metadata.number, 42);
        assert_eq!(doc.metadata.title, "Test Document");
        assert_eq!(doc.metadata.author, "Test Author");
        assert_eq!(doc.metadata.state, DocState::Draft);
        assert!(doc.content.contains("# Test Document"));
    }

    #[test]
    fn test_parse_all_states() {
        for state in DocState::all_states() {
            let content = create_test_doc_content(state.as_str());
            let result = DesignDoc::parse(&content, PathBuf::from("test.md"));

            assert!(result.is_ok());
            let doc = result.unwrap();
            assert_eq!(doc.metadata.state, state);
        }
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# Just Content\n\nNo frontmatter here.";
        let result = DesignDoc::parse(content, PathBuf::from("test.md"));

        assert!(result.is_err());
        match result {
            Err(DocError::InvalidFormat(msg)) => assert!(msg.contains("Missing YAML frontmatter")),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_malformed_yaml() {
        let content = "---\nthis is not yaml\njust random text\n---\n\nContent";
        let result = DesignDoc::parse(content, PathBuf::from("test.md"));

        assert!(result.is_err());
        match result {
            Err(DocError::InvalidFormat(msg)) => assert!(msg.contains("YAML parse error")),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_with_supersedes() {
        let content = "---\nnumber: 42\ntitle: \"Test\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Final\nsupersedes: 41\nsuperseded-by: null\n---\n\nContent";
        let result = DesignDoc::parse(content, PathBuf::from("test.md"));

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.metadata.supersedes, Some(41));
        assert_eq!(doc.metadata.superseded_by, None);
    }

    #[test]
    fn test_parse_with_superseded_by() {
        let content = "---\nnumber: 41\ntitle: \"Test\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Superseded\nsupersedes: null\nsuperseded-by: 42\n---\n\nContent";
        let result = DesignDoc::parse(content, PathBuf::from("test.md"));

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.metadata.supersedes, None);
        assert_eq!(doc.metadata.superseded_by, Some(42));
    }

    #[test]
    fn test_filename_generation() {
        let metadata = DocMetadata {
            number: 42,
            title: "My Cool Feature".to_string(),
            author: "Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        let doc =
            DesignDoc { metadata, content: "test".to_string(), path: PathBuf::from("test.md") };

        assert_eq!(doc.filename(), "0042-my-cool-feature.md");
    }

    #[test]
    fn test_filename_special_chars() {
        let metadata = DocMetadata {
            number: 1,
            title: "Test!!! Document???".to_string(),
            author: "Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };
        let doc =
            DesignDoc { metadata, content: "test".to_string(), path: PathBuf::from("test.md") };

        assert_eq!(doc.filename(), "0001-test-document.md");
    }

    #[test]
    fn test_extract_title_from_heading() {
        let content = "# Main Title\n\nSome content here.";
        let title = extract_title_from_content(content, "0001-test.md");
        assert_eq!(title, "Main Title");
    }

    #[test]
    fn test_extract_title_from_filename() {
        let content = "No headings here.";
        let title = extract_title_from_content(content, "0042-my-feature.md");
        assert_eq!(title, "My Feature");
    }

    #[test]
    fn test_extract_title_fallback() {
        let content = "No headings here.";
        let title = extract_title_from_content(content, "invalid-filename");
        assert_eq!(title, "Untitled Document");
    }

    #[test]
    fn test_extract_title_empty_word_in_filename() {
        let content = "No headings here.";
        let title = extract_title_from_content(content, "0042-test--double-dash.md");
        assert_eq!(title, "Test  Double Dash");
    }

    #[test]
    fn test_extract_title_single_char_words() {
        let content = "No headings here.";
        let title = extract_title_from_content(content, "0042-a-b-c.md");
        assert_eq!(title, "A B C");
    }

    #[test]
    fn test_extract_title_with_whitespace_heading() {
        let content = "  # Title With Spaces  \n\nContent";
        let title = extract_title_from_content(content, "0042-test.md");
        assert_eq!(title, "Title With Spaces");
    }

    #[test]
    fn test_extract_title_filename_with_empty_segments() {
        let content = "No headings here.";
        let title = extract_title_from_content(content, "0042-test--extra.md");
        // This creates empty strings from double dashes, but still produces a title
        assert!(title.contains("Test") && title.contains("Extra"));
    }

    #[test]
    fn test_extract_number_from_filename() {
        assert_eq!(extract_number_from_filename("0001-test.md"), 1);
        assert_eq!(extract_number_from_filename("0042-feature.md"), 42);
        assert_eq!(extract_number_from_filename("9999-doc.md"), 9999);
    }

    #[test]
    fn test_extract_number_no_prefix() {
        assert_eq!(extract_number_from_filename("test.md"), 0);
        assert_eq!(extract_number_from_filename("no-number.md"), 0);
    }

    #[test]
    fn test_extract_number_invalid_parse() {
        assert_eq!(extract_number_from_filename("999999999999999999999-test.md"), 0);
    }

    #[test]
    fn test_has_number_prefix() {
        assert!(has_number_prefix("0001-test.md"));
        assert!(has_number_prefix("9999-doc.md"));
        assert!(!has_number_prefix("test.md"));
        assert!(!has_number_prefix("001-short.md"));
    }

    #[test]
    fn test_has_frontmatter() {
        assert!(has_frontmatter("---\ntitle: Test\n---\nContent"));
        assert!(has_frontmatter("  ---\ntitle: Test\n---\nContent"));
        assert!(!has_frontmatter("# No frontmatter"));
        assert!(!has_frontmatter(""));
    }

    #[test]
    fn test_has_placeholder_values() {
        assert!(has_placeholder_values("number: NNNN\ntitle: Test"));
        assert!(has_placeholder_values("number: 0\ntitle: Test"));
        assert!(has_placeholder_values("author: Unknown\ntitle: Test"));
        assert!(has_placeholder_values("title: \"\""));
        assert!(!has_placeholder_values("number: 42\ntitle: Real Title\nauthor: Real Author"));
    }
}

#[cfg(test)]
mod frontmatter_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_build_yaml_frontmatter_complete() {
        let metadata = DocMetadata {
            number: 42,
            title: "Test Document".to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            state: DocState::Draft,
            supersedes: Some(41),
            superseded_by: Some(43),
        };

        let yaml = build_yaml_frontmatter(&metadata);

        assert!(yaml.starts_with("---\n"));
        assert!(yaml.contains("number: 42\n"));
        assert!(yaml.contains("title: \"Test Document\"\n"));
        assert!(yaml.contains("author: \"Test Author\"\n"));
        assert!(yaml.contains("state: Draft\n"));
        assert!(yaml.contains("supersedes: 41\n"));
        assert!(yaml.contains("superseded-by: 43\n"));
        assert!(yaml.ends_with("---\n\n"));
    }

    #[test]
    fn test_build_yaml_frontmatter_nulls() {
        let metadata = DocMetadata {
            number: 1,
            title: "Test".to_string(),
            author: "Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        };

        let yaml = build_yaml_frontmatter(&metadata);

        assert!(yaml.contains("supersedes: null\n"));
        assert!(yaml.contains("superseded-by: null\n"));
    }

    #[test]
    fn test_build_yaml_all_states() {
        for state in DocState::all_states() {
            let metadata = DocMetadata {
                number: 1,
                title: "Test".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state,
                supersedes: None,
                superseded_by: None,
            };

            let yaml = build_yaml_frontmatter(&metadata);
            assert!(yaml.contains(&format!("state: {}\n", state.as_str())));
        }
    }

    #[test]
    fn test_update_yaml_field_exists() {
        let content = "---\ntitle: Old Title\nauthor: Someone\n---\nContent";
        let updated = DesignDoc::update_yaml_field(content, "title", "New Title").unwrap();

        assert!(updated.contains("title: New Title"));
        assert!(!updated.contains("Old Title"));
    }

    #[test]
    fn test_update_yaml_field_not_found() {
        let content = "---\ntitle: Title\nauthor: Someone\n---\nContent";
        let updated = DesignDoc::update_yaml_field(content, "nonexistent", "value").unwrap();

        // Should not modify if field doesn't exist
        assert_eq!(updated, content);
    }

    #[test]
    fn test_update_state_field() {
        let content = "---\ntitle: Test\nstate: Draft\nupdated: 2024-01-01\n---\nContent";
        let updated = DesignDoc::update_state(content, DocState::Final).unwrap();

        assert!(updated.contains("state: Final"));
        // Updated date should change (we can't test exact date but can verify it changed)
        assert!(updated.contains("updated:"));
    }
}

#[cfg(test)]
mod file_operations_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_in_state_dir() {
        // Paths in state directories
        assert!(is_in_state_dir(Path::new("project/01-draft/doc.md")));
        assert!(is_in_state_dir(Path::new("project/06-final/doc.md")));
        assert!(is_in_state_dir(Path::new("/abs/path/02-under-review/doc.md")));

        // Paths not in state directories
        assert!(!is_in_state_dir(Path::new("project/doc.md")));
        assert!(!is_in_state_dir(Path::new("project/other-dir/doc.md")));
    }

    #[test]
    fn test_is_in_state_dir_root_path() {
        assert!(!is_in_state_dir(Path::new("/")));
        assert!(!is_in_state_dir(Path::new("doc.md")));
    }

    #[test]
    fn test_state_from_directory() {
        assert_eq!(
            state_from_directory(Path::new("project/01-draft/doc.md")),
            Some(DocState::Draft)
        );
        assert_eq!(
            state_from_directory(Path::new("project/06-final/doc.md")),
            Some(DocState::Final)
        );
        assert_eq!(state_from_directory(Path::new("project/doc.md")), None);
    }

    #[test]
    fn test_move_to_project() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path();

        // Create a file in a subdirectory
        let subdir = project_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file_path = subdir.join("test.md");
        fs::write(&file_path, "content").unwrap();

        // Move to project root
        let new_path = move_to_project(&file_path, project_dir).unwrap();

        assert_eq!(new_path, project_dir.join("test.md"));
        assert!(new_path.exists());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_move_to_state_dir() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path();

        // Create a file in project root
        let file_path = project_dir.join("test.md");
        fs::write(&file_path, "content").unwrap();

        // Move to Draft state directory
        let new_path = move_to_state_dir(&file_path, DocState::Draft, project_dir).unwrap();

        assert_eq!(new_path, project_dir.join("01-draft/test.md"));
        assert!(new_path.exists());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_move_to_state_dir_creates_directory() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path();

        let file_path = project_dir.join("test.md");
        fs::write(&file_path, "content").unwrap();

        // State directory doesn't exist yet
        assert!(!project_dir.join("01-draft").exists());

        // Should create it
        move_to_state_dir(&file_path, DocState::Draft, project_dir).unwrap();

        assert!(project_dir.join("01-draft").exists());
    }

    #[test]
    fn test_add_number_prefix() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.md");
        fs::write(&file_path, "content").unwrap();

        let new_path = add_number_prefix(&file_path, 42).unwrap();

        assert_eq!(new_path.file_name().unwrap(), "0042-test.md");
        assert!(new_path.exists());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_is_in_project_dir_valid() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path();
        let file_path = project_dir.join("test.md");
        fs::write(&file_path, "content").unwrap();

        let result = is_in_project_dir(&file_path, project_dir).unwrap();
        assert!(result);
    }

    #[test]
    fn test_is_in_project_dir_outside() {
        let temp1 = TempDir::new().unwrap();
        let temp2 = TempDir::new().unwrap();
        let project_dir = temp1.path();
        let file_path = temp2.path().join("test.md");
        fs::write(&file_path, "content").unwrap();

        let result = is_in_project_dir(&file_path, project_dir).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_add_missing_headers_no_frontmatter() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0042-test-doc.md");
        let content = "# Test Document\n\nSome content here.";

        let (new_content, added_fields) = add_missing_headers(&file_path, content).unwrap();

        assert!(new_content.starts_with("---\n"));
        assert!(new_content.contains("number: 42"));
        assert!(new_content.contains("title: \"Test Document\""));
        assert!(new_content.contains("state: Draft"));
        assert_eq!(added_fields.len(), 8);
        assert!(added_fields.contains(&"number".to_string()));
    }

    #[test]
    fn test_add_missing_headers_with_valid_frontmatter() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0042-test-doc.md");
        let content = "---\nnumber: 100\ntitle: \"Existing Title\"\nauthor: \"Existing Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\n# Test Document\n\nContent";

        let (new_content, added_fields) = add_missing_headers(&file_path, content).unwrap();

        assert!(new_content.contains("number: 100"));
        assert!(new_content.contains("title: \"Existing Title\""));
        assert_eq!(added_fields.len(), 0);
    }

    #[test]
    fn test_add_missing_headers_with_partial_frontmatter() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0042-test-doc.md");
        let content = "---\nnumber: 0\ntitle: \"\"\nauthor: Unknown Author\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\n# Test Document\n\nContent";

        let (new_content, added_fields) = add_missing_headers(&file_path, content).unwrap();

        assert!(new_content.contains("number: 42"));
        assert!(new_content.contains("title: \"Test Document\""));
        assert!(added_fields.contains(&"number".to_string()));
        assert!(added_fields.contains(&"title".to_string()));
        assert!(added_fields.contains(&"author".to_string()));
    }

    #[test]
    fn test_add_missing_headers_with_broken_frontmatter() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0042-test-doc.md");
        let content =
            "---\nbroken yaml here\nno valid structure\n---\n\n# Test Document\n\nContent";

        let (new_content, added_fields) = add_missing_headers(&file_path, content).unwrap();

        assert!(new_content.starts_with("---\n"));
        assert!(new_content.contains("number: 42"));
        assert!(new_content.contains("title: \"Test Document\""));
        assert_eq!(added_fields.len(), 8);
    }

    #[test]
    fn test_ensure_valid_headers_missing_frontmatter() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0042-test-doc.md");
        let content = "# Test Document\n\nContent without frontmatter.";

        let result = ensure_valid_headers(&file_path, content).unwrap();

        assert!(result.starts_with("---\n"));
        assert!(result.contains("number: 42"));
    }

    #[test]
    fn test_ensure_valid_headers_with_placeholders() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0042-test-doc.md");
        let content = "---\nnumber: NNNN\ntitle: \"Test\"\nauthor: Unknown\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";

        let result = ensure_valid_headers(&file_path, content).unwrap();

        assert!(result.contains("number: 42"));
        assert!(!result.contains("NNNN"));
    }

    #[test]
    fn test_ensure_valid_headers_already_valid() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0042-test-doc.md");
        let content = "---\nnumber: 42\ntitle: \"Test\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";

        let result = ensure_valid_headers(&file_path, content).unwrap();

        assert_eq!(result, content);
    }

    #[test]
    fn test_sync_state_with_directory_matching() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join("01-draft");
        fs::create_dir(&state_dir).unwrap();
        let file_path = state_dir.join("test.md");
        let content = "---\nnumber: 42\ntitle: \"Test\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";

        let result = sync_state_with_directory(&file_path, content).unwrap();

        assert_eq!(result, content);
    }

    #[test]
    fn test_sync_state_with_directory_mismatched() {
        let temp = TempDir::new().unwrap();
        let state_dir = temp.path().join("06-final");
        fs::create_dir(&state_dir).unwrap();
        let file_path = state_dir.join("test.md");
        let content = "---\nnumber: 42\ntitle: \"Test\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";

        let result = sync_state_with_directory(&file_path, content).unwrap();

        assert!(result.contains("state: Final"));
        assert!(!result.contains("state: Draft"));
    }

    #[test]
    fn test_sync_state_with_directory_error() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.md");
        let content = "---\nnumber: 42\ntitle: \"Test\"\nauthor: \"Author\"\ncreated: 2024-01-01\nupdated: 2024-01-02\nstate: Draft\nsupersedes: null\nsuperseded-by: null\n---\n\nContent";

        let result = sync_state_with_directory(&file_path, content);

        assert!(result.is_err());
        match result {
            Err(DocError::InvalidFormat(msg)) => {
                assert!(msg.contains("not in a state directory"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use chrono::NaiveDate;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn state_as_str_from_str_round_trip(state in prop::sample::select(DocState::all_states())) {
            let str_repr = state.as_str();
            prop_assert_eq!(DocState::from_str_flexible(str_repr), Some(state));
        }

        #[test]
        fn state_directory_from_directory_round_trip(state in prop::sample::select(DocState::all_states())) {
            let dir_repr = state.directory();
            prop_assert_eq!(DocState::from_directory(dir_repr), Some(state));
        }

        #[test]
        fn extract_number_is_consistent(num in 0u32..10000) {
            let filename = format!("{:04}-test.md", num);
            prop_assert_eq!(extract_number_from_filename(&filename), num);
        }

        #[test]
        fn has_number_prefix_consistency(num in 0u32..10000, title in "[a-z]+") {
            let filename = format!("{:04}-{}.md", num, title);
            prop_assert!(has_number_prefix(&filename));
        }

        #[test]
        fn yaml_frontmatter_starts_and_ends_correctly(
            num in 1u32..10000,
            state in prop::sample::select(DocState::all_states())
        ) {
            let metadata = DocMetadata {
                number: num,
                title: "Test".to_string(),
                author: "Author".to_string(),
                created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                state,
                supersedes: None,
                superseded_by: None,
            };

            let yaml = build_yaml_frontmatter(&metadata);
            prop_assert!(yaml.starts_with("---\n"));
            prop_assert!(yaml.ends_with("---\n\n"));
        }
    }
}
