---
number: 38
title: "Oxur REPL Architecture Overview"
author: "user application"
component: All
tags: [change-me]
created: 2026-01-04
updated: 2026-01-04
state: Under Review
supersedes: null
superseded-by: null
version: 1.0
---

# Oxur REPL Architecture Overview

```
Version: 1.0
Date: January 3, 2026
Status: Definitive Reference
Purpose: An initial sketch of a complete architectural specification for Oxur REPL system; in its current form some of the namings, design decisions, etc., are incomplete, actively being researched for better alternatives, or under actual re-design. A v1.1 of this doc will address all of these rough edges.
```

---

## Document Purpose

This document provides the complete architectural picture of the Oxur REPL system, showing how all components fit together across client, server, and external crates. It serves as the single source of truth for understanding:

- How client and server interact via the protocol
- Where each component lives and what it does
- How compilation flows from user input to execution
- How the REPL integrates with the broader Oxur compilation chain
- What APIs are required from external crates

**Target Audience:** Developers implementing or extending the REPL system

**Related Documents:**

- ODD-0013: Oxur Compilation Chain Architecture (compilation pipeline context)
- ODD-0018: Oxur Remote REPL Protocol Design (protocol layer specification)
- ODD-0030: Oxur REPL Implementation Specification (component implementation details)
- ODD-0026: Oxur REPL Evaluation Strategy (three-tier execution strategy)

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
│  │  │ - history: Vec<HistoryEntry>                    │    │  │
│  │  │ - output_buffer: OutputBuffer                   │    │  │
│  │  └─────────────────────────────────────────────────┘    │  │
│  │                                                         │  │
│  │  Core Methods:                                          │  │
│  │  - eval(code: &str) → Result<Value>                     │  │
│  │    * Parses code (via oxur-lang)                        │  │
│  │    * Decides execution tier                             │  │
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
│  │  - subprocess: Option<ChildProcess>                     │  │
│  │  - code_gen: CodeGenerator                              │  │
│  │  - source_map: Arc<SourceMap>                           │  │
│  │                                                         │  │
│  │  Core Method:                                           │  │
│  │  - eval(form: CoreForm) → Result<Response>              │  │
│  │    * Generates Rust code from Core Forms                │  │
│  │    * Invokes cargo to compile                           │  │
│  │    * Loads library into subprocess                      │  │
│  │    * Executes and captures result                       │  │
│  └─────────────────────────────────────────────────────────┘  │
│                               ↓                               │
│                   Subprocess Communication                    │
│                         (stdin/stdout)                        │
│                               ↓                               │
└───────────────────────────────┼───────────────────────────────┘
                                │
                                │
┌───────────────────────────────┴───────────────────────────────┐
│                      SUBPROCESS PROCESS                       │
│                   (oxur-subprocess binary)                    │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                 Subprocess Runtime                      │  │
│  │                                                         │  │
│  │  Components:                                            │  │
│  │  - variable_store: Box<VariableStore>                   │  │
│  │  - libraries: Vec<Library> (loaded dylibs)              │  │
│  │                                                         │  │
│  │  Protocol:                                              │  │
│  │  - Reads commands from stdin                            │  │
│  │  - LOAD <path> <fn_name>                                │  │
│  │  - PING (health check)                                  │  │
│  │  - Writes results to stdout                             │  │
│  │  - "EVCXR_EXECUTION_COMPLETE" marker                    │  │
│  │                                                         │  │
│  │  Execution:                                             │  │
│  │  - Loads compiled dylib via libloading                  │  │
│  │  - Calls exported function with VariableStore pointer   │  │
│  │  - Captures stdout/stderr during execution              │  │
│  │  - Function mutates VariableStore (variable persistence)│  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
└───────────────────────────────────────────────────────────────┘

External Crates (called by server):
┌──────────────────┐
│   oxur-lang      │ - parse_lisp() → SurfaceForms
│                  │ - expand() → CoreForms
│                  │ - parse_core_forms() → CoreForms
└──────────────────┘

┌──────────────────┐
│   oxur-comp      │ - lower() → RustAst
└──────────────────┘

┌──────────────────┐
│   oxur-ast       │ - print_rust() → String (Rust source)
└──────────────────┘
```

### 1.2 Key Architectural Decisions

**Decision 1: Server-Side Compilation**

- All compilation happens on the server
- Client is thin (protocol only)
- Rationale: Session state lives on server, subprocess is server-local, matches evcxr pattern

**Decision 2: One CachedCompiler Per Session**

- CachedCompiler owned by EvalContext
- Each session has isolated resources (temp dir, subprocess, variable store)
- Rationale: Simplifies lifecycle management, natural ownership

**Decision 3: Subprocess Isolation**

- User code executes in separate process
- Prevents crashes from corrupting REPL state
- Enables restart-on-panic without data loss
- Rationale: Safety, proven pattern from evcxr

**Decision 4: Core Forms as Integration Point**

- oxur-lang produces Core Forms
- CachedCompiler consumes Core Forms
- Clean separation between language and REPL
- Rationale: Stable IR, matches compilation chain architecture

---

## 2. Component Inventory

### 2.1 Client Components (oxur-repl crate)

#### ReplClient

**Location:** `oxur-repl/src/client/client.rs`

**Purpose:** Manages connection to REPL server and sends/receives messages

**Responsibilities:**

- Establish TCP connection to server
- Serialize Request messages via Postcard codec
- Deserialize Response messages
- Handle connection errors and reconnection
- Provide async API for evaluations

**Lifecycle:**

- Created by user application
- Connects on first use
- Persists for multiple requests
- Closed explicitly or on drop

**Dependencies:**

- Transport layer (TCP)
- Codec layer (Postcard)
- Protocol types (Request, Response)

**Key Methods:**

```rust
pub async fn connect(addr: SocketAddr) -> Result<Self>;
pub async fn eval(&mut self, code: &str) -> Result<Response>;
pub async fn create_session(&mut self) -> Result<SessionId>;
pub async fn close_session(&mut self, session: SessionId) -> Result<()>;
pub async fn disconnect(&mut self) -> Result<()>;
```

**Does NOT:**

- Parse Oxur code
- Compile code
- Execute code
- Manage sessions (server does this)

---

### 2.2 Server Components (oxur-repl crate)

#### ReplServer

**Location:** `oxur-repl/src/server/server.rs`

**Purpose:** Accepts connections and manages server lifecycle

**Responsibilities:**

- Bind to TCP socket
- Accept incoming connections
- Spawn handler task per connection
- Graceful shutdown coordination
- Server-wide configuration

**Lifecycle:**

- Created with config (transport, codec, limits)
- Binds to socket
- Runs event loop until shutdown
- Cleans up all sessions on shutdown

**Dependencies:**

- TransportListener (TCP, Unix socket, etc.)
- Codec (Postcard)
- SessionManager
- MessageHandler

**Key Methods:**

```rust
pub fn new(config: ServerConfig<T, C>) -> Self;
pub async fn serve(self) -> Result<()>;
pub async fn shutdown(&self) -> Result<()>;
pub fn local_addr(&self) -> Result<String>;
```

---

#### MessageHandler

**Location:** `oxur-repl/src/server/handler.rs`

**Purpose:** Dispatches protocol operations to appropriate handlers

**Responsibilities:**

- Route operations (Eval, Clone, LoadFile, etc.)
- Delegate to SessionManager for session operations
- Construct Response messages
- Handle protocol-level errors
- Convert evaluation errors to protocol errors

**Lifecycle:**

- Created by ReplServer
- Shared across all connections (Arc)
- Lives for server lifetime

**Dependencies:**

- SessionManager (for session access)
- Protocol types (Request, Response, Operation)

**Key Methods:**

```rust
pub async fn handle_request(&self, req: Request) -> Result<Response>;
async fn handle_eval(&self, req: Request) -> Result<Response>;
async fn handle_clone(&self, req: Request) -> Result<Response>;
async fn handle_close(&self, req: Request) -> Result<Response>;
async fn handle_load_file(&self, req: Request) -> Result<Response>;
```

**Operation Dispatch:**

```rust
match req.op {
    Operation::Clone => {
        // Create new session
        let session_id = self.sessions.create_session().await?;
        Response { session: session_id, status: [SessionCreated], ... }
    }
    Operation::Eval => {
        // Get session, evaluate code
        let session = self.sessions.get_session(&req.session).await?;
        let result = session.eval(&req.params["code"]).await?;
        Response { value: result, status: [Done], ... }
    }
    Operation::Close => {
        // Close session
        self.sessions.remove_session(&req.session).await?;
        Response { status: [SessionClosed], ... }
    }
    // ... other operations
}
```

---

#### SessionManager

**Location:** `oxur-repl/src/server/session.rs`

**Purpose:** Creates, tracks, and manages REPL sessions

**Responsibilities:**

- Create new sessions with unique IDs
- Store and retrieve sessions by ID
- Enforce session limits (max sessions per server)
- Enforce session timeouts (idle sessions)
- Thread-safe concurrent access
- Cleanup on shutdown

**Lifecycle:**

- Created by ReplServer
- Shared across all connections (Arc)
- Lives for server lifetime

**State:**

```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, EvalContext>>>,
    max_sessions: usize,
    session_timeout: Duration,
}
```

**Key Methods:**

```rust
pub async fn create_session(&self) -> Result<SessionId>;
pub async fn get_session(&self, id: &SessionId) -> Result<Arc<Mutex<EvalContext>>>;
pub async fn remove_session(&self, id: &SessionId) -> Result<()>;
pub async fn close_all(&self);
pub fn session_count(&self) -> usize;
```

**Concurrency Model:**

- Read lock for lookups (common case)
- Write lock for creation/removal (rare)
- Each EvalContext behind Mutex for evaluation exclusion

---

#### EvalContext

**Location:** `oxur-repl/src/eval/context.rs`

**Purpose:** Manages evaluation state for a single REPL session

**Responsibilities:**

- Parse code via oxur-lang (mode-dependent)
- Decide execution tier (Calculator, Cached, JIT)
- Delegate to CachedCompiler for tier 2/3
- Manage evaluation history
- Capture stdout/stderr output
- Switch between Lisp/Sexpr modes

**Lifecycle:**

- Created by SessionManager
- One per session
- Lives for session duration
- Destroyed when session closes

**State:**

```rust
pub struct EvalContext {
    session_id: SessionId,
    mode: ReplMode,                    // Lisp or Sexpr
    compiler: CachedCompiler,          // Owned!
    history: Vec<HistoryEntry>,
    output_buffer: OutputBuffer,
}
```

**Key Methods:**

```rust
pub async fn eval(&mut self, code: &str) -> Result<Value>;
pub async fn load_file(&mut self, path: &str) -> Result<Value>;
pub fn set_mode(&mut self, mode: ReplMode);
pub fn mode(&self) -> ReplMode;
pub fn take_output(&mut self) -> (String, String);
pub fn record_history(&mut self, code: String, result: Result<Value>);
```

**Evaluation Logic:**

```rust
pub async fn eval(&mut self, code: &str) -> Result<Value> {
    // 1. Parse based on mode
    let core_forms = match self.mode {
        ReplMode::Lisp => {
            let surface = oxur_lang::parse_lisp(code)?;
            oxur_lang::expand(surface)?
        }
        ReplMode::Sexpr => {
            oxur_lang::parse_core_forms(code)?
        }
    };

    // 2. Decide tier
    let tier = self.decide_tier(&core_forms);

    // 3. Execute
    let result = match tier {
        Tier::Calculator => self.eval_calculator(&core_forms),
        Tier::Cached | Tier::Jit => self.compiler.eval(core_forms).await?,
    };

    // 4. Record history
    self.record_history(code.to_string(), result.clone());

    Ok(result)
}
```

**Integration Point:** This is where oxur-lang is called!

---

#### CachedCompiler

**Location:** `oxur-repl/src/compiler/cached.rs`

**Purpose:** Compiles Core Forms to Rust and executes them

**Responsibilities:**

- Generate Rust code from Core Forms
- Invoke cargo to compile to dylib
- Parse cargo JSON output for errors
- Translate rustc errors to Oxur positions (via source maps)
- Load compiled library into subprocess
- Execute code and capture results
- Manage session temporary directory
- Maintain variable state across evaluations

**Lifecycle:**

- Created by EvalContext
- One per session
- Spawns subprocess on first use
- Cleanup on session close

**State:**

```rust
pub struct CachedCompiler {
    session_id: SessionId,
    session_dir: SessionDir,           // Temp directory management
    state: SessionState,               // Variable names/types, eval counter
    subprocess: Option<ChildProcess>,  // Execution environment
    code_gen: CodeGenerator,           // Core Forms → Rust
    source_map: Arc<SourceMap>,        // Error translation
}
```

**Key Methods:**

```rust
pub async fn eval(&mut self, form: CoreForm) -> Result<Response>;
async fn compile_to_dylib(&self, code: &GeneratedCode) -> Result<PathBuf>;
async fn execute(&mut self, lib_path: &Path) -> Result<ExecResult>;
fn translate_errors(&self, rustc_errors: &[CompilerMessage]) -> Result<Vec<OxurError>>;
```

**Evaluation Flow:**

```rust
pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
    // Clone-try-commit pattern
    let mut tentative_state = self.state.clone();
    tentative_state.eval_counter += 1;

    // Generate Rust code
    let code = self.code_gen.generate(&form, &tentative_state)?;

    // Compile
    let artifact = self.compile_to_dylib(&code).await?;

    // Execute
    let result = self.execute(&artifact).await?;

    // Commit state only on success
    self.state = tentative_state;

    Ok(result)
}
```

**Why Clone-Try-Commit?**

- Compilation can fail (syntax errors, type errors)
- Don't want to corrupt state on failure
- Clone is cheap (just variable names/types)
- Subprocess and temp dir are shared (not cloned)

---

#### CodeGenerator

**Location:** `oxur-repl/src/codegen/generator.rs`

**Purpose:** Converts Core Forms to Rust source code

**Responsibilities:**

- Lower Core Forms to Rust AST (via oxur-comp)
- Wrap in function with VariableStore integration
- Add source map comments (`/* oxur_node=N */`)
- Generate variable load/store code
- Emit complete Rust library

**Lifecycle:**

- Created by CachedCompiler
- Stateless (can be reused)

**Key Methods:**

```rust
pub fn generate(&self, form: &CoreForm, state: &SessionState) -> Result<GeneratedCode>;
fn wrap_in_function(&self, ast: RustAst, state: &SessionState, fn_name: &str) -> RustAst;
fn add_source_map_comments(&self, source: String, map: &SourceMap) -> String;
```

**Generated Code Structure:**

```rust
// src/lib.rs (generated)

// VariableStore implementation (embedded)
mod evcxr_variable_store {
    use std::any::Any;
    use std::collections::HashMap;

    pub struct VariableStore {
        variables: HashMap<String, Box<dyn Any + 'static>>,
    }

    impl VariableStore {
        pub fn put_variable<T: 'static>(&mut self, name: &str, value: T) { ... }
        pub fn check_variable<T: 'static>(&mut self, name: &str) -> bool { ... }
        pub fn take_variable<T: 'static>(&mut self, name: &str) -> T { ... }
    }
}

// Generated function (unique per eval)
#[no_mangle]
pub extern "C" fn run_user_code_5(
    mut store_ptr: *mut evcxr_variable_store::VariableStore
) -> *mut evcxr_variable_store::VariableStore {
    let store = unsafe { &mut *store_ptr };

    // Load variables from store
    if !store.check_variable::<i32>("x") { return store_ptr; }
    let mut x = store.take_variable::<i32>("x");

    // User code (from Core Forms)
    /* oxur_node=42 */ let result = /* oxur_node=43 */ x + 1;

    // Store variables back
    store.put_variable("x", x);
    store.put_variable("result", result);

    store_ptr
}
```

**Integration Point:** Calls oxur-comp and oxur-ast!

---

#### SessionDir

**Location:** `oxur-repl/src/session/dir.rs`

**Purpose:** Manages temporary filesystem for a session

**Responsibilities:**

- Create temp directory structure
- Write Cargo.toml
- Write src/lib.rs
- Provide paths to compiled artifacts
- Cleanup on session close

**Structure:**

```
/tmp/oxur-repl/session-{uuid}/
├── Cargo.toml
├── src/
│   └── lib.rs
└── target/
    └── debug/
        ├── libctx.so
        ├── libeval_001.so
        ├── libeval_002.so
        └── incremental/
```

**Key Methods:**

```rust
pub fn new(session_id: &SessionId) -> Result<Self>;
pub fn root(&self) -> &Path;
pub fn src_path(&self) -> PathBuf;
pub fn target_dir(&self) -> PathBuf;
pub fn cleanup(&self) -> Result<()>;
```

---

#### SourceMap

**Location:** `oxur-repl/src/source_map.rs`

**Purpose:** Tracks transformations for error translation

**Responsibilities:**

- Map Node IDs to original Oxur source positions
- Track Surface → Core → Rust transformations
- Enable rustc error → Oxur error translation

**State:**

```rust
pub struct SourceMap {
    surface_map: HashMap<NodeId, SourcePos>,
    core_to_surface: HashMap<NodeId, NodeId>,
    rust_to_core: HashMap<NodeId, NodeId>,
}
```

**Key Methods:**

```rust
pub fn lookup(&self, node_id: NodeId) -> Option<SourcePos>;
pub fn add_surface_mapping(&mut self, node_id: NodeId, pos: SourcePos);
pub fn add_transformation(&mut self, from: NodeId, to: NodeId);
```

---

### 2.3 Subprocess Components (oxur-subprocess crate)

#### Subprocess Runtime

**Location:** `oxur-subprocess/src/main.rs`

**Purpose:** Isolated execution environment for user code

**Responsibilities:**

- Load compiled dylibs via libloading
- Call exported functions with VariableStore
- Capture stdout/stderr during execution
- Maintain VariableStore across evaluations
- Handle LOAD and PING commands

**Protocol:**

```
stdin commands:
  LOAD <lib_path> <fn_name>   - Load and execute library
  PING                        - Health check

stdout responses:
  PONG                        - Response to PING
  EVCXR_EXECUTION_COMPLETE    - Execution finished
  <stdout from user code>     - Captured output
```

**Main Loop:**

```rust
fn main() {
    let mut runtime = Runtime::new();
    runtime.run_loop();
}

impl Runtime {
    fn run_loop(&mut self) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match self.handle_command(&line?) {
                Ok(_) => {}
                Err(e) => eprintln!("ERROR: {}", e),
            }
        }
    }

    fn handle_command(&mut self, cmd: &str) -> Result<()> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts[0] {
            "LOAD" => {
                let lib_path = parts[1];
                let fn_name = parts[2];
                self.load_and_run(lib_path, fn_name)?;
                println!("EVCXR_EXECUTION_COMPLETE");
            }
            "PING" => println!("PONG"),
            _ => return Err(Error::UnknownCommand),
        }
        Ok(())
    }
}
```

---

#### VariableStore

**Location:** `oxur-subprocess/src/variable_store.rs` (also embedded in generated code)

**Purpose:** Type-erased variable persistence across evaluations

**Responsibilities:**

- Store variables with runtime type tracking
- Type-safe retrieval with downcast
- Variable lifetime management

**Implementation:**

```rust
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
            true  // Variable doesn't exist yet, that's ok
        }
    }

    pub fn take_variable<T: 'static>(&mut self, name: &str) -> T {
        *self.variables.remove(name)
            .expect("Variable missing")
            .downcast()
            .expect("Variable type mismatch")
    }
}
```

**Why Type-Erased?**

- No serialization overhead
- Supports arbitrary user types (no trait bounds)
- Simple implementation (~50 lines)
- Proven pattern from evcxr

---

## 3. Compilation Pipeline

### 3.1 Stage-by-Stage Breakdown

```
User Input: "(+ 1 2)"
  │
  ↓
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
│   match self.mode {                                         │
│       ReplMode::Lisp => oxur_lang::parse_lisp(code),        │
│       ReplMode::Sexpr => oxur_lang::parse_core_forms(code), │
│   }                                                         │
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
│   oxur_lang::expand(surface_forms)                      │
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
│   } else if is_cached(&core_forms) {                    │
│       Tier::Cached      // ~0ms (library loaded)        │
│   } else {                                              │
│       Tier::Jit         // 50-300ms (compile + exec)    │
│   }                                                     │
│                                                         │
│ Tier 1 → eval_calculator() (pure Rust evaluation)       │
│ Tier 2/3 → compiler.eval() (generate & compile)         │
└─────────────────────────────────────────────────────────┘
  │
  ↓ (Tier 2/3 path)
┌─────────────────────────────────────────────────────────┐
│ STAGE 4: Lower (in CodeGenerator)                       │
│ Owner: Server (CachedCompiler → CodeGenerator)          │
│ Crate: oxur-comp                                        │
│                                                         │
│ Input:  Core Forms                                      │
│ Output: Rust AST (syn crate structures)                 │
│                                                         │
│ Implementation:                                         │
│   let rust_ast = oxur_comp::lower(&core_forms)?;        │
│                                                         │
│ Examples:                                               │
│   (define-func add [x y] (+ x y))                       │
│   → syn::ItemFn { ... }                                 │
│                                                         │
│   (if-expr condition then-branch else-branch)           │
│   → syn::ExprIf { ... }                                 │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 5: Wrap (in CodeGenerator)                        │
│ Owner: Server (CodeGenerator)                           │
│                                                         │
│ Takes lowered Rust AST and wraps in:                    │
│   - VariableStore integration                           │
│   - Function signature (extern "C")                     │
│   - Variable load/store code                            │
│   - Source map comments                                 │
│                                                         │
│ Output: Complete Rust AST for library                   │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 6: Generate (in CodeGenerator)                    │
│ Owner: Server (CodeGenerator)                           │
│ Crate: oxur-ast                                         │
│                                                         │
│ Input:  Wrapped Rust AST                                │
│ Output: Rust source code (String)                       │
│                                                         │
│ Implementation:                                         │
│   let source = oxur_ast::print_rust(&wrapped_ast);      │
│                                                         │
│ Result: Formatted, compilable Rust source               │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 7: Write Files (in CachedCompiler)                │
│ Owner: Server (CachedCompiler)                          │
│                                                         │
│ Writes:                                                 │
│   /tmp/oxur-repl/session-abc/src/lib.rs                 │
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
│     --message-format=json                               │
│                                                         │
│ Environment:                                            │
│   CARGO_TARGET_DIR=/tmp/oxur-repl/session-abc/target    │
│   RUSTFLAGS="-C link-arg=-fuse-ld=mold"                 │
│                                                         │
│ Incremental compilation enabled (3-5x speedup)          │
│ opt-level = 2 (balance compile vs runtime perf)         │
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 9: Parse Cargo Output (in CachedCompiler)         │
│ Owner: Server (CachedCompiler)                          │
│                                                         │
│ Parses JSON messages from cargo                         │
│ Checks for compilation errors                           │
│ If errors: translate to Oxur positions via SourceMap    │
│ If success: extract artifact path                       │
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
└─────────────────────────────────────────────────────────┘
  │
  ↓
┌─────────────────────────────────────────────────────────┐
│ STAGE 11: Load & Execute (in CachedCompiler)            │
│ Owner: Server (CachedCompiler)                          │
│ Executor: Subprocess                                    │
│                                                         │
│ 1. Send to subprocess:                                  │
│    LOAD /path/to/libeval_005.so run_user_code_5         │
│                                                         │
│ 2. Subprocess:                                          │
│    - Loads library via libloading                       │
│    - Calls run_user_code_5(variable_store_ptr)          │
│    - Function executes, mutates VariableStore           │
│    - Captures stdout/stderr                             │
│    - Returns                                            │
│                                                         │
│ 3. Subprocess sends:                                    │
│    EVCXR_EXECUTION_COMPLETE                             │
│                                                         │
│ 4. CachedCompiler captures output and result            │
└─────────────────────────────────────────────────────────┘
  │
  ↓
Result: Value (to return to user)
```

### 3.2 Ownership Summary

| Stage | Owner Component | Owner Location | External Crate |
|-------|----------------|----------------|----------------|
| Parse | EvalContext | Server | oxur-lang |
| Expand | EvalContext | Server | oxur-lang |
| Tier Decision | EvalContext | Server | - |
| Lower | CodeGenerator | Server | oxur-comp |
| Wrap | CodeGenerator | Server | - |
| Generate | CodeGenerator | Server | oxur-ast |
| Write | CachedCompiler | Server | - |
| Compile | CachedCompiler | Server | cargo |
| Parse Output | CachedCompiler | Server | - |
| Rename | CachedCompiler | Server | - |
| Execute | CachedCompiler | Server → Subprocess | libloading |

**Key Insight:** ALL stages happen on the server. Client just sends/receives protocol messages.

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
Mode is Lisp, so:
  surface_forms = oxur_lang::parse_lisp("(+ 1 2)")?
  core_forms = oxur_lang::expand(surface_forms)?

Result: CoreForm::FunctionCall {
  function: "+",
  args: [CoreForm::Literal(1), CoreForm::Literal(2)]
}

Step 8: Tier Decision (in EvalContext)
────────────────────────────────────
Check if simple arithmetic: YES
Decision: Tier::Calculator

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

Step 7: Parse & Expand
────────────────────────
surface_forms = oxur_lang::parse_lisp("(defn square [x] (* x x))")
core_forms = oxur_lang::expand(surface_forms)

Result: CoreForm::FunctionDefinition {
  name: "square",
  params: [("x", Type::Infer)],
  body: CoreForm::FunctionCall { ... }
}

Step 8: Tier Decision
──────────────────────
Not simple arithmetic
Check cache: not previously compiled
Decision: Tier::Jit

Step 9: Compilation (in CachedCompiler)
───────────────────────────────────────
compiler.eval(core_forms)

  Step 9a: Code Generation
  ─────────────────────────
  code_gen.generate(&core_forms, &state)
    - oxur_comp::lower(core_forms) → RustAst
    - wrap_in_function(ast, state)
    - oxur_ast::print_rust(wrapped_ast) → String

  Result: Rust source code for lib.rs

  Step 9b: Write Files
  ────────────────────
  Write to /tmp/oxur-repl/session-abc/src/lib.rs

  Step 9c: Invoke Cargo
  ──────────────────────
  cargo build --message-format=json
  Time: 200-300ms (first time), 50-100ms (warm)

  Step 9d: Parse Output
  ──────────────────────
  Read JSON messages from cargo stdout
  Check for errors
  If errors: translate via SourceMap
  Extract artifact path

  Step 9e: Rename Artifact
  ─────────────────────────
  Rename libctx.so → libeval_006.so

Step 10: Execution (in Subprocess)
───────────────────────────────────
CachedCompiler sends to subprocess:
  LOAD /tmp/.../libeval_006.so run_user_code_6

Subprocess:
  - Loads library via libloading
  - Calls run_user_code_6(variable_store_ptr)
  - Function defines square function in VariableStore
  - Returns

Subprocess responds:
  EVCXR_EXECUTION_COMPLETE

Step 11: Result Capture
─────────────────────────
CachedCompiler receives completion marker
Captures any stdout/stderr
Returns result to EvalContext

Step 12-14: Same as simple path
─────────────────────────────────
Response constructed and sent to client
User sees: function defined (or similar feedback)
```

### 4.3 Error Path: Compilation Error

```
Step 1-9a: Same as compilation path
────────────────────────────────────
Generate code, write files, invoke cargo

Step 9b: Cargo Returns Errors
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
        ...
      }]
    }
  }

Step 10: Error Translation
────────────────────────────
CachedCompiler.translate_errors()

  a. Extract span from rustc error
  b. Read line 42 from lib.rs
  c. Find source map comment: /* oxur_node=123 */
  d. Lookup NodeId 123 in SourceMap
  e. Get original Oxur position: test.ox:5:15

  Result: OxurError {
    message: "cannot find value `y` in this scope",
    file: "test.ox",
    line: 5,
    column: 15,
    code: "E0425",
    level: "error"
  }

Step 11: Error Response
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

Step 12: Client Displays Error
────────────────────────────────
Client receives error response
Formats and displays to user:
  Error at test.ox:5:15: cannot find value `y` in this scope
```

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
  6. SessionDir creates temp directory
  7. SessionDir writes Cargo.toml
  8. Store in sessions map
Server → Client: Response { session: "session-abc", status: [SessionCreated] }

ACTIVE SESSION
──────────────
Client → Server: Request { op: Eval, session: "session-abc", ... }
Server:
  1. SessionManager.get_session("session-abc")
  2. Lock EvalContext
  3. Evaluate code
  4. Unlock EvalContext
Server → Client: Response { ... }

(Can have multiple concurrent requests to different sessions)

CLOSE SESSION
─────────────
Client → Server: Request { op: Close, session: "session-abc" }
Server:
  1. SessionManager.remove_session("session-abc")
  2. Drop EvalContext
  3. CachedCompiler drop → kills subprocess
  4. SessionDir drop → cleans temp directory
Server → Client: Response { status: [SessionClosed] }
```

### 5.2 Session Isolation

Each session has completely isolated:

**Filesystem:**

- `/tmp/oxur-repl/session-{uuid}/` - unique temp directory
- Independent Cargo projects
- No shared compiled artifacts

**Process:**

- Separate subprocess per session
- Subprocess crash doesn't affect other sessions
- Independent VariableStore per subprocess

**State:**

- Independent evaluation history
- Independent variable bindings
- Independent REPL mode (Lisp vs Sexpr)

**Concurrency:**

- Sessions can evaluate in parallel
- EvalContext is Mutex-protected (one eval at a time per session)
- SessionManager is Arc<RwLock<...>> (concurrent session access)

### 5.3 Resource Management

**Session Limits:**

- Max sessions per server: configurable (default: 100)
- Enforcement: SessionManager refuses creation if limit reached
- Error: `ErrorKind::Session` with message "session limit reached"

**Session Timeouts:**

- Idle timeout: configurable (default: 30min)
- SessionManager background task checks last activity
- Auto-close idle sessions
- Client gets error on next request to closed session

**Disk Usage:**

- Per session: 30-100MB (incremental compilation cache)
- 100 sessions × 100MB = ~10GB max
- Cleanup on session close
- Stale dir cleanup on server startup (>24h old)

**Memory Usage:**

- Per session: ~10-50MB (EvalContext, VariableStore, buffers)
- Subprocess: ~10-20MB per session
- 100 sessions × 70MB = ~7GB max

**CPU:**

- Compilation is CPU-intensive (30-300ms)
- One compilation per session at a time (Mutex on EvalContext)
- Multiple sessions can compile in parallel
- Tier 1 (calculator) is minimal CPU

---

## 6. Protocol Integration

### 6.1 Protocol Operations

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

### 6.2 Error Propagation

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
Translated via SourceMap
  ↓
Converted to ErrorKind::Lower
  ↓
Passed to MessageHandler
  ↓
Response with Error status + source location

Runtime Error (panic in user code)
  ↓
Subprocess crashes
  ↓
CachedCompiler detects subprocess death
  ↓
Restarts subprocess
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

### 6.3 Streaming Output

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
- **Example:** `(+ 1 2)`, `(* 3 4)`, `(- 10 5)`

**Tier 2: Cached Compilation**

- **Criteria:** Previously compiled code, exact match
- **Detection:** Hash of Core Forms matches cached entry
- **Execution:** Library already loaded in subprocess
- **Performance:** ~1-5ms (function call overhead)
- **Example:** Re-evaluating `(square 5)` after defining square

**Tier 3: JIT Compilation**

- **Criteria:** Complex code, first-time evaluation
- **Execution:** Full compilation pipeline
- **Performance:** 50-300ms (depends on code complexity)
- **Example:** `(defn square [x] (* x x))`, complex expressions

### 7.2 Tier Decision Logic

```rust
// In EvalContext
fn decide_tier(&self, core_forms: &CoreForm) -> Tier {
    // Check Tier 1: Calculator
    if is_simple_arithmetic(core_forms) {
        return Tier::Calculator;
    }

    // Check Tier 2: Cached
    let hash = hash_core_forms(core_forms);
    if self.compiler.is_cached(hash) {
        return Tier::Cached;
    }

    // Default: Tier 3: JIT
    Tier::Jit
}

fn is_simple_arithmetic(form: &CoreForm) -> bool {
    match form {
        CoreForm::Literal(_) => true,
        CoreForm::FunctionCall { function, args } => {
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

### 7.3 Tier Performance Targets

| Tier | First Time | Subsequent | Cache Hit |
|------|-----------|------------|-----------|
| Tier 1 | <1ms | <1ms | N/A |
| Tier 2 | N/A | 1-5ms | 1-5ms |
| Tier 3 | 200-300ms | 50-100ms | See Tier 2 |

**Incremental Compilation Impact:**

- Cold compile (no cache): 200-300ms
- Warm compile (incremental cache): 50-100ms
- 3-5x speedup from incremental compilation

---

## 8. Integration Points with External Crates

### 8.1 oxur-lang Integration

**Required API:**

```rust
// oxur-lang/src/parser.rs
pub fn parse_lisp(source: &str) -> Result<SurfaceForms, ParseError>;
pub fn parse_core_forms(source: &str) -> Result<CoreForms, ParseError>;

// oxur-lang/src/expander.rs
pub fn expand(surface: SurfaceForms) -> Result<CoreForms, ExpandError>;

// Error types
pub enum ParseError {
    UnexpectedToken { position: SourcePos, found: String, expected: String },
    UnmatchedParens { position: SourcePos },
    // ...
}

pub enum ExpandError {
    UnknownMacro { name: String, position: SourcePos },
    MacroExpansionFailed { message: String, position: SourcePos },
    // ...
}
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
    },
    FunctionDefinition {
        name: String,
        params: Vec<(String, Type)>,
        body: Box<CoreForm>,
    },
    IfExpr {
        condition: Box<CoreForm>,
        then_branch: Box<CoreForm>,
        else_branch: Option<Box<CoreForm>>,
    },
    LetBinding {
        bindings: Vec<(String, CoreForm)>,
        body: Box<CoreForm>,
    },
    // ... other forms
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

**Status:** Need to verify this API exists or design it

---

### 8.2 oxur-comp Integration

**Required API:**

```rust
// oxur-comp/src/lower.rs
pub fn lower(core: &CoreForm) -> Result<RustAst, LowerError>;

// Error type
pub enum LowerError {
    UnsupportedForm { form: String, position: SourcePos },
    TypeMismatch { expected: Type, found: Type, position: SourcePos },
    // ...
}
```

**Called by:** CodeGenerator in `generate()` method

**Data Types:**

```rust
// Rust AST (using syn crate)
pub type RustAst = syn::File;  // Or syn::Item, depending on granularity

// Lowering maps:
// CoreForm::FunctionDefinition → syn::ItemFn
// CoreForm::IfExpr → syn::ExprIf
// CoreForm::LetBinding → syn::Stmt::Local
// etc.
```

**Status:** Need to verify this exists or design lowering strategy

---

### 8.3 oxur-ast Integration

**Required API:**

```rust
// oxur-ast/src/printer.rs
pub fn print_rust(ast: &syn::File) -> String;
```

**Called by:** CodeGenerator after wrapping AST

**Implementation:** Likely uses `prettyplease` or `syn::File::to_token_stream()`

**Status:** May already exist; verify

---

### 8.4 Dependency Summary

```
oxur-repl (EvalContext)
  ├─→ oxur-lang::parse_lisp()
  ├─→ oxur-lang::expand()
  └─→ oxur-lang::parse_core_forms()

oxur-repl (CodeGenerator)
  ├─→ oxur-comp::lower()
  └─→ oxur-ast::print_rust()

oxur-repl (CachedCompiler)
  └─→ cargo (external tool)

oxur-subprocess
  └─→ libloading (external crate)
```

**Critical Path Blockers:**

1. oxur-lang API must be defined and implemented
2. oxur-comp lowering must be implemented
3. CoreForm data type must be agreed upon

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
   a. code_gen.generate()
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
   a. code_gen.generate()
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
/tmp/oxur-repl/
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
edition = "2024"

[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[profile.dev]
opt-level = 2        # Balance: compile speed vs runtime performance
incremental = true   # 3-5x speedup on warm builds

# No dependencies in v1.0
# v1.1: user-requested dependencies via (require "crate")
[dependencies]
```

### 10.3 Generated lib.rs Structure

```rust
// src/lib.rs (generated by CodeGenerator)

// Embedded VariableStore (same for all evaluations)
mod evcxr_variable_store {
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
    mut store_ptr: *mut evcxr_variable_store::VariableStore
) -> *mut evcxr_variable_store::VariableStore {
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
│ Translation: Already has Oxur source position           │
│ Source: oxur-lang crate                                 │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Expand                                       │
│ - Unknown macro                                         │
│ - Macro expansion failed                                │
│ - Invalid macro arguments                               │
│                                                         │
│ Handling: EvalContext catches oxur_lang::ExpandError    │
│ Translation: Already has Oxur source position           │
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
│ Process:                                                │
│   rustc error (lib.rs:42) → Node ID → Oxur position     │
│ Source: cargo/rustc                                     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ ErrorKind::Eval                                         │
│ - Runtime panic                                         │
│ - Subprocess crash                                      │
│ - Arithmetic overflow                                   │
│ - Unwrap on None                                        │
│                                                         │
│ Handling: CachedCompiler detects subprocess death       │
│ Translation: Difficult (runtime, no static position)    │
│ Mitigation: Stack traces if available                   │
│ Source: User code at runtime                            │
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
│ Handling: Tokio timeout wrappers                        │
│ Translation: None                                       │
│ Source: Time limits                                     │
└─────────────────────────────────────────────────────────┘
```

### 11.2 Source Map Translation (Critical)

**The Challenge:**

```
User writes Oxur code:
  (defn square [x]
    (+ x y))  ; <-- ERROR: y is undefined (line 2, column 8)

After lowering, rustc sees:
  fn square(x: i32) -> i32 {
      x + y  // <-- ERROR at lib.rs:47:9
  }

We need to map lib.rs:47:9 back to original test.ox:2:8
```

**The Solution:**

1. **Node ID Assignment** (during parsing):

   ```rust
   // When parsing (+ x y), assign Node IDs:
   CoreForm::FunctionCall {
       node_id: 42,
       function: "+",
       args: [
           CoreForm::Variable { node_id: 43, name: "x" },
           CoreForm::Variable { node_id: 44, name: "y" },
       ]
   }

   // Record in SourceMap:
   source_map.add_surface_mapping(44, SourcePos {
       file: "test.ox",
       line: 2,
       column: 8,
   });
   ```

2. **Preserve Node IDs** (during lowering):

   ```rust
   // CodeGenerator embeds Node IDs in comments:
   let result = /* oxur_node=42 */ x + /* oxur_node=44 */ y;
   ```

3. **Extract on Error**:

   ```rust
   // When rustc reports error at lib.rs:47:
   let line = read_line("lib.rs", 47)?;
   // line = "let result = /* oxur_node=42 */ x + /* oxur_node=44 */ y;"

   let node_id = extract_node_id(line)?;  // Finds 44 (nearest to error column)

   let oxur_pos = source_map.lookup(44)?;
   // oxur_pos = SourcePos { file: "test.ox", line: 2, column: 8 }
   ```

4. **Format Error**:

   ```rust
   OxurError {
       message: "cannot find value `y` in this scope",
       file: "test.ox",
       line: 2,
       column: 8,
       code: "E0425",
       level: "error",
   }
   ```

**Fallback:**

If Node ID extraction fails or source map lookup fails:

```rust
OxurError {
    message: "cannot find value `y` in this scope (in generated Rust at lib.rs:47:9)",
    file: "<generated>",
    line: 0,
    column: 0,
    code: "E0425",
    level: "error",
}
```

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

## Conclusion

This architecture provides:

✅ **Clear separation of concerns** - Client handles protocol, server handles execution
✅ **Subprocess isolation** - User code crashes don't corrupt REPL state
✅ **Session-based architecture** - Multiple concurrent users, isolated state
✅ **Three-tier execution** - Optimize for common cases (calculator mode)
✅ **Complete compilation pipeline** - Oxur → Core Forms → Rust → Execution
✅ **Error translation** - Source maps enable Oxur-level error messages
✅ **Scalable design** - Supports single-user and multi-user deployments
✅ **Well-defined integration points** - Clear APIs with oxur-lang, oxur-comp, oxur-ast
✅ **Resource management** - Limits, timeouts, cleanup strategies
✅ **Proven patterns** - Based on evcxr's battle-tested approach

### Next Steps

1. **Verify external crate APIs** - Confirm oxur-lang, oxur-comp, oxur-ast provide required functions
2. **Implement foundation components** - VariableStore, SessionDir, Subprocess runtime
3. **Build integration layer** - CodeGenerator, source maps, error translation
4. **Implement CachedCompiler** - Core compilation engine
5. **Test end-to-end** - Simple eval, complex eval, error cases

---

**Document Status:** Complete - Ready for review and implementation
