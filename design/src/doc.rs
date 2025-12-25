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
