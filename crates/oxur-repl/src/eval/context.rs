// Evaluation context for REPL sessions
//
// Manages session state, tiered execution, and code caching.
// Based on ODD-0026: Oxur REPL Evaluation Strategy.

use crate::eval::LispEvaluator;
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
pub struct EvalContext {
    /// Unique session identifier
    session_id: SessionId,

    /// Evaluation mode (Lisp or Sexpr)
    mode: ReplMode,

    /// Lisp evaluator for Tier 1 (calculator mode)
    lisp_eval: LispEvaluator,

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
    /// use oxur_repl::protocol::ReplMode;
    ///
    /// let ctx = EvalContext::new("session-1".to_string(), ReplMode::Lisp);
    /// ```
    pub fn new(session_id: SessionId, mode: ReplMode) -> Self {
        Self {
            session_id,
            mode,
            lisp_eval: LispEvaluator::new(),
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
        (
            self.stats.tier1_count,
            self.stats.tier2_count,
            self.stats.cache_hits,
        )
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
    pub async fn eval(&mut self, code: &str) -> Result<EvalResult> {
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
    /// Only handles simple arithmetic:
    /// - Literals: integers
    /// - Operations: +, -, *, /
    /// - Nested expressions: `(+ (* 2 3) 4)`
    /// - No variables, no side effects, no control flow
    ///
    /// Returns Some(result) if successful, None if not calculator-eligible.
    fn try_calculator(&mut self, code: &str) -> Option<String> {
        // Use LispEvaluator for calculator mode
        self.lisp_eval.try_eval_calculator(code)
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

        // Compile and execute (placeholder)
        // Real implementation would:
        // 1. Parse with oxur-lang (Parser, Expander)
        // 2. Lower to Rust AST with oxur-comp
        // 3. Generate Rust source
        // 4. Compile to dynamic library with cargo
        // 5. Load and execute

        let result = self.compile_and_execute(code).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Cache the result
        self.cache.insert(cache_key, result.clone());
        self.stats.tier2_count += 1;

        Ok(EvalResult {
            value: result,
            tier: ExecutionTier::CachedCompilation,
            cached: false,
            duration_ms,
            stdout: None,
            stderr: None,
        })
    }

    /// Compile and execute code (placeholder)
    async fn compile_and_execute(&self, code: &str) -> Result<String> {
        // Placeholder implementation
        // For now, just return a placeholder result
        // Real implementation will integrate with oxur-lang and oxur-comp

        // Simulate compilation delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Return placeholder result
        Ok(format!("compiled({})", code))
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
        let ctx = EvalContext::new("test-session".to_string(), ReplMode::Lisp);
        assert_eq!(ctx.session_id(), "test-session");
        assert_eq!(ctx.mode(), ReplMode::Lisp);

        let (tier1, tier2, hits) = ctx.stats();
        assert_eq!(tier1, 0);
        assert_eq!(tier2, 0);
        assert_eq!(hits, 0);
    }

    #[test]
    fn test_clone_to() {
        let mut ctx = EvalContext::new("session-1".to_string(), ReplMode::Lisp);
        ctx.stats.tier1_count = 5;
        ctx.cache.insert("key".to_string(), "value".to_string());

        let cloned = ctx.clone_to("session-2".to_string());
        assert_eq!(cloned.session_id(), "session-2");
        assert_eq!(cloned.cache.len(), 1);

        // Stats should be reset
        let (tier1, tier2, hits) = cloned.stats();
        assert_eq!(tier1, 0);
        assert_eq!(tier2, 0);
        assert_eq!(hits, 0);
    }

    #[tokio::test]
    async fn test_eval_tier1_calculator() {
        let mut ctx = EvalContext::new("test".to_string(), ReplMode::Lisp);

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
        let mut ctx = EvalContext::new("test".to_string(), ReplMode::Lisp);

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
        let mut ctx = EvalContext::new("test".to_string(), ReplMode::Lisp);

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
        let mut ctx = EvalContext::new("test".to_string(), ReplMode::Lisp);

        ctx.eval("(+ x 1)").await.unwrap();
        assert_eq!(ctx.cache_size(), 1);

        ctx.clear_cache();
        assert_eq!(ctx.cache_size(), 0);
    }

    #[test]
    fn test_calculator_mode() {
        let mut ctx = EvalContext::new("test".to_string(), ReplMode::Lisp);

        assert_eq!(ctx.try_calculator("(+ 1 2)"), Some("3".to_string()));
        assert_eq!(ctx.try_calculator("(+ 10 20)"), Some("30".to_string()));
        assert_eq!(ctx.try_calculator("(defn foo [x] x)"), None);
        assert_eq!(ctx.try_calculator("(+ 1 2 3)"), Some("6".to_string())); // Multiple args now supported
    }
}
