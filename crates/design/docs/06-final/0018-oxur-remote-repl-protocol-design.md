---
number: 18
title: "Oxur Remote REPL Protocol Design"
author: "Duncan McGreggor & Claude"
component: REPL
tags: [protocols, networking, sockets, tcp, ipc]
created: 2025-12-28
updated: 2026-01-06
state: Final
supersedes: null
superseded-by: null
version: 1.1
---


# Oxur Remote REPL Protocol Design

## Overview

This document specifies the design and implementation of a remote REPL protocol for Oxur. The protocol enables interactive development by allowing clients to connect to an Oxur REPL server over multiple transport mechanisms (TCP, Unix domain sockets, named pipes, and in-process channels).

The design is inspired by Clojure's nREPL protocol, with adaptations for Rust idioms and Oxur's unique dual-mode REPL architecture (Lisp syntax evaluation + s-expression AST evaluation).

**Architectural Authority:** This document specifies the client-server protocol layer. For server-side implementation architecture (subprocess execution, compilation pipeline, caching), see ODD-0038 (Oxur REPL Architecture v1.2).

**Related Documents:**

- ODD-0038: Oxur REPL Architecture (authoritative architecture specification)
- ODD-0013: Oxur Compilation Chain Architecture (compilation pipeline context)
- ODD-0030: Oxur REPL Implementation Specification (component implementation details)
- ODD-0026: Oxur REPL Evaluation Strategy (three-tier execution strategy)

## Repository Context

**Repository:** `oxur` (multi-crate monorepo)
**Crate:** `oxur-repl`

**Proposed Structure:**

```
oxur-repl/
├── src/
│   ├── lib.rs                  # Public API and re-exports
│   ├── protocol/               # Message types, no I/O dependencies
│   │   ├── mod.rs
│   │   ├── messages.rs         # Request, Response, Operation enums
│   │   ├── session.rs          # Session ID, state types
│   │   └── codec.rs            # Serialization traits and implementations
│   ├── transport/              # Transport trait + implementations
│   │   ├── mod.rs
│   │   ├── traits.rs           # Transport, TransportListener traits
│   │   ├── tcp.rs              # TcpTransport
│   │   ├── unix.rs             # UnixTransport (cfg(unix))
│   │   ├── pipe.rs             # NamedPipeTransport (cfg(windows))
│   │   └── inprocess.rs        # InProcessTransport for testing
│   ├── eval/                   # REPL evaluation coordination
│   │   ├── mod.rs
│   │   └── context.rs          # EvalContext with session state
│   ├── compiler/               # Compilation pipeline
│   │   ├── mod.rs
│   │   └── cached.rs           # CachedCompiler implementation
│   ├── executor/               # Execution strategy
│   │   ├── mod.rs              # Executor trait
│   │   └── subprocess.rs       # SubprocessExecutor (mandatory)
│   ├── cache.rs                # ArtifactCache for compiled artifacts
│   ├── wrapper.rs              # RustAstWrapper for REPL scaffolding
│   ├── type_inference.rs       # TypeInference (rust-analyzer integration)
│   ├── server/                 # Server assembly + connection handling
│   │   ├── mod.rs
│   │   ├── server.rs           # ReplServer with multi-transport support
│   │   ├── handler.rs          # Message dispatch and operation routing
│   │   └── middleware.rs       # Logging, metrics, timeout (future)
│   ├── session/                # Session management
│   │   ├── mod.rs
│   │   ├── manager.rs          # SessionManager for state isolation
│   │   ├── dir.rs              # SessionDir for temp filesystem
│   │   └── state.rs            # SessionState tracking
│   ├── client/                 # Reference client library
│   │   ├── mod.rs
│   │   ├── client.rs           # ReplClient implementation
│   │   └── builder.rs          # Fluent builder API
│   └── bin/
│       └── subprocess.rs       # Subprocess binary (isolated execution)
├── examples/
│   ├── simple_server.rs        # Basic TCP server example
│   ├── simple_client.rs        # Basic client usage
│   └── dual_mode.rs            # Switching between Lisp/s-expr modes
├── tests/
│   ├── integration/            # End-to-end protocol tests
│   └── protocol/               # Message encoding/decoding tests
├── benches/                    # Performance benchmarks
└── Cargo.toml
```

**Cargo.toml Binary Target:**

```toml
[[bin]]
name = "oxur-repl-subprocess"
path = "src/bin/subprocess.rs"
```

## Core Design Principles

1. **Zero-Cost Abstractions:** Trait-based design with monomorphization, no runtime overhead
2. **Transport Agnostic:** Unified API across TCP, Unix sockets, named pipes, in-process
3. **Type Safety:** Rust's type system prevents protocol errors at compile time
4. **Async-First:** Built on Tokio for scalable concurrent connections
5. **Dual-Mode REPL:** First-class support for both Oxur syntax and s-expression AST evaluation
6. **Session Isolation:** Each session maintains independent evaluation state
7. **Subprocess Execution:** Mandatory isolation for Ctrl-C support and crash recovery
8. **Streaming Output:** Separate stdout/stderr from evaluation results
9. **Persistent Caching:** Content-based artifact caching for fast re-evaluation
10. **Future-Proof:** Architecture supports multiple serialization formats

## Architecture

### Two-Protocol Design

The REPL system uses **two separate protocols**:

1. **Client ↔ Server Protocol** (this document's primary focus)
   - Transport: TCP sockets, Unix sockets, named pipes
   - Serialization: Postcard (binary, efficient)
   - Framing: Length-prefixed messages

2. **Server ↔ Subprocess Protocol** (internal, see Section 8)
   - Transport: stdin/stdout
   - Serialization: Text-based
   - Purpose: Isolated code execution with Ctrl-C support

### Layer 1: Protocol Message Format

**Location:** `src/protocol/messages.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type MessageId = String;
pub type SessionId = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Request {
    pub id: MessageId,
    #[serde(default)]
    pub session: SessionId,
    pub op: Operation,
    #[serde(default)]
    pub mode: ReplMode,
    #[serde(flatten)]
    pub params: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Response {
    pub id: MessageId,
    #[serde(default)]
    pub session: SessionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub err: Option<String>,
    pub status: Vec<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub data: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    Clone,
    Eval,
    LoadFile,
    Interrupt,
    Close,
    LsSessions,
    Describe,
    History,
    ClearOutput,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReplMode {
    #[default]
    Lisp,
    Sexpr,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Done,
    Error,
    Interrupted,
    Partial,
    SessionCreated,
    SessionClosed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ErrorInfo {
    pub kind: ErrorKind,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub stack_trace: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    Protocol,
    Session,
    Parse,
    Expand,
    Lower,
    Eval,
    Io,
    Timeout,
    Internal,
    SubprocessCrash,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

pub type Value = serde_json::Value;
```

### Layer 2: Serialization Codec

**Location:** `src/protocol/codec.rs`

```rust
use bytes::Bytes;
use std::io;
use crate::protocol::{Request, Response};

pub trait Codec: Send + Sync + Clone + 'static {
    fn encode_request(&self, req: &Request) -> io::Result<Bytes>;
    fn decode_request(&self, bytes: &[u8]) -> io::Result<Request>;
    fn encode_response(&self, resp: &Response) -> io::Result<Bytes>;
    fn decode_response(&self, bytes: &[u8]) -> io::Result<Response>;
    fn name(&self) -> &'static str;
}

#[derive(Clone)]
pub struct PostcardCodec;

impl PostcardCodec {
    pub fn new() -> Self { Self }
}

impl Codec for PostcardCodec {
    fn encode_request(&self, req: &Request) -> io::Result<Bytes> {
        postcard::to_allocvec(req)
            .map(Bytes::from)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn decode_request(&self, bytes: &[u8]) -> io::Result<Request> {
        postcard::from_bytes(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn encode_response(&self, resp: &Response) -> io::Result<Bytes> {
        postcard::to_allocvec(resp)
            .map(Bytes::from)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn decode_response(&self, bytes: &[u8]) -> io::Result<Response> {
        postcard::from_bytes(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn name(&self) -> &'static str { "postcard" }
}
```

**Framing:** Messages use `LengthDelimitedCodec` with 4-byte big-endian length prefix.
### Layer 3: Transport Abstraction

**Location:** `src/transport/traits.rs`

```rust
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};
use std::io;
use std::net::SocketAddr;

pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static {}
impl<T> Stream for T where T: AsyncRead + AsyncWrite + Unpin + Send + 'static {}

#[async_trait]
pub trait TransportListener: Send + Sync + 'static {
    type Stream: Stream;
    async fn accept(&self) -> io::Result<Self::Stream>;
    fn local_addr(&self) -> io::Result<String>;
    fn transport_type(&self) -> &'static str;
}

pub enum ConnectionString {
    Tcp(SocketAddr),
    #[cfg(unix)]
    Unix(std::path::PathBuf),
    #[cfg(windows)]
    NamedPipe(String),
    InProcess,
}

impl ConnectionString {
    pub fn parse(s: &str) -> io::Result<Self> {
        if s.starts_with("tcp://") {
            let addr = s.strip_prefix("tcp://").unwrap();
            let socket_addr: SocketAddr = addr.parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            Ok(Self::Tcp(socket_addr))
        } else if s.starts_with("unix://") {
            #[cfg(unix)]
            { Ok(Self::Unix(std::path::PathBuf::from(s.strip_prefix("unix://").unwrap()))) }
            #[cfg(not(unix))]
            { Err(io::Error::new(io::ErrorKind::Unsupported, "Unix sockets not supported")) }
        } else if s.starts_with("pipe://") {
            #[cfg(windows)]
            { Ok(Self::NamedPipe(s.strip_prefix("pipe://").unwrap().to_string())) }
            #[cfg(not(windows))]
            { Err(io::Error::new(io::ErrorKind::Unsupported, "Named pipes Windows only")) }
        } else if s == "in-process" || s == "memory" {
            Ok(Self::InProcess)
        } else if s.starts_with('/') {
            #[cfg(unix)]
            { Ok(Self::Unix(std::path::PathBuf::from(s))) }
            #[cfg(not(unix))]
            { Err(io::Error::new(io::ErrorKind::Unsupported, "Unix sockets not supported")) }
        } else {
            let socket_addr: SocketAddr = s.parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            Ok(Self::Tcp(socket_addr))
        }
    }
}
```

### Layer 4: Transport Implementations

#### TCP Transport

```rust
use tokio::net::{TcpListener, TcpStream};

pub struct TcpTransport { listener: TcpListener }

impl TcpTransport {
    pub async fn bind(addr: SocketAddr) -> io::Result<Self> {
        Ok(Self { listener: TcpListener::bind(addr).await? })
    }
}

#[async_trait]
impl TransportListener for TcpTransport {
    type Stream = TcpStream;
    async fn accept(&self) -> io::Result<Self::Stream> {
        Ok(self.listener.accept().await?.0)
    }
    fn local_addr(&self) -> io::Result<String> {
        self.listener.local_addr().map(|a| format!("tcp://{}", a))
    }
    fn transport_type(&self) -> &'static str { "tcp" }
}
```

#### Unix Domain Socket Transport

```rust
#[cfg(unix)]
pub struct UnixTransport {
    listener: tokio::net::UnixListener,
    path: std::path::PathBuf,
}

#[cfg(unix)]
impl UnixTransport {
    pub async fn bind(path: std::path::PathBuf) -> io::Result<Self> {
        if path.exists() { std::fs::remove_file(&path)?; }
        Ok(Self { listener: tokio::net::UnixListener::bind(&path)?, path })
    }
}

#[cfg(unix)]
impl Drop for UnixTransport {
    fn drop(&mut self) { let _ = std::fs::remove_file(&self.path); }
}
```

#### Named Pipe Transport (Windows)

```rust
#[cfg(windows)]
pub struct NamedPipeTransport {
    pipe_name: String,
    current_server: std::sync::Mutex<Option<tokio::net::windows::named_pipe::NamedPipeServer>>,
}
```

#### In-Process Transport

Zero-overhead transport for testing using Tokio channels.

### Layer 5: Server-Side Components

#### 5.1 Evaluation Context

**Location:** `src/eval/context.rs`

```rust
use crate::protocol::ReplMode;
use crate::compiler::CachedCompiler;
use crate::cache::ArtifactCache;
use oxur_smap::SourceMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Evaluation context holding session state
/// 
/// Variables are stored in the subprocess VariableStore,
/// not here. EvalContext coordinates compilation and
/// delegates execution to the subprocess.
pub struct EvalContext {
    session_id: String,
    mode: ReplMode,
    compiler: CachedCompiler,
    cache: Arc<Mutex<ArtifactCache>>,
    history: Vec<HistoryEntry>,
    output_buffer: OutputBuffer,
}

impl EvalContext {
    pub fn new(
        session_id: String,
        mode: ReplMode,
        cache: Arc<Mutex<ArtifactCache>>,
    ) -> Result<Self, EvalError> {
        let compiler = CachedCompiler::new(&session_id)?;
        Ok(Self {
            session_id, mode, compiler, cache,
            history: Vec::new(),
            output_buffer: OutputBuffer::new(),
        })
    }

    pub fn set_mode(&mut self, mode: ReplMode) { self.mode = mode; }
    pub fn mode(&self) -> ReplMode { self.mode }

    /// Evaluate code using three-tier execution strategy
    /// 
    /// Tier 1 (Calculator): Simple arithmetic, <1ms
    /// Tier 2 (Cached): Cache hit from ArtifactCache, 1-5ms
    /// Tier 3 (JIT): Full compilation, 50-300ms
    pub async fn eval(&mut self, code: &str) -> Result<oxur_lang::Value, EvalError> {
        let mut source_map = SourceMap::new();

        // 1. Parse based on mode
        let core_forms = match self.mode {
            ReplMode::Lisp => {
                let surface = oxur_lang::parse_lisp(code, &mut source_map)
                    .map_err(EvalError::Parse)?;
                oxur_lang::expand(surface, &mut source_map)
                    .map_err(EvalError::Expand)?
            }
            ReplMode::Sexpr => {
                oxur_lang::parse_core_forms(code, &mut source_map)
                    .map_err(EvalError::Parse)?
            }
        };

        // 2. Tier 1 check (Calculator)
        if is_simple_arithmetic(&core_forms) {
            return self.eval_calculator(&core_forms);
        }

        // 3. Tier 2 check (Cache)
        let cache_key = self.cache.lock().await.compute_key(code, &source_map);
        if let Some(artifact_path) = self.cache.lock().await.get(&cache_key) {
            return self.compiler.execute_cached(&artifact_path).await
                .map_err(EvalError::Runtime);
        }

        // 4. Tier 3 (JIT Compilation)
        let result = self.compiler.eval(core_forms, source_map).await?;
        if let Some(path) = result.artifact_path.as_ref() {
            self.cache.lock().await.insert(cache_key, path.clone());
        }
        Ok(result.value)
    }

    fn eval_calculator(&self, forms: &oxur_lang::CoreForms) -> Result<oxur_lang::Value, EvalError> {
        oxur_lang::eval_simple(forms).map_err(EvalError::Runtime)
    }

    pub async fn load_file(&mut self, path: &str) -> Result<oxur_lang::Value, EvalError> {
        let source = std::fs::read_to_string(path).map_err(EvalError::Io)?;
        self.eval(&source).await
    }

    pub fn take_output(&mut self) -> (String, String) { self.output_buffer.take() }

    pub fn record_history(&mut self, code: String, result: Result<oxur_lang::Value, EvalError>) {
        self.history.push(HistoryEntry {
            timestamp: std::time::SystemTime::now(),
            code,
            success: result.is_ok(),
        });
    }

    pub fn get_history(&self) -> &[HistoryEntry] { &self.history }

    /// Interrupt by killing and restarting subprocess
    pub fn interrupt(&mut self) -> Result<(), EvalError> {
        self.compiler.kill_subprocess()?;
        self.compiler.restart_subprocess().map_err(EvalError::Internal)
    }
}

fn is_simple_arithmetic(forms: &oxur_lang::CoreForms) -> bool {
    forms.node_count() < 10 && forms.is_pure_arithmetic()
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: std::time::SystemTime,
    pub code: String,
    pub success: bool,
}

struct OutputBuffer { stdout: String, stderr: String }
impl OutputBuffer {
    fn new() -> Self { Self { stdout: String::new(), stderr: String::new() } }
    fn take(&mut self) -> (String, String) {
        (std::mem::take(&mut self.stdout), std::mem::take(&mut self.stderr))
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
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Subprocess crashed: {0}")]
    SubprocessCrash(String),
}
```
#### 5.2 Cached Compiler

**Location:** `src/compiler/cached.rs`

```rust
use crate::executor::SubprocessExecutor;
use crate::wrapper::RustAstWrapper;
use crate::type_inference::TypeInference;
use crate::session::{SessionDir, SessionState};
use oxur_smap::SourceMap;
use std::path::PathBuf;

/// Compilation engine managing the full pipeline
pub struct CachedCompiler {
    session_dir: SessionDir,
    state: SessionState,
    executor: SubprocessExecutor,      // MANDATORY
    rust_ast_wrapper: RustAstWrapper,
    type_inference: TypeInference,
}

impl CachedCompiler {
    pub fn new(session_id: &str) -> Result<Self, CompilerError> {
        let session_dir = SessionDir::new(session_id)?;
        let executor = SubprocessExecutor::new()?;
        Ok(Self {
            session_dir,
            state: SessionState::new(),
            executor,
            rust_ast_wrapper: RustAstWrapper::new(),
            type_inference: TypeInference::new()?,
        })
    }

    pub async fn eval(
        &mut self,
        core_forms: oxur_lang::CoreForms,
        mut source_map: SourceMap,
    ) -> Result<EvalResult, CompilerError> {
        let saved_state = self.state.clone();
        match self.try_eval(core_forms, &mut source_map).await {
            Ok(result) => Ok(result),
            Err(e) => {
                self.state = saved_state;  // Rollback on failure
                Err(e)
            }
        }
    }

    async fn try_eval(
        &mut self,
        core_forms: oxur_lang::CoreForms,
        source_map: &mut SourceMap,
    ) -> Result<EvalResult, CompilerError> {
        // Stage 4: Lower to Rust AST
        let rust_ast = oxur_comp::lower(&core_forms, source_map)
            .map_err(|e| self.translate_error(e, source_map))?;

        // Stage 5: Infer types
        let var_types = self.type_inference.infer_types(&rust_ast)?;

        // Stage 6: Wrap with REPL scaffolding
        let wrapped_ast = self.rust_ast_wrapper.wrap(
            rust_ast, &self.state, &var_types, source_map,
        )?;

        // Stage 7: Generate Rust source
        let rust_source = oxur_ast::print_rust(&wrapped_ast);

        // Stage 8: Write to session directory
        let lib_path = self.session_dir.write_source(&rust_source)?;
        
        // Stage 9: Invoke cargo build
        let artifact_path = self.compile(&lib_path).await?;

        // Stage 10: Execute via subprocess
        let fn_name = self.state.current_fn_name();
        let result = self.executor.execute(&artifact_path, &fn_name).await?;

        self.state.increment_eval_counter();

        Ok(EvalResult { value: result, artifact_path: Some(artifact_path) })
    }

    pub async fn execute_cached(&mut self, artifact_path: &PathBuf) -> Result<oxur_lang::Value, String> {
        let fn_name = format!("run_user_code_{}", self.state.eval_counter());
        self.executor.execute(artifact_path, &fn_name).await
    }

    async fn compile(&self, _source_path: &PathBuf) -> Result<PathBuf, CompilerError> {
        let output = tokio::process::Command::new("cargo")
            .arg("build")
            .arg("--message-format=json")
            .current_dir(self.session_dir.path())
            .output()
            .await?;

        if !output.status.success() {
            return Err(self.parse_cargo_errors(&output.stderr)?);
        }
        self.session_dir.artifact_path()
    }

    fn translate_error(&self, error: oxur_comp::Error, source_map: &SourceMap) -> CompilerError {
        if let Some(rust_pos) = error.position() {
            if let Some(oxur_pos) = source_map.lookup(rust_pos.node_id) {
                return CompilerError::Lower {
                    message: error.message().to_string(),
                    file: oxur_pos.file.clone(),
                    line: oxur_pos.line,
                    column: oxur_pos.column,
                };
            }
        }
        CompilerError::Lower {
            message: error.message().to_string(),
            file: "<unknown>".to_string(), line: 0, column: 0,
        }
    }

    fn parse_cargo_errors(&self, _stderr: &[u8]) -> Result<CompilerError, CompilerError> {
        Err(CompilerError::Cargo("Compilation failed".to_string()))
    }

    pub fn kill_subprocess(&mut self) -> Result<(), CompilerError> {
        self.executor.kill()
    }

    pub fn restart_subprocess(&mut self) -> Result<(), String> {
        self.executor.restart()
    }
}

pub struct EvalResult {
    pub value: oxur_lang::Value,
    pub artifact_path: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lower error at {file}:{line}:{column}: {message}")]
    Lower { message: String, file: String, line: u32, column: u32 },
    #[error("Compilation failed: {0}")]
    Cargo(String),
    #[error("Subprocess error: {0}")]
    Subprocess(String),
}
```

#### 5.3 Subprocess Executor

**Location:** `src/executor/subprocess.rs`

**Why Subprocess is Mandatory:**
1. **Ctrl-C support:** Rust threads cannot be interrupted; subprocess can be killed via SIGKILL
2. **Crash isolation:** User panic doesn't kill the REPL server
3. **Memory isolation:** Separate address space
4. **Clean restart:** Spawn new subprocess on error

```rust
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct SubprocessExecutor {
    subprocess: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl SubprocessExecutor {
    pub fn new() -> Result<Self, std::io::Error> {
        Self::spawn()
    }

    fn spawn() -> Result<Self, std::io::Error> {
        let mut subprocess = Command::new("oxur-repl-subprocess")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = subprocess.stdin.take()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No stdin"))?;
        let stdout = subprocess.stdout.take()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No stdout"))?;

        Ok(Self { subprocess, stdin, stdout: BufReader::new(stdout) })
    }

    pub async fn execute(&mut self, lib_path: &PathBuf, fn_name: &str) -> Result<oxur_lang::Value, String> {
        writeln!(self.stdin, "LOAD_AND_RUN {} {}", lib_path.display(), fn_name)
            .map_err(|e| format!("Send failed: {}", e))?;
        self.stdin.flush().map_err(|e| format!("Flush failed: {}", e))?;

        let mut line = String::new();
        self.stdout.read_line(&mut line).map_err(|e| format!("Read failed: {}", e))?;

        let line = line.trim();
        if line.starts_with("OXUR_EXECUTION_COMPLETE") {
            Ok(oxur_lang::Value::Nil)
        } else if line.starts_with("OXUR_RUNTIME_ERROR:") {
            Err(format!("Runtime error: {}", line.strip_prefix("OXUR_RUNTIME_ERROR:").unwrap_or("").trim()))
        } else {
            Err(format!("Unexpected response: {}", line))
        }
    }

    pub fn kill(&mut self) -> Result<(), std::io::Error> {
        self.subprocess.kill()
    }

    pub fn restart(&mut self) -> Result<(), String> {
        let new = Self::spawn().map_err(|e| format!("Restart failed: {}", e))?;
        *self = new;
        Ok(())
    }
}

impl Drop for SubprocessExecutor {
    fn drop(&mut self) { let _ = self.subprocess.kill(); }
}
```

#### 5.4 Artifact Cache

**Location:** `src/cache.rs`

```rust
use oxur_smap::SourceMap;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Content-based cache for compiled artifacts
/// 
/// Cache survives REPL restarts - artifacts stored on disk.
/// 
/// Locations:
/// - Linux: ~/.cache/oxur/artifacts/
/// - macOS: ~/Library/Caches/oxur/artifacts/
/// - Windows: %LOCALAPPDATA%\oxur\cache\artifacts\
pub struct ArtifactCache {
    cache_dir: PathBuf,
    index: HashMap<String, CachedArtifact>,
}

struct CachedArtifact {
    path: PathBuf,
    created: SystemTime,
}

impl ArtifactCache {
    pub fn new() -> Result<Self, std::io::Error> {
        let cache_dir = Self::cache_directory()?;
        std::fs::create_dir_all(&cache_dir)?;
        let index = Self::load_index(&cache_dir)?;
        Ok(Self { cache_dir, index })
    }

    fn cache_directory() -> Result<PathBuf, std::io::Error> {
        #[cfg(target_os = "linux")]
        { Ok(PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/oxur/artifacts")) }
        #[cfg(target_os = "macos")]
        { Ok(PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Library/Caches/oxur/artifacts")) }
        #[cfg(target_os = "windows")]
        { Ok(PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default()).join("oxur\\cache\\artifacts")) }
    }

    fn load_index(cache_dir: &PathBuf) -> Result<HashMap<String, CachedArtifact>, std::io::Error> {
        let mut index = HashMap::new();
        if cache_dir.exists() {
            for entry in std::fs::read_dir(cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let metadata = entry.metadata()?;
                    index.insert(stem.to_string(), CachedArtifact {
                        path: path.clone(),
                        created: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
                    });
                }
            }
        }
        Ok(index)
    }

    pub fn compute_key(&self, source: &str, source_map: &SourceMap) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        hasher.update(source_map.content_hash().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn get(&self, key: &str) -> Option<PathBuf> {
        self.index.get(key).map(|a| a.path.clone())
    }

    pub fn insert(&mut self, key: String, artifact_path: PathBuf) -> Result<(), std::io::Error> {
        let cached_path = self.cache_dir.join(&key).with_extension(
            artifact_path.extension().unwrap_or_default()
        );
        std::fs::copy(&artifact_path, &cached_path)?;
        self.index.insert(key, CachedArtifact {
            path: cached_path,
            created: SystemTime::now(),
        });
        Ok(())
    }

    pub fn evict_lru(&mut self, keep_n: usize) {
        if self.index.len() <= keep_n { return; }
        let mut entries: Vec<_> = self.index.iter().collect();
        entries.sort_by_key(|(_, v)| v.created);
        let to_remove = entries.len() - keep_n;
        for (key, artifact) in entries.into_iter().take(to_remove) {
            let _ = std::fs::remove_file(&artifact.path);
            self.index.remove(key);
        }
    }
}
```
### Layer 6: Server Implementation

#### Server Core

**Location:** `src/server/server.rs`

```rust
use crate::transport::{TransportListener, Stream};
use crate::protocol::codec::Codec;
use crate::session::SessionManager;
use crate::cache::ArtifactCache;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use std::sync::Arc;
use std::time::Duration;

pub struct ServerConfig<T: TransportListener, C: Codec> {
    pub transport: T,
    pub codec: C,
    pub max_sessions: usize,
    pub session_timeout: Duration,
    pub request_timeout: Duration,
}

pub struct ReplServer<T: TransportListener, C: Codec> {
    config: ServerConfig<T, C>,
    sessions: Arc<SessionManager>,
    cache: Arc<Mutex<ArtifactCache>>,
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: mpsc::Receiver<()>,
}

impl<T: TransportListener, C: Codec> ReplServer<T, C> {
    pub fn new(config: ServerConfig<T, C>) -> Result<Self, ServerError> {
        let cache = Arc::new(Mutex::new(ArtifactCache::new().map_err(ServerError::Cache)?));
        let sessions = Arc::new(SessionManager::new(
            config.max_sessions, config.session_timeout, cache.clone(),
        ));
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        Ok(Self { config, sessions, cache, shutdown_tx, shutdown_rx })
    }

    pub async fn serve(mut self) -> Result<(), ServerError> {
        let sessions_cleanup = self.sessions.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                sessions_cleanup.cleanup_expired().await;
            }
        });

        loop {
            tokio::select! {
                stream = self.config.transport.accept() => {
                    let stream = stream.map_err(ServerError::Transport)?;
                    let codec = self.config.codec.clone();
                    let sessions = self.sessions.clone();
                    let timeout = self.config.request_timeout;
                    tokio::spawn(async move {
                        let _ = handle_connection(stream, codec, sessions, timeout).await;
                    });
                }
                _ = self.shutdown_rx.recv() => break,
            }
        }
        self.sessions.close_all().await;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), ServerError> {
        self.shutdown_tx.send(()).await.map_err(|_| ServerError::ShutdownFailed)
    }
}

async fn handle_connection<S: Stream, C: Codec>(
    stream: S,
    codec: C,
    sessions: Arc<SessionManager>,
    timeout: Duration,
) -> Result<(), ConnectionError> {
    use futures::{SinkExt, StreamExt};
    use crate::server::handler::MessageHandler;
    use crate::protocol::{Request, Response};

    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
    let handler = MessageHandler::new(sessions);

    loop {
        tokio::select! {
            msg = framed.next() => {
                let bytes = match msg {
                    Some(Ok(b)) => b,
                    Some(Err(_)) => continue,
                    None => break,
                };

                let request: Request = match codec.decode_request(&bytes) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let response = match tokio::time::timeout(timeout, handler.handle_request(request.clone())).await {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => error_response(&request, e),
                    Err(_) => timeout_response(&request),
                };

                let bytes = codec.encode_response(&response)?;
                framed.send(bytes.into()).await?;
            }
            _ = tokio::time::sleep(timeout * 10) => break,
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Transport error: {0}")]
    Transport(#[from] std::io::Error),
    #[error("Cache error: {0}")]
    Cache(std::io::Error),
    #[error("Shutdown failed")]
    ShutdownFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### Message Handler

**Location:** `src/server/handler.rs`

```rust
use crate::protocol::{Request, Response, Operation, Status, ErrorInfo, ErrorKind};
use crate::session::SessionManager;
use crate::eval::EvalError;
use std::collections::HashMap;
use std::sync::Arc;

pub struct MessageHandler {
    sessions: Arc<SessionManager>,
}

impl MessageHandler {
    pub fn new(sessions: Arc<SessionManager>) -> Self { Self { sessions } }

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
            out: None, err: None,
            status: vec![Status::Done, Status::SessionCreated],
            error: None,
            data: HashMap::new(),
        })
    }

    async fn handle_eval(&self, req: Request) -> Result<Response, HandlerError> {
        let code = req.params.get("code").and_then(|v| v.as_str())
            .ok_or(HandlerError::MissingParameter("code"))?;

        let ctx = self.sessions.get_context(&req.session).await?;
        let mut ctx = ctx.lock().await;
        let result = ctx.eval(code).await;
        let (stdout, stderr) = ctx.take_output();
        ctx.record_history(code.to_string(), result.clone());

        match result {
            Ok(value) => Ok(Response {
                id: req.id, session: req.session,
                value: Some(serde_json::json!(value.to_string())),
                out: if stdout.is_empty() { None } else { Some(stdout) },
                err: if stderr.is_empty() { None } else { Some(stderr) },
                status: vec![Status::Done],
                error: None, data: HashMap::new(),
            }),
            Err(e) => {
                let kind = error_to_kind(&e);
                Ok(Response {
                    id: req.id, session: req.session,
                    value: None,
                    out: if stdout.is_empty() { None } else { Some(stdout) },
                    err: Some(format!("{}{}", stderr, e)),
                    status: vec![Status::Error],
                    error: Some(ErrorInfo { kind, message: e.to_string(), source_location: None, stack_trace: vec![] }),
                    data: HashMap::new(),
                })
            }
        }
    }

    async fn handle_interrupt(&self, req: Request) -> Result<Response, HandlerError> {
        let ctx = self.sessions.get_context(&req.session).await?;
        let mut ctx = ctx.lock().await;
        ctx.interrupt().map_err(|e| HandlerError::Internal(e.to_string()))?;
        Ok(Response {
            id: req.id, session: req.session,
            value: None, out: None,
            err: Some("Execution interrupted".to_string()),
            status: vec![Status::Done, Status::Interrupted],
            error: None, data: HashMap::new(),
        })
    }

    async fn handle_close(&self, req: Request) -> Result<Response, HandlerError> {
        self.sessions.close_session(&req.session).await?;
        Ok(Response {
            id: req.id, session: req.session,
            value: None, out: None, err: None,
            status: vec![Status::Done, Status::SessionClosed],
            error: None, data: HashMap::new(),
        })
    }

    async fn handle_ls_sessions(&self, req: Request) -> Result<Response, HandlerError> {
        let sessions = self.sessions.list_sessions().await;
        Ok(Response {
            id: req.id, session: req.session,
            value: Some(serde_json::json!({ "sessions": sessions })),
            out: None, err: None,
            status: vec![Status::Done],
            error: None, data: HashMap::new(),
        })
    }

    async fn handle_describe(&self, req: Request) -> Result<Response, HandlerError> {
        Ok(Response {
            id: req.id, session: req.session,
            value: Some(serde_json::json!({
                "versions": { "oxur-repl": env!("CARGO_PKG_VERSION"), "protocol": "1.1" },
                "ops": ["clone", "eval", "load-file", "interrupt", "close", "ls-sessions", "describe", "history", "clear-output"],
                "modes": ["lisp", "sexpr"],
                "features": { "subprocess-execution": true, "artifact-caching": true, "source-maps": true }
            })),
            out: None, err: None,
            status: vec![Status::Done],
            error: None, data: HashMap::new(),
        })
    }

    async fn handle_load_file(&self, req: Request) -> Result<Response, HandlerError> {
        let file = req.params.get("file").and_then(|v| v.as_str())
            .ok_or(HandlerError::MissingParameter("file"))?;
        let ctx = self.sessions.get_context(&req.session).await?;
        let mut ctx = ctx.lock().await;
        let result = ctx.load_file(file).await;
        let (stdout, stderr) = ctx.take_output();
        // Similar response construction as handle_eval
        match result {
            Ok(value) => Ok(Response {
                id: req.id, session: req.session,
                value: Some(serde_json::json!(value.to_string())),
                out: if stdout.is_empty() { None } else { Some(stdout) },
                err: if stderr.is_empty() { None } else { Some(stderr) },
                status: vec![Status::Done], error: None, data: HashMap::new(),
            }),
            Err(e) => Ok(Response {
                id: req.id, session: req.session,
                value: None,
                out: if stdout.is_empty() { None } else { Some(stdout) },
                err: Some(format!("{}{}", stderr, e)),
                status: vec![Status::Error],
                error: Some(ErrorInfo { kind: error_to_kind(&e), message: e.to_string(), source_location: None, stack_trace: vec![] }),
                data: HashMap::new(),
            }),
        }
    }

    async fn handle_history(&self, req: Request) -> Result<Response, HandlerError> {
        let ctx = self.sessions.get_context(&req.session).await?;
        let ctx = ctx.lock().await;
        let history: Vec<_> = ctx.get_history().iter()
            .map(|e| serde_json::json!({ "code": e.code, "success": e.success }))
            .collect();
        Ok(Response {
            id: req.id, session: req.session,
            value: Some(serde_json::json!({ "history": history })),
            out: None, err: None, status: vec![Status::Done],
            error: None, data: HashMap::new(),
        })
    }

    async fn handle_clear_output(&self, req: Request) -> Result<Response, HandlerError> {
        let ctx = self.sessions.get_context(&req.session).await?;
        ctx.lock().await.take_output();
        Ok(Response {
            id: req.id, session: req.session,
            value: None, out: None, err: None,
            status: vec![Status::Done], error: None, data: HashMap::new(),
        })
    }
}

fn error_to_kind(e: &EvalError) -> ErrorKind {
    match e {
        EvalError::Parse(_) => ErrorKind::Parse,
        EvalError::Expand(_) => ErrorKind::Expand,
        EvalError::Lower(_) => ErrorKind::Lower,
        EvalError::Runtime(_) => ErrorKind::Eval,
        EvalError::Io(_) => ErrorKind::Io,
        EvalError::Internal(_) => ErrorKind::Internal,
        EvalError::SubprocessCrash(_) => ErrorKind::SubprocessCrash,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Missing parameter: {0}")]
    MissingParameter(&'static str),
    #[error("Session error: {0}")]
    Session(#[from] crate::session::SessionError),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl HandlerError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::MissingParameter(_) => ErrorKind::Protocol,
            Self::Session(_) => ErrorKind::Session,
            Self::Internal(_) => ErrorKind::Internal,
        }
    }
}
```

#### Session Manager

**Location:** `src/session/manager.rs`

```rust
use crate::eval::EvalContext;
use crate::protocol::ReplMode;
use crate::cache::ArtifactCache;
use tokio::sync::{RwLock, Mutex};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct SessionManager {
    sessions: RwLock<HashMap<String, Session>>,
    max_sessions: usize,
    session_timeout: Duration,
    cache: Arc<Mutex<ArtifactCache>>,
}

struct Session {
    context: Arc<Mutex<EvalContext>>,
    created_at: Instant,
    last_used: Instant,
}

impl SessionManager {
    pub fn new(max_sessions: usize, session_timeout: Duration, cache: Arc<Mutex<ArtifactCache>>) -> Self {
        Self { sessions: RwLock::new(HashMap::new()), max_sessions, session_timeout, cache }
    }

    pub async fn create_session(&self, session_id: String, mode: ReplMode) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        if sessions.len() >= self.max_sessions {
            return Err(SessionError::TooManySessions);
        }
        let context = EvalContext::new(session_id.clone(), mode, self.cache.clone())
            .map_err(|e| SessionError::Creation(e.to_string()))?;
        sessions.insert(session_id, Session {
            context: Arc::new(Mutex::new(context)),
            created_at: Instant::now(),
            last_used: Instant::now(),
        });
        Ok(())
    }

    pub async fn get_context(&self, session_id: &str) -> Result<Arc<Mutex<EvalContext>>, SessionError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).ok_or(SessionError::NotFound)?;
        session.last_used = Instant::now();
        Ok(session.context.clone())
    }

    pub async fn close_session(&self, session_id: &str) -> Result<(), SessionError> {
        self.sessions.write().await.remove(session_id).ok_or(SessionError::NotFound)?;
        Ok(())
    }

    pub async fn list_sessions(&self) -> Vec<String> {
        self.sessions.read().await.keys().cloned().collect()
    }

    pub async fn close_all(&self) {
        self.sessions.write().await.clear();
    }

    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        let now = Instant::now();
        sessions.retain(|_, s| now.duration_since(s.last_used) < self.session_timeout);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Too many sessions")]
    TooManySessions,
    #[error("Creation failed: {0}")]
    Creation(String),
}
```
### Layer 7: Client Implementation

**Location:** `src/client/`

```rust
use crate::protocol::{Request, Response, Operation, Status, ReplMode, MessageId, ErrorInfo};
use crate::protocol::codec::Codec;
use crate::transport::ConnectionString;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ReplClient<C: Codec> {
    framed: Arc<Mutex<Framed<TcpStream, LengthDelimitedCodec>>>,
    codec: C,
    session_id: Option<String>,
    mode: ReplMode,
}

impl<C: Codec> ReplClient<C> {
    pub async fn connect(addr: &str, codec: C) -> Result<Self, ClientError> {
        let conn_str = ConnectionString::parse(addr).map_err(ClientError::Io)?;
        let stream = match conn_str {
            ConnectionString::Tcp(addr) => TcpStream::connect(addr).await?,
            _ => return Err(ClientError::Transport("Only TCP implemented".to_string())),
        };
        let framed = Framed::new(stream, LengthDelimitedCodec::new());
        Ok(Self { framed: Arc::new(Mutex::new(framed)), codec, session_id: None, mode: ReplMode::default() })
    }

    pub async fn clone_session(&mut self) -> Result<String, ClientError> {
        let req = Request { id: Self::generate_id(), session: String::new(), op: Operation::Clone, mode: self.mode, params: HashMap::new() };
        let resp = self.send_request(req).await?;
        if resp.status.contains(&Status::SessionCreated) {
            let session_id = resp.value.and_then(|v| v.get("new-session")).and_then(|v| v.as_str())
                .ok_or(ClientError::InvalidResponse)?.to_string();
            self.session_id = Some(session_id.clone());
            Ok(session_id)
        } else {
            Err(ClientError::OperationFailed(resp))
        }
    }

    pub async fn eval(&self, code: &str) -> Result<EvalResult, ClientError> {
        let session = self.session_id.as_ref().ok_or(ClientError::NoSession)?;
        let mut params = HashMap::new();
        params.insert("code".to_string(), serde_json::Value::String(code.to_string()));
        let req = Request { id: Self::generate_id(), session: session.clone(), op: Operation::Eval, mode: self.mode, params };
        let resp = self.send_request(req).await?;
        Ok(EvalResult { value: resp.value, stdout: resp.out.unwrap_or_default(), stderr: resp.err.unwrap_or_default(), status: resp.status, error: resp.error })
    }

    pub async fn load_file(&self, path: &str) -> Result<EvalResult, ClientError> {
        let session = self.session_id.as_ref().ok_or(ClientError::NoSession)?;
        let mut params = HashMap::new();
        params.insert("file".to_string(), serde_json::Value::String(path.to_string()));
        let req = Request { id: Self::generate_id(), session: session.clone(), op: Operation::LoadFile, mode: self.mode, params };
        let resp = self.send_request(req).await?;
        Ok(EvalResult { value: resp.value, stdout: resp.out.unwrap_or_default(), stderr: resp.err.unwrap_or_default(), status: resp.status, error: resp.error })
    }

    pub async fn interrupt(&self) -> Result<(), ClientError> {
        let session = self.session_id.as_ref().ok_or(ClientError::NoSession)?;
        let req = Request { id: Self::generate_id(), session: session.clone(), op: Operation::Interrupt, mode: self.mode, params: HashMap::new() };
        let resp = self.send_request(req).await?;
        if resp.status.contains(&Status::Interrupted) || resp.status.contains(&Status::Done) { Ok(()) }
        else { Err(ClientError::OperationFailed(resp)) }
    }

    pub fn set_mode(&mut self, mode: ReplMode) { self.mode = mode; }
    pub fn mode(&self) -> ReplMode { self.mode }

    pub async fn close(&mut self) -> Result<(), ClientError> {
        if let Some(session) = &self.session_id {
            let req = Request { id: Self::generate_id(), session: session.clone(), op: Operation::Close, mode: self.mode, params: HashMap::new() };
            self.send_request(req).await?;
            self.session_id = None;
        }
        Ok(())
    }

    async fn send_request(&self, req: Request) -> Result<Response, ClientError> {
        let bytes = self.codec.encode_request(&req).map_err(|e| ClientError::Transport(e.to_string()))?;
        let mut framed = self.framed.lock().await;
        framed.send(bytes.into()).await.map_err(|e| ClientError::Transport(e.to_string()))?;
        let resp_bytes = framed.next().await.ok_or(ClientError::ConnectionClosed)?.map_err(|e| ClientError::Transport(e.to_string()))?;
        self.codec.decode_response(&resp_bytes).map_err(|e| ClientError::Transport(e.to_string()))
    }

    fn generate_id() -> MessageId { uuid::Uuid::new_v4().to_string() }
}

pub struct EvalResult {
    pub value: Option<serde_json::Value>,
    pub stdout: String,
    pub stderr: String,
    pub status: Vec<Status>,
    pub error: Option<ErrorInfo>,
}

impl EvalResult {
    pub fn is_success(&self) -> bool { self.status.contains(&Status::Done) && self.error.is_none() }
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
}
```

## Section 8: Subprocess Protocol (Internal)

The server uses a text protocol over stdin/stdout to communicate with the subprocess.

### Protocol Format

```
Commands (Server → Subprocess via stdin):
  LOAD_AND_RUN <lib_path> <function_name>\n

Responses (Subprocess → Server via stdout):
  OXUR_EXECUTION_COMPLETE\n                    (success)
  OXUR_RUNTIME_ERROR: <error_message>\n        (runtime error)
  OXUR_PANIC_LOCATION: <file>:<line>:<col>\n   (optional panic location)
```

### Subprocess Binary

**Location:** `src/bin/subprocess.rs`

```rust
use std::io::{self, BufRead, Write};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::any::Any;

type VariableStore = HashMap<String, Box<dyn Any + 'static>>;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut variable_store: VariableStore = HashMap::new();
    let mut loaded_libraries: Vec<Library> = Vec::new();

    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };

        if line.starts_with("LOAD_AND_RUN") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 3 {
                writeln!(stdout, "OXUR_RUNTIME_ERROR: Invalid command").unwrap();
                stdout.flush().unwrap();
                continue;
            }

            match load_and_execute(parts[1], parts[2], &mut variable_store, &mut loaded_libraries) {
                Ok(_) => writeln!(stdout, "OXUR_EXECUTION_COMPLETE").unwrap(),
                Err(e) => writeln!(stdout, "OXUR_RUNTIME_ERROR: {}", e).unwrap(),
            }
            stdout.flush().unwrap();
        }
    }
}

fn load_and_execute(lib_path: &str, fn_name: &str, vars: &mut VariableStore, libs: &mut Vec<Library>) -> Result<(), String> {
    let lib = unsafe { Library::new(lib_path).map_err(|e| format!("Load failed: {}", e))? };
    let func: Symbol<extern "C" fn(&mut VariableStore) -> Box<dyn Any + 'static>> = unsafe {
        lib.get(fn_name.as_bytes()).map_err(|e| format!("Symbol not found: {}", e))?
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| func(vars)));
    libs.push(lib);
    match result {
        Ok(_) => Ok(()),
        Err(panic) => {
            let msg = panic.downcast_ref::<&str>().map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "Unknown panic".to_string());
            Err(format!("Panic: {}", msg))
        }
    }
}
```

## Protocol Specification

### Message Flow Examples

#### Session Creation and Evaluation
```
Client → Server: { "id": "msg-1", "op": "clone", "mode": "lisp" }
Server → Client: { "id": "msg-1", "session": "abc123", "status": ["done", "session-created"] }

Client → Server: { "id": "msg-2", "session": "abc123", "op": "eval", "params": {"code": "(+ 1 2)"} }
Server → Client: { "id": "msg-2", "session": "abc123", "value": 3, "status": ["done"] }
```

#### Error with Source Location
```
Client → Server: { "id": "msg-3", "session": "abc123", "op": "eval", "params": {"code": "(+ 1"} }
Server → Client: { "id": "msg-3", "session": "abc123", "status": ["error"],
  "error": { "kind": "parse", "message": "Unexpected EOF", "source_location": {"file": "<repl>", "line": 1, "column": 5} } }
```

#### Interrupt
```
Client → Server: { "id": "msg-4", "session": "abc123", "op": "interrupt" }
Server → Client: { "id": "msg-4", "session": "abc123", "err": "Execution interrupted", "status": ["done", "interrupted"] }
```

### Connection String Formats

| Format | Transport | Example |
|--------|-----------|---------|
| `tcp://host:port` | TCP | `tcp://127.0.0.1:7888` |
| `host:port` | TCP (implicit) | `localhost:7888` |
| `unix://path` | Unix socket | `unix:///tmp/oxur.sock` |
| `pipe://name` | Named pipe | `pipe://oxur-repl` |
| `in-process` | In-process | `in-process` |

## Dependencies

```toml
[package]
name = "oxur-repl"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "oxur-repl-subprocess"
path = "src/bin/subprocess.rs"

[dependencies]
tokio = { version = "1.48", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
postcard = { version = "1.1", features = ["alloc"] }
tokio-util = { version = "0.7", features = ["codec"] }
bytes = "1.5"
futures = "0.3"
async-trait = "0.1"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
libloading = "0.8"
sha2 = "0.10"
oxur-smap = { path = "../oxur-smap" }
oxur-lang = { path = "../oxur-lang" }
oxur-comp = { path = "../oxur-comp" }
oxur-ast = { path = "../oxur-ast" }

[dev-dependencies]
criterion = "0.5"
tempfile = "3.8"
```

## Three-Tier Execution Strategy

| Tier | Criteria | Execution | Latency |
|------|----------|-----------|---------|
| **Tier 1 (Calculator)** | Simple arithmetic (<10 nodes) | Direct Rust eval | <1ms |
| **Tier 2 (Cached)** | Cache hit from ArtifactCache | Load .so + execute | 1-5ms |
| **Tier 3 (JIT)** | Cache miss | Full compilation | 50-300ms |

Cache makes REPL feel like interpreter while being compiler:
```
First:  (defn square [x] (* x x))  → Tier 3  ~100ms
Second: (square 5)                  → Tier 2  ~2ms
[REPL restart]
Third:  (defn square [x] (* x x))  → Tier 2  ~2ms  ← Cache persists!
```

## Resource Management

| Resource | Default |
|----------|---------|
| Max sessions | 100 |
| Session timeout | 30 min |
| Cache size | 1GB / 1000 artifacts |
| Temp directory | `/dev/shm` (Linux), system temp (others) |

## Version History

### Version 1.1 (2026-01-05)
- Added mandatory subprocess execution model
- Added ArtifactCache for persistent caching
- Added oxur-smap integration for source maps
- Updated three-tier execution to cache-centric model
- Added subprocess protocol documentation
- Updated EvalContext to own CachedCompiler
- Added ErrorKind::SubprocessCrash
- Updated Rust edition to 2021

### Version 1.0 (2025-12-28)
- Initial protocol specification

## Glossary

- **Postcard**: Rust-native binary serialization
- **Subprocess**: Isolated process for code execution (mandatory for Ctrl-C)
- **ArtifactCache**: Content-based cache for compiled .so/.dylib files
- **SourceMap**: Multi-stage transformation tracking (Surface → Core → Rust)
- **Three-Tier**: Calculator (Tier 1), Cached (Tier 2), JIT (Tier 3)
- **Surface Forms**: Parsed Oxur syntax before macro expansion
- **Core Forms**: Canonical s-expressions after macro expansion
- **RustAstWrapper**: Component that wraps lowered Rust AST with REPL scaffolding

---

*End of Document*