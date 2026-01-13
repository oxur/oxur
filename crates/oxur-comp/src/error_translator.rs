//! Error translation
//!
//! Translates rustc compilation errors from generated Rust code positions
//! back to original Oxur source code positions using the SourceMap.
//!
//! # Current Implementation
//!
//! Stage 1.10 provides the infrastructure for error translation but doesn't
//! yet implement full position lookup. Error messages show:
//! - The error message from rustc
//! - The generated Rust file position (as fallback)
//! - A note that full translation is not yet implemented
//!
//! # Future Enhancement (Phase 2)
//!
//! Full position translation requires a reverse index:
//! - Rust Position (file:line:col) → Rust NodeId
//! - Built during lowering when generating syn nodes
//! - Fast lookup at error translation time
//!
//! This will enable errors like:
//! ```text
//! error: cannot find value `x` in this scope
//!   --> example.oxur:2:8
//! ```
//!
//! Instead of:
//! ```text
//! error: cannot find value `x` in this scope
//!   --> generated.rs:5:10
//! ```

use crate::RustcDiagnostic;
use oxur_smap::SourceMap;

/// Translates rustc error positions to Oxur source positions
pub struct ErrorTranslator {
    source_map: SourceMap,
}

impl ErrorTranslator {
    /// Create a new error translator with the given source map
    pub fn new(source_map: SourceMap) -> Self {
        Self { source_map }
    }

    /// Translate a rustc diagnostic to Oxur source positions
    ///
    /// Returns a formatted error message with Oxur positions where possible.
    /// If translation is not possible, falls back to showing Rust positions.
    pub fn translate_diagnostic(&self, diagnostic: &RustcDiagnostic) -> String {
        let mut output = String::new();

        // Extract primary position from rustc diagnostic
        if let Some((rust_file, rust_line, rust_col)) = diagnostic.primary_position() {
            // TODO: Look up Rust position in reverse index
            // For now, show Rust position as fallback

            output.push_str(&format!("error: {}\n", diagnostic.message));

            // Show Rust position (fallback until reverse index implemented)
            output.push_str(&format!("  --> {}:{}:{}\n", rust_file, rust_line, rust_col));

            // TODO: Show Oxur position when translation available
            output.push_str("  (Note: Error position translation not yet implemented)\n");
        } else {
            // No position available
            output.push_str(&format!("error: {}\n", diagnostic.message));
        }

        // Show error code if available
        if let Some(code) = &diagnostic.code {
            output.push_str(&format!("  code: {}\n", code.code));
        }

        output
    }

    /// Translate all diagnostics and format as a single error message
    pub fn translate_diagnostics(&self, diagnostics: &[RustcDiagnostic]) -> String {
        diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(|d| self.translate_diagnostic(d))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get a reference to the source map
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RustcDiagnostic;

    #[test]
    fn test_translator_creation() {
        let source_map = oxur_smap::SourceMap::new();
        let translator = ErrorTranslator::new(source_map);
        assert_eq!(translator.source_map().stats().surface_nodes, 0);
    }

    #[test]
    fn test_translate_diagnostic_with_position() {
        let source_map = oxur_smap::SourceMap::new();
        let translator = ErrorTranslator::new(source_map);

        let json = r#"{
            "message": "cannot find value `x` in this scope",
            "code": {
                "code": "E0425",
                "explanation": null
            },
            "level": "error",
            "spans": [
                {
                    "file_name": "generated.rs",
                    "byte_start": 42,
                    "byte_end": 43,
                    "line_start": 5,
                    "line_end": 5,
                    "column_start": 10,
                    "column_end": 11,
                    "is_primary": true,
                    "text": [],
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
        let output = translator.translate_diagnostic(&diagnostic);

        // Should contain error message
        assert!(output.contains("cannot find value `x`"));

        // Should contain Rust position (fallback)
        assert!(output.contains("generated.rs:5:10"));

        // Should contain error code
        assert!(output.contains("E0425"));

        // Should note that translation isn't implemented yet
        assert!(output.contains("translation not yet implemented"));
    }

    #[test]
    fn test_translate_diagnostic_without_position() {
        let source_map = oxur_smap::SourceMap::new();
        let translator = ErrorTranslator::new(source_map);

        let json = r#"{
            "message": "aborting due to previous error",
            "code": null,
            "level": "error",
            "spans": [],
            "children": [],
            "rendered": null
        }"#;

        let diagnostic = RustcDiagnostic::from_json(json).unwrap();
        let output = translator.translate_diagnostic(&diagnostic);

        // Should contain error message
        assert!(output.contains("aborting due to previous error"));

        // Should not contain position
        assert!(!output.contains("-->"));
    }

    #[test]
    fn test_translate_multiple_diagnostics() {
        let source_map = oxur_smap::SourceMap::new();
        let translator = ErrorTranslator::new(source_map);

        let json_lines = r#"{"message": "error 1", "code": null, "level": "error", "spans": [], "children": [], "rendered": null}
{"message": "warning 1", "code": null, "level": "warning", "spans": [], "children": [], "rendered": null}
{"message": "error 2", "code": null, "level": "error", "spans": [], "children": [], "rendered": null}"#;

        let diagnostics = RustcDiagnostic::from_json_lines(json_lines).unwrap();
        let output = translator.translate_diagnostics(&diagnostics);

        // Should contain errors but not warnings
        assert!(output.contains("error 1"));
        assert!(output.contains("error 2"));
        assert!(!output.contains("warning 1"));
    }

    #[test]
    fn test_translator_with_populated_source_map() {
        use oxur_lang::{Expander, Parser};

        // Create a source map with actual mappings
        let source = r#"(deffn main ()
  (println! "Hello"))"#;

        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let _core_forms = expander.expand(surface_forms).unwrap();
        let source_map = expander.source_map().clone();

        // Verify source map has mappings
        let stats = source_map.stats();
        assert!(stats.surface_nodes > 0, "Should have surface mappings");

        let translator = ErrorTranslator::new(source_map);

        // Verify translator has access to source map
        assert!(translator.source_map().stats().surface_nodes > 0);
    }

    #[test]
    fn test_source_map_accessor() {
        let mut source_map = oxur_smap::SourceMap::new();
        let node_id = oxur_smap::new_node_id();
        let pos = oxur_smap::SourcePos::new("test.oxur".to_string(), 1, 1, 1);
        source_map.record_surface_node(node_id, pos);

        let translator = ErrorTranslator::new(source_map);

        // Should be able to access source map through translator
        let retrieved_pos = translator.source_map().get_surface_position(&node_id);
        assert!(retrieved_pos.is_some());
    }
}
