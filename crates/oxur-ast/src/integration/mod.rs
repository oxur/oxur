//! Integration with Rust's standard AST (via syn)

mod from_syn;

pub use from_syn::*;

use crate::ast::Crate;
use crate::error::Result;

/// Parse Rust source code into our AST
pub fn parse_rust_file(source: &str) -> Result<Crate> {
    let syn_file = syn::parse_file(source).map_err(|e| crate::error::ParseError::Expected {
        expected: "valid Rust code".to_string(),
        found: format!("parse error: {}", e),
        pos: crate::error::Position::new(0, 1, 1),
    })?;

    from_syn_file(&syn_file)
}
