//! Code execution infrastructure
//!
//! Manages subprocess-based code execution with isolation,
//! panic catching, and output capture.

mod subprocess;

pub use subprocess::{ExecutionResult, SubprocessExecutor};
