//! Oxur Compiler
//!
//! Handles the backend of the Oxur compilation pipeline:
//! - Stage 3: Lower (Core Forms → Rust AST)
//! - Stage 4: Generate (Rust AST → Rust source)
//! - Stage 5: Compile (Rust source → Binary via rustc)

pub mod codegen;
pub mod compiler;
pub mod lowering;
pub mod rustc_diagnostic;

pub use codegen::CodeGenerator;
pub use compiler::Compiler;
pub use lowering::Lowerer;
pub use rustc_diagnostic::{RustcDiagnostic, RustcSpan};

// Re-export oxur-smap types for convenience
pub use oxur_smap::{new_node_id, NodeId, SourceMap, SourcePos};

/// Result type for compilation operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for compilation
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lowering error: {0}")]
    Lowering(String),

    #[error("Code generation error: {0}")]
    CodeGen(String),

    #[error("Compilation error: {0}")]
    Compile(String),

    #[error("Language error: {0}")]
    Language(#[from] oxur_lang::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
