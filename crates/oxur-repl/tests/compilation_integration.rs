//! Integration tests for the full compilation pipeline
//!
//! Tests the three-tier execution model with actual compilation,
//! caching, and subprocess execution.

use oxur_repl::eval::{EvalContext, ExecutionTier};
use oxur_repl::protocol::{ReplMode, SessionId};

/// Helper to generate valid Rust code for testing
/// Uses code that doesn't print to stdout to avoid interfering with IPC protocol
fn test_code(name: &str) -> String {
    format!(r#"let {} = 42; let _ = {} + 1;"#, name, name)
}

/// Test basic compilation and execution
#[tokio::test]
async fn test_basic_compilation() {
    let session_id = SessionId::new("compile-test-basic");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context with compilation");

    // Valid Rust code (no stdout output to avoid IPC interference)
    let result = ctx.eval(r#"let x = 42; let _ = x + 1;"#).await.expect("Evaluation failed");

    // Should use JustInTime tier (first execution)
    assert_eq!(result.tier, ExecutionTier::JustInTime);
    assert!(!result.cached);
}

/// Test caching behavior - second execution should be faster
#[tokio::test]
async fn test_compilation_caching() {
    let session_id = SessionId::new("compile-test-caching");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context with compilation");

    let code = test_code("cached");

    // First execution - JustInTime (compile and execute)
    let result1 = ctx.eval(&code).await.expect("First evaluation failed");
    assert_eq!(result1.tier, ExecutionTier::JustInTime);
    assert!(!result1.cached);

    // Second execution - should use CachedLoaded tier (library already loaded)
    let result2 = ctx.eval(&code).await.expect("Second evaluation failed");
    // Now that caching is implemented, second execution uses already-loaded library
    assert_eq!(result2.tier, ExecutionTier::CachedLoaded);
}

/// Test that calculator mode bypasses compilation
#[tokio::test]
async fn test_calculator_bypass() {
    let session_id = SessionId::new("compile-test-calculator");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Lisp)
        .expect("Failed to create context with compilation");

    // Simple arithmetic that calculator can handle
    let result = ctx.eval("(+ 1 2)").await.expect("Evaluation failed");

    // Should use Calculator tier (no compilation needed)
    assert_eq!(result.tier, ExecutionTier::Calculator);
    assert_eq!(result.value, "3");
}

/// Test compilation with different optimization levels
#[tokio::test]
async fn test_optimization_levels() {
    // The compiler uses optimization level 2 by default
    let session_id = SessionId::new("compile-test-opt");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context with compilation");

    let result = ctx.eval(&test_code("opt")).await.expect("Evaluation failed");

    assert_eq!(result.tier, ExecutionTier::JustInTime);
}

/// Test multiple different code snippets
#[tokio::test]
async fn test_multiple_compilations() {
    let session_id = SessionId::new("compile-test-multiple");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context with compilation");

    // Compile and execute multiple different code snippets
    let codes = vec![test_code("x"), test_code("y"), test_code("z")];

    for code in &codes {
        let result = ctx.eval(code).await.expect("Evaluation failed");
        assert_eq!(result.tier, ExecutionTier::JustInTime);
    }
}

/// Test session isolation - different sessions should have separate state
#[tokio::test]
async fn test_session_isolation() {
    // Create first session and execute
    let session1_id = SessionId::new("compile-test-iso1");
    let mut ctx1 = EvalContext::with_compilation(session1_id, ReplMode::Sexpr)
        .expect("Failed to create context1");

    let code1 = test_code("iso1");
    let result1 = ctx1.eval(&code1).await.expect("Session1 evaluation failed");
    assert_eq!(result1.tier, ExecutionTier::JustInTime);

    // Drop first context to ensure subprocess cleanup
    drop(ctx1);
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Create second session with different code
    let session2_id = SessionId::new("compile-test-iso2");
    let mut ctx2 = EvalContext::with_compilation(session2_id, ReplMode::Sexpr)
        .expect("Failed to create context2");

    let code2 = test_code("iso2");
    let result2 = ctx2.eval(&code2).await.expect("Session2 evaluation failed");

    // Second session should also compile fresh
    assert_eq!(result2.tier, ExecutionTier::JustInTime);
}

/// Test that wrapper generates valid identifiers
#[tokio::test]
async fn test_wrapper_identifier_generation() {
    let session_id = SessionId::new("compile-test-wrapper");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context");

    // Test with code that will be hashed to create an identifier
    let result = ctx.eval(&test_code("wrapper")).await.expect("Evaluation failed");

    assert_eq!(result.tier, ExecutionTier::JustInTime);
}

/// Test fallback when compilation pipeline is not initialized
#[tokio::test]
async fn test_fallback_without_pipeline() {
    let session_id = SessionId::new("compile-test-fallback");
    let mut ctx = EvalContext::new(session_id, ReplMode::Sexpr);

    // Without compilation pipeline, should fall back to placeholder
    let result = ctx.eval("fallback-test").await.expect("Evaluation failed");

    assert_eq!(result.tier, ExecutionTier::JustInTime);
    assert!(result.value.contains("placeholder"));
}

/// Test different REPL modes
#[tokio::test]
async fn test_different_modes() {
    // Test Lisp mode - uses the full Lisp→Rust compilation pipeline
    // The Lowerer currently only supports deffn with println! macro calls
    let lisp_id = SessionId::new("compile-test-lisp");
    let mut lisp_ctx = EvalContext::with_compilation(lisp_id, ReplMode::Lisp)
        .expect("Failed to create Lisp context");

    // Valid Lisp code that the Lowerer can handle
    let lisp_code = r#"(deffn main () (println! "Hello from Lisp mode"))"#;
    let lisp_result = lisp_ctx.eval(lisp_code).await.expect("Lisp evaluation failed");
    assert_eq!(lisp_result.tier, ExecutionTier::JustInTime);

    // Test Sexpr mode - passes raw Rust code directly to compilation
    let sexpr_id = SessionId::new("compile-test-sexpr");
    let mut sexpr_ctx = EvalContext::with_compilation(sexpr_id, ReplMode::Sexpr)
        .expect("Failed to create Sexpr context");

    let sexpr_result = sexpr_ctx.eval(&test_code("sexpr")).await.expect("Sexpr evaluation failed");
    assert_eq!(sexpr_result.tier, ExecutionTier::JustInTime);
}

/// Test cache persistence across evaluations
#[tokio::test]
async fn test_cache_persistence() {
    let session_id = SessionId::new("compile-test-persistence");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context");

    let code = test_code("persist");

    // First evaluation
    ctx.eval(&code).await.expect("First evaluation failed");

    // The artifact should be in the cache now
    // Second evaluation should reuse the already-loaded library
    let result = ctx.eval(&code).await.expect("Second evaluation failed");

    // Should use CachedLoaded tier since library is already loaded
    assert_eq!(result.tier, ExecutionTier::CachedLoaded);
}

/// Test clone_to functionality with compilation pipeline
#[tokio::test]
async fn test_clone_to_with_pipeline() {
    let session1_id = SessionId::new("compile-test-clone-source");
    let mut ctx1 = EvalContext::with_compilation(session1_id, ReplMode::Sexpr)
        .expect("Failed to create source context");

    // Execute some code in the first context
    ctx1.eval(&test_code("clone1")).await.expect("Source evaluation failed");

    // Clone to a new session
    let session2_id = SessionId::new("compile-test-clone-target");
    let mut ctx2 = ctx1.clone_to(session2_id);

    // The cloned context should work
    let result = ctx2.eval(&test_code("clone2")).await.expect("Cloned context evaluation failed");

    assert_eq!(result.tier, ExecutionTier::JustInTime);
}

/// Test that empty parse results are handled correctly
#[tokio::test]
async fn test_empty_parse_handling() {
    let session_id = SessionId::new("compile-test-empty");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context");

    // Empty code should be handled gracefully
    // Note: Current parser implementation returns empty vec for some inputs
    let result = ctx.eval("").await;

    // Should either succeed with placeholder or handle gracefully
    if let Ok(res) = result {
        assert!(res.value.contains("placeholder") || res.value.is_empty());
    }
}

/// Test concurrent evaluations in the same context
#[tokio::test]
async fn test_concurrent_evaluations() {
    let session_id = SessionId::new("compile-test-concurrent");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context");

    // Note: We can't truly run concurrent evaluations on the same context
    // because eval() takes &mut self. But we can test sequential evaluations
    // rapidly to ensure state is maintained correctly.

    for i in 0..5 {
        let code = test_code(&format!("concurrent_{}", i));
        let result = ctx.eval(&code).await.expect("Evaluation failed");
        assert_eq!(result.tier, ExecutionTier::JustInTime);
    }
}

/// Test hash stability - same code generates same hash
#[tokio::test]
async fn test_hash_stability() {
    let session_id = SessionId::new("compile-test-hash");
    let mut ctx = EvalContext::with_compilation(session_id, ReplMode::Sexpr)
        .expect("Failed to create context");

    let code = test_code("hash");

    // Same code should generate same hash
    let result1 = ctx.eval(&code).await.expect("First evaluation failed");
    let result2 = ctx.eval(&code).await.expect("Second evaluation failed");

    // First should compile, second should use cached/loaded library
    assert_eq!(result1.tier, ExecutionTier::JustInTime);
    assert_eq!(result2.tier, ExecutionTier::CachedLoaded);
}
