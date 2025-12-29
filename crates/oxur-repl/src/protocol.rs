//! REPL Protocol
//!
//! Defines the communication protocol between REPL client and server.

use serde::{Deserialize, Serialize};

/// Request from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplRequest {
    /// Evaluate an expression
    Eval { source: String },

    /// Load a file
    Load { path: String },

    /// Reset the REPL state
    Reset,

    /// Get REPL status
    Status,

    /// Shutdown the server
    Shutdown,
}

/// Response from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplResponse {
    /// Successful evaluation result
    Value { value: String },

    /// Evaluation error
    Error { message: String },

    /// Status information
    Status {
        tier1_count: usize,
        tier2_count: usize,
        tier3_count: usize,
    },

    /// Acknowledgment
    Ok,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = ReplRequest::Eval {
            source: "(+ 1 2)".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: ReplRequest = serde_json::from_str(&json).unwrap();

        match parsed {
            ReplRequest::Eval { source } => assert_eq!(source, "(+ 1 2)"),
            _ => panic!("Wrong variant"),
        }
    }
}
