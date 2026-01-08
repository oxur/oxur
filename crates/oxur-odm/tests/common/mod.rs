// Test infrastructure and utilities for design crate tests

use chrono::NaiveDate;
use design::doc::{DocMetadata, DocState, DesignDoc};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Builder for creating test DocMetadata
pub struct DocMetadataBuilder {
    number: u32,
    title: String,
    author: String,
    created: NaiveDate,
    updated: NaiveDate,
    state: DocState,
    supersedes: Option<u32>,
    superseded_by: Option<u32>,
}

impl DocMetadataBuilder {
    pub fn new() -> Self {
        Self {
            number: 1,
            title: "Test Document".to_string(),
            author: "Test Author".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            state: DocState::Draft,
            supersedes: None,
            superseded_by: None,
        }
    }

    pub fn number(mut self, number: u32) -> Self {
        self.number = number;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    pub fn created(mut self, date: NaiveDate) -> Self {
        self.created = date;
        self
    }

    pub fn updated(mut self, date: NaiveDate) -> Self {
        self.updated = date;
        self
    }

    pub fn state(mut self, state: DocState) -> Self {
        self.state = state;
        self
    }

    pub fn supersedes(mut self, number: u32) -> Self {
        self.supersedes = Some(number);
        self
    }

    pub fn superseded_by(mut self, number: u32) -> Self {
        self.superseded_by = Some(number);
        self
    }

    pub fn build(self) -> DocMetadata {
        DocMetadata {
            number: self.number,
            title: self.title,
            author: self.author,
            created: self.created,
            updated: self.updated,
            state: self.state,
            supersedes: self.supersedes,
            superseded_by: self.superseded_by,
        }
    }
}

impl Default for DocMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a test document with the given parameters
pub fn create_test_doc(number: u32, title: &str, state: DocState) -> DocMetadata {
    DocMetadataBuilder::new().number(number).title(title).state(state).build()
}

/// Create YAML frontmatter from metadata
pub fn create_test_frontmatter(metadata: &DocMetadata) -> String {
    let mut yaml = format!(
        "number: {}\ntitle: \"{}\"\nauthor: \"{}\"\ncreated: {}\nupdated: {}\nstate: {}\n",
        metadata.number,
        metadata.title,
        metadata.author,
        metadata.created,
        metadata.updated,
        metadata.state.as_str()
    );

    if let Some(supersedes) = metadata.supersedes {
        yaml.push_str(&format!("supersedes: {}\n", supersedes));
    }

    if let Some(superseded_by) = metadata.superseded_by {
        yaml.push_str(&format!("superseded-by: {}\n", superseded_by));
    }

    yaml
}

/// Create a complete markdown document with frontmatter and content
pub fn create_test_markdown(metadata: &DocMetadata, body: &str) -> String {
    format!("---\n{}---\n\n{}", create_test_frontmatter(metadata), body)
}

/// Sample markdown content for testing
pub fn sample_markdown_content() -> &'static str {
    "# Test Document Title\n\n\
     This is a test document.\n\n\
     ## Background\n\n\
     Some background information.\n\n\
     ## Proposal\n\n\
     The proposed solution.\n"
}

/// Sample markdown with various issues for normalization testing
pub fn sample_messy_markdown() -> &'static str {
    "# Test Title\n\n\n\n\
     Some text here.\n\n\n\n\n\
     * bullet 1\n\
        * nested bullet\n\
     - bullet 2\n\
     + bullet 3\n\n\n\n\
     ## Heading\n\n\
     More text.\n"
}

/// Assert that a string contains valid YAML frontmatter
pub fn assert_valid_yaml(content: &str) {
    assert!(content.starts_with("---\n"), "Content should start with YAML frontmatter marker");
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    assert_eq!(parts.len(), 3, "Content should have YAML frontmatter between --- markers");

    let frontmatter = parts[1].trim();
    let _: serde_yaml::Value =
        serde_yaml::from_str(frontmatter).expect("Frontmatter should be valid YAML");
}

/// Test project structure builder
pub struct TestProject {
    pub root: TempDir,
}

impl TestProject {
    /// Create a new test project with temporary directory
    pub fn new() -> std::io::Result<Self> {
        let root = TempDir::new()?;
        Ok(Self { root })
    }

    /// Get the root path
    pub fn root_path(&self) -> &Path {
        self.root.path()
    }

    /// Create the .oxd state directory
    pub fn create_state_dir(&self) -> std::io::Result<PathBuf> {
        let state_dir = self.root.path().join(".oxd");
        fs::create_dir_all(&state_dir)?;
        Ok(state_dir)
    }

    /// Create all 10 state directories
    pub fn create_state_dirs(&self) -> std::io::Result<()> {
        let states = [
            "01-draft",
            "02-under-review",
            "03-revised",
            "04-accepted",
            "05-active",
            "06-final",
            "07-deferred",
            "08-rejected",
            "09-withdrawn",
            "10-superseded",
        ];

        for state in &states {
            let state_dir = self.root.path().join(state);
            fs::create_dir_all(state_dir)?;
        }

        Ok(())
    }

    /// Create a test document file in a state directory
    pub fn create_doc_file(
        &self,
        state_dir: &str,
        filename: &str,
        metadata: &DocMetadata,
        body: &str,
    ) -> std::io::Result<PathBuf> {
        let state_path = self.root.path().join(state_dir);
        fs::create_dir_all(&state_path)?;

        let doc_path = state_path.join(filename);
        let content = create_test_markdown(metadata, body);
        fs::write(&doc_path, content)?;

        Ok(doc_path)
    }

    /// Create a simple markdown file without frontmatter
    pub fn create_simple_file(&self, path: &str, content: &str) -> std::io::Result<PathBuf> {
        let file_path = self.root.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, content)?;
        Ok(file_path)
    }

    /// Read file content
    pub fn read_file(&self, path: &str) -> std::io::Result<String> {
        let file_path = self.root.path().join(path);
        fs::read_to_string(file_path)
    }

    /// Check if file exists
    pub fn file_exists(&self, path: &str) -> bool {
        self.root.path().join(path).exists()
    }

    /// Create an index file
    pub fn create_index(&self, content: &str) -> std::io::Result<PathBuf> {
        let index_path = self.root.path().join("INDEX.md");
        fs::write(&index_path, content)?;
        Ok(index_path)
    }
}

impl Default for TestProject {
    fn default() -> Self {
        Self::new().expect("Failed to create test project")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_builder() {
        let metadata = DocMetadataBuilder::new()
            .number(42)
            .title("Custom Title")
            .author("Custom Author")
            .state(DocState::Accepted)
            .build();

        assert_eq!(metadata.number, 42);
        assert_eq!(metadata.title, "Custom Title");
        assert_eq!(metadata.author, "Custom Author");
        assert_eq!(metadata.state, DocState::Accepted);
    }

    #[test]
    fn test_create_test_frontmatter() {
        let metadata = create_test_doc(1, "Test Doc", DocState::Draft);
        let frontmatter = create_test_frontmatter(&metadata);

        assert!(frontmatter.contains("number: 1"));
        assert!(frontmatter.contains("title: \"Test Doc\""));
        assert!(frontmatter.contains("state: Draft"));
    }

    #[test]
    fn test_create_test_markdown() {
        let metadata = create_test_doc(1, "Test", DocState::Draft);
        let markdown = create_test_markdown(&metadata, "# Test\n\nContent here.");

        assert_valid_yaml(&markdown);
        assert!(markdown.contains("# Test"));
        assert!(markdown.contains("Content here."));
    }

    #[test]
    fn test_test_project_creation() {
        let project = TestProject::new().unwrap();
        assert!(project.root_path().exists());
    }

    #[test]
    fn test_create_state_dirs() {
        let project = TestProject::new().unwrap();
        project.create_state_dirs().unwrap();

        assert!(project.root_path().join("01-draft").exists());
        assert!(project.root_path().join("06-final").exists());
        assert!(project.root_path().join("10-superseded").exists());
    }

    #[test]
    fn test_create_doc_file() {
        let project = TestProject::new().unwrap();
        let metadata = create_test_doc(1, "Test", DocState::Draft);

        let path = project.create_doc_file("01-draft", "0001-test.md", &metadata, "Content").unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert_valid_yaml(&content);
        assert!(content.contains("Content"));
    }
}
