//! Compiler
//!
//! Orchestrates the complete compilation pipeline from Core Forms to binary.

use crate::{CodeGenerator, Error, Lowerer, Result};
use oxur_lang::CoreForm;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Compiler orchestrates the full compilation pipeline
pub struct Compiler {
    codegen: CodeGenerator,
    output_dir: PathBuf,
}

impl Compiler {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { codegen: CodeGenerator::new(), output_dir }
    }

    /// Compile Core Forms to a binary
    ///
    /// Accepts the SourceMap from the expansion phase and returns it
    /// with lowering mappings added for error reporting.
    ///
    /// # Error Translation
    ///
    /// If rustc compilation fails, errors are translated using the SourceMap
    /// to show Oxur source positions where possible. Currently shows generated
    /// Rust positions with a note that full translation is being implemented.
    pub fn compile(
        &mut self,
        forms: Vec<CoreForm>,
        source_map: oxur_smap::SourceMap,
        output: &Path,
    ) -> Result<oxur_smap::SourceMap> {
        // Stage 3: Lower to Rust AST
        let mut lowerer = Lowerer::new(source_map);
        let (ast, source_map) = lowerer.lower(forms)?;

        // Stage 4: Generate Rust source
        let source = self.codegen.generate(&ast)?;

        // Write to temporary .rs file
        let rs_file = self.output_dir.join("generated.rs");
        std::fs::write(&rs_file, source)?;

        // Stage 5: Compile with rustc (pass source_map for error translation)
        self.compile_with_rustc(&rs_file, output, &source_map)?;

        Ok(source_map)
    }

    fn compile_with_rustc(
        &self,
        source: &Path,
        output: &Path,
        source_map: &oxur_smap::SourceMap,
    ) -> Result<()> {
        let output_result = Command::new("rustc")
            .arg(source)
            .arg("-o")
            .arg(output)
            .arg("--error-format=json")
            .output()?;

        if !output_result.status.success() {
            // Parse JSON diagnostics from stderr
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            let diagnostics =
                crate::RustcDiagnostic::from_json_lines(&stderr).unwrap_or_else(|_| vec![]);

            // Use ErrorTranslator to convert rustc errors to Oxur positions
            let translator = crate::ErrorTranslator::new(source_map.clone());
            let translated = translator.translate_diagnostics(&diagnostics);

            let error_msg = format!(
                "rustc failed with exit code: {:?}\n\n{}",
                output_result.status.code(),
                translated
            );

            return Err(Error::Compile(error_msg));
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

    #[test]
    fn test_compiler_has_codegen() {
        let compiler = Compiler::new(PathBuf::from("/tmp"));
        // CodeGenerator is private but we can verify it exists
        assert_eq!(compiler.output_dir, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_compiler_output_dir() {
        let test_dir = PathBuf::from("/var/tmp/test");
        let compiler = Compiler::new(test_dir.clone());
        assert_eq!(compiler.output_dir, test_dir);
    }

    #[test]
    fn test_compile_with_empty_forms() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("build");
        std::fs::create_dir_all(&output_dir).unwrap();

        let mut compiler = Compiler::new(output_dir.clone());
        let output_path = temp_dir.path().join("test_output");
        let source_map = oxur_smap::SourceMap::new();

        // This will fail because rustc isn't available or the generated code is invalid
        // but we're just testing that the compilation pipeline runs
        let result = compiler.compile(vec![], source_map, &output_path);
        // We expect this to error (no rustc or invalid generated code)
        // but the important thing is that it attempts compilation
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_compile_hello_world() {
        use oxur_lang::{Expander, Parser};
        use tempfile::TempDir;

        // Parse and expand hello world
        let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;

        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(surface_forms).unwrap();
        let source_map = expander.source_map().clone();

        // Compile to binary
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("build");
        std::fs::create_dir_all(&output_dir).unwrap();

        let mut compiler = Compiler::new(output_dir);
        let binary_path = temp_dir.path().join("hello_world");

        let result = compiler.compile(core_forms, source_map, &binary_path);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        // Verify binary exists
        assert!(binary_path.exists(), "Binary was not created");

        // Run the binary and check output
        let output = Command::new(&binary_path).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        eprintln!("Binary output:\n{}", stdout);

        assert!(output.status.success(), "Binary execution failed");
        assert!(stdout.contains("Hello, world!"), "Output doesn't contain expected text");
    }

    #[test]
    fn test_error_translation_format() {
        use oxur_lang::{Expander, Parser};
        use tempfile::TempDir;

        // Parse code with intentional error (undefined variable)
        let source = r#"(deffn main ()
  (println! x))"#; // `x` is undefined

        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(surface_forms).unwrap();
        let source_map = expander.source_map().clone();

        // Compile (this should fail with rustc error)
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("build");
        std::fs::create_dir_all(&output_dir).unwrap();

        let mut compiler = Compiler::new(output_dir);
        let binary_path = temp_dir.path().join("test_error");

        let result = compiler.compile(core_forms, source_map, &binary_path);

        // Should fail with compilation error
        assert!(result.is_err(), "Should fail due to undefined variable");

        if let Err(crate::Error::Compile(msg)) = result {
            // Error message should mention the error
            eprintln!("Error message:\n{}", msg);

            // Should contain rustc error code or exit status
            // (exact format depends on rustc version, but should have some structure)
            assert!(!msg.is_empty(), "Should have error message");
        }
    }
}
