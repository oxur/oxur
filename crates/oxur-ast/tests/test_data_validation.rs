use oxur_ast::sexp::Parser;
use oxur_ast::ParseError;
use std::fs;
use std::path::PathBuf;

/// Helper to get the path to test-data directory
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data")
}

/// Helper to parse a file and return the result
fn parse_test_file(path: &PathBuf) -> Result<(), String> {
    Parser::parse_file(path).map(|_| ()).map_err(|e| format!("Failed to parse {:?}: {}", path, e))
}

// ===== Simple Examples Validation =====

#[test]
fn test_all_simple_examples_are_valid() {
    let simple_dir = test_data_dir().join("examples/simple");
    let entries = fs::read_dir(&simple_dir)
        .unwrap_or_else(|e| panic!("Failed to read simple examples directory: {}", e));

    let mut count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
            parse_test_file(&path).unwrap();
            count += 1;
        }
    }

    assert!(count >= 8, "Expected at least 8 simple examples, found {}", count);
}

// ===== Intermediate Examples Validation =====

#[test]
fn test_all_intermediate_examples_are_valid() {
    let intermediate_dir = test_data_dir().join("examples/intermediate");
    let entries = fs::read_dir(&intermediate_dir)
        .unwrap_or_else(|e| panic!("Failed to read intermediate examples directory: {}", e));

    let mut count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
            parse_test_file(&path).unwrap();
            count += 1;
        }
    }

    assert!(count >= 6, "Expected at least 6 intermediate examples, found {}", count);
}

// ===== Complex Examples Validation =====

#[test]
fn test_all_complex_examples_are_valid() {
    let complex_dir = test_data_dir().join("examples/complex");
    let entries = fs::read_dir(&complex_dir)
        .unwrap_or_else(|e| panic!("Failed to read complex examples directory: {}", e));

    let mut count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
            parse_test_file(&path).unwrap();
            count += 1;
        }
    }

    assert!(count >= 4, "Expected at least 4 complex examples, found {}", count);
}

// ===== Fixtures Validation =====

#[test]
fn test_all_crate_fixtures_are_valid() {
    let crate_dir = test_data_dir().join("fixtures/crate");
    let entries = fs::read_dir(&crate_dir)
        .unwrap_or_else(|e| panic!("Failed to read crate fixtures directory: {}", e));

    let mut count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
            parse_test_file(&path).unwrap();
            count += 1;
        }
    }

    assert!(count >= 3, "Expected at least 3 crate fixtures, found {}", count);
}

#[test]
fn test_all_item_fixtures_are_valid() {
    let item_dir = test_data_dir().join("fixtures/item");
    let entries = fs::read_dir(&item_dir)
        .unwrap_or_else(|e| panic!("Failed to read item fixtures directory: {}", e));

    let mut count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
            parse_test_file(&path).unwrap();
            count += 1;
        }
    }

    assert!(count >= 5, "Expected at least 5 item fixtures, found {}", count);
}

#[test]
fn test_all_expr_fixtures_are_valid() {
    let expr_dir = test_data_dir().join("fixtures/expr");
    let entries = fs::read_dir(&expr_dir)
        .unwrap_or_else(|e| panic!("Failed to read expr fixtures directory: {}", e));

    let mut count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
            parse_test_file(&path).unwrap();
            count += 1;
        }
    }

    assert!(count >= 5, "Expected at least 5 expr fixtures, found {}", count);
}

#[test]
fn test_all_stmt_fixtures_are_valid() {
    let stmt_dir = test_data_dir().join("fixtures/stmt");
    let entries = fs::read_dir(&stmt_dir)
        .unwrap_or_else(|e| panic!("Failed to read stmt fixtures directory: {}", e));

    let mut count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
            parse_test_file(&path).unwrap();
            count += 1;
        }
    }

    assert!(count >= 4, "Expected at least 4 stmt fixtures, found {}", count);
}

#[test]
fn test_all_block_fixtures_are_valid() {
    let block_dir = test_data_dir().join("fixtures/block");
    let entries = fs::read_dir(&block_dir)
        .unwrap_or_else(|e| panic!("Failed to read block fixtures directory: {}", e));

    let mut count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
            parse_test_file(&path).unwrap();
            count += 1;
        }
    }

    assert!(count >= 3, "Expected at least 3 block fixtures, found {}", count);
}

// ===== Error Cases Validation =====

#[test]
fn test_error_cases_fail_as_expected() {
    let error_dir = test_data_dir().join("error-cases");

    // Test unterminated-list.sexp
    let unterminated_list = error_dir.join("unterminated-list.sexp");
    let result = Parser::parse_file(&unterminated_list);
    assert!(result.is_err(), "unterminated-list.sexp should fail to parse");
    assert!(matches!(result, Err(ParseError::UnterminatedList { .. })));

    // Test unexpected-close.sexp
    let unexpected_close = error_dir.join("unexpected-close.sexp");
    let result = Parser::parse_file(&unexpected_close);
    assert!(result.is_err(), "unexpected-close.sexp should fail to parse");
    assert!(matches!(result, Err(ParseError::UnexpectedCloseParen { .. })));

    // Test unterminated-string.sexp
    let unterminated_string = error_dir.join("unterminated-string.sexp");
    let result = Parser::parse_file(&unterminated_string);
    assert!(result.is_err(), "unterminated-string.sexp should fail to parse");
    assert!(matches!(result, Err(ParseError::LexError(_))));

    // Test invalid-escape.sexp
    let invalid_escape = error_dir.join("invalid-escape.sexp");
    let result = Parser::parse_file(&invalid_escape);
    assert!(result.is_err(), "invalid-escape.sexp should fail to parse");
    assert!(matches!(result, Err(ParseError::LexError(_))));
}

// ===== Comprehensive Validation =====

#[test]
fn test_all_valid_test_data_parses() {
    let mut total_count = 0;
    let test_data = test_data_dir();

    // Examples
    for category in &["simple", "intermediate", "complex"] {
        let dir = test_data.join("examples").join(category);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
                    parse_test_file(&path).unwrap();
                    total_count += 1;
                }
            }
        }
    }

    // Fixtures
    for category in &["crate", "item", "expr", "stmt", "block"] {
        let dir = test_data.join("fixtures").join(category);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sexp") {
                    parse_test_file(&path).unwrap();
                    total_count += 1;
                }
            }
        }
    }

    assert!(total_count >= 38, "Expected at least 38 valid test files, found {}", total_count);
}
