use oxur_ast::commands::verify;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_verify_simple_function() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() {}").unwrap();

    // Verify round-trip with minimal verbosity
    let result = verify::execute(input_file, false);

    assert!(result.is_ok());
}

#[test]
fn test_verify_simple_function_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() {}").unwrap();

    // Verify round-trip with verbose output
    let result = verify::execute(input_file, true);

    assert!(result.is_ok());
}

#[test]
fn test_verify_public_function() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "pub fn public_fn() {}").unwrap();

    let result = verify::execute(input_file, false);

    assert!(result.is_ok());
}

#[test]
fn test_verify_multiple_functions() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    let rust_code = r#"
fn foo() {}
fn bar() {}
"#;
    fs::write(&input_file, rust_code).unwrap();

    let result = verify::execute(input_file, false);

    assert!(result.is_ok());
}

#[test]
fn test_verify_invalid_rust() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() { invalid rust").unwrap();

    // Verification should fail on invalid Rust
    let result = verify::execute(input_file, false);

    assert!(result.is_err());
}

#[test]
fn test_verify_missing_file() {
    let input_file = PathBuf::from("/nonexistent/file.rs");

    // Verification should fail on missing file
    let result = verify::execute(input_file, false);

    assert!(result.is_err());
}

#[test]
fn test_verify_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "").unwrap();

    // Empty file should verify successfully (empty crate)
    let result = verify::execute(input_file, false);

    assert!(result.is_ok());
}

#[test]
fn test_verify_with_verbose_multiple_functions() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    let rust_code = r#"
fn first() {}
fn second() {}
fn third() {}
"#;
    fs::write(&input_file, rust_code).unwrap();

    // Verify with verbose output for multiple items
    let result = verify::execute(input_file, true);

    assert!(result.is_ok());
}

#[test]
fn test_verify_const_function() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "const fn foo() {}").unwrap();

    let result = verify::execute(input_file, false);

    assert!(result.is_ok());
}

#[test]
fn test_verify_unsafe_function() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "unsafe fn dangerous() {}").unwrap();

    let result = verify::execute(input_file, false);

    assert!(result.is_ok());
}
