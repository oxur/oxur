---
number: 17
title: "Recommendations for Future-proofing Multiple REPL Protocols"
author: "Duncan McGreggor & Claude"
component: REPL
tags: [Protocol, Design]
created: 2025-12-28
updated: 2025-12-28
state: Under Review
supersedes: null
superseded-by: null
version: 1.0
---

# Recommendations for Future-proofing Multiple REPL Protocols

**Status:** Design Proposal
**Author:** Research Analysis
**Date:** December 2025
**Target:** Oxur REPL Protocol v0.1+

## Executive Summary

Oxur's REPL protocol should start with **postcard** serialization for optimal performance with Rust clients, while maintaining architectural flexibility to add **MessagePack** support later if cross-language demand emerges. The key insight: protocol message definitions remain serialization-agnostic, allowing multiple wire formats to coexist without code duplication or breaking changes.

**Recommended Initial Choice:** postcard (3.4x faster serialization, smaller wire format, Rust-native ecosystem)
**Migration Path:** Clean addition of MessagePack via multi-protocol server when needed
**Architecture Pattern:** Trait-based protocol abstraction with zero-cost monomorphization

## Why Start with Postcard

### Performance Advantages

- **3.4x faster serialization** (423µs vs 1,447µs for MessagePack)
- **Smaller wire format** (724KB vs 784KB in benchmarks)
- **Lower deserialization overhead** (2.2ms vs 3.0ms)
- **Varint encoding** - efficient for small integers common in REPL protocols

### Ecosystem Benefits

- **16.8M+ downloads** - battle-tested in production
- **Stable wire format** since v1.0.0 (June 2022, Mozilla-sponsored)
- **postcard-rpc crate** - turnkey RPC layer with request/response patterns
- **No_std support** - works in embedded contexts (future-proofing for potential embedded use)
- **Active maintenance** - version 1.1.3 as of late 2024

### Strategic Fit

- **Primary audience is Rust developers** - postcard is idiomatic and ergonomic
- **Serde-native** - `#[derive(Serialize, Deserialize)]` just works
- **Type safety** - Rust's type system catches protocol mismatches at compile time

### Trade-off Acknowledged

- **Rust-only clients** initially - no existing Python/JavaScript/Go implementations
- **Requires custom implementation** for non-Rust languages to parse wire format
- **Acceptable trade-off** if Rust client ecosystem is primary focus

## Architecture for Multi-Protocol Support

The core principle: **separate protocol semantics from wire serialization**.

### Layer 1: Protocol Message Definitions (Serialization-Agnostic)

```rust
// oxur-protocol/src/messages.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core request message - works with ANY serde format
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Request {
    /// Correlation ID for request/response matching
    pub id: String,

    /// Session UUID for state isolation
    pub session: String,

    /// Operation to perform
    pub op: Operation,

    /// REPL mode: Lisp syntax vs s-expression AST
    #[serde(default)]
    pub mode: ReplMode,

    /// Operation-specific parameters
    #[serde(flatten)]
    pub params: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Response {
    /// Echoed correlation ID from request
    pub id: String,

    /// Session UUID
    pub session: String,

    /// Evaluation result (if complete)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Streaming stdout output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,

    /// Streaming stderr output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub err: Option<String>,

    /// Status indicators: ["done"], ["error"], ["interrupted"]
    pub status: Vec<String>,

    /// Error details (if status includes "error")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Operation {
    Clone,
    Eval,
    Interrupt,
    Close,
    Describe,
    LoadFile,
    ListSessions,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum ReplMode {
    #[default]
    Lisp,    // Oxur syntax parser
    Sexpr,   // S-expression AST
}
```

**Key insight:** These types have **zero knowledge** of postcard, MessagePack, or any specific serialization format. They only know serde's abstract data model.

### Layer 2: Protocol Trait (Abstraction)

```rust
// oxur-transport/src/protocol.rs
use async_trait::async_trait;
use bytes::Bytes;
use oxur_protocol::{Request, Response};

/// Abstraction over wire serialization format
#[async_trait]
pub trait Protocol: Send + Sync + 'static {
    /// Serialize a response to bytes
    async fn serialize_response(&self, msg: &Response) -> Result<Bytes, ProtocolError>;

    /// Deserialize bytes to a request
    async fn deserialize_request(&self, bytes: Bytes) -> Result<Request, ProtocolError>;

    /// Protocol identifier for logging/metrics
    fn name(&self) -> &'static str;

    /// Protocol version for capability negotiation
    fn version(&self) -> u32;
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Serialization failed: {0}")]
    SerializationError(String),

    #[error("Deserialization failed: {0}")]
    DeserializationError(String),

    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u32),
}
```

### Layer 3: Concrete Protocol Implementations

#### Postcard Protocol

```rust
// oxur-transport/src/postcard_protocol.rs
use async_trait::async_trait;
use bytes::Bytes;
use crate::protocol::{Protocol, ProtocolError};
use oxur_protocol::{Request, Response};

pub struct PostcardProtocol {
    // Future: add configuration options like max_size, compression, etc.
}

impl PostcardProtocol {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Protocol for PostcardProtocol {
    async fn serialize_response(&self, msg: &Response) -> Result<Bytes, ProtocolError> {
        postcard::to_allocvec(msg)
            .map(Bytes::from)
            .map_err(|e| ProtocolError::SerializationError(e.to_string()))
    }

    async fn deserialize_request(&self, bytes: Bytes) -> Result<Request, ProtocolError> {
        postcard::from_bytes(&bytes)
            .map_err(|e| ProtocolError::DeserializationError(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "postcard"
    }

    fn version(&self) -> u32 {
        1 // Postcard wire format version
    }
}
```

#### MessagePack Protocol (Future Addition)

```rust
// oxur-transport/src/messagepack_protocol.rs
use async_trait::async_trait;
use bytes::Bytes;
use crate::protocol::{Protocol, ProtocolError};
use oxur_protocol::{Request, Response};

pub struct MessagePackProtocol {
    // Configuration for named vs compact encoding, etc.
}

impl MessagePackProtocol {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Protocol for MessagePackProtocol {
    async fn serialize_response(&self, msg: &Response) -> Result<Bytes, ProtocolError> {
        rmp_serde::to_vec(msg)
            .map(Bytes::from)
            .map_err(|e| ProtocolError::SerializationError(e.to_string()))
    }

    async fn deserialize_request(&self, bytes: Bytes) -> Result<Request, ProtocolError> {
        rmp_serde::from_slice(&bytes)
            .map_err(|e| ProtocolError::DeserializationError(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "messagepack"
    }

    fn version(&self) -> u32 {
        1
    }
}
```

**Cost of adding MessagePack:** ~50 lines of code. The message definitions, evaluation engine, session management, transport handling - all unchanged.

## Migration Strategies

### Option 1: Multi-Protocol Server (Recommended)

Run both protocols on different ports/endpoints. **Zero breaking changes** for existing clients.

```rust
// oxur-server/src/main.rs
use oxur_server::{Server, ServerConfig};
use oxur_transport::{PostcardProtocol, MessagePackProtocol, TcpTransport};
use oxur_eval::EvalEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Shared evaluation engine and session state
    let eval_engine = Arc::new(EvalEngine::new());
    let session_manager = Arc::new(SessionManager::new());

    // Postcard server on default port (existing clients)
    let postcard_config = ServerConfig {
        bind_addr: "127.0.0.1:7888".parse()?,
        protocol: PostcardProtocol::new(),
        max_connections: 1000,
        request_timeout: Duration::from_secs(300),
    };

    let postcard_server = Server::new(
        postcard_config,
        eval_engine.clone(),
        session_manager.clone(),
    );

    // MessagePack server on alternate port (new clients)
    let msgpack_config = ServerConfig {
        bind_addr: "127.0.0.1:7889".parse()?,
        protocol: MessagePackProtocol::new(),
        max_connections: 1000,
        request_timeout: Duration::from_secs(300),
    };

    let msgpack_server = Server::new(
        msgpack_config,
        eval_engine,
        session_manager,
    );

    // Run both servers concurrently
    tokio::try_join!(
        postcard_server.serve(),
        msgpack_server.serve(),
    )?;

    Ok(())
}
```

**Benefits:**

- **No breaking changes** - existing Rust clients continue on port 7888
- **Opt-in migration** - clients choose when to upgrade
- **Shared state** - sessions created on postcard port visible to msgpack clients (same SessionManager)
- **Independent evolution** - can version protocols separately
- **Simple configuration** - users specify protocol in connection string: `oxur://localhost:7888` vs `oxur-msgpack://localhost:7889`

**Deployment considerations:**

- **Firewall rules** - open both ports or use load balancer routing
- **Monitoring** - separate metrics per protocol (e.g., `oxur.requests{protocol="postcard"}`)
- **Documentation** - clearly communicate port assignments

### Option 2: Protocol Negotiation at Connection

Client announces protocol preference during handshake. **Single port** with automatic adaptation.

```rust
// oxur-transport/src/negotiation.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ProtocolId {
    PostcardV1,
    MessagePackV1,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientHello {
    pub preferred_protocol: ProtocolId,
    pub supported_protocols: Vec<ProtocolId>,
    pub client_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerHello {
    pub selected_protocol: ProtocolId,
    pub server_version: String,
}

// Server connection handler
async fn handle_connection(stream: TcpStream) -> Result<()> {
    // Read client hello (always encoded as MessagePack for universality)
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
    let hello_bytes = framed.next().await.ok_or("No hello")??;
    let client_hello: ClientHello = rmp_serde::from_slice(&hello_bytes)?;

    // Select protocol (prefer client's first choice if we support it)
    let protocol: Box<dyn Protocol> = match client_hello.preferred_protocol {
        ProtocolId::PostcardV1 => Box::new(PostcardProtocol::new()),
        ProtocolId::MessagePackV1 => Box::new(MessagePackProtocol::new()),
    };

    // Send server hello
    let server_hello = ServerHello {
        selected_protocol: client_hello.preferred_protocol,
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let hello_response = rmp_serde::to_vec(&server_hello)?;
    framed.send(hello_response.into()).await?;

    // Continue with selected protocol
    handle_protocol_session(framed, protocol).await
}
```

**Benefits:**

- **Single port** - simpler deployment
- **Automatic selection** - no manual configuration needed
- **Version negotiation** - can deprecate old protocols over time
- **Fallback support** - client can list multiple protocols in preference order

**Trade-offs:**

- **Extra round-trip** on connection (typically <1ms on loopback)
- **Handshake complexity** - need to agree on hello message format (using MessagePack for hello ensures universality)
- **State management** - protocol selection affects entire connection lifecycle

### Option 3: Per-Message Protocol Tagging

Each message prefixed with protocol indicator. **Most flexible**, probably overkill.

```rust
// Frame format: [1-byte protocol ID][4-byte length][payload]
#[repr(u8)]
pub enum ProtocolTag {
    Postcard = 0x01,
    MessagePack = 0x02,
}

pub struct TaggedCodec {
    postcard: PostcardProtocol,
    msgpack: MessagePackProtocol,
}

impl Decoder for TaggedCodec {
    type Item = Request;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if src.len() < 5 {
            return Ok(None); // Need at least tag + length
        }

        let tag = src[0];
        let len = u32::from_be_bytes(src[1..5].try_into().unwrap()) as usize;

        if src.len() < 5 + len {
            return Ok(None); // Incomplete message
        }

        let payload = src.split_to(5 + len).split_off(5);

        let request = match tag {
            0x01 => self.postcard.deserialize_request(payload.freeze()).await?,
            0x02 => self.msgpack.deserialize_request(payload.freeze()).await?,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown protocol tag")),
        };

        Ok(Some(request))
    }
}
```

**Benefits:**

- **Per-message flexibility** - mix protocols in same connection
- **Future-proof** - can add more protocols without version bumps
- **Debugging** - protocol visible in wire captures

**Trade-offs:**

- **1 byte overhead per message** (negligible for REPL use)
- **Increased complexity** - codec must handle multiple formats
- **Unusual pattern** - most RPC systems fix protocol per-connection

**Verdict:** Only consider if you need to mix protocols within a single session (unlikely for REPL use case).

## Implementation Roadmap

### Phase 1: Postcard Foundation (v0.1)

**Goal:** Ship working REPL with optimal Rust performance

```toml
[dependencies]
postcard = { version = "1.1", features = ["alloc"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.48", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
tokio-serde = { version = "0.9" }
```

**Deliverables:**

- `oxur-protocol` crate with message definitions
- `PostcardProtocol` implementation
- Server supporting postcard on port 7888
- Reference Rust client library
- Integration tests

**Documentation:**

- Wire format specification (link to postcard spec)
- Client library examples
- Performance benchmarks

### Phase 2: Protocol Abstraction (v0.2)

**Goal:** Refactor for multi-protocol support (no user-visible changes)

**Changes:**

- Extract `Protocol` trait
- Refactor server to be protocol-agnostic via trait objects
- Add protocol selection to `ServerConfig`
- Protocol-specific metrics in telemetry

**Testing:**

- Verify postcard behavior unchanged
- Add protocol trait property tests

### Phase 3: MessagePack Support (v0.3+)

**Goal:** Add cross-language client support when demand emerges

**Trigger conditions:**

- Request from non-Rust user community
- Integration with non-Rust tooling (e.g., Python data science workflows)
- Cross-language microservice communication

**Changes:**

- Add `MessagePackProtocol` implementation
- Launch second server on port 7889
- Document protocol selection in connection URLs
- Publish protocol spec for client implementers

**Client libraries to consider:**

- Python: `msgpack` + `asyncio` client
- JavaScript: `@msgpack/msgpack` + WebSocket client
- Go: `github.com/vmihailenco/msgpack` client

### Phase 4: Advanced Features (Future)

**When mature:**

- Compression support (via Protocol trait config)
- Protocol versioning and capability negotiation
- Custom protocol plugins via dynamic loading
- Binary protocol debugging tools

## Zero-Cost Abstraction Guarantees

Rust's trait system ensures the protocol abstraction has **no runtime overhead**:

```rust
// Generic over protocol at compile time
pub struct Server<P: Protocol> {
    protocol: P,
    // ...
}

// Monomorphization generates specialized code
let postcard_server = Server::<PostcardProtocol>::new(...);
let msgpack_server = Server::<MessagePackProtocol>::new(...);
```

After compilation:

- No vtable lookups (if using concrete types)
- No heap allocations for protocol handling
- Serialization inlined and optimized per-format
- Binary size increase only for enabled protocols

**Benchmark expectation:** Identical performance to hand-written postcard-only server.

## Migration Decision Tree

```
Does Oxur have non-Rust users requesting access?
├─ No → Stay with postcard-only (current state)
│       - Optimal performance
│       - Simplest codebase
│       - Re-evaluate quarterly
│
└─ Yes → Assess demand scale
         ├─ Small (1-5 users) → Provide MessagePack spec, let them implement
         │                       - Postcard wire format is documented
         │                       - Community contribution opportunity
         │
         └─ Significant (10+ users or strategic partner)
                  → Implement Option 1: Multi-Protocol Server
                     - Add MessagePackProtocol (~50 lines)
                     - Launch on port 7889
                     - Document both endpoints
                     - Minor version bump (0.x.0)
```

## Testing Strategy

### Protocol Compatibility Tests

```rust
#[cfg(test)]
mod protocol_compat_tests {
    use super::*;

    #[test]
    fn postcard_roundtrip() {
        let proto = PostcardProtocol::new();
        let request = Request {
            id: "test-123".into(),
            session: "session-456".into(),
            op: Operation::Eval,
            mode: ReplMode::Lisp,
            params: vec![("code", "(+ 1 2)")].into_iter().collect(),
        };

        let bytes = proto.serialize_request(&request).unwrap();
        let decoded = proto.deserialize_request(bytes).unwrap();
        assert_eq!(request, decoded);
    }

    #[test]
    fn messagepack_roundtrip() {
        // Same test, different protocol
        let proto = MessagePackProtocol::new();
        // ... identical assertions
    }

    #[test]
    fn cross_protocol_semantic_equivalence() {
        let request = /* ... */;

        // Serialize with postcard
        let postcard_bytes = PostcardProtocol::new()
            .serialize_request(&request).unwrap();

        // Serialize with messagepack
        let msgpack_bytes = MessagePackProtocol::new()
            .serialize_request(&request).unwrap();

        // Deserialize each other's output (should fail - different formats)
        // But semantic meaning should be identical after deserialization
        let postcard_decoded = PostcardProtocol::new()
            .deserialize_request(postcard_bytes).unwrap();
        let msgpack_decoded = MessagePackProtocol::new()
            .deserialize_request(msgpack_bytes).unwrap();

        assert_eq!(postcard_decoded, msgpack_decoded);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn multi_protocol_servers_share_state() {
    // Start both servers
    let eval_engine = Arc::new(EvalEngine::new());
    let sessions = Arc::new(SessionManager::new());

    let postcard_server = spawn_server(7888, PostcardProtocol::new(),
                                       eval_engine.clone(), sessions.clone());
    let msgpack_server = spawn_server(7889, MessagePackProtocol::new(),
                                      eval_engine, sessions);

    // Create session via postcard
    let postcard_client = PostcardClient::connect("127.0.0.1:7888").await?;
    let session_id = postcard_client.clone_session().await?;

    // Eval code to set variable
    postcard_client.eval(session_id, "(def x 42)").await?;

    // Connect with msgpack client to same session
    let msgpack_client = MessagePackClient::connect("127.0.0.1:7889").await?;
    let result = msgpack_client.eval(session_id, "x").await?;

    assert_eq!(result.value, Some("42".to_string()));
}
```

## Documentation Requirements

When adding MessagePack support, provide:

### Wire Format Specification

- Message structure (field names, types, encoding)
- Framing protocol (length-delimited with 4-byte prefix)
- Session lifecycle (clone → eval → close)
- Error handling conventions

### Example Clients

- Python reference implementation
- JavaScript/TypeScript reference implementation
- Connection examples for common scenarios

### Migration Guide

- How to upgrade from postcard-only client
- Performance characteristics comparison
- When to use which protocol

## Conclusion

**Start with postcard** for optimal Rust performance and ecosystem fit. The trait-based architecture ensures that adding MessagePack later is:

- **Low-cost:** ~50 lines of code per protocol
- **Non-breaking:** Existing clients unaffected
- **Zero-overhead:** Monomorphization eliminates abstraction cost

**Migration trigger:** Wait for concrete demand from non-Rust users before adding MessagePack support. Premature optimization for cross-language support would sacrifice performance without clear benefit.

**Recommended approach:** Option 1 (Multi-Protocol Server) when the time comes - it's the simplest to implement, explain, and maintain.

The architecture's elegance lies in separating **what** (protocol semantics) from **how** (wire format), allowing Oxur to evolve its serialization strategy without rewriting the REPL's core logic.
