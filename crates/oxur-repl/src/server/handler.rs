//! Message Handler for REPL Server
//!
//! Processes incoming protocol messages and dispatches to SessionManager.
//! Handles request/response translation and error mapping.
//!
//! Based on ODD-0018: Oxur Remote REPL Protocol Design

use crate::protocol::{
    ErrorInfo, ErrorKind, Operation, OperationResult, Request, Response, SessionInfo, Status,
};
use crate::server::{SessionError, SessionManager};

/// Message handler for REPL protocol requests
///
/// Bridges the protocol layer (Request/Response) with the
/// session management layer (SessionManager).
pub struct MessageHandler {
    /// Session manager
    session_manager: SessionManager,
}

impl MessageHandler {
    /// Create a new message handler
    pub fn new(session_manager: SessionManager) -> Self {
        Self { session_manager }
    }

    /// Handle a REPL request and return a response
    ///
    /// Dispatches to appropriate SessionManager method based on operation type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxur_repl::server::{MessageHandler, SessionManager};
    /// use oxur_repl::protocol::{Request, Operation, ReplMode};
    ///
    /// # async fn example() {
    /// let manager = SessionManager::new();
    /// let handler = MessageHandler::new(manager);
    ///
    /// let request = Request {
    ///     id: 1,
    ///     session_id: "test".to_string(),
    ///     operation: Operation::CreateSession { mode: ReplMode::Lisp },
    /// };
    ///
    /// let response = handler.handle(request).await;
    /// # }
    /// ```
    pub async fn handle(&self, request: Request) -> Response {
        let result = match &request.operation {
            Operation::CreateSession { mode } => {
                self.handle_create_session(&request.session_id, *mode)
            }

            Operation::Clone {
                source_session_id,
            } => self.handle_clone_session(source_session_id, &request.session_id),

            Operation::Eval { code, mode } => {
                self.handle_eval(&request.session_id, code, *mode).await
            }

            Operation::Close => self.handle_close(&request.session_id),

            Operation::LsSessions => self.handle_list_sessions(),

            // Operations not yet implemented
            Operation::LoadFile { .. }
            | Operation::Interrupt
            | Operation::Describe { .. }
            | Operation::History { .. }
            | Operation::ClearOutput => OperationResult::Error {
                error: ErrorInfo {
                    kind: ErrorKind::InvalidRequest,
                    message: "Operation not yet implemented".to_string(),
                    location: None,
                    details: None,
                },
                stdout: None,
                stderr: None,
            },
        };

        Response {
            request_id: request.id,
            session_id: request.session_id,
            result,
        }
    }

    /// Handle CreateSession operation
    fn handle_create_session(
        &self,
        session_id: &str,
        mode: crate::protocol::ReplMode,
    ) -> OperationResult {
        match self
            .session_manager
            .create(session_id.to_string(), mode)
        {
            Ok(_) => OperationResult::Success {
                status: Status {
                    tier: 0,
                    cached: false,
                    duration_ms: 0,
                },
                value: Some(format!("Session {} created", session_id)),
                stdout: None,
                stderr: None,
            },
            Err(e) => self.error_result(e),
        }
    }

    /// Handle Clone operation
    fn handle_clone_session(&self, source_id: &str, target_id: &str) -> OperationResult {
        match self
            .session_manager
            .clone_session(&source_id.to_string(), target_id.to_string())
        {
            Ok(_) => OperationResult::Success {
                status: Status {
                    tier: 0,
                    cached: false,
                    duration_ms: 0,
                },
                value: Some(format!("Session {} cloned to {}", source_id, target_id)),
                stdout: None,
                stderr: None,
            },
            Err(e) => self.error_result(e),
        }
    }

    /// Handle Eval operation
    async fn handle_eval(
        &self,
        session_id: &str,
        code: &str,
        _mode: crate::protocol::ReplMode,
    ) -> OperationResult {
        // Note: mode is ignored because session already has a mode set during creation
        match self
            .session_manager
            .eval(&session_id.to_string(), code)
            .await
        {
            Ok(result) => {
                // Convert ExecutionTier to u8
                let tier_num = match result.tier {
                    crate::eval::ExecutionTier::Calculator => 1,
                    crate::eval::ExecutionTier::CachedCompilation => 2,
                };

                OperationResult::Success {
                    status: Status {
                        tier: tier_num,
                        cached: tier_num == 2 && result.duration_ms < 10, // Heuristic for cached
                        duration_ms: result.duration_ms,
                    },
                    value: Some(result.value),
                    stdout: result.stdout,
                    stderr: result.stderr,
                }
            }
            Err(e) => self.error_result(e),
        }
    }

    /// Handle Close operation
    fn handle_close(&self, session_id: &str) -> OperationResult {
        match self.session_manager.close(&session_id.to_string()) {
            Ok(_) => OperationResult::Success {
                status: Status {
                    tier: 0,
                    cached: false,
                    duration_ms: 0,
                },
                value: Some(format!("Session {} closed", session_id)),
                stdout: None,
                stderr: None,
            },
            Err(e) => self.error_result(e),
        }
    }

    /// Handle LsSessions operation
    fn handle_list_sessions(&self) -> OperationResult {
        match self.session_manager.list() {
            Ok(sessions) => {
                // Convert internal SessionInfo to protocol SessionInfo
                let session_infos = sessions
                    .into_iter()
                    .map(|s| SessionInfo {
                        id: s.id,
                        mode: s.mode,
                        eval_count: s.eval_count,
                        created_at: s.created_at,
                    })
                    .collect();

                OperationResult::Sessions {
                    sessions: session_infos,
                }
            }
            Err(e) => self.error_result(e),
        }
    }

    /// Convert SessionError to error result
    fn error_result(&self, error: SessionError) -> OperationResult {
        let (kind, message) = match error {
            SessionError::NotFound(id) => {
                (ErrorKind::SessionNotFound, format!("Session not found: {}", id))
            }
            SessionError::AlreadyExists(id) => (
                ErrorKind::SessionAlreadyExists,
                format!("Session already exists: {}", id),
            ),
            SessionError::LockPoisoned => {
                (ErrorKind::InternalError, "Lock poisoned".to_string())
            }
        };

        OperationResult::Error {
            error: ErrorInfo {
                kind,
                message,
                location: None,
                details: None,
            },
            stdout: None,
            stderr: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ReplMode;

    #[tokio::test]
    async fn test_handle_create_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        let request = Request {
            id: 1,
            session_id: "test-1".to_string(),
            operation: Operation::CreateSession {
                mode: ReplMode::Lisp,
            },
        };

        let response = handler.handle(request).await;

        assert_eq!(response.request_id, 1);
        assert_eq!(response.session_id, "test-1");
        assert!(matches!(response.result, OperationResult::Success { .. }));
    }

    #[tokio::test]
    async fn test_handle_create_duplicate_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        // Create first session
        handler
            .handle(Request {
                id: 1,
                session_id: "test-1".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        // Try to create duplicate
        let response = handler
            .handle(Request {
                id: 2,
                session_id: "test-1".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        assert_eq!(response.request_id, 2);
        if let OperationResult::Error { error, .. } = response.result {
            assert_eq!(error.kind, ErrorKind::SessionAlreadyExists);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_handle_close_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        // Create session
        handler
            .handle(Request {
                id: 1,
                session_id: "test-1".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        // Close session
        let response = handler
            .handle(Request {
                id: 2,
                session_id: "test-1".to_string(),
                operation: Operation::Close,
            })
            .await;

        assert!(matches!(response.result, OperationResult::Success { .. }));
    }

    #[tokio::test]
    async fn test_handle_close_nonexistent_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        let response = handler
            .handle(Request {
                id: 1,
                session_id: "nonexistent".to_string(),
                operation: Operation::Close,
            })
            .await;

        if let OperationResult::Error { error, .. } = response.result {
            assert_eq!(error.kind, ErrorKind::SessionNotFound);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_handle_list_sessions() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        // Create two sessions
        handler
            .handle(Request {
                id: 1,
                session_id: "test-1".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        handler
            .handle(Request {
                id: 2,
                session_id: "test-2".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Sexpr,
                },
            })
            .await;

        // List sessions
        let response = handler
            .handle(Request {
                id: 3,
                session_id: String::new(), // session_id ignored for LsSessions
                operation: Operation::LsSessions,
            })
            .await;

        if let OperationResult::Sessions { sessions } = response.result {
            assert_eq!(sessions.len(), 2);
            assert_eq!(sessions[0].id, "test-1");
            assert_eq!(sessions[0].mode, ReplMode::Lisp);
            assert_eq!(sessions[1].id, "test-2");
            assert_eq!(sessions[1].mode, ReplMode::Sexpr);
        } else {
            panic!("Expected Sessions response");
        }
    }

    #[tokio::test]
    async fn test_handle_eval() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        // Create session
        handler
            .handle(Request {
                id: 1,
                session_id: "test".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        // Evaluate code
        let response = handler
            .handle(Request {
                id: 2,
                session_id: "test".to_string(),
                operation: Operation::Eval {
                    code: "(+ 1 2)".to_string(),
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        if let OperationResult::Success { value, .. } = response.result {
            assert_eq!(value, Some("3".to_string()));
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn test_handle_eval_nonexistent_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        let response = handler
            .handle(Request {
                id: 1,
                session_id: "nonexistent".to_string(),
                operation: Operation::Eval {
                    code: "(+ 1 2)".to_string(),
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        if let OperationResult::Error { error, .. } = response.result {
            assert_eq!(error.kind, ErrorKind::SessionNotFound);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_handle_clone_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        // Create source session
        handler
            .handle(Request {
                id: 1,
                session_id: "source".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        // Clone session (target_id comes from request.session_id)
        let response = handler
            .handle(Request {
                id: 2,
                session_id: "target".to_string(),
                operation: Operation::Clone {
                    source_session_id: "source".to_string(),
                },
            })
            .await;

        assert!(matches!(response.result, OperationResult::Success { .. }));
    }

    #[tokio::test]
    async fn test_handle_clone_nonexistent_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        let response = handler
            .handle(Request {
                id: 1,
                session_id: "target".to_string(),
                operation: Operation::Clone {
                    source_session_id: "nonexistent".to_string(),
                },
            })
            .await;

        if let OperationResult::Error { error, .. } = response.result {
            assert_eq!(error.kind, ErrorKind::SessionNotFound);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_handle_clone_to_existing_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        // Create source and target sessions
        handler
            .handle(Request {
                id: 1,
                session_id: "source".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        handler
            .handle(Request {
                id: 2,
                session_id: "target".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        // Try to clone to existing session
        let response = handler
            .handle(Request {
                id: 3,
                session_id: "target".to_string(),
                operation: Operation::Clone {
                    source_session_id: "source".to_string(),
                },
            })
            .await;

        if let OperationResult::Error { error, .. } = response.result {
            assert_eq!(error.kind, ErrorKind::SessionAlreadyExists);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_eval_with_output_capture() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        // Create session
        handler
            .handle(Request {
                id: 1,
                session_id: "test".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        // Evaluate code that produces output
        let response = handler
            .handle(Request {
                id: 2,
                session_id: "test".to_string(),
                operation: Operation::Eval {
                    code: r#"println!("Hello from REPL")"#.to_string(),
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        // Note: Output capture is simulated, so just verify eval succeeds
        assert!(matches!(response.result, OperationResult::Success { .. }));
    }

    #[tokio::test]
    async fn test_multiple_operations_same_session() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        // Create session
        handler
            .handle(Request {
                id: 1,
                session_id: "test".to_string(),
                operation: Operation::CreateSession {
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        // Multiple evaluations
        let response1 = handler
            .handle(Request {
                id: 2,
                session_id: "test".to_string(),
                operation: Operation::Eval {
                    code: "(+ 1 2)".to_string(),
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        let response2 = handler
            .handle(Request {
                id: 3,
                session_id: "test".to_string(),
                operation: Operation::Eval {
                    code: "(* 3 4)".to_string(),
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        assert!(matches!(response1.result, OperationResult::Success { .. }));
        assert!(matches!(response2.result, OperationResult::Success { .. }));

        // Close session
        let response3 = handler
            .handle(Request {
                id: 4,
                session_id: "test".to_string(),
                operation: Operation::Close,
            })
            .await;

        assert!(matches!(response3.result, OperationResult::Success { .. }));
    }

    #[tokio::test]
    async fn test_unimplemented_operation() {
        let manager = SessionManager::new();
        let handler = MessageHandler::new(manager);

        let response = handler
            .handle(Request {
                id: 1,
                session_id: "test".to_string(),
                operation: Operation::LoadFile {
                    path: "test.lisp".to_string(),
                    mode: ReplMode::Lisp,
                },
            })
            .await;

        if let OperationResult::Error { error, .. } = response.result {
            assert_eq!(error.kind, ErrorKind::InvalidRequest);
            assert!(error.message.contains("not yet implemented"));
        } else {
            panic!("Expected error response");
        }
    }
}
