//! REPL Server
//!
//! Implements three-tier execution strategy:
//! - Tier 1: Interpreter for simple expressions (<1ms)
//! - Tier 2: Cache of compiled functions (~0ms)
//! - Tier 3: JIT compilation for complex code (50-200ms first time)

use crate::{protocol::*, Result};
use oxur_lang::{Expander, Parser};
use std::collections::HashMap;

/// Execution tier for performance tracking
#[derive(Debug, Clone, Copy)]
enum ExecutionTier {
    Interpreter,
    Cached,
    Jit,
}

/// REPL server with tiered execution
pub struct ReplServer {
    parser: Parser,
    expander: Expander,
    cache: HashMap<String, String>,
    stats: TierStats,
}

#[derive(Debug, Default)]
struct TierStats {
    tier1_count: usize,
    tier2_count: usize,
    tier3_count: usize,
}

impl ReplServer {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(String::new()),
            expander: Expander::new(),
            cache: HashMap::new(),
            stats: TierStats::default(),
        }
    }

    /// Handle a REPL request
    pub fn handle(&mut self, request: ReplRequest) -> Result<ReplResponse> {
        match request {
            ReplRequest::Eval { source } => {
                self.eval(&source)
            }
            ReplRequest::Load { path } => {
                self.load(&path)
            }
            ReplRequest::Reset => {
                self.reset();
                Ok(ReplResponse::Ok)
            }
            ReplRequest::Status => {
                Ok(ReplResponse::Status {
                    tier1_count: self.stats.tier1_count,
                    tier2_count: self.stats.tier2_count,
                    tier3_count: self.stats.tier3_count,
                })
            }
            ReplRequest::Shutdown => {
                Ok(ReplResponse::Ok)
            }
        }
    }

    fn eval(&mut self, source: &str) -> Result<ReplResponse> {
        // Determine execution tier
        let tier = self.choose_tier(source);

        match tier {
            ExecutionTier::Interpreter => {
                self.stats.tier1_count += 1;
                self.eval_interpret(source)
            }
            ExecutionTier::Cached => {
                self.stats.tier2_count += 1;
                self.eval_cached(source)
            }
            ExecutionTier::Jit => {
                self.stats.tier3_count += 1;
                self.eval_jit(source)
            }
        }
    }

    fn choose_tier(&self, source: &str) -> ExecutionTier {
        // Simple heuristic - would be more sophisticated in practice
        if source.len() < 50 {
            ExecutionTier::Interpreter
        } else if self.cache.contains_key(source) {
            ExecutionTier::Cached
        } else {
            ExecutionTier::Jit
        }
    }

    fn eval_interpret(&mut self, source: &str) -> Result<ReplResponse> {
        // Placeholder: direct interpretation
        Ok(ReplResponse::Value {
            value: format!("interpreted: {}", source),
        })
    }

    fn eval_cached(&self, source: &str) -> Result<ReplResponse> {
        // Placeholder: cached function call
        if let Some(result) = self.cache.get(source) {
            Ok(ReplResponse::Value {
                value: result.clone(),
            })
        } else {
            Ok(ReplResponse::Error {
                message: "Cache miss".to_string(),
            })
        }
    }

    fn eval_jit(&mut self, source: &str) -> Result<ReplResponse> {
        // Placeholder: compile and execute
        let result = format!("jit-compiled: {}", source);
        self.cache.insert(source.to_string(), result.clone());
        Ok(ReplResponse::Value { value: result })
    }

    fn load(&mut self, _path: &str) -> Result<ReplResponse> {
        // Placeholder implementation
        Ok(ReplResponse::Ok)
    }

    fn reset(&mut self) {
        self.cache.clear();
        self.stats = TierStats::default();
    }
}

impl Default for ReplServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = ReplServer::new();
        assert_eq!(server.stats.tier1_count, 0);
    }

    #[test]
    fn test_tier_selection() {
        let server = ReplServer::new();

        // Short expression → Tier 1
        let tier = server.choose_tier("(+ 1 2)");
        assert!(matches!(tier, ExecutionTier::Interpreter));

        // Long expression → Tier 3
        let tier = server.choose_tier(&"x".repeat(100));
        assert!(matches!(tier, ExecutionTier::Jit));
    }
}
