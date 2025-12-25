//! Filename sanitization and normalization

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Sanitize a filename to be filesystem-friendly
pub fn sanitize_filename(name: &str) -> String {
    // Remove extension if present
    let stem = if let Some(pos) = name.rfind('.') { &name[..pos] } else { name };

    // Remove number prefix if present (we'll add our own)
    let re = Regex::new(r"^\d{4}-").unwrap();
    let without_number = re.replace(stem, "");

    // Normalize unicode to NFD form, then collect only ASCII chars
    let normalized: String = without_number.nfd().filter(|c| c.is_ascii()).collect();

    // Convert to lowercase
    let mut result = normalized.to_lowercase();

    // Replace spaces and underscores with hyphens
    result = result.replace(' ', "-");
    result = result.replace('_', "-");

    // Remove special characters (keep alphanumeric and hyphens)
    let re = Regex::new(r"[^a-z0-9-]").unwrap();
    result = re.replace_all(&result, "").to_string();

    // Collapse multiple hyphens
    let re = Regex::new(r"-+").unwrap();
    result = re.replace_all(&result, "-").to_string();

    // Trim hyphens from start and end
    result = result.trim_matches('-').to_string();

    // Enforce maximum length (filesystem limit is usually 255, use 100 for safety)
    if result.len() > 100 {
        result.truncate(100);
        result = result.trim_matches('-').to_string();
    }

    // Ensure not empty
    if result.is_empty() {
        result = "untitled".to_string();
    }

    result
}

/// Build filename with number prefix
pub fn build_filename(number: u32, title: &str) -> String {
    let sanitized = sanitize_filename(title);
    format!("{:04}-{}.md", number, sanitized)
}

/// Extract title-like string from filename
pub fn filename_to_title(filename: &str) -> String {
    // Remove extension
    let stem = if let Some(pos) = filename.rfind('.') { &filename[..pos] } else { filename };

    // Remove number prefix
    let re = Regex::new(r"^\d{4}-").unwrap();
    let without_number = re.replace(stem, "");

    // Replace hyphens and underscores with spaces
    let with_spaces = without_number.replace(['-', '_'], " ");

    // Title case each word
    with_spaces
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_basic() {
        assert_eq!(sanitize_filename("My Cool Feature"), "my-cool-feature");
    }

    #[test]
    fn test_sanitize_special_chars() {
        assert_eq!(sanitize_filename("Feature!!!"), "feature");
        assert_eq!(sanitize_filename("my_feature_name"), "my-feature-name");
    }

    #[test]
    fn test_sanitize_unicode() {
        assert_eq!(sanitize_filename("Café"), "cafe");
        assert_eq!(sanitize_filename("naïve"), "naive");
    }

    #[test]
    fn test_sanitize_multiple_hyphens() {
        assert_eq!(sanitize_filename("my---feature"), "my-feature");
    }

    #[test]
    fn test_sanitize_empty() {
        assert_eq!(sanitize_filename("!!!"), "untitled");
    }

    #[test]
    fn test_build_filename() {
        assert_eq!(build_filename(12, "My Cool Feature"), "0012-my-cool-feature.md");
    }

    #[test]
    fn test_filename_to_title() {
        assert_eq!(filename_to_title("0001-my-feature.md"), "My Feature");
        assert_eq!(filename_to_title("my_cool_thing.md"), "My Cool Thing");
    }
}
