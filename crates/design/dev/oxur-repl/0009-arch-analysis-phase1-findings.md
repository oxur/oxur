# REPL Architecture Analysis - Phase 1 Findings

**Session 1 - Document Review Phase**  
**Date:** January 3, 2026

## Executive Summary

After reviewing ODD-0013 (Compilation Chain), ODD-0018 (REPL Protocol), and ODD-0030 (REPL Implementation), I've identified significant **architecture gaps** that need to be addressed before implementation can proceed safely.

### Critical Finding

**ODD-0030 does NOT describe the actual REPL architecture.** It focuses on:
- Individual component implementations (CachedCompiler, VariableStore, Subprocess)
- Performance targets and error handling
- File system organization
- Implementation patterns from evcxr

**What's MISSING:**
- How the protocol (ODD-0018) integrates with the compiler (ODD-0030)
- Who owns compilation? (Client? Server? Both?)
- Where do Core Forms come from?
- How does the client-server split work?
- Complete data flow from user input to result

## Architecture Gaps Identified

### Gap 1: Client-Server Architecture Undefined

**What ODD-0018 Says:**
- There IS a client and server (ReplClient, ReplServer)
- They communicate via TCP with binary protocol
- Messages: Request/Response with Operations (Eval, LoadFile, etc.)
- Sessions are managed by SessionManager on the server

**What ODD-0030 Says:**
- CachedCompiler handles compilation and execution
- Subprocess runs user code
- But NO mention of client/server split
- Implies CachedCompiler is THE REPL

**The Gap:**
Where does CachedCompiler live? Options:
1. **In the server** - Server compiles code, executes in subprocess
2. **In the client** - Client compiles locally, server just manages sessions
3. **Hybrid** - Some compilation in client, some in server

**Current assumption in status analysis:** CachedCompiler is in oxur-repl, but WHERE in oxur-repl (client or server)?

### Gap 2: Compilation Pipeline Ownership

**From ODD-0013 (Compilation Chain):**
```
Oxur Source → Parse → Surface Forms → Expand → Core Forms → Lower → Rust AST → Generate → Rust Source → Compile → Binary
```

**Questions:**
1. Who performs Parse/Expand/Lower? (oxur-lang crate)
2. Who performs Generate/Compile? (CachedCompiler in oxur-repl)
3. Where does this happen? (Client? Server?)
4. How do Core Forms get from oxur-lang to CachedCompiler?

**From ODD-0018 (eval context):**
```rust
pub async fn eval(&mut self, code: &str) -> Result<oxur_lang::Value, EvalError> {
    // 1. Parse code based on current mode:
    //    - ReplMode::Lisp  → Oxur syntax parser
    //    - ReplMode::Sexpr → Core Forms parser
    // 2. Apply tiered execution...
}
```

This suggests parsing happens in EvalContext, which is server-side. But who owns the parsers?

### Gap 3: Component Location Ambiguity

**Undefined Boundaries:**

| Component | Crate? | Client or Server? | Status |
|-----------|--------|-------------------|--------|
| CachedCompiler | oxur-repl | ❓ | Assumed server? |
| CodeGenerator | oxur-repl | ❓ | Assumed server? |
| VariableStore | oxur-repl | Server | Clear (in subprocess) |
| Subprocess | oxur-subprocess | Server | Clear |
| SessionManager | oxur-repl | Server | Clear (in ODD-0018) |
| MessageHandler | oxur-repl | Server | Clear (in ODD-0018) |
| EvalContext | oxur-repl | Server | Clear (in ODD-0018) |
| Parser | oxur-lang | ❓ | Who invokes? |
| Expander | oxur-lang | ❓ | Who invokes? |
| Lowerer | oxur-comp | ❓ | Who invokes? |

### Gap 4: Data Flow Not Documented

**What we know from ODD-0018:**
```
Client sends: Request { op: Eval, session: "abc", params: { "code": "(+ 1 2)" } }
               ↓
Server responds: Response { value: Some(3), out: "", err: "", status: [Done] }
```

**What's missing:**
```
Client sends Eval request
  ↓
Server receives in MessageHandler
  ↓
??? How does code become Core Forms ???
  ↓
??? Who invokes CachedCompiler ???
  ↓
??? Where does compilation happen ???
  ↓
Subprocess executes
  ↓
Server sends Response
```

### Gap 5: REPL Section in ODD-0013 is Incomplete

**From ODD-0013, Section 12 (REPL Architecture):**
- Mentions three-tier execution (Tier 1: Calculator, Tier 2: Cached, Tier 3: JIT)
- Shows integration with compilation pipeline
- But doesn't describe the **actual REPL architecture**
- Doesn't mention client/server split
- Doesn't mention protocol layer

**It describes WHAT the REPL does, not HOW it's architectured.**

## Proposed Architecture (Needs Validation)

Based on synthesis of all three docs, here's my **best guess** at the intended architecture:

### Architecture Hypothesis 1: Server-Side Compilation

```
┌─────────────────────────────────────────────────────────────┐
│                         CLIENT                               │
│  ┌──────────────┐                                           │
│  │  ReplClient  │ - Sends requests                          │
│  │              │ - Receives responses                      │
│  │              │ - No compilation logic                    │
│  └──────────────┘                                           │
└──────────────────────────────┬──────────────────────────────┘
                               │ TCP + Postcard Protocol
                               ↓
┌─────────────────────────────────────────────────────────────┐
│                         SERVER                               │
│  ┌──────────────┐                                           │
│  │ ReplServer   │ - Accepts connections                     │
│  │              │ - Routes to MessageHandler                │
│  └──────────────┘                                           │
│         ↓                                                    │
│  ┌──────────────┐                                           │
│  │MessageHandler│ - Dispatches operations                   │
│  │              │ - Manages sessions via SessionManager     │
│  └──────────────┘                                           │
│         ↓                                                    │
│  ┌──────────────┐                                           │
│  │SessionManager│ - One EvalContext per session             │
│  │              │ - Session isolation                       │
│  └──────────────┘                                           │
│         ↓                                                    │
│  ┌──────────────┐                                           │
│  │ EvalContext  │ - Receives code string                    │
│  │              │ - Invokes oxur-lang parser                │
│  │              │ - Gets Core Forms                         │
│  │              │ - Passes to CachedCompiler                │
│  └──────────────┘                                           │
│         ↓                                                    │
│  ┌─────────────────┐                                        │
│  │CachedCompiler   │ - Receives Core Forms                  │
│  │                 │ - Invokes CodeGenerator                │
│  │                 │ - Compiles via cargo                   │
│  │                 │ - Loads into Subprocess                │
│  │                 │ - Returns result                       │
│  └─────────────────┘                                        │
│         ↓                                                    │
│  ┌─────────────────┐                                        │
│  │  Subprocess     │ - Separate binary (oxur-subprocess)    │
│  │                 │ - Loads compiled libraries             │
│  │                 │ - Executes via libloading              │
│  │                 │ - Manages VariableStore                │
│  └─────────────────┘                                        │
└─────────────────────────────────────────────────────────────┘

External Dependencies:
┌──────────────┐
│  oxur-lang   │ - Parser (Oxur syntax → Surface Forms)
│              │ - Expander (Surface → Core Forms)
└──────────────┘

┌──────────────┐
│  oxur-comp   │ - Lowerer (Core Forms → Rust AST)
└──────────────┘

┌──────────────┐
│  oxur-ast    │ - Rust AST printer (AST → Rust source)
└──────────────┘
```

### Data Flow (Detailed)

```
1. User types: (+ 1 2)
   ↓
2. Client sends:
   Request {
     op: Eval,
     session: "session-abc",
     mode: Lisp,
     params: { "code": "(+ 1 2)" }
   }
   ↓ [TCP + Postcard]
3. Server (MessageHandler) routes to SessionManager
   ↓
4. SessionManager finds/creates EvalContext for session-abc
   ↓
5. EvalContext.eval("(+ 1 2)")
   ↓
6. [IN EVALCONTEXT] Parse based on mode:
   - ReplMode::Lisp: oxur_lang::parse() → Surface Forms
                     oxur_lang::expand() → Core Forms
   - ReplMode::Sexpr: oxur_lang::parse_core_forms() → Core Forms
   ↓
7. EvalContext passes Core Forms to CachedCompiler
   ↓
8. [IN CACHEDCOMPILER]
   a. CodeGenerator.generate(core_forms) → Rust source
   b. Write to session temp dir
   c. Invoke cargo build
   d. Parse cargo JSON output
   e. Check for errors → translate via source maps
   f. Get compiled library path
   ↓
9. [IN CACHEDCOMPILER] Execute:
   a. Send "LOAD lib.so fn_name" to Subprocess
   b. Subprocess loads library via libloading
   c. Subprocess calls function
   d. Function mutates VariableStore
   e. Subprocess captures stdout/stderr
   f. Subprocess returns completion marker
   ↓
10. CachedCompiler receives execution result
    ↓
11. EvalContext returns result to MessageHandler
    ↓
12. MessageHandler creates Response:
    Response {
      id: request.id,
      session: "session-abc",
      value: Some(3),
      out: "",
      err: "",
      status: [Done]
    }
    ↓ [TCP + Postcard]
13. Client receives and displays: 3
```

## Critical Questions for Architecture Document

### Q1: Client-Server Compilation Split?

**Option A: All compilation on server**
- Pro: Simpler client, server has all state
- Pro: Matches sessionful architecture
- Con: Server resource intensive
- Con: Network latency for every eval

**Option B: Compilation on client, execution on server**
- Pro: Distributes load
- Con: Complex state management
- Con: How to share compiled artifacts?
- Con: Doesn't match evcxr pattern (subprocess is local)

**Option C: Full REPL on client, server just coordinates**
- Pro: Lowest latency
- Con: Why have server at all?
- Con: Doesn't match protocol design

**Recommended: Option A** (matches ODD-0018 session architecture and evcxr pattern)

### Q2: Where does oxur-lang integration happen?

**From ODD-0018 (EvalContext.eval):**
```rust
// Integration point with oxur/lang compilation chain:
//
// 1. Parse code based on current mode:
//    - ReplMode::Lisp  → Oxur syntax parser
//    - ReplMode::Sexpr → Core Forms parser
```

**This implies:**
- EvalContext calls into oxur-lang
- Server-side parsing
- oxur-repl depends on oxur-lang

**Questions:**
1. Does oxur-lang provide a `parse()` function?
2. Does oxur-lang provide an `expand()` function?
3. What's the API contract?

### Q3: CachedCompiler location?

**Evidence it's server-side:**
- Manages per-session state (SessionState)
- Manages session temp directories (SessionDir)
- Spawns subprocess (ChildProcess)
- All of these are session-specific
- SessionManager manages sessions
- Therefore CachedCompiler is per-session, server-side

**But where in the server?**

**Option A: Inside EvalContext**
```rust
pub struct EvalContext {
    session_id: SessionId,
    mode: ReplMode,
    compiler: CachedCompiler,  // ← here
    history: Vec<HistoryEntry>,
    output_buffer: OutputBuffer,
}
```

**Option B: Separate, referenced by EvalContext**
```rust
pub struct EvalContext {
    session_id: SessionId,
    mode: ReplMode,
    compiler: Arc<Mutex<CachedCompiler>>,  // ← shared
    ...
}
```

**Option C: MessageHandler manages both**
```rust
pub struct MessageHandler {
    sessions: Arc<SessionManager>,
    compilers: HashMap<SessionId, CachedCompiler>,  // ← parallel
}
```

**Recommended: Option A** (simplest, one session object)

### Q4: Three-tier execution - where does it happen?

**From ODD-0013:**
- Tier 1 (Calculator): <1ms for simple arithmetic
- Tier 2 (Cached): Previously compiled code
- Tier 3 (JIT): Complex expressions (compile & cache)

**From ODD-0018 (EvalContext.eval comment):**
```rust
// 2. Apply tiered execution (from compilation chain doc):
//    Tier 1 (Interpreter): Simple expressions (<10 nodes)
//    Tier 2 (Cached):      Previously compiled code
//    Tier 3 (JIT):         Complex expressions (compile & cache)
```

**This logic must be in:**
1. EvalContext.eval() - decides which tier
2. Or CachedCompiler - decides which tier

**Recommended:** EvalContext decides tier, delegates to appropriate handler

## Key Missing Integration Points

### Integration Point 1: oxur-lang → oxur-repl

**What we need:**
```rust
// In oxur-lang crate:
pub fn parse_lisp(source: &str) -> Result<SurfaceForms>;
pub fn expand(surface: SurfaceForms) -> Result<CoreForms>;
pub fn parse_core_forms(source: &str) -> Result<CoreForms>;

// In oxur-repl (EvalContext):
use oxur_lang::{parse_lisp, expand, CoreForms};

let core_forms = match self.mode {
    ReplMode::Lisp => {
        let surface = oxur_lang::parse_lisp(code)?;
        oxur_lang::expand(surface)?
    }
    ReplMode::Sexpr => {
        oxur_lang::parse_core_forms(code)?
    }
};
```

**Status:** Unclear if this API exists in oxur-lang

### Integration Point 2: Core Forms → Rust Source

**What we need:**
```rust
// In oxur-comp crate:
pub fn lower(core: &CoreForms) -> Result<RustAst>;

// In oxur-ast crate:
pub fn print_rust(ast: &RustAst) -> String;

// In oxur-repl (CodeGenerator):
use oxur_comp::lower;
use oxur_ast::print_rust;

pub fn generate(&self, core: &CoreForms, state: &SessionState) -> Result<String> {
    let rust_ast = oxur_comp::lower(core)?;
    let wrapper_ast = self.wrap_in_function(rust_ast, state);
    let source = oxur_ast::print_rust(&wrapper_ast);
    self.add_source_map_comments(source)
}
```

**Status:** Unclear if lowering exists in oxur-comp

### Integration Point 3: Session ↔ CachedCompiler

**Current unknown:**
- Does SessionManager create CachedCompiler instances?
- Does EvalContext own CachedCompiler?
- Lifecycle management?

**Need to define:**
```rust
// In SessionManager:
pub fn create_session(&self, id: SessionId) -> Result<Session> {
    let eval_context = EvalContext::new(id);
    // Does this create CachedCompiler internally?
    // Or does SessionManager inject it?
}
```

## Recommendations for Architecture Document

The new **REPL Architecture Overview** document should include:

### Section 1: High-Level Architecture Diagram
- Show client, server, subprocess components
- Show protocol layer (TCP + Postcard)
- Show crate boundaries (oxur-repl, oxur-lang, oxur-comp, oxur-ast, oxur-subprocess)

### Section 2: Component Inventory
For each major component, document:
- **Name**: CachedCompiler, EvalContext, etc.
- **Location**: Which crate? Which module?
- **Ownership**: Client-side or server-side?
- **Lifecycle**: When created? When destroyed?
- **Dependencies**: What does it call?
- **Interface**: Key public methods

### Section 3: Compilation Pipeline
- Stage-by-stage data flow
- Who invokes each stage
- Where each stage executes (client/server)
- Error handling at each stage

### Section 4: Integration Points
- oxur-lang → oxur-repl (parsing/expansion)
- oxur-comp → oxur-repl (lowering)
- oxur-ast → oxur-repl (code generation)
- Define required APIs with signatures

### Section 5: Session Architecture
- SessionManager responsibilities
- Session lifecycle
- Per-session resources (EvalContext, CachedCompiler, Subprocess, SessionDir)
- Session isolation guarantees

### Section 6: Protocol Integration
- How Request flows through server
- How Response is constructed
- Operation handling (Eval, LoadFile, etc.)
- Error propagation through protocol

### Section 7: Three-Tier Execution
- Decision logic (which tier for which input?)
- Tier 1: Calculator implementation
- Tier 2: Cached compilation
- Tier 3: JIT compilation
- Performance characteristics

### Section 8: Critical Paths
- Simple eval: `(+ 1 2)` - step by step
- Complex eval: `(defn foo ...)` - step by step
- Error case: syntax error - how it propagates
- Session management: create, eval, close

## Next Steps for This Session

1. ✅ **Document review complete** (this document)
2. ⏭️ **Create REPL Architecture Overview** with sections above
3. ⏭️ **Update ODD-0030** to reference architecture and fix gaps
4. ⏭️ **Review ODD-0026** for alignment (if time permits)

## Open Questions for Discussion

1. **Is my architecture hypothesis correct?** (server-side compilation)
2. **Where exactly does CachedCompiler live?** (in EvalContext? separate?)
3. **What's the API of oxur-lang?** (parse, expand functions)
4. **What's the API of oxur-comp?** (lower function)
5. **Should we document client-side REPL as future work?** (for offline dev)
6. **How do we handle async compilation?** (all async? or blocking in server?)

---

**Status:** Analysis complete, ready for architecture document creation.
