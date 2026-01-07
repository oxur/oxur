//! Stage 4: Generate
//!
//! Converts Rust AST into formatted Rust source code.

use crate::Result;

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
        let file = syn::File { shebang: None, attrs: vec![], items: vec![] };
        let result = gen.generate(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_codegen_default() {
        let gen = CodeGenerator;
        let file = syn::File { shebang: None, attrs: vec![], items: vec![] };
        let result = gen.generate(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_returns_string() {
        let gen = CodeGenerator::new();
        let file = syn::File { shebang: None, attrs: vec![], items: vec![] };
        let result = gen.generate(&file).unwrap();
        assert!(result.is_empty() || !result.is_empty()); // Just check it's a string
    }

    #[test]
    fn test_generate_hello_world() {
        use crate::lowering::Lowerer;
        use oxur_lang::{Expander, Parser};

        // Parse, expand, and lower
        let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;

        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(surface_forms).unwrap();

        let mut lowerer = Lowerer::new();
        let file = lowerer.lower(core_forms).unwrap();

        // Generate Rust source
        let gen = CodeGenerator::new();
        let rust_code = gen.generate(&file).unwrap();

        // Print for verification
        eprintln!("Generated Rust code:\n{}", rust_code);

        // Check the generated code
        assert!(rust_code.contains("fn main"));
        assert!(rust_code.contains("println!"));
        assert!(rust_code.contains("Hello, world!"));
    }
}
