//! REPL Server Implementation
//!
//! TCP-based server that accepts client connections and processes REPL requests.
//! Manages multiple concurrent sessions with shared SessionManager.
//!
//! Based on ODD-0018: Oxur Remote REPL Protocol Design

use crate::server::{MessageHandler, SessionManager};
use crate::transport::{TcpTransport, TcpTransportListener};
use std::io;
use std::sync::Arc;

/// REPL server that accepts TCP connections and processes requests
///
/// Manages multiple concurrent client connections, each with independent
/// request/response handling. All connections share the same SessionManager,
/// allowing sessions to be accessed from multiple clients.
pub struct ReplServer {
    /// Address to bind to
    address: String,

    /// Shared session manager
    session_manager: Arc<SessionManager>,
}

impl ReplServer {
    /// Create a new REPL server
    ///
    /// # Arguments
    ///
    /// * `address` - Address to bind to (e.g., "127.0.0.1:5555")
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxur_repl::server::ReplServer;
    ///
    /// # async fn example() {
    /// let server = ReplServer::new("127.0.0.1:5555");
    /// # }
    /// ```
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            session_manager: Arc::new(SessionManager::new()),
        }
    }

    /// Start the server and listen for connections
    ///
    /// This method blocks until the server is shut down.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Failed to bind to address
    /// - Failed to accept connections
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxur_repl::server::ReplServer;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let mut server = ReplServer::new("127.0.0.1:5555");
    /// server.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&mut self) -> io::Result<()> {
        eprintln!("[INFO] Starting REPL server on {}", self.address);

        let listener = TcpTransportListener::bind(&self.address)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
        eprintln!("[INFO] Server listening on {}", self.address);

        loop {
            match listener.accept().await {
                Ok(transport) => {
                    let peer_addr = transport
                        .peer_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| "unknown".to_string());

                    eprintln!("[INFO] Accepted connection from {}", peer_addr);

                    // Spawn handler for this connection
                    let session_manager = Arc::clone(&self.session_manager);
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(transport, session_manager).await {
                            eprintln!("[ERROR] Connection error ({}): {}", peer_addr, e);
                        } else {
                            eprintln!("[INFO] Connection closed ({})", peer_addr);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to accept connection: {}", e);
                    // Continue accepting other connections
                }
            }
        }
    }

    /// Handle a single client connection
    ///
    /// Reads requests from the transport, processes them via MessageHandler,
    /// and writes responses back.
    async fn handle_connection(
        mut transport: TcpTransport,
        session_manager: Arc<SessionManager>,
    ) -> io::Result<()> {
        use crate::transport::{Transport, TransportError};

        let handler = MessageHandler::new((*session_manager).clone());

        loop {
            // Read request using Transport trait
            let request = match transport.recv_request().await {
                Ok(req) => req,
                Err(TransportError::ConnectionClosed) => {
                    // Clean connection close - not an error
                    eprintln!("[INFO] Connection closed cleanly");
                    return Ok(());
                }
                Err(e) => {
                    // Other errors are actual errors
                    eprintln!("[ERROR] Failed to read request: {:?}", e);
                    return Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)));
                }
            };

            // Process request
            let response = handler.handle(request).await;

            // Write response using Transport trait
            if let Err(e) = transport.send_response(&response).await {
                match e {
                    TransportError::ConnectionClosed => {
                        // Client closed connection after we processed request - that's OK
                        eprintln!("[INFO] Connection closed after response");
                        return Ok(());
                    }
                    _ => {
                        eprintln!("[ERROR] Failed to write response: {:?}", e);
                        return Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)));
                    }
                }
            }
        }
    }

    /// Shutdown the server gracefully
    ///
    /// Closes all active sessions.
    pub async fn shutdown(self) {
        eprintln!("[INFO] Shutting down server...");

        // Close all sessions
        if let Ok(count) = self.session_manager.close_all() {
            eprintln!("[INFO] Closed {} sessions", count);
        }

        eprintln!("[INFO] Server shutdown complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Operation, OperationResult, ReplMode, Request};
    use crate::transport::Transport;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_server_creation() {
        let server = ReplServer::new("127.0.0.1:0");
        assert_eq!(server.address, "127.0.0.1:0");
    }

    #[tokio::test]
    async fn test_server_start_and_connect() {
        // Start server on random port
        let _server = ReplServer::new("127.0.0.1:0");

        // Get the actual bound address
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server in background
        let server_handle = tokio::spawn(async move {
            // Accept one connection then stop
            let transport = listener.accept().await.unwrap();
            let session_manager = Arc::new(SessionManager::new());
            ReplServer::handle_connection(transport, session_manager)
                .await
                .unwrap();
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Connect client
        let mut client = TcpTransport::connect(addr.to_string()).await.unwrap();

        // Send a request
        let request = Request {
            id: 1,
            session_id: "test".to_string(),
            operation: Operation::CreateSession {
                mode: ReplMode::Lisp,
            },
        };

        client.send_request(&request).await.unwrap();

        // Read response
        let response = client.recv_response().await.unwrap();

        assert_eq!(response.request_id, 1);
        assert!(matches!(response.result, OperationResult::Success { .. }));

        // Close client connection
        drop(client);

        // Wait for server to finish
        timeout(Duration::from_secs(1), server_handle)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn test_multiple_requests_same_connection() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server
        let server_handle = tokio::spawn(async move {
            let transport = listener.accept().await.unwrap();
            let session_manager = Arc::new(SessionManager::new());
            ReplServer::handle_connection(transport, session_manager)
                .await
                .ok();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Connect client
        let mut client = TcpTransport::connect(addr.to_string()).await.unwrap();

        // Request 1: Create session
        let req1 = Request {
            id: 1,
            session_id: "test".to_string(),
            operation: Operation::CreateSession {
                mode: ReplMode::Lisp,
            },
        };

        client.send_request(&req1).await.unwrap();
        let resp1 = client.recv_response().await.unwrap();
        assert_eq!(resp1.request_id, 1);

        // Request 2: Eval
        let req2 = Request {
            id: 2,
            session_id: "test".to_string(),
            operation: Operation::Eval {
                code: "(+ 1 2)".to_string(),
                mode: ReplMode::Lisp,
            },
        };

        client.send_request(&req2).await.unwrap();
        let resp2 = client.recv_response().await.unwrap();
        assert_eq!(resp2.request_id, 2);

        if let OperationResult::Success { value, .. } = resp2.result {
            assert_eq!(value, Some("3".to_string()));
        } else {
            panic!("Expected success response");
        }

        // Request 3: Close session
        let req3 = Request {
            id: 3,
            session_id: "test".to_string(),
            operation: Operation::Close,
        };

        client.send_request(&req3).await.unwrap();
        let resp3 = client.recv_response().await.unwrap();
        assert_eq!(resp3.request_id, 3);

        drop(client);
        timeout(Duration::from_secs(1), server_handle)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_concurrent_connections() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let session_manager = Arc::new(SessionManager::new());

        // Spawn server that handles 2 connections
        let sm = Arc::clone(&session_manager);
        let server_handle = tokio::spawn(async move {
            // Accept and handle first connection
            let transport1 = listener.accept().await.unwrap();
            let sm1 = Arc::clone(&sm);
            tokio::spawn(async move {
                ReplServer::handle_connection(transport1, sm1).await.ok();
            });

            // Accept and handle second connection
            let transport2 = listener.accept().await.unwrap();
            ReplServer::handle_connection(transport2, sm).await.ok();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Client 1
        let mut client1 = TcpTransport::connect(addr.to_string()).await.unwrap();
        let req1 = Request {
            id: 1,
            session_id: "session-1".to_string(),
            operation: Operation::CreateSession {
                mode: ReplMode::Lisp,
            },
        };
        client1.send_request(&req1).await.unwrap();
        let resp1 = client1.recv_response().await.unwrap();
        assert_eq!(resp1.request_id, 1);

        // Client 2
        let mut client2 = TcpTransport::connect(addr.to_string()).await.unwrap();
        let req2 = Request {
            id: 1,
            session_id: "session-2".to_string(),
            operation: Operation::CreateSession {
                mode: ReplMode::Sexpr,
            },
        };
        client2.send_request(&req2).await.unwrap();
        let resp2 = client2.recv_response().await.unwrap();
        assert_eq!(resp2.request_id, 1);

        // Both sessions should exist
        assert_eq!(session_manager.count().unwrap(), 2);

        drop(client1);
        drop(client2);

        timeout(Duration::from_secs(1), server_handle)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_connection_error_handling() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let transport = listener.accept().await.unwrap();
            let session_manager = Arc::new(SessionManager::new());
            ReplServer::handle_connection(transport, session_manager).await
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Connect and immediately close without sending anything
        let client = TcpTransport::connect(addr.to_string()).await.unwrap();
        drop(client);

        // Server should handle this gracefully (UnexpectedEof)
        let result = timeout(Duration::from_secs(1), server_handle)
            .await
            .unwrap()
            .unwrap();

        assert!(result.is_ok()); // Should return Ok(()) for clean close
    }
}
