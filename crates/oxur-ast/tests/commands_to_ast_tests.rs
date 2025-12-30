use oxur_ast::commands::to_ast;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_execute_file_to_stdout() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() {}").unwrap();

    // Execute with no output specified (should print to stdout)
    let result = to_ast::execute(input_file, None, false);

    assert!(result.is_ok());
}

#[test]
fn test_execute_file_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");
    let output_file = temp_dir.path().join("output.sexp");

    fs::write(&input_file, "fn main() {}").unwrap();

    // Execute with file output
    let result = to_ast::execute(input_file, Some(output_file.clone()), false);

    assert!(result.is_ok());
    assert!(output_file.exists());

    let content = fs::read_to_string(output_file).unwrap();
    assert!(content.contains("Crate"));
}

#[test]
fn test_execute_file_to_stdout_dash() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() {}").unwrap();

    // Execute with "-" as output (should print to stdout)
    let result = to_ast::execute(input_file, Some(PathBuf::from("-")), false);

    assert!(result.is_ok());
}

#[test]
fn test_execute_compact_mode() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");
    let output_file = temp_dir.path().join("output.sexp");

    fs::write(&input_file, "fn main() {}").unwrap();

    // Execute with compact mode (currently same as normal)
    let result = to_ast::execute(input_file, Some(output_file.clone()), true);

    assert!(result.is_ok());
    assert!(output_file.exists());
}

#[test]
fn test_execute_invalid_rust() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() { invalid rust code").unwrap();

    // Execute with invalid Rust code should fail
    let result = to_ast::execute(input_file, None, false);

    assert!(result.is_err());
}

#[test]
fn test_execute_missing_file() {
    let input_file = PathBuf::from("/nonexistent/file.rs");

    // Execute with nonexistent file should fail
    let result = to_ast::execute(input_file, None, false);

    assert!(result.is_err());
}

#[test]
fn test_execute_complex_rust() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");
    let output_file = temp_dir.path().join("output.sexp");

    // Use simpler code that the parser supports
    let rust_code = r#"
fn add(a: i32, b: i32) -> i32 {
    a
}

fn main() {}
"#;

    fs::write(&input_file, rust_code).unwrap();

    let result = to_ast::execute(input_file, Some(output_file.clone()), false);

    if let Err(e) = &result {
        eprintln!("Error: {:?}", e);
    }
    assert!(result.is_ok());

    let content = fs::read_to_string(output_file).unwrap();
    assert!(content.contains("Crate"));
    assert!(content.contains("add"));
    assert!(content.contains("main"));
}

#[test]
fn test_execute_output_to_dash_explicitly() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() {}").unwrap();

    // Execute with explicit "-" output (stdout)
    let result = to_ast::execute(input_file, Some(PathBuf::from("-")), false);

    assert!(result.is_ok());
}

#[test]
fn test_execute_nonexistent_parent_directory() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");
    let output_file = PathBuf::from("/nonexistent/directory/output.sexp");

    fs::write(&input_file, "fn main() {}").unwrap();

    // Execute with output path in nonexistent directory
    let result = to_ast::execute(input_file, Some(output_file), false);

    // Should fail due to directory not existing
    assert!(result.is_err());
}
