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
    Status { tier1_count: usize, tier2_count: usize, tier3_count: usize },

    /// Acknowledgment
    Ok,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = ReplRequest::Eval { source: "(+ 1 2)".to_string() };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: ReplRequest = serde_json::from_str(&json).unwrap();

        match parsed {
            ReplRequest::Eval { source } => assert_eq!(source, "(+ 1 2)"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_request_load() {
        let req = ReplRequest::Load { path: "test.ox".to_string() };
        match req {
            ReplRequest::Load { path } => assert_eq!(path, "test.ox"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_request_reset() {
        let req = ReplRequest::Reset;
        match req {
            ReplRequest::Reset => (),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_request_status() {
        let req = ReplRequest::Status;
        match req {
            ReplRequest::Status => (),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_request_shutdown() {
        let req = ReplRequest::Shutdown;
        match req {
            ReplRequest::Shutdown => (),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_response_value() {
        let resp = ReplResponse::Value { value: "42".to_string() };
        match resp {
            ReplResponse::Value { value } => assert_eq!(value, "42"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_response_error() {
        let resp = ReplResponse::Error { message: "error".to_string() };
        match resp {
            ReplResponse::Error { message } => assert_eq!(message, "error"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_response_status() {
        let resp = ReplResponse::Status { tier1_count: 1, tier2_count: 2, tier3_count: 3 };
        match resp {
            ReplResponse::Status { tier1_count, tier2_count, tier3_count } => {
                assert_eq!(tier1_count, 1);
                assert_eq!(tier2_count, 2);
                assert_eq!(tier3_count, 3);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_response_ok() {
        let resp = ReplResponse::Ok;
        match resp {
            ReplResponse::Ok => (),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_request_clone() {
        let req = ReplRequest::Eval { source: "test".to_string() };
        let cloned = req.clone();
        match cloned {
            ReplRequest::Eval { source } => assert_eq!(source, "test"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_response_clone() {
        let resp = ReplResponse::Value { value: "test".to_string() };
        let cloned = resp.clone();
        match cloned {
            ReplResponse::Value { value } => assert_eq!(value, "test"),
            _ => panic!("Wrong variant"),
        }
    }
}
