//! Compiler
//!
//! Orchestrates the complete compilation pipeline from Core Forms to binary.

use crate::{CodeGenerator, Error, Lowerer, Result};
use oxur_lang::CoreForm;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Compiler orchestrates the full compilation pipeline
pub struct Compiler {
    lowerer: Lowerer,
    codegen: CodeGenerator,
    output_dir: PathBuf,
}

impl Compiler {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            lowerer: Lowerer::new(),
            codegen: CodeGenerator::new(),
            output_dir,
        }
    }

    /// Compile Core Forms to a binary
    pub fn compile(&mut self, forms: Vec<CoreForm>, output: &Path) -> Result<()> {
        // Stage 3: Lower to Rust AST
        let ast = self.lowerer.lower(forms)?;

        // Stage 4: Generate Rust source
        let source = self.codegen.generate(&ast)?;

        // Write to temporary .rs file
        let rs_file = self.output_dir.join("generated.rs");
        std::fs::write(&rs_file, source)?;

        // Stage 5: Compile with rustc
        self.compile_with_rustc(&rs_file, output)?;

        Ok(())
    }

    fn compile_with_rustc(&self, source: &Path, output: &Path) -> Result<()> {
        let status = Command::new("rustc")
            .arg(source)
            .arg("-o")
            .arg(output)
            .status()?;

        if !status.success() {
            return Err(Error::Compile(format!(
                "rustc failed with exit code: {:?}",
                status.code()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new(PathBuf::from("/tmp"));
        assert_eq!(compiler.output_dir, PathBuf::from("/tmp"));
    }
}
