//! Oxur REPL
//!
//! Provides a Read-Eval-Print-Loop with three-tier execution:
//! - Tier 1: Direct interpretation for simple expressions
//! - Tier 2: Cached compiled functions
//! - Tier 3: JIT compilation for complex code

pub mod protocol;
pub mod client;
pub mod server;

pub use protocol::{ReplRequest, ReplResponse};
pub use client::ReplClient;
pub use server::ReplServer;

/// Result type for REPL operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for REPL
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Evaluation error: {0}")]
    Eval(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Language error: {0}")]
    Language(#[from] oxur_lang::Error),

    #[error("Compilation error: {0}")]
    Compile(#[from] oxur_comp::Error),
}
