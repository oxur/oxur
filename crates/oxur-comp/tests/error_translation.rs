//! Integration tests for error translation
//!
//! Tests the complete error reporting pipeline:
//! Oxur source → Parse → Expand → Lower → Generate → Compile → Error translation

use oxur_comp::Compiler;
use oxur_lang::{Expander, Parser};
use tempfile::TempDir;

/// Helper to compile Oxur code and capture error
fn compile_and_get_error(source: &str) -> Result<String, String> {
    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).map_err(|e| format!("Expand error: {}", e))?;
    let source_map = expander.source_map().clone();

    let temp_dir = TempDir::new().map_err(|e| format!("TempDir error: {}", e))?;
    let output_dir = temp_dir.path().join("build");
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("Create dir error: {}", e))?;

    let mut compiler = Compiler::new(output_dir);
    let binary_path = temp_dir.path().join("test_binary");

    match compiler.compile(core_forms, source_map, &binary_path) {
        Ok(_) => Err("Expected compilation to fail but it succeeded".to_string()),
        Err(e) => Ok(e.to_string()),
    }
}

#[test]
fn test_lowering_error_reported() {
    // This test uses code that fails at lowering stage (variable in println!)
    // Tests that lowering errors are properly reported
    let source = r#"(deffn main ()
  (println! x))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Should contain error message about unsupported syntax
    assert!(
        error.contains("Lowering error") || error.contains("Only single string arguments"),
        "Error should describe lowering limitation: {}",
        error
    );

    // Error reporting infrastructure is working (caught and formatted)
    assert!(!error.is_empty(), "Error message should not be empty: {}", error);
}

#[test]
fn test_multiple_lowering_errors() {
    // Tests error reporting for code with multiple issues
    let source = r#"(deffn main ()
  (println! x)
  (println! y))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Should report error from lowering
    assert!(
        error.contains("Lowering error") || error.contains("error"),
        "Error should be reported: {}",
        error
    );

    // Error message should not be empty
    assert!(!error.is_empty(), "Error message should exist: {}", error);
}

#[test]
fn test_error_message_format() {
    let source = r#"(deffn main ()
  (println! undefined_var))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Error should mention the problem
    assert!(
        error.contains("Lowering error") || error.contains("error"),
        "Should report error: {}",
        error
    );

    // Error message should provide useful information
    assert!(
        !error.is_empty() && error.len() > 10,
        "Should have meaningful error message: {}",
        error
    );
}

#[test]
fn test_valid_code_no_error() {
    let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;

    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).unwrap();
    let source_map = expander.source_map().clone();

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("build");
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut compiler = Compiler::new(output_dir);
    let binary_path = temp_dir.path().join("test_binary");

    // Should compile successfully
    let result = compiler.compile(core_forms, source_map, &binary_path);
    assert!(result.is_ok(), "Valid code should compile: {:?}", result.err());

    // Binary should exist
    assert!(binary_path.exists(), "Binary should be created");
}

#[test]
fn test_error_information_included() {
    // Tests that errors include useful information
    let source = r#"(deffn main ()
  (println! nonexistent))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Should contain error information (lowering or compilation)
    assert!(
        error.contains("Lowering error") || error.contains("error"),
        "Error should provide information: {}",
        error
    );

    // Should describe the specific problem
    assert!(
        error.contains("string") || error.contains("arguments") || error.contains("supported"),
        "Error should describe the issue: {}",
        error
    );
}

#[test]
fn test_source_map_passed_through() {
    let source = r#"(deffn main ()
  (println! "Test"))"#;

    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).unwrap();
    let source_map = expander.source_map().clone();

    // SourceMap should have surface mappings
    let stats = source_map.stats();
    assert!(stats.surface_nodes > 0, "Should have surface mappings before compilation");

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("build");
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut compiler = Compiler::new(output_dir);
    let binary_path = temp_dir.path().join("test_binary");

    // Compile should return SourceMap with lowering mappings added
    let result_map = compiler.compile(core_forms, source_map, &binary_path).unwrap();

    // Should have both surface and lowering mappings
    let result_stats = result_map.stats();
    assert!(result_stats.surface_nodes > 0, "Should preserve surface mappings after compilation");
    assert!(result_stats.lowerings > 0, "Should add lowering mappings during compilation");
}

#[test]
fn test_error_with_unsupported_syntax() {
    // Tests error reporting for unsupported macro arguments
    let source = r#"(deffn main ()
  (println! invalid_identifier))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Error should describe the problem
    assert!(
        error.contains("Lowering error") || error.contains("error"),
        "Should describe the error: {}",
        error
    );

    // Should be informative
    assert!(
        error.contains("string") || error.contains("arguments"),
        "Should explain limitation: {}",
        error
    );
}

#[test]
fn test_error_reporting_infrastructure() {
    // This test verifies the error reporting infrastructure works
    let source = r#"(deffn main ()
  (println! x))"#;

    let error = compile_and_get_error(source).expect("Should produce error");

    // Error reporting infrastructure should provide useful error
    // Note: With current lowering limitations, this produces a lowering error
    // In the future with fuller lowering, this would produce a rustc error

    // 1. Should have error message
    assert!(
        error.contains("Lowering error") || error.contains("error"),
        "Should report error: {}",
        error
    );

    // 2. Should be informative
    assert!(error.len() > 10, "Should have meaningful error message: {}", error);

    // 3. Should describe the issue
    assert!(
        error.contains("string") || error.contains("arguments") || error.contains("supported"),
        "Should describe what went wrong: {}",
        error
    );
}
