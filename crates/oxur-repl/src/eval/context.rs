// Evaluation context for REPL sessions
//
// Manages session state, tiered execution, and code caching.
// Based on ODD-0026: Oxur REPL Evaluation Strategy.

use crate::eval::{output_capture::OutputCapturer, LispEvaluator, SexprEvaluator};
use crate::protocol::{ReplMode, SessionId};
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;

/// Evaluation errors
#[derive(Debug, Error)]
pub enum EvalError {
    /// Syntax error during parsing
    #[error("Syntax error: {0}")]
    SyntaxError(String),

    /// Type error during compilation
    #[error("Type error: {0}")]
    TypeError(String),

    /// Runtime evaluation error
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    /// Compilation failed
    #[error("Compilation failed: {0}")]
    CompilationError(String),

    /// Unsupported operation for tier
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

pub type Result<T> = std::result::Result<T, EvalError>;

/// Execution tier used for evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionTier {
    /// Tier 1: Calculator mode (interpret simple arithmetic)
    /// Target: <1ms response time, ~100 lines of code
    Calculator,

    /// Tier 2: Cached compilation (compile everything else)
    /// First time: 50-200ms, Cached: ~0ms
    CachedCompilation,
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
#[derive(Clone)]
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
        }
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
    pub fn clone_to(&self, new_session_id: SessionId) -> Self {
        Self {
            session_id: new_session_id,
            mode: self.mode,
            lisp_eval: LispEvaluator::new(),
            sexpr_eval: SexprEvaluator::new(),
            cache: self.cache.clone(),
            stats: ExecutionStats::default(),
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
    /// Returns error if parsing, compilation, or execution fails.
    pub async fn eval(&mut self, code: impl AsRef<str>) -> Result<EvalResult> {
        let code = code.as_ref();
        let start = Instant::now();

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

    /// Evaluate using Tier 2 (Cached Compilation)
    ///
    /// Compiles code, caches result, and executes.
    async fn eval_tier2(&mut self, code: &str, start: Instant) -> Result<EvalResult> {
        // Generate cache key (hash of code)
        let cache_key = self.hash_code(code);

        // Check cache first
        if let Some(cached_result) = self.cache.get(&cache_key) {
            let duration_ms = start.elapsed().as_millis() as u64;
            self.stats.tier2_count += 1;
            self.stats.cache_hits += 1;

            return Ok(EvalResult {
                value: cached_result.clone(),
                tier: ExecutionTier::CachedCompilation,
                cached: true,
                duration_ms,
                stdout: None,
                stderr: None,
            });
        }

        // Compile and execute with output capture
        // Steps 1-2 (parse and expand) are now implemented
        // Steps 3-6 (lower, codegen, compile, execute) are TODO
        let (result, stdout, stderr) = self.compile_and_execute(code).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Cache the result
        self.cache.insert(cache_key, result.clone());
        self.stats.tier2_count += 1;

        Ok(EvalResult {
            value: result,
            tier: ExecutionTier::CachedCompilation,
            cached: false,
            duration_ms,
            stdout,
            stderr,
        })
    }

    /// Compile and execute code
    ///
    /// Tier 2 execution path:
    /// 1. Parse code using mode-specific parser (Lisp or Sexpr)
    /// 2. Convert to CoreForms (oxur-lang IR)
    /// 3. Lower to Rust AST (oxur-comp) [TODO: when ready]
    /// 4. Generate Rust source [TODO: when ready]
    /// 5. Compile to dynamic library [TODO: when ready]
    /// 6. Load and execute with output capture [TODO: when ready]
    ///
    /// Returns (result_value, stdout, stderr)
    async fn compile_and_execute(
        &mut self,
        code: &str,
    ) -> Result<(String, Option<String>, Option<String>)> {
        // Step 1: Parse code to CoreForms using mode-specific parser
        let core_forms = match self.mode {
            ReplMode::Lisp => {
                // Use Lisp parser to parse and convert to CoreForms
                let forms = self
                    .lisp_eval
                    .parse(code)
                    .map_err(|e| EvalError::SyntaxError(format!("Lisp parse error: {}", e)))?;

                forms
            }
            ReplMode::Sexpr => {
                // Use Sexpr parser to parse and convert to CoreForms
                let form = self.sexpr_eval.parse_to_core(code).map_err(|e| {
                    EvalError::SyntaxError(format!("S-expression parse error: {}", e))
                })?;

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
        // TODO: Integrate with oxur-comp when ready:
        // - Lower CoreForms to Rust AST
        // - Generate Rust source code
        // - Compile to dynamic library with cargo
        // - Load library and execute
        // - Capture stdout/stderr during execution

        // For now, simulate compilation delay and execution
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Simulate execution with output capture
        let result = capturer.with_capture(|| {
            // When we actually execute compiled code, output will be captured here
            // For now, simulate some output for demonstration
            use crate::eval::output_capture::simulate_execution;
            simulate_execution(code, &capturer)
        });

        let output = capturer.get_output();

        // Return result with captured output
        Ok((result, output.stdout_option(), output.stderr_option()))
    }

    /// Generate cache key from code
    fn hash_code(&self, code: &str) -> String {
        // Simple hash for now (in production, use a real hash function)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Clear the compilation cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
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
    async fn test_eval_tier2_compilation() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        let result = ctx.eval("(defn foo [x] (* x 2))").await.unwrap();
        assert!(result.value.contains("compiled"));
        assert_eq!(result.tier, ExecutionTier::CachedCompilation);
        assert!(!result.cached);

        let (tier1, tier2, _) = ctx.stats();
        assert_eq!(tier1, 0);
        assert_eq!(tier2, 1);
    }

    #[tokio::test]
    async fn test_eval_caching() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        // First evaluation - not cached
        let result1 = ctx.eval("(defn bar [x] x)").await.unwrap();
        assert!(!result1.cached);

        // Second evaluation - should be cached
        let result2 = ctx.eval("(defn bar [x] x)").await.unwrap();
        assert!(result2.cached);
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
    async fn test_tier2_lisp_mode() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        // Code that's not calculator-eligible goes to Tier 2
        let result = ctx.eval("(defn foo [x] x)").await.unwrap();

        assert_eq!(result.tier, ExecutionTier::CachedCompilation);
        assert!(!result.cached);
        assert!(result.value.contains("mode: Lisp"));
    }

    #[tokio::test]
    async fn test_tier2_sexpr_mode() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Sexpr);

        // Complex code goes to Tier 2 (not handled by calculator)
        // Using an invalid symbol that can't be evaluated in calculator
        let result = ctx.eval("undefined-symbol").await.unwrap();

        assert_eq!(result.tier, ExecutionTier::CachedCompilation);
        assert!(!result.cached);
        assert!(result.value.contains("executed"));
    }

    #[tokio::test]
    async fn test_tier_fallback() {
        let mut ctx = EvalContext::new(SessionId::new("test"), ReplMode::Lisp);

        // Tier 1 (calculator) - fast path
        let result1 = ctx.eval("(+ 1 2)").await.unwrap();
        assert_eq!(result1.tier, ExecutionTier::Calculator);
        assert_eq!(result1.value, "3");

        // Tier 2 (compilation) - complex code
        let result2 = ctx.eval("(defn add [a b] (+ a b))").await.unwrap();
        assert_eq!(result2.tier, ExecutionTier::CachedCompilation);
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
}
