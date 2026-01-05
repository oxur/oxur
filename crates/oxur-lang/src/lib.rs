//! Oxur Language Processing
//!
//! This crate handles the frontend of the Oxur compilation pipeline:
//! - Stage 1: Parse (Oxur syntax → Surface Forms)
//! - Stage 2: Expand (Surface Forms → Core Forms)
//!
//! Core Forms are the stable intermediate representation (IR) that serves
//! as the contract between the frontend (oxur-lang) and backend (oxur-comp).

pub mod core_forms;
pub mod expander;
pub mod parser;

pub use core_forms::{CoreForm, NodeId};
pub use expander::Expander;
pub use parser::Parser;

// Re-export oxur-smap types for convenience
pub use oxur_smap::{new_node_id, SourceMap, SourcePos};

/// Result type for language operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for language processing
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error at {location}: {message}")]
    Parse { message: String, location: Location },

    #[error("Expansion error at node {node_id}: {message}")]
    Expand { message: String, node_id: NodeId },

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_display() {
        let loc = Location { line: 10, column: 25 };
        assert_eq!(format!("{}", loc), "10:25");
    }

    #[test]
    fn test_parse_error_display() {
        let err = Error::Parse {
            message: "unexpected token".to_string(),
            location: Location { line: 1, column: 5 },
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Parse error"));
        assert!(msg.contains("1:5"));
        assert!(msg.contains("unexpected token"));
    }

    #[test]
    fn test_expand_error_display() {
        let err = Error::Expand {
            message: "macro expansion failed".to_string(),
            node_id: NodeId::from_raw(42),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Expansion error"));
        assert!(msg.contains("NodeId(42)"));
        assert!(msg.contains("macro expansion failed"));
    }

    #[test]
    fn test_syntax_error_display() {
        let err = Error::Syntax("invalid syntax".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid syntax"));
        assert!(msg.contains("invalid syntax"));
    }
}
