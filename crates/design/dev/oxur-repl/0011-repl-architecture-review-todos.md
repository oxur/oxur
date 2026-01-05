# REPL Architecture Review - TODOs and Decisions

**Session 1 - Ongoing**  
**Date:** January 3, 2026

## Decisions Made During Brainstorming

### ✅ Decision 2: Source Map as Separate Crate

**Context:** Source mapping needs to track transformations across the entire compilation pipeline (oxur-lang → oxur-comp → oxur-ast → oxur-repl). Each crate needs to write to it, but only REPL needs to read from it for error translation.

**Decision:** Create new crate: `oxur-smap`

**Rationale:**
- **No circular dependencies** - Foundation crate, all others depend on it
- **Independent use** - Each crate can use it without depending on REPL
- **Clean separation** - Single responsibility (source tracking)
- **Testable** - Each stage's mapping can be tested in isolation
- **First-class feature** - Makes source mapping a core Oxur differentiator
- **Unique in Lisp space** - LFE, other interop Lisps have nothing like this
- **Brandable** - "smap" is concise, memorable, invites explanation

**Alternative Names Considered:**
- `oxur-source-map` - Clear but verbose
- `oxur-sourcemap` - Slightly shorter
- `oxur-trace` - Good but might confuse with stack traces
- `oxur-origin` - Beautiful, philosophical
- `oxur-rosetta` - Brilliant metaphor (translation between languages) but maybe too clever
- `oxur-compass` - Navigation metaphor
- `oxur-beacon` - Guidance metaphor
- **CHOSEN: `oxur-smap`** - Pragmatic, cool, brandable

**Why `oxur-smap`:**
- Short, memorable, technical
- Invites curiosity ("what's smap?")
- Easy to type in imports
- Modern feel (like kubectl, npm)
- Great for marketing: "smap - Source mapping that actually works"

**Core Types to Define:**
```rust
// oxur-smap/src/lib.rs

pub struct NodeId(u32);

pub struct SourcePos {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub length: u32,
}

pub struct SourceMap {
    surface_positions: HashMap<NodeId, SourcePos>,
    surface_to_core: HashMap<NodeId, NodeId>,
    core_to_rust: HashMap<NodeId, NodeId>,
}

impl SourceMap {
    pub fn record_surface_node(&mut self, node: NodeId, pos: SourcePos);
    pub fn record_expansion(&mut self, surface: NodeId, core: NodeId);
    pub fn record_lowering(&mut self, core: NodeId, rust: NodeId);
    pub fn lookup(&self, rust_node: NodeId) -> Option<SourcePos>;
}
```

**Dependency Graph:**
```
oxur-smap (no dependencies)
     ↑
     ├─ oxur-lang  (records parse/expand mappings)
     ├─ oxur-comp  (records lowering mappings)
     ├─ oxur-ast   (uses NodeId for comments)
     └─ oxur-repl  (uses for error translation)
```

**🚨 CRITICAL: This is now a PREREQUISITE for REPL implementation**

This crate must be designed and implemented BEFORE REPL work begins because:
1. oxur-lang needs it for parser/expander
2. oxur-comp needs it for lowering
3. oxur-ast needs NodeId type for comment generation
4. oxur-repl needs it for error translation

**Next Steps:**
- [ ] Create oxur-smap crate with basic structure
- [ ] Define NodeId generation strategy (global counter? UUID?)
- [ ] Implement core SourceMap with HashMap backing
- [ ] Add API documentation and examples
- [ ] Consider advanced features:
  - Span highlighting (start/end positions)
  - Multi-file support (multiple source files)
  - Performance optimization (caching lookups)
  - Serialization (for IDE integration?)

**Impact on Implementation Timeline:**
- Add to Phase 0 (Preparatory Work): Design and implement oxur-smap
- All other phases depend on this being ready
- Estimated: 1-2 weeks for basic implementation + tests

---

### ✅ Decision 1: Rename CodeGenerator → RustAstWrapper

**Context:** The component in `oxur-repl` that wraps Rust AST for REPL execution was confusingly named `CodeGenerator`, suggesting it does lowering when it actually just adds REPL-specific scaffolding.

**Decision:** Rename to `RustAstWrapper`

**Rationale:**
- Crystal clear: wraps *Rust* AST specifically (not Surface Forms, not Core Forms)
- Extensible: leaves room for other wrappers (ErrorWrapper, ValueWrapper, etc.)
- Self-documenting: name tells you exactly what goes in
- Prevents confusion: no ambiguity about which AST

**Location:** 
- File: `oxur-repl/src/wrapper.rs` (renamed from `src/codegen/generator.rs`)
- Component: `RustAstWrapper` (renamed from `CodeGenerator`)

**Responsibilities (unchanged):**
- Takes pure Rust AST from oxur-comp::lower()
- Adds VariableStore integration
- Adds extern "C" wrapper function
- Adds source map comments
- Generates variable load/store code
- Emits complete library AST

**Key Point:** This component does NOT do lowering - it wraps already-lowered Rust AST.

---

## Documents That Need Updates

### Priority 1: Architecture Documents

- [ ] **REPL Architecture Overview**
  - Section 2.3: Component Inventory - Update CodeGenerator → RustAstWrapper
  - Section 3: Compilation Pipeline - Update stage 5 "Wrap" to reference RustAstWrapper
  - Section 4.1.3: CodeGenerator → RustAstWrapper (entire section)
  - All code examples showing CodeGenerator
  - File paths: `codegen/generator.rs` → `wrapper.rs`

- [ ] **ODD-0030 v1.1** (REPL Implementation Specification)
  - Section 4.1.3: Component specification for CodeGenerator → RustAstWrapper
  - All references to CodeGenerator in other sections
  - File path updates

### Priority 2: Related Documents

- [ ] **ODD-0026 v1.1** (Evaluation Strategy)
  - Section 3: Component Placement table - Update CodeGenerator → RustAstWrapper
  - Any code examples or references

### Priority 3: Session Documentation

- [ ] **Session 1 Plan** - Add decision to completed items
- [ ] **Session 1 Summary** (when we wrap up) - Document this decision

---

## Future Questions/Topics to Explore

*(Add new topics here as we discuss)*

### 🔥 In Progress: Subprocess Architecture Deep Dive

**The Big Questions:**
1. Do we even need a subprocess model?
2. If yes, what IPC mechanism should we use?

**Communication Options Evaluated:**
- **stdin/stdout** (evcxr approach) - Simple but fragile (text parsing, stdout mixing)
- **Network/loopback** (TCP/Unix sockets) - Structured, reuses protocol
- **IPC** (shared memory, message queues) - Fastest but overkill
- **In-process** (no subprocess) - Simplest but no crash isolation

**💡 KEY INSIGHT: We Already Have a Protocol!**

ODD-0018 defines Request/Response types with postcard serialization. 
What if the subprocess IS just a mini REPL server?

**✅ TENTATIVE DECISION: Unix Domain Sockets + Protocol Reuse**

**Architecture:**
```
CachedCompiler
  ↓ Spawns subprocess with socket path
Subprocess (mini REPL server)
  ↓ Binds Unix socket
  ↓ Accepts ONE connection
  ↓ Uses same Request/Response types
  ↓ Returns structured responses
CachedCompiler connects and sends requests
```

**Why This is Good:**
- ✅ Reuses existing protocol work (Request/Response types, postcard)
- ✅ Clean separation (user stdout in Response.out, not mixed with protocol)
- ✅ Type-safe binary protocol (not string parsing)
- ✅ Testable with same tools as REPL client
- ✅ Future-proof (could support remote execution)
- ✅ Unix sockets work on all platforms (Linux, macOS, Windows 10+)

**Why This Might Be Too Clever:**
- ⚠️ Subprocess isn't really a "REPL server" - it's an execution sandbox
- ⚠️ Protocol abstraction might add unnecessary overhead
- ⚠️ Are we over-engineering?

**Still Need to Decide:**
- [ ] Subprocess vs. in-process for v1.0?
  - Subprocess: Safety net, crash isolation, restart capability
  - In-process: Simpler, faster, standard REPL model
- [ ] Performance benchmarks: Unix socket vs in-process vs stdin/stdout
- [ ] Research evcxr's decision history (git archaeology)

**Open Questions:**
1. Is "subprocess as mini REPL server" the right abstraction?
2. What's the performance cost of Unix sockets vs in-process?
3. Why did evcxr choose their approach? (git history investigation needed)
4. Should we prototype both and measure?

**Next Steps:**
- [ ] Benchmark Unix socket latency vs in-process function call
- [ ] Research evcxr commit history for subprocess decision rationale
- [ ] Prototype minimal version of each approach
- [ ] Decide based on performance data + engineering simplicity

**Status:** Tentative architecture, needs validation through research + benchmarks

- 

---

## Prerequisites Identified

### oxur-smap Crate (Decision 2)

**Status:** Must be implemented before REPL work begins

This became clear during architecture review: source mapping isn't just a REPL concern, it's a foundational capability needed across the entire compilation pipeline. See Decision 2 above for full details.

**Blocks:**
- oxur-lang (parser/expander need to record mappings)
- oxur-comp (lowering needs to record mappings)
- oxur-ast (needs NodeId for comment generation)
- oxur-repl (needs SourceMap for error translation)

**Recommendation:** Make oxur-smap the first infrastructure crate we build.

---

## Notes

- Keep this file updated as we make more decisions
- Each decision should include: Context, Decision, Rationale, Location, Impact
- Track which documents need updates
- Session can continue or we can hand off to next session with clear TODO list

---

**Status:** Active brainstorming session
**Last Updated:** January 3, 2026

### ✅ Decision 4: Temp Directory Strategy

**Context:** Session directories need filesystem access for rustc/cargo compilation. Where should they live?

**Question:** In-memory (tmpfs/ramfs) or regular filesystem? Cross-platform strategy?

**Decision:** Best-effort tmpfs with graceful fallback

**Implementation:**
```rust
fn get_repl_temp_root() -> PathBuf {
    // Try environment variable first (user override)
    if let Ok(custom) = std::env::var("OXUR_REPL_TEMP_DIR") {
        return PathBuf::from(custom);
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try /dev/shm first (guaranteed RAM-backed on Linux)
        let shm = PathBuf::from("/dev/shm");
        if shm.exists() && shm.is_dir() {
            return shm.join("oxur-repl");
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Check if user mounted a RAM disk at conventional location
        let ramdisk = PathBuf::from("/Volumes/OxurREPL");
        if ramdisk.exists() && ramdisk.is_dir() {
            return ramdisk;
        }
    }
    
    // Fallback: system temp directory (cross-platform)
    // Modern OS will cache hot files in RAM anyway
    std::env::temp_dir().join("oxur-repl")
}

fn session_dir(session_id: &SessionId) -> PathBuf {
    get_repl_temp_root().join(format!("session-{}", session_id))
}
```

**Rationale:**
- **Elegant** - Users don't need to know/care about tmpfs
- **Performance** - Linux gets RAM-backed storage automatically (~2-3% faster)
- **Graceful** - Falls back to regular temp if tmpfs unavailable
- **Overrideable** - OXUR_REPL_TEMP_DIR for power users
- **Cross-platform** - Works everywhere, optimizes where possible

**Benefits by Platform:**

| Platform | Automatic | Manual Override |
|----------|-----------|-----------------|
| Linux | ✅ /dev/shm (RAM) | OXUR_REPL_TEMP_DIR |
| macOS | OS cache | RAM disk at /Volumes/OxurREPL |
| Windows | OS cache | OXUR_REPL_TEMP_DIR (ImDisk) |

**Performance Impact:**
- Linux: ~2-3% faster (free!)
- Others: Negligible (OS caching is good)
- Filesystem I/O is <3% of total time

**Documentation:**
- Main docs: Don't mention (it just works)
- Advanced: Performance tuning section only

**Location:** `oxur-repl/src/session/dir.rs`

**Status:** ✅ Decided

---
