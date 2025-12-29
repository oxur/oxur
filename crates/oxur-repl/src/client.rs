//! REPL Client
//!
//! Handles user interaction and communication with the REPL server.

use crate::{protocol::*, Result};

/// REPL client for user interaction
pub struct ReplClient {
    // In a full implementation, this would handle communication
    // with the server process
}

impl ReplClient {
    pub fn new() -> Self {
        Self {}
    }

    /// Send a request to the server
    pub fn send(&mut self, request: ReplRequest) -> Result<ReplResponse> {
        // Placeholder implementation
        match request {
            ReplRequest::Eval { .. } => {
                Ok(ReplResponse::Value {
                    value: "result".to_string(),
                })
            }
            ReplRequest::Status => {
                Ok(ReplResponse::Status {
                    tier1_count: 0,
                    tier2_count: 0,
                    tier3_count: 0,
                })
            }
            _ => Ok(ReplResponse::Ok),
        }
    }

    /// Run the interactive REPL loop
    pub fn run(&mut self) -> Result<()> {
        println!("Oxur REPL v0.1.0");
        println!("Type (exit) to quit");

        // Placeholder - would read from stdin and evaluate
        Ok(())
    }
}

impl Default for ReplClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = ReplClient::new();
    }
}
