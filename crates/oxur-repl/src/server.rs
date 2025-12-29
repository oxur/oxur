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
    #[allow(dead_code)]
    parser: Parser,
    #[allow(dead_code)]
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
            ReplRequest::Eval { source } => self.eval(&source),
            ReplRequest::Load { path } => self.load(&path),
            ReplRequest::Reset => {
                self.reset();
                Ok(ReplResponse::Ok)
            }
            ReplRequest::Status => Ok(ReplResponse::Status {
                tier1_count: self.stats.tier1_count,
                tier2_count: self.stats.tier2_count,
                tier3_count: self.stats.tier3_count,
            }),
            ReplRequest::Shutdown => Ok(ReplResponse::Ok),
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
        Ok(ReplResponse::Value { value: format!("interpreted: {}", source) })
    }

    fn eval_cached(&self, source: &str) -> Result<ReplResponse> {
        // Placeholder: cached function call
        if let Some(result) = self.cache.get(source) {
            Ok(ReplResponse::Value { value: result.clone() })
        } else {
            Ok(ReplResponse::Error { message: "Cache miss".to_string() })
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
    fn test_server_default() {
        let server = ReplServer::default();
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

    #[test]
    fn test_tier_selection_boundary() {
        let server = ReplServer::new();
        // Exactly 49 chars -> Tier 1
        let tier = server.choose_tier(&"x".repeat(49));
        assert!(matches!(tier, ExecutionTier::Interpreter));
        // Exactly 50 chars -> Tier 3
        let tier = server.choose_tier(&"x".repeat(50));
        assert!(matches!(tier, ExecutionTier::Jit));
    }

    #[test]
    fn test_handle_eval() {
        let mut server = ReplServer::new();
        let req = ReplRequest::Eval { source: "test".to_string() };
        let resp = server.handle(req);
        assert!(resp.is_ok());
    }

    #[test]
    fn test_handle_load() {
        let mut server = ReplServer::new();
        let req = ReplRequest::Load { path: "test.ox".to_string() };
        let resp = server.handle(req);
        assert!(resp.is_ok());
    }

    #[test]
    fn test_handle_reset() {
        let mut server = ReplServer::new();
        server.stats.tier1_count = 10;
        let req = ReplRequest::Reset;
        let resp = server.handle(req);
        assert!(resp.is_ok());
        match resp.unwrap() {
            ReplResponse::Ok => (),
            _ => panic!("Expected Ok response"),
        }
    }

    #[test]
    fn test_handle_status() {
        let mut server = ReplServer::new();
        server.stats.tier1_count = 1;
        server.stats.tier2_count = 2;
        server.stats.tier3_count = 3;
        let req = ReplRequest::Status;
        let resp = server.handle(req);
        assert!(resp.is_ok());
        match resp.unwrap() {
            ReplResponse::Status { tier1_count, tier2_count, tier3_count } => {
                assert_eq!(tier1_count, 1);
                assert_eq!(tier2_count, 2);
                assert_eq!(tier3_count, 3);
            }
            _ => panic!("Expected Status response"),
        }
    }

    #[test]
    fn test_handle_shutdown() {
        let mut server = ReplServer::new();
        let req = ReplRequest::Shutdown;
        let resp = server.handle(req);
        assert!(resp.is_ok());
        match resp.unwrap() {
            ReplResponse::Ok => (),
            _ => panic!("Expected Ok response"),
        }
    }

    #[test]
    fn test_tier_stats() {
        let stats = TierStats { tier1_count: 5, tier2_count: 10, tier3_count: 15 };
        assert_eq!(stats.tier1_count, 5);
        assert_eq!(stats.tier2_count, 10);
        assert_eq!(stats.tier3_count, 15);
    }

    #[test]
    fn test_tier_stats_default() {
        let stats = TierStats::default();
        assert_eq!(stats.tier1_count, 0);
        assert_eq!(stats.tier2_count, 0);
        assert_eq!(stats.tier3_count, 0);
    }

    #[test]
    fn test_eval_short_source() {
        let mut server = ReplServer::new();
        let req = ReplRequest::Eval { source: "short".to_string() };
        let resp = server.handle(req);
        assert!(resp.is_ok());
        assert_eq!(server.stats.tier1_count, 1);
    }

    #[test]
    fn test_eval_long_source() {
        let mut server = ReplServer::new();
        let req = ReplRequest::Eval { source: "x".repeat(100) };
        let resp = server.handle(req);
        assert!(resp.is_ok());
        assert_eq!(server.stats.tier3_count, 1);
    }

    #[test]
    fn test_reset_clears_cache() {
        let mut server = ReplServer::new();
        // Add something to cache via JIT
        let req = ReplRequest::Eval { source: "x".repeat(100) };
        server.handle(req).unwrap();
        assert!(server.cache.len() > 0);

        // Reset should clear it
        server.handle(ReplRequest::Reset).unwrap();
        assert_eq!(server.cache.len(), 0);
    }

    #[test]
    fn test_reset_clears_stats() {
        let mut server = ReplServer::new();
        server.stats.tier1_count = 10;
        server.stats.tier2_count = 20;
        server.stats.tier3_count = 30;

        server.handle(ReplRequest::Reset).unwrap();
        assert_eq!(server.stats.tier1_count, 0);
        assert_eq!(server.stats.tier2_count, 0);
        assert_eq!(server.stats.tier3_count, 0);
    }
}
