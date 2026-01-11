//! Test hybrid execution: def variables accessible in compiled code

use oxur_repl::eval::EvalContext;
use oxur_repl::protocol::{ReplMode, SessionId};

#[tokio::test]
async fn test_def_variable_in_calculator() {
    let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

    // Define a variable
    let result = ctx.eval("(def x:i64 42)").await.unwrap();
    assert_eq!(result.value, "42");

    // Use it in calculator mode
    let result = ctx.eval("(+ x 8)").await.unwrap();
    assert_eq!(result.value, "50");
}

#[tokio::test]
async fn test_def_variable_in_compiled_code() {
    let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

    // Define a variable
    let result = ctx.eval("(def base:i64 5)").await.unwrap();
    assert_eq!(result.value, "5");

    // EXPERIMENT: Try to use it in compiled Rust code (method call with Oxur syntax)
    // Correct syntax is (receiver:method args...) not receiver.method(args)
    let result = ctx.eval("(base:pow 3)").await;

    match result {
        Ok(res) => {
            println!("SUCCESS! Hybrid mode works!");
            println!("Result: {}", res.value);
            println!("Tier: {}", res.tier.display_name());
            // 5^3 = 125
            assert_eq!(res.value, "125");
        }
        Err(e) => {
            println!("ROUGH EDGE FOUND:");
            println!("Error: {}", e);
            // Document this rough edge - we expected it might not work!
            panic!("Hybrid mode failed (this is expected - documenting the rough edge): {}", e);
        }
    }
}
