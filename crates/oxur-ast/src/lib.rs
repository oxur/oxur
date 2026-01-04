//! Oxur AST: Rust AST manipulation via S-expressions
//!
//! This crate provides bidirectional conversion between Rust source code
//! and S-expression representations, enabling programmatic AST manipulation.
#![doc = include_str!("../README.md")]

pub mod ast;
pub mod builder;
pub mod commands;
pub mod error;
pub mod gen_rs;
pub mod gen_sexp;
pub mod integration;
pub mod sexp;

// Re-export commonly used items
pub use ast::Crate;
pub use builder::AstBuilder;
pub use error::{LexError, ParseError, Position, Result};
pub use gen_rs::generate_rust;
pub use gen_sexp::Generator;
pub use integration::parse_rust_file;
pub use sexp::{print_sexp, Parser, Printer, SExp};
