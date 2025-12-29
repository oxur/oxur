---
number: 18
title: "Oxur Remote REPL Protocol Design"
author: "allowing clients"
created: 2025-12-28
updated: 2025-12-28
state: Under Review
supersedes: null
superseded-by: null
---

# Oxur Remote REPL Protocol Design

**Document Version:** 1.0
**Last Updated:** 2025-12-28
**Status:** Design Specification

## Overview

This document specifies the design and implementation of a remote REPL protocol for Oxur. The protocol enables interactive development by allowing clients to connect to an Oxur REPL server over multiple transport mechanisms (TCP, Unix domain sockets, named pipes, and in-process channels).

The design is inspired by Clojure's nREPL protocol, with adaptations for Rust idioms and Oxur's unique dual-mode REPL architecture (Lisp syntax evaluation + s-expression AST evaluation).

## Repository Context

**Repository:** `oxur` (multi-crate monorepo)
**Crate:** `oxur-repl`

**Proposed Structure:**

```
oxur-repl/
├── src/
│   ├── lib.rs              # Public API and re-exports
│   ├── protocol/           # Message types, no I/O dependencies
│   │   ├── mod.rs
│   │   ├── messages.rs     # Request, Response, Operation enums
│   │   ├── session.rs      # Session ID, state types
│   │   └── codec.rs        # Serialization traits and implementations
│   ├── transport/          # Transport trait + implementations
│   │   ├── mod.rs
│   │   ├── traits.rs       # Transport, TransportListener traits
│   │   ├── tcp.rs          # TcpTransport
│   │   ├── unix.rs         # UnixTransport (cfg(unix))
│   │   ├── pipe.rs         # NamedPipeTransport (cfg(windows))
│   │   └── inprocess.rs    # InProcessTransport for testing
│   ├── eval/               # REPL evaluation engine
│   │   ├── mod.rs
│   │   ├── context.rs      # EvalContext with bindings
│   │   ├── lisp_mode.rs    # Oxur syntax parser + tiered eval
│   │   ├── sexpr_mode.rs   # S-expression AST parser + tiered eval
│   │   └── executor.rs     # Tiered execution strategy
│   ├── server/             # Server assembly + connection handling
│   │   ├── mod.rs
│   │   ├── server.rs       # ReplServer with multi-transport support
│   │   ├── handler.rs      # Message dispatch and operation routing
│   │   ├── session.rs      # SessionManager for state isolation
│   │   └── middleware.rs   # Logging, metrics, timeout (future)
│   └── client/             # Reference client library
│       ├── mod.rs
│       ├── client.rs       # ReplClient implementation
│       └── builder.rs      # Fluent builder API
├── examples/
│   ├── simple_server.rs    # Basic TCP server example
│   ├── simple_client.rs    # Basic client usage
│   └── dual_mode.rs        # Switching between Lisp/s-expr modes
├── tests/
│   ├── integration/        # End-to-end protocol tests
│   └── protocol/           # Message encoding/decoding tests
├── benches/                # Performance benchmarks
└── Cargo.toml
```

**Implementation Strategy:** Build incrementally, starting with core protocol and one transport (TCP), then expand to other transports and advanced features.

## Core Design Principles

1. **Zero-Cost Abstractions:** Trait-based design with monomorphization, no runtime overhead
2. **Transport Agnostic:** Unified API across TCP, Unix sockets, named pipes, in-process
3. **Type Safety:** Rust's type system prevents protocol errors at compile time
4. **Async-First:** Built on Tokio for scalable concurrent connections
5. **Dual-Mode REPL:** First-class support for both Oxur syntax and s-expression AST evaluation
6. **Session Isolation:** Each session maintains independent evaluation state
7. **Streaming Output:** Separate stdout/stderr from evaluation results
8. **Future-Proof:** Architecture supports multiple serialization formats (postcard v0.1, MessagePack future)

## Architecture

### Layer 1: Protocol Message Format

**Location:** `src/protocol/messages.rs`

The protocol uses strongly-typed messages that serialize to postcard's compact binary format.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for requests/responses (correlation)
pub type MessageId = String;

/// Session identifier for state isolation
pub type SessionId = String;

/// Request message from client to server
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Request {
    /// Unique message identifier for correlation
    pub id: MessageId,

    /// Session identifier (empty for session-less operations)
    #[serde(default)]
    pub session: SessionId,

    /// Operation to perform
    pub op: Operation,

    /// REPL evaluation mode
    #[serde(default)]
    pub mode: ReplMode,

    /// Operation-specific parameters
    #[serde(flatten)]
    pub params: HashMap<String, Value>,
}

/// Response message from server to client
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Response {
    /// Echoed message ID from request
    pub id: MessageId,

    /// Session identifier
    #[serde(default)]
    pub session: SessionId,

    /// Evaluation result value (if complete)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,

    /// Captured stdout during evaluation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,

    /// Captured stderr during evaluation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub err: Option<String>,

    /// Status indicators
    pub status: Vec<Status>,

    /// Error information (if status contains Error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,

    /// Additional response metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub data: HashMap<String, Value>,
}

/// Operations supported by the REPL protocol
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    /// Create a new session (returns session ID)
    Clone,

    /// Evaluate code in a session
    Eval,

    /// Load and evaluate a file
    LoadFile,

    /// Interrupt running evaluation
    Interrupt,

    /// Close a session
    Close,

    /// List active sessions
    LsSessions,

    /// Describe the REPL server (capabilities, version, etc.)
    Describe,

    /// Retrieve evaluation history
    History,

    /// Clear output buffer
    ClearOutput,
}

/// REPL evaluation modes (Oxur's dual-mode architecture)
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReplMode {
    /// Oxur syntax: (defn add [x y] (+ x y))
    #[default]
    Lisp,

    /// S-expression AST: (define-func add ...)
    Sexpr,
}

/// Response status indicators
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// Operation completed successfully
    Done,

    /// Protocol-level error occurred
    Error,

    /// Evaluation was interrupted
    Interrupted,

    /// More output/responses coming (streaming)
    Partial,

    /// Session created
    SessionCreated,

    /// Session closed
    SessionClosed,
}

/// Error information for failed operations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ErrorInfo {
    /// Error kind/category
    pub kind: ErrorKind,

    /// Human-readable error message
    pub message: String,

    /// Source location (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,

    /// Stack trace (if available)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub stack_trace: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    /// Protocol violation (malformed message, unknown op, etc.)
    Protocol,

    /// Session not found or invalid
    Session,

    /// Parse error (invalid syntax)
    Parse,

    /// Macro expansion error
    Expand,

    /// Type error during lowering
    Lower,

    /// Runtime evaluation error
    Eval,

    /// File I/O error
    Io,

    /// Operation timeout
    Timeout,

    /// Internal server error
    Internal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Generic value type for protocol data
/// (Uses serde_json::Value for compatibility with postcard)
pub type Value = serde_json::Value;
```

**Key Design Decisions:**

- **Strong typing:** `Operation` and `ReplMode` are enums, not strings (compile-time safety)
- **Correlation IDs:** Every request has unique `id` that response echoes back
- **Session model:** Explicit session IDs for state isolation (unlike Zylisp's implicit model)
- **Dual-mode support:** `ReplMode` enum enables switching between Lisp syntax and s-expr AST
- **Streaming:** `Status::Partial` enables multi-response evaluation (for long-running code)
- **Error taxonomy:** `ErrorKind` distinguishes protocol vs evaluation vs system errors
- **Flat params:** `#[serde(flatten)]` allows operation-specific fields without nested objects

### Layer 2: Serialization Codec

**Location:** `src/protocol/codec.rs`

The codec layer abstracts serialization format from the protocol, enabling future support for multiple formats (postcard for Rust clients, MessagePack for cross-language).

```rust
use async_trait::async_trait;
use bytes::Bytes;
use std::io;
use crate::protocol::{Request, Response};

/// Trait for encoding/decoding protocol messages
#[async_trait]
pub trait Codec: Send + Sync + 'static {
    /// Serialize a request to bytes
    async fn encode_request(&self, req: &Request) -> io::Result<Bytes>;

    /// Deserialize bytes to a request
    async fn decode_request(&self, bytes: Bytes) -> io::Result<Request>;

    /// Serialize a response to bytes
    async fn encode_response(&self, resp: &Response) -> io::Result<Bytes>;

    /// Deserialize bytes to a response
    async fn decode_response(&self, bytes: Bytes) -> io::Result<Response>;

    /// Codec identifier for logging/debugging
    fn name(&self) -> &'static str;
}

/// Postcard codec (v0.1 primary implementation)
pub struct PostcardCodec;

impl PostcardCodec {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Codec for PostcardCodec {
    async fn encode_request(&self, req: &Request) -> io::Result<Bytes> {
        postcard::to_allocvec(req)
            .map(Bytes::from)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn decode_request(&self, bytes: Bytes) -> io::Result<Request> {
        postcard::from_bytes(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn encode_response(&self, resp: &Response) -> io::Result<Bytes> {
        postcard::to_allocvec(resp)
            .map(Bytes::from)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn decode_response(&self, bytes: Bytes) -> io::Result<Response> {
        postcard::from_bytes(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn name(&self) -> &'static str {
        "postcard"
    }
}

/// MessagePack codec (future implementation for cross-language support)
pub struct MessagePackCodec;

impl MessagePackCodec {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Codec for MessagePackCodec {
    async fn encode_request(&self, req: &Request) -> io::Result<Bytes> {
        rmp_serde::to_vec(req)
            .map(Bytes::from)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn decode_request(&self, bytes: Bytes) -> io::Result<Request> {
        rmp_serde::from_slice(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn encode_response(&self, resp: &Response) -> io::Result<Bytes> {
        rmp_serde::to_vec(resp)
            .map(Bytes::from)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn decode_response(&self, bytes: Bytes) -> io::Result<Response> {
        rmp_serde::from_slice(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn name(&self) -> &'static str {
        "messagepack"
    }
}
```

**Framing:** Messages are framed using `tokio_util::codec::LengthDelimitedCodec` with 4-byte big-endian length prefix. The codec layer sits on top of this framing.

### Layer 3: Transport Abstraction

**Location:** `src/transport/traits.rs`

Transport layer provides a unified interface for all connection types, leveraging Tokio's `AsyncRead + AsyncWrite` traits.

```rust
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};
use std::io;
use std::net::SocketAddr;

/// Generic bidirectional stream (TCP, Unix socket, named pipe, etc.)
pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static {}

// Blanket implementation for any type satisfying constraints
impl<T> Stream for T where T: AsyncRead + AsyncWrite + Unpin + Send + 'static {}

/// Transport listener that accepts incoming connections
#[async_trait]
pub trait TransportListener: Send + Sync + 'static {
    /// Stream type this transport produces
    type Stream: Stream;

    /// Accept a new connection
    async fn accept(&self) -> io::Result<Self::Stream>;

    /// Local address the listener is bound to
    fn local_addr(&self) -> io::Result<String>;

    /// Transport type name for logging
    fn transport_type(&self) -> &'static str;
}

/// Connection string format for transport auto-detection
pub enum ConnectionString {
    /// TCP: "tcp://host:port" or "host:port"
    Tcp(SocketAddr),

    /// Unix socket: "unix:///path/to/socket" or "/path/to/socket"
    #[cfg(unix)]
    Unix(std::path::PathBuf),

    /// Named pipe: "pipe://pipename" (Windows)
    #[cfg(windows)]
    NamedPipe(String),

    /// In-process: "in-process" or "memory"
    InProcess,
}

impl ConnectionString {
    /// Parse connection string with auto-detection
    pub fn parse(s: &str) -> io::Result<Self> {
        // Implementation handles various formats:
        // - "tcp://127.0.0.1:7888"
        // - "localhost:7888"
        // - "unix:///tmp/oxur.sock"
        // - "/tmp/oxur.sock"
        // - "pipe://oxur-repl"
        // - "in-process"
        todo!()
    }
}
```

### Layer 4: Transport Implementations

#### 4.1 TCP Transport

**Location:** `src/transport/tcp.rs`

```rust
use crate::transport::{TransportListener, Stream};
use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use std::io;
use std::net::SocketAddr;

pub struct TcpTransport {
    listener: TcpListener,
}

impl TcpTransport {
    pub async fn bind(addr: SocketAddr) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { listener })
    }
}

#[async_trait]
impl TransportListener for TcpTransport {
    type Stream = TcpStream;

    async fn accept(&self) -> io::Result<Self::Stream> {
        let (stream, _addr) = self.listener.accept().await?;
        Ok(stream)
    }

    fn local_addr(&self) -> io::Result<String> {
        self.listener.local_addr()
            .map(|addr| format!("tcp://{}", addr))
    }

    fn transport_type(&self) -> &'static str {
        "tcp"
    }
}
```

#### 4.2 Unix Domain Socket Transport

**Location:** `src/transport/unix.rs`

```rust
#[cfg(unix)]
use crate::transport::{TransportListener, Stream};
use async_trait::async_trait;
use tokio::net::{UnixListener, UnixStream};
use std::io;
use std::path::PathBuf;

pub struct UnixTransport {
    listener: UnixListener,
    path: PathBuf,
}

impl UnixTransport {
    pub async fn bind(path: PathBuf) -> io::Result<Self> {
        // Remove existing socket file if present
        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let listener = UnixListener::bind(&path)?;
        Ok(Self { listener, path })
    }
}

#[async_trait]
impl TransportListener for UnixTransport {
    type Stream = UnixStream;

    async fn accept(&self) -> io::Result<Self::Stream> {
        let (stream, _addr) = self.listener.accept().await?;
        Ok(stream)
    }

    fn local_addr(&self) -> io::Result<String> {
        Ok(format!("unix://{}", self.path.display()))
    }

    fn transport_type(&self) -> &'static str {
        "unix"
    }
}

impl Drop for UnixTransport {
    fn drop(&mut self) {
        // Clean up socket file
        let _ = std::fs::remove_file(&self.path);
    }
}
```

#### 4.3 Named Pipe Transport (Windows)

**Location:** `src/transport/pipe.rs`

```rust
#[cfg(windows)]
use crate::transport::{TransportListener, Stream};
use async_trait::async_trait;
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};
use std::io;

pub struct NamedPipeTransport {
    // Windows named pipes require at least one instance to always exist
    // We maintain a pool and spawn new instances as needed
    pipe_name: String,
    current_server: Option<NamedPipeServer>,
}

impl NamedPipeTransport {
    pub async fn bind(name: String) -> io::Result<Self> {
        let pipe_name = format!(r"\\.\pipe\{}", name);
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_name)?;

        Ok(Self {
            pipe_name,
            current_server: Some(server),
        })
    }
}

#[async_trait]
impl TransportListener for NamedPipeTransport {
    type Stream = NamedPipeServer;

    async fn accept(&self) -> io::Result<Self::Stream> {
        let server = self.current_server.take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No server instance"))?;

        // Wait for client connection
        server.connect().await?;

        // Create next instance for future connections
        let next_server = ServerOptions::new()
            .create(&self.pipe_name)?;
        self.current_server = Some(next_server);

        Ok(server)
    }

    fn local_addr(&self) -> io::Result<String> {
        Ok(format!("pipe://{}", self.pipe_name.trim_start_matches(r"\\.\pipe\")))
    }

    fn transport_type(&self) -> &'static str {
        "named-pipe"
    }
}
```

#### 4.4 In-Process Transport

**Location:** `src/transport/inprocess.rs`

Zero-overhead transport for testing and embedded use cases. Uses Tokio channels instead of sockets.

```rust
use crate::transport::{TransportListener, Stream};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::mpsc;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Bidirectional channel that implements AsyncRead + AsyncWrite
pub struct ChannelStream {
    rx: mpsc::UnboundedReceiver<bytes::Bytes>,
    tx: mpsc::UnboundedSender<bytes::Bytes>,
    read_buf: Option<bytes::Bytes>,
}

impl AsyncRead for ChannelStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // Read from channel into buffer
        // Implementation handles partial reads and buffering
        todo!()
    }
}

impl AsyncWrite for ChannelStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // Write to channel
        // Implementation converts to Bytes and sends
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub struct InProcessTransport {
    accept_rx: mpsc::UnboundedReceiver<ChannelStream>,
    connect_tx: mpsc::UnboundedSender<ChannelStream>,
}

impl InProcessTransport {
    pub fn new() -> (Self, InProcessConnector) {
        let (accept_tx, accept_rx) = mpsc::unbounded_channel();
        let (connect_tx, connect_rx) = mpsc::unbounded_channel();

        let listener = Self { accept_rx, connect_tx };
        let connector = InProcessConnector { accept_tx, connect_rx };

        (listener, connector)
    }
}

#[async_trait]
impl TransportListener for InProcessTransport {
    type Stream = ChannelStream;

    async fn accept(&self) -> io::Result<Self::Stream> {
        self.accept_rx.recv().await
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    fn local_addr(&self) -> io::Result<String> {
        Ok("in-process://memory".to_string())
    }

    fn transport_type(&self) -> &'static str {
        "in-process"
    }
}

/// Client-side connector for in-process transport
pub struct InProcessConnector {
    accept_tx: mpsc::UnboundedSender<ChannelStream>,
    connect_rx: mpsc::UnboundedReceiver<ChannelStream>,
}

impl InProcessConnector {
    pub async fn connect(&mut self) -> io::Result<ChannelStream> {
        let (client_tx, server_rx) = mpsc::unbounded_channel();
        let (server_tx, client_rx) = mpsc::unbounded_channel();

        let server_stream = ChannelStream {
            rx: server_rx,
            tx: server_tx,
            read_buf: None,
        };

        let client_stream = ChannelStream {
            rx: client_rx,
            tx: client_tx,
            read_buf: None,
        };

        self.accept_tx.send(server_stream)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Server closed"))?;

        Ok(client_stream)
    }
}
```

### Layer 5: Evaluation Engine

**Location:** `src/eval/`

The evaluation layer integrates with Oxur's existing tiered execution architecture (from the compilation chain doc).

#### Evaluation Context

**Location:** `src/eval/context.rs`

```rust
use crate::protocol::ReplMode;
use std::collections::HashMap;
use std::sync::Arc;

/// Evaluation context holding session state
pub struct EvalContext {
    /// Session identifier
    session_id: String,

    /// Current REPL mode
    mode: ReplMode,

    /// Variable bindings (for interpreted tier)
    bindings: HashMap<String, oxur_lang::Value>,

    /// Compiled function cache (for JIT tier)
    compiled_cache: HashMap<String, Arc<dyn Fn(&[oxur_lang::Value]) -> oxur_lang::Value>>,

    /// Evaluation history
    history: Vec<HistoryEntry>,

    /// Stdout/stderr capture
    output_buffer: OutputBuffer,
}

impl EvalContext {
    pub fn new(session_id: String, mode: ReplMode) -> Self {
        Self {
            session_id,
            mode,
            bindings: HashMap::new(),
            compiled_cache: HashMap::new(),
            history: Vec::new(),
            output_buffer: OutputBuffer::new(),
        }
    }

    /// Switch between Lisp syntax and s-expression modes
    pub fn set_mode(&mut self, mode: ReplMode) {
        self.mode = mode;
    }

    /// Get current mode
    pub fn mode(&self) -> ReplMode {
        self.mode
    }

    /// Evaluate code using tiered execution strategy
    pub async fn eval(&mut self, code: &str) -> Result<oxur_lang::Value, EvalError> {
        // Integration point with oxur/lang compilation chain:
        //
        // 1. Parse code based on current mode:
        //    - ReplMode::Lisp  → Oxur syntax parser
        //    - ReplMode::Sexpr → Core Forms parser
        //
        // 2. Apply tiered execution (from compilation chain doc):
        //    Tier 1 (Interpreter): Simple expressions (<10 nodes)
        //    Tier 2 (Cached):      Previously compiled code
        //    Tier 3 (JIT):         Complex expressions (compile & cache)
        //
        // 3. Capture stdout/stderr during evaluation
        //
        // 4. Return result or error

        todo!("Integrate with oxur/lang compilation pipeline")
    }

    /// Load and evaluate a file
    pub async fn load_file(&mut self, path: &str) -> Result<oxur_lang::Value, EvalError> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| EvalError::Io(e))?;
        self.eval(&source).await
    }

    /// Get captured output and clear buffer
    pub fn take_output(&mut self) -> (String, String) {
        self.output_buffer.take()
    }

    /// Add entry to history
    pub fn record_history(&mut self, code: String, result: Result<oxur_lang::Value, EvalError>) {
        self.history.push(HistoryEntry {
            timestamp: std::time::SystemTime::now(),
            code,
            result,
        });
    }
}

struct HistoryEntry {
    timestamp: std::time::SystemTime,
    code: String,
    result: Result<oxur_lang::Value, EvalError>,
}

struct OutputBuffer {
    stdout: String,
    stderr: String,
}

impl OutputBuffer {
    fn new() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    fn take(&mut self) -> (String, String) {
        let stdout = std::mem::take(&mut self.stdout);
        let stderr = std::mem::take(&mut self.stderr);
        (stdout, stderr)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Expand error: {0}")]
    Expand(String),

    #[error("Lower error: {0}")]
    Lower(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### Mode-Specific Evaluators

**Location:** `src/eval/lisp_mode.rs` and `src/eval/sexpr_mode.rs`

These modules implement the dual-mode evaluation:

```rust
// src/eval/lisp_mode.rs
use super::context::EvalContext;
use oxur_lang::Value;

/// Parse and evaluate Oxur syntax (defn, when, ->, etc.)
pub async fn eval_lisp_syntax(
    ctx: &mut EvalContext,
    source: &str,
) -> Result<Value, EvalError> {
    // 1. Parse Oxur syntax to Surface Forms
    // 2. Expand macros (defn → define-func, etc.)
    // 3. Lower to Core Forms
    // 4. Execute via tiered strategy
    todo!()
}

// src/eval/sexpr_mode.rs
use super::context::EvalContext;
use oxur_lang::Value;

/// Parse and evaluate s-expression AST (Core Forms directly)
pub async fn eval_sexpr_ast(
    ctx: &mut EvalContext,
    source: &str,
) -> Result<Value, EvalError> {
    // 1. Parse Core Forms directly (skip Surface Forms stage)
    // 2. Execute via tiered strategy
    // 3. Return result
    todo!()
}
```

**Key Insight:** The dual-mode architecture allows users to:

- Write code in natural Oxur syntax (`ReplMode::Lisp`)
- Inspect/debug the expanded Core Forms (`ReplMode::Sexpr`)
- This mirrors the compilation chain's Surface Forms → Core Forms transformation

### Layer 6: Server Implementation

**Location:** `src/server/`

#### Server Core

**Location:** `src/server/server.rs`

```rust
use crate::transport::{TransportListener, Stream};
use crate::protocol::{Request, Response, Operation};
use crate::protocol::codec::Codec;
use crate::server::{SessionManager, MessageHandler};
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_serde::Framed as SerdeFramed;
use std::sync::Arc;
use std::time::Duration;

/// REPL server configuration
pub struct ServerConfig<T: TransportListener, C: Codec> {
    pub transport: T,
    pub codec: C,
    pub max_sessions: usize,
    pub session_timeout: Duration,
    pub request_timeout: Duration,
}

/// REPL server
pub struct ReplServer<T: TransportListener, C: Codec> {
    config: ServerConfig<T, C>,
    sessions: Arc<SessionManager>,
    handler: Arc<MessageHandler>,
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: mpsc::Receiver<()>,
}

impl<T: TransportListener, C: Codec> ReplServer<T, C> {
    pub fn new(config: ServerConfig<T, C>) -> Self {
        let sessions = Arc::new(SessionManager::new(
            config.max_sessions,
            config.session_timeout,
        ));

        let handler = Arc::new(MessageHandler::new(sessions.clone()));

        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Self {
            config,
            sessions,
            handler,
            shutdown_tx,
            shutdown_rx,
        }
    }

    /// Start the server (blocks until shutdown)
    pub async fn serve(mut self) -> Result<(), ServerError> {
        loop {
            tokio::select! {
                // Accept new connection
                stream = self.config.transport.accept() => {
                    let stream = stream?;
                    let codec = self.config.codec.clone();
                    let handler = self.handler.clone();
                    let timeout = self.config.request_timeout;

                    // Spawn connection handler
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, codec, handler, timeout).await {
                            eprintln!("Connection error: {}", e);
                        }
                    });
                }

                // Shutdown signal
                _ = self.shutdown_rx.recv() => {
                    break;
                }
            }
        }

        // Graceful shutdown: close all sessions
        self.sessions.close_all().await;

        Ok(())
    }

    /// Trigger graceful shutdown
    pub async fn shutdown(&self) -> Result<(), ServerError> {
        self.shutdown_tx.send(()).await
            .map_err(|_| ServerError::ShutdownFailed)?;
        Ok(())
    }

    /// Get server listening address
    pub fn local_addr(&self) -> Result<String, ServerError> {
        self.config.transport.local_addr()
            .map_err(ServerError::Transport)
    }
}

/// Handle a single client connection
async fn handle_connection<S, C>(
    stream: S,
    codec: C,
    handler: Arc<MessageHandler>,
    timeout: Duration,
) -> Result<(), ConnectionError>
where
    S: Stream,
    C: Codec,
{
    // Frame messages with length prefix
    let framed = Framed::new(stream, LengthDelimitedCodec::new());

    // Add codec layer for (de)serialization
    let mut transport = SerdeFramed::new(
        framed,
        CodecAdapter::new(codec),
    );

    // Read requests and send responses
    loop {
        tokio::select! {
            // Read next request
            msg = transport.next() => {
                let msg = match msg {
                    Some(Ok(request)) => request,
                    Some(Err(e)) => {
                        eprintln!("Decode error: {}", e);
                        continue;
                    }
                    None => break, // Client disconnected
                };

                // Handle request with timeout
                let response = tokio::time::timeout(
                    timeout,
                    handler.handle_request(msg)
                ).await;

                let response = match response {
                    Ok(Ok(resp)) => resp,
                    Ok(Err(e)) => error_response(&msg, e),
                    Err(_) => timeout_response(&msg),
                };

                // Send response
                if let Err(e) = transport.send(response).await {
                    eprintln!("Send error: {}", e);
                    break;
                }
            }

            // Connection timeout (if idle)
            _ = tokio::time::sleep(timeout * 10) => {
                break;
            }
        }
    }

    Ok(())
}

fn error_response(req: &Request, error: HandlerError) -> Response {
    Response {
        id: req.id.clone(),
        session: req.session.clone(),
        value: None,
        out: None,
        err: None,
        status: vec![Status::Error],
        error: Some(ErrorInfo {
            kind: error.kind(),
            message: error.to_string(),
            source_location: None,
            stack_trace: vec![],
        }),
        data: HashMap::new(),
    }
}

fn timeout_response(req: &Request) -> Response {
    Response {
        id: req.id.clone(),
        session: req.session.clone(),
        value: None,
        out: None,
        err: None,
        status: vec![Status::Error],
        error: Some(ErrorInfo {
            kind: ErrorKind::Timeout,
            message: "Request timeout".to_string(),
            source_location: None,
            stack_trace: vec![],
        }),
        data: HashMap::new(),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Transport error: {0}")]
    Transport(#[from] std::io::Error),

    #[error("Shutdown failed")]
    ShutdownFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Adapter to bridge Codec trait with tokio-serde expectations
struct CodecAdapter<C: Codec> {
    codec: C,
}

impl<C: Codec> CodecAdapter<C> {
    fn new(codec: C) -> Self {
        Self { codec }
    }
}

// Implementation of tokio_serde traits for CodecAdapter
// (bridges our Codec trait with tokio-serde's expectations)
```

#### Message Handler

**Location:** `src/server/handler.rs`

Dispatches requests to appropriate operation handlers.

```rust
use crate::protocol::{Request, Response, Operation, Status};
use crate::server::SessionManager;
use std::sync::Arc;

pub struct MessageHandler {
    sessions: Arc<SessionManager>,
}

impl MessageHandler {
    pub fn new(sessions: Arc<SessionManager>) -> Self {
        Self { sessions }
    }

    pub async fn handle_request(&self, req: Request) -> Result<Response, HandlerError> {
        match req.op {
            Operation::Clone => self.handle_clone(req).await,
            Operation::Eval => self.handle_eval(req).await,
            Operation::LoadFile => self.handle_load_file(req).await,
            Operation::Interrupt => self.handle_interrupt(req).await,
            Operation::Close => self.handle_close(req).await,
            Operation::LsSessions => self.handle_ls_sessions(req).await,
            Operation::Describe => self.handle_describe(req).await,
            Operation::History => self.handle_history(req).await,
            Operation::ClearOutput => self.handle_clear_output(req).await,
        }
    }

    async fn handle_clone(&self, req: Request) -> Result<Response, HandlerError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.sessions.create_session(session_id.clone(), req.mode).await?;

        Ok(Response {
            id: req.id,
            session: session_id.clone(),
            value: Some(serde_json::json!({ "new-session": session_id })),
            out: None,
            err: None,
            status: vec![Status::Done, Status::SessionCreated],
            error: None,
            data: HashMap::new(),
        })
    }

    async fn handle_eval(&self, req: Request) -> Result<Response, HandlerError> {
        let code = req.params.get("code")
            .and_then(|v| v.as_str())
            .ok_or(HandlerError::MissingParameter("code"))?;

        let mut ctx = self.sessions.get_context(&req.session).await?;

        // Capture output
        let result = ctx.eval(code).await;
        let (stdout, stderr) = ctx.take_output();

        // Record in history
        ctx.record_history(code.to_string(), result.clone());

        match result {
            Ok(value) => Ok(Response {
                id: req.id,
                session: req.session,
                value: Some(value_to_json(value)),
                out: if stdout.is_empty() { None } else { Some(stdout) },
                err: if stderr.is_empty() { None } else { Some(stderr) },
                status: vec![Status::Done],
                error: None,
                data: HashMap::new(),
            }),
            Err(e) => Ok(Response {
                id: req.id,
                session: req.session,
                value: None,
                out: if stdout.is_empty() { None } else { Some(stdout) },
                err: Some(stderr + &e.to_string()),
                status: vec![Status::Error],
                error: Some(ErrorInfo {
                    kind: error_to_kind(&e),
                    message: e.to_string(),
                    source_location: None,
                    stack_trace: vec![],
                }),
                data: HashMap::new(),
            }),
        }
    }

    async fn handle_load_file(&self, req: Request) -> Result<Response, HandlerError> {
        let file = req.params.get("file")
            .and_then(|v| v.as_str())
            .ok_or(HandlerError::MissingParameter("file"))?;

        let mut ctx = self.sessions.get_context(&req.session).await?;
        let result = ctx.load_file(file).await;

        // Similar response construction as handle_eval
        todo!()
    }

    async fn handle_interrupt(&self, req: Request) -> Result<Response, HandlerError> {
        // Send interrupt signal to session's evaluation
        // (Implementation uses tokio::sync::watch or CancellationToken)
        todo!()
    }

    async fn handle_close(&self, req: Request) -> Result<Response, HandlerError> {
        self.sessions.close_session(&req.session).await?;

        Ok(Response {
            id: req.id,
            session: req.session,
            value: None,
            out: None,
            err: None,
            status: vec![Status::Done, Status::SessionClosed],
            error: None,
            data: HashMap::new(),
        })
    }

    async fn handle_ls_sessions(&self, req: Request) -> Result<Response, HandlerError> {
        let sessions = self.sessions.list_sessions().await;

        Ok(Response {
            id: req.id,
            session: req.session,
            value: Some(serde_json::json!({ "sessions": sessions })),
            out: None,
            err: None,
            status: vec![Status::Done],
            error: None,
            data: HashMap::new(),
        })
    }

    async fn handle_describe(&self, req: Request) -> Result<Response, HandlerError> {
        Ok(Response {
            id: req.id,
            session: req.session,
            value: Some(serde_json::json!({
                "versions": {
                    "oxur-repl": env!("CARGO_PKG_VERSION"),
                    "protocol": "1.0",
                },
                "ops": {
                    "clone": {},
                    "eval": { "requires": ["code"] },
                    "load-file": { "requires": ["file"] },
                    "interrupt": {},
                    "close": {},
                    "ls-sessions": {},
                    "describe": {},
                    "history": {},
                    "clear-output": {},
                },
                "modes": ["lisp", "sexpr"],
                "codecs": ["postcard"],
            })),
            out: None,
            err: None,
            status: vec![Status::Done],
            error: None,
            data: HashMap::new(),
        })
    }

    async fn handle_history(&self, req: Request) -> Result<Response, HandlerError> {
        let ctx = self.sessions.get_context(&req.session).await?;
        let history = ctx.get_history();

        Ok(Response {
            id: req.id,
            session: req.session,
            value: Some(serde_json::json!({ "history": history })),
            out: None,
            err: None,
            status: vec![Status::Done],
            error: None,
            data: HashMap::new(),
        })
    }

    async fn handle_clear_output(&self, req: Request) -> Result<Response, HandlerError> {
        let mut ctx = self.sessions.get_context(&req.session).await?;
        ctx.take_output(); // Discard

        Ok(Response {
            id: req.id,
            session: req.session,
            value: None,
            out: None,
            err: None,
            status: vec![Status::Done],
            error: None,
            data: HashMap::new(),
        })
    }
}

fn value_to_json(value: oxur_lang::Value) -> serde_json::Value {
    // Convert Oxur value to JSON representation
    // (For protocol transmission)
    todo!()
}

fn error_to_kind(error: &EvalError) -> ErrorKind {
    match error {
        EvalError::Parse(_) => ErrorKind::Parse,
        EvalError::Expand(_) => ErrorKind::Expand,
        EvalError::Lower(_) => ErrorKind::Lower,
        EvalError::Runtime(_) => ErrorKind::Eval,
        EvalError::Io(_) => ErrorKind::Io,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Missing required parameter: {0}")]
    MissingParameter(&'static str),

    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    #[error("Evaluation error: {0}")]
    Eval(#[from] EvalError),
}

impl HandlerError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::MissingParameter(_) => ErrorKind::Protocol,
            Self::Session(_) => ErrorKind::Session,
            Self::Eval(e) => error_to_kind(e),
        }
    }
}
```

#### Session Manager

**Location:** `src/server/session.rs`

Manages session lifecycle and state isolation.

```rust
use crate::eval::EvalContext;
use crate::protocol::ReplMode;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    max_sessions: usize,
    session_timeout: Duration,
}

struct Session {
    context: Arc<RwLock<EvalContext>>,
    created_at: Instant,
    last_used: Instant,
}

impl SessionManager {
    pub fn new(max_sessions: usize, session_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_sessions,
            session_timeout,
        }
    }

    pub async fn create_session(
        &self,
        session_id: String,
        mode: ReplMode,
    ) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;

        if sessions.len() >= self.max_sessions {
            return Err(SessionError::TooManySessions);
        }

        let context = EvalContext::new(session_id.clone(), mode);
        let session = Session {
            context: Arc::new(RwLock::new(context)),
            created_at: Instant::now(),
            last_used: Instant::now(),
        };

        sessions.insert(session_id, session);
        Ok(())
    }

    pub async fn get_context(&self, session_id: &str) -> Result<Arc<RwLock<EvalContext>>, SessionError> {
        let mut sessions = self.sessions.write().await;

        let session = sessions.get_mut(session_id)
            .ok_or(SessionError::NotFound)?;

        // Update last used timestamp
        session.last_used = Instant::now();

        Ok(session.context.clone())
    }

    pub async fn close_session(&self, session_id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
            .ok_or(SessionError::NotFound)?;
        Ok(())
    }

    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.iter()
            .map(|(id, session)| SessionInfo {
                id: id.clone(),
                created_at: session.created_at,
                last_used: session.last_used,
            })
            .collect()
    }

    pub async fn close_all(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.clear();
    }

    /// Cleanup expired sessions (call periodically)
    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        let now = Instant::now();

        sessions.retain(|_, session| {
            now.duration_since(session.last_used) < self.session_timeout
        });
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: Instant,
    pub last_used: Instant,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Too many sessions")]
    TooManySessions,
}
```

### Layer 7: Client Implementation

**Location:** `src/client/`

Reference client library for connecting to Oxur REPL servers.

```rust
use crate::protocol::{Request, Response, Operation, ReplMode, MessageId};
use crate::protocol::codec::Codec;
use crate::transport::ConnectionString;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_serde::Framed as SerdeFramed;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ReplClient<C: Codec> {
    transport: Arc<Mutex<ClientTransport>>,
    codec: C,
    session_id: Option<String>,
    mode: ReplMode,
}

type ClientTransport = SerdeFramed<
    Framed<TcpStream, LengthDelimitedCodec>,
    Response,
    Request,
    CodecAdapter,
>;

impl<C: Codec> ReplClient<C> {
    pub async fn connect(addr: &str, codec: C) -> Result<Self, ClientError> {
        let conn_str = ConnectionString::parse(addr)?;

        let stream = match conn_str {
            ConnectionString::Tcp(addr) => TcpStream::connect(addr).await?,
            #[cfg(unix)]
            ConnectionString::Unix(path) => todo!("Unix socket client"),
            #[cfg(windows)]
            ConnectionString::NamedPipe(name) => todo!("Named pipe client"),
            ConnectionString::InProcess => todo!("In-process client"),
        };

        let framed = Framed::new(stream, LengthDelimitedCodec::new());
        let transport = SerdeFramed::new(framed, CodecAdapter::new(codec.clone()));

        Ok(Self {
            transport: Arc::new(Mutex::new(transport)),
            codec,
            session_id: None,
            mode: ReplMode::default(),
        })
    }

    /// Create a new session
    pub async fn clone_session(&mut self) -> Result<String, ClientError> {
        let req = Request {
            id: Self::generate_id(),
            session: String::new(),
            op: Operation::Clone,
            mode: self.mode,
            params: HashMap::new(),
        };

        let resp = self.send_request(req).await?;

        if resp.status.contains(&Status::SessionCreated) {
            let session_id = resp.value
                .and_then(|v| v.get("new-session"))
                .and_then(|v| v.as_str())
                .ok_or(ClientError::InvalidResponse)?
                .to_string();

            self.session_id = Some(session_id.clone());
            Ok(session_id)
        } else {
            Err(ClientError::OperationFailed(resp))
        }
    }

    /// Evaluate code in current session
    pub async fn eval(&self, code: &str) -> Result<EvalResult, ClientError> {
        let session = self.session_id.as_ref()
            .ok_or(ClientError::NoSession)?;

        let mut params = HashMap::new();
        params.insert("code".to_string(), serde_json::Value::String(code.to_string()));

        let req = Request {
            id: Self::generate_id(),
            session: session.clone(),
            op: Operation::Eval,
            mode: self.mode,
            params,
        };

        let resp = self.send_request(req).await?;

        Ok(EvalResult {
            value: resp.value,
            stdout: resp.out.unwrap_or_default(),
            stderr: resp.err.unwrap_or_default(),
            status: resp.status,
            error: resp.error,
        })
    }

    /// Load and evaluate a file
    pub async fn load_file(&self, path: &str) -> Result<EvalResult, ClientError> {
        let session = self.session_id.as_ref()
            .ok_or(ClientError::NoSession)?;

        let mut params = HashMap::new();
        params.insert("file".to_string(), serde_json::Value::String(path.to_string()));

        let req = Request {
            id: Self::generate_id(),
            session: session.clone(),
            op: Operation::LoadFile,
            mode: self.mode,
            params,
        };

        let resp = self.send_request(req).await?;

        Ok(EvalResult {
            value: resp.value,
            stdout: resp.out.unwrap_or_default(),
            stderr: resp.err.unwrap_or_default(),
            status: resp.status,
            error: resp.error,
        })
    }

    /// Switch REPL mode (Lisp syntax vs s-expression)
    pub fn set_mode(&mut self, mode: ReplMode) {
        self.mode = mode;

        // If we have an active session, we should notify the server
        // (Future enhancement: send mode-change operation)
    }

    /// Get current REPL mode
    pub fn mode(&self) -> ReplMode {
        self.mode
    }

    /// Close current session
    pub async fn close(&mut self) -> Result<(), ClientError> {
        if let Some(session) = &self.session_id {
            let req = Request {
                id: Self::generate_id(),
                session: session.clone(),
                op: Operation::Close,
                mode: self.mode,
                params: HashMap::new(),
            };

            self.send_request(req).await?;
            self.session_id = None;
        }
        Ok(())
    }

    /// Send request and await response
    async fn send_request(&self, req: Request) -> Result<Response, ClientError> {
        let mut transport = self.transport.lock().await;

        transport.send(req).await
            .map_err(|e| ClientError::Transport(e.to_string()))?;

        let resp = transport.next().await
            .ok_or(ClientError::ConnectionClosed)?
            .map_err(|e| ClientError::Transport(e.to_string()))?;

        if resp.status.contains(&Status::Error) {
            Err(ClientError::ServerError(resp))
        } else {
            Ok(resp)
        }
    }

    fn generate_id() -> MessageId {
        uuid::Uuid::new_v4().to_string()
    }
}

pub struct EvalResult {
    pub value: Option<serde_json::Value>,
    pub stdout: String,
    pub stderr: String,
    pub status: Vec<Status>,
    pub error: Option<ErrorInfo>,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("No active session")]
    NoSession,

    #[error("Invalid response")]
    InvalidResponse,

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Operation failed: {0:?}")]
    OperationFailed(Response),

    #[error("Server error: {0:?}")]
    ServerError(Response),
}
```

## Protocol Specification

### Message Flow Examples

#### 1. Session Creation and Evaluation

```
Client → Server: {
  "id": "msg-1",
  "session": "",
  "op": "Clone",
  "mode": "Lisp",
  "params": {}
}

Server → Client: {
  "id": "msg-1",
  "session": "session-abc123",
  "value": {"new-session": "session-abc123"},
  "status": ["Done", "SessionCreated"]
}

Client → Server: {
  "id": "msg-2",
  "session": "session-abc123",
  "op": "Eval",
  "mode": "Lisp",
  "params": {"code": "(+ 1 2)"}
}

Server → Client: {
  "id": "msg-2",
  "session": "session-abc123",
  "value": 3,
  "status": ["Done"]
}
```

#### 2. Mode Switching (Lisp → S-expression)

```
Client → Server: {
  "id": "msg-3",
  "session": "session-abc123",
  "op": "Eval",
  "mode": "Lisp",
  "params": {"code": "(defn double [x] (* x 2))"}
}

Server → Client: {
  "id": "msg-3",
  "session": "session-abc123",
  "value": "<fn double>",
  "status": ["Done"]
}

Client → Server: {
  "id": "msg-4",
  "session": "session-abc123",
  "op": "Eval",
  "mode": "Sexpr",  // Switched to s-expression mode
  "params": {"code": "(define-func double [x i32] (multiply x 2))"}
}

Server → Client: {
  "id": "msg-4",
  "session": "session-abc123",
  "value": "<fn double>",
  "status": ["Done"]
}
```

#### 3. Error Handling (Parse Error)

```
Client → Server: {
  "id": "msg-5",
  "session": "session-abc123",
  "op": "Eval",
  "mode": "Lisp",
  "params": {"code": "(+ 1"}  // Missing closing paren
}

Server → Client: {
  "id": "msg-5",
  "session": "session-abc123",
  "status": ["Error"],
  "error": {
    "kind": "Parse",
    "message": "Unexpected EOF: expected closing parenthesis",
    "source_location": {
      "file": "<repl>",
      "line": 1,
      "column": 5
    }
  }
}
```

#### 4. Output Capture

```
Client → Server: {
  "id": "msg-6",
  "session": "session-abc123",
  "op": "Eval",
  "mode": "Lisp",
  "params": {"code": "(do (println \"Hello\") (+ 1 2))"}
}

Server → Client: {
  "id": "msg-6",
  "session": "session-abc123",
  "value": 3,
  "out": "Hello\n",
  "status": ["Done"]
}
```

### Connection String Formats

| Format | Transport | Example |
|--------|-----------|---------|
| `tcp://host:port` | TCP | `tcp://127.0.0.1:7888` |
| `host:port` | TCP (implicit) | `localhost:7888` |
| `unix://path` | Unix socket | `unix:///tmp/oxur-repl.sock` |
| `/path/to/socket` | Unix socket (implicit) | `/tmp/oxur-repl.sock` |
| `pipe://name` | Named pipe (Windows) | `pipe://oxur-repl` |
| `in-process` | In-process channels | `in-process` |

## Implementation Roadmap

### Phase 1: Core Protocol (Weeks 1-2)

**Goal:** Working protocol with single transport

- [ ] Define protocol message types (`protocol/messages.rs`)
- [ ] Implement postcard codec (`protocol/codec.rs`)
- [ ] Create transport traits (`transport/traits.rs`)
- [ ] Implement TCP transport (`transport/tcp.rs`)
- [ ] Basic integration tests

**Deliverable:** TCP server that accepts connections and echoes messages

### Phase 2: Evaluation Integration (Weeks 3-4)

**Goal:** Connect to Oxur compilation chain

- [ ] Implement `EvalContext` with session state (`eval/context.rs`)
- [ ] Integrate Lisp syntax parser (`eval/lisp_mode.rs`)
- [ ] Integrate s-expression parser (`eval/sexpr_mode.rs`)
- [ ] Wire up tiered execution strategy
- [ ] Output capture (stdout/stderr)

**Deliverable:** Server that can evaluate Oxur code in both modes

### Phase 3: Server Implementation (Week 5)

**Goal:** Production-ready server

- [ ] Session manager with lifecycle management (`server/session.rs`)
- [ ] Message handler with all operations (`server/handler.rs`)
- [ ] Connection handling with framing/codec
- [ ] Graceful shutdown
- [ ] Error handling and reporting

**Deliverable:** Fully functional REPL server

### Phase 4: Client Library (Week 6)

**Goal:** Ergonomic Rust client

- [ ] Connection management (`client/client.rs`)
- [ ] Builder API for configuration (`client/builder.rs`)
- [ ] Session operations (clone, eval, close)
- [ ] Mode switching
- [ ] Error handling

**Deliverable:** Reference client library with examples

### Phase 5: Additional Transports (Week 7)

**Goal:** Cross-platform support

- [ ] Unix domain socket transport (`transport/unix.rs`)
- [ ] Named pipe transport for Windows (`transport/pipe.rs`)
- [ ] In-process transport for testing (`transport/inprocess.rs`)
- [ ] Transport auto-detection from connection strings

**Deliverable:** Multi-transport support with unified API

### Phase 6: Advanced Features (Week 8)

**Goal:** Production polish

- [ ] History operation implementation
- [ ] Interrupt operation (cancellation)
- [ ] Streaming responses for long evaluations
- [ ] Session timeout and cleanup
- [ ] Metrics and logging middleware

**Deliverable:** Production-ready v0.1 release

## Testing Strategy

### Unit Tests

**Protocol Layer:**

- Message serialization/deserialization round-trips
- Codec encode/decode correctness
- Error type conversions
- Operation enum completeness

**Transport Layer:**

- Connection string parsing
- Transport trait implementations
- Framing correctness
- Graceful connection close

**Evaluation Layer:**

- Mode switching
- Output capture
- History recording
- Error propagation

### Integration Tests

**End-to-End:**

```rust
#[tokio::test]
async fn test_tcp_eval_workflow() {
    // Start server
    let server = ReplServer::new(/* config */).await;
    tokio::spawn(server.serve());

    // Connect client
    let mut client = ReplClient::connect("localhost:7888", PostcardCodec::new()).await.unwrap();

    // Create session
    let session_id = client.clone_session().await.unwrap();
    assert!(!session_id.is_empty());

    // Evaluate code
    let result = client.eval("(+ 1 2)").await.unwrap();
    assert_eq!(result.value, Some(serde_json::json!(3)));

    // Close session
    client.close().await.unwrap();
}
```

**Multi-Transport:**

```rust
#[tokio::test]
async fn test_all_transports_identical_behavior() {
    for transport in vec!["tcp://localhost:7888", "unix:///tmp/test.sock", "in-process"] {
        let client = ReplClient::connect(transport, PostcardCodec::new()).await.unwrap();
        // Same test assertions for all transports
    }
}
```

**Dual-Mode:**

```rust
#[tokio::test]
async fn test_mode_switching() {
    let mut client = setup_client().await;

    // Lisp mode
    client.set_mode(ReplMode::Lisp);
    let r1 = client.eval("(defn add [x y] (+ x y))").await.unwrap();

    // S-expression mode
    client.set_mode(ReplMode::Sexpr);
    let r2 = client.eval("(define-func sub [x i32 y i32] (subtract x y))").await.unwrap();

    // Both should succeed
    assert!(r1.status.contains(&Status::Done));
    assert!(r2.status.contains(&Status::Done));
}
```

### Benchmark Tests

**Latency:**

```rust
#[tokio::test]
async fn bench_simple_eval_latency() {
    let client = setup_client().await;

    let start = Instant::now();
    for _ in 0..1000 {
        client.eval("(+ 1 2)").await.unwrap();
    }
    let elapsed = start.elapsed();

    let avg_latency = elapsed / 1000;
    assert!(avg_latency < Duration::from_millis(10), "Average latency too high");
}
```

**Throughput:**

```rust
#[tokio::test]
async fn bench_concurrent_clients() {
    let mut tasks = vec![];
    for i in 0..100 {
        tasks.push(tokio::spawn(async move {
            let client = ReplClient::connect("localhost:7888", PostcardCodec::new()).await.unwrap();
            client.eval(&format!("(+ {} 1)", i)).await.unwrap();
        }));
    }

    let start = Instant::now();
    for task in tasks {
        task.await.unwrap();
    }
    let elapsed = start.elapsed();

    println!("100 concurrent clients: {:?}", elapsed);
}
```

## Future Enhancements

These features are **not** part of v0.1 but the architecture supports them:

### 1. MessagePack Codec (v0.2)

Enable cross-language clients by adding MessagePack serialization:

```rust
// Already structured in codec trait
let msgpack_codec = MessagePackCodec::new();
let client = ReplClient::connect("localhost:7889", msgpack_codec).await?;
```

See companion document "Recommendations for Future-proofing Multiple REPL Protocols" for migration strategy.

### 2. Streaming Responses (v0.3)

Long-running evaluations send incremental output:

```rust
// Status::Partial indicates more responses coming
Server → Client: {"id": "msg-1", "out": "Processing...\n", "status": ["Partial"]}
Server → Client: {"id": "msg-1", "out": "Almost done...\n", "status": ["Partial"]}
Server → Client: {"id": "msg-1", "value": 42, "status": ["Done"]}
```

### 3. Advanced Operations (v0.4)

- **Code completion:** `complete` operation with cursor position
- **Symbol documentation:** `doc` operation
- **Jump-to-definition:** `definition` operation
- **Namespace inspection:** `ns-list`, `ns-vars` operations

### 4. Security Features (v1.0)

- **TLS support:** Encrypt TCP connections
- **Authentication:** Token-based auth for remote access
- **Sandboxing:** Restrict file system access per session

### 5. Middleware System (v1.1)

Pluggable middleware for cross-cutting concerns:

```rust
server.add_middleware(LoggingMiddleware::new());
server.add_middleware(MetricsMiddleware::new());
server.add_middleware(RateLimitMiddleware::new(100));
```

## Error Handling Philosophy

**Two Error Categories:**

1. **Protocol Errors** (set `error` field, `Status::Error`):
   - Malformed messages
   - Unknown operations
   - Session not found
   - Codec failures
   - Network/transport issues

2. **Evaluation Results** (set `value` or `error` field, `Status::Done`):
   - Parse errors
   - Type errors
   - Runtime errors
   - These are *successful protocol exchanges* where evaluation produced an error

**Example:**

```rust
// Protocol error - connection failed
let result = client.eval("(+ 1 2)").await;
match result {
    Err(ClientError::Transport(e)) => {
        // This is a protocol/transport error
        eprintln!("Connection failed: {}", e);
    }
    Ok(eval_result) => {
        if let Some(error) = eval_result.error {
            // This is an evaluation error (protocol succeeded)
            eprintln!("Evaluation failed: {}", error.message);
        } else {
            println!("Result: {:?}", eval_result.value);
        }
    }
}
```

## Integration with Oxur Compilation Chain

The REPL protocol integrates with Oxur's existing architecture at these points:

### 1. Tiered Execution Strategy

From the compilation chain document, Oxur uses three-tier execution:

- **Tier 1 (Interpreter):** Simple expressions (<10 nodes), ~1ms latency
- **Tier 2 (Cached):** Previously compiled code, ~0ms (function call)
- **Tier 3 (JIT):** Complex expressions, 50-200ms first time, cached after

The `eval` operation delegates to this existing system:

```rust
// In eval/context.rs
pub async fn eval(&mut self, code: &str) -> Result<Value, EvalError> {
    // 1. Parse based on mode
    let ast = match self.mode {
        ReplMode::Lisp => parse_oxur_syntax(code)?,   // Surface Forms
        ReplMode::Sexpr => parse_core_forms(code)?,   // Core Forms directly
    };

    // 2. Tiered execution (from compilation chain)
    let value = oxur_lang::execute_tiered(ast, &mut self.bindings, &mut self.compiled_cache)?;

    Ok(value)
}
```

### 2. Source Maps

The compilation chain tracks provenance through Node IDs. When errors occur, the REPL protocol surfaces this:

```rust
// ErrorInfo includes source location
pub struct ErrorInfo {
    pub source_location: Option<SourceLocation>,  // From source map
    pub stack_trace: Vec<String>,                  // From compilation chain
}
```

### 3. Mode Switching

The dual-mode architecture allows users to:

1. **Write naturally** in `ReplMode::Lisp`:

   ```lisp
   (defn factorial [n]
     (if (<= n 1)
       1
       (* n (factorial (- n 1)))))
   ```

2. **Inspect expansions** in `ReplMode::Sexpr`:

   ```lisp
   (define-func factorial [n i32]
     (if-expr (<= n 1)
       1
       (multiply n (factorial (subtract n 1)))))
   ```

This mirrors the Surface Forms → Core Forms transformation in the compilation pipeline.

## Dependencies

```toml
[dependencies]
# Core async runtime
tokio = { version = "1.48", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
postcard = { version = "1.1", features = ["alloc"] }

# For future MessagePack support
rmp-serde = { version = "1.3", optional = true }

# Protocol framing
tokio-util = { version = "0.7", features = ["codec"] }
tokio-serde = { version = "0.9" }
bytes = "1.5"

# Async traits
async-trait = "0.1"

# Error handling
thiserror = "1.0"

# UUID generation
uuid = { version = "1.0", features = ["v4", "serde"] }

# Windows named pipes (platform-specific)
[target.'cfg(windows)'.dependencies]
tokio = { version = "1.48", features = ["net"] }

# Integration with Oxur compilation chain
oxur-lang = { path = "../oxur-lang" }

[dev-dependencies]
criterion = "0.5"
tempfile = "3.8"

[features]
default = ["postcard"]
messagepack = ["rmp-serde"]
```

## Success Criteria

The v0.1 implementation is complete when:

1. ✅ All three core transports work (TCP, Unix, in-process)
2. ✅ Postcard codec fully implemented and tested
3. ✅ Dual-mode REPL (Lisp syntax + s-expression) functional
4. ✅ Core operations implemented: clone, eval, load-file, close, describe
5. ✅ Session management with isolation
6. ✅ Output capture (stdout/stderr) during evaluation
7. ✅ Integration with Oxur's tiered execution
8. ✅ Comprehensive error handling and reporting
9. ✅ Reference client library with examples
10. ✅ Integration tests cover all major workflows
11. ✅ Documentation complete (README, examples, protocol spec)
12. ✅ Architecture supports future MessagePack codec

## Example Usage

### Server

```rust
use oxur_repl::server::{ReplServer, ServerConfig};
use oxur_repl::transport::TcpTransport;
use oxur_repl::protocol::codec::PostcardCodec;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create TCP transport
    let transport = TcpTransport::bind("127.0.0.1:7888".parse()?).await?;

    // Configure server
    let config = ServerConfig {
        transport,
        codec: PostcardCodec::new(),
        max_sessions: 100,
        session_timeout: Duration::from_secs(3600),
        request_timeout: Duration::from_secs(300),
    };

    // Start server
    let server = ReplServer::new(config);
    println!("Oxur REPL server listening on tcp://127.0.0.1:7888");

    server.serve().await?;
    Ok(())
}
```

### Client

```rust
use oxur_repl::client::ReplClient;
use oxur_repl::protocol::codec::PostcardCodec;
use oxur_repl::protocol::ReplMode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to server
    let mut client = ReplClient::connect(
        "localhost:7888",
        PostcardCodec::new()
    ).await?;

    // Create session
    let session_id = client.clone_session().await?;
    println!("Session created: {}", session_id);

    // Evaluate Oxur code
    let result = client.eval("(+ 1 2)").await?;
    println!("Result: {:?}", result.value);

    // Switch to s-expression mode
    client.set_mode(ReplMode::Sexpr);
    let result = client.eval("(add 10 20)").await?;
    println!("Result: {:?}", result.value);

    // Load a file
    let result = client.load_file("examples/factorial.ox").await?;
    println!("Loaded: {:?}", result.value);

    // Close session
    client.close().await?;

    Ok(())
}
```

## Comparison with Zylisp Protocol

| Aspect | Zylisp (Go + bencode) | Oxur (Rust + postcard) |
|--------|----------------------|------------------------|
| **Serialization** | bencode (text-based integers) | postcard (binary, varint) |
| **Wire size** | Larger | ~50% smaller |
| **Type system** | Go interfaces, runtime checks | Rust enums, compile-time safety |
| **Session model** | Implicit (connection = session) | Explicit (session IDs) |
| **Modes** | Single evaluation mode | Dual (Lisp syntax + s-expr AST) |
| **Transport** | TCP, Unix, in-process | TCP, Unix, named pipes, in-process |
| **Async** | Goroutines | Tokio (async/await) |
| **Error handling** | Go errors, status strings | Typed errors, enum status |
| **Framing** | JSON newline-delimited | Length-prefixed binary |
| **Cross-language** | Limited (bencode clients exist) | Postcard (Rust-only v0.1), MessagePack (future) |

**Key improvements over Zylisp:**

- **Type safety:** Rust's type system prevents protocol errors at compile time
- **Performance:** Postcard is 3x faster serialization, smaller wire format
- **Dual-mode:** Inspect macro expansions by switching to s-expression mode
- **Explicit sessions:** Better suited for server scenarios with session persistence
- **Windows support:** Named pipes for native Windows IPC

## Conclusion

This design provides:

✅ **Clean separation of concerns** - protocol, transport, evaluation layers independent
✅ **Type-safe protocol** - Rust enums prevent runtime protocol errors
✅ **Dual-mode REPL** - Lisp syntax and s-expression AST evaluation
✅ **Multi-transport support** - TCP, Unix sockets, named pipes, in-process
✅ **Integration with Oxur** - Leverages existing tiered execution
✅ **Future-proof architecture** - Supports MessagePack and advanced features
✅ **Zero-cost abstractions** - Trait-based design with monomorphization
✅ **Comprehensive error handling** - Protocol vs evaluation errors clearly distinguished

### Next Steps

1. **Review this document** with stakeholders
2. **Set up oxur-repl crate** in repository
3. **Begin Phase 1** (core protocol with TCP transport)
4. **Integrate with oxur/lang** compilation chain
5. **Iterate based on REPL usage** 🦀

---

**"In Lisp, code is data. In Rust, safety is fearless. In Oxur REPL, we get interactive development without compromise."**

---

## Appendix: Glossary

**Postcard**: Rust-native binary serialization format, optimized for size and speed

**Codec**: Serialization/deserialization layer abstracting wire format

**Transport**: Communication mechanism (TCP, Unix socket, named pipe, etc.)

**Session**: Isolated evaluation context with independent state

**ReplMode**: Evaluation mode switch (Lisp syntax vs s-expression AST)

**Operation**: Typed protocol action (clone, eval, close, etc.)

**Correlation ID**: Unique message identifier for request/response matching

**Status**: Typed response indicator (done, error, partial, etc.)

**Tiered Execution**: Oxur's three-level evaluation strategy (interpreter, cached, JIT)

**Surface Forms**: Parsed Oxur syntax before macro expansion

**Core Forms**: Canonical s-expressions after macro expansion (the IR)

---

*End of Document*
