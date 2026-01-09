//! Oxur: A Lisp dialect that treats Rust as its compilation target and runtime
//!
//! This is the umbrella crate for the Oxur project, providing convenient access
//! to all major components of the Oxur ecosystem.
//!
//! # Overview
//!
//! Oxur is a Lisp dialect designed to leverage Rust's type system, ownership model,
//! and ecosystem while providing Lisp's expressiveness and metaprogramming capabilities.
//!
//! # Architecture
//!
//! The Oxur compilation pipeline consists of five stages:
//!
//! 1. **Parse** → Surface Forms (S-expressions)
//! 2. **Expand** → Core Forms (Canonical IR)
//! 3. **Lower** → Rust AST
//! 4. **Codegen** → Rust Source Code
//! 5. **Compile** → Binary (via rustc)
//!
//! # Crates
//!
//! This umbrella crate re-exports the following components:
//!
//! - **`oxur_lang`**: The Oxur Lisp compiler (stages 1-2)
//! - **`oxur_comp`**: The backend compiler (stages 3-5)
//! - **`oxur_ast`**: Bidirectional Rust AST ↔ S-expression conversion
//! - **`oxur_repl`**: Interactive REPL server/client
//! - **`oxur_cli`**: Common CLI utilities and infrastructure
//! - **`oxur_pretty`**: Pretty printing for code and data structures
//!
//! # Quick Start
//!
//! ```rust
//! // Example usage will be added as the API stabilizes
//! use oxur::*;
//! ```
//!
//! # Design Philosophy
//!
//! 1. **100% Rust Interoperability** - Can call any Rust code, Rust can call any Oxur code
//! 2. **Rust Semantics, Lisp Syntax** - Not Lisp semantics adapted to Rust
//! 3. **Canonical S-expressions** - Single authoritative format for AST representation
//! 4. **Round-trip Preservation** - X → transform → X must preserve meaning
//! 5. **Type-First Design** - Leverage Rust's type system fully
//!
//! # Project Status
//!
//! Oxur is in active development. Different components are at various stages:
//!
//! - **oxur-ast**: ~80% complete, functional conversion
//! - **oxur-lang**: Planning stage
//! - **oxur-comp**: Planning stage
//! - **oxur-repl**: Planning stage
//! - **oxur-cli**: Early stage, utilities available
//!
//! # Resources
//!
//! - [Repository](https://github.com/oxur/oxur)
//! - [Documentation](https://docs.rs/oxur)
//! - [Design Documents](https://github.com/oxur/oxur/tree/main/crates/oxur-odm/docs)
//!
//! # License
//!
//! Licensed under either of Apache License, Version 2.0 or MIT license at your option.

#![warn(missing_docs)]
#![warn(clippy::all)]

// Re-export main components
pub use oxur_ast;
pub use oxur_cli;
pub use oxur_comp;
pub use oxur_lang;
pub use oxur_pretty;
pub use oxur_repl;

/// The version of the Oxur umbrella crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the Oxur project
pub const NAME: &str = env!("CARGO_PKG_NAME");
