//! Metadata extraction from document content

use crate::doc::DocState;
use regex::Regex;

/// Extracted metadata from a document
#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub state_hint: Option<DocState>,
    pub has_frontmatter: bool,
    pub first_heading: Option<String>,
}

impl ExtractedMetadata {
    /// Extract metadata from document content
    pub fn from_content(content: &str) -> Self {
        let mut meta = ExtractedMetadata {
            title: None,
            author: None,
            state_hint: None,
            has_frontmatter: content.trim_start().starts_with("---\n"),
            first_heading: None,
        };

        // Skip frontmatter if present
        let content_start = if meta.has_frontmatter {
            if let Some(end) = content[4..].find("\n---\n") {
                end + 8
            } else if let Some(end) = content[4..].find("\n---") {
                end + 7
            } else {
                0
            }
        } else {
            0
        };

        let body = if content_start < content.len() { &content[content_start..] } else { content };

        // Extract first H1 heading
        for line in body.lines() {
            let trimmed = line.trim();
            if let Some(heading_text) = trimmed.strip_prefix("# ") {
                let heading = heading_text.trim().to_string();
                meta.first_heading = Some(heading.clone());
                if meta.title.is_none() {
                    meta.title = Some(heading);
                }
                break;
            }
        }

        // Look for author hints in content
        let author_patterns = [
            r"(?i)author:\s*(.+)",
            r"(?i)by\s+([A-Z][a-z]+\s+[A-Z][a-z]+)",
            r"(?i)written by\s+(.+)",
        ];

        for pattern in &author_patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(caps) = re.captures(body) {
                if let Some(author_match) = caps.get(1) {
                    let author = author_match.as_str().trim().to_string();
                    // Basic validation - should look like a name
                    if author.len() > 2 && author.len() < 100 {
                        meta.author = Some(author);
                        break;
                    }
                }
            }
        }

        // Detect state hints from content
        meta.state_hint = detect_state_hint(body);

        meta
    }
}

/// Detect likely state from document content
fn detect_state_hint(content: &str) -> Option<DocState> {
    let lower = content.to_lowercase();

    // Look for explicit state markers
    if lower.contains("work in progress") || lower.contains("wip") {
        return Some(DocState::Draft);
    }

    if lower.contains("ready for review") || lower.contains("please review") {
        return Some(DocState::UnderReview);
    }

    if lower.contains("approved") || lower.contains("accepted") {
        return Some(DocState::Accepted);
    }

    if lower.contains("implemented") || lower.contains("complete") {
        return Some(DocState::Final);
    }

    if lower.contains("rejected") || lower.contains("not approved") {
        return Some(DocState::Rejected);
    }

    if lower.contains("deferred") || lower.contains("postponed") {
        return Some(DocState::Deferred);
    }

    // Default to None if unclear (will default to Draft)
    None
}

/// Check if content looks like valid markdown
pub fn is_valid_markdown(content: &str) -> bool {
    // Basic checks
    if content.is_empty() {
        return false;
    }

    // Check for common markdown elements
    let has_text = content.len() > 10;

    // Check it's not binary (has mostly printable chars)
    let printable_ratio =
        content.chars().filter(|c| c.is_ascii_graphic() || c.is_whitespace()).count() as f64
            / content.len() as f64;

    has_text && printable_ratio > 0.9
}

/// Detect markdown issues
pub fn analyze_markdown(content: &str) -> Vec<String> {
    let mut issues = Vec::new();

    // Skip frontmatter for analysis
    let body = if content.trim_start().starts_with("---\n") {
        if let Some(end) = content[4..].find("\n---\n") {
            &content[end + 8..]
        } else {
            content
        }
    } else {
        content
    };

    // Check for multiple H1s (should only have one)
    let h1_count = body.lines().filter(|line| line.trim().starts_with("# ")).count();
    if h1_count == 0 {
        issues.push("No H1 heading found".to_string());
    } else if h1_count > 1 {
        issues.push(format!("Multiple H1 headings found ({})", h1_count));
    }

    // Check for inconsistent list markers
    let bullet_types: Vec<char> = body
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('+') {
                trimmed.chars().next()
            } else {
                None
            }
        })
        .collect();

    if !bullet_types.is_empty() {
        let first = bullet_types[0];
        if !bullet_types.iter().all(|&c| c == first) {
            issues.push("Inconsistent bullet point markers (-, *, +)".to_string());
        }
    }

    // Check for very long lines (> 120 chars)
    let long_lines = body.lines().filter(|line| line.len() > 120).count();
    if long_lines > 5 {
        issues.push(format!("Many long lines ({} lines > 120 chars)", long_lines));
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metadata_with_h1() {
        let content = "# My Title\n\nSome content here.";
        let meta = ExtractedMetadata::from_content(content);

        assert_eq!(meta.title, Some("My Title".to_string()));
        assert_eq!(meta.first_heading, Some("My Title".to_string()));
        assert!(!meta.has_frontmatter);
    }

    #[test]
    fn test_extract_metadata_no_heading() {
        let content = "Just some text with no heading.";
        let meta = ExtractedMetadata::from_content(content);

        assert_eq!(meta.title, None);
        assert_eq!(meta.first_heading, None);
    }

    #[test]
    fn test_extract_metadata_with_frontmatter() {
        let content = "---\ntitle: YAML Title\n---\n\n# Markdown Title\n\nContent.";
        let meta = ExtractedMetadata::from_content(content);

        assert!(meta.has_frontmatter);
        assert_eq!(meta.first_heading, Some("Markdown Title".to_string()));
    }

    #[test]
    fn test_extract_author_from_author_field() {
        let content = "# Title\n\nAuthor: John Doe\n\nContent here.";
        let meta = ExtractedMetadata::from_content(content);

        assert_eq!(meta.author, Some("John Doe".to_string()));
    }

    #[test]
    fn test_extract_author_from_by_pattern() {
        let content = "# Title\n\nBy Alice Smith\n\nContent here.";
        let meta = ExtractedMetadata::from_content(content);

        assert_eq!(meta.author, Some("Alice Smith".to_string()));
    }

    #[test]
    fn test_extract_author_written_by() {
        let content = "# Title\n\nWritten by Bob Jones\n\nContent.";
        let meta = ExtractedMetadata::from_content(content);

        assert_eq!(meta.author, Some("Bob Jones".to_string()));
    }

    #[test]
    fn test_extract_no_author() {
        let content = "# Title\n\nNo author information here.";
        let meta = ExtractedMetadata::from_content(content);

        assert_eq!(meta.author, None);
    }

    #[test]
    fn test_detect_state_hint_draft() {
        assert_eq!(detect_state_hint("This is work in progress"), Some(DocState::Draft));
        assert_eq!(detect_state_hint("WIP - still working"), Some(DocState::Draft));
    }

    #[test]
    fn test_detect_state_hint_under_review() {
        assert_eq!(detect_state_hint("Ready for review"), Some(DocState::UnderReview));
        assert_eq!(detect_state_hint("Please review this"), Some(DocState::UnderReview));
    }

    #[test]
    fn test_detect_state_hint_accepted() {
        assert_eq!(detect_state_hint("This has been approved"), Some(DocState::Accepted));
        assert_eq!(detect_state_hint("Accepted by the team"), Some(DocState::Accepted));
    }

    #[test]
    fn test_detect_state_hint_final() {
        assert_eq!(detect_state_hint("This is implemented"), Some(DocState::Final));
        assert_eq!(detect_state_hint("Work is complete"), Some(DocState::Final));
    }

    #[test]
    fn test_detect_state_hint_rejected() {
        assert_eq!(detect_state_hint("This was rejected"), Some(DocState::Rejected));
        assert_eq!(detect_state_hint("The proposal was rejected"), Some(DocState::Rejected));
    }

    #[test]
    fn test_detect_state_hint_deferred() {
        assert_eq!(detect_state_hint("This is deferred"), Some(DocState::Deferred));
        assert_eq!(detect_state_hint("Postponed for now"), Some(DocState::Deferred));
    }

    #[test]
    fn test_detect_state_hint_none() {
        assert_eq!(detect_state_hint("Regular content with no hints"), None);
    }

    #[test]
    fn test_is_valid_markdown_valid() {
        let content = "# Title\n\nThis is valid markdown content.";
        assert!(is_valid_markdown(content));
    }

    #[test]
    fn test_is_valid_markdown_empty() {
        assert!(!is_valid_markdown(""));
    }

    #[test]
    fn test_is_valid_markdown_too_short() {
        assert!(!is_valid_markdown("short"));
    }

    #[test]
    fn test_is_valid_markdown_mostly_binary() {
        let binary = "\x00\x01\x02\x03\x04\x05 some text \x06\x07\x08";
        assert!(!is_valid_markdown(binary));
    }

    #[test]
    fn test_analyze_markdown_no_h1() {
        let content = "## Subheading\n\nNo H1 here.";
        let issues = analyze_markdown(content);

        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("No H1 heading")));
    }

    #[test]
    fn test_analyze_markdown_multiple_h1() {
        let content = "# First\n\nContent\n\n# Second\n\nMore content";
        let issues = analyze_markdown(content);

        assert!(issues.iter().any(|i| i.contains("Multiple H1")));
    }

    #[test]
    fn test_analyze_markdown_inconsistent_bullets() {
        let content = "# Title\n\n- First\n* Second\n+ Third";
        let issues = analyze_markdown(content);

        assert!(issues.iter().any(|i| i.contains("Inconsistent bullet")));
    }

    #[test]
    fn test_analyze_markdown_long_lines() {
        let long_line = "a".repeat(150);
        let content = format!(
            "# Title\n\n{}\n{}\n{}\n{}\n{}\n{}",
            long_line, long_line, long_line, long_line, long_line, long_line
        );
        let issues = analyze_markdown(&content);

        assert!(issues.iter().any(|i| i.contains("long lines")));
    }

    #[test]
    fn test_analyze_markdown_perfect() {
        let content =
            "# Title\n\nThis is a well-formatted document.\n\n- Bullet 1\n- Bullet 2\n\nAll good!";
        let issues = analyze_markdown(content);

        // Should have no issues or only minor ones
        assert!(!issues.iter().any(|i| i.contains("No H1")));
        assert!(!issues.iter().any(|i| i.contains("Multiple H1")));
    }

    #[test]
    fn test_analyze_markdown_skips_frontmatter() {
        let content = "---\ntitle: Test\n---\n\n# Real Title\n\nContent";
        let issues = analyze_markdown(content);

        // Should not count frontmatter title as H1
        assert!(!issues.iter().any(|i| i.contains("Multiple H1")));
    }

    #[test]
    fn test_extract_metadata_case_insensitive_author() {
        let content = "# Title\n\nAUTHOR: Jane Doe\n\nContent.";
        let meta = ExtractedMetadata::from_content(content);

        assert_eq!(meta.author, Some("Jane Doe".to_string()));
    }

    #[test]
    fn test_extract_metadata_complete() {
        let content = "# My Document\n\nAuthor: Test Author\n\nThis is work in progress.";
        let meta = ExtractedMetadata::from_content(content);

        assert_eq!(meta.title, Some("My Document".to_string()));
        assert_eq!(meta.author, Some("Test Author".to_string()));
        assert_eq!(meta.state_hint, Some(DocState::Draft));
        assert!(!meta.has_frontmatter);
    }

    #[test]
    fn test_extract_metadata_with_incomplete_frontmatter() {
        let content = "---\ntitle: Test\n\n# Title\n\nContent";
        let meta = ExtractedMetadata::from_content(content);

        // Should handle malformed frontmatter gracefully
        assert!(meta.has_frontmatter);
    }
}
