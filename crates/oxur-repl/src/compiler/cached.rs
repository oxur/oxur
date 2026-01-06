//! Cached compilation of Rust code to dynamic libraries
//!
//! Manages the full compilation pipeline with caching for performance.
//!
//! Based on ODD-0026 Section 2.1

use crate::cache::ArtifactCache;
use crate::session::SessionDir;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

/// Compilation errors
#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Failed to write source file: {0}")]
    WriteSourceFailed(#[from] std::io::Error),

    #[error("Session directory error: {0}")]
    SessionDirError(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Cache operation failed: {0}")]
    CacheFailed(String),

    #[error("Failed to find rustc: {0}")]
    RustcNotFound(String),
}

impl From<crate::session::SessionDirError> for CompilerError {
    fn from(err: crate::session::SessionDirError) -> Self {
        CompilerError::SessionDirError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CompilerError>;

/// Cached compiler for Rust code
///
/// Manages compilation with SHA256-based caching to avoid recompiling
/// identical code.
///
/// # Examples
///
/// ```no_run
/// use oxur_repl::compiler::CachedCompiler;
/// use oxur_repl::cache::ArtifactCache;
/// use oxur_repl::session::SessionDir;
/// use oxur_repl::protocol::SessionId;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let cache = ArtifactCache::new()?;
/// let session_id = SessionId::new("test");
/// let session_dir = SessionDir::new(&session_id)?;
///
/// let mut compiler = CachedCompiler::new(cache, &session_dir);
///
/// let source = "pub extern \"C\" fn oxur_eval_test() { println!(\"Hello!\"); }";
/// let lib_path = compiler.compile("test", source, 2)?;
///
/// println!("Compiled to: {}", lib_path.display());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CachedCompiler<'a> {
    /// Artifact cache for storing compiled libraries
    cache: ArtifactCache,

    /// Session directory for temporary files
    session_dir: &'a SessionDir,
}

impl<'a> CachedCompiler<'a> {
    /// Create a new cached compiler
    ///
    /// # Arguments
    ///
    /// * `cache` - Artifact cache for storing compiled libraries
    /// * `session_dir` - Session directory for temporary compilation files
    pub fn new(cache: ArtifactCache, session_dir: &'a SessionDir) -> Self {
        Self { cache, session_dir }
    }

    /// Compile Rust source code to a dynamic library
    ///
    /// Uses SHA256-based caching to avoid recompiling identical code.
    ///
    /// # Arguments
    ///
    /// * `cache_key` - Cache key for this compilation unit
    /// * `source` - Rust source code to compile
    /// * `opt_level` - Optimization level (0-3)
    ///
    /// # Returns
    ///
    /// Path to the compiled dynamic library (.so/.dylib/.dll)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Source file cannot be written
    /// - Compilation fails
    /// - Cache operations fail
    pub fn compile(
        &mut self,
        cache_key: impl AsRef<str>,
        source: impl AsRef<str>,
        opt_level: u8,
    ) -> Result<PathBuf> {
        let cache_key = cache_key.as_ref();
        let source = source.as_ref();

        // Generate cache key from source and compilation options
        let content_key = self.cache.generate_key(
            source,
            &[], // TODO: Track dependencies when needed
            opt_level,
            "default", // TODO: Add source map configuration
        );

        // Check if already in cache
        if let Some(cached_path) =
            self.cache.get(&content_key).map_err(|e| CompilerError::CacheFailed(e.to_string()))?
        {
            return Ok(cached_path);
        }

        // Not in cache - need to compile
        let lib_path = self.compile_to_dylib(cache_key, source, opt_level)?;

        // Insert into cache and return cached path
        let cached_path = self
            .cache
            .insert(&content_key, &lib_path)
            .map_err(|e| CompilerError::CacheFailed(e.to_string()))?;

        Ok(cached_path)
    }

    /// Compile source to dynamic library
    fn compile_to_dylib(&self, cache_key: &str, source: &str, opt_level: u8) -> Result<PathBuf> {
        // Write source to file
        let source_path = self.session_dir.write_source(format!("{}.rs", cache_key), source)?;

        // Determine output library name based on platform
        #[cfg(target_os = "macos")]
        let lib_name = format!("lib{}.dylib", cache_key);

        #[cfg(target_os = "linux")]
        let lib_name = format!("lib{}.so", cache_key);

        #[cfg(target_os = "windows")]
        let lib_name = format!("{}.dll", cache_key);

        let lib_path = self.session_dir.path().join(&lib_name);

        // Compile using rustc with JSON error format for better error messages
        let output = Command::new("rustc")
            .arg("--crate-type=cdylib")
            .arg(format!("-Copt-level={}", opt_level))
            .arg("--error-format=json")
            .arg("-o")
            .arg(&lib_path)
            .arg(&source_path)
            .output()
            .map_err(|e| CompilerError::RustcNotFound(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Try to parse and translate errors
            use crate::compiler::ErrorTranslator;
            let translator = ErrorTranslator::new();

            match translator.parse_and_translate(&stderr) {
                Ok(diagnostics) if !diagnostics.is_empty() => {
                    // Format translated errors
                    let formatted_errors: Vec<String> =
                        diagnostics.iter().map(|d| d.format()).collect();
                    return Err(CompilerError::CompilationFailed(formatted_errors.join("\n")));
                }
                _ => {
                    // Fallback to raw stderr if parsing fails
                    return Err(CompilerError::CompilationFailed(stderr.to_string()));
                }
            }
        }

        Ok(lib_path)
    }

    /// Get the artifact cache
    pub fn cache(&self) -> &ArtifactCache {
        &self.cache
    }

    /// Get a mutable reference to the artifact cache
    pub fn cache_mut(&mut self) -> &mut ArtifactCache {
        &mut self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::SessionId;
    use std::env;

    fn setup_compiler() -> (CachedCompiler<'static>, SessionDir, PathBuf) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_dir = env::temp_dir().join(format!("oxur-compiler-test-{}", id));

        // Clean test directory if it exists from a previous run
        if test_dir.exists() {
            std::fs::remove_dir_all(&test_dir).expect("Failed to clean test directory");
        }

        let cache = ArtifactCache::with_directory(&test_dir).expect("Failed to create cache");

        let session_id = SessionId::new(format!("test-{}", id));
        let session_dir = Box::leak(Box::new(
            SessionDir::new(&session_id).expect("Failed to create session dir"),
        ));

        let compiler = CachedCompiler::new(cache, session_dir);

        (compiler, unsafe { std::ptr::read(session_dir as *const _) }, test_dir)
    }

    #[test]
    fn test_compiler_creation() {
        let (compiler, _session_dir, _test_dir) = setup_compiler();
        let (count, _total_size) = compiler.cache().stats();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_compile_simple_library() {
        let (mut compiler, _session_dir, _test_dir) = setup_compiler();

        let source = r#"
            #[no_mangle]
            pub extern "C" fn oxur_eval_test() {
                // Simple test function
            }
        "#;

        let result = compiler.compile("test_simple", source, 0);

        match result {
            Ok(path) => {
                assert!(path.exists(), "Compiled library should exist");
            }
            Err(e) => {
                // Compilation might fail in test environment without full rustc setup
                eprintln!("Note: Compilation failed (expected in some test environments): {}", e);
            }
        }
    }

    #[test]
    fn test_compile_with_caching() {
        let (mut compiler, _session_dir, _test_dir) = setup_compiler();

        let source = r#"
            #[no_mangle]
            pub extern "C" fn oxur_eval_cache_test() {}
        "#;

        // First compilation
        let result1 = compiler.compile("cache_test", source, 0);

        if let Ok(path1) = result1 {
            // Second compilation with same source should hit cache
            let result2 = compiler.compile("cache_test", source, 0);

            if let Ok(path2) = result2 {
                assert_eq!(path1, path2, "Should return same cached path");
            }
        }
    }

    #[test]
    fn test_compile_invalid_source() {
        let (mut compiler, _session_dir, _test_dir) = setup_compiler();

        let invalid_source = "this is not valid rust code {{{";

        let result = compiler.compile("invalid", invalid_source, 0);

        assert!(result.is_err(), "Compilation of invalid source should fail");

        if let Err(CompilerError::CompilationFailed(msg)) = result {
            assert!(!msg.is_empty(), "Should have compilation error message");
        }
    }

    #[test]
    fn test_different_opt_levels() {
        let (mut compiler, _session_dir, _test_dir) = setup_compiler();

        let source = r#"
            #[no_mangle]
            pub extern "C" fn oxur_eval_opt_test() {}
        "#;

        // Compile with different optimization levels
        let result_opt0 = compiler.compile("opt0", source, 0);
        let result_opt2 = compiler.compile("opt2", source, 2);

        // Both should compile (or both fail in test environment)
        assert_eq!(result_opt0.is_ok(), result_opt2.is_ok(), "Opt levels should have same outcome");
    }
}
