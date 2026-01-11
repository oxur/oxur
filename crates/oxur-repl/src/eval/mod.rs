// Evaluation layer for REPL
//
// Manages session state, tiered execution, and code compilation.

mod context;
mod lisp_mode;
pub mod output_capture;
mod sexpr_mode;
pub mod stats;

// Re-export public types
pub use context::{EvalContext, EvalError, EvalResult, ExecutionTier, Result};
pub use lisp_mode::{LispEvaluator, TypedValue};
pub use output_capture::{CapturedOutput, OutputCapturer};
pub use sexpr_mode::SexprEvaluator;

// Re-export metrics types (EvalMetrics replaces StatsCollector)
pub use crate::metrics::{CacheStats, EvalMetrics, Percentiles};

// Re-export resource stats (not duplicated in metrics)
pub use stats::{get_resource_stats, ResourceStats};

// Backwards compatibility alias
#[deprecated(since = "0.2.0", note = "Use EvalMetrics instead")]
pub type StatsCollector = EvalMetrics;
