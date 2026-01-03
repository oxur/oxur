// Evaluation layer for REPL
//
// Manages session state, tiered execution, and code compilation.

mod context;
mod lisp_mode;

// Re-export public types
pub use context::{EvalContext, EvalError, EvalResult, ExecutionTier, Result};
pub use lisp_mode::LispEvaluator;
