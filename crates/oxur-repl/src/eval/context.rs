// Evaluation context for REPL sessions
//
// Manages session state, tiered execution, and code caching.
// Based on ODD-0026: Oxur REPL Evaluation Strategy.

use crate::cache::ArtifactCache;
use crate::compiler::CachedCompiler;
use crate::eval::{output_capture::OutputCapturer, LispEvaluator, SexprEvaluator};
use crate::executor::SubprocessExecutor;
use crate::protocol::{ReplMode, SessionId};
use crate::session::SessionDir;
use crate::type_inference::TypeInference;
use crate::wrapper::RustAstWrapper;
use oxur_smap::SourcePos;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use thiserror::Error;

/// Evaluation errors
///
/// All errors include source position tracking using oxur-smap::SourcePos
/// to enable precise error reporting back to the original Oxur source code.
#[derive(Debug, Error)]
pub enum EvalError {
    /// Syntax error during parsing
    #[error("Syntax error at {pos}: {msg}")]
    SyntaxError { msg: String, pos: SourcePos },

    /// Type error during compilation
    #[error("Type error at {pos}: {msg}")]
    TypeError { msg: String, pos: SourcePos },

    /// Runtime evaluation error
    #[error("Runtime error at {pos}: {msg}")]
    RuntimeError { msg: String, pos: SourcePos },

    /// Compilation failed
    #[error("Compilation failed at {pos}: {msg}")]
    CompilationError { msg: String, pos: SourcePos },

    /// Unsupported operation for tier
    #[error("Unsupported operation at {pos}: {msg}")]
    UnsupportedOperation { msg: String, pos: SourcePos },
}

pub type Result<T> = std::result::Result<T, EvalError>;

/// Execution tier used for evaluation
///
/// Three-tier model per ODD-0026 Section 2:
/// - Tier 1: Interpret literal arithmetic only
/// - Tier 2: Execute from already-loaded library (cache hit)
/// - Tier 3: Compile and load (cache miss or first time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExecutionTier {
    /// Tier 1: Calculator mode - interpret simple arithmetic (<1ms)
    ///
    /// Only handles literal arithmetic expressions with no variables.
    /// Examples: `(+ 1 2)`, `(* 3 4)`, `(/ 10 2)`
    Calculator,

    /// Tier 2: Cached and loaded - execute from loaded library (~1-5ms)
    ///
    /// Code was previously compiled and the dynamic library is already
    /// loaded in the subprocess. Just calls the function.
    CachedLoaded,

    /// Tier 3: Just-in-time - compile, cache, and load (~50-300ms)
    ///
    /// First time seeing this code or library not yet loaded.
    /// Compiles to Rust, invokes cargo, loads dylib into subprocess.
    JustInTime,
}

/// Result of an evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct EvalResult {
    /// The resulting value as a string
    pub value: String,

    /// Execution tier used
    pub tier: ExecutionTier,

    /// Whether result came from cache
    pub cached: bool,

    /// Execution duration in milliseconds
    pub duration_ms: u64,

    /// Standard output captured during evaluation
    pub stdout: Option<String>,

    /// Standard error captured during evaluation
    pub stderr: Option<String>,
}

/// Evaluation context for a REPL session
///
/// Manages session state, execution tiers, and caching.
///
/// # Architecture
///
/// Uses Arc<Mutex<>> for components that need shared mutable state:
/// - ArtifactCache: Shared persistent cache across clones
/// - CachedCompiler: Shared compilation state
/// - SubprocessExecutor: Single subprocess per session (not cloned)
/// - SessionDir: Shared session directory
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// Unique session identifier
    session_id: SessionId,

    /// Evaluation mode (Lisp or Sexpr)
    mode: ReplMode,

    /// Lisp evaluator for Tier 1 (calculator mode in Lisp mode)
    lisp_eval: LispEvaluator,

    /// S-expression evaluator for Tier 1 (calculator mode in Sexpr mode)
    sexpr_eval: SexprEvaluator,

    /// Cached compiled code (hash -> result)
    cache: HashMap<String, String>,

    /// Execution statistics
    stats: ExecutionStats,

    /// Artifact cache for compiled libraries (shared)
    artifact_cache: Option<Arc<Mutex<ArtifactCache>>>,

    /// Cached compiler (shared)
    compiler: Option<Arc<Mutex<CachedCompiler>>>,

    /// Subprocess executor (shared, single instance per session)
    executor: Option<Arc<Mutex<SubprocessExecutor>>>,

    /// Session directory (shared)
    session_dir: Option<Arc<SessionDir>>,

    /// Rust AST wrapper for REPL scaffolding
    wrapper: RustAstWrapper,

    /// Type inference engine (Phase 7)
    type_inference: TypeInference,

    /// Source map for error translation (Phase 4)
    ///
    /// Tracks transformations from original Oxur source through Surface Forms,
    /// Core Forms, and Rust AST for precise error position mapping.
    source_map: oxur_smap::SourceMap,
}

/// Execution statistics for the session
#[derive(Debug, Clone, Default)]
struct ExecutionStats {
    /// Number of Tier 1 (calculator) evaluations
    tier1_count: u64,

    /// Number of Tier 2 (compiled) evaluations
    tier2_count: u64,

    /// Number of cache hits
    cache_hits: u64,
}

impl EvalContext {
    /// Create a new evaluation context
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_repl::eval::EvalContext;
    /// use oxur_repl::protocol::{ReplMode, SessionId};
    ///
    /// let ctx = EvalContext::new(SessionId::new("session-1"), ReplMode::Lisp);
    /// ```
    pub fn new(session_id: SessionId, mode: ReplMode) -> Self {
        Self {
            session_id,
            mode,
            lisp_eval: LispEvaluator::new(),
            sexpr_eval: SexprEvaluator::new(),
            cache: HashMap::new(),
            stats: ExecutionStats::default(),
            artifact_cache: None,
            compiler: None,
            executor: None,
            session_dir: None,
            wrapper: RustAstWrapper::new(),
            type_inference: TypeInference::new(),
            source_map: oxur_smap::SourceMap::new(),
        }
    }

    /// Create a new evaluation context with full compilation pipeline
    ///
    /// Initializes all Tier 2/3 components for actual compilation and execution.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Session directory cannot be created
    /// - Artifact cache cannot be initialized
    /// - Subprocess cannot be spawned
    pub fn with_compilation(session_id: SessionId, mode: ReplMode) -> Result<Self> {
        // Create session directory
        let session_dir =
            SessionDir::new(&session_id).map_err(|e| EvalError::CompilationError {
                msg: format!("Failed to create session directory: {}", e),
                pos: SourcePos::repl(1, 1, 1),
            })?;
        let session_dir = Arc::new(session_dir);

        // Create artifact cache
        let artifact_cache = ArtifactCache::new().map_err(|e| EvalError::CompilationError {
            msg: format!("Failed to create artifact cache: {}", e),
            pos: SourcePos::repl(1, 1, 1),
        })?;
        let artifact_cache = Arc::new(Mutex::new(artifact_cache));

        // Create compiler with Arc<SessionDir>
        let cache_clone = {
            let guard = artifact_cache.lock().expect("Artifact cache mutex poisoned");
            (*guard).clone()
        };
        let compiler = CachedCompiler::new(cache_clone, Arc::clone(&session_dir));
        let compiler = Arc::new(Mutex::new(compiler));

        // Create subprocess executor
        let executor = SubprocessExecutor::new().map_err(|e| EvalError::CompilationError {
            msg: format!("Failed to create subprocess executor: {}", e),
            pos: SourcePos::repl(1, 1, 1),
        })?;
        let executor = Arc::new(Mutex::new(executor));

        Ok(Self {
            session_id,
            mode,
            lisp_eval: LispEvaluator::new(),
            sexpr_eval: SexprEvaluator::new(),
            cache: HashMap::new(),
            stats: ExecutionStats::default(),
            artifact_cache: Some(artifact_cache),
            compiler: Some(compiler),
            executor: Some(executor),
            session_dir: Some(session_dir),
            wrapper: RustAstWrapper::new(),
            type_inference: TypeInference::new(),
            source_map: oxur_smap::SourceMap::new(),
        })
    }

    /// Get the session ID
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Get the evaluation mode
    pub fn mode(&self) -> ReplMode {
        self.mode
    }

    /// Get execution statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (self.stats.tier1_count, self.stats.tier2_count, self.stats.cache_hits)
    }

    /// Clone this context into a new session
    ///
    /// Creates a new context with a different session ID but the same
    /// cache and state. Resets execution statistics.
    ///
    /// Note: Shared components (Arc<Mutex<>>) are cloned (reference counted),
    /// so they share state across clones.
    pub fn clone_to(&self, new_session_id: SessionId) -> Self {
        Self {
            session_id: new_session_id,
            mode: self.mode,
            lisp_eval: LispEvaluator::new(),
            sexpr_eval: SexprEvaluator::new(),
            cache: self.cache.clone(),
            stats: ExecutionStats::default(),
            artifact_cache: self.artifact_cache.clone(),
            compiler: self.compiler.clone(),
            executor: self.executor.clone(),
            session_dir: self.session_dir.clone(),
            wrapper: RustAstWrapper::new(),
            type_inference: TypeInference::new(),
            source_map: oxur_smap::SourceMap::new(), // Fresh source map for new session
        }
    }

    /// Evaluate code in this context
    ///
    /// Uses tiered execution strategy:
    /// - Tier 1 (Calculator): Simple arithmetic literals only
    /// - Tier 2 (Cached Compilation): Everything else
    ///
    /// # Errors
    ///
    /// Returns `EvalError` if evaluation fails due to:
    /// - `SyntaxError`: Invalid syntax in the input code
    /// - `TypeError`: Type mismatch or invalid type operations
    /// - `RuntimeError`: Runtime errors during execution
    /// - `CompilationError`: Failed to compile to executable code
    /// - `UnsupportedOperation`: Operation not supported in current tier
    pub async fn eval(&mut self, code: impl AsRef<str>) -> Result<EvalResult> {
        let code = code.as_ref();
        let start = Instant::now();

        // Reset source map for this evaluation
        self.source_map = oxur_smap::SourceMap::new();

        // Attempt Tier 1 (Calculator mode) first
        if let Some(result) = self.try_calculator(code) {
            let duration_ms = start.elapsed().as_millis() as u64;
            self.stats.tier1_count += 1;

            return Ok(EvalResult {
                value: result,
                tier: ExecutionTier::Calculator,
                cached: false,
                duration_ms,
                stdout: None,
                stderr: None,
            });
        }

        // Fall through to Tier 2 (Cached Compilation)
        self.eval_tier2(code, start).await
    }

    /// Try to evaluate using Tier 1 (Calculator mode)
    ///
    /// Dispatches to the appropriate evaluator based on mode:
    /// - Lisp mode: Simple arithmetic with Lisp syntax
    /// - Sexpr mode: Canonical s-expressions with keywords
    ///
    /// Returns Some(result) if successful, None if not calculator-eligible.
    fn try_calculator(&mut self, code: &str) -> Option<String> {
        match self.mode {
            ReplMode::Lisp => self.lisp_eval.try_eval_calculator(code),
            ReplMode::Sexpr => self.sexpr_eval.try_eval_calculator(code),
        }
    }

    /// Evaluate using Tier 2/3 (Cached or JIT Compilation)
    ///
    /// Distinguishes between:
    /// - Tier 2 (CachedLoaded): Library already loaded in subprocess, just execute
    /// - Tier 3 (JustInTime): Need to compile and/or load library before executing
    async fn eval_tier2(&mut self, code: &str, start: Instant) -> Result<EvalResult> {
        // Generate cache key (hash of code)
        let cache_key = self.hash_code(code);

        // If executor exists, check if library is already loaded (Tier 2)
        if let Some(executor) = &self.executor {
            let is_loaded = executor.lock().expect("Executor mutex poisoned").is_loaded(&cache_key);

            if is_loaded {
                // Tier 2: Library already loaded, just execute
                let exec_result =
                    executor.lock().expect("Executor mutex poisoned").execute(&cache_key).map_err(
                        |e| EvalError::RuntimeError {
                            msg: format!("Execution failed: {}", e),
                            pos: SourcePos::repl(1, 1, code.len() as u32),
                        },
                    )?;

                let duration_ms = start.elapsed().as_millis() as u64;
                self.stats.tier2_count += 1;
                self.stats.cache_hits += 1;

                let result_value = match exec_result {
                    crate::executor::ExecutionResult::Success { output } => output,
                    crate::executor::ExecutionResult::RuntimeError { message } => {
                        return Err(EvalError::RuntimeError {
                            msg: message,
                            pos: SourcePos::repl(1, 1, code.len() as u32),
                        });
                    }
                    crate::executor::ExecutionResult::Panic { location, message } => {
                        return Err(EvalError::RuntimeError {
                            msg: format!("Panic at {}: {}", location, message),
                            pos: SourcePos::repl(1, 1, code.len() as u32),
                        });
                    }
                };

                // Cache the result for future reference
                self.cache.insert(cache_key, result_value.clone());

                return Ok(EvalResult {
                    value: result_value,
                    tier: ExecutionTier::CachedLoaded,
                    cached: true,
                    duration_ms,
                    stdout: None,
                    stderr: None,
                });
            }
        } else {
            // No executor - check string cache for simple caching (backward compatibility)
            if let Some(cached_result) = self.cache.get(&cache_key) {
                let duration_ms = start.elapsed().as_millis() as u64;
                self.stats.tier2_count += 1;
                self.stats.cache_hits += 1;

                return Ok(EvalResult {
                    value: cached_result.clone(),
                    tier: ExecutionTier::CachedLoaded,
                    cached: true,
                    duration_ms,
                    stdout: None,
                    stderr: None,
                });
            }
        }

        // Tier 3: Need to compile and/or load library
        // Full compilation pipeline: parse, expand, wrap, compile, load, execute
        let (result, stdout, stderr) = self.compile_and_execute(code).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Cache the result
        self.cache.insert(cache_key, result.clone());
        self.stats.tier2_count += 1; // Track as tier2 (compiled) in stats

        Ok(EvalResult {
            value: result,
            tier: ExecutionTier::JustInTime,
            cached: false,
            duration_ms,
            stdout,
            stderr,
        })
    }

    /// Compile and execute code using the full pipeline
    ///
    /// Tier 2/3 execution path:
    /// 1. Parse code using mode-specific parser (Lisp or Sexpr)
    /// 2. Convert to CoreForms (oxur-lang IR)
    /// 3. Wrap with REPL scaffolding (RustAstWrapper)
    /// 4. Compile to dynamic library (CachedCompiler)
    /// 5. Load library into subprocess (SubprocessExecutor)
    /// 6. Execute with output capture
    ///
    /// Returns (result_value, stdout, stderr)
    async fn compile_and_execute(
        &mut self,
        code: &str,
    ) -> Result<(String, Option<String>, Option<String>)> {
        // Step 1: Parse code to CoreForms using mode-specific parser
        // Record surface nodes in source map during parsing
        let core_forms = match self.mode {
            ReplMode::Lisp => {
                // Use Lisp parser to parse and convert to CoreForms
                let forms = self.lisp_eval.parse(code).map_err(|e| EvalError::SyntaxError {
                    msg: format!("Lisp parse error: {}", e),
                    pos: SourcePos::repl(1, 1, code.len() as u32),
                })?;

                // Record surface nodes (Phase 4 placeholder - will be done by parser in Phase 6+)
                // For now, create one surface node per form
                for (idx, _form) in forms.iter().enumerate() {
                    let node = oxur_smap::new_node_id();
                    let pos = oxur_smap::SourcePos::repl(1, 1, code.len() as u32);
                    self.source_map.record_surface_node(node, pos);

                    // In Phase 6+, the parser will provide actual NodeIds and positions
                    // For now this is a placeholder to enable comment generation
                    let _ = idx; // suppress unused warning
                }

                forms
            }
            ReplMode::Sexpr => {
                // Use Sexpr parser to parse and convert to CoreForms
                let form =
                    self.sexpr_eval.parse_to_core(code).map_err(|e| EvalError::SyntaxError {
                        msg: format!("S-expression parse error: {}", e),
                        pos: SourcePos::repl(1, 1, code.len() as u32),
                    })?;

                // Record surface node (Phase 4 placeholder)
                let node = oxur_smap::new_node_id();
                let pos = oxur_smap::SourcePos::repl(1, 1, code.len() as u32);
                self.source_map.record_surface_node(node, pos);

                vec![form]
            }
        };

        // Create output capturer for this execution
        let capturer = OutputCapturer::new();

        // Handle empty parse results (Parser is placeholder for now)
        if core_forms.is_empty() {
            // Parser returned empty - it's a placeholder
            // For now, just acknowledge we received code
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            let result = format!("compiled(placeholder, mode: {:?})", self.mode);
            let output = capturer.get_output();
            return Ok((result, output.stdout_option(), output.stderr_option()));
        }

        // Step 2: Expand macros (when Expander is ready)
        // TODO: Use oxur_lang::Expander to expand macros
        // let mut expander = Expander::new();
        // let expanded = expander.expand(core_forms)?;

        // Step 3-6: Compile and execute with output capture
        // Check if compilation pipeline is available
        let (compiler, executor) = match (&self.compiler, &self.executor) {
            (Some(c), Some(e)) => (c, e),
            _ => {
                // Fall back to simulation if pipeline not initialized
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                let result = format!("compiled(placeholder, mode: {:?})", self.mode);
                let output = capturer.get_output();
                return Ok((result, output.stdout_option(), output.stderr_option()));
            }
        };

        // Generate cache key
        let cache_key = self.hash_code(code);

        // Step 3: Wrap code with REPL scaffolding
        // Pass SourceMap for Phase 4 error translation
        let wrapped_code =
            self.wrapper.wrap(&cache_key, code, Some(&self.source_map)).map_err(|e| EvalError::CompilationError {
                msg: format!("Failed to wrap code: {}", e),
                pos: SourcePos::repl(1, 1, code.len() as u32),
            })?;

        // Step 4: Compile to dynamic library
        let lib_path = compiler
            .lock()
            .expect("Compiler mutex poisoned")
            .compile(&cache_key, &wrapped_code, 2)
            .map_err(|e| EvalError::CompilationError {
                msg: format!("Compilation failed: {}", e),
                pos: SourcePos::repl(1, 1, code.len() as u32),
            })?;

        // Step 5: Load library into subprocess
        executor
            .lock()
            .expect("Executor mutex poisoned")
            .load_library(&lib_path, &cache_key)
            .map_err(|e| EvalError::CompilationError {
                msg: format!("Failed to load library: {}", e),
                pos: SourcePos::repl(1, 1, code.len() as u32),
            })?;

        // Step 6: Execute in subprocess
        let exec_result =
            executor.lock().expect("Executor mutex poisoned").execute(&cache_key).map_err(|e| {
                EvalError::RuntimeError {
                    msg: format!("Execution failed: {}", e),
                    pos: SourcePos::repl(1, 1, code.len() as u32),
                }
            })?;

        // Extract result and handle different execution outcomes
        let (result, stdout, stderr) = match exec_result {
            crate::executor::ExecutionResult::Success { output } => (output, None, None),
            crate::executor::ExecutionResult::RuntimeError { message } => {
                return Err(EvalError::RuntimeError {
                    msg: message,
                    pos: SourcePos::repl(1, 1, code.len() as u32),
                });
            }
            crate::executor::ExecutionResult::Panic { location, message } => {
                return Err(EvalError::RuntimeError {
                    msg: format!("Panic at {}: {}", location, message),
                    pos: SourcePos::repl(1, 1, code.len() as u32),
                });
            }
        };

        Ok((result, stdout, stderr))
    }

    /// Generate cache key from code
    fn hash_code(&self, code: &str) -> String {
        // Simple hash for now (in production, use a real hash function)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        // Prefix with 'h' to make it a valid Rust identifier
        format!("h{:x}", hasher.finish())
    }

    /// Clear the compilation cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    // ========================================================================
    // Type Inference Methods (Phase 7)
    // ========================================================================

    /// Infer types from Rust code
    ///
    /// Parses the provided Rust code and extracts type information for
    /// all variable declarations. This can be used to implement the `:type`
    /// REPL command.
    ///
    /// # Arguments
    ///
    /// * `code` - Rust source code to analyze
    ///
    /// # Returns
    ///
    /// Vector of (variable_name, type_info) pairs
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_repl::eval::EvalContext;
    /// use oxur_repl::protocol::{ReplMode, SessionId};
    ///
    /// let ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);
    ///
    /// let code = r#"
    ///     fn main() {
    ///         let x: i32 = 42;
    ///         let y = "hello";
    ///     }
    /// "#;
    ///
    /// let types = ctx.infer_types(code);
    /// assert_eq!(types.len(), 2);
    /// ```
    pub fn infer_types(&self, code: impl AsRef<str>) -> Vec<(String, String, bool)> {
        self.type_inference
            .infer_from_code(code)
            .into_iter()
            .map(|(name, info)| (name, info.type_name, info.is_mutable))
            .collect()
    }

    /// Register a variable's type information
    ///
    /// Manually register type information for a variable. This is useful for
    /// tracking types across REPL evaluations.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `type_name` - Type as a string (e.g., "i32", "String")
    /// * `is_mutable` - Whether the variable is mutable
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_repl::eval::EvalContext;
    /// use oxur_repl::protocol::{ReplMode, SessionId};
    ///
    /// let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);
    /// ctx.register_variable_type("x", "i32", false);
    ///
    /// let type_name = ctx.get_variable_type("x");
    /// assert_eq!(type_name, Some("i32".to_string()));
    /// ```
    pub fn register_variable_type(
        &mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
        is_mutable: bool,
    ) {
        use crate::type_inference::TypeInfo;
        let type_info = TypeInfo::new(type_name).with_mutable(is_mutable);
        self.type_inference.register_variable(name, type_info);
    }

    /// Get the type of a variable
    ///
    /// Returns the type name for a previously registered variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    ///
    /// # Returns
    ///
    /// Some(type_name) if variable is tracked, None otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_repl::eval::EvalContext;
    /// use oxur_repl::protocol::{ReplMode, SessionId};
    ///
    /// let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);
    /// ctx.register_variable_type("x", "i32", false);
    ///
    /// assert_eq!(ctx.get_variable_type("x"), Some("i32".to_string()));
    /// assert_eq!(ctx.get_variable_type("y"), None);
    /// ```
    pub fn get_variable_type(&self, name: impl AsRef<str>) -> Option<String> {
        self.type_inference.get_variable_type(name).ok().map(|info| info.type_name.clone())
    }

    /// Get all tracked variables with their types
    ///
    /// Returns a list of all variables currently tracked by the type inference
    /// engine along with their type information.
    ///
    /// # Returns
    ///
    /// Vector of (variable_name, type_name, is_mutable) tuples
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_repl::eval::EvalContext;
    /// use oxur_repl::protocol::{ReplMode, SessionId};
    ///
    /// let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);
    /// ctx.register_variable_type("x", "i32", false);
    /// ctx.register_variable_type("y", "String", true);
    ///
    /// let vars = ctx.get_all_variable_types();
    /// assert_eq!(vars.len(), 2);
    /// ```
    pub fn get_all_variable_types(&self) -> Vec<(String, String, bool)> {
        self.type_inference
            .all_variables()
            .map(|(name, info)| (name.clone(), info.type_name.clone(), info.is_mutable))
            .collect()
    }

    /// Clear all tracked type information
    ///
    /// Removes all tracked variable types from the type inference engine.
    /// Useful when starting a new REPL session or resetting state.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_repl::eval::EvalContext;
    /// use oxur_repl::protocol::{ReplMode, SessionId};
    ///
    /// let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);
    /// ctx.register_variable_type("x", "i32", false);
    /// assert_eq!(ctx.get_all_variable_types().len(), 1);
    ///
    /// ctx.clear_type_information();
    /// assert_eq!(ctx.get_all_variable_types().len(), 0);
    /// ```
    pub fn clear_type_information(&mut self) {
        self.type_inference.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context() {
        let ctx = EvalContext::new(SessionId::new("test-session"), ReplMode::Lisp);
        assert_eq!(ctx.session_id(), &SessionId::new("test-session"));
        assert_eq!(ctx.mode(), ReplMode::Lisp);

        let (tier1, tier2, hits) = ctx.stats();
        assert_eq!(tier1, 0);
        assert_eq!(tier2, 0);
        assert_eq!(hits, 0);
    }

    #[test]
    fn test_clone_to() {
        let mut ctx = EvalContext::new(SessionId::new("session-1"), ReplMode::Lisp);
        ctx.stats.tier1_count = 5;
        ctx.cache.insert("key".to_string(), "value".to_string());

        let cloned = ctx.clone_to(SessionId::new("session-2"));
        assert_eq!(cloned.session_id(), &SessionId::new("session-2"));
        assert_eq!(cloned.cache.len(), 1);

        // Stats should be reset
        let (tier1, tier2, hits) = cloned.stats();
        assert_eq!(tier1, 0);
        assert_eq!(tier2, 0);
        assert_eq!(hits, 0);
    }

    #[tokio::test]
    async fn test_eval_tier1_calculator() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        let result = ctx.eval("(+ 1 2)").await.unwrap();
        assert_eq!(result.value, "3");
        assert_eq!(result.tier, ExecutionTier::Calculator);
        assert!(!result.cached);
        assert!(result.duration_ms < 10); // Should be very fast

        let (tier1, tier2, _) = ctx.stats();
        assert_eq!(tier1, 1);
        assert_eq!(tier2, 0);
    }

    #[tokio::test]
    async fn test_eval_tier3_jit_compilation() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        let result = ctx.eval("(defn foo [x] (* x 2))").await.unwrap();
        assert!(result.value.contains("compiled"));
        assert_eq!(result.tier, ExecutionTier::JustInTime);
        assert!(!result.cached);

        let (tier1, tier2, _) = ctx.stats();
        assert_eq!(tier1, 0);
        assert_eq!(tier2, 1);
    }

    #[tokio::test]
    async fn test_eval_caching() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        // First evaluation - not cached (Tier 3: JIT)
        let result1 = ctx.eval("(defn bar [x] x)").await.unwrap();
        assert!(!result1.cached);
        assert_eq!(result1.tier, ExecutionTier::JustInTime);

        // Second evaluation - should be cached (Tier 2: CachedLoaded)
        let result2 = ctx.eval("(defn bar [x] x)").await.unwrap();
        assert!(result2.cached);
        assert_eq!(result2.tier, ExecutionTier::CachedLoaded);
        assert_eq!(result2.value, result1.value);

        let (_, tier2, hits) = ctx.stats();
        assert_eq!(tier2, 2);
        assert_eq!(hits, 1);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        ctx.eval("(+ x 1)").await.unwrap();
        assert_eq!(ctx.cache_size(), 1);

        ctx.clear_cache();
        assert_eq!(ctx.cache_size(), 0);
    }

    #[test]
    fn test_calculator_mode() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        assert_eq!(ctx.try_calculator("(+ 1 2)"), Some("3".to_string()));
        assert_eq!(ctx.try_calculator("(+ 10 20)"), Some("30".to_string()));
        assert_eq!(ctx.try_calculator("(defn foo [x] x)"), None);
        assert_eq!(ctx.try_calculator("(+ 1 2 3)"), Some("6".to_string())); // Multiple args now supported
    }

    #[tokio::test]
    async fn test_tier3_lisp_mode() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        // Code that's not calculator-eligible goes to Tier 3 (JIT) first time
        let result = ctx.eval("(defn foo [x] x)").await.unwrap();

        assert_eq!(result.tier, ExecutionTier::JustInTime);
        assert!(!result.cached);
        assert!(result.value.contains("mode: Lisp"));
    }

    #[tokio::test]
    async fn test_tier3_sexpr_mode() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Sexpr);

        // Complex code goes to Tier 3 (JIT) first time (not handled by calculator)
        // Using an invalid symbol that can't be evaluated in calculator
        let result = ctx.eval("undefined-symbol").await.unwrap();

        assert_eq!(result.tier, ExecutionTier::JustInTime);
        assert!(!result.cached);
        // Without compilation pipeline initialized, falls back to placeholder mode
        assert!(result.value.contains("placeholder"));
    }

    #[tokio::test]
    async fn test_tier_fallback() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        // Tier 1 (calculator) - fast path
        let result1 = ctx.eval("(+ 1 2)").await.unwrap();
        assert_eq!(result1.tier, ExecutionTier::Calculator);
        assert_eq!(result1.value, "3");

        // Tier 3 (JIT compilation) - complex code, first time
        let result2 = ctx.eval("(defn add [a b] (+ a b))").await.unwrap();
        assert_eq!(result2.tier, ExecutionTier::JustInTime);
    }

    #[tokio::test]
    async fn test_mode_specific_calculators() {
        // Test Lisp mode calculator
        let mut lisp_ctx = EvalContext::new(SessionId::new("lisp-test"), ReplMode::Lisp);
        let lisp_result = lisp_ctx.eval("(* 3 4)").await.unwrap();
        assert_eq!(lisp_result.tier, ExecutionTier::Calculator);
        assert_eq!(lisp_result.value, "12");

        // Test Sexpr mode calculator
        let mut sexpr_ctx = EvalContext::new(SessionId::new("sexpr-test"), ReplMode::Sexpr);
        let sexpr_result = sexpr_ctx.eval("(/ 10 2)").await.unwrap();
        assert_eq!(sexpr_result.tier, ExecutionTier::Calculator);
        assert_eq!(sexpr_result.value, "5");
    }

    #[test]
    fn test_source_pos_creation() {
        let pos = SourcePos::repl(3, 15, 10);
        assert_eq!(pos.line, 3);
        assert_eq!(pos.column, 15);
        assert_eq!(pos.length, 10);
        assert_eq!(pos.file, "<repl>");
    }

    #[test]
    fn test_source_pos_display() {
        let pos = SourcePos::repl(3, 15, 10);
        assert_eq!(pos.to_string(), "<repl>:3:15");
    }

    #[test]
    fn test_source_pos_equality() {
        let pos1 = SourcePos::repl(3, 15, 10);
        let pos2 = SourcePos::repl(3, 15, 10);
        let pos3 = SourcePos::repl(4, 10, 5);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_eval_error_includes_source_pos() {
        let err = EvalError::SyntaxError {
            msg: "Unexpected token".to_string(),
            pos: SourcePos::repl(2, 5, 1),
        };

        let err_string = err.to_string();
        assert!(err_string.contains("<repl>:2:5"));
        assert!(err_string.contains("Unexpected token"));
    }

    #[test]
    fn test_all_error_variants_include_source_pos() {
        let pos = SourcePos::repl(1, 1, 1);

        let errors = vec![
            EvalError::SyntaxError { msg: "syntax".to_string(), pos: pos.clone() },
            EvalError::TypeError { msg: "type".to_string(), pos: pos.clone() },
            EvalError::RuntimeError { msg: "runtime".to_string(), pos: pos.clone() },
            EvalError::CompilationError { msg: "compilation".to_string(), pos: pos.clone() },
            EvalError::UnsupportedOperation { msg: "unsupported".to_string(), pos: pos.clone() },
        ];

        for err in errors {
            let err_string = err.to_string();
            assert!(err_string.contains("<repl>:1:1"));
        }
    }
}
