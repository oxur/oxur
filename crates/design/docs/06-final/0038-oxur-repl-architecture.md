---
number: 38
title: "Oxur REPL Architecture"
author: "Duncan McGreggor & Claude"
component: All
tags: [repl, architecture, definitive]
created: 2026-01-04
updated: 2026-01-10
state: Final
supersedes: null
superseded-by: null
version: 1.4
---


# Oxur REPL Architecture Overview

## Document Purpose

This document provides the complete architectural picture of the Oxur REPL system, showing how all components fit together across client, server, subprocess, and external crates. It serves as the single source of truth for understanding:

- How client and server interact via the protocol
- Where each component lives and what it does
- How compilation flows from user input to execution
- How subprocess execution enables interactive interruption (Ctrl-C)
- How the REPL integrates with the broader Oxur compilation chain
- What APIs are required from external crates
- How source mapping enables rustc-quality error messages

Note that extensive adjustments to the architecture were made after several highly detailed reviews of both the evcxr implementation as well as its development history.

**Target Audience:** Developers implementing or extending the REPL system

**Related Documents:**

- ODD-0013: Oxur Compilation Chain Architecture (compilation pipeline context)
- ODD-0018: Oxur Remote REPL Protocol Design (protocol layer specification)
- ODD-0030: Oxur REPL Implementation Specification (component implementation details)
- ODD-0026: Oxur REPL Evaluation Strategy (three-tier execution strategy)

**Research Foundation:**

This architecture is informed by comprehensive analysis of evcxr (Rust REPL/Jupyter kernel):

- Git archaeology: 6+ years of commit history (2018-2024)
- Web research: Documentation, issues, PRs, design rationale
- Pattern validation: What works, what doesn't, lessons learned
- See: `evcxr-research-synthesis.md` for detailed findings

---

## Table of Contents

1. [High-Level Architecture](#1-high-level-architecture)
2. [Component Inventory](#2-component-inventory)
3. [Compilation Pipeline](#3-compilation-pipeline)
4. [Data Flow: Complete Request Lifecycle](#4-data-flow-complete-request-lifecycle)
5. [Session Architecture](#5-session-architecture)
6. [Protocol Integration](#6-protocol-integration)
7. [Three-Tier Execution Strategy](#7-three-tier-execution-strategy)
8. [Integration Points with External Crates](#8-integration-points-with-external-crates)
9. [Critical Paths: Examples](#9-critical-paths-examples)
10. [File System Organization](#10-file-system-organization)
11. [Error Flow and Translation](#11-error-flow-and-translation)
12. [Deployment Topology](#12-deployment-topology)
13. [Performance Considerations](#13-performance-considerations)
14. [Conclusion](#14-conclusion)
15. [Version History](#15-version-history)

---

## 1. High-Level Architecture

### 1.1 System Components

```
┌──────────────────────────────────────────────────────────────┐
│                         CLIENT PROCESS                       │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                    ReplClient                          │  │
│  │  - Manages connection to server                        │  │
│  │  - Sends Request messages                              │  │
│  │  - Receives Response messages                          │  │
│  │  - Handles reconnection logic                          │  │
│  │  - NO compilation logic                                │  │
│  │  - NO evaluation logic                                 │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               │ TCP Socket + Binary Protocol
                               │ (Postcard serialization)
                               │ Length-prefixed framing
                               │
┌──────────────────────────────┴────────────────────────────────┐
│                         SERVER PROCESS                        │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                    ReplServer                           │  │
│  │  - Accepts TCP connections                              │  │
│  │  - Spawns handler per connection                        │  │
│  │  - Routes messages to MessageHandler                    │  │
│  │  - Manages graceful shutdown                            │  │
│  └─────────────────────────────────────────────────────────┘  │
│                               ↓                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                  MessageHandler                         │  │
│  │  - Dispatches operations (Eval, Clone, Close, etc.)     │  │
│  │  - Delegates to SessionManager                          │  │
│  │  - Constructs Response messages                         │  │
│  │  - Handles protocol errors                              │  │
│  └─────────────────────────────────────────────────────────┘  │
│                               ↓                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                  SessionManager                         │  │
│  │  - Creates and tracks sessions by ID                    │  │
│  │  - Maps SessionId → EvalContext                         │  │
│  │  - Enforces session limits and timeouts                 │  │
│  │  - Thread-safe session access (Arc<RwLock<...>>)        │  │
│  └─────────────────────────────────────────────────────────┘  │
│                               ↓                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │          EvalContext (one per session)                  │  │
│  │  ┌─────────────────────────────────────────────────┐    │  │
│  │  │ Session State:                                  │    │  │
│  │  │ - session_id: SessionId                         │    │  │
│  │  │ - mode: ReplMode (Lisp | Sexpr)                 │    │  │
│  │  │ - compiler: CachedCompiler                      │    │  │
│  │  │ - cache: Arc<ArtifactCache>         ◄─ NEW      │    │  │
│  │  │ - history: Vec<HistoryEntry>                    │    │  │
│  │  │ - output_buffer: OutputBuffer                   │    │  │
│  │  └─────────────────────────────────────────────────┘    │  │
│  │                                                         │  │
│  │  Core Methods:                                          │  │
│  │  - eval(code: &str) → Result<Value>                     │  │
│  │    * Parses code (via oxur-lang)                        │  │
│  │    * Decides execution tier                             │  │
│  │    * Checks cache before compiling      ◄─ NEW          │  │
│  │    * Delegates to CachedCompiler                        │  │
│  │  - load_file(path: &str) → Result<Value>                │  │
│  │  - set_mode(mode: ReplMode)                             │  │
│  └─────────────────────────────────────────────────────────┘  │
│                               ↓                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │          CachedCompiler (owned by EvalContext)          │  │
│  │                                                         │  │
│  │  Components:                                            │  │
│  │  - session_dir: SessionDir (temp filesystem)            │  │
│  │  - state: SessionState (variables, eval counter)        │  │
│  │  - executor: SubprocessExecutor    ◄─ MANDATORY         │  │
│  │  - rust_ast_wrapper: RustAstWrapper  ◄─ RENAMED         │  │
│  │  - source_map: Arc<SourceMap>      ◄─ from oxur-smap    │  │
│  │  - type_inference: TypeInference   ◄─ NEW               │  │
│  │                                                         │  │
│  │  Core Method:                                           │  │
│  │  - eval(form: CoreForm) → Result<Response>              │  │
│  │    * Generates Rust code from Core Forms                │  │
│  │    * Wraps with REPL scaffolding (RustAstWrapper)       │  │
│  │    * Invokes cargo to compile                           │  │
│  │    * Loads library into subprocess                      │  │
│  │    * Executes via stdin/stdout protocol ◄─ NEW          │  │
│  │    * Captures result and updates state                  │  │
│  └─────────────────────────────────────────────────────────┘  │
│                               ↓                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │          SubprocessExecutor                     ◄─ NEW  │  │
│  │                                                         │  │
│  │  - subprocess: Child (isolated process)                 │  │
│  │  - stdin: ChildStdin (commands)                         │  │
│  │  - stdout: BufReader<ChildStdout> (responses)           │  │
│  │  - protocol: SubprocessProtocol (text-based)            │  │
│  │                                                         │  │
│  │  Why Subprocess?                                        │  │
│  │  ✅ Enables Ctrl-C interruption (threads can't be       │  │
│  │     interrupted in Rust)                                │  │
│  │  ✅ Crash isolation (user panic doesn't kill REPL)      │  │
│  │  ✅ Clean restart on error                              │  │
│  │  ✅ Memory isolation                                    │  │
│  │                                                         │  │
│  │  Protocol: Simple text over stdin/stdout                │  │
│  │  - Command: "LOAD_AND_RUN <path> <fn_name>\n"           │  │
│  │  - Success: "OXUR_EXECUTION_COMPLETE\n"                 │  │
│  │  - Error: "OXUR_RUNTIME_ERROR: <msg>\n"                 │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
└───────────────────────────────────────────────────────────────┘
                               ↓
┌───────────────────────────────────────────────────────────────┐
│                       SUBPROCESS (Isolated)                   │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                     Runtime                             │  │
│  │  - Listens on stdin for commands                        │  │
│  │  - Loads dynamic libraries (.so/.dylib/.dll)            │  │
│  │  - Maintains VariableStore                              │  │
│  │  - Executes user code in isolation                      │  │
│  │  - Returns results via stdout                           │  │
│  │  - Can be killed (Ctrl-C) without affecting REPL        │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              VariableStore                              │  │
│  │  - vars: HashMap<String, Box<dyn Any + 'static>>        │  │
│  │                                                         │  │
│  │  Constraint: All values must be owned ('static)         │  │
│  │  - No inter-variable references possible                │  │
│  │  - Aligns with Lisp immutable semantics                 │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 1.2 External Crates (Compilation Pipeline)

```
┌─────────────────────────────────────────────────────────────┐
│                   FOUNDATION CRATE                          │
├─────────────────────────────────────────────────────────────┤
│  oxur-smap (no dependencies) ◄─ NEW, PHASE 0 PREREQUISITE   │
│    - NodeId: Unique identifier for AST nodes                │
│    - SourcePos: Original source position (file, line, col)  │
│    - SourceMap: Multi-stage transformation tracking         │
│      * surface_positions: NodeId → SourcePos                │
│      * surface_to_core: NodeId → NodeId                     │
│      * core_to_rust: NodeId → NodeId                        │
│    - content_hash(): For cache key generation               │
│                                                             │
│  Why This Matters:                                          │
│  - Enables rustc errors → original Oxur source positions    │
│  - NO other Lisp has multi-stage source tracking            │
│  - Unique differentiating feature                           │
└─────────────────────────────────────────────────────────────┘
                             ↓
┌─────────────────────────────────────────────────────────────┐
│                   COMPILATION CRATES                        │
├─────────────────────────────────────────────────────────────┤
│  oxur-lang (parsing & expansion)                            │
│    - parse_lisp(source: &str, source_map: &mut SourceMap)   │
│      → Result<SurfaceForms>                                 │
│    - expand(surface: SurfaceForms, source_map: &mut         │
│      SourceMap) → Result<CoreForms>                         │
│    - Records Surface→Core transformations in source_map     │
├─────────────────────────────────────────────────────────────┤
│  oxur-comp (lowering to Rust)                               │
│    - lower(core: &CoreForm, source_map: &mut SourceMap)     │
│      → Result<RustAst>                                      │
│    - Records Core→Oxur AST→syn transformations in source_map│
├─────────────────────────────────────────────────────────────┤
│  oxur-ast (Rust AST manipulation)                           │
│    - print_rust(ast: &syn::File) → String                   │
│    - Uses NodeId from oxur-smap for source comments         │
└─────────────────────────────────────────────────────────────┘
                             ↓
                      REPL coordinates
                      all stages on server
```

### 1.3 The Critical Constraint: Why Subprocess is Mandatory

**Finding from evcxr research:**

> "Don't ask Jupyter to 'interrupt kernel', it won't work. Rust threads can't be interrupted."

**The Problem:**

- Rust threads cannot be forcibly stopped (by design, for safety)
- `thread::spawn()` cannot be killed once started
- Infinite loops in user code would hang REPL forever
- No safe way to implement Ctrl-C with in-process execution

**The Solution:**

- Subprocess can be killed via `SIGKILL` signal
- User presses Ctrl-C → REPL kills subprocess → spawns new one
- Session state preserved in server (variables, history)
- Seamless recovery from hangs, crashes, infinite loops

**evcxr Validation:**

- Subprocess present from day one (commit 2018-09-25)
- Zero fundamental changes in 6+ years
- Proven architecture for interactive Rust execution

**For Oxur:**

- Subprocess execution is **mandatory**, not optional
- `Executor` trait kept for testability (mocking in tests)
- Production code always uses `SubprocessExecutor`

### 1.4 Dependency Flow

```
Client (thin, protocol only)
  ↓ TCP socket
Server (all compilation here)
  ↓ owns
SessionManager
  ↓ creates
EvalContext
  ↓ owns
CachedCompiler
  ↓ owns
SubprocessExecutor ◄─ MANDATORY
  ↓ manages
Subprocess (isolated execution)

External crates used by server:
  oxur-smap ← foundation (all depend on this)
  oxur-lang ← parsing & expansion
  oxur-comp ← lowering to Rust
  oxur-ast  ← AST manipulation
```

**Key Principle:** ALL compilation happens on the server. The client is a thin protocol endpoint with zero compilation logic.

### 1.5 CLI Integration

The REPL is accessed through the unified `oxur` CLI tool, specifically via `oxur repl`. When `oxur` is invoked without a subcommand, it defaults to `oxur repl`.

#### Binary Location

**Crate:** `oxur-cli`
**Binary:** `./bin/oxur` (via `cargo build`)

#### Command-Line Interface

```
oxur repl [FLAGS]

FLAGS:
  -i, --interactive              Start the default built-in REPL server and connect
                                 to it with the built-in client. This is the default
                                 behavior when no flags are specified.

  -c, --connect [<HOST:PORT>]    Connect to a running REPL server with the built-in
                                 client. Default: 127.0.0.1:5099

  --no-color                     Disable ANSI colors in interactive or connect modes.
                                 No effect otherwise.

  -s, --serve <PATH|HOST:PORT>   Start a REPL server only (no client). If PATH is
                                 given, binds to a Unix domain socket. If HOST:PORT
                                 is given, binds to TCP/IP.

  --ack <ACK-PORT>               Acknowledge the port of this server to another
                                 nREPL server running on ACK-PORT. Used for tooling
                                 integration (editor plugins, etc.).

  -t, --transport <TRANSPORT>    The transport module to use.
                                 Default: oxur_repl::transport::tcp

  -h, --help                     Print help information
```

#### Default Behavior (`-i`/`--interactive`)

When `oxur repl` is invoked without flags (or with `-i`):

```
┌─────────────────────────────────────────────────────────────────┐
│                      oxur repl (default mode)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  Terminal Interface                      │    │
│  │  - rustyline/reedline for line editing                  │    │
│  │  - Command history (persistent across sessions)         │    │
│  │  - Ctrl-C handling (interrupts evaluation)              │    │
│  │  - Ctrl-D handling (exits REPL)                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                           │                                     │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │               InProcessTransport                         │    │
│  │  - Zero-copy message passing via channels               │    │
│  │  - No serialization overhead                            │    │
│  │  - Fastest possible client-server communication         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                     │                │                          │
│                     ▼                ▼                          │
│  ┌──────────────────────┐  ┌───────────────────────┐           │
│  │     ReplClient       │  │      ReplServer       │           │
│  │  (in-process)        │◄─►│  (in-process)         │           │
│  └──────────────────────┘  └───────────────────────┘           │
│                                      │                          │
│                                      ▼                          │
│                           ┌───────────────────┐                │
│                           │  SubprocessExecutor│                │
│                           │  (isolated process)│                │
│                           └───────────────────┘                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

1. **Terminal Interface** spawns and manages user interaction
2. **InProcessTransport** provides zero-overhead communication between client and server
3. **ReplClient** and **ReplServer** run in the same process, communicating via channels
4. **SubprocessExecutor** runs in a separate process for isolation and Ctrl-C support

#### Server Mode (`-s`/`--serve`)

```
┌─────────────────────────────────────────────────────────────────┐
│               oxur repl --serve 127.0.0.1:5099                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    ReplServer                            │    │
│  │  - Listens on TCP socket or Unix domain socket          │    │
│  │  - Accepts multiple client connections                   │    │
│  │  - Manages sessions per client                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                           │                                     │
│              ┌────────────┴────────────┐                       │
│              ▼                         ▼                       │
│  ┌──────────────────────┐  ┌───────────────────────┐           │
│  │  Session 1           │  │  Session 2            │           │
│  │  (EvalContext +      │  │  (EvalContext +       │           │
│  │   Subprocess)        │  │   Subprocess)         │           │
│  └──────────────────────┘  └───────────────────────┘           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

Standalone server for remote connections or tooling integration (e.g., editor plugins).

#### Connect Mode (`-c`/`--connect`)

```
oxur repl --connect 127.0.0.1:5099
```

Connects to an existing REPL server. Useful for:

- Connecting to a remote Oxur REPL
- Connecting from an editor plugin
- Multiple terminals sharing one REPL session

#### ACK Protocol (`--ack`)

The `--ack` flag implements nREPL-style acknowledgment:

```
oxur repl --serve 0.0.0.0:0 --ack 5099
```

1. Server binds to an ephemeral port (`:0`)
2. Server connects to ACK-PORT and sends its actual bound port
3. Tooling on ACK-PORT receives the port and can connect

This enables editors to launch REPL servers and discover their ports programmatically.

---

## 2. Component Inventory

This section catalogs all components in the REPL system, organized by location and responsibility.

### 2.1 Foundation Crate (NEW)

#### oxur-smap

**Location:** `oxur-smap/` (separate crate, no dependencies)

**Purpose:** Multi-stage source mapping for error translation

**Ownership:** Foundation crate - all other crates depend on this

**Core Types:**

```rust
// oxur-smap/src/lib.rs

/// Unique identifier for AST nodes across all compilation stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    pub fn new() -> Self {
        // Global atomic counter for uniqueness
    }
}

/// Source position in original Oxur code
#[derive(Debug, Clone)]
pub struct SourcePos {
    pub file: String,      // Source file path
    pub line: u32,         // 1-indexed line number
    pub column: u32,       // 1-indexed column number
    pub length: u32,       // Span length for highlighting
}

/// Tracks AST transformations for error reporting
pub struct SourceMap {
    // Surface Form positions (from parsing)
    surface_positions: HashMap<NodeId, SourcePos>,

    // Transformation chains
    surface_to_core: HashMap<NodeId, NodeId>,  // Expansion
    core_to_rust: HashMap<NodeId, NodeId>,     // Lowering
}

impl SourceMap {
    pub fn new() -> Self;

    // Called by oxur-lang during parsing
    pub fn record_surface_node(&mut self, node: NodeId, pos: SourcePos);

    // Called by oxur-lang during expansion
    pub fn record_expansion(&mut self, surface: NodeId, core: NodeId);

    // Called by oxur-comp during lowering (crosses semantic boundary via Oxur AST)
    pub fn record_lowering(&mut self, core: NodeId, rust: NodeId);

    // Called by oxur-repl during error translation
    pub fn lookup(&self, rust_node: NodeId) -> Option<SourcePos> {
        // Traverse backwards: Rust → Core → Surface → SourcePos
        let core_node = self.core_to_rust.iter()
            .find(|(_, &r)| r == rust_node)?
            .0;
        let surface_node = self.surface_to_core.iter()
            .find(|(_, &c)| c == *core_node)?
            .0;
        self.surface_positions.get(surface_node).cloned()
    }

    // For cache key generation
    pub fn content_hash(&self) -> String {
        // SHA256 hash of mapping structure
    }
}
```

**Why This Exists:**

- Tracks transformations across entire compilation pipeline
- Enables rustc errors to be translated back to original Oxur source
- NO other Lisp implementation has this (unique differentiator)
- Foundation for rustc-quality error messages in a Lisp

**Dependencies:** None (foundation crate)

**Depended on by:** oxur-lang, oxur-comp, oxur-ast, oxur-repl

---

### 2.2 External Crates (Compilation Pipeline)

**Note:** These crates are outside the REPL codebase but are required dependencies.

#### oxur-lang

**Location:** Separate crate

**Purpose:** Parsing Oxur source code and macro expansion

**REPL Integration Points:**

```rust
// Required API (must exist before REPL implementation)
pub fn parse_lisp(
    source: &str,
    source_map: &mut SourceMap
) -> Result<SurfaceForms>;

pub fn expand(
    surface: SurfaceForms,
    source_map: &mut SourceMap
) -> Result<CoreForms>;

pub fn parse_core_forms(
    source: &str,
    source_map: &mut SourceMap
) -> Result<CoreForms>;
```

**Responsibility:** Records Surface→Core transformations in SourceMap

#### oxur-comp

**Location:** Separate crate

**Purpose:** Lowering Core Forms to Rust AST via Oxur AST intermediate layer

**Note:** Per ODD-0013, this internally crosses the semantic boundary from Lisp concepts to Rust concepts via Oxur AST (S-expressions of Rust concepts), then converts to syn structures. Current implementation combines these steps.

**REPL Integration Points:**

```rust
// Required API
pub fn lower(
    core: &CoreForm,
    source_map: &mut SourceMap
) -> Result<syn::File>;
```

**Responsibility:** Records Core→Oxur AST→syn transformations in SourceMap

#### oxur-ast

**Location:** Separate crate

**Purpose:** Rust AST manipulation and code generation

**REPL Integration Points:**

```rust
// Required API
pub fn print_rust(ast: &syn::File) -> String;
```

**Responsibility:** Uses NodeId from oxur-smap for source map comments

---

### 2.3 oxur-repl Components

#### ReplClient

**Location:** `oxur-repl/src/client.rs`

**Ownership:** Client process

**Purpose:** Protocol endpoint for connecting to REPL server

**Responsibilities:**

- Manages TCP connection to server
- Sends `Request` messages (serialized with postcard)
- Receives `Response` messages
- Handles reconnection on disconnect
- **NO compilation logic** (thin client)
- **NO evaluation logic** (all on server)

**Dependencies:** ODD-0018 protocol types

#### ReplServer

**Location:** `oxur-repl/src/server.rs`

**Ownership:** Server process

**Purpose:** Accept client connections and route messages

**Responsibilities:**

- Binds to TCP port and accepts connections
- Spawns one `MessageHandler` per connection
- Manages graceful shutdown
- Handles server-level errors

**Concurrency:** One thread per client connection

#### MessageHandler

**Location:** `oxur-repl/src/handler.rs`

**Ownership:** Server (one per connection)

**Purpose:** Protocol message dispatch

**Responsibilities:**

- Deserializes `Request` messages from client
- Dispatches operations (Eval, Clone, Close, LoadFile, etc.)
- Delegates to `SessionManager`
- Serializes `Response` messages back to client
- Handles protocol-level errors

**Thread Safety:** Each handler has unique connection, no shared state

#### SessionManager

**Location:** `oxur-repl/src/session/manager.rs`

**Ownership:** Server (shared across all handlers)

**Purpose:** Multi-session coordination

**State:**

```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Mutex<EvalContext>>>>>,
    config: SessionConfig,
}

pub struct SessionConfig {
    max_sessions: usize,           // Limit concurrent sessions
    session_timeout: Duration,     // Idle timeout
    cleanup_interval: Duration,    // Periodic cleanup
}
```

**Responsibilities:**

- Creates new sessions with unique IDs
- Maps `SessionId` → `Arc<Mutex<EvalContext>>`
- Enforces session limits (max concurrent sessions)
- Implements session timeouts (cleanup idle sessions)
- Thread-safe access (`Arc<RwLock<...>>`)

**Concurrency Strategy:**

- Read lock: Lookup existing session
- Write lock: Create new session or remove session
- Per-session mutex: One eval at a time per session

#### EvalContext

**Location:** `oxur-repl/src/eval/context.rs`

**Ownership:** Server (one per session, owned by SessionManager)

**Purpose:** Per-session evaluation state and coordination

**State:**

```rust
pub struct EvalContext {
    session_id: SessionId,
    mode: ReplMode,                    // Lisp | Sexpr
    compiler: CachedCompiler,
    cache: Arc<ArtifactCache>,         // ◄─ NEW
    history: Vec<HistoryEntry>,
    output_buffer: OutputBuffer,
}
```

**Responsibilities:**

- Parses user input (delegates to oxur-lang)
- Decides execution tier (Calculator | Cached | JIT)
- Checks cache before compiling (NEW)
- Delegates compilation to `CachedCompiler`
- Manages session history
- Buffers output for client

**Key Methods:**

```rust
impl EvalContext {
    pub fn eval(&mut self, code: &str) -> Result<Value> {
        // 1. Parse (oxur-lang::parse_lisp)
        // 2. Expand (oxur-lang::expand)
        // 3. Decide tier (Calculator/Cached/JIT)
        // 4. Check cache (NEW)
        // 5. If cache miss, compile via CachedCompiler
        // 6. Execute and return result
    }

    pub fn load_file(&mut self, path: &str) -> Result<Value>;
    pub fn set_mode(&mut self, mode: ReplMode);
}
```

**Thread Safety:** Wrapped in `Arc<Mutex<...>>` by SessionManager (one eval at a time)

#### CachedCompiler

**Location:** `oxur-repl/src/compiler/cached.rs`

**Ownership:** Server (owned by EvalContext, one per session)

**Purpose:** Compile Core Forms to dynamic libraries

**State:**

```rust
pub struct CachedCompiler {
    session_dir: SessionDir,
    state: SessionState,
    executor: SubprocessExecutor,      // ◄─ MANDATORY (was Option)
    rust_ast_wrapper: RustAstWrapper,  // ◄─ RENAMED
    source_map: Arc<SourceMap>,        // ◄─ from oxur-smap
    type_inference: TypeInference,     // ◄─ NEW
}
```

**Responsibilities:**

- Lowers Core Forms to Rust AST (via oxur-comp, crossing semantic boundary through Oxur AST)
- Wraps AST with REPL scaffolding (RustAstWrapper)
- Invokes cargo to compile to dylib
- Parses cargo output for errors
- Translates errors using SourceMap
- Loads library into subprocess
- Executes code via SubprocessExecutor
- Updates SessionState on success

**Key Method:**

```rust
impl CachedCompiler {
    pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
        // 1. Lower: CoreForm → Oxur AST → syn AST (via oxur-comp)
        // 2. Wrap: Add REPL scaffolding (RustAstWrapper)
        // 3. Generate: syn AST → String (via oxur-ast)
        // 4. Write: Save to session_dir
        // 5. Compile: Invoke cargo build
        // 6. Parse errors: rustc JSON → SourceMap lookup
        // 7. Load: Into subprocess
        // 8. Execute: Via stdin/stdout protocol
        // 9. Update state: On success
        // 10. Return: Response with result/error
    }
}
```

**Thread Safety:** Owned by EvalContext (protected by its mutex)

#### RustAstWrapper (RENAMED from CodeGenerator)

**Location:** `oxur-repl/src/wrapper.rs` (was `src/codegen/generator.rs`)

**Ownership:** Server (owned by CachedCompiler)

**Purpose:** Wrap pure Rust AST with REPL-specific scaffolding

**CRITICAL:** This component does NOT do lowering. Lowering happens in oxur-comp. RustAstWrapper only wraps already-lowered Rust AST.

**Responsibilities:**

- Takes pure Rust AST from oxur-comp::lower()
- Adds VariableStore integration code
- Adds `extern "C"` wrapper function for dynamic loading
- Adds source map comments (NodeId annotations)
- Generates variable load/store code
- Emits complete library AST ready for compilation

**Example Output:**

```rust
// Generated by RustAstWrapper

use std::any::Any;
use std::collections::HashMap;

// User's lowered code (from oxur-comp)
fn user_code_5() -> i32 {
    /* oxur_node=300 */ x + y
}

// REPL scaffolding (added by RustAstWrapper)
#[no_mangle]
pub extern "C" fn run_user_code_5(
    vars: &mut HashMap<String, Box<dyn Any + 'static>>
) -> Box<dyn Any + 'static> {
    // Load variables from store
    let x: i32 = *vars.get("x").unwrap().downcast_ref().unwrap();
    let y: i32 = *vars.get("y").unwrap().downcast_ref().unwrap();

    // Execute user code
    let result = user_code_5();

    // Store result
    vars.insert("_".to_string(), Box::new(result));

    Box::new(result)
}
```

**Why the Rename:**

- "CodeGenerator" implied it does code generation/lowering
- Actually just wraps already-generated Rust AST
- "RustAstWrapper" is crystal clear about responsibility

#### SubprocessExecutor (NEW - MANDATORY)

**Location:** `oxur-repl/src/executor/subprocess.rs`

**Ownership:** Server (owned by CachedCompiler)

**Purpose:** Execute user code in isolated subprocess

**Why Mandatory (from evcxr research):**

1. **Ctrl-C support:** Rust threads cannot be interrupted; subprocess can be killed
2. **Crash isolation:** User panic doesn't kill REPL
3. **Memory isolation:** Separate address space
4. **Clean restart:** Spawn new subprocess on error

**State:**

```rust
pub struct SubprocessExecutor {
    subprocess: Child,                    // Isolated process
    stdin: ChildStdin,                    // Command channel
    stdout: BufReader<ChildStdout>,       // Response channel
    protocol: SubprocessProtocol,         // Text-based
}
```

**Protocol (stdin/stdout text):**

```
REPL → Subprocess (via stdin):
  "LOAD_AND_RUN /path/to/libeval_005.so run_user_code_5\n"

Subprocess → REPL (via stdout):
  On success: "OXUR_EXECUTION_COMPLETE\n"
  On error:   "OXUR_RUNTIME_ERROR: panic message\n"
```

**Why stdin/stdout (not Unix sockets):**

- evcxr used this for 6+ years without issues
- Simple, portable, proven
- Text protocol is sufficient
- Unix sockets deferred to v1.1+ if needed

**Methods:**

```rust
impl SubprocessExecutor {
    pub fn new() -> Result<Self> {
        // Spawn subprocess: oxur-repl-subprocess binary
        // Connect stdin/stdout
    }

    pub fn execute(&mut self, lib_path: &Path, fn_name: &str)
        -> Result<Response> {
        // Send LOAD_AND_RUN command
        // Wait for response
        // Parse result
        // Handle errors
    }

    pub fn restart(&mut self) -> Result<()> {
        // Kill old subprocess
        // Spawn new one
        // Variable state preserved in server
    }
}
```

**Executor Trait (for Testing):**

**Location:** `oxur-repl/src/executor/mod.rs`

```rust
pub trait Executor {
    fn execute(&mut self, lib_path: &Path, fn_name: &str)
        -> Result<Response>;
}

impl Executor for SubprocessExecutor { ... }

// Test-only implementation
#[cfg(test)]
impl Executor for MockExecutor { ... }
```

**Note:** `Executor` trait exists for testability ONLY. Production code always uses `SubprocessExecutor`.

#### ArtifactCache (NEW - MANDATORY)

**Location:** `oxur-repl/src/cache.rs`

**Ownership:** Server (shared via `Arc<Mutex<ArtifactCache>>`)

**Purpose:** Content-based caching of compiled artifacts

**Why Mandatory (from evcxr research):**

- evcxr waited 5 years to add caching (biggest regret)
- When added (2023), it was "major performance improvement"
- Cache is CRITICAL for REPL responsiveness

**State:**

```rust
pub struct ArtifactCache {
    cache_dir: PathBuf,                    // ~/.cache/oxur/artifacts/
    index: HashMap<String, CachedArtifact>,
}

pub struct CachedArtifact {
    path: PathBuf,        // Path to compiled .so/.dylib/.dll
    created: SystemTime,  // For LRU eviction
}
```

**Thread Safety:**

The `ArtifactCache` is wrapped in `Arc<Mutex<...>>` when shared across sessions:

```rust
type SharedCache = Arc<Mutex<ArtifactCache>>;
```

**Cache Key Generation:**

```rust
impl ArtifactCache {
    pub fn cache_key(
        source: &str,
        deps: &[Dependency],
        opt_level: OptLevel,
        source_map: &SourceMap,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        for dep in deps {
            hasher.update(dep.to_string().as_bytes());
        }
        hasher.update(&[opt_level as u8]);
        hasher.update(source_map.content_hash().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn get(&self, key: &str) -> Option<PathBuf>;
    pub fn insert(&mut self, key: String, artifact: PathBuf);
    pub fn evict_lru(&mut self, keep_n: usize);  // LRU policy
}
```

**Cache Location (platform-appropriate):**

- Linux: `~/.cache/oxur/artifacts/`
- macOS: `~/Library/Caches/oxur/artifacts/`
- Windows: `%LOCALAPPDATA%\oxur\cache\artifacts\`

**Performance Impact:**

- Cache hit: 1-5ms (library load only)
- Cache miss: 50-300ms (full compilation)
- Critical for iterative REPL development

#### TypeInference (NEW)

**Location:** `oxur-repl/src/type_inference.rs`

**Ownership:** Server (owned by CachedCompiler)

**Purpose:** Infer types of variables using rust-analyzer

**Why Needed:**

- Variables are type-erased in VariableStore
- Need to know types to generate correct code
- evcxr tried compiler error hack (4 years, removed 2022)
- rust-analyzer provides clean solution

**Integration:**

```rust
use rust_analyzer::Analysis;

pub struct TypeInference {
    analysis: Analysis,
}

impl TypeInference {
    pub fn infer_type(&self, code: &str, var_name: &str)
        -> Result<String> {
        // Use rust-analyzer to infer type
        self.analysis.infer_type_at_position(code, var_name)
    }
}
```

**Fallback Strategy:**

```rust
fn handle_inference_failure(var_name: &str) -> Error {
    Error::TypeInferenceFailure {
        var: var_name.to_string(),
        message: format!(
            "Cannot determine type of variable `{}`. \
             Please add an explicit type annotation.",
            var_name
        )
    }
}
```

**Implementation Choice:**

- v1.0: Use rust-analyzer as library (simpler)
- v1.1: Consider LSP if build times problematic (per evcxr maintainer suggestion)

#### SessionDir

**Location:** `oxur-repl/src/session/dir.rs`

**Ownership:** Server (owned by CachedCompiler)

**Purpose:** Manage temporary filesystem for compilation

**Responsibilities:**

- Creates session-specific temp directory
- Provides paths for source files, Cargo.toml, artifacts
- Cleans up on session end
- Implements best-effort tmpfs optimization

**Temp Directory Selection (Decision 4):**

```rust
fn get_repl_temp_root() -> PathBuf {
    // 1. User override
    if let Ok(custom) = env::var("OXUR_REPL_TEMP_DIR") {
        return PathBuf::from(custom);
    }

    // 2. Platform-specific RAM-backed storage
    #[cfg(target_os = "linux")]
    {
        let shm = PathBuf::from("/dev/shm");
        if shm.exists() && shm.is_dir() {
            return shm.join("oxur-repl");
        }
    }

    #[cfg(target_os = "macos")]
    {
        let ramdisk = PathBuf::from("/Volumes/OxurREPL");
        if ramdisk.exists() && ramdisk.is_dir() {
            return ramdisk;
        }
    }

    // 3. Fallback to system temp
    env::temp_dir().join("oxur-repl")
}

impl SessionDir {
    pub fn new(session_id: &SessionId) -> Result<Self> {
        let root = get_repl_temp_root()
            .join(format!("session-{}", session_id));
        fs::create_dir_all(&root)?;

        // Create structure:
        // session-{id}/
        //   Cargo.toml
        //   src/
        //     lib.rs
        //   target/
        //     debug/
        //       libeval_NNN.so
    }
}
```

**Performance:**

- Linux: Automatic tmpfs (~2-3% faster)
- macOS/Windows: OS caching (good enough)
- User can override via `OXUR_REPL_TEMP_DIR`

#### SessionState

**Location:** `oxur-repl/src/session/state.rs`

**Ownership:** Server (owned by CachedCompiler)

**Purpose:** Track session-level state

**State:**

```rust
pub struct SessionState {
    eval_counter: u32,        // Increments on each eval
    variables: HashSet<String>, // Track variable names
    last_value: Option<Value>,  // Result of last eval
}
```

**Clone-Try-Commit Pattern:**

```rust
impl CachedCompiler {
    async fn eval(&mut self, form: CoreForm) -> Result<Response> {
        // Clone state before attempting compilation
        let saved_state = self.state.clone();

        // Try compilation and execution
        match self.try_eval(&form).await {
            Ok(response) => {
                // Success: Keep new state
                Ok(response)
            }
            Err(e) => {
                // Failure: Restore old state
                self.state = saved_state;
                Err(e)
            }
        }
    }
}
```

**Why This Pattern:**

- Prevents state corruption on errors
- Ensures clean recovery from failed evals
- User can retry without side effects

---

### 2.4 Subprocess Components

These components run in the isolated subprocess, which is a **binary target within the oxur-repl crate**.

#### Subprocess Binary

**Location:** `oxur-repl/src/bin/subprocess.rs`

**Built as:** `oxur-repl-subprocess` binary

**Cargo.toml configuration:**

```toml
# oxur-repl/Cargo.toml
[[bin]]
name = "oxur-repl-subprocess"
path = "src/bin/subprocess.rs"
```

**Spawned by:** `SubprocessExecutor::new()` in the server process

#### Runtime

**Location:** `oxur-repl/src/bin/subprocess.rs` (main function and Runtime struct)

**Ownership:** Subprocess

**Purpose:** Listen for commands and execute user code

**Protocol Implementation:**

```rust
fn main() {
    let mut runtime = Runtime::new();
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let cmd = line.unwrap();

        if cmd.starts_with("LOAD_AND_RUN") {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            let lib_path = parts[1];
            let fn_name = parts[2];

            match runtime.load_and_execute(lib_path, fn_name) {
                Ok(_) => println!("OXUR_EXECUTION_COMPLETE"),
                Err(e) => println!("OXUR_RUNTIME_ERROR: {}", e),
            }
        }

        stdout.flush().unwrap();
    }
}
```

**Responsibilities:**

- Read commands from stdin
- Load dynamic libraries via libloading
- Execute functions
- Maintain VariableStore
- Write responses to stdout

#### VariableStore

**Location:** `oxur-repl/src/subprocess/variable_store.rs` (shared module used by subprocess binary)

**Ownership:** Subprocess (owned by Runtime)

**Purpose:** Type-erased variable persistence

**Implementation:**

```rust
pub struct VariableStore {
    vars: HashMap<String, Box<dyn Any + 'static>>,
}

impl VariableStore {
    pub fn get<T: 'static>(&self, name: &str) -> Option<&T> {
        self.vars.get(name)?
            .downcast_ref::<T>()
    }

    pub fn set(&mut self, name: String, value: Box<dyn Any + 'static>) {
        self.vars.insert(name, value);
    }
}
```

**The 'static Constraint (Decision 7):**

Variables must be owned (cannot hold references to other variables):

```rust
// This CANNOT work:
let all_values = vec![10, 20, 30];
let some_values = &all_values[2..3];  // ERROR: not 'static

// Must clone instead:
let some_values = all_values[2..3].to_vec();
```

**Why This is OK:**

- Aligns with Lisp semantics (typically immutable data)
- Prevents lifetime complexity
- Proven pattern (6+ years in evcxr)

#### VariableStore in Generated Code

**Note:** The VariableStore type definition is also embedded in generated code (see Section 10.3) to ensure ABI compatibility between the subprocess runtime and dynamically loaded libraries. Both the subprocess and generated libraries must agree on the VariableStore memory layout.

---

### 2.5 Component Dependency Summary

```
Foundation:
  oxur-smap (no dependencies)

External Crates (depend on oxur-smap):
  oxur-lang
  oxur-comp
  oxur-ast

Server Components:
  ReplServer
    └─ MessageHandler
         └─ SessionManager
              └─ EvalContext
                   ├─ ArtifactCache (shared via Arc<Mutex<...>>)
                   └─ CachedCompiler
                        ├─ SessionDir
                        ├─ SessionState
                        ├─ SubprocessExecutor (mandatory)
                        ├─ RustAstWrapper
                        ├─ TypeInference
                        └─ SourceMap (from oxur-smap)

Subprocess Components (binary target in oxur-repl):
  Runtime (oxur-repl/src/bin/subprocess.rs)
    └─ VariableStore

Client Component:
  ReplClient (thin, protocol only)
```

---

## 3. Compilation Pipeline

### 3.1 Stage-by-Stage Breakdown

```
User Input: "(+ 1 2)"
  │
  ↓
┌─────────────────────────────────────────────────────────────┐
│ STAGE 0: Cache Check (in EvalContext)               ◄─ NEW  │
│ Owner: Server (EvalContext)                                 │
│                                                             │
│ Before parsing, check if we've compiled this before:        │
│                                                             │
│   let cache_key = compute_cache_key(code);                  │
│   if let Some(artifact) = cache.get(&cache_key) {           │
│       return execute_cached(artifact);  // 1-5ms!           │
│   }                                                         │
│                                                             │
│ Cache hit: Skip compilation entirely (Tier 2)               │
│ Cache miss: Continue to parsing (Tier 3)                    │
└─────────────────────────────────────────────────────────────┘
  │
  ↓ (Cache miss - must compile)
┌─────────────────────────────────────────────────────────────┐
│ STAGE 1: Parse (in EvalContext)                             │
│ Owner: Server (EvalContext)                                 │
│ Crate: oxur-lang                                            │
│                                                             │
│ Input:  Raw text string                                     │
│ Output: Surface Forms (if Lisp mode)                        │
│         OR Core Forms (if Sexpr mode)                       │
│                                                             │
│ Implementation:                                             │
│   let mut source_map = SourceMap::new();  ◄─ NEW            │
│   match self.mode {                                         │
│       ReplMode::Lisp => {                                   │
│           oxur_lang::parse_lisp(code, &mut source_map)      │
│       }                                                     │
│       ReplMode::Sexpr => {                                  │
│           oxur_lang::parse_core_forms(code, &mut source_map)│
│       }                                                     │
│   }                                                         │
│                                                             │
│ SourceMap Recording:                                        │
│   - Records NodeId → SourcePos for each parsed node         │
│   - Enables error translation later                         │
└─────────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 2: Expand (only in Lisp mode)                     │
│ Owner: Server (EvalContext)                             │
│ Crate: oxur-lang                                        │
│                                                         │
│ Input:  Surface Forms                                   │
│ Output: Core Forms                                      │
│                                                         │
│ Implementation:                                         │
│   oxur_lang::expand(surface_forms, &mut source_map)     │
│                                                         │
│ SourceMap Recording:                           ◄─ NEW   │
│   - Records Surface NodeId → Core NodeId                │
│   - Tracks macro expansion transformations              │
│                                                         │
│ Transformations:                                        │
│   defn → define-func                                    │
│   when → if-expr                                        │
│   -> → nested function calls                            │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 3: Tier Decision (in EvalContext)                 │
│ Owner: Server (EvalContext)                             │
│                                                         │
│ Decision Logic:                                         │
│   if is_simple_arithmetic(&core_forms) {                │
│       Tier::Calculator  // <1ms                         │
│   } else {                                              │
│       // Cache already checked in Stage 0               │
│       Tier::Jit         // 50-300ms (compile + exec)    │
│   }                                                     │
│                                                         │
│ Note: Tier::Cached path taken in Stage 0 (cache hit)    │
│                                                         │
│ Tier 1 → eval_calculator() (pure Rust evaluation)       │
│ Tier 3 → compiler.eval() (generate & compile)           │
└─────────────────────────────────────────────────────────┘
  │
  ↓ (Tier 3 path - must compile)
┌─────────────────────────────────────────────────────────┐
│ STAGE 4: Lower (in RustAstWrapper)         ◄─ RENAMED   │
│ Owner: Server (CachedCompiler → RustAstWrapper)         │
│ Crate: oxur-comp                                        │
│                                                         │
│ Input:  Core Forms                                      │
│ Output: Rust AST (syn crate structures)                 │
│                                                         │
│ Implementation:                                         │
│   let rust_ast = oxur_comp::lower(                      │
│       &core_forms,                                      │
│       &mut source_map    ◄─ NEW                         │
│   )?;                                                   │
│                                                         │
│ Note (per ODD-0013):                          ◄─ NEW    │
│   This internally involves two operations:              │
│   1. Core Forms → Oxur AST (semantic boundary)          │
│      Crosses from Lisp to Rust concepts                 │
│   2. Oxur AST → syn AST (de-S-expressioning)            │
│      Converts S-expressions to syn structures           │
│   Current implementation combines these steps.          │
│                                                         │
│ SourceMap Recording:                           ◄─ NEW   │
│   - Records Core NodeId → Rust NodeId                   │
│   - Critical for error translation                      │
│                                                         │
│ Examples:                                               │
│   (define-func add [x y] (+ x y))                       │
│   → (Item :kind (Fn ...)) [Oxur AST]                    │
│   → syn::ItemFn { ... }   [syn AST]                     │
│                                                         │
│   (if-expr condition then-branch else-branch)           │
│   → (Expr :kind (If ...)) [Oxur AST]                    │
│   → syn::ExprIf { ... }   [syn AST]                     │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 5: Wrap (in RustAstWrapper)          ◄─ RENAMED   │
│ Owner: Server (RustAstWrapper)                          │
│                                                         │
│ Takes pure Rust AST (from oxur-comp) and wraps with:    │
│   - VariableStore integration                           │
│   - Function signature (extern "C")                     │
│   - Variable load/store code                            │
│   - Source map comments (NodeId annotations)   ◄─ NEW   │
│                                                         │
│ CRITICAL: This stage does NOT do lowering!              │
│ Lowering happens in Stage 4 (oxur-comp).                │
│ RustAstWrapper only adds REPL scaffolding.              │
│                                                         │
│ Output: Complete Rust AST for library                   │
│                                                         │
│ Example:                                                │
│   // User's lowered code (from oxur-comp)               │
│   fn user_code_5() -> i32 {                             │
│       /* oxur_node=300 */ x + y                         │
│   }                                                     │
│                                                         │
│   // REPL scaffolding (added by RustAstWrapper)         │
│   #[no_mangle]                                          │
│   pub extern "C" fn run_user_code_5(                    │
│       vars: &mut HashMap<String, Box<dyn Any>>          │
│   ) -> Box<dyn Any> {                                   │
│       let x: i32 = /* load from vars */;                │
│       let y: i32 = /* load from vars */;                │
│       let result = user_code_5();                       │
│       Box::new(result)                                  │
│   }                                                     │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 6: Generate (in RustAstWrapper)      ◄─ RENAMED   │
│ Owner: Server (RustAstWrapper)                          │
│ Crate: oxur-ast                                         │
│                                                         │
│ Input:  Wrapped Rust AST                                │
│ Output: Rust source code (String)                       │
│                                                         │
│ Implementation:                                         │
│   let source = oxur_ast::print_rust(&wrapped_ast);      │
│                                                         │
│ Result: Formatted, compilable Rust source               │
│                                                         │
│ SourceMap integration:                         ◄─ NEW   │
│   - NodeId comments embedded in source                  │
│   - Used later for error translation                    │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 7: Write Files (in CachedCompiler)                │
│ Owner: Server (CachedCompiler)                          │
│                                                         │
│ Writes to session directory:                            │
│   Linux:   /dev/shm/oxur-repl/session-abc/src/lib.rs    │
│   macOS:   /var/folders/.../oxur-repl/session-abc/...   │
│   Windows: C:\Users\...\Temp\oxur-repl\session-abc\...  │
│                                                         │
│ Note: tmpfs optimization on Linux (~2-3% faster)        │
│                                                         │
│ Cargo.toml already exists (created by SessionDir)       │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 8: Compile (in CachedCompiler)                    │
│ Owner: Server (CachedCompiler)                          │
│ Tool: cargo                                             │
│                                                         │
│ Command:                                                │
│   cargo build \                                         │
│     --target x86_64-unknown-linux-gnu \                 │
│     --message-format=json \                             │
│     --release                      ◄─ Configurable      │
│                                                         │
│ Environment:                                            │
│   CARGO_TARGET_DIR=/tmp/oxur-repl/session-abc/target    │
│   RUSTFLAGS="-C link-arg=-fuse-ld=mold"  (optional)     │
│                                                         │
│ Optimization level: Configurable                        │
│   - Debug (default): -C opt-level=0 (faster compile)    │
│   - Release: -C opt-level=3 (faster runtime)            │
│                                                         │
│ Incremental compilation: Enabled (3-5x speedup)         │
│                                                         │
│ Performance: 50-300ms typical                           │
│   (See Section 13 for detailed breakdown)               │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 9: Parse Cargo Output (in CachedCompiler)         │
│ Owner: Server (CachedCompiler)                          │
│                                                         │
│ Parses JSON messages from cargo (--message-format=json) │
│                                                         │
│ On compilation errors:                                  │
│   1. Extract rustc error JSON                           │
│   2. Parse Rust source position (file, line, column)    │
│   3. Extract NodeId from source comments                │
│   4. Lookup original position via SourceMap:   ◄─ NEW   │
│      rust_node → core_node → surface_node → SourcePos   │
│   5. Render error with ariadne (beautiful formatting)   │
│   6. Return error to user                               │
│                                                         │
│ On success:                                             │
│   - Extract artifact path from cargo JSON               │
│   - Proceed to Stage 10                                 │
│                                                         │
│ Output: Path to compiled dylib                          │
│   e.g., target/debug/libeval_005.so                     │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 10: Rename Artifact (in CachedCompiler)           │
│ Owner: Server (CachedCompiler)                          │
│                                                         │
│ Rename (or copy on Windows):                            │
│   libctx.so → libeval_005.so                            │
│                                                         │
│ Unique name per evaluation (prevents caching issues)    │
│                                                         │
│ Naming: libeval_{eval_counter}.{so|dylib|dll}           │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 11: Cache Artifact (in CachedCompiler)   ◄─ NEW   │
│ Owner: Server (CachedCompiler)                          │
│                                                         │
│ Store compiled artifact in persistent cache:            │
│                                                         │
│   let cache_key = ArtifactCache::cache_key(             │
│       &source,                                          │
│       &deps,                                            │
│       opt_level,                                        │
│       &source_map                                       │
│   );                                                    │
│                                                         │
│   cache.insert(cache_key, artifact_path.clone());       │
│                                                         │
│ Cache location: ~/.cache/oxur/artifacts/{hash}.so       │
│                                                         │
│ Future evals: Stage 0 will find this (1-5ms speedup!)   │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 12: Load & Execute (in SubprocessExecutor) ◄─ UPD │
│ Owner: Server (CachedCompiler → SubprocessExecutor)     │
│ Executor: Subprocess (MANDATORY)                ◄─ NEW  │
│                                                         │
│ 1. Send command via stdin:                     ◄─ NEW   │
│    "LOAD_AND_RUN /path/to/libeval_005.so \              │
│     run_user_code_5\n"                                  │
│                                                         │
│ 2. Subprocess (isolated process):                       │
│    - Reads command from stdin                           │
│    - Loads library via libloading::Library::new()       │
│    - Gets function: lib.get(b"run_user_code_5")         │
│    - Calls: fn(&mut VariableStore) -> Box<dyn Any>      │
│    - Function executes, mutates VariableStore           │
│    - Captures stdout/stderr (separate from protocol)    │
│    - Returns result                                     │
│                                                         │
│ 3. Subprocess sends response via stdout:       ◄─ NEW   │
│    On success: "OXUR_EXECUTION_COMPLETE\n"              │
│    On error:   "OXUR_RUNTIME_ERROR: <msg>\n"            │
│                                                         │
│ 4. SubprocessExecutor parses response:                  │
│    - Reads from subprocess stdout                       │
│    - Parses protocol message                            │
│    - Constructs Response with result/error              │
│    - Returns to CachedCompiler                          │
│                                                         │
│ Why Subprocess? (from evcxr research)          ◄─ NEW   │
│   ✅ Enables Ctrl-C (Rust threads can't be interrupted) │
│   ✅ Crash isolation (panic doesn't kill REPL)          │
│   ✅ Clean restart on error                             │
│   ✅ Memory isolation (separate address space)          │
│                                                         │
│ IPC Overhead: ~100-200μs (negligible vs 50-300ms        │
│ compilation)                                            │
└─────────────────────────────────────────────────────────┘
  │
  ↓
Result: Value (to return to user)
```

### 3.2 Ownership Summary

| Stage | Owner Component | Owner Location | External Crate |
|-------|----------------|----------------|----------------|
| Cache Check | EvalContext | Server | - |
| Parse | EvalContext | Server | oxur-lang |
| Expand | EvalContext | Server | oxur-lang |
| Tier Decision | EvalContext | Server | - |
| Lower | RustAstWrapper | Server | oxur-comp |
| Wrap | RustAstWrapper | Server | - |
| Generate | RustAstWrapper | Server | oxur-ast |
| Write | CachedCompiler | Server | - |
| Compile | CachedCompiler | Server | cargo |
| Parse Output | CachedCompiler | Server | - |
| Rename | CachedCompiler | Server | - |
| Cache Artifact | CachedCompiler | Server | - |
| Execute | SubprocessExecutor | Server → Subprocess | libloading |

**Key Insights:**

- ALL stages happen on the server. Client just sends/receives protocol messages.
- SourceMap threading: Parse → Expand → Lower (via Oxur AST) - all record transformations
- Cache check happens BEFORE parsing (Stage 0 - fastest path)
- Cache store happens AFTER compilation (Stage 11 - benefits future evals)
- Subprocess execution is MANDATORY (not optional) for Ctrl-C support

### 3.3 SourceMap Integration Across Pipeline

**Thread Pattern:**

```rust
// EvalContext creates and owns the SourceMap
impl EvalContext {
    pub fn eval(&mut self, code: &str) -> Result<Value> {
        // Create source map for this evaluation
        let mut source_map = SourceMap::new();

        // Stage 1: Parse records surface positions
        let surface = oxur_lang::parse_lisp(code, &mut source_map)?;

        // Stage 2: Expand records surface→core transformations
        let core = oxur_lang::expand(surface, &mut source_map)?;

        // Pass to compiler with source map
        self.compiler.eval(core, source_map).await
    }
}

// CachedCompiler continues threading
impl CachedCompiler {
    pub async fn eval(
        &mut self,
        core: CoreForm,
        source_map: SourceMap
    ) -> Result<Response> {
        // Stage 4: Lower records core→oxur ast→syn transformations
        let rust_ast = oxur_comp::lower(&core, &mut source_map)?;

        // Stage 5-6: RustAstWrapper uses source_map
        let source = self.rust_ast_wrapper.generate(rust_ast, &source_map)?;

        // Stage 9: Error translation uses source_map
        if let Err(cargo_error) = self.compile(&source).await {
            return Err(self.translate_error(cargo_error, &source_map));
        }

        // ...
    }
}
```

**Transformation Chain:**

```
User writes:     test.ox:5:15  "(+ x y)"
                                   ↑ error here
    ↓ parse
Surface Node:    NodeId(100)    SourcePos(test.ox, 5, 15, 3)
    ↓ expand
Core Node:       NodeId(200)    <- Surface NodeId(100)
    ↓ lower
Rust Node:       NodeId(300)    <- Core NodeId(200)
    ↓ compile
rustc error:     lib.rs:42       "cannot find value `y`"
    ↓ translate (via SourceMap)
Original error:  test.ox:5:15    "cannot find value `y` in this scope"
```

**Lookup Algorithm:**

```rust
impl SourceMap {
    pub fn lookup(&self, rust_node: NodeId) -> Option<SourcePos> {
        // Traverse backwards through transformation chain

        // Step 1: Rust → Core
        let core_node = self.core_to_rust.iter()
            .find(|(_, &r)| r == rust_node)?
            .0;

        // Step 2: Core → Surface
        let surface_node = self.surface_to_core.iter()
            .find(|(_, &c)| c == *core_node)?
            .0;

        // Step 3: Surface → Original Position
        self.surface_positions.get(surface_node).cloned()
    }
}
```

### 3.4 Caching Impact on Pipeline

**Without Cache (Cold):**

```
Stages 0-12:  50-300ms total
  - Stage 0: Cache miss (1ms)
  - Stages 1-10: Compilation (48-298ms)
  - Stage 11: Cache store (1ms)
  - Stage 12: Execute (1-5ms)
```

**With Cache (Warm):**

```
Stage 0:      Cache hit (1-5ms total)
  - Load cached artifact
  - Skip stages 1-11 entirely!
  - Jump directly to Stage 12 (execute)

50-200x faster!
```

**Speedup by Type:**

| Evaluation Type | Cold (miss) | Warm (hit) | Speedup |
|----------------|-------------|------------|---------|
| Simple function | 50ms | 1ms | 50x |
| Complex logic | 150ms | 2ms | 75x |
| Large module | 300ms | 5ms | 60x |

**Key Insight:** Cache transforms perceived performance from "compiling" (50-300ms) to "interpreting" (1-5ms).

### 3.5 Performance Breakdown by Stage

**Typical timings (cold compilation):**

```
Stage 0:  Cache check           ~1ms    (0.3%)
Stage 1:  Parse                 ~1ms    (0.3%)
Stage 2:  Expand                ~2ms    (0.7%)
Stage 3:  Tier decision         <1ms    (0.1%)
Stage 4:  Lower                 ~5ms    (1.7%)
Stage 5:  Wrap                  ~2ms    (0.7%)
Stage 6:  Generate              ~1ms    (0.3%)
Stage 7:  Write files           <1ms    (0.03%)
Stage 8:  Compile (cargo)       ~280ms  (93%)   ← Dominates!
Stage 9:  Parse output          ~1ms    (0.3%)
Stage 10: Rename artifact       <1ms    (0.03%)
Stage 11: Cache store           ~1ms    (0.3%)
Stage 12: Execute               ~5ms    (1.7%)
─────────────────────────────────────────────
Total:                          ~300ms
```

**The 90% Rule:** Stage 8 (cargo compilation) dominates total time.

**Optimization Priorities:**

1. **Cache hits** (skip Stage 8 entirely) - 50-200x speedup
2. **Incremental compilation** (faster Stage 8) - 3-5x speedup
3. **tmpfs** (faster Stage 7 writes) - 2-3% speedup
4. Everything else: <1% impact

**See Section 13** for detailed performance analysis and optimization strategies.

---

## 4. Data Flow: Complete Request Lifecycle

### 4.1 Happy Path: Simple Eval

```
Step 1: User Input
───────────────────
User types: (+ 1 2)
Client: ReplClient.eval("(+ 1 2)")

Step 2: Client → Server
───────────────────────
Client serializes:
  Request {
    id: "msg-123",
    session: "session-abc",
    op: Operation::Eval,
    mode: ReplMode::Lisp,
    params: { "code": "(+ 1 2)" }
  }

Postcard serialization → Bytes
Length-prefix framing → [len][data]
TCP send

Step 3: Server Receives
─────────────────────────
ReplServer accepts connection
Spawns handler task
Handler deserializes Request
Routes to MessageHandler

Step 4: MessageHandler Dispatch
──────────────────────────────
MessageHandler.handle_request(req)
Matches on Operation::Eval
Calls handle_eval(req)

Step 5: Session Lookup
───────────────────────
MessageHandler → SessionManager.get_session("session-abc")
SessionManager finds EvalContext for session
Returns Arc<Mutex<EvalContext>>

Step 6: Evaluation
──────────────────
Lock EvalContext
Call eval_context.eval("(+ 1 2)")

Step 7: Parse (in EvalContext)
────────────────────────────
Create SourceMap for this evaluation           ◄─ NEW
Mode is Lisp, so:
  let mut source_map = SourceMap::new();        ◄─ NEW
  surface_forms = oxur_lang::parse_lisp(
      "(+ 1 2)",
      &mut source_map                           ◄─ NEW
  )?
  core_forms = oxur_lang::expand(
      surface_forms,
      &mut source_map                           ◄─ NEW
  )?

Result: CoreForm::FunctionCall {
  function: "+",
  args: [CoreForm::Literal(1), CoreForm::Literal(2)],
  node_id: NodeId(42),                          ◄─ NEW
}

SourceMap now contains Surface→Core mappings   ◄─ NEW

Step 8: Tier Decision (in EvalContext)
────────────────────────────────────
Check if simple arithmetic: YES
Decision: Tier::Calculator
(Note: Cache check skipped for Calculator tier)

Step 9: Calculator Evaluation
──────────────────────────────
eval_context.eval_calculator(&core_forms)
Direct Rust evaluation (no compilation)
Result: Value::Int(3)
Time: <1ms

Step 10: Construct Response
─────────────────────────────
MessageHandler creates Response:
  Response {
    id: "msg-123",
    session: "session-abc",
    value: Some(Value::Int(3)),
    out: "",
    err: "",
    status: [Status::Done],
    error: None,
    data: {}
  }

Step 11: Server → Client
─────────────────────────
Serialize Response via Postcard
Frame with length prefix
TCP send

Step 12: Client Displays
──────────────────────────
Client receives and deserializes
Returns Value::Int(3) to caller
User sees: 3
```

### 4.2 Complex Path: Compilation Required

```
Step 1: User Input
───────────────────
User types: (defn square [x] (* x x))
Client: ReplClient.eval("(defn square [x] (* x x))")

Step 2-6: Same as simple path
─────────────────────────────
Request sent, routed to EvalContext

Step 7: Cache Check (NEW - Stage 0)              ◄─ NEW
──────────────────────────────────
Before parsing, check cache:
  let cache_key = compute_cache_key(code);
  if let Some(artifact) = cache.get(&cache_key) {
      // Cache hit! Skip compilation entirely
      return execute_cached(artifact);  // 1-5ms
  }

Cache miss → Continue to parsing

Step 8: Parse & Expand                           ◄─ RENUMBERED
────────────────────────
let mut source_map = SourceMap::new();           ◄─ NEW
surface_forms = oxur_lang::parse_lisp(
    "(defn square [x] (* x x))",
    &mut source_map                              ◄─ NEW
)
core_forms = oxur_lang::expand(
    surface_forms,
    &mut source_map                              ◄─ NEW
)

Result: CoreForm::FunctionDefinition {
  name: "square",
  params: [("x", Type::Infer)],
  body: CoreForm::FunctionCall { ... },
  node_id: NodeId(100),                          ◄─ NEW
}

Step 9: Tier Decision                            ◄─ RENUMBERED
──────────────────────
Not simple arithmetic
(Cache already checked in Step 7)
Decision: Tier::Jit (must compile)

Step 10: Compilation (in CachedCompiler)         ◄─ RENUMBERED
───────────────────────────────────────
compiler.eval(core_forms, source_map)           ◄─ NEW parameter

  Step 10a: Type Inference (NEW)                 ◄─ NEW
  ──────────────────────────
  For each variable without explicit type:
    type_inference.infer_type(code, var_name)

  Uses rust-analyzer to determine types

  Step 10b: Code Generation                      ◄─ RENUMBERED
  ─────────────────────────
  rust_ast_wrapper.generate(&core_forms, &state, &source_map)
                   ^^^^^^^ RENAMED from code_gen
    - oxur_comp::lower(core_forms, &mut source_map)  ◄─ NEW
    - wrap_in_function(ast, state)
    - oxur_ast::print_rust(wrapped_ast) → String

  Result: Rust source code for lib.rs
  (with /* oxur_node=X */ comments)              ◄─ NEW

  Step 10c: Write Files                          ◄─ RENUMBERED
  ────────────────────
  Write to session directory (tmpfs on Linux):
    Linux:   /dev/shm/oxur-repl/session-abc/src/lib.rs
    macOS:   /var/folders/.../session-abc/src/lib.rs
    Windows: %TEMP%\oxur-repl\session-abc\src\lib.rs

  Step 10d: Invoke Cargo                         ◄─ RENUMBERED
  ──────────────────────
  cargo build --message-format=json
  Time: 200-300ms (cold), 50-100ms (warm with incremental)

  Step 10e: Parse Output                         ◄─ RENUMBERED
  ──────────────────────
  Read JSON messages from cargo stdout
  Check for errors
  If errors: translate via SourceMap (multi-stage lookup)
  Extract artifact path

  Step 10f: Rename Artifact                      ◄─ RENUMBERED
  ─────────────────────────
  Rename libctx.so → libeval_006.so

  Step 10g: Cache Artifact (NEW)                 ◄─ NEW
  ──────────────────────────
  Store in persistent cache:
    cache.insert(cache_key, artifact_path.clone());

  Location: ~/.cache/oxur/artifacts/{hash}.so

  Future evaluations: Skip 10a-10f entirely!

Step 11: Execution (in SubprocessExecutor)       ◄─ RENUMBERED + UPDATED
───────────────────────────────────────
SubprocessExecutor sends via stdin:              ◄─ NEW protocol
  "LOAD_AND_RUN /tmp/.../libeval_006.so run_user_code_6\n"

Subprocess (isolated process):
  - Reads command from stdin
  - Loads library via libloading
  - Calls run_user_code_6(&mut variable_store)
  - Function defines square function in VariableStore
  - Returns result

Subprocess responds via stdout:                  ◄─ NEW protocol
  "OXUR_EXECUTION_COMPLETE\n"

Why subprocess? (not in-process)                 ◄─ NEW
  - Rust threads cannot be interrupted
  - Ctrl-C support requires killable process
  - Crash isolation (user panic doesn't kill REPL)
  - Proven architecture (6+ years in evcxr)

Step 12: Result Capture                          ◄─ RENUMBERED
─────────────────────────
SubprocessExecutor parses response
Captures any stdout/stderr (separate from protocol)
Returns result to CachedCompiler
CachedCompiler returns to EvalContext

Step 13-15: Same as simple path                  ◄─ RENUMBERED
─────────────────────────────────
Response constructed and sent to client
User sees: function defined (or similar feedback)
```

### 4.3 Cached Path: Cache Hit (NEW)             ◄─ ENTIRELY NEW SECTION

```
Step 1: User Input
───────────────────
User types: (square 5)  ; Function already compiled before
Client: ReplClient.eval("(square 5)")

Step 2-6: Same as other paths
─────────────────────────────
Request sent, routed to EvalContext

Step 7: Cache Check (Stage 0)
──────────────────────────────
Before parsing, compute cache key:
  let cache_key = compute_cache_key(code);

Cache key computation:
  SHA256(code + deps + opt_level + source_map_structure)

Check cache:
  if let Some(artifact_path) = cache.get(&cache_key) {
      // ✅ CACHE HIT!
      // Skip stages 1-11 of compilation pipeline!

      // Jump directly to execution
      return executor.execute(artifact_path, fn_name);
  }

✅ Cache hit! Artifact already compiled.

Step 8: Execute Cached Artifact
────────────────────────────────
SubprocessExecutor.execute(cached_artifact_path, fn_name)

Send via stdin:
  "LOAD_AND_RUN ~/.cache/oxur/artifacts/{hash}.so run_user_code_14\n"

Subprocess:
  - Loads cached library
  - Executes function
  - Returns result

Subprocess responds:
  "OXUR_EXECUTION_COMPLETE\n"

Time: 1-5ms (vs 50-300ms for compilation)
Speedup: 50-200x faster!

Step 9-11: Return result
─────────────────────────
Construct Response, send to client
User sees: result (instantly!)

Impact:
  Cold (cache miss): 50-300ms compilation + 1-5ms execution
  Warm (cache hit):  1-5ms execution only

Cache makes REPL feel like interpreter while being compiler!
```

### 4.4 Error Path: Compilation Error

```
Step 1-10b: Same as compilation path
────────────────────────────────────
Generate code, write files, invoke cargo

Step 10c: Cargo Returns Errors                   ◄─ RENUMBERED
───────────────────────────────
Cargo JSON output contains:
  {
    "reason": "compiler-message",
    "message": {
      "message": "cannot find value `y` in this scope",
      "code": { "code": "E0425" },
      "level": "error",
      "spans": [{
        "file_name": ".../src/lib.rs",
        "line_start": 42,
        "line_end": 42,
        "column_start": 15,
        "column_end": 16,
        ...
      }]
    }
  }

Step 11: Error Translation (Multi-Stage Lookup)  ◄─ UPDATED + RENUMBERED
────────────────────────────────────────────
CachedCompiler.translate_error()

  a. Extract span from rustc error
     Position: lib.rs:42:15

  b. Read line 42 from lib.rs
     "    /* oxur_node=301 */ x + /* oxur_node=302 */ y"

  c. Extract NodeId at error column (15)         ◄─ NEW
     Nearest comment: /* oxur_node=302 */
     NodeId: 302 (Rust node)

  d. Multi-stage SourceMap lookup:               ◄─ NEW
     source_map.lookup(NodeId(302))

     Internally:
       Rust NodeId(302) → Core NodeId(202)
       Core NodeId(202) → Surface NodeId(102)
       Surface NodeId(102) → SourcePos {
         file: "test.ox",
         line: 5,
         column: 15,
         length: 1
       }

  e. Construct beautiful error with ariadne      ◄─ NEW

  Result: OxurError {
    message: "cannot find value `y` in this scope",
    file: "test.ox",
    line: 5,
    column: 15,
    code: "E0425",
    level: "error"
  }

Step 12: Error Response                          ◄─ RENUMBERED
────────────────────────
MessageHandler creates Response:
  Response {
    id: "msg-123",
    session: "session-abc",
    value: None,
    out: "",
    err: "",
    status: [Status::Error],
    error: Some(ErrorInfo {
      kind: ErrorKind::Lower,
      message: "cannot find value `y` in this scope",
      source_location: Some(SourceLocation {
        file: "test.ox",
        line: 5,
        column: 15
      }),
      stack_trace: []
    }),
    data: {}
  }

Step 13: Client Displays Error                   ◄─ RENUMBERED
────────────────────────────────
Client receives error response
Formats with ariadne (if available):

Error[E0425]: cannot find value `y` in this scope
  ┌─ test.ox:5:15
  │
5 │   (+ x y))
  │        ^ cannot find value in this scope
  │
```

### 4.5 Error Path: Runtime Error (Subprocess)    ◄─ NEW SECTION

```
Step 1-11: Same as compilation path
────────────────────────────────
Code compiles successfully, begins execution

Step 12: Runtime Panic in Subprocess
─────────────────────────────────────
User code panics during execution:
  panic!("Index out of bounds!")

Subprocess catches panic
Sends error via stdout protocol:
  "OXUR_RUNTIME_ERROR: Index out of bounds!\n"

Optional stack trace:
  "OXUR_PANIC_LOCATION: lib.rs:47:9\n"

Step 13: SubprocessExecutor Handles Error
──────────────────────────────────────────
Reads "OXUR_RUNTIME_ERROR" from stdout
Parses error message
Returns Error::Runtime to CachedCompiler

Note: Runtime errors don't have source positions
(they occur after compilation, SourceMap can't help)

Step 14: Error Response
────────────────────────
Response {
  id: "msg-123",
  session: "session-abc",
  value: None,
  status: [Status::Error],
  error: Some(ErrorInfo {
    kind: ErrorKind::Eval,
    message: "Index out of bounds!",
    source_location: None,  // Runtime error, no position
    stack_trace: []         // Could include if available
  })
}

Step 15: Subprocess Restart (Optional)
───────────────────────────────────────
If panic was severe:
  SubprocessExecutor.restart()
  - Kill old subprocess
  - Spawn new one
  - Variable state preserved in server
  - Transparent to user
```

### 4.6 Data Flow Summary

**Component Interactions:**

```
ReplClient (thin)
  ↓ TCP + Protocol
MessageHandler
  ↓ Dispatch
SessionManager
  ↓ Lookup
EvalContext
  ├─ Cache Check ───────→ ArtifactCache (NEW)
  ├─ Parse ─────────────→ oxur-lang + SourceMap (NEW)
  ├─ Expand ────────────→ oxur-lang + SourceMap (NEW)
  └─ Compile ───────────→ CachedCompiler
      ├─ Type Infer ───→ TypeInference (NEW)
      ├─ Lower ────────→ oxur-comp + SourceMap (NEW)
      ├─ Wrap ─────────→ RustAstWrapper (RENAMED)
      ├─ Generate ─────→ oxur-ast
      ├─ Compile ──────→ cargo
      ├─ Translate ────→ SourceMap (NEW)
      ├─ Cache ────────→ ArtifactCache (NEW)
      └─ Execute ──────→ SubprocessExecutor (MANDATORY)
          └─────────────→ Subprocess (isolated)
```

**Performance by Path:**

| Path | Latency | Components Involved |
|------|---------|-------------------|
| Calculator | <1ms | Client, Handler, EvalContext only |
| Cached | 1-5ms | + ArtifactCache + SubprocessExecutor |
| JIT (cold) | 50-300ms | + Full compilation pipeline |
| JIT (warm) | 50-100ms | + Incremental compilation |

**Key Observations:**

1. **Cache is critical** - Transforms 50-300ms → 1-5ms (50-200x!)
2. **SourceMap threads through** - Parse → Expand → Lower (via Oxur AST) → Error translation
3. **Subprocess is mandatory** - Not optional (Ctrl-C requires it)
4. **All work server-side** - Client is truly thin (protocol only)

---

## 5. Session Architecture

### 5.1 Session Lifecycle

```
CREATE SESSION
──────────────
Client → Server: Request { op: Clone }
Server:
  1. SessionManager.create_session()
  2. Generate unique SessionId (UUID)
  3. Create new EvalContext
  4. EvalContext creates CachedCompiler
  5. CachedCompiler creates SessionDir
  6. SessionDir creates temp directory (tmpfs on Linux)    ◄─ NEW
  7. SessionDir writes Cargo.toml
  8. CachedCompiler spawns SubprocessExecutor             ◄─ NEW
  9. SubprocessExecutor spawns subprocess (isolated)      ◄─ NEW
  10. Initialize shared ArtifactCache (Arc)               ◄─ NEW
  11. Store in sessions map
Server → Client: Response { session: "session-abc", status: [SessionCreated] }

Subprocess now running, waiting for commands via stdin   ◄─ NEW

ACTIVE SESSION
──────────────
Client → Server: Request { op: Eval, session: "session-abc", ... }
Server:
  1. SessionManager.get_session("session-abc")
  2. Lock EvalContext
  3. Check cache (Stage 0)                                ◄─ NEW
  4. If cache miss: Compile and cache artifact            ◄─ NEW
  5. Execute via SubprocessExecutor (stdin/stdout)        ◄─ NEW
  6. Unlock EvalContext
Server → Client: Response { ... }

(Can have multiple concurrent requests to different sessions)

CLOSE SESSION
─────────────
Client → Server: Request { op: Close, session: "session-abc" }
Server:
  1. SessionManager.remove_session("session-abc")
  2. Drop EvalContext
  3. CachedCompiler drop → kills subprocess               ◄─ UPDATED
     (Send SIGKILL to subprocess Child)                   ◄─ NEW
  4. SessionDir drop → cleans temp directory
  5. Cached artifacts remain in ~/.cache/oxur/            ◄─ NEW
     (persists across sessions for reuse)
Server → Client: Response { status: [SessionClosed] }
```

### 5.2 Session Isolation

Each session has completely isolated:

**Filesystem:**

- Session temp: `/dev/shm/oxur-repl/session-{uuid}/` (Linux tmpfs)  ◄─ UPDATED
- Session temp: `/var/folders/.../oxur-repl/session-{uuid}/` (macOS)
- Session temp: `%TEMP%\oxur-repl\session-{uuid}\` (Windows)
- Independent Cargo projects
- Shared cache: `~/.cache/oxur/artifacts/` (cross-session)  ◄─ NEW

**Process:**

- Separate subprocess per session (MANDATORY)              ◄─ UPDATED
- Subprocess crash doesn't affect other sessions
- Subprocess can be killed (Ctrl-C support)                ◄─ NEW
- Independent VariableStore per subprocess
- Communication via stdin/stdout text protocol             ◄─ NEW

**State:**

- Independent evaluation history
- Independent variable bindings
- Independent REPL mode (Lisp vs Sexpr)
- Independent SourceMap per evaluation                     ◄─ NEW

**Concurrency:**

- Sessions can evaluate in parallel
- EvalContext is Mutex-protected (one eval at a time per session)
- SessionManager is Arc<RwLock<...>> (concurrent session access)
- ArtifactCache is Arc<Mutex<...>> (shared, thread-safe)   ◄─ NEW

### 5.3 Cache Sharing Across Sessions                      ◄─ NEW SECTION

**Shared Global Cache:**

```rust
// Created once on server startup
let artifact_cache = Arc::new(Mutex::new(ArtifactCache::new()));

// Shared with all sessions
impl SessionManager {
    pub fn create_session(&mut self) -> SessionId {
        let eval_context = EvalContext::new(
            session_id,
            self.artifact_cache.clone(),  // ◄─ Shared Arc
        );
        // ...
    }
}
```

**Benefits:**

1. **Cross-session reuse** - Session A compiles code, Session B gets cache hit
2. **Persistent across restarts** - Cache survives server restart
3. **Faster warm-up** - New sessions benefit from previous work

**Cache Location:**

- Linux: `~/.cache/oxur/artifacts/`
- macOS: `~/Library/Caches/oxur/artifacts/`
- Windows: `%LOCALAPPDATA%\oxur\cache\artifacts\`

**Cache Lifetime:**

- Lives longer than any individual session
- Cleaned up by LRU policy (not by session close)
- Default: Keep 1GB or 1000 artifacts (whichever reached first)

### 5.4 Resource Management

**Session Limits:**

- Max sessions per server: configurable (default: 100)
- Enforcement: SessionManager refuses creation if limit reached
- Error: `ErrorKind::Session` with message "session limit reached"

**Session Timeouts:**

- Idle timeout: configurable (default: 30min)
- SessionManager background task checks last activity
- Auto-close idle sessions
- Client gets error on next request to closed session
- Subprocess killed on timeout                             ◄─ NEW

**Disk Usage:**

Per session (temp directory):

- Cargo project: ~5MB
- Incremental cache: ~30-100MB
- Artifacts: ~10-50MB (varies by code size)
- Total per session: ~50-150MB

Global cache (shared):                                     ◄─ NEW

- Default max: 1GB total
- ~1000 cached artifacts typical
- LRU eviction when limit reached
- Configurable via OXUR_CACHE_SIZE_MB

Total for 100 sessions:

- Session temps: 100 × 100MB = ~10GB
- Shared cache: ~1GB (independent of session count)
- Total: ~11GB typical

Cleanup:

- Session temp deleted on close
- Global cache persists (LRU managed)
- Stale session dirs cleanup on server startup (>24h old)

**Memory Usage:**

Per session (server process):

- EvalContext: ~5MB
- CachedCompiler: ~10MB
- Subprocess: ~10-20MB (separate process)             ◄─ UPDATED
- Buffers, history: ~5MB
- Total per session: ~30-40MB server + 10-20MB subprocess

Global cache (shared):                                ◄─ NEW

- Index in memory: ~1-5MB (1000 entries)
- Negligible compared to session memory

Total for 100 sessions:

- Server process: 100 × 35MB = ~3.5GB
- Subprocesses: 100 × 15MB = ~1.5GB
- Total: ~5GB

**CPU:**

- Compilation is CPU-intensive (30-300ms)
- One compilation per session at a time (Mutex on EvalContext)
- Multiple sessions can compile in parallel
- Tier 1 (calculator) is minimal CPU
- Cache hits have negligible CPU (<1ms)                ◄─ NEW

### 5.5 Subprocess Management                           ◄─ NEW SECTION

**Subprocess Lifecycle:**

```
Session Created
    ↓
Spawn SubprocessExecutor
    ↓
Spawn subprocess (oxur-repl-subprocess binary)
    ↓
Subprocess binds stdin/stdout
    ↓
Waits for commands
    ↓
ACTIVE: Processes LOAD_AND_RUN commands
    ↓
Session Closed / Timeout
    ↓
Send SIGKILL to subprocess
    ↓
Subprocess terminates
    ↓
VariableStore lost (not persisted)
```

**Subprocess Restart (on crash):**

```rust
impl SubprocessExecutor {
    pub fn execute(&mut self, lib_path: &Path, fn_name: &str)
        -> Result<Response>
    {
        match self.try_execute(lib_path, fn_name) {
            Ok(response) => Ok(response),
            Err(e) if e.is_subprocess_crash() => {
                // Subprocess died, restart it
                log::warn!("Subprocess crashed, restarting...");
                self.restart()?;

                // Note: VariableStore lost!
                // This is acceptable - user can re-eval definitions

                Err(Error::SubprocessCrashed {
                    message: "Subprocess crashed, session reset".to_string()
                })
            }
            Err(e) => Err(e),
        }
    }

    fn restart(&mut self) -> Result<()> {
        // Kill old subprocess
        self.subprocess.kill()?;

        // Spawn new one
        let new_subprocess = spawn_subprocess()?;
        self.subprocess = new_subprocess;
        self.stdin = new_subprocess.stdin.take().unwrap();
        self.stdout = BufReader::new(
            new_subprocess.stdout.take().unwrap()
        );

        Ok(())
    }
}
```

**Why Not Persist Variables:**

- Subprocess has in-memory VariableStore (Box<dyn Any>)
- Cannot serialize arbitrary Rust types
- Restart = clean state (acceptable trade-off)
- Alternative: User can save important definitions in files

**Ctrl-C Handling:**

When user presses Ctrl-C during long-running code:

1. Client sends interrupt signal (platform-specific)
2. Server kills subprocess via SIGKILL
3. Spawns new subprocess immediately
4. Returns error to client: "Execution interrupted"
5. Session continues (new subprocess ready)

This is WHY subprocess is mandatory - Rust threads cannot be interrupted!

---

## 6. Protocol Integration

The REPL uses **two separate protocols**:

1. **Client ↔ Server:** TCP + Postcard (ODD-0018)
2. **Server ↔ Subprocess:** stdin/stdout + Text ◄─ NEW

### 6.1 Client-Server Protocol

**Transport:** TCP sockets
**Serialization:** Postcard (binary, efficient)
**Framing:** Length-prefixed messages
**Full specification:** See ODD-0018 (Oxur Remote REPL Protocol Design)

#### Protocol Operations

| Operation | Handler | Server Action | Response |
|-----------|---------|---------------|----------|
| `Clone` | `handle_clone` | Create new session | `SessionId`, status: `SessionCreated` |
| `Eval` | `handle_eval` | Evaluate code in session | `value`, `out`, `err`, status: `Done` or `Error` |
| `LoadFile` | `handle_load_file` | Read file, evaluate | Same as `Eval` |
| `Close` | `handle_close` | Remove session | status: `SessionClosed` |
| `LsSessions` | `handle_ls_sessions` | List active sessions | List of `SessionId` |
| `Describe` | `handle_describe` | Server capabilities | Server version, features |
| `Interrupt` | `handle_interrupt` | Kill running eval | status: `Interrupted` |
| `History` | `handle_history` | Get eval history | List of history entries |

**Note:** The `out` and `err` fields in Response capture subprocess stdout/stderr, which is separate from the subprocess protocol channel (see Section 6.2).

### 6.2 Subprocess Protocol (Internal) ◄─ NEW

While the client-server protocol handles network communication, the server uses a separate simple text protocol to communicate with subprocesses.

**Protocol Format:**

```
Commands (Server → Subprocess via stdin):
  LOAD_AND_RUN <lib_path> <function_name>\n

Responses (Subprocess → Server via stdout):
  OXUR_EXECUTION_COMPLETE\n                    (success)
  OXUR_RUNTIME_ERROR: <error_message>\n        (error)
```

**Example Session:**

```
Server writes to subprocess stdin:
  "LOAD_AND_RUN /tmp/libeval_005.so run_user_code_5\n"

Subprocess reads command
Subprocess loads library
Subprocess executes function
Subprocess writes to stdout:
  "OXUR_EXECUTION_COMPLETE\n"

Server reads response
Server parses "OXUR_EXECUTION_COMPLETE"
Server returns success to EvalContext
```

**Why Text Protocol (not Binary):**

- Proven stable (6+ years in evcxr)
- Simple implementation
- Easy to debug (can see messages)
- No serialization complexity
- stdout mixing not an issue (user output captured separately)

**Why Not Reuse Client Protocol:**

- Different trust model (server owns subprocess)
- Different lifetime (subprocess is ephemeral)
- Different complexity needs (subprocess is simple)
- Text is sufficient for this use case

**Implementation:**

```rust
impl SubprocessExecutor {
    pub fn execute(&mut self, lib_path: &Path, fn_name: &str)
        -> Result<ExecutionResult>
    {
        // 1. Send command via stdin
        writeln!(self.stdin, "LOAD_AND_RUN {} {}",
                 lib_path.display(), fn_name)?;
        self.stdin.flush()?;

        // 2. Read response via stdout
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;

        // 3. Parse response
        if line.starts_with("OXUR_EXECUTION_COMPLETE") {
            Ok(ExecutionResult::Success { /* ... */ })
        } else if line.starts_with("OXUR_RUNTIME_ERROR:") {
            let msg = line.strip_prefix("OXUR_RUNTIME_ERROR: ")
                         .unwrap_or("");
            Err(Error::RuntimeError(msg.to_string()))
        } else {
            Err(Error::ProtocolError(line))
        }
    }
}
```

**See Section 1.4** for complete subprocess architecture rationale.

### 6.3 Error Propagation

```
Oxur Language Error (parse, expand)
  ↓
Caught by EvalContext
  ↓
Converted to ErrorKind::Parse or ErrorKind::Expand
  ↓
Passed to MessageHandler
  ↓
MessageHandler creates Response with Error status
  ↓
Sent to client

Compilation Error (rustc)
  ↓
Caught by CachedCompiler
  ↓
Translated via SourceMap                          ◄─ NEW (oxur-smap)
  ↓
Converted to ErrorKind::Lower
  ↓
Passed to MessageHandler
  ↓
Response with Error status + source location

Runtime Error (panic in user code)
  ↓
Subprocess crashes                                 ◄─ UPDATED
  ↓
SubprocessExecutor detects crash (broken pipe)     ◄─ NEW
  ↓
Restarts subprocess automatically                  ◄─ NEW
  ↓
Converted to ErrorKind::Eval
  ↓
Response with Error status

Protocol Error (malformed message)
  ↓
Caught by MessageHandler
  ↓
Converted to ErrorKind::Protocol
  ↓
Response with Error status
```

### 6.4 Streaming Output

For long-running evaluations:

```
Client → Server: Request { op: Eval, code: "long_computation()" }

Server starts evaluation
  ↓
User code prints to stdout
  ↓
Subprocess captures output
  ↓
Server → Client: Response { status: [Partial], out: "Progress: 10%" }
  ↓
More output
  ↓
Server → Client: Response { status: [Partial], out: "Progress: 50%" }
  ↓
Computation completes
  ↓
Server → Client: Response { status: [Done], value: result, out: "Done!" }
```

**Implementation Note:** v1.0 may not implement streaming; single response on completion is acceptable.

---

## 7. Three-Tier Execution Strategy

### 7.1 Tier Criteria

**Tier 1: Calculator Mode**

- **Criteria:** Simple arithmetic, literal values, no function definitions
- **Detection:** AST analysis (< 10 nodes, only +,-,*,/ operators)
- **Execution:** Direct Rust evaluation, no compilation
- **Performance:** <1ms
- **Caching:** N/A (no compilation to cache)
- **Example:** `(+ 1 2)`, `(* 3 4)`, `(- 10 5)`

**Tier 2: Cached Compilation (NEW - Enhanced)**

- **Criteria:** Previously compiled code via ArtifactCache  ◄─ NEW
- **Detection:** Content-based cache key matches existing artifact  ◄─ NEW
  - Cache key: SHA256(source + deps + opt_level + source_map)
- **Location:** Stage 0 (before parsing!)  ◄─ NEW
- **Execution:** Load pre-compiled .so/.dylib/.dll from cache
- **Performance:** 1-5ms (library load + function call)
- **Cache location:** `~/.cache/oxur/artifacts/` (persistent)  ◄─ NEW
- **Example:** Re-evaluating `(square 5)` after defining square
- **Hit rate:** High after warm-up (50-200x speedup)  ◄─ NEW

**Tier 3: JIT Compilation**

- **Criteria:** New code, cache miss, complex expressions
- **Execution:** Full compilation pipeline (12 stages)  ◄─ UPD
- **Performance:** 50-300ms (depends on code complexity)
- **Post-compile:** Artifact stored in ArtifactCache  ◄─ NEW
- **Example:** `(defn square [x] (* x x))` (first time), complex expressions

### 7.2 Tier Decision Logic (Updated)

```rust
// In EvalContext
fn decide_tier(&self, code: &str) -> Tier {
    // ──────────────────────────────────────────────
    // TIER 1: Calculator (fastest - no compilation)
    // ──────────────────────────────────────────────

    // Quick check before parsing
    if looks_like_simple_arithmetic(code) {
        // Actually parse to verify
        if let Ok(core_form) = quick_parse(code) {
            if is_simple_arithmetic(&core_form) {
                return Tier::Calculator;  // <1ms path
            }
        }
    }

    // ──────────────────────────────────────────────
    // TIER 2: Cache Check (NEW - Stage 0)
    // ──────────────────────────────────────────────

    // Must parse to generate cache key
    let mut source_map = SourceMap::new();
    let surface = oxur_lang::parse_lisp(code, &mut source_map)?;
    let core = oxur_lang::expand(surface, &mut source_map)?;

    // Generate cache key (content-based)
    let cache_key = ArtifactCache::cache_key(
        code,
        &self.dependencies,
        self.opt_level,
        &source_map,
    );

    // Check cache (persistent across sessions)
    if self.cache.get(&cache_key).is_some() {
        return Tier::Cached;  // 1-5ms path
    }

    // ──────────────────────────────────────────────
    // TIER 3: JIT (compile and cache)
    // ──────────────────────────────────────────────

    Tier::Jit  // 50-300ms path, but stores in cache for next time
}

fn is_simple_arithmetic(form: &CoreForm) -> bool {
    match form {
        CoreForm::Literal(_) => true,
        CoreForm::FunctionCall { function, args, .. } => {
            is_arithmetic_op(function)
            && args.len() <= 3
            && args.iter().all(|a| is_simple_arithmetic(a))
        }
        _ => false
    }
}

fn is_arithmetic_op(name: &str) -> bool {
    matches!(name, "+" | "-" | "*" | "/" | "%" | "==" | "<" | ">")
}
```

### 7.3 Cache Architecture Integration

**Stage 0: Pre-Compilation Cache Check**

```rust
impl EvalContext {
    pub fn eval(&mut self, code: &str) -> Result<Value> {
        // ═════════════════════════════════════════════
        // STAGE 0: Check persistent cache FIRST
        // ═════════════════════════════════════════════

        // Generate cache key (requires parsing)
        let mut source_map = SourceMap::new();
        let surface = oxur_lang::parse_lisp(code, &mut source_map)?;
        let core = oxur_lang::expand(surface, &mut source_map)?;

        let cache_key = ArtifactCache::cache_key(
            code,
            &self.deps,
            self.opt_level,
            &source_map,
        );

        // Try cache lookup
        if let Some(artifact_path) = self.cache.get(&cache_key) {
            // CACHE HIT - skip compilation entirely!
            return self.execute_cached_artifact(artifact_path);
        }

        // CACHE MISS - continue to compilation
        // ... rest of pipeline (Stages 1-12)
    }
}
```

**Stage 11: Post-Compilation Cache Store**

```rust
impl CachedCompiler {
    async fn eval(&mut self, core: CoreForm, source_map: SourceMap)
        -> Result<Response>
    {
        // ... Stages 4-10: Lower, wrap, generate, compile ...

        let artifact_path = self.compile(&source).await?;

        // ═════════════════════════════════════════════
        // STAGE 11: Store in persistent cache
        // ═════════════════════════════════════════════

        let cache_key = ArtifactCache::cache_key(
            &source,
            &self.deps,
            self.opt_level,
            &source_map,
        );

        // Copy artifact to cache directory
        let cached_path = self.cache.insert(cache_key, artifact_path)?;

        // Future evals with same code: instant cache hit!

        // ... Stage 12: Execute ...
    }
}
```

### 7.4 Tier Performance Targets (Updated)

| Tier | First Eval | Subsequent Eval | Notes |
|------|-----------|----------------|-------|
| **Tier 1** | <1ms | <1ms | No caching needed (pure calculation) |
| **Tier 2** | N/A | 1-5ms | Cache hit (load .so + call) |
| **Tier 3** | 50-300ms | See Tier 2 | First time: compile + cache store |

**Performance Breakdown (Typical Cold Compilation):**

```
Tier 3 (JIT - First Time):
  Stage 0:  Cache miss           ~1ms
  Stage 1:  Parse                ~1ms
  Stage 2:  Expand               ~2ms
  Stage 3:  Tier decision        <1ms
  Stage 4:  Lower                ~5ms
  Stage 5:  Wrap                 ~2ms
  Stage 6:  Generate             ~1ms
  Stage 7:  Write                <1ms
  Stage 8:  Compile (cargo)      ~280ms  ← Dominates!
  Stage 9:  Parse output         ~1ms
  Stage 10: Rename               <1ms
  Stage 11: Cache store          ~1ms   ← NEW
  Stage 12: Execute              ~5ms
  ────────────────────────────────────
  Total:                         ~300ms

Tier 2 (Cached - Subsequent):
  Stage 0:  Cache hit + load     ~1-5ms
  ────────────────────────────────────
  Total:                         ~1-5ms

Speedup: 60-300x faster!
```

**Cache Impact Analysis:**

| Scenario | Cold (Tier 3) | Warm (Tier 2) | Speedup |
|----------|---------------|---------------|---------|
| Simple function | 50ms | 1ms | 50x |
| Complex logic | 150ms | 2ms | 75x |
| Large module | 300ms | 5ms | 60x |

### 7.5 Cache Persistence and Warm-up

**Session Lifecycle:**

```
Session 1 (Fresh Start):
  > (defn square [x] (* x x))    → Tier 3 (compile)  ~100ms
  > (square 5)                   → Tier 2 (cached)   ~2ms
  > (square 10)                  → Tier 2 (cached)   ~2ms

[REPL restart]

Session 2 (Cache Persists!):
  > (defn square [x] (* x x))    → Tier 2 (cached!)  ~2ms  ← Instant!
  > (square 5)                   → Tier 2 (cached)   ~2ms
```

**Key Insight:** Cache survives REPL restarts! First eval of each day still benefits from yesterday's compilation.

**Cache Warming Strategy:**

```rust
// On REPL startup, pre-load common definitions
impl ReplSession {
    fn warm_cache(&mut self) {
        // Standard library functions already cached
        for common_fn in &COMMON_FUNCTIONS {
            if self.cache.get(&common_fn.cache_key).is_none() {
                // Compile and cache in background
                self.compile_and_cache_async(common_fn);
            }
        }
    }
}
```

### 7.6 Incremental Compilation vs Persistent Cache

**Two Separate Optimizations:**

```
┌─────────────────────────────────────────────────┐
│ PERSISTENT CACHE (ArtifactCache)                │
│ - Stores final .so/.dylib/.dll artifacts        │
│ - Location: ~/.cache/oxur/artifacts/            │
│ - Survives: REPL restarts, system reboots       │
│ - Hit: 1-5ms (load artifact + execute)          │
│ - Miss: Full compilation (but stores for next)  │
│                                                 │
│ Benefits: 50-300x speedup on cache hit          │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│ INCREMENTAL COMPILATION (cargo)                 │
│ - Stores intermediate build artifacts           │
│ - Location: session_dir/target/                 │
│ - Survives: Single REPL session only            │
│ - Benefit: 3-5x faster recompile in same session│
│                                                 │
│ Use case: Modifying code iteratively            │
└─────────────────────────────────────────────────┘
```

**Combined Effect:**

```
Scenario: User defines, then modifies function

> (defn foo [x] (+ x 1))
  Tier 3: 100ms (cold compile, cache miss)
  → Stores in: ArtifactCache + incremental cache

> (defn foo [x] (+ x 2))  ; Modified!
  Tier 3: 30ms (incremental compile, new cache key)
  → Stores in: ArtifactCache (new key) + incremental cache

> (defn foo [x] (+ x 1))  ; Back to original
  Tier 2: 2ms (cache hit!)
  → Loads from: ArtifactCache (original key still cached)
```

### 7.7 Decision Flow Diagram

```
User Input: "(+ x y)"
        ↓
   ┌────────────────┐
   │ Quick Check:   │
   │ Arithmetic?    │
   └────┬───────────┘
        │
    Yes │ No
        ↓    ↓
    ┌───────┐ ┌──────────────┐
    │ Tier 1│ │ Parse & Hash │
    │ <1ms  │ │              │
    └───────┘ └──────┬───────┘
                     │
              ┌──────────────┐
              │ Stage 0:     │
              │ Cache Check  │
              └──────┬───────┘
                     │
               Hit │ │ Miss
                   ↓ ↓
            ┌──────┐ ┌──────────────┐
            │Tier 2│ │   Tier 3     │
            │1-5ms │ │ 50-300ms     │
            │      │ │ (+ cache     │
            │      │ │  store)      │
            └──────┘ └──────────────┘
```

### 7.8 Tier Selection Examples

**Example 1: Simple Arithmetic → Tier 1**

```lisp
> (+ 2 3)
Tier: Calculator
Time: <1ms
Reason: Simple arithmetic, no variables
```

**Example 2: Function Call (Cached) → Tier 2**

```lisp
> (defn double [x] (* x 2))  ; First time
Tier: JIT (Tier 3)
Time: 85ms
Reason: New function definition, cache miss

> (double 5)  ; Subsequent call
Tier: Cached (Tier 2)
Time: 2ms
Reason: Artifact found in cache
```

**Example 3: Modified Code → Tier 3 (New Cache Entry)**

```lisp
> (defn triple [x] (* x 3))  ; First version
Tier: JIT (Tier 3)
Time: 90ms
Cache: Stores key abc123...

> (defn triple [x] (* x 4))  ; Modified (different hash!)
Tier: JIT (Tier 3)
Time: 30ms (incremental help)
Cache: Stores NEW key def456...

> (defn triple [x] (* x 3))  ; Back to original
Tier: Cached (Tier 2)
Time: 2ms
Reason: Original key abc123 still in cache!
```

**Example 4: Complex Expression → Tier 3**

```lisp
> (let [xs (range 100)] (map (fn [x] (* x x)) xs))
Tier: JIT (Tier 3)
Time: 180ms
Reason: Complex expression, many nodes, cache miss
```

### 7.9 Performance Monitoring

**User-Facing Diagnostics:**

```lisp
> (set-option :show-timings true)
true

> (defn factorial [n] (if (= n 0) 1 (* n (factorial (- n 1)))))
Tier:      JIT (cache miss)
Parse:     1ms
Expand:    2ms
Lower:     4ms
Compile:   92ms
Cache:     1ms (stored)
Execute:   3ms
Total:     103ms

> (factorial 5)
Tier:      Cached (cache hit!)
Load:      1ms
Execute:   <1ms
Total:     2ms

Speedup: 51x faster!
```

### 7.10 Optimization Recommendations

**For Users:**

1. **Avoid constantly modifying code** - Each modification creates new cache entry
2. **Define functions once** - Maximum cache benefit
3. **Use Tier 1 for quick math** - Simple arithmetic is instant
4. **Warm up cache** - First run of each function pays compilation cost

**For Implementers:**

1. **Prioritize cache hits** - Biggest performance win (50-200x)
2. **Keep Tier 1 threshold low** - Only trivial arithmetic qualifies
3. **Monitor cache hit rate** - Aim for >80% after warm-up
4. **Implement cache eviction** - LRU policy, configurable size limit

### 7.11 Future Enhancements (v1.1+)

**Potential Tier 2 Improvements:**

1. **Partial Caching** - Cache individual functions, not entire sessions
2. **Shared Cache** - Multiple REPL sessions share artifacts
3. **Pre-compilation** - Background compile of common patterns
4. **Smart Eviction** - Keep frequently-used artifacts, evict cold ones

**Potential New Tier:**

**Tier 2.5: Inline Interpretation** (between Cached and JIT)

- Bytecode interpreter for medium-complexity code
- Faster than JIT compilation (5-20ms)
- Slower than native execution
- Use case: One-off complex expressions

---

**See Section 13 (Performance Considerations)** for detailed benchmarks, profiling data, and optimization strategies.

---

## 8. Integration Points with External Crates

### 8.1 oxur-smap Integration (NEW - Foundation)

**Location:** `oxur-smap/` (separate crate, no dependencies)

**Purpose:** Foundation crate for multi-stage source mapping

**Required Types:**

```rust
// oxur-smap/src/lib.rs

/// Unique identifier for AST nodes across all compilation stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    /// Generate a new unique NodeId
    pub fn new() -> Self {
        // Thread-safe atomic counter
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        NodeId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Source position in original Oxur code
#[derive(Debug, Clone, PartialEq)]
pub struct SourcePos {
    pub file: String,      // Source file path
    pub line: u32,         // 1-indexed line number
    pub column: u32,       // 1-indexed column number
    pub length: u32,       // Span length (for highlighting)
}

/// Tracks AST transformations across compilation stages
pub struct SourceMap {
    surface_positions: HashMap<NodeId, SourcePos>,
    surface_to_core: HashMap<NodeId, NodeId>,
    core_to_rust: HashMap<NodeId, NodeId>,
}

impl SourceMap {
    pub fn new() -> Self;

    // Called by oxur-lang during parsing
    pub fn record_surface_node(&mut self, node: NodeId, pos: SourcePos);

    // Called by oxur-lang during expansion
    pub fn record_expansion(&mut self, surface: NodeId, core: NodeId);

    // Called by oxur-comp during lowering (crosses semantic boundary via Oxur AST)
    pub fn record_lowering(&mut self, core: NodeId, rust: NodeId);

    // Called by oxur-repl for error translation
    pub fn lookup(&self, rust_node: NodeId) -> Option<SourcePos>;

    // For cache key generation (content-based hashing)
    pub fn content_hash(&self) -> String;
}
```

**Used by:** oxur-lang, oxur-comp, oxur-ast, oxur-repl

**Dependency:** All other crates depend on this (foundation layer)

**Status:** MUST be implemented in Phase 0 (prerequisite for all other work)

---

### 8.2 oxur-lang Integration

**Required API:**

```rust
// oxur-lang/src/parser.rs
pub fn parse_lisp(
    source: &str,
    source_map: &mut SourceMap    // ◄─ NEW parameter
) -> Result<SurfaceForms, ParseError>;

pub fn parse_core_forms(
    source: &str,
    source_map: &mut SourceMap    // ◄─ NEW parameter
) -> Result<CoreForms, ParseError>;

// oxur-lang/src/expander.rs
pub fn expand(
    surface: SurfaceForms,
    source_map: &mut SourceMap    // ◄─ NEW parameter
) -> Result<CoreForms, ExpandError>;

// Error types
pub enum ParseError {
    UnexpectedToken {
        position: SourcePos,      // Uses SourcePos from oxur-smap
        found: String,
        expected: String
    },
    UnmatchedParens { position: SourcePos },
    // ...
}

pub enum ExpandError {
    UnknownMacro { name: String, position: SourcePos },
    MacroExpansionFailed { message: String, position: SourcePos },
    // ...
}
```

**SourceMap Recording Responsibilities:**

During `parse_lisp()`:

```rust
// For each parsed node
let node_id = NodeId::new();
source_map.record_surface_node(node_id, SourcePos {
    file: source_file.clone(),
    line: token.line,
    column: token.column,
    length: token.text.len() as u32,
});
```

During `expand()`:

```rust
// For each surface→core transformation
source_map.record_expansion(surface_node_id, core_node_id);
```

**Called by:** EvalContext in `eval()` method

**Data Types:**

```rust
// Core Forms (canonical S-expressions)
pub enum CoreForm {
    Literal(Literal),
    Variable(String),
    FunctionCall {
        function: String,
        args: Vec<CoreForm>,
        node_id: NodeId,          // ◄─ NEW: Track this node
    },
    FunctionDefinition {
        name: String,
        params: Vec<(String, Type)>,
        body: Box<CoreForm>,
        node_id: NodeId,          // ◄─ NEW
    },
    IfExpr {
        condition: Box<CoreForm>,
        then_branch: Box<CoreForm>,
        else_branch: Option<Box<CoreForm>>,
        node_id: NodeId,          // ◄─ NEW
    },
    LetBinding {
        bindings: Vec<(String, CoreForm)>,
        body: Box<CoreForm>,
        node_id: NodeId,          // ◄─ NEW
    },
    // ... other forms (all need node_id)
}

pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    // ...
}

pub enum Type {
    Infer,              // Let rustc infer
    Named(String),      // e.g., "i32", "String"
    Generic { ... },    // For generics
    // ...
}
```

**Status:** API must be defined and implemented with SourceMap support

---

### 8.3 oxur-comp Integration

**Required API:**

```rust
// oxur-comp/src/lower.rs
pub fn lower(
    core: &CoreForm,
    source_map: &mut SourceMap    // ◄─ NEW parameter
) -> Result<RustAst, LowerError>;

// Error type
pub enum LowerError {
    UnsupportedForm {
        form: String,
        position: SourcePos       // Uses SourcePos from oxur-smap
    },
    TypeMismatch {
        expected: Type,
        found: Type,
        position: SourcePos
    },
    // ...
}
```

**SourceMap Recording Responsibilities:**

During `lower()`:

```rust
// For each core→rust transformation
let rust_node_id = NodeId::new();
source_map.record_lowering(core_form.node_id, rust_node_id);

// Embed rust_node_id in generated Rust AST (as attribute or comment)
```

**Called by:** RustAstWrapper in `generate()` method

**Data Types:**

```rust
// Rust AST (using syn crate)
pub type RustAst = syn::File;  // Or syn::Item, depending on granularity

// Lowering maps:
// CoreForm::FunctionDefinition → syn::ItemFn (with node_id attribute)
// CoreForm::IfExpr → syn::ExprIf (with node_id comment)
// CoreForm::LetBinding → syn::Stmt::Local (with node_id comment)
// etc.
```

**Example Lowering with NodeId:**

```rust
// Input: CoreForm
CoreForm::FunctionCall {
    function: "+".to_string(),
    args: vec![Variable("x"), Variable("y")],
    node_id: NodeId(200),  // From expand stage
}

// Output: Rust AST with NodeId embedded
syn::Expr::Binary {
    left: syn::Ident("x"),
    op: syn::BinOp::Add,
    right: syn::Ident("y"),
    attrs: vec![
        syn::Attribute {
            // #[oxur_node = "300"]
            path: syn::Path::from("oxur_node"),
            tokens: quote!(= "300"),
        }
    ]
}

// SourceMap records: NodeId(200) → NodeId(300)
```

**Status:** Must be implemented with SourceMap support

---

### 8.4 oxur-ast Integration

**Required API:**

```rust
// oxur-ast/src/printer.rs
pub fn print_rust(ast: &syn::File) -> String;
```

**Responsibilities:**

1. Convert Rust AST to source string
2. Preserve NodeId annotations (as comments)
3. Format code readably

**NodeId Preservation:**

```rust
// Input: syn::Expr with attribute
#[oxur_node = "300"]
x + y

// Output: Source with comment
/* oxur_node=300 */ x + y
```

**Why Comments (not attributes):**

- Attributes may be stripped by rustc
- Comments survive compilation
- Easy to parse from error messages

**Called by:** RustAstWrapper after wrapping AST

**Implementation:** Likely uses `prettyplease` or custom printer

**Status:** Must preserve NodeId information in output

---

### 8.5 rust-analyzer Integration (NEW)

**Purpose:** Type inference for variables

**Required API:**

```rust
// Use rust-analyzer as library (v1.0 approach)
use rust_analyzer::Analysis;

pub struct TypeInference {
    analysis: Analysis,
}

impl TypeInference {
    pub fn new() -> Self;

    pub fn infer_type(
        &self,
        code: &str,
        var_name: &str
    ) -> Result<String, InferenceError>;
}

pub enum InferenceError {
    AnalysisFailed(String),
    TypeNotFound { var: String },
    AmbiguousType { var: String, candidates: Vec<String> },
}
```

**Alternative (v1.1+):** Use LSP protocol instead of library

- Avoids rust-analyzer build time impact
- More complex integration
- Per evcxr maintainer recommendation

**Called by:** CachedCompiler when variable type needed

**Example Usage:**

```rust
// User defines variable
let code = "let x = vec![1, 2, 3];";

// Infer type
let type_str = type_inference.infer_type(code, "x")?;
// Result: "Vec<i32>"

// Generate correct load code
let load_code = format!(
    "let {}: {} = vars.get(\"{}\").unwrap().downcast_ref().unwrap();",
    "x", type_str, "x"
);
```

**Status:** Implement in Phase 1

---

### 8.6 Dependency Summary

```
                    oxur-smap (foundation)
                         ↑
         ┌───────────────┼───────────────┐
         │               │               │
    oxur-lang       oxur-comp       oxur-ast
         ↑               ↑               ↑
         └───────────────┴───────────────┘
                         │
                    oxur-repl
         ┌───────────────┼───────────────┐
         │               │               │
    EvalContext   RustAstWrapper  CachedCompiler
         │               │               │
         └───────────────┴───────────────┘
                         │
                 SubprocessExecutor
                         ↓
                   Subprocess
```

**Detailed Call Graph:**

```
oxur-repl (EvalContext)
  ├─→ oxur-lang::parse_lisp(code, &mut source_map)
  ├─→ oxur-lang::expand(surface, &mut source_map)
  └─→ oxur-lang::parse_core_forms(code, &mut source_map)

oxur-repl (RustAstWrapper)
  ├─→ oxur-comp::lower(core, &mut source_map)
  └─→ oxur-ast::print_rust(ast)

oxur-repl (CachedCompiler)
  ├─→ cargo build (external tool)
  └─→ TypeInference::infer_type() (rust-analyzer)

oxur-repl (SubprocessExecutor)
  └─→ Subprocess via stdin/stdout protocol

Subprocess
  └─→ libloading::Library::new() (external crate)
```

---

### 8.7 Critical Path Blockers

**Phase 0 Prerequisites (BLOCKING):**

1. **oxur-smap crate** - Foundation for all others
   - NodeId, SourcePos, SourceMap types
   - Must exist before any other crate can be implemented
   - Status: Design complete, needs implementation

2. **ArtifactCache design** - Cache key generation
   - Depends on SourceMap::content_hash()
   - Required for day-one caching
   - Status: Design complete, needs implementation

**Phase 1 Prerequisites (BLOCKING):**

1. **oxur-lang API** - Parsing and expansion
   - Must accept `&mut SourceMap` parameter
   - Must record transformations
   - Must populate CoreForm with NodeId
   - Status: API defined, needs implementation

2. **oxur-comp API** - Lowering to Rust
   - Must accept `&mut SourceMap` parameter
   - Must record core→rust transformations
   - Must embed NodeId in generated AST
   - Status: API defined, needs implementation

3. **oxur-ast API** - Printing with NodeId preservation
   - Must preserve NodeId as comments
   - Must format readably
   - Status: API defined, needs implementation

4. **rust-analyzer integration** - Type inference
   - Library or LSP approach
   - Must handle inference failures gracefully
   - Status: Approach decided (library for v1.0)

**Data Type Agreement:**

1. **CoreForm definition** - Canonical IR
   - Must be agreed upon by oxur-lang and oxur-comp
   - Must include NodeId in all variants
   - Must support Oxur language features
   - Status: Structure defined, needs finalization

---

### 8.8 API Contracts and Invariants

**SourceMap Threading Contract:**

```rust
// INVARIANT: SourceMap must be passed through entire pipeline

// Step 1: Parse records surface nodes
let mut source_map = SourceMap::new();
let surface = parse_lisp(code, &mut source_map)?;
// POST: source_map contains Surface NodeId → SourcePos mappings

// Step 2: Expand records transformations
let core = expand(surface, &mut source_map)?;
// POST: source_map contains Surface → Core mappings

// Step 3: Lower records transformations
let rust = lower(&core, &mut source_map)?;
// POST: source_map contains Core → Oxur AST → syn mappings

// Step 4: Error translation uses complete map
let original_pos = source_map.lookup(rust_node_id)?;
// REQUIRES: All three stages have recorded mappings
```

**NodeId Uniqueness Contract:**

```rust
// INVARIANT: Each AST node has unique NodeId across all stages

impl NodeId {
    pub fn new() -> Self {
        // Thread-safe global counter ensures uniqueness
        // No two nodes will ever have the same ID
    }
}
```

**Error Position Contract:**

```rust
// INVARIANT: All errors must include SourcePos

pub trait ErrorWithPosition {
    fn position(&self) -> SourcePos;
}

impl ErrorWithPosition for ParseError { ... }
impl ErrorWithPosition for ExpandError { ... }
impl ErrorWithPosition for LowerError { ... }

// Enables uniform error display
fn display_error<E: ErrorWithPosition>(err: &E) {
    let pos = err.position();
    println!("Error at {}:{}:{}", pos.file, pos.line, pos.column);
}
```

---

### 8.9 Integration Testing Strategy

**Cross-Crate Tests:**

```rust
// Test full pipeline integration
#[test]
fn test_source_map_end_to_end() {
    let code = "(+ x y)";
    let mut source_map = SourceMap::new();

    // Parse
    let surface = oxur_lang::parse_lisp(code, &mut source_map).unwrap();

    // Expand
    let core = oxur_lang::expand(surface, &mut source_map).unwrap();

    // Lower
    let rust = oxur_comp::lower(&core, &mut source_map).unwrap();

    // Verify we can lookup original position
    let rust_node_id = extract_node_id(&rust);
    let original_pos = source_map.lookup(rust_node_id).unwrap();

    assert_eq!(original_pos.line, 1);
    assert_eq!(original_pos.column, 1);
}
```

**Mock Implementations for Testing:**

```rust
// Mock oxur-lang for REPL testing
#[cfg(test)]
mod mock_oxur_lang {
    pub fn parse_lisp(
        source: &str,
        source_map: &mut SourceMap
    ) -> Result<SurfaceForms> {
        // Simplified parser for testing
    }
}
```

---

### 8.10 Future Integration Points (v1.1+)

**Potential Additions:**

1. **LSP Integration** (alternative to rust-analyzer library)
   - Pros: Faster builds, more stable API
   - Cons: More complex integration
   - Trigger: Build times become problematic

2. **IDE Integration** (via SourceMap serialization)
   - Export SourceMap for editor plugins
   - Enable jump-to-definition
   - Requires serialization support in oxur-smap

3. **Debugger Integration** (via SourceMap)
   - Map Rust stack traces to Oxur source
   - Requires DWARF debug info coordination

4. **Jupyter Kernel** (similar to evcxr_jupyter)
   - Separate crate: oxur-jupyter
   - Reuses compilation pipeline
   - ZMQ protocol instead of TCP

---

**Status Summary:**

| Component | Status | Priority | Blocker |
|-----------|--------|----------|---------|
| oxur-smap | Design complete | P0 | All others |
| oxur-lang API | Defined | P1 | oxur-smap |
| oxur-comp API | Defined | P1 | oxur-smap |
| oxur-ast API | Defined | P1 | oxur-smap |
| rust-analyzer | Approach decided | P1 | - |
| CoreForm agreement | In progress | P1 | - |

**Next Step:** Implement oxur-smap (Phase 0), then proceed with integration implementations.

---

## 9. Critical Paths: Examples

### 9.1 Simple Arithmetic

```
Input: (+ 1 2)
Time: <1ms
Tier: Calculator

Flow:
1. Client.eval("(+ 1 2)")
2. Request sent to server
3. MessageHandler → SessionManager → EvalContext
4. EvalContext.eval("(+ 1 2)")
   a. parse_lisp() → SurfaceForms
   b. expand() → CoreForm::FunctionCall("+", [Literal(1), Literal(2)])
   c. decide_tier() → Tier::Calculator
   d. eval_calculator() → direct Rust: 1 + 2 = 3
5. Result: Value::Int(3)
6. Response sent to client
7. Client displays: 3

No compilation, no subprocess, instant result.
```

### 9.2 Function Definition (First Time)

```
Input: (defn square [x] (* x x))
Time: 200-300ms (cold compile)
Tier: JIT

Flow:
1. Client.eval("(defn square [x] (* x x))")
2. Request sent to server
3. MessageHandler → SessionManager → EvalContext
4. EvalContext.eval(...)
   a. parse_lisp() → SurfaceForms (defn form)
   b. expand() → CoreForm::FunctionDefinition { ... }
   c. decide_tier() → Tier::Jit (not cached)
   d. compiler.eval(core_forms)
5. CachedCompiler.eval()
   a. rust_ast_wrapper.generate()
      i.   oxur_comp::lower() → syn::ItemFn
      ii.  wrap_in_function() → complete library AST
      iii. oxur_ast::print_rust() → Rust source
      iv.  add source map comments
   b. write to session_dir/src/lib.rs
   c. invoke cargo build
   d. parse cargo JSON output
   e. rename artifact to libeval_007.so
   f. send LOAD command to subprocess
   g. subprocess loads library
   h. subprocess calls function (defines square in VariableStore)
   i. subprocess returns completion marker
6. Result: Success (function defined)
7. Response sent to client
8. Client displays: Function square defined

Compilation occurred, took ~250ms
```

### 9.3 Function Call (Cached)

```
Input: (square 5)
Time: 1-5ms
Tier: Cached (after defining square)

Flow:
1. Client.eval("(square 5)")
2. Request sent to server
3. MessageHandler → SessionManager → EvalContext
4. EvalContext.eval("(square 5)")
   a. parse_lisp() → CoreForm::FunctionCall("square", [Literal(5)])
   b. decide_tier() → Tier::Cached (square already defined in VariableStore)
   c. compiler.eval() → quick path
5. CachedCompiler.eval()
   a. Generate code (calls square function from VariableStore)
   b. Already compiled (cache hit)
   c. Execute in subprocess
   d. Result: 25
6. Response sent to client
7. Client displays: 25

Library reused, fast execution.
```

### 9.4 Compilation Error

```
Input: (defn bad [x] (+ x y))  ; y is undefined
Time: 200-300ms (compilation attempted)
Tier: JIT

Flow:
1. Client.eval("(defn bad [x] (+ x y))")
2. Request to server
3. EvalContext.eval()
   a. parse_lisp() → success
   b. expand() → CoreForm::FunctionDefinition
   c. decide_tier() → Tier::Jit
   d. compiler.eval()
4. CachedCompiler.eval()
   a. rust_ast_wrapper.generate()
      i.  lower() → syn::ItemFn with `x + y`
      ii. wrap_in_function()
      iii. print_rust() → source with undefined `y`
   b. write to lib.rs
   c. cargo build
   d. parse JSON → ERRORS found
   e. translate_errors()
      i.  Extract rustc span (lib.rs:42)
      ii. Find source map comment: /* oxur_node=456 */
      iii. Lookup in SourceMap → "test.ox:3:20"
      iv. Create OxurError
5. Return Err(CompileError::RustcErrors([oxur_error]))
6. MessageHandler catches error
7. Response with Error status
8. Client displays:
   Error at test.ox:3:20: cannot find value `y` in this scope

Error successfully translated to original source position.
```

---

## 10. File System Organization

### 10.1 Directory Structure

```
/dev/shm/oxur-repl/  (Linux tmpfs)  ◄─ NEW
or
/var/folders/.../oxur-repl/  (macOS)
or
%TEMP%\oxur-repl\  (Windows)

Contents:
├── session-550e8400-e29b-41d4-a716-446655440000/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   │   └── lib.rs                      # Generated code
│   ├── target/
│   │   └── x86_64-unknown-linux-gnu/
│   │       └── debug/
│   │           ├── libctx.so
│   │           ├── libeval_001.so
│   │           ├── libeval_002.so
│   │           ├── libeval_003.so
│   │           └── incremental/        # Cargo's incremental cache
│   │               └── ctx-{hash}/
│   └── metadata.json                   # Session metadata (optional)
│
├── session-6ba7b810-9dad-11d1-80b4-00c04fd430c8/
│   └── ... (same structure)
│
└── session-6ba7b811-9dad-11d1-80b4-00c04fd430c9/
    └── ... (same structure)
```

### 10.2 Cargo.toml Template

```toml
[package]
name = "ctx"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[profile.dev]
opt-level = 0        # Fastest compile time for REPL iteration
incremental = true   # 3-5x speedup on warm builds

# No dependencies in v1.0
# v1.1: user-requested dependencies via (require "crate")
[dependencies]
```

**Note on Optimization Level:**

- `opt-level = 0`: Fastest compilation (default for REPL development)
- `opt-level = 2`: Balanced (user can enable via `:optimization-level`)
- `opt-level = 3`: Slowest compile, fastest runtime (for production scripts)

See Section 13.6 for detailed performance trade-offs.

### 10.3 Generated lib.rs Structure

```rust
// src/lib.rs (generated by RustAstWrapper)

// Embedded VariableStore (same for all evaluations)
// Note: This must match the VariableStore in the subprocess runtime
// to ensure ABI compatibility when loading dynamic libraries.
mod oxur_variable_store {
    use std::any::Any;
    use std::collections::HashMap;

    pub struct VariableStore {
        variables: HashMap<String, Box<dyn Any + 'static>>,
    }

    impl VariableStore {
        pub fn put_variable<T: 'static>(&mut self, name: &str, value: T) {
            self.variables.insert(name.to_owned(), Box::new(value));
        }

        pub fn check_variable<T: 'static>(&mut self, name: &str) -> bool {
            if let Some(v) = self.variables.get(name) {
                v.downcast_ref::<T>().is_some()
            } else {
                true
            }
        }

        pub fn take_variable<T: 'static>(&mut self, name: &str) -> T {
            *self.variables.remove(name)
                .expect("Variable missing")
                .downcast()
                .expect("Variable type mismatch")
        }
    }
}

// Generated function (unique per evaluation)
#[no_mangle]
pub extern "C" fn run_user_code_7(
    mut store_ptr: *mut oxur_variable_store::VariableStore
) -> *mut oxur_variable_store::VariableStore {
    let store = unsafe { &mut *store_ptr };

    // Load existing variables
    // (Generated based on SessionState.variables)
    if !store.check_variable::<i32>("x") { return store_ptr; }
    let mut x = store.take_variable::<i32>("x");

    // User code (lowered from Core Forms, with source map comments)
    /* oxur_node=42 */ let result = /* oxur_node=43 */ x * /* oxur_node=44 */ x;

    // Store variables back
    store.put_variable("x", x);
    store.put_variable("result", result);

    store_ptr
}
```

### 10.4 Cleanup Strategy

**On Session Close:**

```rust
impl Drop for SessionDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
```

**On Server Startup:**

```rust
fn cleanup_stale_sessions() {
    let repl_dir = Path::new("/tmp/oxur-repl");
    if !repl_dir.exists() { return; }

    for entry in std::fs::read_dir(repl_dir)? {
        let path = entry?.path();
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified) = metadata.modified() {
                let age = SystemTime::now().duration_since(modified)?;
                if age > Duration::from_secs(24 * 60 * 60) {  // >24h
                    let _ = std::fs::remove_dir_all(path);
                }
            }
        }
    }
}
```

**On Server Shutdown:**

```rust
impl Drop for SessionManager {
    fn drop(&mut self) {
        // close_all() was already called by ReplServer
        // Sessions dropped → SessionDirs dropped → cleanup
    }
}
```

---

### 10.5 Global Cache Directory ◄─ NEW

**Linux:**

```
~/.cache/oxur/
└── artifacts/
    ├── a1b2c3d4...f8.so
    ├── e5f6g7h8...a2.so
    └── ... (content-addressed by SHA256)
```

**macOS:**

```
~/Library/Caches/oxur/
└── artifacts/
    └── ...
```

**Windows:**

```
%LOCALAPPDATA%\oxur\cache\
└── artifacts\
    └── ...
```

**Cache Management:**

- Location: Platform-appropriate cache dir
- Naming: SHA256 hash of (source + deps + opt_level)
- Lifetime: Persistent across sessions
- Eviction: LRU when > 1GB or > 1000 files
- Shared: All sessions use same cache

---

## 11. Error Flow and Translation

### 11.1 Error Categories

```
┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Protocol                                     │
│ - Malformed message                                     │
│ - Unknown operation                                     │
│ - Invalid session ID                                    │
│ - Missing required field                                │
│                                                         │
│ Handling: MessageHandler catches, returns error Response│
│ Translation: None (protocol-level)                      │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Session                                      │
│ - Session not found                                     │
│ - Session limit reached                                 │
│ - Session timeout                                       │
│                                                         │
│ Handling: SessionManager returns error                  │
│ Translation: None                                       │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Parse                                        │
│ - Unexpected token                                      │
│ - Unmatched parentheses                                 │
│ - Invalid syntax                                        │
│                                                         │
│ Handling: EvalContext catches oxur_lang::ParseError     │
│ Translation: Already has Oxur source position  ◄─ UPD   │
│   (SourcePos from oxur-smap)                            │
│ Source: oxur-lang crate                                 │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Expand                                       │
│ - Unknown macro                                         │
│ - Macro expansion failed                                │
│ - Invalid macro arguments                               │
│                                                         │
│ Handling: EvalContext catches oxur_lang::ExpandError    │
│ Translation: Already has Oxur source position  ◄─ UPD   │
│   (SourcePos from oxur-smap)                            │
│ Source: oxur-lang crate                                 │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Lower                                        │
│ - Type mismatch                                         │
│ - Undefined variable                                    │
│ - Cannot find value in scope                            │
│ - Borrow checker violations                             │
│                                                         │
│ Handling: CachedCompiler catches rustc errors           │
│ Translation: *** SOURCE MAP REQUIRED ***                │
│ Process: Multi-stage lookup                    ◄─ NEW   │
│   rustc error (lib.rs:42)                               │
│     → Extract NodeId from source                        │
│     → Lookup Rust NodeId → Core NodeId                  │
│     → Lookup Core NodeId → Surface NodeId               │
│     → Lookup Surface NodeId → SourcePos                 │
│     → Return original Oxur position                     │
│ Source: cargo/rustc                                     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Eval                                         │
│ - Runtime panic                                         │
│ - Subprocess crash                                      │
│ - Arithmetic overflow                                   │
│ - Unwrap on None                                        │
│                                                         │
│ Handling: SubprocessExecutor detects crash    ◄─ UPD    │
│   Protocol: "OXUR_RUNTIME_ERROR: <msg>\n"      ◄─ NEW   │
│ Translation: Difficult (runtime, no static position)    │
│ Mitigation: Stack traces if available                   │
│ Source: User code at runtime in subprocess              │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Io                                           │
│ - File not found                                        │
│ - Permission denied                                     │
│ - Disk full                                             │
│                                                         │
│ Handling: Various components                            │
│ Translation: None (system-level)                        │
│ Source: File system                                     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Timeout                                      │
│ - Request timeout                                       │
│ - Compilation timeout                                   │
│ - Execution timeout                                     │
│                                                         │
│ Handling: Tokio timeout wrappers + subprocess kill      │
│ Translation: None                                       │
│ Source: Time limits                                     │
└─────────────────────────────────────────────────────────┘
```

### 11.2 Source Map Translation (Multi-Stage Lookup)

**The Challenge:**

```
User writes Oxur code:
  test.ox:
    (defn square [x]
      (+ x y))  ; <-- ERROR: y is undefined (line 2, column 8)

After compilation pipeline:
  Surface Forms → Core Forms → Oxur AST → syn AST → Rust source

  Generated lib.rs:
    fn square(x: i32) -> i32 {
        /* oxur_node=300 */ x + /* oxur_node=301 */ y
    }                                            ^
                                                 |
                            rustc error at lib.rs:47:51

We need to map:
  lib.rs:47:51 → Rust NodeId(301) → Core NodeId(201)
    → Surface NodeId(101) → test.ox:2:8
```

**The Solution (Multi-Stage):**

#### Stage 1: Parse - Record Surface Positions

```rust
// oxur-lang/src/parser.rs
pub fn parse_lisp(source: &str, source_map: &mut SourceMap)
    -> Result<SurfaceForms>
{
    // For each parsed token/node
    let node_id = NodeId::new();  // e.g., NodeId(101)

    source_map.record_surface_node(node_id, SourcePos {
        file: "test.ox".to_string(),
        line: 2,
        column: 8,
        length: 1,  // Single character 'y'
    });

    // Build Surface Form with this NodeId
    SurfaceForm::Variable {
        name: "y".to_string(),
        node_id,  // NodeId(101)
    }
}

// SourceMap state after parsing:
// surface_positions: {
//     NodeId(101) => SourcePos { file: "test.ox", line: 2, col: 8, len: 1 }
// }
```

#### Stage 2: Expand - Record Transformations

```rust
// oxur-lang/src/expander.rs
pub fn expand(surface: SurfaceForms, source_map: &mut SourceMap)
    -> Result<CoreForms>
{
    // Transform Surface to Core, creating new nodes
    let core_node_id = NodeId::new();  // e.g., NodeId(201)

    // Record the transformation
    source_map.record_expansion(
        NodeId(101),  // Surface node
        NodeId(201),  // Core node
    );

    // Build Core Form
    CoreForm::Variable {
        name: "y".to_string(),
        node_id: core_node_id,  // NodeId(201)
    }
}

// SourceMap state after expansion:
// surface_positions: { NodeId(101) => SourcePos(...) }
// surface_to_core: { NodeId(101) => NodeId(201) }
```

#### Stage 3: Lower - Record More Transformations

```rust
// oxur-comp/src/lower.rs
pub fn lower(core: &CoreForm, source_map: &mut SourceMap)
    -> Result<RustAst>
{
    match core {
        CoreForm::Variable { name, node_id } => {
            // Create Rust AST node
            let rust_node_id = NodeId::new();  // e.g., NodeId(301)

            // Record transformation
            source_map.record_lowering(
                *node_id,      // Core NodeId(201)
                rust_node_id,  // Rust NodeId(301)
            );

            // Create syn::Ident with attribute
            syn::Ident {
                name: name.clone(),
                attrs: vec![
                    syn::Attribute {
                        path: syn::Path::from("oxur_node"),
                        tokens: quote!(= #rust_node_id),
                    }
                ]
            }
        }
    }
}

// SourceMap state after lowering:
// surface_positions: { NodeId(101) => SourcePos(...) }
// surface_to_core: { NodeId(101) => NodeId(201) }
// core_to_rust: { NodeId(201) => NodeId(301) }
```

#### Stage 4: Generate - Preserve NodeIds as Comments

```rust
// RustAstWrapper generates, oxur-ast prints
let generated_source = oxur_ast::print_rust(&wrapped_ast);

// Result:
fn square(x: i32) -> i32 {
    /* oxur_node=300 */ x + /* oxur_node=301 */ y
}

// NodeIds embedded as comments, survive compilation
```

#### Stage 5: Error Translation - Lookup Original Position

```rust
// CachedCompiler catches rustc error
impl CachedCompiler {
    fn translate_error(
        &self,
        cargo_error: CargoError,
        source_map: &SourceMap,
    ) -> Error {
        // 1. Parse rustc JSON error
        let rustc_error: RustcDiagnostic =
            serde_json::from_str(&cargo_error.message)?;

        // rustc_error = {
        //   "message": "cannot find value `y` in this scope",
        //   "spans": [{
        //     "file_name": "lib.rs",
        //     "line_start": 47,
        //     "line_end": 47,
        //     "column_start": 51,
        //     "column_end": 52,
        //   }]
        // }

        // 2. Read source line
        let line = read_source_line(
            "lib.rs",
            rustc_error.spans[0].line_start
        )?;
        // line = "    /* oxur_node=300 */ x + /* oxur_node=301 */ y"

        // 3. Extract NodeId near error column
        let node_id = extract_node_id_at_column(
            &line,
            rustc_error.spans[0].column_start
        )?;
        // node_id = NodeId(301) (closest comment to column 51)

        // 4. MULTI-STAGE LOOKUP via SourceMap
        let original_pos = source_map.lookup(node_id)?;

        // source_map.lookup(NodeId(301)) internally does:
        //   Step 1: Rust → Core
        //     core_to_rust.find(|(_, &r)| r == NodeId(301))
        //     => NodeId(201)
        //
        //   Step 2: Core → Surface
        //     surface_to_core.find(|(_, &c)| c == NodeId(201))
        //     => NodeId(101)
        //
        //   Step 3: Surface → Position
        //     surface_positions.get(NodeId(101))
        //     => SourcePos { file: "test.ox", line: 2, col: 8 }

        // 5. Construct Oxur error
        Error::Compile {
            message: rustc_error.message,
            position: original_pos,  // test.ox:2:8
            code: rustc_error.code,
            level: rustc_error.level,
        }
    }
}
```

**Implementation of Multi-Stage Lookup:**

```rust
// oxur-smap/src/lib.rs
impl SourceMap {
    pub fn lookup(&self, rust_node: NodeId) -> Option<SourcePos> {
        // Step 1: Rust NodeId → Core NodeId
        let core_node = self.core_to_rust
            .iter()
            .find(|(_, &rust)| rust == rust_node)
            .map(|(core, _)| core)?;

        // Step 2: Core NodeId → Surface NodeId
        let surface_node = self.surface_to_core
            .iter()
            .find(|(_, &core)| core == *core_node)
            .map(|(surface, _)| surface)?;

        // Step 3: Surface NodeId → SourcePos
        self.surface_positions.get(surface_node).cloned()
    }
}
```

### 11.3 Error Display with ariadne

**Beautiful Error Formatting:**

```rust
use ariadne::{Report, ReportKind, Label, Source};

fn display_error(error: &Error, source: &str) {
    let report = Report::build(ReportKind::Error, &error.position.file, 0)
        .with_code(&error.code)
        .with_message(&error.message)
        .with_label(
            Label::new((
                &error.position.file,
                error.position.column as usize
                    ..error.position.column as usize + error.position.length as usize
            ))
            .with_message("cannot find value in this scope")
        );

    report.finish().print(Source::from(source)).unwrap();
}
```

**Output:**

```
Error[E0425]: cannot find value `y` in this scope
  ┌─ test.ox:2:8
  │
2 │   (+ x y))
  │        ^ cannot find value in this scope
  │
```

### 11.4 Edge Cases and Fallbacks

**Case 1: NodeId Comment Missing**

```rust
// If oxur-ast didn't preserve comment:
fn square(x: i32) -> i32 {
    x + y  // No comment!
}

// Fallback: Report error at generated position
Error::Compile {
    message: "cannot find value `y` (in generated code at lib.rs:47:9)",
    position: SourcePos {
        file: "<generated>".to_string(),
        line: 0,
        column: 0,
        length: 0,
    },
    // ...
}
```

**Case 2: NodeId Not in SourceMap**

```rust
// If lookup fails (mapping not recorded):
match source_map.lookup(rust_node_id) {
    Some(pos) => Error::Compile {
        message: rustc_error.message,
        position: pos,
        // ...
    },
    None => Error::Compile {
        message: format!(
            "{} (source map lookup failed for NodeId({}))",
            rustc_error.message,
            rust_node_id.0
        ),
        position: default_position(),
        // ...
    }
}
```

**Case 3: Multiple Errors**

```rust
// rustc may report multiple errors
let translated_errors: Vec<Error> = rustc_diagnostics
    .iter()
    .filter_map(|diag| {
        self.translate_error(diag, source_map).ok()
    })
    .collect();

// Return first error, log others
if let Some(first) = translated_errors.first() {
    for other in &translated_errors[1..] {
        log::warn!("Additional error: {}", other);
    }
    Err(first.clone())
} else {
    // All translations failed
    Err(Error::CompilationFailed {
        message: "Multiple compilation errors (translation failed)".to_string(),
    })
}
```

### 11.5 Runtime Errors (Subprocess)

**Subprocess Protocol for Errors:**

```rust
// SubprocessExecutor communicates via stdin/stdout

// On runtime error in subprocess:
impl Runtime {
    fn handle_panic(&self, panic_info: &PanicInfo) {
        // Send error via stdout
        println!(
            "OXUR_RUNTIME_ERROR: {}",
            panic_info.to_string()
        );

        // Optionally: Include stack trace
        if let Some(location) = panic_info.location() {
            println!(
                "OXUR_PANIC_LOCATION: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
    }
}

// SubprocessExecutor receives and parses:
impl SubprocessExecutor {
    fn execute(&mut self, lib_path: &Path, fn_name: &str)
        -> Result<Response>
    {
        // Send command
        writeln!(
            self.stdin,
            "LOAD_AND_RUN {} {}",
            lib_path.display(),
            fn_name
        )?;

        // Read response
        let mut response = String::new();
        self.stdout.read_line(&mut response)?;

        // Parse protocol message
        if response.starts_with("OXUR_EXECUTION_COMPLETE") {
            Ok(Response::success())
        } else if response.starts_with("OXUR_RUNTIME_ERROR:") {
            let msg = response
                .strip_prefix("OXUR_RUNTIME_ERROR:")
                .unwrap()
                .trim();

            Err(Error::Runtime {
                message: msg.to_string(),
                // Runtime errors don't have source positions
                // (they occur after compilation)
            })
        } else {
            Err(Error::ProtocolViolation {
                message: format!("Unexpected response: {}", response),
            })
        }
    }
}
```

**Limitation:**

Runtime errors (panics, overflows, etc.) occur AFTER compilation, so SourceMap can't help. We can only report:

- The error message
- Stack trace (if available)
- No source position mapping

**Possible Enhancement (v1.1+):**

Map Rust stack trace frames back to Oxur source:

```rust
thread 'main' panicked at 'index out of bounds', lib.rs:42:5
  lib.rs:42:5  /* oxur_node=567 */
    => test.ox:15:12  (via SourceMap lookup)
```

### 11.6 Error Flow Diagram

```
┌────────────────────────────────────────────────────┐
│ User Input: "(+ x y)"                              │
└─────────────────────┬──────────────────────────────┘
                      ↓
              ┌────────────────┐
              │ Parse (Stage 1)│
              │ Records:       │
              │ NodeId → Pos   │
              └───────┬────────┘
                      ↓
              ┌────────────────┐
              │Expand (Stage 2)│
              │ Records:       │
              │ Surface → Core │
              └───────┬────────┘
                      ↓
              ┌────────────────┐
              │Lower (Stage 4) │
              │ Records:       │
              │Core → Oxr AST  │
              │→ syn           │
              └───────┬────────┘
                      ↓
              ┌─────────────────┐
              │Generate(Stage 6)│
              │ Embeds NodeIds  │
              │ as comments     │
              └───────┬─────────┘
                      ↓
              ┌────────────────┐
              │Compile(Stage 8)│
              │ cargo build    │
              └───────┬────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
        ↓                           ↓
   ┌─────────┐              ┌──────────────┐
   │ SUCCESS │              │    ERROR     │
   └─────────┘              └──────┬───────┘
                                   ↓
                         ┌─────────────────┐
                         │ Parse rustc JSON│
                         │ Extract position│
                         └────────┬────────┘
                                  ↓
                         ┌─────────────────┐
                         │Read source line │
                         │Extract NodeId   │
                         └────────┬────────┘
                                  ↓
                         ┌──────────────────┐
                         │SourceMap.lookup  │
                         │ Rust→Core→Surface│
                         │ →SourcePos       │
                         └────────┬─────────┘
                                  ↓
                         ┌─────────────────┐
                         │ Format with     │
                         │ ariadne         │
                         │ Beautiful error!│
                         └─────────────────┘
```

### 11.7 Testing Error Translation

**Unit Test Example:**

```rust
#[test]
fn test_error_translation_undefined_variable() {
    // Setup
    let mut source_map = SourceMap::new();

    // Simulate parse stage
    let surface_node = NodeId::new();
    source_map.record_surface_node(surface_node, SourcePos {
        file: "test.ox".to_string(),
        line: 2,
        column: 8,
        length: 1,
    });

    // Simulate expand stage
    let core_node = NodeId::new();
    source_map.record_expansion(surface_node, core_node);

    // Simulate lower stage
    let rust_node = NodeId::new();
    source_map.record_lowering(core_node, rust_node);

    // Lookup
    let result = source_map.lookup(rust_node).unwrap();

    // Verify
    assert_eq!(result.file, "test.ox");
    assert_eq!(result.line, 2);
    assert_eq!(result.column, 8);
}
```

**Integration Test:**

```rust
#[test]
fn test_full_error_pipeline() {
    // Real Oxur code with error
    let code = r#"
        (defn broken []
          (+ x y))  ; y is undefined
    "#;

    // Run through pipeline
    let mut source_map = SourceMap::new();
    let surface = oxur_lang::parse_lisp(code, &mut source_map).unwrap();
    let core = oxur_lang::expand(surface, &mut source_map).unwrap();
    let rust = oxur_comp::lower(&core, &mut source_map).unwrap();

    // Attempt compilation (will fail)
    let result = compile_and_translate(rust, &source_map);

    // Verify error points to original Oxur source
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.position.file, "test.ox");
    assert_eq!(error.position.line, 3);  // Line with (+ x y)
    assert!(error.message.contains("cannot find value `y`"));
}
```

---

**Summary:**

Error translation is the **killer feature** of Oxur's source mapping. By tracking transformations across multiple stages (Surface → Core → Oxur AST → syn), we can provide rustc-quality error messages that point to the original Oxur source code, not generated Rust.

This is what makes Oxur different from other Lisps - errors that feel like a native compiler, not a translation layer.

---

## 12. Deployment Topology

### 12.1 Single-Server Deployment

```
┌──────────────────────────────────────────────────────────┐
│                    Developer Machine                     │
│                                                          │
│  ┌────────────────┐            ┌──────────────────────┐  │
│  │ Editor/IDE     │            │  oxur-repl-server    │  │
│  │ (Emacs, VSCode)│────────────│  (listening on       │  │
│  │                │  TCP       │   localhost:7888)    │  │
│  │  ReplClient    │  Postcard  │                      │  │
│  └────────────────┘            │  - SessionManager    │  │
│                                │  - Multiple sessions │  │
│                                │  - Subprocesses      │  │
│                                └──────────────────────┘  │
│                                                          │
│  Temp filesystem: /tmp/oxur-repl/session-*/              │
└──────────────────────────────────────────────────────────┘
```

**Use Case:** Local development, single user

**Characteristics:**

- One server process
- Multiple client connections (different buffers in editor)
- Multiple sessions per server
- All on localhost

---

### 12.2 Multi-User Server Deployment

```
┌────────────────┐       ┌────────────────┐       ┌────────────────┐
│ Developer 1    │       │ Developer 2    │       │ Developer 3    │
│ (ReplClient)   │       │ (ReplClient)   │       │ (ReplClient)   │
└────────┬───────┘       └────────┬───────┘       └────────┬───────┘
         │ TCP                    │ TCP                    │ TCP
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                                  ↓
                    ┌─────────────────────────────┐
                    │   Shared Server Machine     │
                    │                             │
                    │  oxur-repl-server           │
                    │  (listening on 0.0.0.0:7888)│
                    │                             │
                    │  - SessionManager           │
                    │  - Session 1 (Dev 1)        │
                    │  - Session 2 (Dev 2)        │
                    │  - Session 3 (Dev 3)        │
                    │  - Resource limits enforced │
                    │                             │
                    │  /tmp/oxur-repl/            │
                    │    session-{dev1}/          │
                    │    session-{dev2}/          │
                    │    session-{dev3}/          │
                    └─────────────────────────────┘
```

**Use Case:** Team development, shared server

**Characteristics:**

- One server process
- Multiple remote clients
- Session isolation per user/project
- Resource limits (max sessions, disk, memory)
- Authentication/authorization (future: v1.1)

**Security Considerations (v1.1):**

- Add authentication (API keys, OAuth)
- Network encryption (TLS)
- Resource quotas per user
- Sandboxing (seccomp, containers)

---

### 12.3 Kubernetes/Container Deployment (Future)

```
┌──────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │            Load Balancer / Ingress             │  │
│  └──────────────────┬─────────────────────────────┘  │
│                     │                                │
│       ┌─────────────┼────────────┬───────────┐       │
│       ↓             ↓            ↓           ↓       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │ Pod 1   │  │ Pod 2   │  │ Pod 3   │  │ Pod N   │  │
│  │         │  │         │  │         │  │         │  │
│  │ REPL    │  │ REPL    │  │ REPL    │  │ REPL    │  │
│  │ Server  │  │ Server  │  │ Server  │  │ Server  │  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  │
│                                                      │
│  Session affinity: Required (sticky sessions)        │
│  Persistent volumes: For /tmp/oxur-repl/ (optional)  │
└──────────────────────────────────────────────────────┘
```

**Use Case:** Large-scale deployment, high availability

**Requirements:**

- Session affinity (client must reconnect to same pod)
- Or: Distributed session store (Redis, etc.) - v2.0
- Persistent temp storage or ephemeral acceptable
- Resource limits per pod

---

## 13. Performance Considerations

This section documents performance-critical design decisions informed by evcxr research and benchmarking.

### 13.1 Artifact Caching Strategy

**Background:** evcxr waited 5 years to add caching (commit 86d20a2, 2023-10-20). When added, it was described as "major performance improvement" - the biggest win in project history.

**For Oxur:** Caching is mandatory from day one.

#### Cache Architecture

```rust
// oxur-repl/src/cache.rs

pub struct ArtifactCache {
    cache_dir: PathBuf,      // Platform-appropriate location
    index: HashMap<String, CachedArtifact>,
    config: CacheConfig,
}

pub struct CacheConfig {
    max_size_mb: usize,      // Total cache size limit
    max_entries: usize,      // Max number of cached artifacts
    eviction: EvictionPolicy, // LRU, LFU, or time-based
}
```

#### Cache Key Generation

**Content-based hashing ensures deterministic cache hits:**

```rust
pub fn cache_key(
    source: &str,              // Generated Rust source
    deps: &[Dependency],       // External crates
    opt_level: OptLevel,       // Debug vs Release
    source_map: &SourceMap,    // Include mapping in key
) -> String {
    let mut hasher = Sha256::new();

    // Source code (after all transformations)
    hasher.update(source.as_bytes());

    // Dependencies affect compilation
    for dep in deps {
        hasher.update(dep.name.as_bytes());
        hasher.update(dep.version.as_bytes());
    }

    // Optimization level changes output
    hasher.update(&[opt_level as u8]);

    // Source map affects generated code (comments)
    hasher.update(source_map.content_hash().as_bytes());

    format!("{:x}", hasher.finalize())
}
```

#### Cache Locations (Platform-Appropriate)

```rust
fn cache_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("~/.cache"))
            .join("oxur")
            .join("artifacts")
    }

    #[cfg(target_os = "macos")]
    {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("~/Library/Caches"))
            .join("oxur")
            .join("artifacts")
    }

    #[cfg(target_os = "windows")]
    {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("%LOCALAPPDATA%"))
            .join("oxur")
            .join("cache")
            .join("artifacts")
    }
}
```

#### Performance Impact

| Scenario | Cache Miss | Cache Hit | Speedup |
|----------|-----------|-----------|---------|
| Simple expr | 50-100ms | 1-2ms | 25-100x |
| Complex function | 200-300ms | 2-5ms | 40-150x |
| Large module | 500-1000ms | 5-10ms | 50-200x |

**Key Insight:** Cache hits are 50-200x faster than compilation.

#### Eviction Strategy

```rust
impl ArtifactCache {
    pub fn evict_if_needed(&mut self) {
        // Check size limit
        let total_size = self.total_size_mb();
        if total_size > self.config.max_size_mb {
            self.evict_lru(self.config.max_entries / 2);
        }

        // Check entry count limit
        if self.index.len() > self.config.max_entries {
            self.evict_lru(self.config.max_entries / 2);
        }
    }

    fn evict_lru(&mut self, keep_n: usize) {
        // Sort by last access time
        let mut entries: Vec<_> = self.index.iter()
            .map(|(k, v)| (k.clone(), v.last_accessed))
            .collect();
        entries.sort_by_key(|(_, time)| *time);

        // Remove oldest entries
        for (key, _) in entries.iter().skip(keep_n) {
            if let Some(entry) = self.index.remove(key) {
                fs::remove_file(&entry.path).ok();
            }
        }
    }
}
```

**Default Configuration:**

- Max size: 1GB
- Max entries: 1000
- Eviction: LRU (keep most recently used)

---

### 13.2 Temp Directory Performance

**Filesystem I/O is <3% of total compilation time.**

#### Breakdown (typical cold compilation)

```
┌─────────────────────────────────────────┐
│ COMPILATION TIME BREAKDOWN              │
├─────────────────────────────────────────┤
│ Parse Oxur code:        ~1ms   (0.4%)   │
│ Expand macros:          ~2ms   (0.9%)   │
│ Lower to Rust:          ~5ms   (2.3%)   │
│ Write lib.rs:           ~0.02ms (0.01%) │ ← Filesystem
│ Spawn cargo:            ~10ms  (4.5%)   │
│ rustc compile:          ~200ms (90%)    │ ← Dominates!
│ Write dylib:            ~1ms   (0.4%)   │ ← Filesystem
│ Load dylib:             ~5ms   (2.3%)   │ ← Filesystem
├─────────────────────────────────────────┤
│ TOTAL:                  ~224ms          │
│ Filesystem I/O:         ~6ms   (2.7%)   │
└─────────────────────────────────────────┘
```

**Key Insight:** Even eliminating ALL filesystem I/O saves <3% total time.

#### tmpfs Optimization (Best Effort)

**Linux:** Automatic tmpfs gives ~2-3% improvement (free!)

```rust
fn get_repl_temp_root() -> PathBuf {
    // Try /dev/shm (RAM-backed) on Linux
    #[cfg(target_os = "linux")]
    {
        let shm = PathBuf::from("/dev/shm");
        if shm.exists() && shm.is_dir() {
            return shm.join("oxur-repl");
        }
    }

    // Fallback to system temp (OS will cache hot files anyway)
    env::temp_dir().join("oxur-repl")
}
```

**macOS/Windows:** OS caching makes fallback acceptable.

**User Override:** `OXUR_REPL_TEMP_DIR` for power users.

**Decision:** Best-effort tmpfs is elegant zero-config optimization.

---

### 13.3 Panic Handling Trade-offs

**Finding from evcxr (commit a144454, 2019):**

> "Performance bottlenecks aren't always where you expect. The overhead wasn't in catching panics at runtime, but in the code generation the compiler had to do."

#### The Cost of Panic Preservation

When panic preservation is ENABLED:

```rust
// Generated code wraps execution in panic::catch_unwind
let result = panic::catch_unwind(|| {
    user_code()
});
```

**Impact:**

- Compilation overhead: ~10-20% slower
- Runtime overhead: Negligible (<1ms)
- Code size: ~15% larger

**Why Compilation is Slower:**

- Compiler must generate unwind tables
- More complex control flow
- Additional type information tracking

#### Configuration

```rust
pub struct ReplConfig {
    preserve_vars_on_panic: bool,  // Default: false
    optimization_level: OptLevel,  // Default: Debug
}
```

**Recommended Defaults:**

- Development REPL: panic preservation OFF (faster iteration)
- Production scripts: panic preservation ON (safer)

**User Control:**

```rust
// In REPL
> (set-option :preserve-vars-on-panic true)
> (set-option :optimization-level :release)
```

---

### 13.4 Three-Tier Execution Performance

**The three tiers have dramatically different performance characteristics:**

#### Tier 1: Calculator (Instant)

```lisp
> (+ 2 3)
5
```

**Performance:** <1ms

- No compilation
- Pure calculation in eval_context
- Pattern: Simple arithmetic, constants

**When to Use:**

- Single arithmetic operation
- No variables or functions
- Immediate result expected

#### Tier 2: Cached (Fast)

```lisp
> (defn square [x] (* x x))  ; First time: compiles
> (square 5)                  ; Subsequent: cached
25
```

**Performance:** 1-5ms

- Cache hit: Load existing dylib
- No compilation needed
- Pattern: Previously evaluated code

**When to Use:**

- Function already defined
- Code hasn't changed
- Cache key matches

#### Tier 3: JIT (Slower)

```lisp
> (defn complex-logic [...]  ; First time, complex code
    (loop [...] ...))
```

**Performance:** 50-300ms

- Full compilation pipeline
- cargo invocation
- rustc optimization
- Pattern: New code, complex logic

**When to Use:**

- First-time evaluation
- Code modified
- Cache miss

#### Decision Logic

```rust
fn decide_tier(&self, form: &CoreForm) -> Tier {
    // Tier 1: Pure calculation
    if form.is_simple_arithmetic() {
        return Tier::Calculator;
    }

    // Tier 2: Check cache
    let cache_key = self.cache.cache_key(&form);
    if self.cache.get(&cache_key).is_some() {
        return Tier::Cached;
    }

    // Tier 3: Must compile
    Tier::JIT
}
```

**Performance Expectations:**

| User Action | Typical Tier | Latency | User Experience |
|-------------|--------------|---------|-----------------|
| `(+ 1 2)` | Calculator | <1ms | Instant |
| `(square 5)` (defined) | Cached | 1-5ms | Imperceptible |
| `(defn foo ...)` (new) | JIT | 50-300ms | Perceptible pause |

**Key Insight:** Caching makes Tier 2 feel like an interpreter (1-5ms) while giving Tier 3 native performance.

---

### 13.5 Subprocess Overhead

#### Process Spawn Cost

**One-time cost per session:**

```
Subprocess spawn:           ~10-20ms
Initial variable setup:     ~1-2ms
Total session startup:      ~12-22ms
```

**Amortized:** This cost is paid once per session, not per eval.

#### IPC Latency (stdin/stdout)

**Per-eval overhead:**

```
Write command to stdin:     ~50-100μs
Read response from stdout:  ~50-100μs
Total IPC overhead:         ~100-200μs
```

**Compared to compilation:**

- Compilation: 50-300ms
- IPC overhead: 0.1-0.2ms (0.04-0.4% of total)

**Verdict:** IPC overhead is negligible.

#### Why stdin/stdout is Fast Enough

From evcxr 6-year experience:

- Simple text protocol
- No serialization overhead for commands
- Buffered I/O efficient
- Kernel optimizes pipe throughput

**Theoretical Unix socket improvement:**

- Potential: 10-20% faster IPC
- Actual impact: 0.04% → 0.03% of total time
- **Not worth the complexity**

---

### 13.6 Optimization Levels

#### Debug vs Release Builds

```rust
pub enum OptLevel {
    Debug,    // -C opt-level=0 (default for REPL)
    Release,  // -C opt-level=3 (for production scripts)
}
```

**Performance Comparison:**

| Optimization | Compile Time | Runtime Speed | Use Case |
|--------------|--------------|---------------|----------|
| Debug | 50-100ms | 1x (baseline) | Interactive REPL |
| Release | 200-500ms | 2-10x faster | Production scripts |

**Decision:** Default to Debug for REPL (faster iteration), allow user to request Release.

**User Control:**

```lisp
> (set-optimization-level :release)
> (defn compute-intensive [...] ...)  ; Compiled with -O3
```

#### LTO (Link-Time Optimization)

**NOT recommended for REPL:**

- Compilation time: +50-100%
- Runtime improvement: 10-20%
- Trade-off not worth it for interactive use

**Possible for final build:**

```lisp
> (compile-to-binary "myapp.ox" :lto true)
```

---

### 13.7 Performance Monitoring

#### Built-in Metrics

```rust
pub struct PerformanceMetrics {
    pub parse_time: Duration,
    pub expand_time: Duration,
    pub lower_time: Duration,
    pub compile_time: Duration,
    pub load_time: Duration,
    pub execute_time: Duration,
    pub total_time: Duration,
    pub cache_hit: bool,
    pub tier: Tier,
}
```

**User Access:**

```lisp
> (set-option :show-timings true)
> (defn foo [] 42)
Parsed:    1ms
Expanded:  2ms
Lowered:   5ms
Compiled:  87ms
Loaded:    3ms
Executed:  <1ms
Total:     98ms (JIT, cache miss)
```

#### Profiling Infrastructure

```rust
#[cfg(feature = "profiling")]
impl CachedCompiler {
    fn profile_eval(&mut self, form: CoreForm) -> Result<Response> {
        let mut metrics = PerformanceMetrics::default();

        let start = Instant::now();
        // ... each stage records its duration
        metrics.total_time = start.elapsed();

        self.emit_metrics(metrics);
    }
}
```

---

### 13.8 Performance Best Practices

#### For Users

1. **Define functions once, call many times** (leverage Tier 2 caching)
2. **Use simple arithmetic directly** (Tier 1 is instant)
3. **Enable Release mode for compute-intensive code**
4. **Monitor with `:show-timings` to identify bottlenecks**

#### For Implementers

1. **Profile before optimizing** (don't assume where bottlenecks are)
2. **Cache aggressively** (biggest performance win)
3. **Start with Debug optimization** (faster iteration)
4. **Use tmpfs where available** (free 2-3% improvement)
5. **Avoid complex IPC** (stdin/stdout is fast enough)

---

### 13.9 Known Performance Limitations

#### 1. First-Time Compilation

**Unavoidable:** First eval of any code requires full compilation (50-300ms).

**Mitigation:**

- Precompile standard library
- Cache commonly used functions
- Provide progress indication for long compiles

#### 2. Struct Redefinition

**evcxr Limitation (applies to Oxur):**

Cannot redefine structs in same session:

```lisp
> (defstruct Point [x y])
> (defstruct Point [x y z])  ; ERROR: Point already defined
```

**Reason:** Compiled code contains type information that can't be invalidated.

**Workaround:** Restart session or use different struct name.

#### 3. Cold Cache Performance

**First session of the day:** No cached artifacts, everything compiles from scratch.

**Typical experience:**

- First few evals: 50-300ms each
- After warming cache: 1-5ms typical

**Mitigation:**

- Persistent cache survives REPL restarts
- Precompile standard library on first install

#### 4. Memory Growth

**Subprocess accumulates memory:**

- Each eval loads a new dylib
- Previous dylibs remain loaded
- Memory grows over session lifetime

**Mitigation:**

- Subprocess restart on memory threshold
- User can force restart: `(restart-subprocess)`

---

### 13.10 Future Optimizations (v1.1+)

#### Potential Improvements

1. **Incremental Compilation**
   - Only recompile changed functions
   - Requires tracking dependencies
   - Possible 30-50% speedup for large sessions

2. **Precompiled Standard Library**
   - Common functions already compiled
   - Zero overhead for std library calls
   - Reduces cold-start time

3. **Unix Socket IPC**
   - Binary protocol instead of text
   - Potential 10-20% IPC improvement
   - Negligible overall impact (<0.1%)

4. **Parallel Compilation**
   - Compile multiple functions simultaneously
   - Requires dependency analysis
   - Possible 20-40% speedup for batch loads

**Priority:** Focus on caching (biggest win) before exotic optimizations.

---

### 13.11 Summary: Performance Principles

1. **Caching is King** - 50-200x speedup, must have from day one
2. **Compilation Dominates** - 90% of time is rustc, optimize cache hits instead
3. **Simple IPC is Fast Enough** - stdin/stdout proven for 6 years
4. **tmpfs is Free Optimization** - 2-3% improvement with zero config
5. **Profile Before Optimizing** - Bottlenecks aren't always where you expect
6. **Three Tiers Matter** - Calculator/Cached/JIT have different UX implications

**The Bottom Line:**

> A well-cached REPL (Tier 2) feels like an interpreter (1-5ms) while providing native performance (Tier 3). This is the key to Oxur's competitive advantage.

---

## 14. Conclusion

This architecture provides:

- ✅ **Clear separation of concerns** - Client handles protocol, server handles execution
- ✅ **Subprocess isolation** - User code crashes don't corrupt REPL state
- ✅ **Session-based architecture** - Multiple concurrent users, isolated state
- ✅ **Three-tier execution** - Optimize for common cases (calculator mode)
- ✅ **Complete compilation pipeline** - Oxur → Core Forms → Oxur AST → Rust → Execution
- ✅ **Error translation** - Source maps enable Oxur-level error messages
- ✅ **Scalable design** - Supports single-user and multi-user deployments
- ✅ **Well-defined integration points** - Clear APIs with oxur-lang, oxur-comp, oxur-ast
- ✅ **Resource management** - Limits, timeouts, cleanup strategies
- ✅ **Proven patterns** - Based on evcxr's battle-tested approach

### Next Steps

1. **Verify external crate APIs** - Confirm oxur-lang, oxur-comp, oxur-ast provide required functions
2. **Implement foundation components** - VariableStore, SessionDir, Subprocess runtime
3. **Build integration layer** - CodeGenerator, source maps, error translation
4. **Implement CachedCompiler** - Core compilation engine
5. **Test end-to-end** - Simple eval, complex eval, error cases

---

## 15. Version History

### Version 1.4 (2026-01-10)

Updated compilation pipeline descriptions to reflect ODD-0013's improved clarity about the Oxur AST buffer zone.

**Changes:**

1. **Oxur AST Intermediate Layer Clarified**
   - Updated all pipeline flow descriptions to mention Oxur AST
   - Stage 4 (Lower) now explicitly notes it crosses semantic boundary via Oxur AST
   - Pipeline flows updated from "Core Forms → Rust AST" to "Core Forms → Oxur AST → syn AST"

2. **Semantic Boundary Documentation**
   - Added explanation that Oxur AST is where we cross from Lisp to Rust concepts
   - Noted current implementation combines Stages 3+4 per ODD-0013
   - Clarified Oxur AST acts as buffer zone protecting from changes in both directions

3. **Pipeline References Updated Throughout**
   - Section 2.2: oxur-comp purpose now mentions Oxur AST intermediate layer
   - Section 3.1: Stage 4 includes detailed note about internal operations
   - Section 11: Error translation description updated to include Oxur AST stage
   - All SourceMap tracking references updated: "Surface → Core → Oxur AST → syn"

4. **Examples Enhanced**
   - Stage 4 examples now show both Oxur AST and syn AST representations
   - `(define-func ...) → (Item :kind (Fn ...)) → syn::ItemFn`
   - Makes the transformation clearer for implementers

**Impact:** Documentation clarity only - no architectural changes, aligns with ODD-0013 v1.2

---

### Version 1.3 (2026-01-07)

Added CLI integration architecture for `oxur repl` command.

**Changes:**

1. **New Section 1.5: CLI Integration**
   - Documents `oxur repl` command-line interface
   - Flags: `-i/--interactive`, `-c/--connect`, `-s/--serve`, `--ack`, `-t/--transport`, `--no-color`
   - Default behavior: in-memory client/server via InProcessTransport
   - Architecture diagrams for interactive, server, and connect modes

2. **InProcessTransport Architecture**
   - Zero-copy message passing via Tokio channels
   - Fastest possible client-server communication for default mode
   - No serialization overhead for local REPL usage

3. **Server Mode Architecture**
   - TCP and Unix domain socket support
   - Multi-session support with session isolation
   - ACK protocol for editor/tooling integration (nREPL-style)

4. **Terminal Interface**
   - rustyline/reedline for line editing
   - Persistent command history across sessions
   - Ctrl-C (interrupt) and Ctrl-D (exit) handling

**Impact:** Architecture addition - defines how users interact with the REPL via CLI

---

### Version 1.2 (2026-01-05)

Documentation cleanup and branding consistency.

**Changes:**

1. **Subprocess Location Clarified**
   - WAS: References to `oxur-subprocess/` as separate crate
   - NOW: Subprocess is a binary target within `oxur-repl` crate
   - Location: `oxur-repl/src/bin/subprocess.rs`
   - Built as: `oxur-repl-subprocess` binary
   - Added Cargo.toml `[[bin]]` configuration example

2. **Protocol Markers Rebranded**
   - WAS: `EVCXR_EXECUTION_COMPLETE`, `EVCXR_RUNTIME_ERROR`, `EVCXR_PANIC_LOCATION`
   - NOW: `OXUR_EXECUTION_COMPLETE`, `OXUR_RUNTIME_ERROR`, `OXUR_PANIC_LOCATION`
   - Rationale: Oxur-branded protocol, not copying evcxr naming

3. **VariableStore Module Rebranded**
   - WAS: `evcxr_variable_store` in generated code examples
   - NOW: `oxur_variable_store`
   - Added clarification about ABI compatibility between subprocess and generated libraries

4. **Rust Edition Updated**
   - WAS: `edition = "2024"` in Cargo.toml template
   - NOW: `edition = "2021"` (stable, widely supported)

5. **Optimization Level Consistency**
   - Clarified default is `opt-level = 0` for fastest REPL iteration
   - Section 10.2 and Section 13.6 now consistent

6. **Component Locations Clarified**
   - Added explicit file locations for:
     - `Executor` trait: `oxur-repl/src/executor/mod.rs`
     - `TypeInference`: `oxur-repl/src/type_inference.rs`
     - Subprocess runtime: `oxur-repl/src/bin/subprocess.rs`
     - VariableStore: `oxur-repl/src/subprocess/variable_store.rs`

7. **ArtifactCache Thread Safety**
   - Clarified that `ArtifactCache` is wrapped in `Arc<Mutex<...>>` when shared
   - Added `type SharedCache = Arc<Mutex<ArtifactCache>>` example

8. **VariableStore Architecture Clarified**
   - Explained that VariableStore exists in two places:
     1. Subprocess runtime (maintains state)
     2. Generated code (embedded for ABI compatibility)

**Impact:** Documentation only - no architectural changes from v1.1

---

### Version 1.1 (2026-01-04)

Complete architecture finalization based on:

- Session 1 architecture review and decisions
- evcxr comprehensive research (git archaeology + web documentation)
- 6+ years of proven patterns validation

**Major Changes:**

1. **Decision 1: CodeGenerator → RustAstWrapper** (Naming Clarity)
   - Renamed throughout document to clarify responsibility
   - Component wraps already-lowered Rust AST (doesn't do lowering)
   - Location: `oxur-repl/src/wrapper.rs` (was `src/codegen/generator.rs`)

2. **Decision 2: oxur-smap Foundation Crate** (NEW - Phase 0 Prerequisite)
   - Dedicated source mapping crate (no dependencies)
   - Multi-stage tracking: Surface → Core → Oxur AST → syn → Error translation
   - Added to all integration points
   - Unique differentiator (no other Lisp has this)

3. **Decision 3: Subprocess Execution MANDATORY** (MAJOR REVISION)
   - WAS: InProcessExecutor default for v1.0
   - NOW: SubprocessExecutor mandatory (not optional)
   - REASON: Rust threads cannot be interrupted - subprocess required for Ctrl-C
   - evcxr evidence: Subprocess from day one, unchanged 6+ years
   - Executor trait kept for testing only

4. **Decision 3a: IPC Mechanism - stdin/stdout** (SIMPLIFIED)
   - WAS: Unix sockets + protocol reuse
   - NOW: stdin/stdout text protocol
   - REASON: 6 years proven stable in evcxr, simpler implementation
   - Text protocol: `LOAD_AND_RUN <path> <fn>` / `OXUR_EXECUTION_COMPLETE`
   - Unix sockets deferred to v1.1+ (if needed)

5. **Decision 4: Temp Directory Strategy** (Elegant Optimization)
   - Best-effort tmpfs with graceful fallback
   - Linux: `/dev/shm` (RAM-backed, ~2-3% faster)
   - macOS/Windows: OS cache (good enough)
   - User override: `OXUR_REPL_TEMP_DIR` environment variable
   - Zero configuration, works everywhere

6. **Decision 5: Artifact Caching MANDATORY** (NEW - Critical for Performance)
   - WAS: Future consideration
   - NOW: Day-one requirement (Phase 0)
   - REASON: evcxr's biggest regret (waited 5 years)
   - Content-based caching: `~/.cache/oxur/artifacts/`
   - Cache key: SHA256(source + deps + opt_level + source_map)

7. **Decision 6: Type Inference Strategy** (NEW - Avoid 4-Year Hack)
   - Use rust-analyzer from day one
   - evcxr spent 4 years with compiler error hack (2018-2022)
   - Hack removed entirely in commit 5cbc3a0 (2022-08-28)
   - Start with RA as library, consider LSP if build times problematic

8. **Decision 7: Variable Store Constraints** (Documented)
   - `Box<dyn Any + 'static>` requires owned data
   - No inter-variable references possible
   - Aligns with Lisp semantics (immutable data structures)

9. **Decision 8: Panic Handling** (Configurable)
   - Optional panic preservation (default: OFF for performance)
   - evcxr finding: Code generation overhead, not runtime cost

**Component Updates:**

- Added: `ArtifactCache` module (new, Phase 0)
- Added: `TypeInference` module (new, Phase 1)
- Added: `SubprocessProtocol` (stdin/stdout text)
- Renamed: `CodeGenerator` → `RustAstWrapper`
- Removed: `InProcessExecutor` as production option (test-only)

**Architecture Validation:**

- Subprocess execution: ✅ Proven (6+ years, zero fundamental changes)
- stdin/stdout IPC: ✅ Proven (stable, portable, simple)
- Variable storage: ✅ Proven (`Box<dyn Any + 'static>` works)
- Caching: ✅ Critical (major performance win when added)
- Type inference: ✅ rust-analyzer mature (avoid compiler hack)

**Timeline Impact:**

- Phase 0: 3-4 weeks (was 2-3 weeks, +1 week for caching infrastructure)
- Total v1.0: 7-10 weeks (was 6-9 weeks)
- Trade-off: Avoid 5 years of caching regret, 4 years of type inference hacks

### Version 1.0 (2026-01-03)

Initial architecture specification with identified gaps and areas requiring research.

---

**Document Status:** Complete - Ready for review and implementation
