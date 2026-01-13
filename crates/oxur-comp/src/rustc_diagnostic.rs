//! rustc diagnostic parser
//!
//! Parses JSON diagnostic output from rustc to extract error positions.

use serde::Deserialize;

/// A diagnostic message from rustc
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcDiagnostic {
    /// The main error message
    pub message: String,

    /// Optional error code (e.g., E0425)
    pub code: Option<RustcCode>,

    /// Severity level: "error", "warning", "note", "help"
    pub level: String,

    /// Source code spans where the error occurred
    pub spans: Vec<RustcSpan>,

    /// Child diagnostics (notes, suggestions)
    pub children: Vec<RustcDiagnostic>,

    /// Rendered text output (optional)
    pub rendered: Option<String>,
}

impl RustcDiagnostic {
    /// Parse a rustc diagnostic from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Parse multiple diagnostics from JSON lines
    pub fn from_json_lines(json_lines: &str) -> Result<Vec<Self>, serde_json::Error> {
        json_lines
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str)
            .collect()
    }

    /// Get the primary span (the main location of the error)
    pub fn primary_span(&self) -> Option<&RustcSpan> {
        self.spans.iter().find(|s| s.is_primary)
    }

    /// Get the primary position as (file, line, column)
    pub fn primary_position(&self) -> Option<(String, usize, usize)> {
        self.primary_span().map(|span| (span.file_name.clone(), span.line_start, span.column_start))
    }

    /// Check if this is an error (vs warning or note)
    pub fn is_error(&self) -> bool {
        self.level == "error"
    }

    /// Check if this is a warning
    pub fn is_warning(&self) -> bool {
        self.level == "warning"
    }
}

/// Error code from rustc
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcCode {
    /// Error code (e.g., "E0425")
    pub code: String,

    /// Long explanation text (optional)
    #[serde(default)]
    pub explanation: Option<String>,
}

/// A source code span in a rustc diagnostic
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcSpan {
    /// Source file path
    pub file_name: String,

    /// Byte offset start (0-indexed)
    pub byte_start: usize,

    /// Byte offset end (0-indexed)
    pub byte_end: usize,

    /// Line number start (1-indexed)
    pub line_start: usize,

    /// Line number end (1-indexed)
    pub line_end: usize,

    /// Column number start (1-indexed)
    pub column_start: usize,

    /// Column number end (1-indexed)
    pub column_end: usize,

    /// Whether this is the primary location
    pub is_primary: bool,

    /// Text snippets
    pub text: Vec<RustcSpanText>,

    /// Optional label text
    pub label: Option<String>,

    /// Optional suggested replacement
    #[serde(default)]
    pub suggested_replacement: Option<String>,

    /// Applicability of suggestion
    #[serde(default)]
    pub suggestion_applicability: Option<String>,

    /// Macro expansion context
    #[serde(default)]
    pub expansion: Option<Box<RustcExpansion>>,
}

/// Text snippet from a span
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcSpanText {
    /// The source text
    pub text: String,

    /// Start of highlight in text (1-indexed)
    pub highlight_start: usize,

    /// End of highlight in text (1-indexed)
    pub highlight_end: usize,
}

/// Macro expansion context (simplified)
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustcExpansion {
    /// Span where the macro was expanded
    pub span: RustcSpan,

    /// Name of the macro
    pub macro_decl_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_error() {
        let json = r#"{
            "message": "cannot find value `x` in this scope",
            "code": {
                "code": "E0425",
                "explanation": null
            },
            "level": "error",
            "spans": [
                {
                    "file_name": "test.rs",
                    "byte_start": 42,
                    "byte_end": 43,
                    "line_start": 3,
                    "line_end": 3,
                    "column_start": 5,
                    "column_end": 6,
                    "is_primary": true,
                    "text": [
                        {
                            "text": "    x",
                            "highlight_start": 5,
                            "highlight_end": 6
                        }
                    ],
                    "label": "not found in this scope",
                    "suggested_replacement": null,
                    "suggestion_applicability": null,
                    "expansion": null
                }
            ],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        assert_eq!(diagnostic.message, "cannot find value `x` in this scope");
        assert_eq!(diagnostic.level, "error");
        assert!(diagnostic.is_error());
        assert!(!diagnostic.is_warning());
        assert_eq!(diagnostic.spans.len(), 1);

        let span = &diagnostic.spans[0];
        assert_eq!(span.file_name, "test.rs");
        assert_eq!(span.line_start, 3);
        assert_eq!(span.column_start, 5);
        assert!(span.is_primary);
    }

    #[test]
    fn test_primary_position() {
        let json = r#"{
            "message": "test error",
            "code": null,
            "level": "error",
            "spans": [
                {
                    "file_name": "test.rs",
                    "byte_start": 0,
                    "byte_end": 1,
                    "line_start": 1,
                    "line_end": 1,
                    "column_start": 1,
                    "column_end": 2,
                    "is_primary": true,
                    "text": [],
                    "label": null,
                    "suggested_replacement": null,
                    "suggestion_applicability": null,
                    "expansion": null
                }
            ],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        let (file, line, col) = diagnostic.primary_position().unwrap();
        assert_eq!(file, "test.rs");
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_parse_warning() {
        let json = r#"{
            "message": "unused variable",
            "code": null,
            "level": "warning",
            "spans": [],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        assert!(diagnostic.is_warning());
        assert!(!diagnostic.is_error());
    }

    #[test]
    fn test_parse_multiple_diagnostics() {
        let json_lines = r#"{"message": "error 1", "code": null, "level": "error", "spans": [], "children": [], "rendered": null}
{"message": "error 2", "code": null, "level": "error", "spans": [], "children": [], "rendered": null}"#;

        let diagnostics = RustcDiagnostic::from_json_lines(json_lines).unwrap();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].message, "error 1");
        assert_eq!(diagnostics[1].message, "error 2");
    }

    #[test]
    fn test_no_primary_span() {
        let json = r#"{
            "message": "note",
            "code": null,
            "level": "note",
            "spans": [],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        assert!(diagnostic.primary_span().is_none());
        assert!(diagnostic.primary_position().is_none());
    }

    #[test]
    fn test_error_code_extraction() {
        let json = r#"{
            "message": "test",
            "code": {
                "code": "E0425",
                "explanation": "Some explanation"
            },
            "level": "error",
            "spans": [],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        assert!(diagnostic.code.is_some());
        let code = diagnostic.code.unwrap();
        assert_eq!(code.code, "E0425");
        assert_eq!(code.explanation, Some("Some explanation".to_string()));
    }
}
