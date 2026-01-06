// TCP transport implementation for REPL communication
//
// Provides async TCP socket transport for network-based REPL connections.

use super::traits::{
    helpers, SplitTransport, Transport, TransportError, TransportReader, TransportWriter,
};
use crate::protocol::{Request, Response};
use std::net::SocketAddr;
use tokio::io::{AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};

/// TCP transport for REPL communication
///
/// Wraps a `TcpStream` and implements the `Transport` trait for async
/// message-based communication over TCP.
#[derive(Debug)]
pub struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    /// Create a new TCP transport from an existing stream
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Connect to a REPL server at the given address
    ///
    /// # Errors
    ///
    /// Returns error if connection fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_repl::transport::TcpTransport;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport = TcpTransport::connect("127.0.0.1:9000").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(addr: impl Into<String>) -> Result<Self, TransportError> {
        let addr_str = addr.into();
        let stream = TcpStream::connect(&addr_str)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("{}: {}", addr_str, e)))?;

        Ok(Self::new(stream))
    }

    /// Get the local address this transport is bound to
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    /// Get the remote address this transport is connected to
    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream.peer_addr()
    }
}

#[async_trait::async_trait]
impl Transport for TcpTransport {
    async fn send_request(&mut self, request: &Request) -> Result<(), TransportError> {
        helpers::send_request(&mut self.stream, request).await
    }

    async fn send_response(&mut self, response: &Response) -> Result<(), TransportError> {
        helpers::send_response(&mut self.stream, response).await
    }

    async fn recv_request(&mut self) -> Result<Request, TransportError> {
        helpers::recv_request(&mut self.stream).await
    }

    async fn recv_response(&mut self) -> Result<Response, TransportError> {
        helpers::recv_response(&mut self.stream).await
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.stream.shutdown().await?;
        Ok(())
    }
}

impl SplitTransport for TcpTransport {
    type Reader = TcpTransportReader;
    type Writer = TcpTransportWriter;

    fn split(self) -> (Self::Reader, Self::Writer) {
        let (read_half, write_half) = tokio::io::split(self.stream);
        (TcpTransportReader { reader: read_half }, TcpTransportWriter { writer: write_half })
    }
}

/// Reader half of a split TCP transport
#[derive(Debug)]
pub struct TcpTransportReader {
    reader: ReadHalf<TcpStream>,
}

#[async_trait::async_trait]
impl TransportReader for TcpTransportReader {
    async fn recv_request(&mut self) -> Result<Request, TransportError> {
        helpers::recv_request(&mut self.reader).await
    }

    async fn recv_response(&mut self) -> Result<Response, TransportError> {
        helpers::recv_response(&mut self.reader).await
    }
}

/// Writer half of a split TCP transport
#[derive(Debug)]
pub struct TcpTransportWriter {
    writer: WriteHalf<TcpStream>,
}

#[async_trait::async_trait]
impl TransportWriter for TcpTransportWriter {
    async fn send_request(&mut self, request: &Request) -> Result<(), TransportError> {
        helpers::send_request(&mut self.writer, request).await
    }

    async fn send_response(&mut self, response: &Response) -> Result<(), TransportError> {
        helpers::send_response(&mut self.writer, response).await
    }

    async fn flush(&mut self) -> Result<(), TransportError> {
        self.writer.flush().await?;
        Ok(())
    }
}

/// TCP listener for accepting REPL connections
///
/// Server-side TCP listener that accepts incoming client connections
/// and returns `TcpTransport` instances.
#[derive(Debug)]
pub struct TcpTransportListener {
    listener: TcpListener,
}

impl TcpTransportListener {
    /// Bind to the given address and start listening
    ///
    /// # Errors
    ///
    /// Returns error if binding fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_repl::transport::TcpTransportListener;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let listener = TcpTransportListener::bind("127.0.0.1:9000").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bind(addr: impl Into<String>) -> Result<Self, TransportError> {
        let addr_str = addr.into();
        let listener = TcpListener::bind(&addr_str)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("bind {}: {}", addr_str, e)))?;

        Ok(Self { listener })
    }

    /// Accept a new incoming connection
    ///
    /// Returns a `TcpTransport` for the accepted connection.
    ///
    /// # Errors
    ///
    /// Returns error if accept fails.
    pub async fn accept(&self) -> Result<TcpTransport, TransportError> {
        let (stream, _addr) = self.listener.accept().await?;
        Ok(TcpTransport::new(stream))
    }

    /// Get the local address this listener is bound to
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MessageId, Operation, OperationResult, ReplMode, SessionId, Status};

    #[tokio::test]
    async fn test_tcp_connect_accept() {
        // Start a listener on localhost
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Connect to it
        let client_handle =
            tokio::spawn(async move { TcpTransport::connect(addr.to_string()).await.unwrap() });

        // Accept the connection
        let server_transport = listener.accept().await.unwrap();

        let client_transport = client_handle.await.unwrap();

        // Verify we're connected
        assert!(client_transport.peer_addr().is_ok());
        assert!(server_transport.peer_addr().is_ok());
    }

    #[tokio::test]
    async fn test_tcp_send_recv_request() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let request = Request {
            id: MessageId::new(1),
            session_id: SessionId::new("test-session"),
            operation: Operation::Eval { code: "(+ 1 2)".to_string(), mode: ReplMode::Lisp },
        };

        let request_clone = request.clone();

        // Client sends request
        let client_handle = tokio::spawn(async move {
            let mut client = TcpTransport::connect(addr.to_string()).await.unwrap();
            client.send_request(&request_clone).await.unwrap();
        });

        // Server receives request
        let server_handle = tokio::spawn(async move {
            let mut server = listener.accept().await.unwrap();
            let received = server.recv_request().await.unwrap();
            assert_eq!(received.id, MessageId::new(1));
            assert_eq!(received.session_id, SessionId::new("test-session"));
        });

        client_handle.await.unwrap();
        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_tcp_send_recv_response() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

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

        let response_clone = response.clone();

        // Server sends response
        let server_handle = tokio::spawn(async move {
            let mut server = listener.accept().await.unwrap();
            server.send_response(&response_clone).await.unwrap();
        });

        // Client receives response
        let client_handle = tokio::spawn(async move {
            let mut client = TcpTransport::connect(addr.to_string()).await.unwrap();
            let received = client.recv_response().await.unwrap();
            assert_eq!(received.request_id, MessageId::new(42));
        });

        let _ = tokio::join!(server_handle, client_handle);
    }

    #[tokio::test]
    async fn test_tcp_bidirectional() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

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

        let request_clone = request.clone();
        let response_clone = response.clone();

        // Client: send request, receive response
        let client_handle = tokio::spawn(async move {
            let mut client = TcpTransport::connect(addr.to_string()).await.unwrap();
            client.send_request(&request_clone).await.unwrap();
            let recv_response = client.recv_response().await.unwrap();
            assert_eq!(recv_response.request_id, MessageId::new(1));
            assert_eq!(recv_response.session_id, SessionId::new("session-1"));
        });

        // Server: receive request, send response
        let server_handle = tokio::spawn(async move {
            let mut server = listener.accept().await.unwrap();
            let recv_request = server.recv_request().await.unwrap();
            assert_eq!(recv_request.id, MessageId::new(1));
            server.send_response(&response_clone).await.unwrap();
        });

        let _ = tokio::join!(client_handle, server_handle);
    }

    #[tokio::test]
    async fn test_tcp_split_transport() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let request = Request {
            id: MessageId::new(99),
            session_id: SessionId::new("split-test"),
            operation: Operation::LsSessions,
        };

        let request_clone = request.clone();

        // Client splits and sends
        let client_handle = tokio::spawn(async move {
            let client = TcpTransport::connect(addr.to_string()).await.unwrap();
            let (_reader, mut writer) = client.split();
            writer.send_request(&request_clone).await.unwrap();
            writer.flush().await.unwrap();
        });

        // Server splits and receives
        let server_handle = tokio::spawn(async move {
            let server = listener.accept().await.unwrap();
            let (mut reader, _writer) = server.split();
            let received = reader.recv_request().await.unwrap();
            assert_eq!(received.id, MessageId::new(99));
            assert_eq!(received.session_id, SessionId::new("split-test"));
        });

        let _ = tokio::join!(client_handle, server_handle);
    }

    #[tokio::test]
    async fn test_tcp_close() {
        let listener = TcpTransportListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client_handle = tokio::spawn(async move {
            let mut client = TcpTransport::connect(addr.to_string()).await.unwrap();
            client.close().await.unwrap();
        });

        let server_handle = tokio::spawn(async move {
            let mut server = listener.accept().await.unwrap();
            // Try to receive after client closes
            let result = server.recv_request().await;
            assert!(matches!(result, Err(TransportError::ConnectionClosed)));
        });

        let _ = tokio::join!(client_handle, server_handle);
    }

    #[tokio::test]
    async fn test_tcp_connection_failed() {
        // Try to connect to a port that's not listening
        let result = TcpTransport::connect("127.0.0.1:1").await;
        assert!(matches!(result, Err(TransportError::ConnectionFailed(_))));
    }
}
