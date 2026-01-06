---
number: 30
title: "Oxur REPL Implementation Specification"
author: "Claude Code & Duncan McGreggor"
component: REPL
created: 2026-01-03
updated: 2026-01-06
state: Final
supersedes: null
superseded-by: null
version: 1.2
---


# Oxur REPL Implementation Specification

*Synthesized from evcxr Audits*

```
Version: 1.2
Date: January 5, 2026
Status: Definitive Reference
```

**IMPORTANT:** This document focuses on component implementation details. For the complete system architecture (client/server, data flow, integration points), see **ODD-0038: Oxur REPL Architecture (v1.2)**.

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

- Two-process execution model (SubprocessExecutor is MANDATORY)
- Type-erased variable storage (`Box<dyn Any>`)
- Cargo-based compilation
- Incremental compilation (always enabled)
- opt-level 0 for fastest REPL iteration
- JSON error parsing
- Unique library naming
- stdin/stdout IPC protocol

**BUILD ourselves:**

- Skip evcxr_runtime dependency - implement `OxurDisplay` with structured `DisplayValue` enum
- Source map integration via oxur-smap (Oxur → Rust error translation)
- Tier 1 calculator mode (<1ms for literal arithmetic)
- Network protocol integration (postcard serialization)
- Client/server architecture integration
- Artifact caching (Phase 0 - MANDATORY)
- Type inference via rust-analyzer (Phase 1)

**DEFER to v1.1:**

- rustc wrapper for dynamic linking optimization
- External dependency management `(require "crate")`
- Auto-fix compilation errors
- Async mode auto-detection
- Unix sockets for IPC (stdin/stdout sufficient for v1.0)

### 1.3 Risk Assessment

**High Priority Risks:**

1. **Multi-Session Resource Management** (High likelihood, High impact)
   - One subprocess per session could exhaust resources
   - Mitigation: Session limits (5 per user), 30min idle timeout, subprocess pooling in v1.1

2. **Source Map Accuracy** (Medium likelihood, High impact)
   - Mapping rustc errors to Oxur source requires careful tracking
   - Mitigation: Comprehensive source maps at each stage via oxur-smap, fuzzy matching fallback

3. **First Compilation Latency** (High likelihood, Medium impact)
   - Cold compile is 200-300ms
   - Mitigation: Progress indicators, artifact caching (Phase 0), Tier 1 for instant feedback

---

## 2. Architecture Overview

### 2.1 Purpose and Scope

This section provides a high-level overview of how the REPL components fit into the broader Oxur system. **For complete architectural details, see ODD-0038: Oxur REPL Architecture (v1.2)**, which includes:

- Complete client/server architecture
- Detailed component diagrams
- Full data flow specifications
- Integration point specifications
- Session management architecture
- Protocol integration patterns
- Three-tier execution strategy

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
3. **Subprocess Isolation** - Each session has its own subprocess for safety (MANDATORY)
4. **Protocol Integration** - Client/server communicate via ODD-0018 protocol
5. **Artifact Caching** - Content-based caching to avoid recompilation (MANDATORY Phase 0)

### 2.3 Component Locations

| Component | Crate | Module | Owned By | Description |
|-----------|-------|--------|----------|-------------|
| ReplClient | oxur-repl | client/ | User code | Protocol client |
| ReplServer | oxur-repl | server/ | Server process | TCP listener |
| MessageHandler | oxur-repl | server/ | ReplServer | Operation dispatcher |
| SessionManager | oxur-repl | server/ | ReplServer | Session lifecycle |
| EvalContext | oxur-repl | eval/ | SessionManager | Per-session state |
| CachedCompiler | oxur-repl | compiler/ | EvalContext | Compilation engine |
| RustAstWrapper | oxur-repl | wrapper.rs | CachedCompiler | Wraps lowered Rust AST |
| SessionDir | oxur-repl | session/ | CachedCompiler | Temp filesystem |
| ArtifactCache | oxur-repl | cache/ | EvalContext (shared) | Content-based caching |
| TypeInference | oxur-repl | type_inference.rs | CachedCompiler | Type inference via RA |
| SubprocessExecutor | oxur-repl | executor/ | CachedCompiler | Subprocess execution |
| Executor (trait) | oxur-repl | executor/mod.rs | - | Execution backend trait |
| VariableStore | oxur-repl | subprocess/variable_store.rs | Subprocess & Generated | Type-erased storage |
| Subprocess Runtime | oxur-repl | bin/subprocess.rs | - | Binary target for execution |

**Note on SourceMap Location:**

SourceMap functionality is provided by the **oxur-smap** crate, a foundation crate with no dependencies. This is imported as a dependency and used throughout the compilation pipeline for multi-stage error translation.

**Note on Subprocess:**

The subprocess is a **binary target** within the oxur-repl crate, not a separate crate. It's defined in `oxur-repl/src/bin/subprocess.rs` and built as `oxur-repl-subprocess` binary.

**Note on VariableStore:**

VariableStore exists in two places for ABI compatibility:
1. **Subprocess runtime** (`subprocess/variable_store.rs`) - Maintains state
2. **Generated code** - Embedded copy for ABI compatibility between subprocess and libraries

### 2.4 Compilation Pipeline Ownership

```
User Input: "(+ 1 2)"
  ↓
EvalContext (server)
  ├─→ oxur-lang::parse_lisp() → SurfaceForms
  ├─→ oxur-lang::expand() → CoreForms
  ├─→ Tier decision (Calculator / Cached / JIT)
  └─→ Check ArtifactCache (if Cached/JIT tier)
      ↓
CachedCompiler (server)
  ├─→ RustAstWrapper
  │   ├─→ oxur-comp::lower() → RustAst
  │   └─→ oxur-ast::print_rust() → Rust source
  ├─→ cargo build → dylib
  └─→ SubprocessExecutor: load & execute via stdin/stdout
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

**Required from oxur-smap:**

```rust
pub struct SourceMap {
    // Multi-stage source tracking
}

impl SourceMap {
    pub fn new() -> Self;
    pub fn lookup(&self, node_id: NodeId) -> Option<SourcePos>;
    pub fn add_surface_mapping(&mut self, node_id: NodeId, pos: SourcePos);
    pub fn add_transformation(&mut self, from: NodeId, to: NodeId);
}
```

**Status:** These APIs need to be verified or designed in their respective crates.

### 2.6 Session Architecture

Each session is completely isolated:

- **Filesystem:** Separate temp directory (best-effort tmpfs at `/dev/shm` on Linux, fallback to `/tmp`)
- **Process:** Separate subprocess (prevents crash propagation)
- **State:** Independent EvalContext, VariableStore, history, ArtifactCache
- **Concurrency:** Mutex on EvalContext (one eval at a time per session)

Sessions managed by SessionManager:

- Creates/destroys sessions
- Enforces limits (max sessions, timeouts)
- Thread-safe access (`Arc<RwLock<HashMap<SessionId, EvalContext>>>`)

**Temporary Directory Strategy:**

- Linux: `/dev/shm/oxur-repl/session-{uuid}/` (RAM-backed, best-effort)
- macOS/Windows: `/tmp/oxur-repl/session-{uuid}/` (OS cache handles it)
- User override: `OXUR_REPL_TEMP_DIR` environment variable

### 2.7 Three-Tier Execution Strategy

**Tier 1 - Calculator Mode (<1ms):**
- Direct evaluation for simple arithmetic
- No compilation required
- Patterns: `(+ 1 2)`, `(* 3.14 2)`

**Tier 2 - Cached (<10ms):**
- Previously compiled artifacts
- Content-based cache lookup
- Fast library loading only

**Tier 3 - JIT Compilation (50-300ms):**
- Full compilation pipeline
- First time for new code
- Incremental compilation benefits

See ODD-0038 Section 7 and ODD-0026 for complete three-tier strategy details.

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
- Variables must be owned (`Box<dyn Any + 'static>` requires ownership)
- No inter-variable references possible (aligns with Lisp semantics)

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

**Status:** Decided (UPDATED v1.2)

**Decision:** Use cargo as build orchestrator, not direct rustc. Use opt-level 0 for fastest REPL iteration.

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
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[profile.dev]
opt-level = 0        # Fastest REPL iteration (users can override if needed)
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
- **opt-level 0:** Prioritize compile time over runtime performance for interactive REPL
- **edition 2021:** Stable, widely supported (not bleeding-edge 2024)

---

### ADR-004: Temporary File Management

**Status:** Decided (UPDATED v1.2)

**Decision:** Per-session temporary directories with best-effort tmpfs, cleaned on close.

**Structure:**

```
# Linux (best-effort tmpfs)
/dev/shm/oxur-repl/session-{uuid}/
├── Cargo.toml
├── src/lib.rs
├── target/
│   └── {triple}/debug/
│       ├── libctx.so
│       ├── libeval_001.so  # Unique per eval
│       ├── libeval_002.so
│       └── incremental/
└── metadata.json

# macOS/Windows (fallback)
/tmp/oxur-repl/session-{uuid}/
└── (same structure)
```

**Temporary Directory Strategy:**

- **Linux:** Try `/dev/shm` first (RAM-backed, ~2-3% faster), fallback to `/tmp`
- **macOS/Windows:** Use `/tmp` or system temp (OS cache handles it well)
- **User override:** `OXUR_REPL_TEMP_DIR` environment variable
- **Zero configuration:** Works everywhere with graceful fallback

**Cleanup Strategy:**

- Normal close: Delete entire directory
- Server shutdown: Delete all session directories
- Startup: Clean stale dirs (>24h old)

**Disk Space:** 30-100MB per session (incremental cache)

---

### ADR-005: Error Translation and Source Mapping

**Status:** Decided (UPDATED v1.2)

**Decision:** Multi-stage source mapping with Node IDs via oxur-smap foundation crate.

**Process:**

```
Oxur Source (.ox:5:15)
  ↓ Node ID: 42 (tracked by oxur-smap)
Surface Forms
  ↓ Node ID: 43 (tracked by oxur-smap)
Core Forms
  ↓ Node ID: 44 (tracked by oxur-smap)
Rust AST
  ↓ Node ID: 45 (in comment, tracked by oxur-smap)
Generated Rust (lib.rs:123:10)
  ↓ rustc error
Parse error + Node ID
  ↓ oxur-smap lookup
Translate to Oxur position
```

**Generated Code Pattern:**

```rust
/* oxur_node=42 */ let x = /* oxur_node=43 */ 10 + /* oxur_node=44 */ 20;
```

**oxur-smap Integration:**

```rust
use oxur_smap::SourceMap;

pub struct ErrorTranslator {
    source_map: Arc<SourceMap>,  // From oxur-smap crate
}

impl ErrorTranslator {
    pub fn translate(&self, rustc_err: &CompilerMessage) -> OxurError {
        // Use oxur-smap to track and translate positions
        let node_id = extract_node_id_from_error(rustc_err)?;
        let oxur_pos = self.source_map.lookup(node_id)?;
        // Build Oxur error with original source position
    }
}
```

**Fallback:** If mapping fails, show Rust error with note about generated code.

**Unique Differentiator:** No other Lisp REPL has multi-stage source mapping this comprehensive.

---

### ADR-006: Code Generation Strategy

**Status:** Decided (UPDATED v1.2 - Component Renamed)

**Decision:** Generate complete Rust libraries with wrapper function per eval. Use RustAstWrapper (not CodeGenerator) to clarify that we wrap already-lowered Rust AST.

**Component Name:** `RustAstWrapper` (renamed from CodeGenerator for clarity)
**Rationale:** This component wraps already-lowered Rust AST with REPL scaffolding. It does NOT perform the lowering from Core Forms to Rust - that's done by oxur-comp.

**Template:**

```rust
// Generated src/lib.rs

mod oxur_variable_store {
    // VariableStore implementation (embedded for ABI compat)
}

#[no_mangle]
pub extern "C" fn run_user_code_5(
    mut store_ptr: *mut oxur_variable_store::VariableStore
) -> *mut oxur_variable_store::VariableStore {
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

**Note:** Module is named `oxur_variable_store` (not `evcxr_variable_store`) for Oxur branding.

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
**What to Share:** Subprocess, directory, source map, artifact cache (single instance)

---

### ADR-009: Client-Server Architecture

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
- All execution (SubprocessExecutor)
- Session management
- Protocol handling

**Consequences:**

- Client is simple and portable
- All heavy lifting on server
- Network latency per eval (acceptable for dev use)
- Resource management centralized

---

### ADR-010: Component Ownership

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
    compiler: CachedCompiler,          // ← Owned
    cache: Arc<Mutex<ArtifactCache>>,  // ← Shared, thread-safe
    history: Vec<HistoryEntry>,
    output_buffer: OutputBuffer,
}
```

**Note on ArtifactCache:** While owned by EvalContext in the ownership hierarchy, the cache is wrapped in `Arc<Mutex<...>>` for thread-safe shared access.

**Alternatives Considered:**

1. **Separate management** - More complex, no benefit
2. **Shared via Arc** - Unnecessary overhead for compiler itself, no concurrent access needed
3. **In MessageHandler** - Breaks session encapsulation

**Consequences:**

- Clear ownership model
- Simple drop semantics
- No Arc/Mutex overhead for compiler (cache uses Arc<Mutex> where needed)
- One compiler lifetime = one session lifetime

---

### ADR-011: Subprocess Execution (MANDATORY)

**Status:** Decided (UPDATED v1.2 - Clarified as MANDATORY)

**Decision:** SubprocessExecutor is MANDATORY for v1.0, not optional.

**Rationale:**

- **Rust threads cannot be interrupted** - Ctrl-C requires subprocess isolation
- evcxr evidence: Subprocess from day one, unchanged for 6+ years
- Proven stable architecture
- No viable in-process alternative for interactive interruption

**IPC Mechanism:** stdin/stdout text protocol

**Protocol Commands:**
- `LOAD_AND_RUN <path> <fn>` - Load library and execute function
- `OXUR_EXECUTION_COMPLETE` - Execution completed successfully
- `OXUR_RUNTIME_ERROR` - Runtime error occurred
- `OXUR_PANIC_LOCATION` - Panic with location info

**Note:** Protocol markers use `OXUR_*` prefix (not `EVCXR_*`) for Oxur branding.

**Executor Trait:** Kept for testing purposes only (InProcessExecutor for unit tests).

**Location:** `oxur-repl/src/executor/mod.rs`

**Consequences:**

- User code cannot crash REPL server
- Ctrl-C works reliably
- Session can restart after panic
- Memory isolation per session
- Slightly higher overhead (acceptable for interactive use)

---

### ADR-012: Artifact Caching (MANDATORY)

**Status:** Decided (NEW - v1.2)

**Decision:** Content-based artifact caching is MANDATORY for Phase 0.

**Rationale:**

- evcxr's biggest regret: Waited 5 years to add caching
- Dramatic performance improvement for repeated evaluations
- Enables Tier 2 (Cached) execution strategy
- Must be designed in from the start, hard to retrofit

**Cache Strategy:**

- **Location:** `~/.cache/oxur/artifacts/` (XDG Base Dir on Linux, equivalent on other platforms)
- **Cache Key:** SHA256(source + deps + opt_level + source_map)
- **Content:** Compiled dylib files
- **Invalidation:** Hash mismatch = recompile
- **Cleanup:** LRU with size limit (e.g., 1GB max)

**Implementation:**

```rust
pub struct ArtifactCache {
    cache_dir: PathBuf,
    index: HashMap<CacheKey, CacheEntry>,
}

pub struct CacheKey {
    source_hash: [u8; 32],  // SHA256
}

pub struct CacheEntry {
    artifact_path: PathBuf,
    created: SystemTime,
    last_used: SystemTime,
    size: u64,
}

impl ArtifactCache {
    pub fn get(&self, key: &CacheKey) -> Option<PathBuf> {
        // Return cached artifact if exists
    }

    pub fn put(&mut self, key: CacheKey, artifact: PathBuf) -> Result<()> {
        // Copy artifact to cache, update index
    }
}

// Type alias for shared cache (thread-safe)
pub type SharedCache = Arc<Mutex<ArtifactCache>>;
```

**Consequences:**

- Second eval of same code: <10ms (vs 50-300ms)
- Disk space usage: ~1GB for cache
- Complexity: Cache invalidation logic
- Performance: Massive improvement for common workflows

---

### ADR-013: Type Inference Strategy

**Status:** Decided (NEW - v1.2)

**Decision:** Use rust-analyzer from day one for type inference.

**Rationale:**

- evcxr spent 4 years with a compiler error hack (2018-2022)
- Hack was fragile and eventually removed (commit 5cbc3a0, 2022-08-28)
- rust-analyzer is now mature and available as a library
- Start with the right approach, avoid technical debt

**Implementation Strategy:**

- **Phase 1:** Use rust-analyzer as library (ra_ap_* crates)
- **Future:** Consider LSP if library approach has build time issues

**Location:** `oxur-repl/src/type_inference.rs`

**Integration:**

```rust
pub struct TypeInference {
    ra_context: RustAnalyzerContext,
}

impl TypeInference {
    pub fn infer_type(&self, code: &str, position: usize) -> Option<Type> {
        // Use rust-analyzer to infer type at position
    }
}
```

**Consequences:**

- Accurate type inference from day one
- Avoid 4 years of technical debt
- Slightly more complex setup
- Better user experience (e.g., for `:type` command)

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
    mode: ReplMode,                           // Lisp or Sexpr
    compiler: CachedCompiler,                 // Owned compilation engine
    cache: Arc<Mutex<ArtifactCache>>,         // Shared, thread-safe
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
            cache: Arc::new(Mutex::new(ArtifactCache::new()?)),
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
            Tier::Cached => {
                // Check cache first
                let cache_key = compute_cache_key(&core_forms);
                if let Some(artifact) = self.cache.lock().unwrap().get(&cache_key) {
                    self.compiler.execute_cached(artifact).await?
                } else {
                    self.compiler.eval(core_forms).await?
                }
            }
            Tier::Jit => {
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
        let cache_key = compute_cache_key(core_forms);
        if self.cache.lock().unwrap().get(&cache_key).is_some() {
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
- Uses `ArtifactCache` for tier 2
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
    session_dir: SessionDir,                    // Temp directory management
    state: SessionState,                        // Variable names/types, eval counter
    executor: SubprocessExecutor,               // MANDATORY - not Option
    wrapper: RustAstWrapper,                    // RENAMED from code_gen
    source_map: Arc<SourceMap>,                 // From oxur-smap
    type_inference: TypeInference,              // NEW - Phase 1
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
            executor: SubprocessExecutor::new(&session_dir)?,  // MANDATORY
            wrapper: RustAstWrapper::new(),                    // RENAMED
            source_map: Arc::new(SourceMap::new()),
            type_inference: TypeInference::new()?,
        })
    }

    pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
        // Clone-try-commit pattern
        let mut tentative_state = self.state.clone();
        tentative_state.eval_counter += 1;

        // Generate Rust code
        let code = self.wrapper.generate(&form, &tentative_state)?;

        // Compile
        let artifact = self.compile_to_dylib(&code).await?;

        // Execute via subprocess
        let result = self.executor.execute(&artifact).await?;

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

    fn translate_errors(
        &self,
        rustc_errors: &[CompilerMessage]
    ) -> Result<Vec<OxurError>> {
        // Use SourceMap (from oxur-smap) to translate rustc errors to Oxur positions
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
}
```

**Integration Points:**

- Calls `RustAstWrapper::generate()` (RENAMED from CodeGenerator)
- Calls `cargo build` (external process)
- Communicates with subprocess via `SubprocessExecutor`
- Uses `SourceMap` (from oxur-smap) for error translation
- Uses `TypeInference` for type inference

---

#### 4.1.3 RustAstWrapper

**Location:** `oxur-repl/src/wrapper.rs` (UPDATED from `codegen/generator.rs`)
**Ownership:** Owned by CachedCompiler

**Purpose:** Wraps already-lowered Rust AST with REPL scaffolding. Does NOT perform lowering itself.

**Name Rationale:** Previously called CodeGenerator, but renamed to RustAstWrapper to clarify that this component wraps already-lowered Rust AST (from oxur-comp) rather than generating code from scratch.

**Core Methods:**

```rust
pub struct RustAstWrapper {
    // Stateless; can be reused
}

impl RustAstWrapper {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate(
        &self,
        form: &CoreForm,
        state: &SessionState
    ) -> Result<GeneratedCode> {
        // 1. Lower Core Forms to Rust AST (via oxur-comp)
        let rust_ast = oxur_comp::lower(form)?;

        // 2. Wrap in function with VariableStore integration
        let fn_name = format!("run_user_code_{}", state.eval_counter);
        let wrapped_ast = self.wrap_in_function(rust_ast, state, &fn_name);

        // 3. Generate Rust source (via oxur-ast)
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
        // - oxur_variable_store module (NOT evcxr_variable_store)
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

- Calls `oxur_comp::lower()` - Receives already-lowered Rust AST
- Calls `oxur_ast::print_rust()` - Converts AST to source code
- Wraps with REPL scaffolding (VariableStore, function signature)

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
        // Try tmpfs first (Linux only), fallback to /tmp
        let base = if cfg!(target_os = "linux") && Path::new("/dev/shm").exists() {
            PathBuf::from("/dev/shm")
        } else {
            std::env::var("OXUR_REPL_TEMP_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir())
        };

        let root = base.join("oxur-repl")
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

#### 4.1.5 ArtifactCache

**Location:** `oxur-repl/src/cache/mod.rs`
**Ownership:** Shared by EvalContext (Arc<Mutex<...>>)

**Purpose:** Content-based caching of compiled artifacts.

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use sha2::{Sha256, Digest};

pub struct ArtifactCache {
    cache_dir: PathBuf,
    index: HashMap<CacheKey, CacheEntry>,
    total_size: u64,
    max_size: u64,  // e.g., 1GB
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CacheKey {
    source_hash: [u8; 32],  // SHA256
}

pub struct CacheEntry {
    artifact_path: PathBuf,
    created: SystemTime,
    last_used: SystemTime,
    size: u64,
}

impl ArtifactCache {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::cache_dir()?;
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            index: HashMap::new(),
            total_size: 0,
            max_size: 1024 * 1024 * 1024,  // 1GB
        })
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<PathBuf> {
        if let Some(entry) = self.index.get_mut(key) {
            entry.last_used = SystemTime::now();
            Some(entry.artifact_path.clone())
        } else {
            None
        }
    }

    pub fn put(&mut self, key: CacheKey, artifact: &Path) -> Result<()> {
        // Copy artifact to cache
        let cache_path = self.cache_dir.join(format!("{:x}", key.source_hash[0]));
        std::fs::copy(artifact, &cache_path)?;

        let size = std::fs::metadata(&cache_path)?.len();

        // Add to index
        let entry = CacheEntry {
            artifact_path: cache_path,
            created: SystemTime::now(),
            last_used: SystemTime::now(),
            size,
        };

        self.index.insert(key, entry);
        self.total_size += size;

        // Evict if over limit
        self.evict_if_needed()?;

        Ok(())
    }

    fn evict_if_needed(&mut self) -> Result<()> {
        while self.total_size > self.max_size {
            // Find LRU entry
            let oldest = self.index.iter()
                .min_by_key(|(_, e)| e.last_used)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest {
                let entry = self.index.remove(&key).unwrap();
                std::fs::remove_file(&entry.artifact_path)?;
                self.total_size -= entry.size;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn cache_dir() -> Result<PathBuf> {
        // XDG Base Directory on Linux, equivalent on other platforms
        let base = if cfg!(target_os = "linux") {
            std::env::var("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").expect("HOME not set");
                    PathBuf::from(home).join(".cache")
                })
        } else if cfg!(target_os = "macos") {
            let home = std::env::var("HOME").expect("HOME not set");
            PathBuf::from(home).join("Library/Caches")
        } else {
            // Windows
            PathBuf::from(std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set"))
        };

        Ok(base.join("oxur").join("artifacts"))
    }

    pub fn compute_key(
        source: &str,
        deps: &[String],
        opt_level: u8,
        source_map: &[u8]
    ) -> CacheKey {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        for dep in deps {
            hasher.update(dep.as_bytes());
        }
        hasher.update(&[opt_level]);
        hasher.update(source_map);

        let hash = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash[..]);

        CacheKey { source_hash: key }
    }
}

// Type alias for shared cache
pub type SharedCache = Arc<Mutex<ArtifactCache>>;
```

---

#### 4.1.6 TypeInference

**Location:** `oxur-repl/src/type_inference.rs`
**Ownership:** Owned by CachedCompiler

**Purpose:** Type inference using rust-analyzer.

```rust
use ra_ap_hir::Semantics;
use ra_ap_ide::Analysis;
use ra_ap_syntax::SourceFile;

pub struct TypeInference {
    // rust-analyzer context
    analysis: Analysis,
}

impl TypeInference {
    pub fn new() -> Result<Self> {
        // Initialize rust-analyzer
        // Phase 1 implementation
        todo!("Implement rust-analyzer integration")
    }

    pub fn infer_type(&self, code: &str, position: usize) -> Option<String> {
        // Use rust-analyzer to infer type at position
        // Returns type as string (e.g., "i32", "Vec<String>")
        todo!("Implement type inference")
    }
}
```

**Integration:**

- Used by CachedCompiler for `:type` command
- Helps with variable type tracking
- Future: Auto-import suggestions

---

### 4.2 Subprocess Components

#### 4.2.1 SubprocessExecutor

**Location:** `oxur-repl/src/executor/subprocess.rs`
**Ownership:** Owned by CachedCompiler
**Trait:** Implements `Executor` trait (defined in `executor/mod.rs`)

**Purpose:** Manages subprocess execution and communication.

```rust
pub struct SubprocessExecutor {
    subprocess: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout_reader: Option<BufReader<ChildStdout>>,
}

impl SubprocessExecutor {
    pub fn new(session_dir: &SessionDir) -> Result<Self> {
        Ok(Self {
            subprocess: None,
            stdin: None,
            stdout_reader: None,
        })
    }

    pub fn ensure_running(&mut self) -> Result<()> {
        if self.subprocess.is_none() {
            let mut child = Command::new("oxur-repl-subprocess")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            self.stdin = Some(child.stdin.take().unwrap());
            self.stdout_reader = Some(BufReader::new(child.stdout.take().unwrap()));
            self.subprocess = Some(child);
        }
        Ok(())
    }

    pub async fn execute(&mut self, artifact: &Path) -> Result<ExecResult> {
        self.ensure_running()?;

        // Send LOAD_AND_RUN command
        let cmd = format!("LOAD_AND_RUN {} run_user_code", artifact.display());
        writeln!(self.stdin.as_mut().unwrap(), "{}", cmd)?;

        // Capture output
        let mut stdout = String::new();
        let mut stderr = String::new();

        loop {
            let mut line = String::new();
            self.stdout_reader.as_mut().unwrap().read_line(&mut line)?;

            if line.trim() == "OXUR_EXECUTION_COMPLETE" {
                break;
            } else if line.trim().starts_with("OXUR_RUNTIME_ERROR") {
                return Err(Error::RuntimeError(stderr));
            } else if line.trim().starts_with("OXUR_PANIC_LOCATION") {
                return Err(Error::Panic(line));
            } else {
                stdout.push_str(&line);
            }
        }

        Ok(ExecResult {
            stdout,
            stderr,
            value: None,
        })
    }
}

impl Drop for SubprocessExecutor {
    fn drop(&mut self) {
        if let Some(mut child) = self.subprocess.take() {
            let _ = child.kill();
        }
    }
}
```

**Protocol Commands (Oxur-branded):**
- `LOAD_AND_RUN <path> <fn>` - Load library and execute
- `OXUR_EXECUTION_COMPLETE` - Success marker
- `OXUR_RUNTIME_ERROR` - Error marker
- `OXUR_PANIC_LOCATION` - Panic marker

---

#### 4.2.2 Executor Trait

**Location:** `oxur-repl/src/executor/mod.rs`
**Purpose:** Abstraction for execution backends (testing vs production).

```rust
pub trait Executor {
    async fn execute(&mut self, artifact: &Path) -> Result<ExecResult>;
}

pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub value: Option<DisplayValue>,
}

// Production implementation
impl Executor for SubprocessExecutor {
    async fn execute(&mut self, artifact: &Path) -> Result<ExecResult> {
        // Full subprocess execution
    }
}

// Test-only implementation
#[cfg(test)]
pub struct InProcessExecutor;

#[cfg(test)]
impl Executor for InProcessExecutor {
    async fn execute(&mut self, artifact: &Path) -> Result<ExecResult> {
        // Direct dlopen for testing (no Ctrl-C support)
    }
}
```

---

#### 4.2.3 Subprocess Runtime

**Location:** `oxur-repl/src/bin/subprocess.rs` (UPDATED - binary target)
**Build:** Binary target within oxur-repl crate, built as `oxur-repl-subprocess`

**Cargo.toml Configuration:**

```toml
[[bin]]
name = "oxur-repl-subprocess"
path = "src/bin/subprocess.rs"
```

**Purpose:** Isolated execution environment for user code.

```rust
// oxur-repl/src/bin/subprocess.rs

use std::io::{self, BufRead};

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
            Some(&"LOAD_AND_RUN") => {
                let lib_path = parts.get(1).ok_or("Missing lib path")?;
                let fn_name = parts.get(2).ok_or("Missing function name")?;
                self.load_and_run(lib_path, fn_name)?;
                println!("OXUR_EXECUTION_COMPLETE");  // Oxur-branded marker
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

#### 4.2.4 VariableStore

**Location:** `oxur-repl/src/subprocess/variable_store.rs` (UPDATED)
**Also:** Embedded in generated code (same implementation)

**Purpose:** Type-erased variable persistence with dual-location architecture.

**Dual-Location Architecture:**

1. **Subprocess runtime** (`subprocess/variable_store.rs`) - Maintains state across evaluations
2. **Generated code** - Embedded copy for ABI compatibility between subprocess and dynamically loaded libraries

Both locations use identical code to ensure ABI compatibility.

**Module Name:** `oxur_variable_store` (Oxur-branded, not `evcxr_variable_store`)

*See ADR-001 for complete implementation.*

**Key Point:** The VariableStore implementation must be byte-for-byte identical in both locations to maintain ABI compatibility across the FFI boundary.

---

## 5. rustc Invocation Reference

### 5.1 The Cargo Command

```bash
# Primary invocation
cargo build \
  --target x86_64-unknown-linux-gnu \
  --message-format=json

# With environment
CARGO_TARGET_DIR=/path/to/session/target
RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### 5.2 Cargo.toml

```toml
[package]
name = "ctx"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[profile.dev]
opt-level = 0        # Fastest REPL iteration
incremental = true   # 3-5x speedup
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

### 6.1 Session Directory Structure

```
# Linux (best-effort tmpfs)
/dev/shm/oxur-repl/session-{uuid}/
├── Cargo.toml
├── src/lib.rs           # Generated code
├── target/
│   └── debug/
│       ├── libctx.so
│       ├── libeval_001.so
│       └── incremental/
└── metadata.json

# macOS/Windows (fallback)
/tmp/oxur-repl/session-{uuid}/
└── (same structure)
```

### 6.2 Artifact Cache Structure

```
# Linux
~/.cache/oxur/artifacts/
├── a3/
│   └── a3f8b2c1... (SHA256 hash)
├── b4/
│   └── b4e9c3d2...
└── cache_index.json

# macOS
~/Library/Caches/oxur/artifacts/
└── (same structure)

# Windows
%LOCALAPPDATA%\oxur\artifacts\
└── (same structure)
```

### 6.3 Lifecycle

- Created: On session create/clone
- Active: During session
- Cleaned: On session close or server shutdown
- Stale cleanup: >24h old dirs removed on startup

### 6.4 Disk Space

- Per session: 30-100MB (incremental cache)
- Artifact cache: ~1GB (configurable)
- 10-30 sessions + cache = ~2GB total

### 6.5 Environment Variables

- `OXUR_REPL_TEMP_DIR` - Override temp directory location
- `XDG_CACHE_HOME` - Override cache directory (Linux)

---

## 7. Performance Expectations

| Operation | Target | Notes |
|-----------|--------|-------|
| Tier 1 eval (calc) | <1ms | Pure Rust arithmetic |
| Tier 2 first compile | 200-300ms | Cold, no cache |
| Tier 2 warm compile | 50-100ms | Incremental cache hit |
| Tier 2 cached (reused) | <10ms | Artifact cache hit |
| Session startup | <100ms | Create dir, spawn subprocess |
| Session cleanup | <50ms | Remove temp files |
| Library loading | 1-5ms | libloading dylib |

**Optimization Strategy:**

1. **Incremental compilation** - Always enabled (3-5x speedup)
2. **Artifact caching** - Content-based caching (Phase 0, MANDATORY)
3. **Fast linker** - Auto-detect mold/lld
4. **Tier 1 fast path** - <1ms for simple arithmetic
5. **tmpfs on Linux** - Best-effort RAM-backed filesystem (~2-3% faster)
6. **Progress indicators** - Show for first compile (>200ms)

---

## 8. Error Handling Strategy

### 8.1 Error Translation Pipeline

```
rustc error (lib.rs:42:10)
  ↓ Parse cargo JSON
Structured error + span
  ↓ Extract Node ID from comment
Node ID 123
  ↓ oxur-smap lookup
Oxur source position (test.ox:5:15)
  ↓ Format error
OxurError with context
```

### 8.2 Implementation

```rust
use oxur_smap::SourceMap;

pub struct ErrorTranslator {
    source_map: Arc<SourceMap>,  // From oxur-smap crate
}

impl ErrorTranslator {
    pub fn translate(&self, rustc_err: &CompilerMessage) -> OxurError {
        // 1. Extract span
        let span = rustc_err.primary_span()?;

        // 2. Read line, find Node ID
        let line = read_line(&span.file, span.line)?;
        let node_id = extract_node_id(&line)?;  // Parse /* oxur_node=N */

        // 3. Lookup original position via oxur-smap
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
fn test_rust_ast_wrapping() {
    let form = CoreForm::Literal(Literal::Int(42));
    let wrapper = RustAstWrapper::new();
    let code = wrapper.generate(&form, &vars, 1)?;
    assert!(code.source.contains("/* oxur_node="));
    assert!(code.source.contains("oxur_variable_store"));  // Not evcxr_variable_store
}

#[test]
fn test_artifact_cache() {
    let mut cache = ArtifactCache::new().unwrap();
    let key = ArtifactCache::compute_key("source", &[], 0, &[]);

    // Miss
    assert!(cache.get(&key).is_none());

    // Put
    cache.put(key.clone(), Path::new("artifact.so")).unwrap();

    // Hit
    assert!(cache.get(&key).is_some());
}
```

### 9.2 Integration Tests

```rust
#[tokio::test]
async fn test_full_session() {
    let mut compiler = CachedCompiler::new(SessionId::new())?;

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
    let mut compiler = CachedCompiler::new(SessionId::new())?;

    // Reference undefined variable
    let form = parse("(+ x 1)");  // x not defined
    let result = compiler.eval(form).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("x"));
}

#[tokio::test]
async fn test_subprocess_isolation() {
    let mut compiler = CachedCompiler::new(SessionId::new())?;

    // Cause panic in subprocess
    let form = parse("(panic \"test\")");
    let result = compiler.eval(form).await;

    assert!(result.is_err());

    // Subprocess should restart, session should continue
    let form2 = parse("(+ 1 2)");
    let result2 = compiler.eval(form2).await?;
    assert_eq!(result2.value, Some(DisplayValue::Text("3")));
}

#[tokio::test]
async fn test_artifact_cache_integration() {
    let mut eval_ctx = EvalContext::new(SessionId::new())?;

    // First eval - compiles
    let start = Instant::now();
    eval_ctx.eval("(+ 1 2)").await?;
    let first_duration = start.elapsed();

    // Second eval - cached
    let start = Instant::now();
    eval_ctx.eval("(+ 1 2)").await?;
    let second_duration = start.elapsed();

    // Cached should be much faster
    assert!(second_duration < first_duration / 5);
    assert!(second_duration.as_millis() < 20);
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

    #[test]
    fn test_cache_key_stability(
        source in ".*",
        opt_level in 0u8..3
    ) {
        let key1 = ArtifactCache::compute_key(&source, &[], opt_level, &[]);
        let key2 = ArtifactCache::compute_key(&source, &[], opt_level, &[]);
        prop_assert_eq!(key1, key2);
    }
}
```

---

## 10. Implementation Roadmap

### Phase 0: Foundation & Caching (Weeks 1-2) **(UPDATED - NEW PHASE)**

**Goal:** Core infrastructure with artifact caching

- [ ] Create project structure
- [ ] Implement VariableStore (dual-location architecture)
- [ ] Create SessionDir management (with tmpfs support)
- [ ] **Implement ArtifactCache (MANDATORY)**
- [ ] Build oxur-repl-subprocess binary target
- [ ] Write failing integration tests

**Deliverable:** Foundation ready with caching infrastructure

**Rationale:** Artifact caching must be designed in from the start (evcxr's 5-year regret).

**Dependencies:** 
- oxur-smap crate (for source mapping foundation)

### Phase 1: Subprocess Execution (Week 3)

**Goal:** Reliable subprocess execution with stdin/stdout protocol

- [ ] Implement SubprocessExecutor (MANDATORY, not optional)
- [ ] Implement Executor trait
- [ ] Test subprocess spawn/communication
- [ ] Test OXUR_* protocol markers
- [ ] Test subprocess isolation (crash recovery)

**Deliverable:** Can execute code in isolated subprocess

**Note:** SubprocessExecutor is MANDATORY for v1.0 (Ctrl-C requires subprocess).

### Phase 2: Code Generation (Week 4)

**Goal:** Oxur → Rust lowering with REPL scaffolding

- [ ] Implement RustAstWrapper (RENAMED from CodeGenerator)
- [ ] Add source map tracking (via oxur-smap)
- [ ] Generate wrapper functions with oxur_variable_store
- [ ] Test with various Core Forms
- [ ] **Define integration APIs with oxur-lang, oxur-comp**

**Deliverable:** Can wrap lowered Rust AST with REPL scaffolding

**Blockers:** Requires Core Forms definition and oxur-comp lowering implementation.

### Phase 3: Compilation Integration (Week 5)

**Goal:** Cargo integration with caching

- [ ] Implement cargo invocation
- [ ] Parse JSON error output
- [ ] Implement incremental compilation
- [ ] Add unique library naming
- [ ] Integrate with ArtifactCache
- [ ] Test on all platforms

**Deliverable:** Fast compilation with incremental and artifact caching

### Phase 4: Error Translation (Week 6)

**Goal:** High-quality error messages via oxur-smap

- [ ] Implement error parser
- [ ] Build source map lookup (via oxur-smap)
- [ ] Translate rustc → Oxur positions
- [ ] Test with various error types
- [ ] Test fuzzy matching fallback

**Deliverable:** Errors point to Oxur source locations

### Phase 5: EvalContext Integration (Week 7)

**Goal:** Connect compilation to session management

- [ ] Implement EvalContext
- [ ] Integrate with CachedCompiler
- [ ] Connect to SessionManager
- [ ] Implement tier decision logic
- [ ] Test multi-session scenarios

**Deliverable:** Complete session-based evaluation

### Phase 6: Calculator Mode (Week 8)

**Goal:** <1ms evaluation for simple arithmetic

- [ ] Implement Tier 1 interpreter
- [ ] Pattern match literal arithmetic
- [ ] Benchmark performance (<1ms target)
- [ ] Integration with tier decision

**Deliverable:** Fast path for simple math

### Phase 7: Type Inference (Week 9) **(NEW)**

**Goal:** Type inference via rust-analyzer

- [ ] Integrate rust-analyzer as library
- [ ] Implement TypeInference component
- [ ] Add `:type` command support
- [ ] Test type inference accuracy

**Deliverable:** Type inference for REPL commands

### Phase 8: End-to-End Integration (Week 10)

**Goal:** Complete REPL system

- [ ] Connect MessageHandler to EvalContext
- [ ] Test full client→server→eval flow
- [ ] Verify protocol compliance (ODD-0018)
- [ ] Test error propagation through protocol
- [ ] Test artifact cache across sessions

**Deliverable:** Working end-to-end REPL system

### Phase 9: Polish (Week 11)

**Goal:** Production ready

- [ ] Performance tuning
- [ ] Documentation
- [ ] Platform testing (Linux/macOS/Windows)
- [ ] User testing
- [ ] v1.0 release

**Total Timeline:** 11 weeks (was 8 weeks in v1.1)

**Timeline Increase Rationale:**
- Phase 0: +1 week for artifact caching infrastructure
- Phase 7: +1 week for type inference
- Phase 8-9: Renumbered, +1 week for comprehensive testing

**Trade-off:** Avoid 5 years of caching regret, 4 years of type inference hacks.

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

# Hashing for cache keys
sha2 = "0.10"

# Regex for error parsing
regex = "1.10"
uuid = { version = "1.6", features = ["v4", "serde"] }

# Type inference (Phase 7)
ra_ap_hir = "0.0.200"
ra_ap_ide = "0.0.200"
ra_ap_syntax = "0.0.200"

# Internal dependencies
oxur-lang = { path = "../oxur-lang" }
oxur-comp = { path = "../oxur-comp" }
oxur-ast = { path = "../oxur-ast" }
oxur-smap = { path = "../oxur-smap" }  # NEW - source mapping foundation
```

**Cargo.toml for Subprocess Binary:**

```toml
[[bin]]
name = "oxur-repl-subprocess"
path = "src/bin/subprocess.rs"
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

- Use oxur-smap for comprehensive multi-stage tracking
- Node IDs in all generated code
- Fuzzy matching fallback
- Show both Rust and Oxur errors if uncertain

**Fallback:** Clear error message if translation fails

### Risk 3: Compilation Performance

**Likelihood:** High | **Impact:** Medium

**Mitigation:**

- Always use incremental compilation
- Artifact caching (Phase 0, MANDATORY)
- Fast linker auto-detection
- tmpfs on Linux (best-effort)
- Progress indicators for >200ms
- Tier 1 provides instant feedback

**Fallback:** Accept delay, optimize v1.1

### Risk 4: Platform-Specific Issues

**Likelihood:** Medium | **Impact:** Medium

**Mitigation:**

- CI for Linux/macOS/Windows
- Platform-specific file operations
- Handle library extensions correctly
- Windows DLL locking workarounds

**Fallback:** Focus Linux first, others in v1.1

### Risk 5: Memory Growth

**Likelihood:** High | **Impact:** Low

**Mitigation:**

- Document to users
- Session restart command
- Monitor and warn
- Kill subprocess on close

**Fallback:** Accept as design trade-off

### Risk 6: Artifact Cache Invalidation

**Likelihood:** Medium | **Impact:** Medium

**Mitigation:**

- Comprehensive cache keys (source + deps + opt_level + source_map)
- SHA256 for collision resistance
- LRU eviction with size limits
- Clear cache command for users

**Fallback:** Cache miss just means recompile

---

## 13. See Also

**Required Reading:**

- **ODD-0038: Oxur REPL Architecture (v1.2)** - Complete system architecture
- **ODD-0018: Oxur Remote REPL Protocol Design** - Network protocol specification
- **ODD-0026: Oxur REPL Evaluation Strategy** - Three-tier execution strategy
- **ODD-0013: Oxur Compilation Chain Architecture** - Compilation pipeline context

**Foundation Crates:**

- **oxur-smap** - Multi-stage source mapping for error translation
- **oxur-lang** - Parsing and macro expansion
- **oxur-comp** - Core Forms → Rust AST lowering
- **oxur-ast** - Rust AST → source code generation

**Research Documents:**

- `evcxr-research-synthesis.md` - 6+ years of evcxr analysis, patterns, and lessons learned

---

## 14. Revision History

### Version 1.2 (2026-01-05)

**Major Changes - Alignment with ODD-0038 v1.2:**

1. **Component Rename: CodeGenerator → RustAstWrapper**
   - Clarifies that component wraps already-lowered Rust AST
   - Updated throughout document
   - Location updated: `wrapper.rs` (was `codegen/generator.rs`)

2. **Protocol Markers Rebranded**
   - All protocol markers now use `OXUR_*` prefix
   - `OXUR_EXECUTION_COMPLETE`, `OXUR_RUNTIME_ERROR`, `OXUR_PANIC_LOCATION`
   - Removed `EVCXR_*` references

3. **VariableStore Module Rebranded**
   - Module name: `oxur_variable_store` (not `evcxr_variable_store`)
   - Location clarified: `oxur-repl/src/subprocess/variable_store.rs`
   - Dual-location architecture documented (subprocess + generated code)

4. **Subprocess Binary Location Clarified**
   - Binary target within `oxur-repl` crate (not separate crate)
   - Location: `oxur-repl/src/bin/subprocess.rs`
   - Built as: `oxur-repl-subprocess`
   - Added Cargo.toml `[[bin]]` configuration

5. **Optimization Level Updated**
   - Changed from `opt-level = 2` to `opt-level = 0`
   - Rationale: Prioritize REPL iteration speed over runtime performance
   - Updated ADR-003

6. **Rust Edition Updated**
   - Changed from `edition = "2024"` to `edition = "2021"`
   - Rationale: Stable, widely supported

7. **ArtifactCache Thread Safety Clarified**
   - Specified `Arc<Mutex<ArtifactCache>>` for shared access
   - Added type alias: `SharedCache = Arc<Mutex<ArtifactCache>>`

8. **oxur-smap Integration**
   - SourceMap now from `oxur-smap` foundation crate
   - Updated component table
   - Added to dependencies

9. **Subprocess Execution Status**
   - Clarified as MANDATORY (not optional) in ADR-011
   - SubprocessExecutor required for Ctrl-C support
   - InProcessExecutor for testing only

10. **Artifact Caching Elevated to Phase 0**
    - Now MANDATORY for v1.0 (was future consideration)
    - Added ADR-012
    - Updated roadmap (new Phase 0)

11. **Type Inference Added**
    - New component: TypeInference
    - Added ADR-013
    - New Phase 7 in roadmap

12. **Temporary Directory Strategy**
    - Best-effort tmpfs on Linux (`/dev/shm`)
    - Fallback to `/tmp` on macOS/Windows
    - Environment variable: `OXUR_REPL_TEMP_DIR`
    - Updated ADR-004

13. **Component Locations Updated**
    - Added explicit file paths for all components
    - Clarified ownership hierarchy
    - Added Executor trait location

14. **Cross-References Updated**
    - Added explicit ODD numbers
    - References to ODD-0038 v1.2
    - References to oxur-smap

15. **Roadmap Timeline Extended**
    - 11 weeks total (was 8 weeks)
    - New Phase 0 for caching
    - New Phase 7 for type inference
    - Rationale: Avoid years of technical debt

**Impact:** Significant alignment with architecture v1.2. Core implementation approach unchanged, but branding, naming, and priorities updated.

---

### Version 1.1 (2026-01-04)

**Major Additions:**

- Section 2: Architecture Overview (NEW)
- ADR-009: Client-Server Architecture (NEW)
- ADR-010: Component Ownership (NEW)

**Updates:**

- Section 4: Added Location and Ownership fields
- Section 10: Phase 5/7 renamed for clarity
- Section 11: Added internal dependencies

**Key Improvements:**

- Architecture clarity with component locations
- Integration points for cross-crate dependencies
- References to REPL Architecture Overview

---

### Version 1.0 (2026-01-03)

Initial specification based on evcxr audits.

---

## 15. Conclusion

This specification provides a complete, actionable blueprint for implementing Oxur's REPL based on proven patterns from evcxr, **now fully aligned with ODD-0038: Oxur REPL Architecture (v1.2)**.

**Key Takeaways:**

1. **Adopt proven patterns** - evcxr validates our approach (6+ years stable)
2. **Server-side compilation** - All heavy lifting on server, thin client
3. **Subprocess execution MANDATORY** - Required for Ctrl-C, not optional
4. **Clear component ownership** - CachedCompiler owned by EvalContext
5. **Artifact caching from day one** - Phase 0, avoid 5 years of regret
6. **Type inference from day one** - rust-analyzer, avoid 4 years of hacks
7. **Multi-stage source mapping** - oxur-smap for rustc-quality errors
8. **Well-defined integration points** - APIs specified for all external crates
9. **Oxur branding throughout** - oxur_variable_store, OXUR_* markers
10. **Ship iteratively** - v1.0 core (11 weeks), v1.1 optimizations

**Architecture Reference:** See **ODD-0038: Oxur REPL Architecture (v1.2)** for:

- Complete system architecture diagrams
- Detailed data flow specifications
- Integration point details
- Session management architecture
- Protocol integration patterns
- Three-tier execution strategy

**Ready to implement.** 🚀

---

## 16. Appendix: Audit Summary

### Pattern Adoption Matrix

| Pattern | Source | Priority | Status | Notes |
|---------|--------|----------|--------|-------|
| Subprocess isolation | evcxr_repl | P0 | ✅ Adopt | MANDATORY (Ctrl-C) |
| Type-erased storage | evcxr_repl | P0 | ✅ Adopt | Box<dyn Any> pattern |
| Cargo compilation | evcxr | P0 | ✅ Adopt | Better than rustc direct |
| Incremental compilation | evcxr | P0 | ✅ Adopt | 3-5x speedup |
| Artifact caching | evcxr (late add) | P0 | ✅ Adopt | Phase 0, MANDATORY |
| opt-level 0 | REPL best practice | P0 | ✅ Adopt | Fastest iteration |
| stdin/stdout IPC | evcxr | P0 | ✅ Adopt | 6 years stable |
| Unique library naming | evcxr | P0 | ✅ Adopt | Windows compat |
| JSON error parsing | evcxr | P0 | ✅ Adopt | Structured errors |
| rust-analyzer types | Modern approach | P1 | ✅ Adopt | Phase 1 |
| Clone-try-commit | evcxr_repl | P1 | 🔄 Adapt | Simplified version |
| tmpfs on Linux | Performance | P1 | 🔄 Adapt | Best-effort |
| rustc wrapper | evcxr | P2 | ⏸️ Defer | v1.1 optimization |
| Auto-fix errors | evcxr_repl | P2 | ❌ Skip | Too complex |
| evcxr_runtime | evcxr | P3 | ❌ Skip | Not needed |

### Key Metrics

| Metric | evcxr | Oxur Target | Status |
|--------|-------|-------------|--------|
| Cold compile | 200-300ms | 200-300ms | ✅ Match |
| Warm compile | 50-100ms | 50-100ms | ✅ Match |
| Cached (artifact) | N/A | <10ms | ✅ Better |
| Calculator eval | N/A | <1ms | ✅ Better |
| Library loading | 1-5ms | 1-5ms | ✅ Match |
| Memory per session | 20-100MB | 30-100MB | ✅ Acceptable |

### Confidence Levels

**High Confidence (✅):**

- Subprocess architecture works (6+ years proven)
- Variable storage via Box<dyn Any> works
- Cargo compilation is viable
- Incremental compilation provides speedup
- stdin/stdout IPC is stable
- Artifact caching provides major performance win
- Platform handling is well-understood

**Medium Confidence (⚠️):**

- Source map accuracy (needs testing with oxur-smap)
- Multi-session resource management (new territory)
- rust-analyzer integration (modern approach)

**Low Confidence (❓):**

- rustc wrapper necessity (measure first, defer to v1.1)
- Optimal session limits (user testing needed)

---

**Document Status:** Definitive Reference for Oxur REPL Implementation (v1.2)
**Aligned With:** ODD-0038: Oxur REPL Architecture v1.2 (2026-01-05)
**Next Steps:** Begin Phase 0 (Foundation & Caching) implementation