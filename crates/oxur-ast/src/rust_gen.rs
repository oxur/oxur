//! Rust code generation from syn AST
//!
//! This module provides two code generation strategies:
//!
//! 1. **Fast generation** (`gen_rust`) - Uses quote's ToTokens for rapid output (~50-100ms for 100 files)
//!    - Best for: REPL, debugging, internal tooling, rapid iteration
//!    - Output: Valid Rust code, minimal formatting
//!
//! 2. **Pretty generation** (`gen_rust_pretty`) - Adds prettyplease formatting (~400-500ms for 100 files)
//!    - Best for: Production code, publishing, human review
//!    - Output: Beautifully formatted, idiomatic Rust
//!
//! ## Pipeline Comparison
//!
//! **Fast path:**
//! ```text
//! syn AST → ToTokens → TokenStream → .to_string() → rustc
//! ```
//!
//! **Pretty path:**
//! ```text
//! syn AST → ToTokens → TokenStream → .to_string() → syn::parse_file() → prettyplease::unparse() → rustc
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use oxur_ast::rust_gen::{gen_rust, gen_rust_pretty};
//!
//! # fn example(syn_file: syn::File) -> anyhow::Result<()> {
//! // Fast generation for REPL/debugging
//! let code = gen_rust(&syn_file)?;
//!
//! // Pretty generation for production
//! let pretty_code = gen_rust_pretty(&syn_file)?;
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use quote::ToTokens;

/// Generate Rust source code from syn AST (fast path)
///
/// Uses `quote::ToTokens` to generate valid Rust code quickly.
/// Output is functional but not prettified.
///
/// **Performance:** ~50-100ms for 100 files
///
/// **Use cases:**
/// - REPL evaluation
/// - Debugging and development
/// - Internal tooling
/// - Rapid iteration workflows
///
/// # Arguments
///
/// * `syn_ast` - Any syn AST node that implements ToTokens (File, Item, Expr, etc.)
///
/// # Returns
///
/// Returns a String containing valid Rust source code.
///
/// # Example
///
/// ```no_run
/// use oxur_ast::rust_gen::gen_rust;
/// use syn::parse_quote;
///
/// # fn example() -> anyhow::Result<()> {
/// let item: syn::Item = parse_quote! {
///     fn add(a: i32, b: i32) -> i32 { a + b }
/// };
///
/// let code = gen_rust(&item)?;
/// println!("{}", code);
/// # Ok(())
/// # }
/// ```
pub fn gen_rust<T: ToTokens>(syn_ast: &T) -> Result<String> {
    let tokens = syn_ast.to_token_stream();
    Ok(tokens.to_string())
}

/// Generate formatted Rust source code from syn AST (pretty path)
///
/// Uses `quote::ToTokens` followed by `prettyplease::unparse()` to generate
/// beautifully formatted, idiomatic Rust code.
///
/// **Performance:** ~400-500ms for 100 files (5x slower than fast path)
///
/// **Use cases:**
/// - Final production code generation
/// - Code intended for human review
/// - Publishing/distribution
/// - When formatting quality matters
///
/// # Arguments
///
/// * `syn_file` - A syn::File AST node
///
/// # Returns
///
/// Returns a String containing beautifully formatted Rust source code.
///
/// # Example
///
/// ```no_run
/// use oxur_ast::rust_gen::gen_rust_pretty;
/// use syn::parse_quote;
///
/// # fn example() -> anyhow::Result<()> {
/// let file: syn::File = parse_quote! {
///     fn add(a: i32, b: i32) -> i32 { a + b }
/// };
///
/// let pretty_code = gen_rust_pretty(&file)?;
/// println!("{}", pretty_code);
/// # Ok(())
/// # }
/// ```
pub fn gen_rust_pretty(syn_file: &syn::File) -> Result<String> {
    // Fast generation first
    let code = gen_rust(syn_file)?;

    // Parse back to syn (required by prettyplease)
    let parsed = syn::parse_file(&code)?;

    // Pretty print
    let pretty = prettyplease::unparse(&parsed);

    Ok(pretty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_gen_rust_simple_fn() {
        let item: syn::Item = parse_quote! {
            fn add(a: i32, b: i32) -> i32 { a + b }
        };

        let code = gen_rust(&item).unwrap();
        assert!(code.contains("fn add"));
        assert!(code.contains("a: i32"));
        assert!(code.contains("b: i32"));
    }

    #[test]
    fn test_gen_rust_pretty_simple_fn() {
        let file: syn::File = parse_quote! {
            fn add(a: i32, b: i32) -> i32 { a + b }
        };

        let pretty_code = gen_rust_pretty(&file).unwrap();
        assert!(pretty_code.contains("fn add"));
        assert!(pretty_code.contains("a: i32"));
        assert!(pretty_code.contains("b: i32"));
        // Pretty version should have proper formatting
        assert!(pretty_code.contains("\n"));
    }

    #[test]
    fn test_gen_rust_struct() {
        let item: syn::Item = parse_quote! {
            pub struct Point { pub x: i32, pub y: i32 }
        };

        let code = gen_rust(&item).unwrap();
        assert!(code.contains("pub struct Point"));
        assert!(code.contains("pub x"));
        assert!(code.contains("pub y"));
    }

    #[test]
    fn test_gen_rust_pretty_preserves_semantics() {
        let file: syn::File = parse_quote! {
            fn multiply(x: i32, y: i32) -> i32 {
                x * y
            }
        };

        let pretty_code = gen_rust_pretty(&file).unwrap();

        // Should parse back successfully
        let reparsed = syn::parse_file(&pretty_code);
        assert!(reparsed.is_ok());
    }
}
