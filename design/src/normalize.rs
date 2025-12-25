//! Markdown content normalization

use regex::Regex;

/// Normalize markdown content
pub fn normalize_markdown(content: &str) -> String {
    let mut normalized = content.to_string();

    // Standardize bullet points to use '-'
    normalized = standardize_bullets(&normalized);

    // Ensure single blank line between sections
    normalized = normalize_spacing(&normalized);

    // Ensure headings have blank line before/after
    normalized = normalize_headings(&normalized);

    // Trim trailing whitespace from lines
    normalized = trim_line_whitespace(&normalized);

    // Ensure file ends with single newline
    normalized = normalized.trim_end().to_string() + "\n";

    normalized
}

fn standardize_bullets(content: &str) -> String {
    let re = Regex::new(r"^(\s*)[\*\+]\s").unwrap();
    content
        .lines()
        .map(|line| re.replace(line, "${1}- ").to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_spacing(content: &str) -> String {
    // Replace 3+ newlines with 2 newlines
    let re = Regex::new(r"\n{3,}").unwrap();
    re.replace_all(content, "\n\n").to_string()
}

fn normalize_headings(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let is_heading = line.trim().starts_with('#');

        if is_heading {
            // Add blank line before heading (unless first line or already blank)
            if i > 0 && !result.is_empty() {
                let last = result.last().unwrap_or(&"");
                if !last.is_empty() {
                    result.push("");
                }
            }

            result.push(line);

            // Check if next line needs a blank line after heading
            if i < lines.len() - 1 {
                let next = lines[i + 1];
                if !next.trim().is_empty() && !next.trim().starts_with('#') {
                    result.push("");
                }
            }
        } else {
            result.push(line);
        }
    }

    result.join("\n")
}

fn trim_line_whitespace(content: &str) -> String {
    content.lines().map(|line| line.trim_end()).collect::<Vec<_>>().join("\n")
}

/// Strip incomplete or malformed YAML frontmatter
pub fn strip_bad_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---\n") {
        return content.to_string();
    }

    // Find closing ---
    if let Some(end_pos) = trimmed[4..].find("\n---\n") {
        let frontmatter = &trimmed[4..end_pos + 4];

        // Check if frontmatter looks valid (has : on most lines)
        let lines: Vec<&str> = frontmatter.lines().filter(|l| !l.trim().is_empty()).collect();
        let valid_lines = lines.iter().filter(|line| line.contains(':')).count();

        if !lines.is_empty() && valid_lines < lines.len() / 2 {
            // Looks malformed, strip it
            return trimmed[end_pos + 8..].to_string();
        }
    } else if let Some(end_pos) = trimmed[4..].find("\n---") {
        // Handle case where --- is at end of file
        let frontmatter = &trimmed[4..end_pos + 4];
        let lines: Vec<&str> = frontmatter.lines().filter(|l| !l.trim().is_empty()).collect();
        let valid_lines = lines.iter().filter(|line| line.contains(':')).count();

        if !lines.is_empty() && valid_lines < lines.len() / 2 {
            return trimmed[end_pos + 7..].to_string();
        }
    }

    content.to_string()
}

/// Strip any existing frontmatter (valid or not)
pub fn strip_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---\n") {
        return content.to_string();
    }

    // Find closing ---
    if let Some(end_pos) = trimmed[4..].find("\n---\n") {
        return trimmed[end_pos + 8..].to_string();
    } else if let Some(end_pos) = trimmed[4..].find("\n---") {
        return trimmed[end_pos + 7..].to_string();
    }

    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standardize_bullets() {
        let input = "* item 1\n+ item 2\n- item 3";
        let expected = "- item 1\n- item 2\n- item 3";
        assert_eq!(standardize_bullets(input), expected);
    }

    #[test]
    fn test_normalize_spacing() {
        let input = "line 1\n\n\n\nline 2";
        let expected = "line 1\n\nline 2";
        assert_eq!(normalize_spacing(input), expected);
    }

    #[test]
    fn test_trim_whitespace() {
        let input = "line 1   \nline 2  ";
        let expected = "line 1\nline 2";
        assert_eq!(trim_line_whitespace(input), expected);
    }

    #[test]
    fn test_strip_frontmatter() {
        let input = "---\ntitle: Test\n---\n\n# Content";
        let expected = "\n\n# Content";
        assert_eq!(strip_frontmatter(input), expected);
    }
}
