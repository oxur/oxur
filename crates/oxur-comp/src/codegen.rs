//! Stage 4: Generate
//!
//! Converts Rust AST into formatted Rust source code.

use crate::{Result};

/// Code generator produces formatted Rust source
pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate formatted Rust source from AST
    pub fn generate(&self, file: &syn::File) -> Result<String> {
        // Use prettyplease for formatting
        Ok(prettyplease::unparse(file))
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_empty_file() {
        let gen = CodeGenerator::new();
        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![],
        };
        let result = gen.generate(&file);
        assert!(result.is_ok());
    }
}
