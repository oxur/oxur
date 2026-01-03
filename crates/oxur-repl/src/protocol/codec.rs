// Postcard codec with length-prefixed framing
//
// Implements binary serialization for Request/Response messages using Postcard.
// Each message is prefixed with a u32 length in little-endian byte order.
//
// Wire format:
//   [4 bytes: message length (u32 LE)][N bytes: postcard payload]

use super::{ReplMode, Request, Response, SessionInfo};
use std::io::{self, Read, Write};
use thiserror::Error;

/// Maximum message size (10 MB)
///
/// Prevents unbounded memory allocation from malicious or corrupted data.
/// Based on design assumption that code snippets are reasonably sized.
const MAX_MESSAGE_SIZE: u32 = 10 * 1024 * 1024;

/// Codec errors
#[derive(Debug, Error)]
pub enum CodecError {
    /// I/O error during read/write
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Postcard serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    /// Message exceeds maximum size
    #[error("Message size {0} exceeds maximum {MAX_MESSAGE_SIZE}")]
    MessageTooLarge(u32),

    /// Connection closed unexpectedly
    #[error("Connection closed unexpectedly")]
    UnexpectedEof,
}

pub type Result<T> = std::result::Result<T, CodecError>;

/// Postcard codec for Request/Response messages
///
/// Handles length-prefixed binary serialization using Postcard format.
pub struct PostcardCodec;

impl PostcardCodec {
    /// Encode a Request to bytes
    ///
    /// Returns serialized message with length prefix.
    pub fn encode_request(request: &Request) -> Result<Vec<u8>> {
        let payload = postcard::to_allocvec(request)?;
        Self::frame_message(&payload)
    }

    /// Encode a Response to bytes
    ///
    /// Returns serialized message with length prefix.
    pub fn encode_response(response: &Response) -> Result<Vec<u8>> {
        let payload = postcard::to_allocvec(response)?;
        Self::frame_message(&payload)
    }

    /// Decode a Request from bytes
    ///
    /// # Errors
    ///
    /// Returns error if message is malformed or exceeds size limit.
    pub fn decode_request(data: &[u8]) -> Result<Request> {
        let payload = Self::unframe_message(data)?;
        Ok(postcard::from_bytes(payload)?)
    }

    /// Decode a Response from bytes
    ///
    /// # Errors
    ///
    /// Returns error if message is malformed or exceeds size limit.
    pub fn decode_response(data: &[u8]) -> Result<Response> {
        let payload = Self::unframe_message(data)?;
        Ok(postcard::from_bytes(payload)?)
    }

    /// Write a framed Request to a writer
    ///
    /// # Errors
    ///
    /// Returns error if serialization or write fails.
    pub fn write_request<W: Write>(writer: &mut W, request: &Request) -> Result<()> {
        let framed = Self::encode_request(request)?;
        writer.write_all(&framed)?;
        writer.flush()?;
        Ok(())
    }

    /// Write a framed Response to a writer
    ///
    /// # Errors
    ///
    /// Returns error if serialization or write fails.
    pub fn write_response<W: Write>(writer: &mut W, response: &Response) -> Result<()> {
        let framed = Self::encode_response(response)?;
        writer.write_all(&framed)?;
        writer.flush()?;
        Ok(())
    }

    /// Read a framed Request from a reader
    ///
    /// # Errors
    ///
    /// Returns error if read fails, message is malformed, or exceeds size limit.
    pub fn read_request<R: Read>(reader: &mut R) -> Result<Request> {
        let payload = Self::read_framed_message(reader)?;
        Ok(postcard::from_bytes(&payload)?)
    }

    /// Read a framed Response from a reader
    ///
    /// # Errors
    ///
    /// Returns error if read fails, message is malformed, or exceeds size limit.
    pub fn read_response<R: Read>(reader: &mut R) -> Result<Response> {
        let payload = Self::read_framed_message(reader)?;
        Ok(postcard::from_bytes(&payload)?)
    }

    // Internal framing helpers

    fn frame_message(payload: &[u8]) -> Result<Vec<u8>> {
        let len = payload.len();
        if len > MAX_MESSAGE_SIZE as usize {
            return Err(CodecError::MessageTooLarge(len as u32));
        }

        let mut framed = Vec::with_capacity(4 + len);
        framed.extend_from_slice(&(len as u32).to_le_bytes());
        framed.extend_from_slice(payload);
        Ok(framed)
    }

    fn unframe_message(data: &[u8]) -> Result<&[u8]> {
        if data.len() < 4 {
            return Err(CodecError::UnexpectedEof);
        }

        let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if len > MAX_MESSAGE_SIZE {
            return Err(CodecError::MessageTooLarge(len));
        }

        let payload_start = 4;
        let payload_end = payload_start + len as usize;

        if data.len() < payload_end {
            return Err(CodecError::UnexpectedEof);
        }

        Ok(&data[payload_start..payload_end])
    }

    fn read_framed_message<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
        // Read 4-byte length prefix
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes);

        if len > MAX_MESSAGE_SIZE {
            return Err(CodecError::MessageTooLarge(len));
        }

        // Read payload
        let mut payload = vec![0u8; len as usize];
        reader.read_exact(&mut payload)?;
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Operation, OperationResult, ReplMode, Status};
    use std::io::Cursor;

    #[test]
    fn test_encode_decode_request() {
        let request = Request {
            id: 1,
            session_id: "test-session".to_string(),
            operation: Operation::Eval {
                code: "(+ 1 2)".to_string(),
                mode: ReplMode::Lisp,
            },
        };

        let encoded = PostcardCodec::encode_request(&request).unwrap();
        let decoded = PostcardCodec::decode_request(&encoded).unwrap();
        assert_eq!(request, decoded);
    }

    #[test]
    fn test_encode_decode_response() {
        let response = Response {
            request_id: 1,
            session_id: "test-session".to_string(),
            result: OperationResult::Success {
                status: Status {
                    tier: 1,
                    cached: false,
                    duration_ms: 5,
                },
                value: Some("3".to_string()),
                stdout: None,
                stderr: None,
            },
        };

        let encoded = PostcardCodec::encode_response(&response).unwrap();
        let decoded = PostcardCodec::decode_response(&encoded).unwrap();
        assert_eq!(response, decoded);
    }

    #[test]
    fn test_write_read_request() {
        let request = Request {
            id: 42,
            session_id: "write-read-test".to_string(),
            operation: Operation::LsSessions,
        };

        let mut buffer = Vec::new();
        PostcardCodec::write_request(&mut buffer, &request).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = PostcardCodec::read_request(&mut cursor).unwrap();
        assert_eq!(request, decoded);
    }

    #[test]
    fn test_write_read_response() {
        let response = Response {
            request_id: 99,
            session_id: "response-test".to_string(),
            result: OperationResult::Sessions {
                sessions: vec![
                    SessionInfo {
                        id: "session1".to_string(),
                        mode: ReplMode::Lisp,
                        eval_count: 0,
                        created_at: 0,
                    },
                    SessionInfo {
                        id: "session2".to_string(),
                        mode: ReplMode::Sexpr,
                        eval_count: 5,
                        created_at: 1000,
                    },
                ],
            },
        };

        let mut buffer = Vec::new();
        PostcardCodec::write_response(&mut buffer, &response).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = PostcardCodec::read_response(&mut cursor).unwrap();
        assert_eq!(response, decoded);
    }

    #[test]
    fn test_message_too_large() {
        // Create payload that exceeds MAX_MESSAGE_SIZE
        let huge_code = "x".repeat((MAX_MESSAGE_SIZE + 1) as usize);
        let request = Request {
            id: 1,
            session_id: "test".to_string(),
            operation: Operation::Eval {
                code: huge_code,
                mode: ReplMode::Lisp,
            },
        };

        let result = PostcardCodec::encode_request(&request);
        assert!(matches!(result, Err(CodecError::MessageTooLarge(_))));
    }

    #[test]
    fn test_unexpected_eof() {
        // Incomplete message (length prefix without payload)
        let incomplete = vec![0x0A, 0x00, 0x00, 0x00]; // Says 10 bytes follow, but nothing there

        let result = PostcardCodec::decode_request(&incomplete);
        assert!(matches!(result, Err(CodecError::UnexpectedEof)));
    }

    #[test]
    fn test_framing_format() {
        let request = Request {
            id: 1,
            session_id: "test".to_string(),
            operation: Operation::Interrupt,
        };

        let encoded = PostcardCodec::encode_request(&request).unwrap();

        // First 4 bytes should be length
        let len = u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
        assert_eq!(len as usize, encoded.len() - 4);

        // Remaining bytes should be postcard payload
        let payload = &encoded[4..];
        let decoded: Request = postcard::from_bytes(payload).unwrap();
        assert_eq!(request, decoded);
    }
}
