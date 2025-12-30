use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_binary_path() -> PathBuf {
    // Get the compiled binary path
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../target/debug/aster");
    path
}

#[test]
fn test_cli_to_ast_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("to-ast")
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn test_cli_to_ast_simple_file() {
    let binary = get_binary_path();
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");
    let output_file = temp_dir.path().join("output.sexp");

    fs::write(&input_file, "fn main() {}").unwrap();

    let output = Command::new(&binary)
        .arg("to-ast")
        .arg(&input_file)
        .arg("--output")
        .arg(&output_file)
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_file.exists());
}

#[test]
fn test_cli_to_ast_missing_file() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("to-ast")
        .arg("/nonexistent/file.rs")
        .output()
        .expect("Failed to execute binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error"));
}

#[test]
fn test_cli_to_rust_simple_file() {
    let binary = get_binary_path();
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.sexp");
    let output_file = temp_dir.path().join("output.rs");

    fs::write(&input_file, "(Crate :items ())").unwrap();

    let output = Command::new(&binary)
        .arg("to-rust")
        .arg(&input_file)
        .arg("--output")
        .arg(&output_file)
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_file.exists());
}

#[test]
fn test_cli_to_rust_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("to-rust")
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn test_cli_verify_simple_file() {
    let binary = get_binary_path();
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() {}").unwrap();

    let output = Command::new(&binary)
        .arg("verify")
        .arg(&input_file)
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_cli_verify_verbose() {
    let binary = get_binary_path();
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.rs");

    fs::write(&input_file, "fn main() {}").unwrap();

    let output = Command::new(&binary)
        .arg("verify")
        .arg(&input_file)
        .arg("--verbose")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_cli_verify_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("verify")
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn test_cli_invalid_command() {
    let binary = get_binary_path();

    let output =
        Command::new(&binary).arg("invalid-command").output().expect("Failed to execute binary");

    assert!(!output.status.success());
}
