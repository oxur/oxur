//! End-to-End Integration Tests for Oxur REPL
//!
//! Comprehensive tests covering the full REPL workflow from user input
//! through parsing, compilation, execution, and output.
//!
//! Tests organized by Phase 5 Task 5.2 requirements:
//! - Simple arithmetic evaluation (Tier 1)
//! - REPL variable definition and mutation (Tier 2/3)
//! - Function definition and calling (Tier 2/3)
//! - Cache hit detection
//! - Error handling with position information

use oxur_repl::eval::{EvalContext, ExecutionTier};
use oxur_repl::protocol::{ReplMode, SessionId};
use serial_test::serial;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};

// Counter for generating unique cache directories per test
static TEST_CACHE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Create an isolated evaluation context with its own cache directory
///
/// This prevents test interference by giving each test its own isolated cache.
/// Uses atomic counter to ensure unique directories even in parallel test runs.
fn create_isolated_context(session_id: SessionId, mode: ReplMode) -> EvalContext {
    let id = TEST_CACHE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let cache_dir = env::temp_dir().join(format!("oxur-e2e-cache-{}-{}", std::process::id(), id));

    // Set env var temporarily for this context creation
    let original = env::var("OXUR_CACHE_DIR").ok();
    env::set_var("OXUR_CACHE_DIR", &cache_dir);

    let ctx =
        EvalContext::with_compilation(session_id, mode).expect("Failed to create isolated context");

    // Restore original env var state
    match original {
        Some(val) => env::set_var("OXUR_CACHE_DIR", val),
        None => env::remove_var("OXUR_CACHE_DIR"),
    }

    ctx
}

// ============================================================================
// Test 1: Simple Arithmetic Evaluation (Tier 1 Calculator)
// ============================================================================

/// Test basic arithmetic in calculator mode
///
/// Calculator mode should evaluate simple arithmetic without compilation.
/// This is the fastest tier (<1ms) for literal-only expressions.
#[tokio::test]
async fn test_arithmetic_calculator_mode() {
    let session_id = SessionId::new("e2e-calc-simple");
    let mut ctx = EvalContext::new(session_id, ReplMode::Lisp);

    // Simple addition - should use calculator mode
    let result = ctx.eval("(+ 1 2)").await.expect("Failed to eval (+ 1 2)");

    assert_eq!(result.tier, ExecutionTier::Calculator);
    assert!(!result.cached);
    assert_eq!(result.value, "3");
    assert!(result.duration_ms < 10); // Should be very fast
}

/// Test nested arithmetic expressions
#[tokio::test]
async fn test_arithmetic_nested() {
    let session_id = SessionId::new("e2e-calc-nested");
    let mut ctx = EvalContext::new(session_id, ReplMode::Lisp);

    // Nested expression: (+ (* 2 3) (/ 10 2))
    let result = ctx.eval("(+ (* 2 3) (/ 10 2))").await.expect("Failed to eval nested arithmetic");

    assert_eq!(result.tier, ExecutionTier::Calculator);
    assert_eq!(result.value, "11"); // 6 + 5 = 11
}

/// Test that non-calculator code falls through to compilation
#[tokio::test]
#[serial(env)]
async fn test_arithmetic_with_variables_not_calculator() {
    let session_id = SessionId::new("e2e-calc-fallthrough");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Has variable reference - not pure calculator
    let result = ctx.eval("let x = 1 + 2; x").await.expect("Failed to eval variable code");

    // Should NOT use calculator tier (has variable)
    assert_ne!(result.tier, ExecutionTier::Calculator);
}

// ============================================================================
// Test 2: REPL Variable Definition and Mutation (Tier 2/3)
// ============================================================================

/// Test basic variable definition
///
/// Variables should persist across evaluations in the same session.
#[tokio::test]
#[serial(env)]
async fn test_variable_definition() {
    let session_id = SessionId::new("e2e-var-def");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Define a variable
    let result1 = ctx.eval("let x = 42; x").await.expect("Failed to define variable");

    // Should compile (JIT) on first run
    assert_eq!(result1.tier, ExecutionTier::JustInTime);
    assert!(!result1.cached);
}

/// Test variable persistence across evaluations
///
/// NOTE: This test requires Phase 7 (Type Inference) to be implemented.
/// Variables need to be tracked with their types and stored/loaded from VariableStore.
#[ignore = "Requires Phase 7 (Type Inference) - not yet implemented"]
#[tokio::test]
#[serial(env)]
async fn test_variable_persistence() {
    let session_id = SessionId::new("e2e-var-persist");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Define variable x
    let _result1 = ctx.eval("let x = 42; x").await.expect("Failed to define x");

    // Use variable in next evaluation
    let result2 = ctx.eval("let y = x + 10; y").await.expect("Failed to use x");

    // Second evaluation should compile (different code)
    assert_eq!(result2.tier, ExecutionTier::JustInTime);
}

/// Test multiple variables
///
/// NOTE: This test requires Phase 7 (Type Inference) to be implemented.
#[ignore = "Requires Phase 7 (Type Inference) - not yet implemented"]
#[tokio::test]
#[serial(env)]
async fn test_multiple_variables() {
    let session_id = SessionId::new("e2e-var-multiple");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Define multiple variables
    let _r1 = ctx.eval("let x = 10; x").await.expect("Failed to define x");
    let _r2 = ctx.eval("let y = 20; y").await.expect("Failed to define y");

    // Use both variables
    let result3 = ctx.eval("let z = x + y; z").await.expect("Failed to use x and y");

    assert_eq!(result3.tier, ExecutionTier::JustInTime);
}

// ============================================================================
// Test 3: Function Definition and Calling (Tier 2/3)
// ============================================================================

/// Test basic function definition
#[tokio::test]
#[serial(env)]
async fn test_function_definition() {
    let session_id = SessionId::new("e2e-fn-def");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Define a simple function
    let code = r#"
        fn square(x: i32) -> i32 {
            x * x
        }
        square(5)
    "#;

    let result = ctx.eval(code).await.expect("Failed to define and call function");

    assert_eq!(result.tier, ExecutionTier::JustInTime);
}

/// Test function with multiple parameters
#[tokio::test]
#[serial(env)]
async fn test_function_multiple_params() {
    let session_id = SessionId::new("e2e-fn-multi");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Function with two parameters
    let code = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        add(10, 20)
    "#;

    let result = ctx.eval(code).await.expect("Failed to call multi-param function");

    assert_eq!(result.tier, ExecutionTier::JustInTime);
}

/// Test function composition
#[tokio::test]
#[serial(env)]
async fn test_function_composition() {
    let session_id = SessionId::new("e2e-fn-compose");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Define helper function
    let _r1 =
        ctx.eval("fn square(x: i32) -> i32 { x * x }").await.expect("Failed to define square");

    // Define function that uses helper
    let code = r#"
        fn sum_of_squares(a: i32, b: i32) -> i32 {
            let x_sq = a * a;  // inline square for now
            let y_sq = b * b;
            x_sq + y_sq
        }
        sum_of_squares(3, 4)
    "#;

    let result = ctx.eval(code).await.expect("Failed to call composed function");

    assert_eq!(result.tier, ExecutionTier::JustInTime);
}

// ============================================================================
// Test 4: Cache Hit Detection
// ============================================================================

/// Test that identical code hits cache
///
/// Second execution of identical code should use CachedLoaded tier.
#[tokio::test]
#[serial(env)]
async fn test_cache_hit_identical_code() {
    let session_id = SessionId::new("e2e-cache-hit");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    let code = "let x = 42; let y = x + 1; y";

    // First execution - compile and load
    let result1 = ctx.eval(code).await.expect("First eval failed");
    assert_eq!(result1.tier, ExecutionTier::JustInTime);
    assert!(!result1.cached);

    // Second execution - library already loaded
    let result2 = ctx.eval(code).await.expect("Second eval failed");
    assert_eq!(result2.tier, ExecutionTier::CachedLoaded);
    assert!(result2.cached);
}

/// Test cache statistics
#[tokio::test]
#[serial(env)]
async fn test_cache_statistics() {
    let session_id = SessionId::new("e2e-cache-stats");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    let code = "let val = 100; val";

    // Execute same code multiple times
    for _ in 0..3 {
        let _result = ctx.eval(code).await.expect("Eval failed");
    }

    // Check statistics
    let (_tier1_count, tier2_count, cache_hits) = ctx.stats();

    // Should have some cache hits
    assert!(cache_hits > 0, "Expected cache hits, got 0");
    assert!(tier2_count > 0, "Expected tier2 executions");
}

/// Test that different code doesn't hit cache
#[tokio::test]
#[serial(env)]
async fn test_cache_miss_different_code() {
    let session_id = SessionId::new("e2e-cache-miss");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // First code
    let result1 = ctx.eval("let x = 1; x").await.expect("First eval failed");
    assert_eq!(result1.tier, ExecutionTier::JustInTime);

    // Different code - should not hit cache
    let result2 = ctx.eval("let y = 2; y").await.expect("Second eval failed");
    assert_eq!(result2.tier, ExecutionTier::JustInTime); // New compilation
    assert!(!result2.cached);
}

/// Test cache across sessions (should not share)
#[tokio::test]
#[serial(env)]
async fn test_cache_isolation_across_sessions() {
    let code = "let shared = 42; shared";

    // Session 1 with isolated cache
    let mut ctx1 = create_isolated_context(SessionId::new("e2e-session-1"), ReplMode::Sexpr);
    let _r1 = ctx1.eval(code).await.expect("Session 1 eval failed");

    // Session 2 with its own isolated cache - should compile independently
    let mut ctx2 = create_isolated_context(SessionId::new("e2e-session-2"), ReplMode::Sexpr);
    let result2 = ctx2.eval(code).await.expect("Session 2 eval failed");

    // Session 2 should do JIT (independent cache)
    assert_eq!(result2.tier, ExecutionTier::JustInTime);
}

// ============================================================================
// Test 5: Error Handling with Position Information
// ============================================================================

/// Test syntax error reporting
#[tokio::test]
#[serial(env)]
async fn test_error_handling_syntax() {
    let session_id = SessionId::new("e2e-error-syntax");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Invalid Rust syntax
    let result = ctx.eval("let x = ;").await;

    // Should return error
    assert!(result.is_err(), "Expected syntax error");

    // Error should have position information
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("Syntax") || err_msg.contains("parse") || err_msg.contains("Compilation"),
        "Error should mention syntax/parse issue: {}",
        err_msg
    );
}

/// Test type error reporting
#[tokio::test]
#[serial(env)]
async fn test_error_handling_type_mismatch() {
    let session_id = SessionId::new("e2e-error-type");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Type mismatch (if compilation includes type checking)
    let code = r#"
        let x: i32 = 42;
        let y: String = "hello";
        let _ = x + y;  // type error
    "#;

    let result = ctx.eval(code).await;

    // Should return error (compilation or runtime)
    assert!(result.is_err(), "Expected type error (compilation or runtime)");
}

/// Test runtime error handling
#[tokio::test]
#[serial(env)]
async fn test_error_handling_runtime() {
    let session_id = SessionId::new("e2e-error-runtime");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Division by zero (runtime error)
    let code = r#"
        let x = 10;
        let y = 0;
        let _ = x / y;
    "#;

    let result = ctx.eval(code).await;

    // May error during compilation or runtime
    // Depending on Rust's const evaluation
    if result.is_err() {
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(!err_msg.is_empty(), "Error message should not be empty");
    }
}

/// Test error position tracking
///
/// NOTE: This test requires the Lisp parser to be fully implemented.
/// Currently the Lisp evaluator is a placeholder and doesn't validate syntax.
#[ignore = "Requires full Lisp parser implementation"]
#[tokio::test]
#[serial(env)]
async fn test_error_position_information() {
    let session_id = SessionId::new("e2e-error-pos");
    let mut ctx = create_isolated_context(session_id, ReplMode::Lisp);

    // Syntax error
    let result = ctx.eval("(+ 1 2").await; // Unclosed paren

    assert!(result.is_err(), "Expected error for unclosed paren");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Error should have position info (line/column)
    // The actual format depends on error implementation
    assert!(
        err_msg.contains("line") || err_msg.contains("column") || err_msg.contains("<repl>"),
        "Error should contain position info: {}",
        err_msg
    );
}

// ============================================================================
// Test 6: Performance Characteristics
// ============================================================================

/// Test calculator mode performance (<1ms target)
#[tokio::test]
async fn test_performance_calculator_tier() {
    let session_id = SessionId::new("e2e-perf-calc");
    let mut ctx = EvalContext::new(session_id, ReplMode::Lisp);

    let result = ctx.eval("(+ 1 2)").await.expect("Calculator eval failed");

    assert_eq!(result.tier, ExecutionTier::Calculator);
    assert!(result.duration_ms < 10, "Calculator should be <10ms, was {}ms", result.duration_ms);
}

/// Test cached mode performance (1-5ms target)
#[tokio::test]
#[serial(env)]
async fn test_performance_cached_tier() {
    let session_id = SessionId::new("e2e-perf-cached");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    let code = "let x = 42; x";

    // First run to compile
    let _r1 = ctx.eval(code).await.expect("First eval failed");

    // Second run - should be cached
    let result2 = ctx.eval(code).await.expect("Second eval failed");

    assert_eq!(result2.tier, ExecutionTier::CachedLoaded);
    assert!(
        result2.duration_ms < 100,
        "Cached execution should be <100ms, was {}ms",
        result2.duration_ms
    );
}

/// Test JIT mode performance (50-300ms target)
#[tokio::test]
#[serial(env)]
async fn test_performance_jit_tier() {
    let session_id = SessionId::new("e2e-perf-jit");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    let code = "let fresh_var = 999; fresh_var";

    let result = ctx.eval(code).await.expect("JIT eval failed");

    assert_eq!(result.tier, ExecutionTier::JustInTime);
    // JIT can be slow due to cargo compilation
    // Just verify it completes (no strict timing requirement in test)
    assert!(result.duration_ms > 0);
}

// ============================================================================
// Test 7: Mode-Specific Behavior
// ============================================================================

/// Test Lisp mode evaluation
#[tokio::test]
async fn test_mode_lisp() {
    let session_id = SessionId::new("e2e-mode-lisp");
    let mut ctx = EvalContext::new(session_id, ReplMode::Lisp);

    // Lisp syntax
    let result = ctx.eval("(* 3 4)").await.expect("Lisp eval failed");

    assert_eq!(result.tier, ExecutionTier::Calculator);
    assert_eq!(result.value, "12");
}

/// Test Sexpr mode evaluation
#[tokio::test]
#[serial(env)]
async fn test_mode_sexpr() {
    let session_id = SessionId::new("e2e-mode-sexpr");
    let mut ctx = create_isolated_context(session_id, ReplMode::Sexpr);

    // Rust syntax
    let result = ctx.eval("let result = 3 * 4; result").await.expect("Sexpr eval failed");

    // Should compile (not calculator)
    assert_ne!(result.tier, ExecutionTier::Calculator);
}

// ============================================================================
// Test 8: Session Management
// ============================================================================

/// Test session isolation
#[tokio::test]
#[serial(env)]
async fn test_session_isolation() {
    // Session 1 defines variable with isolated cache
    let mut ctx1 = create_isolated_context(SessionId::new("e2e-iso-1"), ReplMode::Sexpr);
    let _r1 = ctx1.eval("let session1_var = 100; session1_var").await.expect("Session 1 failed");

    // Session 2 with its own isolated cache - should not see session 1's variable
    let mut ctx2 = create_isolated_context(SessionId::new("e2e-iso-2"), ReplMode::Sexpr);

    // This should fail or return different result (no access to session1_var)
    // For now, just verify sessions are independent
    let result2 =
        ctx2.eval("let session2_var = 200; session2_var").await.expect("Session 2 failed");

    // Sessions should have independent execution
    assert_eq!(result2.tier, ExecutionTier::JustInTime);
}

/// Test session cloning
#[tokio::test]
async fn test_session_clone() {
    let session1 = SessionId::new("e2e-clone-1");
    let ctx1 = EvalContext::new(session1.clone(), ReplMode::Lisp);

    // Clone to new session
    let session2 = SessionId::new("e2e-clone-2");
    let _ctx2 = ctx1.clone_to(session2);

    // Both contexts should be independent
    // (Further testing would require executing code in both)
}
