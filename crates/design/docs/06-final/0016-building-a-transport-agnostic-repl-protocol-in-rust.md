---
number: 16
title: "Research: Building a transport-agnostic REPL protocol in Rust"
author: "Duncan McGreggor & Claude"
created: 2025-12-28
updated: 2025-12-28
state: Final
supersedes: null
superseded-by: null
---

# Building a transport-agnostic REPL protocol in Rust

Rust's async ecosystem provides excellent primitives for building an nREPL-style protocol for Oxur, with **MessagePack + Tokio** emerging as the optimal combination. The key finding: unlike Go's bencode approach (Zylisp), Rust can leverage serde's zero-friction integration with high-performance binary formats while maintaining cross-language client compatibility. No mature Rust-native nREPL server exists today—this represents an opportunity to build the definitive implementation.

The recommended architecture separates protocol definition, transport abstraction, and evaluation logic into distinct crates, following rust-analyzer's proven layered design. This enables the dual-mode REPL requirement (Oxur syntax + s-expression AST) through protocol-level `mode` parameters rather than separate server instances.

## Existing Rust REPL protocols offer limited precedent

**evcxr** (6.3k stars) dominates Rust's REPL landscape but lacks a network protocol—it relies on Jupyter's ZMQ infrastructure for remote access. Its architecture nonetheless offers valuable patterns: the `EvalContext` struct with `eval()` method, dynamic library loading via `libloading`, and stdout capture using delimiter markers (`EVCXR_BEGIN_CONTENT`). The crate separation (`evcxr` core, `evcxr_repl` CLI, `evcxr_jupyter` kernel) models excellent separation of concerns.

**lsp-server** (rust-analyzer's foundation) provides the most relevant protocol implementation pattern. It uses JSON-RPC 2.0 with Content-Length framing over stdio/TCP, demonstrating transport abstraction via a `Connection` struct wrapping `crossbeam-channel` pairs. The key insight: protocol handling is fully decoupled from I/O—messages flow through channels while transport runs in separate threads.

**ruply** implements an nREPL *client* in Rust connecting to Clojure servers via bencode, confirming bencode's simplicity but also its limitations (no floats, no booleans). No mature Rust nREPL *server* exists—Oxur could fill this gap.

## MessagePack beats bencode for Rust REPL protocols

Benchmarks from the `rust_serialization_benchmark` project reveal stark performance differences. The recommended serialization stack prioritizes **cross-language compatibility** over raw speed, since REPL latency is dominated by evaluation, not serialization.

| Format | Serialize | Deserialize | Wire Size | Cross-lang | Serde |
|--------|-----------|-------------|-----------|------------|-------|
| **rmp-serde** | 1.38ms | 3.1ms | **24 bytes** | 50+ langs | ✅ Native |
| postcard | 0.45ms | 2.2ms | 28 bytes | Rust-only | ✅ Native |
| bincode 2.0 | 0.30ms | 2.0ms | 35 bytes | Rust-only | ✅ Native |
| prost (protobuf) | 0.90ms | 3.5ms | 40 bytes | 10+ langs | Partial |
| bencode | ~3ms | ~5ms | 48 bytes | Many | Via crate |

**MessagePack (rmp-serde)** emerges as the primary recommendation for these reasons:

- **Smallest wire size**: Critical for responsive REPL feedback loops
- **50+ language implementations**: Clients in Python, JavaScript, Clojure, Go, Ruby already exist
- **Full serde integration**: `#[derive(Serialize, Deserialize)]` works seamlessly
- **Self-describing format**: Clients can parse without schema files, enabling dynamic tooling
- **Better than bencode**: Native float/boolean support, smaller encoding, faster parsing

For Rust-only clients (VSCode extensions compiled to WASM, for example), **postcard** offers 3x faster serialization with a stable wire format specification and the `postcard-rpc` crate providing a turnkey RPC layer.

## Tokio provides complete transport layer coverage

The async runtime choice is unambiguous: **Tokio** has native support for all required transports and dominates the ecosystem. Key finding: async-std was **discontinued in March 2025**—avoid it for new projects.

**Transport abstraction pattern** using Tokio's traits:

```rust
use tokio::io::{AsyncRead, AsyncWrite};

// Generic handler working with any transport
async fn handle_stream<S>(stream: S) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let framed = Framed::new(stream, LengthDelimitedCodec::new());
    // Protocol handling identical regardless of transport
}

// All these types satisfy the bounds:
// - tokio::net::TcpStream
// - tokio::net::UnixStream
// - tokio::net::windows::named_pipe::NamedPipeServer
// - interprocess::local_socket::tokio::LocalSocketStream
```

**Cross-platform IPC** is best handled by the **interprocess** crate (5.6M downloads), which provides `LocalSocketListener`/`LocalSocketStream` types abstracting Unix sockets (Linux/macOS) and named pipes (Windows) behind a unified API. For Windows specifically, Tokio's built-in `tokio::net::windows::named_pipe` module handles the complex server lifecycle pattern where at least one pipe instance must always exist.

**Recommended transport implementations**:

| Platform | Local IPC | Network | Stdio |
|----------|-----------|---------|-------|
| Linux/macOS | Unix domain socket | TCP | stdin/stdout |
| Windows | Named pipe | TCP | stdin/stdout |
| Cross-platform | `interprocess` crate | `TcpListener` | `tokio::io::std{in,out}` |

## Message framing and RPC patterns from established projects

**tokio-util's `LengthDelimitedCodec`** handles framing with a 4-byte big-endian length prefix—simpler and more efficient than LSP's `Content-Length: N\r\n\r\n` header approach. Combined with **tokio-serde**, this enables typed message handling:

```rust
let framed = Framed::new(stream, LengthDelimitedCodec::new());
let typed = tokio_serde::Framed::new(framed, MessagePack::<Response, Request>::default());
typed.send(request).await?;
let response: Response = typed.next().await??;
```

**Session management** follows nREPL's proven model with these key patterns:

- **Correlation IDs**: Every request carries a UUID `id`; responses echo it back for multiplexing
- **Session isolation**: UUID-based sessions maintain per-client state (variables, history)
- **Multi-response evaluation**: Single `eval` request produces multiple response messages (stdout chunks, then final value, then `status: ["done"]`)
- **Streaming output**: Separate `out` and `err` fields in responses, sent incrementally during evaluation

**Recommended message structure** (nREPL-inspired, MessagePack-encoded):

```rust
#[derive(Serialize, Deserialize)]
struct Request {
    id: String,           // UUID correlation ID
    session: String,      // Session UUID
    op: String,           // "eval", "interrupt", "clone", "close", "describe"
    #[serde(default)]
    mode: ReplMode,       // "lisp" or "sexpr" for Oxur's dual-mode requirement
    #[serde(flatten)]
    params: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    id: String,
    session: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    out: Option<String>,   // Streaming stdout
    #[serde(skip_serializing_if = "Option::is_none")]
    err: Option<String>,   // Streaming stderr
    status: Vec<String>,   // ["done"], ["error"], ["interrupted"]
}
```

## Architecture layers follow rust-analyzer's proven separation

The **rust-analyzer** codebase demonstrates exceptional separation of concerns that maps directly to REPL requirements:

```
oxur-repl/
├── oxur-protocol/       # Message types, no I/O dependencies
│   ├── messages.rs      # Request, Response, Operation enums
│   ├── session.rs       # Session ID, state types
│   └── version.rs       # Protocol version negotiation
├── oxur-transport/      # Transport trait + implementations
│   ├── trait.rs         # TransportListener, Stream traits
│   ├── tcp.rs           # TcpTransportListener
│   ├── unix.rs          # UnixTransportListener (cfg(unix))
│   ├── pipe.rs          # NamedPipeListener (cfg(windows))
│   └── stdio.rs         # StdioTransport for embedded use
├── oxur-eval/           # Core evaluation engine (LLVM integration)
│   ├── context.rs       # EvalContext with bindings
│   ├── lisp_mode.rs     # Oxur syntax parser + eval
│   └── sexpr_mode.rs    # S-expression AST parser + eval
├── oxur-server/         # Server assembly + connection handling
│   ├── server.rs        # Multi-transport server
│   ├── handler.rs       # Message dispatch (Tower-style)
│   └── middleware.rs    # Logging, timeout, rate limiting
└── oxur-client/         # Reference client library
```

**Key architectural invariants** (borrowed from rust-analyzer):

1. **Protocol layer never does I/O**: All input flows through message types
2. **Core never knows about serialization**: Evaluation receives parsed ASTs, returns structured results
3. **Server is stateless per-request**: Session state lives in `SessionManager`, not connection handlers
4. **Cancellation via explicit operation**: `interrupt` op triggers `CancellationToken`, not TCP disconnect

**Plugin/extension architecture** for REPL commands uses static trait-based registration initially:

```rust
pub trait ReplCommand: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn execute(&self, ctx: &mut EvalContext, args: &[Value]) -> Result<Value>;
}

struct CommandRegistry {
    commands: HashMap<String, Box<dyn ReplCommand>>,
}
```

For future dynamic plugins (hot-reloadable extensions), the **abi_stable** crate provides stable Rust ABI with `#[sabi_trait]` for FFI-safe trait objects.

## Comparison to Go/Zylisp bencode approach

The Go nREPL (Zylisp) uses bencode for historical nREPL compatibility. Rust's approach differs meaningfully:

| Aspect | Go/Zylisp (bencode) | Rust/Oxur (MessagePack) |
|--------|---------------------|-------------------------|
| **Wire size** | Larger (text-encoded integers) | **~50% smaller** |
| **Type support** | Strings, ints, lists, dicts only | Full types including floats, booleans |
| **Performance** | Adequate | **3-5x faster** serialization |
| **Serde integration** | N/A | Native derive macros |
| **Client compatibility** | Clojure nREPL clients | New clients required, but trivial with msgpack libs |
| **Schema evolution** | None | Tolerates unknown fields |

**Trade-off**: Oxur won't have out-of-box compatibility with existing Clojure nREPL clients (CIDER, Calva, vim-fireplace). However, MessagePack's ubiquitous language support means writing new clients is straightforward—a Python client takes ~100 lines. The performance and type-system benefits outweigh compatibility with a different language's tooling.

For Zylisp/bencode compatibility mode (if desired), a **bridge layer** could translate MessagePack ↔ bencode at the transport edge, allowing legacy clients to connect while the core protocol uses efficient binary encoding.

## Recommended implementation path

**Phase 1: Protocol foundation**

- Define message types in `oxur-protocol` crate with serde derives
- Implement `LengthDelimitedCodec` + `rmp-serde` framing
- Core operations: `clone`, `eval`, `interrupt`, `close`, `describe`
- Add `mode` parameter for Lisp syntax vs s-expression switching

**Phase 2: Transport abstraction**

- `TransportListener` trait with `accept() -> impl AsyncRead + AsyncWrite`
- TCP implementation first (simplest for testing)
- Unix socket support for local IPC
- Windows named pipe support via `interprocess` crate

**Phase 3: Evaluation integration**

- `EvalContext` struct holding LLVM-compiled state
- Session manager mapping UUIDs to contexts
- Streaming output via `mpsc::channel` for stdout/stderr capture
- Interrupt support via `CancellationToken`

**Crate dependencies summary**:

```toml
[dependencies]
tokio = { version = "1.48", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
tokio-serde = { version = "0.9", features = ["messagepack"] }
rmp-serde = "1.3"
serde = { version = "1.0", features = ["derive"] }
interprocess = { version = "2.2", features = ["tokio"] }
uuid = { version = "1.0", features = ["v4"] }
thiserror = "1.0"
```

## Conclusion

Building Oxur's REPL protocol in Rust benefits from a mature async ecosystem that Go lacked when Zylisp was built. The combination of **MessagePack** for efficient cross-language serialization, **Tokio** for unified multi-transport support, and **rust-analyzer's layered architecture** pattern provides a robust foundation.

The dual-mode requirement (Lisp syntax + s-expression AST) is elegantly handled at the protocol level—a `mode` field in requests routes to different parser/evaluator paths while sharing session state, transport handling, and output streaming infrastructure. This design allows users to switch modes mid-session or run different clients in different modes against the same server.

No existing Rust crate provides an nREPL-style server—this positions Oxur to define the standard Rust REPL protocol implementation, potentially usable by future Rust REPL projects beyond Lisp languages.
