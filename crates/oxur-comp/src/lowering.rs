//! Stage 3: Lower
//!
//! Converts Core Forms into Rust AST using the syn crate.

use crate::{Result};
use oxur_lang::{CoreForm, NodeId};
use std::collections::HashMap;

/// Lowerer converts Core Forms to Rust AST
pub struct Lowerer {
    node_map: HashMap<NodeId, syn::Expr>,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            node_map: HashMap::new(),
        }
    }

    /// Lower Core Forms to Rust AST
    pub fn lower(&mut self, _forms: Vec<CoreForm>) -> Result<syn::File> {
        // Placeholder implementation
        Ok(syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![],
        })
    }
}

impl Default for Lowerer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowerer_creation() {
        let lowerer = Lowerer::new();
        assert_eq!(lowerer.node_map.len(), 0);
    }
}
