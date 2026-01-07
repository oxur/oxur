// In-process transport for same-process REPL communication
//
// Provides zero-overhead message passing via Tokio channels.
// Used for the default interactive REPL mode where client and server
// run in the same process.

use super::traits::{Transport, TransportError, TransportReader, TransportWriter};
use crate::protocol::{Request, Response};
use tokio::sync::mpsc;

/// Create a connected pair of in-process transports
///
/// Returns (client, server) transport pair connected via channels.
/// Messages are passed directly without serialization for maximum performance.
///
/// # Example
///
/// ```
/// use oxur_repl::transport::inprocess::channel;
///
/// # async fn example() {
/// let (client, server) = channel();
///
/// // Client can send requests to server
/// // Server can send responses to client
/// # }
/// ```
pub fn channel() -> (InProcessClient, InProcessServer) {
    // Client -> Server: Requests
    let (request_tx, request_rx) = mpsc::unbounded_channel();
    // Server -> Client: Responses
    let (response_tx, response_rx) = mpsc::unbounded_channel();

    let client = InProcessClient { request_tx, response_rx, closed: false };

    let server = InProcessServer { request_rx, response_tx, closed: false };

    (client, server)
}

/// Client-side in-process transport
///
/// Sends requests to the server and receives responses.
#[derive(Debug)]
pub struct InProcessClient {
    request_tx: mpsc::UnboundedSender<Request>,
    response_rx: mpsc::UnboundedReceiver<Response>,
    closed: bool,
}

#[async_trait::async_trait]
impl Transport for InProcessClient {
    async fn send_request(&mut self, request: &Request) -> Result<(), TransportError> {
        if self.closed {
            return Err(TransportError::ConnectionClosed);
        }
        self.request_tx.send(request.clone()).map_err(|_| TransportError::ConnectionClosed)
    }

    async fn send_response(&mut self, _response: &Response) -> Result<(), TransportError> {
        // Clients don't send responses
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Clients cannot send responses",
        )))
    }

    async fn recv_request(&mut self) -> Result<Request, TransportError> {
        // Clients don't receive requests
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Clients cannot receive requests",
        )))
    }

    async fn recv_response(&mut self) -> Result<Response, TransportError> {
        if self.closed {
            return Err(TransportError::ConnectionClosed);
        }
        self.response_rx.recv().await.ok_or(TransportError::ConnectionClosed)
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.closed = true;
        Ok(())
    }
}

/// Server-side in-process transport
///
/// Receives requests from the client and sends responses.
#[derive(Debug)]
pub struct InProcessServer {
    request_rx: mpsc::UnboundedReceiver<Request>,
    response_tx: mpsc::UnboundedSender<Response>,
    closed: bool,
}

#[async_trait::async_trait]
impl Transport for InProcessServer {
    async fn send_request(&mut self, _request: &Request) -> Result<(), TransportError> {
        // Servers don't send requests
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Servers cannot send requests",
        )))
    }

    async fn send_response(&mut self, response: &Response) -> Result<(), TransportError> {
        if self.closed {
            return Err(TransportError::ConnectionClosed);
        }
        self.response_tx.send(response.clone()).map_err(|_| TransportError::ConnectionClosed)
    }

    async fn recv_request(&mut self) -> Result<Request, TransportError> {
        if self.closed {
            return Err(TransportError::ConnectionClosed);
        }
        self.request_rx.recv().await.ok_or(TransportError::ConnectionClosed)
    }

    async fn recv_response(&mut self) -> Result<Response, TransportError> {
        // Servers don't receive responses
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Servers cannot receive responses",
        )))
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.closed = true;
        Ok(())
    }
}

/// Split client transport (reader receives responses, writer sends requests)
pub struct InProcessClientReader {
    response_rx: mpsc::UnboundedReceiver<Response>,
}

pub struct InProcessClientWriter {
    request_tx: mpsc::UnboundedSender<Request>,
}

#[async_trait::async_trait]
impl TransportReader for InProcessClientReader {
    async fn recv_request(&mut self) -> Result<Request, TransportError> {
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Client reader cannot receive requests",
        )))
    }

    async fn recv_response(&mut self) -> Result<Response, TransportError> {
        self.response_rx.recv().await.ok_or(TransportError::ConnectionClosed)
    }
}

#[async_trait::async_trait]
impl TransportWriter for InProcessClientWriter {
    async fn send_request(&mut self, request: &Request) -> Result<(), TransportError> {
        self.request_tx.send(request.clone()).map_err(|_| TransportError::ConnectionClosed)
    }

    async fn send_response(&mut self, _response: &Response) -> Result<(), TransportError> {
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Client writer cannot send responses",
        )))
    }

    async fn flush(&mut self) -> Result<(), TransportError> {
        // Channels don't need flushing
        Ok(())
    }
}

/// Split server transport (reader receives requests, writer sends responses)
pub struct InProcessServerReader {
    request_rx: mpsc::UnboundedReceiver<Request>,
}

pub struct InProcessServerWriter {
    response_tx: mpsc::UnboundedSender<Response>,
}

#[async_trait::async_trait]
impl TransportReader for InProcessServerReader {
    async fn recv_request(&mut self) -> Result<Request, TransportError> {
        self.request_rx.recv().await.ok_or(TransportError::ConnectionClosed)
    }

    async fn recv_response(&mut self) -> Result<Response, TransportError> {
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Server reader cannot receive responses",
        )))
    }
}

#[async_trait::async_trait]
impl TransportWriter for InProcessServerWriter {
    async fn send_request(&mut self, _request: &Request) -> Result<(), TransportError> {
        Err(TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Server writer cannot send requests",
        )))
    }

    async fn send_response(&mut self, response: &Response) -> Result<(), TransportError> {
        self.response_tx.send(response.clone()).map_err(|_| TransportError::ConnectionClosed)
    }

    async fn flush(&mut self) -> Result<(), TransportError> {
        // Channels don't need flushing
        Ok(())
    }
}

impl InProcessClient {
    /// Split into reader and writer halves for concurrent use
    pub fn split(self) -> (InProcessClientReader, InProcessClientWriter) {
        (
            InProcessClientReader { response_rx: self.response_rx },
            InProcessClientWriter { request_tx: self.request_tx },
        )
    }
}

impl InProcessServer {
    /// Split into reader and writer halves for concurrent use
    pub fn split(self) -> (InProcessServerReader, InProcessServerWriter) {
        (
            InProcessServerReader { request_rx: self.request_rx },
            InProcessServerWriter { response_tx: self.response_tx },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MessageId, Operation, OperationResult, ReplMode, SessionId, Status};

    #[tokio::test]
    async fn test_channel_creation() {
        let (_client, _server) = channel();
    }

    #[tokio::test]
    async fn test_send_request_recv_request() {
        let (mut client, mut server) = channel();

        let request = Request {
            id: MessageId::new(1),
            session_id: SessionId::new("test-session"),
            operation: Operation::Eval { code: "(+ 1 2)".to_string(), mode: ReplMode::Lisp },
        };

        // Client sends request
        client.send_request(&request).await.unwrap();

        // Server receives request
        let received = server.recv_request().await.unwrap();
        assert_eq!(received.id, MessageId::new(1));
        assert_eq!(received.session_id, SessionId::new("test-session"));
    }

    #[tokio::test]
    async fn test_send_response_recv_response() {
        let (mut client, mut server) = channel();

        let response = Response {
            request_id: MessageId::new(42),
            session_id: SessionId::new("test-session"),
            result: OperationResult::Success {
                status: Status { tier: 1, cached: false, duration_ms: 5 },
                value: Some("3".to_string()),
                stdout: None,
                stderr: None,
            },
        };

        // Server sends response
        server.send_response(&response).await.unwrap();

        // Client receives response
        let received = client.recv_response().await.unwrap();
        assert_eq!(received.request_id, MessageId::new(42));
    }

    #[tokio::test]
    async fn test_bidirectional_communication() {
        let (mut client, mut server) = channel();

        let request = Request {
            id: MessageId::new(1),
            session_id: SessionId::new("session-1"),
            operation: Operation::Eval { code: "(+ 2 3)".to_string(), mode: ReplMode::Lisp },
        };

        let response = Response {
            request_id: MessageId::new(1),
            session_id: SessionId::new("session-1"),
            result: OperationResult::Success {
                status: Status { tier: 1, cached: false, duration_ms: 2 },
                value: Some("5".to_string()),
                stdout: None,
                stderr: None,
            },
        };

        // Client sends request
        client.send_request(&request).await.unwrap();

        // Server receives request
        let recv_request = server.recv_request().await.unwrap();
        assert_eq!(recv_request.id, MessageId::new(1));

        // Server sends response
        server.send_response(&response).await.unwrap();

        // Client receives response
        let recv_response = client.recv_response().await.unwrap();
        assert_eq!(recv_response.request_id, MessageId::new(1));
        if let OperationResult::Success { value, .. } = recv_response.result {
            assert_eq!(value, Some("5".to_string()));
        } else {
            panic!("Expected success result");
        }
    }

    #[tokio::test]
    async fn test_multiple_requests() {
        let (mut client, mut server) = channel();

        // Send multiple requests
        for i in 1..=5 {
            let request = Request {
                id: MessageId::new(i),
                session_id: SessionId::new("test"),
                operation: Operation::Eval {
                    code: format!("(+ {} {})", i, i),
                    mode: ReplMode::Lisp,
                },
            };
            client.send_request(&request).await.unwrap();
        }

        // Receive all requests
        for i in 1..=5 {
            let received = server.recv_request().await.unwrap();
            assert_eq!(received.id, MessageId::new(i));
        }
    }

    #[tokio::test]
    async fn test_connection_closed_on_drop() {
        let (client, mut server) = channel();

        // Drop client
        drop(client);

        // Server should get ConnectionClosed
        let result = server.recv_request().await;
        assert!(matches!(result, Err(TransportError::ConnectionClosed)));
    }

    #[tokio::test]
    async fn test_client_cannot_send_response() {
        let (mut client, _server) = channel();

        let response = Response {
            request_id: MessageId::new(1),
            session_id: SessionId::new("test"),
            result: OperationResult::Success {
                status: Status { tier: 1, cached: false, duration_ms: 0 },
                value: None,
                stdout: None,
                stderr: None,
            },
        };

        let result = client.send_response(&response).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_server_cannot_send_request() {
        let (_client, mut server) = channel();

        let request = Request {
            id: MessageId::new(1),
            session_id: SessionId::new("test"),
            operation: Operation::LsSessions,
        };

        let result = server.send_request(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close() {
        let (mut client, mut server) = channel();

        // Close client
        client.close().await.unwrap();

        // Further operations should fail
        let request = Request {
            id: MessageId::new(1),
            session_id: SessionId::new("test"),
            operation: Operation::LsSessions,
        };
        let result = client.send_request(&request).await;
        assert!(matches!(result, Err(TransportError::ConnectionClosed)));

        // Close server
        server.close().await.unwrap();
        let result = server.recv_request().await;
        assert!(matches!(result, Err(TransportError::ConnectionClosed)));
    }

    #[tokio::test]
    async fn test_split_client() {
        let (client, mut server) = channel();
        let (mut reader, mut writer) = client.split();

        let request = Request {
            id: MessageId::new(99),
            session_id: SessionId::new("split-test"),
            operation: Operation::LsSessions,
        };

        let response = Response {
            request_id: MessageId::new(99),
            session_id: SessionId::new("split-test"),
            result: OperationResult::Success {
                status: Status { tier: 1, cached: false, duration_ms: 1 },
                value: None,
                stdout: None,
                stderr: None,
            },
        };

        // Writer sends request
        writer.send_request(&request).await.unwrap();

        // Server receives and responds
        let _ = server.recv_request().await.unwrap();
        server.send_response(&response).await.unwrap();

        // Reader receives response
        let recv = reader.recv_response().await.unwrap();
        assert_eq!(recv.request_id, MessageId::new(99));
    }

    #[tokio::test]
    async fn test_split_server() {
        let (mut client, server) = channel();
        let (mut reader, mut writer) = server.split();

        let request = Request {
            id: MessageId::new(77),
            session_id: SessionId::new("split-server-test"),
            operation: Operation::CreateSession { mode: ReplMode::Lisp },
        };

        let response = Response {
            request_id: MessageId::new(77),
            session_id: SessionId::new("split-server-test"),
            result: OperationResult::Success {
                status: Status { tier: 1, cached: false, duration_ms: 0 },
                value: None,
                stdout: None,
                stderr: None,
            },
        };

        // Client sends request
        client.send_request(&request).await.unwrap();

        // Server reader receives
        let recv_req = reader.recv_request().await.unwrap();
        assert_eq!(recv_req.id, MessageId::new(77));

        // Server writer responds
        writer.send_response(&response).await.unwrap();

        // Client receives response
        let recv_resp = client.recv_response().await.unwrap();
        assert_eq!(recv_resp.request_id, MessageId::new(77));
    }

    #[tokio::test]
    async fn test_concurrent_usage() {
        let (client, server) = channel();
        let (mut client_reader, mut client_writer) = client.split();
        let (mut server_reader, mut server_writer) = server.split();

        // Spawn client writer
        let client_write_handle = tokio::spawn(async move {
            for i in 1..=10 {
                let request = Request {
                    id: MessageId::new(i),
                    session_id: SessionId::new("concurrent"),
                    operation: Operation::Eval {
                        code: format!("(+ {} 1)", i),
                        mode: ReplMode::Lisp,
                    },
                };
                client_writer.send_request(&request).await.unwrap();
            }
        });

        // Spawn server reader + writer (echo back)
        let server_handle = tokio::spawn(async move {
            for _ in 1..=10 {
                let req = server_reader.recv_request().await.unwrap();
                let response = Response {
                    request_id: req.id,
                    session_id: req.session_id.clone(),
                    result: OperationResult::Success {
                        status: Status { tier: 1, cached: false, duration_ms: 1 },
                        value: Some("ok".to_string()),
                        stdout: None,
                        stderr: None,
                    },
                };
                server_writer.send_response(&response).await.unwrap();
            }
        });

        // Client reader collects responses
        let client_read_handle = tokio::spawn(async move {
            let mut count = 0;
            for _ in 1..=10 {
                let _resp = client_reader.recv_response().await.unwrap();
                count += 1;
            }
            count
        });

        client_write_handle.await.unwrap();
        server_handle.await.unwrap();
        let received_count = client_read_handle.await.unwrap();

        // Should have received all 10 responses
        assert_eq!(received_count, 10);
    }
}
