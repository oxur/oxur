//! Oxur AST: Rust AST manipulation via S-expressions
//!
//! This crate provides bidirectional conversion between Rust source code
//! and S-expression representations, enabling programmatic AST manipulation.
#![doc = include_str!("../README.md")]

pub mod ast;
pub mod builder;
pub mod codegen;
pub mod commands;
pub mod error;
pub mod generator;
pub mod integration;
pub mod sexp;

// Re-export commonly used items
pub use ast::Crate;
pub use builder::AstBuilder;
pub use codegen::generate_rust;
pub use error::{LexError, ParseError, Position, Result};
pub use generator::Generator;
pub use integration::parse_rust_file;
pub use sexp::{print_sexp, Parser, Printer, SExp};
