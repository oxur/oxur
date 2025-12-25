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
