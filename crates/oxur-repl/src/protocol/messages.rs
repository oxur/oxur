// Protocol message types for Oxur REPL communication
//
// Based on ODD-0018: Oxur Remote REPL Protocol Design
//
// This module defines the core message types for bidirectional
// communication between REPL client and server.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Universally unique identifier for REPL sessions
///
/// Format: UUID v4 as string (36 characters with hyphens)
/// Example: "550e8400-e29b-41d4-a716-446655440000"
pub type SessionId = String;

/// Monotonic message correlation identifier
///
/// Used to match responses to requests. Must be unique per session.
pub type MessageId = u64;

/// Source location in Oxur code
///
/// Tracks byte offsets and line/column positions for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Byte offset from start of input
    pub offset: usize,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Client request to REPL server
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// Unique identifier for this request
    pub id: MessageId,
    /// Session this request belongs to
    pub session_id: SessionId,
    /// The operation to perform
    pub operation: Operation,
}

/// Server response to client request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// ID of request this responds to
    pub request_id: MessageId,
    /// Session this response belongs to
    pub session_id: SessionId,
    /// The result of the operation
    pub result: OperationResult,
}

/// Operations the REPL server can perform
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Operation {
    /// Create a new session
    CreateSession {
        /// Evaluation mode (Lisp or Sexpr)
        mode: ReplMode,
    },

    /// Clone an existing session
    Clone {
        /// ID of session to clone from
        source_session_id: SessionId,
    },

    /// Evaluate code in the session
    Eval {
        /// Source code to evaluate
        code: String,
        /// Evaluation mode (Lisp or Sexpr)
        mode: ReplMode,
    },

    /// Load and evaluate code from a file
    LoadFile {
        /// Path to file (relative to server's working directory)
        path: String,
        /// Evaluation mode
        mode: ReplMode,
    },

    /// Interrupt running evaluation
    Interrupt,

    /// Close the session
    Close,

    /// List all active sessions
    LsSessions,

    /// Describe a symbol or value
    Describe {
        /// Symbol name to describe
        symbol: String,
    },

    /// Get evaluation history
    History {
        /// Maximum number of entries to return
        limit: Option<usize>,
    },

    /// Clear output buffer
    ClearOutput,
}

/// Result of an operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OperationResult {
    /// Operation succeeded
    Success {
        /// Status information
        status: Status,
        /// Optional result value
        value: Option<String>,
        /// Standard output
        stdout: Option<String>,
        /// Standard error
        stderr: Option<String>,
    },

    /// Operation failed
    Error {
        /// Error information
        error: ErrorInfo,
        /// Partial output before error
        stdout: Option<String>,
        /// Error output
        stderr: Option<String>,
    },

    /// Session list response
    Sessions {
        /// List of active sessions
        sessions: Vec<SessionInfo>,
    },

    /// History response
    HistoryEntries {
        /// History entries (most recent first)
        entries: Vec<HistoryEntry>,
    },
}

/// Evaluation mode for the REPL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReplMode {
    /// Lisp syntax with syntactic sugar
    Lisp,
    /// Raw S-expressions (canonical form)
    Sexpr,
}

impl fmt::Display for ReplMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplMode::Lisp => write!(f, "Lisp"),
            ReplMode::Sexpr => write!(f, "Sexpr"),
        }
    }
}

/// Status information after successful operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// Execution tier used (1 = Calculator, 2 = Cached Compilation)
    pub tier: u8,
    /// Whether result was cached
    pub cached: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

/// Session information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    pub id: SessionId,
    /// Evaluation mode (Lisp or Sexpr)
    pub mode: ReplMode,
    /// Number of evaluations performed
    pub eval_count: u64,
    /// Creation timestamp (milliseconds since epoch)
    pub created_at: u64,
}

/// Detailed error information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error classification
    pub kind: ErrorKind,
    /// Human-readable error message
    pub message: String,
    /// Source location where error occurred (if applicable)
    pub location: Option<SourceLocation>,
    /// Detailed backtrace or context
    pub details: Option<String>,
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)?;
        if let Some(loc) = &self.location {
            write!(f, " at {}", loc)?;
        }
        Ok(())
    }
}

/// Error classification for structured error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Syntax error in source code
    SyntaxError,
    /// Type error during compilation
    TypeError,
    /// Runtime evaluation error
    RuntimeError,
    /// Compilation failed
    CompilationError,
    /// Session not found
    SessionNotFound,
    /// Session already exists
    SessionAlreadyExists,
    /// File I/O error
    IoError,
    /// Operation interrupted
    Interrupted,
    /// Invalid request
    InvalidRequest,
    /// Internal server error
    InternalError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::SyntaxError => write!(f, "Syntax Error"),
            ErrorKind::TypeError => write!(f, "Type Error"),
            ErrorKind::RuntimeError => write!(f, "Runtime Error"),
            ErrorKind::CompilationError => write!(f, "Compilation Error"),
            ErrorKind::SessionNotFound => write!(f, "Session Not Found"),
            ErrorKind::SessionAlreadyExists => write!(f, "Session Already Exists"),
            ErrorKind::IoError => write!(f, "I/O Error"),
            ErrorKind::Interrupted => write!(f, "Interrupted"),
            ErrorKind::InvalidRequest => write!(f, "Invalid Request"),
            ErrorKind::InternalError => write!(f, "Internal Error"),
        }
    }
}

/// History entry from evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Entry number (monotonically increasing)
    pub number: usize,
    /// Code that was evaluated
    pub code: String,
    /// Result of evaluation (if successful)
    pub result: Option<String>,
    /// Timestamp in milliseconds since UNIX epoch
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation { offset: 42, line: 3, column: 15 };
        assert_eq!(loc.to_string(), "line 3, column 15");
    }

    #[test]
    fn test_repl_mode_display() {
        assert_eq!(ReplMode::Lisp.to_string(), "Lisp");
        assert_eq!(ReplMode::Sexpr.to_string(), "Sexpr");
    }

    #[test]
    fn test_error_kind_display() {
        assert_eq!(ErrorKind::SyntaxError.to_string(), "Syntax Error");
        assert_eq!(ErrorKind::RuntimeError.to_string(), "Runtime Error");
    }

    #[test]
    fn test_error_info_display() {
        let error = ErrorInfo {
            kind: ErrorKind::SyntaxError,
            message: "Unexpected token".to_string(),
            location: Some(SourceLocation { offset: 10, line: 2, column: 5 }),
            details: None,
        };
        assert_eq!(error.to_string(), "Syntax Error: Unexpected token at line 2, column 5");
    }

    #[test]
    fn test_request_roundtrip() {
        let request = Request {
            id: 1,
            session_id: "test-session".to_string(),
            operation: Operation::Eval { code: "(+ 1 2)".to_string(), mode: ReplMode::Lisp },
        };

        // Serialize to JSON for debugging
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_response_success_roundtrip() {
        let response = Response {
            request_id: 1,
            session_id: "test-session".to_string(),
            result: OperationResult::Success {
                status: Status { tier: 1, cached: false, duration_ms: 5 },
                value: Some("3".to_string()),
                stdout: None,
                stderr: None,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn test_response_error_roundtrip() {
        let response = Response {
            request_id: 2,
            session_id: "test-session".to_string(),
            result: OperationResult::Error {
                error: ErrorInfo {
                    kind: ErrorKind::RuntimeError,
                    message: "Division by zero".to_string(),
                    location: None,
                    details: None,
                },
                stdout: None,
                stderr: Some("Error occurred".to_string()),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}
