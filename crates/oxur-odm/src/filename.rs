//! Filename sanitization and normalization

use regex::Regex;
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

/// Sanitize a filename to be filesystem-friendly
///
/// This function sanitizes characters only — it does NOT strip file extensions.
/// Callers that need extension removal should use `Path::file_stem()` first.
pub fn sanitize_filename(name: &str) -> String {
    // Remove number prefix if present (we'll add our own)
    let re = Regex::new(r"^\d{4}-").unwrap();
    let without_number = re.replace(name, "");

    // Normalize unicode to NFD form, then collect only ASCII chars
    let normalized: String = without_number.nfd().filter(|c| c.is_ascii()).collect();

    // Convert to lowercase
    let mut result = normalized.to_lowercase();

    // Replace spaces and underscores with hyphens
    result = result.replace(' ', "-");
    result = result.replace('_', "-");

    // Remove special characters (keep alphanumeric, hyphens, and dots)
    let re = Regex::new(r"[^a-z0-9.\-]").unwrap();
    result = re.replace_all(&result, "").to_string();

    // Collapse multiple hyphens and multiple dots
    let re = Regex::new(r"-+").unwrap();
    result = re.replace_all(&result, "-").to_string();
    let re = Regex::new(r"\.+").unwrap();
    result = re.replace_all(&result, ".").to_string();

    // Trim hyphens and dots from start and end
    result = result.trim_matches(&['-', '.'][..]).to_string();

    // Enforce maximum length (filesystem limit is usually 255, use 100 for safety)
    if result.len() > 100 {
        result.truncate(100);
        result = result.trim_matches(&['-', '.'][..]).to_string();
    }

    // Ensure not empty
    if result.is_empty() {
        result = "untitled".to_string();
    }

    result
}

/// Slugify a string (alias for sanitize_filename)
pub fn slugify(s: &str) -> String {
    sanitize_filename(s)
}

/// Build filename with number prefix
pub fn build_filename(number: u32, title: &str) -> String {
    let sanitized = sanitize_filename(title);
    // Strip trailing .md to avoid double extension (e.g., when a title contains "native-image.md")
    let stem = sanitized.strip_suffix(".md").unwrap_or(&sanitized);
    format!("{:04}-{}.md", number, stem)
}

/// Extract title-like string from filename
pub fn filename_to_title(filename: &str) -> String {
    // Remove final extension (using stdlib to correctly handle multi-dot names)
    let stem = Path::new(filename).file_stem().and_then(|s| s.to_str()).unwrap_or(filename);

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

    // Edge case tests
    #[test]
    fn test_sanitize_very_long_filename() {
        let long_name = "a".repeat(200);
        let result = sanitize_filename(&long_name);
        assert!(result.len() <= 100, "Filename should be truncated to 100 chars");
    }

    #[test]
    fn test_sanitize_exactly_100_chars() {
        let name = "a".repeat(100);
        let result = sanitize_filename(&name);
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_sanitize_all_special_characters() {
        assert_eq!(sanitize_filename("!@#$%^&*()"), "untitled");
        assert_eq!(sanitize_filename("[]{}|\\"), "untitled");
        assert_eq!(sanitize_filename("<>?/~`"), "untitled");
    }

    #[test]
    fn test_sanitize_unicode_emojis() {
        assert_eq!(sanitize_filename("😀 Happy File 😀"), "happy-file");
        assert_eq!(sanitize_filename("🚀 Rocket 🌟"), "rocket");
    }

    #[test]
    fn test_sanitize_unicode_rtl_text() {
        // Arabic text should be filtered out (not ASCII)
        assert_eq!(sanitize_filename("مرحبا"), "untitled");
        // Mixed RTL and ASCII
        assert_eq!(sanitize_filename("hello مرحبا world"), "hello-world");
    }

    #[test]
    fn test_sanitize_mixed_case() {
        assert_eq!(sanitize_filename("MyMixedCaseFileName"), "mymixedcasefilename");
        assert_eq!(sanitize_filename("CamelCase"), "camelcase");
    }

    #[test]
    fn test_sanitize_leading_trailing_hyphens() {
        assert_eq!(sanitize_filename("---my-file---"), "my-file");
        assert_eq!(sanitize_filename("--file--"), "file");
    }

    #[test]
    fn test_sanitize_numbers_only() {
        assert_eq!(sanitize_filename("123456"), "123456");
        assert_eq!(sanitize_filename("0000"), "0000");
    }

    #[test]
    fn test_sanitize_removes_number_prefix() {
        assert_eq!(sanitize_filename("0001-existing-prefix"), "existing-prefix");
        assert_eq!(sanitize_filename("9999-test"), "test");
    }

    #[test]
    fn test_sanitize_preserves_dots() {
        // sanitize_filename does NOT strip extensions — dots are preserved
        assert_eq!(sanitize_filename("my-file.md"), "my-file.md");
        assert_eq!(sanitize_filename("test.txt"), "test.txt");
        assert_eq!(sanitize_filename("file.tar.gz"), "file.tar.gz");
    }

    #[test]
    fn test_sanitize_preserves_version_numbers() {
        assert_eq!(sanitize_filename("Phase 2.3 Title"), "phase-2.3-title");
        assert_eq!(sanitize_filename("milestone-2.3-feature"), "milestone-2.3-feature");
    }

    #[test]
    fn test_sanitize_with_non_extension_dot() {
        // Dot followed by digits
        assert_eq!(sanitize_filename("version-1.0"), "version-1.0");
        // Dot followed by mixed content
        assert_eq!(sanitize_filename("phase-2.3-google-drive"), "phase-2.3-google-drive");
    }

    #[test]
    fn test_build_filename_with_dotted_title() {
        assert_eq!(
            build_filename(17, "Phase 2 Milestone 2.3 Google Drive Ingestion"),
            "0017-phase-2-milestone-2.3-google-drive-ingestion.md"
        );
    }

    #[test]
    fn test_filename_to_title_with_dots() {
        assert_eq!(
            filename_to_title("0017-phase-2-milestone-2.3-ingestion.md"),
            "Phase 2 Milestone 2.3 Ingestion"
        );
    }

    #[test]
    fn test_sanitize_whitespace_variations() {
        assert_eq!(sanitize_filename("  spaces  everywhere  "), "spaces-everywhere");
        assert_eq!(sanitize_filename("tab\tseparated"), "tabseparated");
        assert_eq!(sanitize_filename("new\nline"), "newline");
    }

    #[test]
    fn test_build_filename_strips_md_extension_from_title() {
        // Title containing ".md" (e.g., from a heading like "Guide — native-image.md")
        assert_eq!(
            build_filename(5, "Claude Code Prompt: Guide #04 — `native-image.md`"),
            "0005-claude-code-prompt-guide-04-native-image.md"
        );
        // Direct .md suffix
        assert_eq!(build_filename(1, "my-file.md"), "0001-my-file.md");
        // No double .md
        assert!(!build_filename(1, "test.md").ends_with(".md.md"));
    }

    #[test]
    fn test_build_filename_preserves_non_md_dots() {
        // Version numbers in titles should still work
        assert_eq!(build_filename(17, "Phase 2.3 Feature"), "0017-phase-2.3-feature.md");
    }

    #[test]
    fn test_build_filename_edge_numbers() {
        assert_eq!(build_filename(0, "test"), "0000-test.md");
        assert_eq!(build_filename(1, "test"), "0001-test.md");
        assert_eq!(build_filename(9999, "test"), "9999-test.md");
    }

    #[test]
    fn test_build_filename_empty_title() {
        assert_eq!(build_filename(42, "!!!"), "0042-untitled.md");
        assert_eq!(build_filename(42, ""), "0042-untitled.md");
    }

    #[test]
    fn test_filename_to_title_no_prefix() {
        assert_eq!(filename_to_title("no-prefix-here.md"), "No Prefix Here");
        assert_eq!(filename_to_title("simple.md"), "Simple");
    }

    #[test]
    fn test_filename_to_title_no_extension() {
        assert_eq!(filename_to_title("0001-no-extension"), "No Extension");
        assert_eq!(filename_to_title("just-a-name"), "Just A Name");
    }

    #[test]
    fn test_filename_to_title_empty() {
        assert_eq!(filename_to_title(""), "");
    }

    #[test]
    fn test_filename_to_title_single_word() {
        assert_eq!(filename_to_title("word"), "Word");
        assert_eq!(filename_to_title("0042-word"), "Word");
    }

    #[test]
    fn test_sanitize_consecutive_spaces() {
        assert_eq!(sanitize_filename("multiple    spaces"), "multiple-spaces");
    }

    #[test]
    fn test_sanitize_unicode_accents_comprehensive() {
        assert_eq!(sanitize_filename("résumé"), "resume");
        assert_eq!(sanitize_filename("Übung"), "ubung");
        assert_eq!(sanitize_filename("niño"), "nino");
        assert_eq!(sanitize_filename("François"), "francois");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn sanitize_is_idempotent(s in "\\PC*") {
            let once = sanitize_filename(&s);
            let twice = sanitize_filename(&once);
            prop_assert_eq!(once, twice);
        }

        #[test]
        fn sanitize_respects_length_limit(s in "\\PC*") {
            let result = sanitize_filename(&s);
            prop_assert!(result.len() <= 100);
        }

        #[test]
        fn sanitize_never_empty_for_valid_input(s in "[a-zA-Z0-9 ]+") {
            let result = sanitize_filename(&s);
            prop_assert!(!result.is_empty());
        }

        #[test]
        fn sanitize_only_contains_valid_chars(s in "\\PC*") {
            let result = sanitize_filename(&s);
            if result != "untitled" {
                prop_assert!(result
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.'));
            }
        }

        #[test]
        fn sanitize_no_leading_or_trailing_hyphens_or_dots(s in "\\PC*") {
            let result = sanitize_filename(&s);
            if result != "untitled" {
                prop_assert!(!result.starts_with('-'));
                prop_assert!(!result.ends_with('-'));
                prop_assert!(!result.starts_with('.'));
                prop_assert!(!result.ends_with('.'));
            }
        }

        #[test]
        fn build_filename_always_has_correct_format(num in 0u32..10000, title in "\\PC*") {
            let result = build_filename(num, &title);
            let expected_prefix = format!("{:04}-", num);
            prop_assert!(result.ends_with(".md"));
            prop_assert!(result.starts_with(&expected_prefix));
        }

        #[test]
        fn sanitize_lowercase_property(s in "[A-Za-z0-9 ]+") {
            let result = sanitize_filename(&s);
            prop_assert_eq!(result.to_lowercase(), result);
        }

        #[test]
        fn no_consecutive_hyphens(s in "\\PC*") {
            let result = sanitize_filename(&s);
            prop_assert!(!result.contains("--"));
        }
    }
}
