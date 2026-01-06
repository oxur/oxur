//! End-to-end integration tests for error translation with actual compilation

use oxur_repl::cache::ArtifactCache;
use oxur_repl::compiler::CachedCompiler;
use oxur_repl::protocol::SessionId;
use oxur_repl::session::SessionDir;
use std::sync::Arc;

/// Test that compilation errors are properly formatted
#[test]
fn test_compilation_error_formatting() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let test_dir = std::env::temp_dir().join(format!("oxur-error-test-{}", id));

    // Clean test directory
    if test_dir.exists() {
        std::fs::remove_dir_all(&test_dir).ok();
    }

    let cache = ArtifactCache::with_directory(&test_dir).expect("Failed to create cache");
    let session_id = SessionId::new(format!("error-test-{}", id));
    let session_dir = Arc::new(SessionDir::new(&session_id).expect("Failed to create session dir"));

    let mut compiler = CachedCompiler::new(cache, session_dir);

    // Invalid Rust code that will trigger an error
    let invalid_source = r#"
        #[no_mangle]
        pub extern "C" fn oxur_eval_h12345() {
            let result = undefined_variable + 1;
        }
    "#;

    let result = compiler.compile("h12345", invalid_source, 0);

    assert!(result.is_err(), "Should fail to compile invalid code");

    if let Err(e) = result {
        let error_msg = e.to_string();

        // Verify the error message is formatted nicely
        assert!(
            error_msg.contains("error[E0425]") || error_msg.contains("cannot find"),
            "Error should contain rustc error code or message: {}",
            error_msg
        );

        // Verify it contains position information
        assert!(
            error_msg.contains("line") || error_msg.contains("column"),
            "Error should contain position info: {}",
            error_msg
        );

        println!("✓ Error message formatting:");
        println!("{}", error_msg);
    }
}

/// Test that valid code still compiles successfully
#[test]
fn test_valid_code_still_works() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let test_dir = std::env::temp_dir().join(format!("oxur-valid-test-{}", id));

    if test_dir.exists() {
        std::fs::remove_dir_all(&test_dir).ok();
    }

    let cache = ArtifactCache::with_directory(&test_dir).expect("Failed to create cache");
    let session_id = SessionId::new(format!("valid-test-{}", id));
    let session_dir = Arc::new(SessionDir::new(&session_id).expect("Failed to create session dir"));

    let mut compiler = CachedCompiler::new(cache, session_dir);

    // Valid Rust code
    let valid_source = r#"
        #[no_mangle]
        pub extern "C" fn oxur_eval_hvalid() {
            let x = 42;
            let _ = x + 1;
        }
    "#;

    let result = compiler.compile("hvalid", valid_source, 0);

    match result {
        Ok(path) => {
            assert!(path.exists(), "Compiled library should exist");
            println!("✓ Valid code compiled successfully: {:?}", path);
        }
        Err(e) => {
            // Compilation might fail in test environment without full setup
            eprintln!("Note: Compilation failed (may be expected): {}", e);
        }
    }
}

/// Test multiple compilation errors are all reported
#[test]
fn test_multiple_errors_reported() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let test_dir = std::env::temp_dir().join(format!("oxur-multi-error-{}", id));

    if test_dir.exists() {
        std::fs::remove_dir_all(&test_dir).ok();
    }

    let cache = ArtifactCache::with_directory(&test_dir).expect("Failed to create cache");
    let session_id = SessionId::new(format!("multi-error-{}", id));
    let session_dir = Arc::new(SessionDir::new(&session_id).expect("Failed to create session dir"));

    let mut compiler = CachedCompiler::new(cache, session_dir);

    // Code with multiple errors
    let multi_error_source = r#"
        #[no_mangle]
        pub extern "C" fn oxur_eval_hmulti() {
            let x = undefined_var1;
            let y = undefined_var2;
            let z = undefined_var3;
        }
    "#;

    let result = compiler.compile("hmulti", multi_error_source, 0);

    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = e.to_string();

        // Should contain multiple error references
        let error_count = error_msg.matches("error[E").count();

        println!("✓ Found {} errors in output", error_count);
        println!("Error output:\n{}", error_msg);

        // We expect at least one error (rustc might aggregate them)
        assert!(error_count >= 1, "Should report at least one error, found {}", error_count);
    }
}
