// Evaluation layer for REPL
//
// Manages session state, tiered execution, and code compilation.

mod context;

// Re-export public types
pub use context::{EvalContext, EvalError, EvalResult, ExecutionTier};
