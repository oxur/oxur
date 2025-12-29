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
            ReplRequest::Eval { .. } => Ok(ReplResponse::Value { value: "result".to_string() }),
            ReplRequest::Status => {
                Ok(ReplResponse::Status { tier1_count: 0, tier2_count: 0, tier3_count: 0 })
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

    #[test]
    fn test_client_default() {
        let _client = ReplClient::default();
    }

    #[test]
    fn test_send_eval() {
        let mut client = ReplClient::new();
        let req = ReplRequest::Eval { source: "(+ 1 2)".to_string() };
        let resp = client.send(req);
        assert!(resp.is_ok());
        match resp.unwrap() {
            ReplResponse::Value { value } => assert_eq!(value, "result"),
            _ => panic!("Expected Value response"),
        }
    }

    #[test]
    fn test_send_status() {
        let mut client = ReplClient::new();
        let req = ReplRequest::Status;
        let resp = client.send(req);
        assert!(resp.is_ok());
        match resp.unwrap() {
            ReplResponse::Status { tier1_count, tier2_count, tier3_count } => {
                assert_eq!(tier1_count, 0);
                assert_eq!(tier2_count, 0);
                assert_eq!(tier3_count, 0);
            }
            _ => panic!("Expected Status response"),
        }
    }

    #[test]
    fn test_send_reset() {
        let mut client = ReplClient::new();
        let req = ReplRequest::Reset;
        let resp = client.send(req);
        assert!(resp.is_ok());
    }

    #[test]
    fn test_send_load() {
        let mut client = ReplClient::new();
        let req = ReplRequest::Load { path: "test.ox".to_string() };
        let resp = client.send(req);
        assert!(resp.is_ok());
    }

    #[test]
    fn test_send_shutdown() {
        let mut client = ReplClient::new();
        let req = ReplRequest::Shutdown;
        let resp = client.send(req);
        assert!(resp.is_ok());
    }

    #[test]
    fn test_run_does_not_panic() {
        let mut client = ReplClient::new();
        let result = client.run();
        assert!(result.is_ok());
    }
}
