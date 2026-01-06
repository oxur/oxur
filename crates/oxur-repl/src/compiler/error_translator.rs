//! Error translation from Rust positions to Oxur positions
//!
//! Parses rustc JSON error output and translates positions back to
//! original Oxur source code using source maps (oxur-smap).
//!
//! Based on ODD-0030 Phase 4: Error Translation

use oxur_smap::{SourceMap, SourcePos};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error translation errors
#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("Failed to parse rustc JSON: {0}")]
    JsonParseFailed(#[from] serde_json::Error),

    #[error("No source map available")]
    NoSourceMap,

    #[error("Position lookup failed: {0}")]
    LookupFailed(String),
}

pub type Result<T> = std::result::Result<T, TranslationError>;

/// Rustc diagnostic level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
    #[serde(rename = "failure-note")]
    FailureNote,
    #[serde(other)]
    Other,
}

/// Span information from rustc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustcSpan {
    pub file_name: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub label: Option<String>,
}

/// Code suggestion from rustc (for future use)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustcSuggestion {
    pub message: String,
    pub applicability: String,
    pub spans: Vec<RustcSpan>,
    #[serde(default)]
    pub replacement: Option<String>,
}

/// Rustc diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustcDiagnostic {
    pub message: String,
    pub code: Option<RustcCode>,
    pub level: DiagnosticLevel,
    pub spans: Vec<RustcSpan>,
    #[serde(default)]
    pub children: Vec<RustcDiagnostic>,
    #[serde(default)]
    pub rendered: Option<String>,
}

/// Error code from rustc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustcCode {
    pub code: String,
    pub explanation: Option<String>,
}

/// Translated diagnostic with Oxur positions
#[derive(Debug, Clone)]
pub struct TranslatedDiagnostic {
    pub message: String,
    pub level: DiagnosticLevel,
    pub code: Option<String>,
    pub primary_span: Option<TranslatedSpan>,
    pub secondary_spans: Vec<TranslatedSpan>,
    pub children: Vec<TranslatedDiagnostic>,
    pub suggestion: Option<String>,
}

/// Translated span with Oxur position
#[derive(Debug, Clone)]
pub struct TranslatedSpan {
    pub pos: SourcePos,
    pub label: Option<String>,
    pub source_text: Option<String>,
}

/// Error translator
///
/// Parses rustc JSON error output and translates positions using source maps.
///
/// # Example
///
/// ```no_run
/// use oxur_repl::compiler::ErrorTranslator;
/// use oxur_smap::SourceMap;
///
/// let source_map = SourceMap::new();
/// let translator = ErrorTranslator::with_source_map(source_map);
///
/// let rustc_stderr = r#"{"message":"cannot find value","level":"error",...}"#;
/// let errors = translator.parse_and_translate(rustc_stderr)?;
///
/// for error in errors {
///     println!("{}", error.format());
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ErrorTranslator {
    source_map: Option<SourceMap>,
}

impl ErrorTranslator {
    /// Create a new error translator without source map
    ///
    /// Errors will be passed through without position translation.
    pub fn new() -> Self {
        Self { source_map: None }
    }

    /// Create a new error translator with source map
    pub fn with_source_map(source_map: SourceMap) -> Self {
        Self { source_map: Some(source_map) }
    }

    /// Parse rustc JSON error output and translate positions
    ///
    /// # Arguments
    ///
    /// * `stderr` - Raw stderr from rustc with `--error-format=json`
    ///
    /// # Returns
    ///
    /// Vector of translated diagnostics with Oxur positions
    pub fn parse_and_translate(&self, stderr: &str) -> Result<Vec<TranslatedDiagnostic>> {
        let mut diagnostics = Vec::new();

        // Parse each line of JSON output
        for line in stderr.lines() {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse JSON diagnostic
            match serde_json::from_str::<RustcDiagnostic>(line) {
                Ok(diag) => {
                    // Translate and collect
                    let translated = self.translate_diagnostic(&diag)?;
                    diagnostics.push(translated);
                }
                Err(e) => {
                    // Not all lines are JSON diagnostics (e.g., "Compiling..." messages)
                    // Only fail on actual JSON parse errors
                    if line.starts_with('{') {
                        return Err(TranslationError::JsonParseFailed(e));
                    }
                }
            }
        }

        Ok(diagnostics)
    }

    /// Translate a single diagnostic
    fn translate_diagnostic(&self, diag: &RustcDiagnostic) -> Result<TranslatedDiagnostic> {
        // Find primary span
        let primary_span =
            diag.spans.iter().find(|s| s.is_primary).and_then(|s| self.translate_span(s).ok());

        // Translate secondary spans
        let secondary_spans = diag
            .spans
            .iter()
            .filter(|s| !s.is_primary)
            .filter_map(|s| self.translate_span(s).ok())
            .collect();

        // Translate children recursively
        let children = diag
            .children
            .iter()
            .filter_map(|child| self.translate_diagnostic(child).ok())
            .collect();

        Ok(TranslatedDiagnostic {
            message: diag.message.clone(),
            level: diag.level.clone(),
            code: diag.code.as_ref().map(|c| c.code.clone()),
            primary_span,
            secondary_spans,
            children,
            suggestion: None, // TODO: Extract suggestions
        })
    }

    /// Translate a rustc span to Oxur position
    fn translate_span(&self, span: &RustcSpan) -> Result<TranslatedSpan> {
        // If no source map, return Rust positions as-is
        let pos = if let Some(_source_map) = &self.source_map {
            // TODO: Implement actual source map lookup
            // For now, create a placeholder position
            SourcePos::repl(
                span.line_start as u32,
                span.column_start as u32,
                (span.byte_end - span.byte_start) as u32,
            )
        } else {
            // No source map - use Rust positions directly
            SourcePos::repl(
                span.line_start as u32,
                span.column_start as u32,
                (span.byte_end - span.byte_start) as u32,
            )
        };

        Ok(TranslatedSpan { pos, label: span.label.clone(), source_text: None })
    }
}

impl Default for ErrorTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslatedDiagnostic {
    /// Format the diagnostic as a user-friendly error message
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Level and message
        let level_str = match self.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Note => "note",
            DiagnosticLevel::Help => "help",
            DiagnosticLevel::FailureNote => "failure-note",
            DiagnosticLevel::Other => "diagnostic",
        };

        if let Some(code) = &self.code {
            output.push_str(&format!("{}[{}]: {}\n", level_str, code, self.message));
        } else {
            output.push_str(&format!("{}: {}\n", level_str, self.message));
        }

        // Primary span with position
        if let Some(span) = &self.primary_span {
            output.push_str(&format!(" --> line {}, column {}\n", span.pos.line, span.pos.column));

            if let Some(label) = &span.label {
                output.push_str(&format!("  | {}\n", label));
            }
        }

        // Children
        for child in &self.children {
            let child_str = child.format();
            for line in child_str.lines() {
                output.push_str(&format!("  {}\n", line));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_translator_creation() {
        let translator = ErrorTranslator::new();
        assert!(translator.source_map.is_none());
    }

    #[test]
    fn test_parse_simple_error() {
        let translator = ErrorTranslator::new();

        let json = r#"{"message":"cannot find value `x` in this scope","code":{"code":"E0425","explanation":"..."},"level":"error","spans":[{"file_name":"test.rs","byte_start":0,"byte_end":1,"line_start":1,"line_end":1,"column_start":5,"column_end":6,"is_primary":true,"label":"not found in this scope"}],"children":[]}"#;

        let result = translator.parse_and_translate(json);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();
        assert_eq!(diagnostics.len(), 1);

        let diag = &diagnostics[0];
        assert_eq!(diag.message, "cannot find value `x` in this scope");
        assert_eq!(diag.level, DiagnosticLevel::Error);
        assert_eq!(diag.code, Some("E0425".to_string()));
    }

    #[test]
    fn test_parse_empty_input() {
        let translator = ErrorTranslator::new();
        let result = translator.parse_and_translate("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_format_diagnostic() {
        let diag = TranslatedDiagnostic {
            message: "test error".to_string(),
            level: DiagnosticLevel::Error,
            code: Some("E0001".to_string()),
            primary_span: Some(TranslatedSpan {
                pos: SourcePos::repl(10, 15, 5),
                label: Some("here".to_string()),
                source_text: None,
            }),
            secondary_spans: vec![],
            children: vec![],
            suggestion: None,
        };

        let formatted = diag.format();
        assert!(formatted.contains("error[E0001]: test error"));
        assert!(formatted.contains("line 10, column 15"));
        assert!(formatted.contains("here"));
    }
}
