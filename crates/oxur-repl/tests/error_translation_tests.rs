//! Integration tests for error translation
//!
//! Tests the ErrorTranslator with real rustc JSON error output

use oxur_repl::compiler::ErrorTranslator;

/// Test parsing a real rustc error (E0425 - cannot find value)
#[test]
fn test_parse_real_error_e0425() {
    let translator = ErrorTranslator::new();

    let json = r#"{"message":"cannot find value `x` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist;\n```\n"},"level":"error","spans":[{"file_name":"test.rs","byte_start":50,"byte_end":51,"line_start":5,"line_end":5,"column_start":5,"column_end":6,"is_primary":true,"text":[{"text":"    x + 1","highlight_start":5,"highlight_end":6}],"label":"not found in this scope","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0425]: cannot find value `x` in this scope\n --> test.rs:5:5\n  |\n5 |     x + 1\n  |     ^ not found in this scope\n\n"}"#;

    let result = translator.parse_and_translate(json);
    assert!(result.is_ok(), "Failed to parse error: {:?}", result.err());

    let diagnostics = result.unwrap();
    assert_eq!(diagnostics.len(), 1);

    let diag = &diagnostics[0];
    assert_eq!(diag.message, "cannot find value `x` in this scope");
    assert_eq!(diag.code, Some("E0425".to_string()));

    // Check formatting
    let formatted = diag.format();
    assert!(formatted.contains("error[E0425]"));
    assert!(formatted.contains("cannot find value"));
    assert!(formatted.contains("line 5, column 5"));
}

/// Test parsing multiple errors in one output
#[test]
fn test_parse_multiple_errors() {
    let translator = ErrorTranslator::new();

    let json = r#"{"message":"cannot find value `x` in this scope","code":{"code":"E0425","explanation":"..."},"level":"error","spans":[{"file_name":"test.rs","byte_start":0,"byte_end":1,"line_start":1,"line_end":1,"column_start":1,"column_end":2,"is_primary":true}],"children":[]}
{"message":"cannot find value `y` in this scope","code":{"code":"E0425","explanation":"..."},"level":"error","spans":[{"file_name":"test.rs","byte_start":10,"byte_end":11,"line_start":2,"line_end":2,"column_start":1,"column_end":2,"is_primary":true}],"children":[]}"#;

    let result = translator.parse_and_translate(json);
    assert!(result.is_ok());

    let diagnostics = result.unwrap();
    assert_eq!(diagnostics.len(), 2);

    assert_eq!(diagnostics[0].message, "cannot find value `x` in this scope");
    assert_eq!(diagnostics[1].message, "cannot find value `y` in this scope");
}

/// Test parsing warning
#[test]
fn test_parse_warning() {
    let translator = ErrorTranslator::new();

    let json = r#"{"message":"unused variable: `x`","code":{"code":"unused_variables","explanation":null},"level":"warning","spans":[{"file_name":"test.rs","byte_start":0,"byte_end":1,"line_start":1,"line_end":1,"column_start":5,"column_end":6,"is_primary":true,"label":"`x` is never read"}],"children":[{"message":"`#[warn(unused_variables)]` on by default","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":null}"#;

    let result = translator.parse_and_translate(json);
    assert!(result.is_ok());

    let diagnostics = result.unwrap();
    assert_eq!(diagnostics.len(), 1);

    let diag = &diagnostics[0];
    assert!(matches!(diag.level, oxur_repl::compiler::DiagnosticLevel::Warning));
    assert_eq!(diag.message, "unused variable: `x`");
    assert_eq!(diag.children.len(), 1);
}

/// Test parsing error with secondary spans
#[test]
fn test_parse_error_with_secondary_spans() {
    let translator = ErrorTranslator::new();

    let json = r#"{"message":"mismatched types","code":{"code":"E0308","explanation":"..."},"level":"error","spans":[{"file_name":"test.rs","byte_start":50,"byte_end":55,"line_start":5,"line_end":5,"column_start":10,"column_end":15,"is_primary":true,"label":"expected `i32`, found `&str`"},{"file_name":"test.rs","byte_start":20,"byte_end":25,"line_start":2,"line_end":2,"column_start":10,"column_end":15,"is_primary":false,"label":"expected due to this"}],"children":[],"rendered":null}"#;

    let result = translator.parse_and_translate(json);
    assert!(result.is_ok());

    let diagnostics = result.unwrap();
    assert_eq!(diagnostics.len(), 1);

    let diag = &diagnostics[0];
    assert_eq!(diag.code, Some("E0308".to_string()));
    assert!(diag.primary_span.is_some());
    assert_eq!(diag.secondary_spans.len(), 1);
}

/// Test handling non-JSON lines (build output)
#[test]
fn test_skip_non_json_lines() {
    let translator = ErrorTranslator::new();

    let mixed_output = r#"   Compiling test v0.1.0
{"message":"cannot find value `x` in this scope","code":{"code":"E0425","explanation":"..."},"level":"error","spans":[{"file_name":"test.rs","byte_start":0,"byte_end":1,"line_start":1,"line_end":1,"column_start":1,"column_end":2,"is_primary":true}],"children":[]}
error: could not compile `test` (lib) due to 1 previous error"#;

    let result = translator.parse_and_translate(mixed_output);
    assert!(result.is_ok());

    let diagnostics = result.unwrap();
    // Should parse the one JSON error, ignoring other lines
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, Some("E0425".to_string()));
}

/// Test empty stderr
#[test]
fn test_empty_stderr() {
    let translator = ErrorTranslator::new();
    let result = translator.parse_and_translate("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

/// Test formatting diagnostic without code
#[test]
fn test_format_without_code() {
    use oxur_repl::compiler::{DiagnosticLevel, TranslatedDiagnostic};

    let diag = TranslatedDiagnostic {
        message: "test message".to_string(),
        level: DiagnosticLevel::Error,
        code: None,
        primary_span: None,
        secondary_spans: vec![],
        children: vec![],
        suggestion: None,
    };

    let formatted = diag.format();
    assert!(formatted.contains("error: test message"));
    assert!(!formatted.contains("["));
}

/// Test formatting with children
#[test]
fn test_format_with_children() {
    use oxur_repl::compiler::{DiagnosticLevel, TranslatedDiagnostic};

    let child = TranslatedDiagnostic {
        message: "help: consider using...".to_string(),
        level: DiagnosticLevel::Help,
        code: None,
        primary_span: None,
        secondary_spans: vec![],
        children: vec![],
        suggestion: None,
    };

    let diag = TranslatedDiagnostic {
        message: "test error".to_string(),
        level: DiagnosticLevel::Error,
        code: Some("E0001".to_string()),
        primary_span: None,
        secondary_spans: vec![],
        children: vec![child],
        suggestion: None,
    };

    let formatted = diag.format();
    assert!(formatted.contains("error[E0001]: test error"));
    assert!(formatted.contains("help: consider using"));
}
