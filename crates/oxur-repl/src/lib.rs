//! Oxur REPL
//!
//! Provides a Read-Eval-Print-Loop with two-tier execution:
//! - Tier 1: Calculator mode (interpret simple arithmetic only)
//! - Tier 2: Cached compilation (compile and cache everything else)
//!
//! Based on ODD-0018: Oxur Remote REPL Protocol Design
//! and ODD-0026: Oxur REPL Evaluation Strategy

pub mod eval;
pub mod protocol;
pub mod server;
pub mod transport;

// Stub client for backwards compatibility with oxur-cli
mod client;
pub use client::ReplClient;

// Re-export oxur-smap types for convenience
pub use oxur_smap::{new_node_id, NodeId, SourceMap, SourcePos};

/// Result type for REPL operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for REPL
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Evaluation error: {0}")]
    Eval(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Codec error: {0}")]
    Codec(#[from] protocol::CodecError),

    #[error("Language error: {0}")]
    Language(#[from] oxur_lang::Error),

    #[error("Compilation error: {0}")]
    Compile(#[from] oxur_comp::Error),
}
