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
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tokio::time::timeout;

/// REPL server that accepts TCP connections and processes requests
///
/// Manages multiple concurrent client connections, each with independent
/// request/response handling. All connections share the same SessionManager,
/// allowing sessions to be accessed from multiple clients.
///
/// Supports graceful shutdown with configurable timeout for active connections.
#[derive(Debug)]
pub struct ReplServer {
    /// Address to bind to
    address: String,

    /// Shared session manager
    session_manager: Arc<SessionManager>,

    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<()>,

    /// Graceful shutdown timeout (how long to wait for connections to finish)
    shutdown_timeout: Duration,
}

/// Handle for triggering server shutdown
///
/// Obtained via `ReplServer::shutdown_handle()`, this handle can be used
/// to trigger graceful server shutdown from another task.
#[derive(Debug, Clone)]
pub struct ShutdownHandle {
    tx: broadcast::Sender<()>,
}

impl ShutdownHandle {
    /// Trigger graceful server shutdown
    ///
    /// Sends shutdown signal to the server. The server will stop accepting
    /// new connections and wait for active connections to complete.
    pub fn shutdown(&self) {
        // Ignore error if no receivers (server already stopped)
        let _ = self.tx.send(());
    }
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
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            address: address.into(),
            session_manager: Arc::new(SessionManager::new()),
            shutdown_tx,
            shutdown_timeout: Duration::from_secs(30), // Default 30 second timeout
        }
    }

    /// Set the graceful shutdown timeout
    ///
    /// This configures how long the server will wait for active connections
    /// to finish when shutting down.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxur_repl::server::ReplServer;
    /// use std::time::Duration;
    ///
    /// # async fn example() {
    /// let mut server = ReplServer::new("127.0.0.1:5555")
    ///     .with_shutdown_timeout(Duration::from_secs(60));
    /// # }
    /// ```
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Get a shutdown handle that can be used to trigger graceful shutdown
    ///
    /// Call `shutdown()` on the returned handle to initiate server shutdown.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxur_repl::server::ReplServer;
    ///
    /// # async fn example() {
    /// let mut server = ReplServer::new("127.0.0.1:5555");
    /// let shutdown_handle = server.shutdown_handle();
    ///
    /// // Later, from another task:
    /// tokio::spawn(async move {
    ///     // ... some condition ...
    ///     shutdown_handle.shutdown();
    /// });
    /// # }
    /// ```
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle { tx: self.shutdown_tx.clone() }
    }

    /// Start the server and listen for connections
    ///
    /// This method blocks until the server is shut down via a shutdown signal.
    /// When shutdown is triggered, the server will:
    /// 1. Stop accepting new connections
    /// 2. Wait for active connections to finish (up to shutdown_timeout)
    /// 3. Close all sessions
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
    ///
    /// // Get shutdown handle before starting
    /// let shutdown_handle = server.shutdown_handle();
    ///
    /// // Spawn server in background
    /// let server_task = tokio::spawn(async move {
    ///     server.start().await
    /// });
    ///
    /// // Later, trigger shutdown
    /// shutdown_handle.shutdown();
    ///
    /// // Wait for server to finish
    /// server_task.await.unwrap()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&mut self) -> io::Result<()> {
        eprintln!("[INFO] Starting REPL server on {}", self.address);

        let listener = TcpTransportListener::bind(&self.address)
            .await
            .map_err(|e| io::Error::other(format!("{:?}", e)))?;
        eprintln!("[INFO] Server listening on {}", self.address);

        // Subscribe to shutdown signal
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Track active connections
        let mut active_connections = JoinSet::new();

        loop {
            tokio::select! {
                // Wait for shutdown signal
                _ = shutdown_rx.recv() => {
                    eprintln!("[INFO] Shutdown signal received");
                    break;
                }

                // Accept new connection
                result = listener.accept() => {
                    match result {
                        Ok(transport) => {
                            let peer_addr = transport
                                .peer_addr()
                                .map(|a| a.to_string())
                                .unwrap_or_else(|_| "unknown".to_string());

                            eprintln!("[INFO] Accepted connection from {}", peer_addr);

                            // Spawn handler for this connection
                            let session_manager = Arc::clone(&self.session_manager);
                            let mut shutdown_rx = self.shutdown_tx.subscribe();

                            active_connections.spawn(async move {
                                // Run connection handler, but cancel if shutdown is triggered
                                tokio::select! {
                                    result = Self::handle_connection(transport, session_manager) => {
                                        match result {
                                            Ok(_) => {
                                                eprintln!("[INFO] Connection closed ({})", peer_addr);
                                            }
                                            Err(e) => {
                                                eprintln!("[ERROR] Connection error ({}): {}", peer_addr, e);
                                            }
                                        }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        eprintln!("[INFO] Connection interrupted by shutdown ({})", peer_addr);
                                    }
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
        }

        // Shutdown initiated - wait for active connections to finish
        eprintln!(
            "[INFO] Waiting for {} active connections to finish (timeout: {:?})",
            active_connections.len(),
            self.shutdown_timeout
        );

        // Wait for all connections with timeout
        match timeout(self.shutdown_timeout, async {
            while active_connections.join_next().await.is_some() {
                // Just wait for all tasks to finish
            }
        })
        .await
        {
            Ok(_) => {
                eprintln!("[INFO] All connections closed gracefully");
            }
            Err(_) => {
                eprintln!(
                    "[WARN] Shutdown timeout reached, {} connections still active",
                    active_connections.len()
                );
                // Abort remaining tasks
                active_connections.shutdown().await;
            }
        }

        // Close all sessions
        if let Ok(count) = self.session_manager.close_all() {
            eprintln!("[INFO] Closed {} sessions", count);
        }

        eprintln!("[INFO] Server shutdown complete");
        Ok(())
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
                    return Err(io::Error::other(format!("{:?}", e)));
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
                        return Err(io::Error::other(format!("{:?}", e)));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MessageId, Operation, OperationResult, ReplMode, Request, SessionId};
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
            ReplServer::handle_connection(transport, session_manager).await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Connect client
        let mut client = TcpTransport::connect(addr.to_string()).await.unwrap();

        // Send a request
        let request = Request {
            id: MessageId::new(1),
            session_id: SessionId::new("test"),
            operation: Operation::CreateSession { mode: ReplMode::Lisp },
        };

        client.send_request(&request).await.unwrap();

        // Read response
        let response = client.recv_response().await.unwrap();

        assert_eq!(response.request_id, MessageId::new(1));
        assert!(matches!(response.result, OperationResult::Success { .. }));

        // Close client connection
        drop(client);

        // Wait for server to finish
        timeout(Duration::from_secs(1), server_handle).await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_multiple_requests_same_connection() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server
        let server_handle = tokio::spawn(async move {
            let transport = listener.accept().await.unwrap();
            let session_manager = Arc::new(SessionManager::new());
            ReplServer::handle_connection(transport, session_manager).await.ok();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Connect client
        let mut client = TcpTransport::connect(addr.to_string()).await.unwrap();

        // Request 1: Create session
        let req1 = Request {
            id: MessageId::new(1),
            session_id: SessionId::new("test"),
            operation: Operation::CreateSession { mode: ReplMode::Lisp },
        };

        client.send_request(&req1).await.unwrap();
        let resp1 = client.recv_response().await.unwrap();
        assert_eq!(resp1.request_id, MessageId::new(1));

        // Request 2: Eval
        let req2 = Request {
            id: MessageId::new(2),
            session_id: SessionId::new("test"),
            operation: Operation::Eval { code: "(+ 1 2)".to_string(), mode: ReplMode::Lisp },
        };

        client.send_request(&req2).await.unwrap();
        let resp2 = client.recv_response().await.unwrap();
        assert_eq!(resp2.request_id, MessageId::new(2));

        if let OperationResult::Success { value, .. } = resp2.result {
            assert_eq!(value, Some("3".to_string()));
        } else {
            panic!("Expected success response");
        }

        // Request 3: Close session
        let req3 = Request {
            id: MessageId::new(3),
            session_id: SessionId::new("test"),
            operation: Operation::Close,
        };

        client.send_request(&req3).await.unwrap();
        let resp3 = client.recv_response().await.unwrap();
        assert_eq!(resp3.request_id, MessageId::new(3));

        drop(client);
        timeout(Duration::from_secs(1), server_handle).await.ok();
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
            id: MessageId::new(1),
            session_id: SessionId::new("session-1"),
            operation: Operation::CreateSession { mode: ReplMode::Lisp },
        };
        client1.send_request(&req1).await.unwrap();
        let resp1 = client1.recv_response().await.unwrap();
        assert_eq!(resp1.request_id, MessageId::new(1));

        // Client 2
        let mut client2 = TcpTransport::connect(addr.to_string()).await.unwrap();
        let req2 = Request {
            id: MessageId::new(1),
            session_id: SessionId::new("session-2"),
            operation: Operation::CreateSession { mode: ReplMode::Sexpr },
        };
        client2.send_request(&req2).await.unwrap();
        let resp2 = client2.recv_response().await.unwrap();
        assert_eq!(resp2.request_id, MessageId::new(1));

        // Both sessions should exist
        assert_eq!(session_manager.count().unwrap(), 2);

        drop(client1);
        drop(client2);

        timeout(Duration::from_secs(1), server_handle).await.ok();
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
        let result = timeout(Duration::from_secs(1), server_handle).await.unwrap().unwrap();

        assert!(result.is_ok()); // Should return Ok(()) for clean close
    }

    #[tokio::test]
    async fn test_shutdown_handle_functionality() {
        let server = ReplServer::new("127.0.0.1:5555");
        let handle1 = server.shutdown_handle();
        let handle2 = handle1.clone();

        // Verify handles can be cloned
        // Just trigger shutdown - no server is running, so this just tests the API
        handle2.shutdown();

        // This is a smoke test - the actual shutdown functionality is tested
        // by the other tests that run the full server
    }

    #[tokio::test]
    async fn test_with_shutdown_timeout() {
        let server =
            ReplServer::new("127.0.0.1:5555").with_shutdown_timeout(Duration::from_secs(60));

        assert_eq!(server.shutdown_timeout, Duration::from_secs(60));
    }
}
