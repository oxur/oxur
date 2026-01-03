//! Oxur REPL
//!
//! Provides a Read-Eval-Print-Loop with three-tier execution:
//! - Tier 1: Calculator mode (interpret literals only)
//! - Tier 2: Cached compilation (compile and cache everything else)
//!
//! Based on ODD-0018: Oxur Remote REPL Protocol Design

pub mod protocol;

// Old client/server modules (will be replaced in Phases 2-3)
// mod client;
// mod server;

// pub use client::ReplClient;
// pub use server::ReplServer;

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
