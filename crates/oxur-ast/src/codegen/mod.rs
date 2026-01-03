//! Rust code generation from Oxur AST
//!
//! This module provides functionality to generate Rust source code from
//! Oxur's internal AST representation.
//!
//! # Overview
//!
//! The code generation process takes an Oxur `Crate` AST and produces
//! valid Rust source code as a String. The generated code maintains
//! proper indentation, formatting, and Rust syntax.
//!
//! # Example
//!
//! ```no_run
//! use oxur_ast::ast::*;
//! use oxur_ast::codegen::generate_rust;
//!
//! # fn example() -> anyhow::Result<()> {
//! // Given an Oxur AST crate node...
//! let crate_node: Crate = todo!();
//!
//! // Generate Rust source code
//! let rust_code = generate_rust(&crate_node)?;
//!
//! // The output is valid Rust code that can be compiled
//! println!("{}", rust_code);
//! # Ok(())
//! # }
//! ```

mod expr;
mod generics;
mod item;
mod rust;
mod types;

pub use rust::RustCodegen;

use crate::ast::Crate;
use anyhow::Result;

/// Generate Rust source code from an Oxur AST Crate node
///
/// This is the main entry point for code generation. It creates a
/// `RustCodegen` instance and uses it to generate formatted Rust code.
///
/// # Arguments
///
/// * `crate_node` - The root Crate AST node to generate code from
///
/// # Returns
///
/// Returns a `String` containing valid Rust source code, or an error
/// if code generation fails.
///
/// # Example
///
/// ```no_run
/// use oxur_ast::codegen::generate_rust;
/// use oxur_ast::ast::Crate;
///
/// # fn example(crate_node: Crate) -> anyhow::Result<()> {
/// let rust_code = generate_rust(&crate_node)?;
/// println!("{}", rust_code);
/// # Ok(())
/// # }
/// ```
pub fn generate_rust(crate_node: &Crate) -> Result<String> {
    let mut codegen = RustCodegen::new();
    codegen.generate_crate(crate_node)
}
