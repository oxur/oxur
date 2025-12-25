//! Design document types and parsing

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocError {
    #[error("Invalid document format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),
}

/// Document state following the Zylisp pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocState {
    Draft,
    UnderReview,
    Final,
    Superseded,
}

impl DocState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocState::Draft => "Draft",
            DocState::UnderReview => "Under Review",
            DocState::Final => "Final",
            DocState::Superseded => "Superseded",
        }
    }

    pub fn directory(&self) -> &'static str {
        match self {
            DocState::Draft => "01-drafts",
            DocState::UnderReview => "02-under-review",
            DocState::Final => "03-final",
            DocState::Superseded => "04-superseded",
        }
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
}
