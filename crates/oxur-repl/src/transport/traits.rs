// Transport abstraction for REPL communication
//
// Provides trait-based abstraction over different connection types:
// - TCP sockets (network communication)
// - Unix domain sockets (local IPC)
// - Named pipes (Windows IPC)
// - In-process channels (same-process communication)

use crate::protocol::{PostcardCodec, Request, Response};
use std::io;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};

/// Transport-level errors
#[derive(Debug, Error)]
pub enum TransportError {
    /// I/O error during transport operation
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Codec error during message encoding/decoding
    #[error("Codec error: {0}")]
    Codec(#[from] crate::protocol::CodecError),

    /// Connection closed by peer
    #[error("Connection closed")]
    ConnectionClosed,

    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Timeout waiting for operation
    #[error("Operation timed out")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, TransportError>;

/// Transport abstraction for bidirectional communication
///
/// This trait abstracts over different connection types (TCP, Unix sockets,
/// named pipes, in-process channels) to provide a uniform API for sending
/// and receiving REPL messages.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Send a Request to the remote peer
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails or connection is broken.
    async fn send_request(&mut self, request: &Request) -> Result<()>;

    /// Send a Response to the remote peer
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails or connection is broken.
    async fn send_response(&mut self, response: &Response) -> Result<()>;

    /// Receive a Request from the remote peer
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails or connection is broken.
    async fn recv_request(&mut self) -> Result<Request>;

    /// Receive a Response from the remote peer
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails or connection is broken.
    async fn recv_response(&mut self) -> Result<Response>;

    /// Close the transport connection gracefully
    async fn close(&mut self) -> Result<()>;
}

/// Split transport into separate reader and writer halves
///
/// Enables full-duplex communication where reading and writing can happen
/// concurrently in separate tasks.
pub trait SplitTransport: Transport {
    /// Reader half of the transport
    type Reader: TransportReader;

    /// Writer half of the transport
    type Writer: TransportWriter;

    /// Split transport into independent reader and writer
    ///
    /// After splitting, the original transport cannot be used.
    fn split(self) -> (Self::Reader, Self::Writer);
}

/// Reader half of a split transport
#[async_trait::async_trait]
pub trait TransportReader: Send {
    /// Receive a Request from the remote peer
    async fn recv_request(&mut self) -> Result<Request>;

    /// Receive a Response from the remote peer
    async fn recv_response(&mut self) -> Result<Response>;
}

/// Writer half of a split transport
#[async_trait::async_trait]
pub trait TransportWriter: Send {
    /// Send a Request to the remote peer
    async fn send_request(&mut self, request: &Request) -> Result<()>;

    /// Send a Response to the remote peer
    async fn send_response(&mut self, response: &Response) -> Result<()>;

    /// Flush any buffered data
    async fn flush(&mut self) -> Result<()>;
}

/// Helper functions for working with AsyncRead/AsyncWrite streams
pub(crate) mod helpers {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Read a framed message from an async reader
    ///
    /// Reads length prefix (4 bytes) then payload.
    pub async fn read_framed<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
        // Read 4-byte length prefix
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes).await.map_err(|e| match e.kind() {
            io::ErrorKind::UnexpectedEof => TransportError::ConnectionClosed,
            _ => TransportError::Io(e),
        })?;

        let len = u32::from_le_bytes(len_bytes);

        // Validate length
        const MAX_MESSAGE_SIZE: u32 = 10 * 1024 * 1024; // 10 MB
        if len > MAX_MESSAGE_SIZE {
            return Err(TransportError::Codec(crate::protocol::CodecError::MessageTooLarge(len)));
        }

        // Read payload
        let mut payload = vec![0u8; len as usize];
        reader.read_exact(&mut payload).await.map_err(|e| match e.kind() {
            io::ErrorKind::UnexpectedEof => TransportError::ConnectionClosed,
            _ => TransportError::Io(e),
        })?;

        Ok(payload)
    }

    /// Write a framed message to an async writer
    ///
    /// Writes length prefix (4 bytes) then payload, then flushes.
    pub async fn write_framed<W: AsyncWrite + Unpin>(writer: &mut W, payload: &[u8]) -> Result<()> {
        let len = payload.len() as u32;

        // Write length prefix
        writer.write_all(&len.to_le_bytes()).await?;

        // Write payload
        writer.write_all(payload).await?;

        // Flush to ensure data is sent
        writer.flush().await?;

        Ok(())
    }

    /// Send a Request over an async writer
    pub async fn send_request<W: AsyncWrite + Unpin>(
        writer: &mut W,
        request: &Request,
    ) -> Result<()> {
        let payload = PostcardCodec::encode_request(request)?;
        write_framed(writer, &payload).await
    }

    /// Send a Response over an async writer
    pub async fn send_response<W: AsyncWrite + Unpin>(
        writer: &mut W,
        response: &Response,
    ) -> Result<()> {
        let payload = PostcardCodec::encode_response(response)?;
        write_framed(writer, &payload).await
    }

    /// Receive a Request from an async reader
    pub async fn recv_request<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Request> {
        let payload = read_framed(reader).await?;
        Ok(PostcardCodec::decode_request(&payload)?)
    }

    /// Receive a Response from an async reader
    pub async fn recv_response<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Response> {
        let payload = read_framed(reader).await?;
        Ok(PostcardCodec::decode_response(&payload)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Operation, ReplMode};

    #[tokio::test]
    async fn test_send_recv_request() {
        let (mut client, mut server) = tokio::io::duplex(1024);

        let request = Request {
            id: 1,
            session_id: "test-session".to_string(),
            operation: Operation::Eval { code: "(+ 1 2)".to_string(), mode: ReplMode::Lisp },
        };

        // Send in one task, receive in another
        let send_handle = tokio::spawn(async move {
            helpers::send_request(&mut client, &request).await.unwrap();
        });

        let recv_handle = tokio::spawn(async move {
            let received = helpers::recv_request(&mut server).await.unwrap();
            assert_eq!(received.id, 1);
            assert_eq!(received.session_id, "test-session");
        });

        send_handle.await.unwrap();
        recv_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_send_recv_response() {
        let (mut client, mut server) = tokio::io::duplex(1024);

        let response = Response {
            request_id: 42,
            session_id: "test-session".to_string(),
            result: crate::protocol::OperationResult::Success {
                status: crate::protocol::Status { tier: 1, cached: false, duration_ms: 5 },
                value: Some("3".to_string()),
                stdout: None,
                stderr: None,
            },
        };

        let send_handle = tokio::spawn(async move {
            helpers::send_response(&mut server, &response).await.unwrap();
        });

        let recv_handle = tokio::spawn(async move {
            let received = helpers::recv_response(&mut client).await.unwrap();
            assert_eq!(received.request_id, 42);
        });

        send_handle.await.unwrap();
        recv_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_connection_closed() {
        let (mut client, server) = tokio::io::duplex(64);

        // Close server side
        drop(server);

        // Try to read from client should fail with ConnectionClosed
        let result = helpers::recv_request(&mut client).await;
        assert!(matches!(result, Err(TransportError::ConnectionClosed)));
    }

    #[tokio::test]
    async fn test_message_too_large() {
        // Create a request with huge payload
        let huge_code = "x".repeat(11 * 1024 * 1024); // 11 MB
        let request = Request {
            id: 1,
            session_id: "test".to_string(),
            operation: Operation::Eval { code: huge_code, mode: ReplMode::Lisp },
        };

        // Encoding should fail
        let result = PostcardCodec::encode_request(&request);
        assert!(result.is_err());
    }
}
