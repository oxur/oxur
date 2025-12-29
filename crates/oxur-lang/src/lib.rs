//! Oxur Language Processing
//!
//! This crate handles the frontend of the Oxur compilation pipeline:
//! - Stage 1: Parse (Oxur syntax → Surface Forms)
//! - Stage 2: Expand (Surface Forms → Core Forms)
//!
//! Core Forms are the stable intermediate representation (IR) that serves
//! as the contract between the frontend (oxur-lang) and backend (oxur-comp).

pub mod parser;
pub mod expander;
pub mod core_forms;
pub mod source_map;

pub use core_forms::{CoreForm, NodeId};
pub use parser::Parser;
pub use expander::Expander;
pub use source_map::SourceMap;

/// Result type for language operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for language processing
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error at {location}: {message}")]
    Parse {
        message: String,
        location: Location,
    },

    #[error("Expansion error at node {node_id}: {message}")]
    Expand {
        message: String,
        node_id: NodeId,
    },

    #[error("Invalid syntax: {0}")]
    Syntax(String),
}

/// Source location for error reporting
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
