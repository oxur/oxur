---
number: 30
title: "Oxur REPL Implementation Specification"
author: "Claude Code & Duncan McGreggor"
component: REPL
created: 2026-01-03
updated: 2026-01-04
state: Draft
supersedes: null
superseded-by: null
version: 1.1
---


# Oxur REPL Implementation Specification

*Synthesized from evcxr Audits*

```
Version: 1.1
Date: January 3, 2026
Status: Definitive Reference
```

**IMPORTANT:** This document focuses on component implementation details. For the complete system architecture (client/server, data flow, integration points), see the **REPL Architecture Overview** document.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [Architecture Decision Records](#3-architecture-decision-records)
4. [Component Implementation Specifications](#4-component-implementation-specifications)
5. [rustc Invocation Reference](#5-rustc-invocation-reference)
6. [File System Organization](#6-file-system-organization)
7. [Performance Expectations](#7-performance-expectations)
8. [Error Handling Strategy](#8-error-handling-strategy)
9. [Testing Strategy](#9-testing-strategy)
10. [Implementation Roadmap](#10-implementation-roadmap)
11. [Dependencies and Versioning](#11-dependencies-and-versioning)
12. [Risk Mitigation](#12-risk-mitigation)
13. [See Also](#13-see-also)
14. [Revision History](#14-revision-history)
15. [Conclusion](#15-conclusion)
16. [Appendix: Audit Summary](#16-appendix-audit-summary)

---

## 1. Executive Summary

### 1.1 Key Findings

After auditing evcxr_repl, evcxr_runtime, and evcxr's compiler integration:

1. **Two-Process Model is Essential** - Subprocess isolation prevents user code crashes from corrupting REPL state. Enables restart-on-panic without data loss.

2. **evcxr_runtime is NOT a Runtime** (Critical Discovery) - Despite its name, it's only 75 lines of MIME output formatting. The real runtime is `evcxr_internal_runtime.rs` using `Box<dyn Any>` for variable storage.

3. **Cargo > Direct rustc** - evcxr uses cargo, not direct rustc. This provides dependency management, incremental compilation, and build orchestration with only 10-20ms overhead.

4. **Incremental Compilation Provides 3-5x Speedup** - First compile: 200-300ms. Subsequent: 50-100ms. Critical for interactive experience.

5. **Type-Erased Variable Storage Works** - `Box<dyn Any>` with runtime type checking enables variable persistence without serialization.

6. **rustc Wrapper is Advanced Optimization** - Forces dynamic linking for faster loading, but adds complexity. Defer to v1.1.

7. **Platform Differences Matter** - Windows DLL locking requires file copying. macOS has filesystem timestamp quirks. Handle carefully.

### 1.2 Strategic Recommendations

**ADOPT from evcxr:**

- Two-process execution model
- Type-erased variable storage (`Box<dyn Any>`)
- Cargo-based compilation
- Incremental compilation (always enabled)
- opt-level 2 for balance
- JSON error parsing
- Unique library naming

**BUILD ourselves:**

- Skip evcxr_runtime dependency - implement `OxurDisplay` with structured `DisplayValue` enum
- Source map integration (Oxur → Rust error translation)
- Tier 1 calculator mode (<1ms for literal arithmetic)
- Network protocol integration (postcard serialization)
- Client/server architecture integration

**DEFER to v1.1:**

- rustc wrapper for dynamic linking optimization
- External dependency management `(require "crate")`
- Auto-fix compilation errors
- Async mode auto-detection

### 1.3 Risk Assessment

**High Priority Risks:**

1. **Multi-Session Resource Management** (High likelihood, High impact)
   - One subprocess per session could exhaust resources
   - Mitigation: Session limits (5 per user), 30min idle timeout, subprocess pooling in v1.1

2. **Source Map Accuracy** (Medium likelihood, High impact)
   - Mapping rustc errors to Oxur source requires careful tracking
   - Mitigation: Comprehensive source maps at each stage, fuzzy matching fallback

3. **First Compilation Latency** (High likelihood, Medium impact)
   - Cold compile is 200-300ms
   - Mitigation: Progress indicators, pre-compile on startup, Tier 1 for instant feedback

---

## 2. Architecture Overview

### 2.1 Purpose and Scope

This section provides a high-level overview of how the REPL components fit into the broader Oxur system. **For complete architectural details, see the REPL Architecture Overview document**, which includes:

- Complete client/server architecture
- Detailed component diagrams
- Full data flow specifications
- Integration point specifications
- Session management architecture
- Protocol integration patterns

### 2.2 System Architecture Summary

**Client-Server Model:**

```
┌─────────────┐                ┌──────────────────────────────┐
│   Client    │   TCP/Binary   │         Server               │
│  ReplClient │◄──────────────►│  ReplServer                  │
│             │   Protocol     │    │                         │
│  (Thin)     │   (Postcard)   │    ↓                         │
│             │                │  MessageHandler              │
│             │                │    │                         │
│             │                │    ↓                         │
│             │                │  SessionManager              │
│             │                │    │                         │
│             │                │    ↓                         │
│             │                │  EvalContext (per session)   │
│             │                │    │                         │
│             │                │    ↓                         │
│             │                │  CachedCompiler              │
│             │                │    │                         │
│             │                │    ↓                         │
│             │                │  Subprocess (isolated)       │
└─────────────┘                └──────────────────────────────┘
```

**Key Architectural Decisions:**

1. **Server-Side Compilation** - All compilation happens on the server, not the client
2. **CachedCompiler Owned by EvalContext** - One compiler per session, simplest ownership
3. **Subprocess Isolation** - Each session has its own subprocess for safety
4. **Protocol Integration** - Client/server communicate via ODD-0018 protocol

### 2.3 Component Locations

| Component | Crate | Module | Owned By | Description |
|-----------|-------|--------|----------|-------------|
| ReplClient | oxur-repl | client/ | User code | Protocol client |
| ReplServer | oxur-repl | server/ | Server process | TCP listener |
| MessageHandler | oxur-repl | server/ | ReplServer | Operation dispatcher |
| SessionManager | oxur-repl | server/ | ReplServer | Session lifecycle |
| EvalContext | oxur-repl | eval/ | SessionManager | Per-session state |
| CachedCompiler | oxur-repl | compiler/ | EvalContext | Compilation engine |
| CodeGenerator | oxur-repl | codegen/ | CachedCompiler | Core Forms → Rust |
| SessionDir | oxur-repl | session/ | CachedCompiler | Temp filesystem |
| SourceMap | oxur-repl | source_map.rs | CachedCompiler | Error translation |
| VariableStore | oxur-subprocess | variable_store.rs | Subprocess | Type-erased storage |
| Subprocess Runtime | oxur-subprocess | main.rs | - | Execution environment |

### 2.4 Compilation Pipeline Ownership

```
User Input: "(+ 1 2)"
  ↓
EvalContext (server)
  ├─→ oxur-lang::parse_lisp() → SurfaceForms
  ├─→ oxur-lang::expand() → CoreForms
  └─→ Tier decision
      ↓
CachedCompiler (server)
  ├─→ CodeGenerator
  │   ├─→ oxur-comp::lower() → RustAst
  │   └─→ oxur-ast::print_rust() → Rust source
  ├─→ cargo build → dylib
  └─→ Subprocess: load & execute
```

**Critical Insight:** ALL stages happen on the server. The client is purely a protocol endpoint.

### 2.5 Integration Points

**Required from oxur-lang:**

```rust
pub fn parse_lisp(source: &str) -> Result<SurfaceForms>;
pub fn expand(surface: SurfaceForms) -> Result<CoreForms>;
pub fn parse_core_forms(source: &str) -> Result<CoreForms>;
```

**Required from oxur-comp:**

```rust
pub fn lower(core: &CoreForm) -> Result<RustAst>;
```

**Required from oxur-ast:**

```rust
pub fn print_rust(ast: &syn::File) -> String;
```

**Status:** These APIs need to be verified or designed in their respective crates.

### 2.6 Session Architecture

Each session is completely isolated:

- **Filesystem:** Separate temp directory (`/tmp/oxur-repl/session-{uuid}/`)
- **Process:** Separate subprocess (prevents crash propagation)
- **State:** Independent EvalContext, VariableStore, history
- **Concurrency:** Mutex on EvalContext (one eval at a time per session)

Sessions managed by SessionManager:

- Creates/destroys sessions
- Enforces limits (max sessions, timeouts)
- Thread-safe access (`Arc<RwLock<HashMap<SessionId, EvalContext>>>`)

---

## 3. Architecture Decision Records

### ADR-001: Value Representation and Variable Storage

**Status:** Decided

**Decision:** Use type-erased storage with `Box<dyn Any>` (evcxr pattern).

**Rationale:**

- Proven in production (evcxr)
- No serialization overhead
- Supports arbitrary user types
- Simple implementation (~50 lines)

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

**Consequences:**

- Variables persist across evaluations
- Type safety via runtime checking
- No trait bounds on user types
- Can't serialize (acceptable for v1.0)

---

### ADR-002: Output Capture Mechanism

**Status:** Decided

**Decision:** Use OS-level subprocess pipes, structured in Response messages.

**Rationale:**

- Simple and reliable
- Works across library boundaries
- Natural fit for subprocess architecture
- Separate stdout/stderr streams

**Implementation:**

```rust
pub struct Response {
    pub value: Option<DisplayValue>,  // Rich display
    pub out: String,                   // Captured stdout
    pub err: String,                   // Captured stderr
    pub status: Vec<Status>,           // Errors, warnings
}

pub enum DisplayValue {
    Text(String),
    Html(String),
    Image { mime: String, data: Vec<u8> },
    Custom { mime: String, content: Vec<u8> },
}
```

No stdout parsing needed - DisplayValue is explicitly set by generated code.

---

### ADR-003: Compilation Strategy

**Status:** Decided

**Decision:** Use cargo as build orchestrator, not direct rustc.

**Cargo Invocation:**

```bash
cargo build \
  --target x86_64-unknown-linux-gnu \
  --message-format=json

# Environment:
CARGO_TARGET_DIR=/path/to/session/target
RUSTFLAGS="-C link-arg=-fuse-ld=mold"  # If available
```

**Cargo.toml Template:**

```toml
[package]
name = "ctx"
version = "1.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[profile.dev]
opt-level = 2        # Balance compile vs runtime perf
incremental = true   # 3-5x speedup on warm builds

[dependencies]
# Future: user-requested deps via (require ...)
```

**Rationale:**

- Dependency management "for free"
- Incremental compilation "for free"
- Standard tooling
- Only 10-20ms overhead vs direct rustc
- Proven by evcxr

---

### ADR-004: Temporary File Management

**Status:** Decided

**Decision:** Per-session temporary directories, cleaned on close.

**Structure:**

```
/tmp/oxur-repl/session-{uuid}/
├── Cargo.toml
├── src/lib.rs
├── target/
│   └── {triple}/debug/
│       ├── libctx.so
│       ├── libeval_001.so  # Unique per eval
│       ├── libeval_002.so
│       └── incremental/
└── metadata.json
```

**Cleanup Strategy:**

- Normal close: Delete entire directory
- Server shutdown: Delete all session directories
- Startup: Clean stale dirs (>24h old)

**Disk Space:** 30-100MB per session (incremental cache)

---

### ADR-005: Error Translation and Source Mapping

**Status:** Decided

**Decision:** Multi-stage source mapping with Node IDs.

**Process:**

```
Oxur Source (.ox:5:15)
  ↓ Node ID: 42
Surface Forms
  ↓ Node ID: 43
Core Forms
  ↓ Node ID: 44
Rust AST
  ↓ Node ID: 45 (in comment)
Generated Rust (lib.rs:123:10)
  ↓ rustc error
Parse error + Node ID
  ↓ Source map lookup
Translate to Oxur position
```

**Generated Code Pattern:**

```rust
/* oxur_node=42 */ let x = /* oxur_node=43 */ 10 + /* oxur_node=44 */ 20;
```

**Data Structure:**

```rust
pub struct SourceMap {
    surface_map: HashMap<NodeId, SourcePos>,
    core_to_surface: HashMap<NodeId, NodeId>,
    rust_to_core: HashMap<NodeId, NodeId>,
}
```

**Fallback:** If mapping fails, show Rust error with note about generated code.

---

### ADR-006: Code Generation Strategy

**Status:** Decided

**Decision:** Generate complete Rust libraries with wrapper function per eval.

**Template:**

```rust
// Generated src/lib.rs

mod evcxr_variable_store {
    // VariableStore implementation
}

#[no_mangle]
pub extern "C" fn run_user_code_5(
    mut store_ptr: *mut evcxr_variable_store::VariableStore
) -> *mut evcxr_variable_store::VariableStore {
    let store = unsafe { &mut *store_ptr };

    // Check and load variables
    if !store.check_variable::<i32>("x") { return store_ptr; }
    let mut x = store.take_variable::<i32>("x");

    // User code with source map comments
    /* oxur_node=42 */ let result = /* oxur_node=43 */ x + 1;

    // Store variables
    store.put_variable("x", x);

    store_ptr
}
```

---

### ADR-007: Dependency Management

**Status:** Proposed (Defer to v1.1)

**Decision:** No external dependencies in v1.0. Implement in v1.1.

**v1.0:** Return clear error if user tries `(require "crate")`

**v1.1 API:**

```oxur
(require "serde" :version "1.0")
(require "tokio" :features ["full"])
```

---

### ADR-008: Session State Management

**Status:** Decided

**Decision:** Simplified clone-try-commit pattern.

**Pattern:**

```rust
pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
    // 1. Clone state (cheap - just variable names/types)
    let mut tentative_state = self.state.clone();

    // 2. Update tentative state
    tentative_state.eval_counter += 1;

    // 3. Generate code with tentative state
    let code = generate(&tentative_state)?;

    // 4. Compile (might fail!)
    let artifact = compile(&code).await?;

    // 5. Execute
    let result = execute(&artifact).await?;

    // 6. Commit only on success
    self.state = tentative_state;

    Ok(result)
}
```

**What to Clone:** Variable names/types (cheap)
**What to Share:** Subprocess, directory, source map (single instance)

---

### ADR-009: Client-Server Architecture **(NEW)**

**Status:** Decided

**Decision:** Server-side compilation, thin client.

**Rationale:**

- Session state lives on server
- Subprocess is server-local (can't be remote)
- Matches evcxr pattern (local REPL)
- Simplifies client implementation
- Leverages existing protocol (ODD-0018)

**Client Responsibilities:**

- Send Request messages
- Receive Response messages
- Handle reconnection
- NO compilation, parsing, or execution logic

**Server Responsibilities:**

- All parsing (via oxur-lang)
- All compilation (CachedCompiler)
- All execution (Subprocess)
- Session management
- Protocol handling

**Consequences:**

- Client is simple and portable
- All heavy lifting on server
- Network latency per eval (acceptable for dev use)
- Resource management centralized

---

### ADR-010: Component Ownership **(NEW)**

**Status:** Decided

**Decision:** CachedCompiler owned by EvalContext.

**Rationale:**

- One-to-one relationship (one compiler per session)
- Simplest lifecycle management
- Natural ownership (EvalContext owns its compiler)
- Drop cascade works naturally

**Structure:**

```rust
pub struct EvalContext {
    session_id: SessionId,
    mode: ReplMode,
    compiler: CachedCompiler,  // ← Owned
    history: Vec<HistoryEntry>,
    output_buffer: OutputBuffer,
}
```

**Alternatives Considered:**

1. **Separate management** - More complex, no benefit
2. **Shared via Arc** - Unnecessary overhead, no concurrent access needed
3. **In MessageHandler** - Breaks session encapsulation

**Consequences:**

- Clear ownership model
- Simple drop semantics
- No Arc/Mutex overhead for compiler
- One compiler lifetime = one session lifetime

---

## 4. Component Implementation Specifications

This section provides detailed implementation specifications for each component. Component locations and ownership are defined in Section 2.3.

### 4.1 Server Components

#### 4.1.1 EvalContext

**Location:** `oxur-repl/src/eval/context.rs`
**Ownership:** Created and owned by SessionManager

**Purpose:** Manages evaluation state for a single REPL session.

**State:**

```rust
pub struct EvalContext {
    session_id: SessionId,
    mode: ReplMode,                    // Lisp or Sexpr
    compiler: CachedCompiler,          // Owned compilation engine
    history: Vec<HistoryEntry>,
    output_buffer: OutputBuffer,
}
```

**Core Methods:**

```rust
impl EvalContext {
    pub fn new(session_id: SessionId) -> Result<Self> {
        Ok(Self {
            session_id: session_id.clone(),
            mode: ReplMode::Lisp,
            compiler: CachedCompiler::new(session_id)?,
            history: Vec::new(),
            output_buffer: OutputBuffer::new(),
        })
    }

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
            Tier::Cached | Tier::Jit => {
                self.compiler.eval(core_forms).await?
            }
        };

        // 4. Record history
        self.record_history(code.to_string(), result.clone());

        Ok(result)
    }

    pub async fn load_file(&mut self, path: &str) -> Result<Value> {
        let source = tokio::fs::read_to_string(path).await?;
        self.eval(&source).await
    }

    pub fn set_mode(&mut self, mode: ReplMode) {
        self.mode = mode;
    }

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

    fn eval_calculator(&self, form: &CoreForm) -> Result<Value> {
        // Direct Rust evaluation for simple arithmetic
        // No compilation needed
        todo!("Implement calculator mode")
    }
}
```

**Integration Points:**

- Calls `oxur_lang::parse_lisp()` and `oxur_lang::expand()`
- Calls `CachedCompiler::eval()` for tier 2/3
- Called by MessageHandler via SessionManager

---

#### 4.1.2 CachedCompiler

**Location:** `oxur-repl/src/compiler/cached.rs`
**Ownership:** Owned by EvalContext

**Purpose:** Compiles Core Forms to Rust and executes them.

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

pub struct SessionState {
    variables: HashMap<String, TypeInfo>,
    eval_counter: u32,
}
```

**Core Methods:**

```rust
impl CachedCompiler {
    pub fn new(session_id: SessionId) -> Result<Self> {
        let session_dir = SessionDir::new(&session_id)?;

        Ok(Self {
            session_id,
            session_dir,
            state: SessionState::new(),
            subprocess: None,
            code_gen: CodeGenerator::new(),
            source_map: Arc::new(SourceMap::new()),
        })
    }

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

    async fn compile_to_dylib(&self, code: &GeneratedCode) -> Result<PathBuf> {
        // Write src/lib.rs
        tokio::fs::write(self.session_dir.src_path(), &code.source).await?;

        // Invoke cargo
        let output = Command::new("cargo")
            .arg("build")
            .arg("--message-format=json")
            .current_dir(self.session_dir.root())
            .env("CARGO_TARGET_DIR", self.session_dir.target_dir())
            .env("RUSTFLAGS", self.rustflags())
            .output().await?;

        // Parse JSON output
        let messages = parse_cargo_messages(&output.stdout)?;

        // Check for errors
        if let Some(errors) = messages.errors() {
            let oxur_errors = self.translate_errors(errors)?;
            return Err(CompileError::RustcErrors(oxur_errors));
        }

        // Find artifact
        let artifact = messages.artifact_path()
            .ok_or(CompileError::NoArtifact)?;

        // Rename to unique name
        let unique = format!("libeval_{:03}.so", self.state.eval_counter);
        let renamed = self.session_dir.target_dir().join(unique);
        self.rename_or_copy(&artifact, &renamed)?;

        Ok(renamed)
    }

    async fn execute(&mut self, lib_path: &Path) -> Result<ExecResult> {
        // Ensure subprocess is running
        if self.subprocess.is_none() {
            self.subprocess = Some(self.spawn_subprocess().await?);
        }

        let subprocess = self.subprocess.as_mut().unwrap();

        // Send LOAD command
        subprocess.send_line(&format!(
            "LOAD {} {}",
            lib_path.display(),
            code.fn_name
        )).await?;

        // Capture output until completion
        let mut stdout = String::new();
        let mut stderr = String::new();

        loop {
            let line = subprocess.recv_line().await?;
            if line == "EVCXR_EXECUTION_COMPLETE" {
                break;
            }
            // Parse output markers
            // TODO: Implement output parsing
        }

        Ok(ExecResult { stdout, stderr, value: None })
    }

    fn translate_errors(
        &self,
        rustc_errors: &[CompilerMessage]
    ) -> Result<Vec<OxurError>> {
        // Use SourceMap to translate rustc errors to Oxur positions
        // See Section 8 for details
        todo!("Implement error translation")
    }

    fn rustflags(&self) -> String {
        if has_mold() {
            "-C link-arg=-fuse-ld=mold"
        } else if has_lld() {
            "-C link-arg=-fuse-ld=lld"
        } else {
            ""
        }.to_string()
    }

    pub fn is_cached(&self, hash: u64) -> bool {
        // Check if this Core Form hash has been compiled before
        // TODO: Implement cache lookup
        false
    }
}
```

**Integration Points:**

- Calls `CodeGenerator::generate()`
- Calls `cargo build` (external process)
- Communicates with subprocess via stdin/stdout
- Uses `SourceMap` for error translation

---

#### 4.1.3 CodeGenerator

**Location:** `oxur-repl/src/codegen/generator.rs`
**Ownership:** Owned by CachedCompiler

**Purpose:** Converts Core Forms to Rust source code.

**Core Methods:**

```rust
pub struct CodeGenerator {
    // Stateless; can be reused
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate(
        &self,
        form: &CoreForm,
        state: &SessionState
    ) -> Result<GeneratedCode> {
        // 1. Lower Core Forms to Rust AST
        let rust_ast = oxur_comp::lower(form)?;

        // 2. Wrap in function with VariableStore integration
        let fn_name = format!("run_user_code_{}", state.eval_counter);
        let wrapped_ast = self.wrap_in_function(rust_ast, state, &fn_name);

        // 3. Generate Rust source
        let source = oxur_ast::print_rust(&wrapped_ast);

        // 4. Add source map comments
        let source_with_maps = self.add_source_map_comments(source);

        Ok(GeneratedCode {
            source: source_with_maps,
            fn_name,
        })
    }

    fn wrap_in_function(
        &self,
        ast: RustAst,
        state: &SessionState,
        fn_name: &str
    ) -> RustAst {
        // Generate complete library with:
        // - VariableStore module
        // - Variable load code
        // - User code
        // - Variable store code
        todo!("Implement function wrapping")
    }

    fn add_source_map_comments(&self, source: String) -> String {
        // Insert /* oxur_node=N */ comments
        todo!("Implement source map comments")
    }
}
```

**Integration Points:**

- Calls `oxur_comp::lower()`
- Calls `oxur_ast::print_rust()`

---

#### 4.1.4 SessionDir

**Location:** `oxur-repl/src/session/dir.rs`
**Ownership:** Owned by CachedCompiler

**Purpose:** Manages temporary filesystem for a session.

```rust
pub struct SessionDir {
    root: PathBuf,
    session_id: SessionId,
}

impl SessionDir {
    pub fn new(session_id: &SessionId) -> Result<Self> {
        let root = PathBuf::from("/tmp/oxur-repl")
            .join(format!("session-{}", session_id));

        // Create directory structure
        std::fs::create_dir_all(&root)?;
        std::fs::create_dir_all(root.join("src"))?;

        // Write Cargo.toml
        let cargo_toml = include_str!("../templates/Cargo.toml");
        std::fs::write(root.join("Cargo.toml"), cargo_toml)?;

        Ok(Self {
            root,
            session_id: session_id.clone(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn src_path(&self) -> PathBuf {
        self.root.join("src/lib.rs")
    }

    pub fn target_dir(&self) -> PathBuf {
        self.root.join("target")
    }
}

impl Drop for SessionDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
```

---

#### 4.1.5 SourceMap

**Location:** `oxur-repl/src/source_map.rs`
**Ownership:** Shared by CachedCompiler (Arc)

**Purpose:** Tracks transformations for error translation.

```rust
pub struct SourceMap {
    surface_map: HashMap<NodeId, SourcePos>,
    core_to_surface: HashMap<NodeId, NodeId>,
    rust_to_core: HashMap<NodeId, NodeId>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            surface_map: HashMap::new(),
            core_to_surface: HashMap::new(),
            rust_to_core: HashMap::new(),
        }
    }

    pub fn lookup(&self, node_id: NodeId) -> Option<SourcePos> {
        // Traverse backwards: Rust → Core → Surface → SourcePos
        let core_id = self.rust_to_core.get(&node_id)?;
        let surface_id = self.core_to_surface.get(core_id)?;
        self.surface_map.get(surface_id).cloned()
    }

    pub fn add_surface_mapping(&mut self, node_id: NodeId, pos: SourcePos) {
        self.surface_map.insert(node_id, pos);
    }

    pub fn add_transformation(&mut self, from: NodeId, to: NodeId) {
        // Record transformation (e.g., Core → Rust)
        // Used during error translation
    }
}

pub struct SourcePos {
    pub file: String,
    pub line: u32,
    pub column: u32,
}
```

---

### 4.2 Subprocess Components

#### 4.2.1 Subprocess Runtime

**Location:** `oxur-subprocess/src/main.rs`
**Ownership:** Separate binary, spawned by CachedCompiler

**Purpose:** Isolated execution environment for user code.

```rust
// oxur-subprocess/main.rs

fn main() {
    let mut runtime = Runtime::new();
    runtime.run_loop();
}

struct Runtime {
    libraries: Vec<Library>,
    variable_store: Box<VariableStore>,
}

impl Runtime {
    fn new() -> Self {
        Self {
            libraries: Vec::new(),
            variable_store: Box::new(VariableStore::new()),
        }
    }

    fn run_loop(&mut self) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    continue;
                }
            };

            if let Err(e) = self.handle_command(&line) {
                eprintln!("ERROR: {}", e);
            }
        }
    }

    fn handle_command(&mut self, cmd: &str) -> Result<()> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        match parts.get(0) {
            Some(&"LOAD") => {
                let lib_path = parts.get(1).ok_or("Missing lib path")?;
                let fn_name = parts.get(2).ok_or("Missing function name")?;
                self.load_and_run(lib_path, fn_name)?;
                println!("EVCXR_EXECUTION_COMPLETE");
            }
            Some(&"PING") => {
                println!("PONG");
            }
            _ => {
                return Err(Error::UnknownCommand);
            }
        }
        Ok(())
    }

    fn load_and_run(&mut self, path: &str, name: &str) -> Result<()> {
        use libloading::Library;

        // Load library
        let lib = unsafe { Library::new(path)? };

        // Get function
        let func = unsafe {
            lib.get::<extern "C" fn(*mut c_void) -> *mut c_void>(
                name.as_bytes()
            )?
        };

        // Execute with VariableStore
        let store_ptr = &mut *self.variable_store as *mut _ as *mut c_void;
        unsafe {
            func(store_ptr);
        }

        // Keep library loaded
        self.libraries.push(lib);

        Ok(())
    }
}
```

---

#### 4.2.2 VariableStore

**Location:** `oxur-subprocess/src/variable_store.rs`
**Also:** Embedded in generated code (same implementation)

**Purpose:** Type-erased variable persistence.

*See ADR-001 for complete implementation.*

---

## 5. rustc Invocation Reference

### 5.1 The Cargo Command

```bash
# Primary invocation
cargo build \
  --target x86_64-unknown-linux-gnu \
  --message-format=json

# With environment
CARGO_TARGET_DIR=/tmp/oxur-session-abc/target
RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### 5.2 Cargo.toml

```toml
[package]
name = "ctx"
version = "1.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[profile.dev]
opt-level = 2
incremental = true
```

### 5.3 Platform Specifics

**Linux:**

- Extension: `.so`
- Linker: mold (preferred), lld, system default
- Command: Standard cargo

**macOS:**

- Extension: `.dylib`
- Linker: system (mold/lld not available)
- May need timestamp workaround

**Windows:**

- Extension: `.dll`
- No "lib" prefix
- Must COPY not rename (DLL locking)

---

## 6. File System Organization

```
/tmp/oxur-repl/
├── session-{uuid-1}/
│   ├── Cargo.toml
│   ├── src/lib.rs           # Generated code
│   ├── target/
│   │   └── debug/
│   │       ├── libctx.so
│   │       ├── libeval_001.so
│   │       └── incremental/
│   └── metadata.json
├── session-{uuid-2}/
│   └── ...
└── session-{uuid-3}/
    └── ...
```

**Lifecycle:**

- Created: On session create/clone
- Active: During session
- Cleaned: On session close or server shutdown
- Stale cleanup: >24h old dirs removed on startup

**Disk Space:**

- Per session: 30-100MB (incremental cache)
- 10-30 sessions = ~1GB total

---

## 7. Performance Expectations

| Operation | Target | Notes |
|-----------|--------|-------|
| Tier 1 eval (calc) | <1ms | Pure Rust arithmetic |
| Tier 2 first compile | 200-300ms | Cold, no cache |
| Tier 2 warm compile | 50-100ms | Incremental cache hit |
| Tier 2 cached (reused) | <10ms | Library already loaded |
| Session startup | <100ms | Create dir, spawn subprocess |
| Session cleanup | <50ms | Remove temp files |
| Library loading | 1-5ms | libloading dylib |

**Optimization Strategy:**

1. **Incremental compilation** - Always enabled (3-5x speedup)
2. **Fast linker** - Auto-detect mold/lld
3. **Tier 1 fast path** - <1ms for simple arithmetic
4. **Progress indicators** - Show for first compile (>200ms)

---

## 8. Error Handling Strategy

### 8.1 Error Translation Pipeline

```
rustc error (lib.rs:42:10)
  ↓ Parse cargo JSON
Structured error + span
  ↓ Extract Node ID from comment
Node ID 123
  ↓ Source map lookup
Oxur source position (test.ox:5:15)
  ↓ Format error
OxurError with context
```

### 8.2 Implementation

```rust
pub struct ErrorTranslator {
    source_map: Arc<SourceMap>,
}

impl ErrorTranslator {
    pub fn translate(&self, rustc_err: &CompilerMessage) -> OxurError {
        // 1. Extract span
        let span = rustc_err.primary_span()?;

        // 2. Read line, find Node ID
        let line = read_line(&span.file, span.line)?;
        let node_id = extract_node_id(&line)?;  // Parse /* oxur_node=N */

        // 3. Lookup original position
        let oxur_pos = self.source_map.lookup(node_id)?;

        // 4. Build Oxur error
        OxurError {
            message: rustc_err.message,
            file: oxur_pos.file,
            line: oxur_pos.line,
            col: oxur_pos.col,
            code: rustc_err.error_code,
            level: rustc_err.level,
        }
    }
}

fn extract_node_id(line: &str) -> Option<NodeId> {
    let re = Regex::new(r"/\* oxur_node=(\d+) \*/").unwrap();
    re.captures(line)?.get(1)?.as_str().parse().ok()
}
```

### 8.3 Cargo JSON Parsing

```rust
#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    message: Option<CompilerMessage>,
}

#[derive(Deserialize)]
struct CompilerMessage {
    message: String,
    code: Option<ErrorCode>,
    level: String,  // "error", "warning", "note"
    spans: Vec<Span>,
}

fn parse_cargo_errors(output: &str) -> Vec<CompilerMessage> {
    output.lines()
        .filter_map(|line| serde_json::from_str::<CargoMessage>(line).ok())
        .filter(|m| m.reason == "compiler-message")
        .filter_map(|m| m.message)
        .filter(|m| m.level == "error")
        .collect()
}
```

---

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[test]
fn test_variable_storage() {
    let mut store = VariableStore::new();
    store.put_variable("x", 42i32);
    assert!(store.check_variable::<i32>("x"));
    assert_eq!(store.take_variable::<i32>("x"), 42);
}

#[test]
fn test_calculator_mode() {
    let result = calculator_eval("(+ 1 2)").unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_code_generation() {
    let form = CoreForm::Literal(Literal::Int(42));
    let code = CodeGenerator::new().generate(&form, &vars, 1)?;
    assert!(code.source.contains("/* oxur_node="));
}
```

### 9.2 Integration Tests

```rust
#[tokio::test]
async fn test_full_session() {
    let mut compiler = CachedCompiler::new(SessionId::new(), project)?;

    // Define variable
    let form1 = parse("(def x 42)");
    compiler.eval(form1).await?;

    // Use variable
    let form2 = parse("(+ x 1)");
    let result = compiler.eval(form2).await?;

    assert_eq!(result.value, Some(DisplayValue::Text("43")));
}

#[tokio::test]
async fn test_compilation_error() {
    let mut compiler = CachedCompiler::new(SessionId::new(), project)?;

    // Reference undefined variable
    let form = parse("(+ x 1)");  // x not defined
    let result = compiler.eval(form).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("x"));
}
```

### 9.3 Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_calculator_matches_compilation(
        expr in arb_arithmetic()
    ) {
        let tier1 = calculator_eval(&expr)?;
        let tier2 = compile_and_eval(&expr)?;
        prop_assert_eq!(tier1, tier2);
    }
}
```

---

## 10. Implementation Roadmap

### Phase 1: Foundation (Week 1)

**Goal:** Basic compilation and execution

- [ ] Create project structure
- [ ] Implement VariableStore
- [ ] Create SessionDir management
- [ ] Write failing integration tests
- [ ] Build oxur-subprocess binary

**Deliverable:** Can compile and execute simple Rust code in subprocess

**Note:** Protocol layer (ODD-0018) already implemented in Phases 1-3.

### Phase 2: Code Generation (Week 2)

**Goal:** Oxur → Rust lowering

- [ ] Implement CodeGenerator
- [ ] Add source map tracking
- [ ] Generate wrapper functions
- [ ] Test with various Core Forms
- [ ] **Define integration APIs with oxur-lang, oxur-comp** *(NEW)*

**Deliverable:** Can lower Oxur Core Forms to Rust

**Blockers:** Requires Core Forms definition and oxur-comp lowering implementation.

### Phase 3: Compilation Integration (Week 3)

**Goal:** Cargo integration and caching

- [ ] Implement cargo invocation
- [ ] Parse JSON error output
- [ ] Implement incremental compilation
- [ ] Add unique library naming
- [ ] Test on all platforms

**Deliverable:** Fast compilation with incremental cache

### Phase 4: Error Translation (Week 4)

**Goal:** High-quality error messages

- [ ] Implement error parser
- [ ] Build source map lookup
- [ ] Translate rustc → Oxur positions
- [ ] Test with various error types

**Deliverable:** Errors point to Oxur source

### Phase 5: EvalContext Integration (Week 5) *(UPDATED)*

**Goal:** Connect compilation to protocol layer

- [ ] Implement EvalContext
- [ ] Integrate with CachedCompiler
- [ ] Connect to SessionManager
- [ ] Implement tier decision logic
- [ ] Test multi-session scenarios

**Deliverable:** Complete server-side evaluation pipeline

### Phase 6: Calculator Mode (Week 6)

**Goal:** <1ms evaluation

- [ ] Implement Tier 1 interpreter
- [ ] Pattern match literal arithmetic
- [ ] Benchmark performance
- [ ] Integration with Tier 2

**Deliverable:** Fast path for simple math

### Phase 7: End-to-End Integration (Week 7) *(UPDATED)*

**Goal:** Complete REPL system

- [ ] Connect MessageHandler to EvalContext
- [ ] Test full client→server→eval flow
- [ ] Verify protocol compliance
- [ ] Test error propagation through protocol

**Deliverable:** Working end-to-end REPL system

### Phase 8: Polish (Week 8)

**Goal:** Production ready

- [ ] Performance tuning
- [ ] Documentation
- [ ] Platform testing
- [ ] User testing
- [ ] v1.0 release

---

## 11. Dependencies and Versioning

```toml
[dependencies]
# Core
tokio = { version = "1.35", features = ["full"] }
libloading = "0.8"
tempfile = "3.8"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
postcard = { version = "1.0", features = ["alloc"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Other
regex = "1.10"
uuid = { version = "1.6", features = ["v4", "serde"] }

# Internal dependencies (NEW)
oxur-lang = { path = "../oxur-lang" }
oxur-comp = { path = "../oxur-comp" }
oxur-ast = { path = "../oxur-ast" }
```

**Feature Flags:**

```toml
[features]
default = []
fast-linker = []  # Auto-detect mold/lld
```

---

## 12. Risk Mitigation

### Risk 1: Multi-Session Resource Exhaustion

**Likelihood:** High | **Impact:** High

**Mitigation:**

- Limit 5 sessions per user
- 30min idle timeout
- Monitor subprocess count
- Subprocess pooling in v1.1

**Fallback:** Queue requests if resource limit hit

### Risk 2: Source Map Accuracy

**Likelihood:** Medium | **Impact:** High

**Mitigation:**

- Comprehensive source maps at each stage
- Node IDs in all generated code
- Fuzzy matching fallback
- Show both Rust and Oxur errors if uncertain

**Fallback:** Clear error message if translation fails

### Risk 3: Compilation Performance

**Likelihood:** High | **Impact:** Medium

**Mitigation:**

- Always use incremental compilation
- Fast linker auto-detection
- Progress indicators for >200ms
- Tier 1 provides instant feedback

**Fallback:** Accept delay, optimize v1.1

### Risk 4: Platform-Specific Issues

**Likelihood:** Medium | **Impact:** Medium

**Mitigation:**

- CI for Linux/macOS/Windows
- Platform-specific file operations
- Handle library extensions correctly

**Fallback:** Focus Linux first, others in v1.1

### Risk 5: Memory Growth

**Likelihood:** High | **Impact:** Low

**Mitigation:**

- Document to users
- Session restart command
- Monitor and warn
- Kill subprocess on close

**Fallback:** Accept as design trade-off

---

## 13. See Also

---

## 14. Revision History

### Changes from v1.0 to v1.1

**Date:** January 3, 2026

#### Major Additions

- **Section 2: Architecture Overview (NEW)** - High-level system architecture, component locations, compilation pipeline ownership, integration points
- **ADR-009: Client-Server Architecture (NEW)** - Formalizes server-side compilation decision, defines client vs server responsibilities
- **ADR-010: Component Ownership (NEW)** - Documents CachedCompiler ownership by EvalContext

#### Updates

- **Section 4: Component Implementation Specifications** - Added Location and Ownership fields to each component, clarified integration points with external crates
- **Section 10: Implementation Roadmap** - Phase 5 renamed to "EvalContext Integration", Phase 7 renamed to "End-to-End Integration", added blockers
- **Section 11: Dependencies** - Added internal dependencies: oxur-lang, oxur-comp, oxur-ast

#### Key Improvements

- Architecture clarity: explicit component locations and ownership
- Integration points: clear APIs defined for cross-crate dependencies
- References REPL Architecture Overview document for complete architectural details

## 15. Conclusion

This specification provides a complete, actionable blueprint for implementing Oxur's REPL based on proven patterns from evcxr, **now integrated with the complete client-server architecture defined in ODD-0018 and detailed in the REPL Architecture Overview**.

**Key Takeaways:**

1. **Adopt proven patterns** - evcxr validates our approach
2. **Server-side compilation** - All heavy lifting on server, thin client
3. **Clear component ownership** - CachedCompiler owned by EvalContext
4. **Well-defined integration points** - APIs specified for oxur-lang, oxur-comp, oxur-ast
5. **Simplify where possible** - Skip evcxr_runtime, rustc wrapper initially
6. **Invest in quality** - Source maps, error translation critical
7. **Ship iteratively** - v1.0 core, v1.1 optimizations

**Architecture Clarity:** See **REPL Architecture Overview** for:

- Complete system architecture diagrams
- Detailed data flow specifications
- Integration point details
- Session management architecture
- Protocol integration patterns

**Ready to implement.** 🚀

---

## 16. Appendix: Audit Summary

### Pattern Adoption Matrix

| Pattern | Source | Priority | Status | Notes |
|---------|--------|----------|--------|-------|
| Subprocess isolation | evcxr_repl | P0 | ✅ Adopt | Critical for safety |
| Type-erased storage | evcxr_repl | P0 | ✅ Adopt | Box<dyn Any> pattern |
| Cargo compilation | evcxr | P0 | ✅ Adopt | Better than rustc direct |
| Incremental compilation | evcxr | P0 | ✅ Adopt | 3-5x speedup |
| opt-level 2 | evcxr | P0 | ✅ Adopt | Balanced perf |
| Unique library naming | evcxr | P0 | ✅ Adopt | Windows compat |
| JSON error parsing | evcxr | P0 | ✅ Adopt | Structured errors |
| Clone-try-commit | evcxr_repl | P1 | 🔄 Adapt | Simplified version |
| rustc wrapper | evcxr | P2 | ⏸️ Defer | v1.1 optimization |
| Auto-fix errors | evcxr_repl | P2 | ❌ Skip | Too complex |
| evcxr_runtime | evcxr | P3 | ❌ Skip | Not needed |

### Key Metrics

| Metric | evcxr | Oxur Target | Status |
|--------|-------|-------------|--------|
| Cold compile | 200-300ms | 200-300ms | ✅ Match |
| Warm compile | 50-100ms | 50-100ms | ✅ Match |
| Calculator eval | N/A | <1ms | ✅ Better |
| Library loading | 1-5ms | 1-5ms | ✅ Match |
| Memory per session | 20-100MB | 30-100MB | ✅ Acceptable |

### Confidence Levels

**High Confidence (✅):**

- Subprocess architecture works
- Variable storage via Box<dyn Any> works
- Cargo compilation is viable
- Incremental compilation provides speedup
- Platform handling is well-understood

**Medium Confidence (⚠️):**

- Source map accuracy (needs testing)
- Multi-session resource management (new territory)
- Cache effectiveness (workload-dependent)

**Low Confidence (❓):**

- rustc wrapper necessity (measure first)
- Optimal session limits (user testing needed)

---

**Document Status:** Definitive Reference for Oxur REPL Implementation
**Version:** 1.1 (Updated with architecture clarifications)
**Next Steps:** Begin Phase 1 (Foundation) implementation