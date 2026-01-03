# Claude Prompt: Synthesize evcxr Audit Reports for Oxur REPL Design

## Your Mission

You are receiving three comprehensive audit reports of the evcxr project components. Your job is to synthesize these findings into actionable architectural decisions for Oxur's REPL implementation.

**You are NOT just summarizing** - you are making architectural decisions based on evidence.

## Context: What is Oxur?

Oxur is a Lisp dialect that compiles to Rust with 100% interoperability. Key design documents:

### Compilation Chain Architecture (Doc 0013)
```
Oxur Source (.ox files)
    ↓ Stage 1: Parse → Surface Forms (S-expressions with sugar)
    ↓ Stage 2: Expand → Core Forms (canonical S-expressions, the IR)
    ↓ Stage 3: Lower → Rust AST
    ↓ Stage 4: Generate → Rust Source
    ↓ Stage 5: Compile → Binary (rustc)
```

**Key principle**: Core Forms are the stable IR between language and Rust

### REPL Protocol Design (Doc 0018)
- Multi-transport (TCP, Unix sockets, named pipes, in-process)
- Session isolation with explicit session IDs
- Dual-mode: Lisp syntax + S-expression (debug) modes
- Postcard serialization (binary, compact)
- Operations: clone, eval, load-file, interrupt, close, describe, history
- Streaming responses for long operations

### REPL Evaluation Strategy (Doc 0019)

**Two-Tier Execution Model**:

**Tier 1: Calculator Mode** (Interpret)
- Literal arithmetic only: `(+ 1 2)`, `(* 3 4)`
- No variables, side effects, or control flow
- Target: <1ms response
- ~100 lines of code
- Does NOT use evcxr

**Tier 2: Cached Compilation** (Compile)
- Everything else (variables, functions, IO, control flow)
- First time: 50-200ms (full rustc to .so/.dylib)
- Cached: ~0ms (reuse compiled library)
- DOES use evcxr_runtime for value representation

**S-expression Mode**:
- Always compiles (no calculator fast path)
- For debugging compiler, not regular use
- Consistency over speed

## The Three Audit Reports

You will receive:

1. **evcxr_repl Audit** - REPL architecture patterns
   - State management
   - Error recovery
   - Dependency handling
   - Output capture
   - rustc integration
   - UX patterns

2. **evcxr_runtime Audit** - Runtime integration
   - Value representation
   - Display/Debug formatting
   - Execution model
   - Output capture implementation
   - Error handling
   - Integration API

3. **evcxr Compiler Audit** - Compilation mechanics
   - rustc invocation patterns
   - Temporary file management
   - Dynamic library compilation
   - Code generation templates
   - Incremental compilation
   - Error parsing

## Your Deliverables

Produce a comprehensive markdown document with these sections:

### 1. Executive Summary (1-2 pages)

**Key Findings**:
- 5-7 most important discoveries across all audits
- Surprises or unexpected insights
- Patterns that change our approach

**Strategic Recommendations**:
- High-level architectural decisions
- What to adopt from evcxr
- What to build ourselves
- What to skip entirely

**Risk Assessment**:
- Technical risks in our approach
- Integration challenges
- Performance concerns
- Maintenance considerations

### 2. Architecture Decision Records (ADRs)

For each major decision, create an ADR:

#### ADR Template:

**ADR-N: [Decision Title]**

**Status**: Decided | Proposed | Superseded

**Context**:
What's the situation and problem? (2-3 paragraphs)

**Decision**:
What are we doing? (1 paragraph, crystal clear)

**Options Considered**:
1. **Option A**: [Description]
   - Pros: ...
   - Cons: ...
   - Evidence from audits: ...

2. **Option B**: [Description]
   - Pros: ...
   - Cons: ...
   - Evidence from audits: ...

3. **Recommended: Option C**: [Description]
   - Pros: ...
   - Cons: ...
   - Evidence from audits: ...

**Consequences**:
- What follows from this decision?
- What does it enable?
- What does it prevent?
- What's the implementation complexity?

**Related Decisions**:
- Links to other ADRs this affects

---

**Create ADRs for at minimum**:

1. **Value Representation**: Use evcxr_runtime as-is, wrap it, or build our own?
2. **Output Capture**: Adopt evcxr's approach, adapt it, or use different mechanism?
3. **Compilation Strategy**: rustc flags, incremental compilation, caching approach
4. **Temporary File Management**: Directory structure, cleanup strategy, session isolation
5. **Error Translation**: How to map rustc errors to Oxur source via source maps
6. **Code Generation**: How to wrap Oxur Core Forms for compilation
7. **Dependency Management**: Handle external crates (future, but architect for it)
8. **Session State**: How to maintain state across evaluations

### 3. Integration Architecture

**Component Diagram**:
```
┌─────────────────────────────────────────┐
│         Oxur REPL Server                │
│  (from protocol design doc 0018)        │
└─────────────────┬───────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌────────┐  ┌──────────┐  ┌─────────────┐
│ Tier 1 │  │  Tier 2  │  │  Session    │
│  Calc  │  │ Compiler │  │  Manager    │
└────────┘  └────┬─────┘  └─────────────┘
                 │
      ┌──────────┼──────────┐
      ▼          ▼          ▼
  ┌──────┐  ┌────────┐  ┌──────────┐
  │rustc │  │ evcxr_ │  │ Artifact │
  │      │  │runtime │  │  Cache   │
  └──────┘  └────────┘  └──────────┘
```

**Create this diagram showing**:
- How components fit together
- Data flow between components
- Which parts use evcxr patterns
- Which parts are Oxur-specific

**Integration Points**:
For each interface between components:
- What data is exchanged?
- What protocols are used?
- What are the performance characteristics?
- What error handling is needed?

### 4. Implementation Specification

**CachedCompiler Implementation**:
```rust
// Provide ACTUAL, compilable code based on audit findings

use evcxr_runtime::{Runtime, EvalResult};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct CachedCompiler {
    runtime: Runtime,
    cache: HashMap<CodeHash, CompiledCode>,
    incremental_dir: PathBuf,
    temp_dir: TempDir,
}

impl CachedCompiler {
    pub fn new(session_id: &str) -> Result<Self> {
        // Based on audit findings, exact initialization
        todo!()
    }
    
    pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
        // Based on audit findings, exact evaluation flow
        // Include:
        // - Cache lookup
        // - Compilation if needed
        // - Output capture
        // - Error handling
        // - Response construction
        todo!()
    }
    
    async fn compile_to_dylib(&self, rust_code: &str) -> Result<PathBuf> {
        // Based on compiler audit, exact rustc invocation
        // Include:
        // - Temp file creation
        // - rustc command construction
        // - Error parsing
        // - Artifact location
        todo!()
    }
    
    fn execute(&mut self, compiled: &CompiledCode) -> Result<EvalResult> {
        // Based on runtime audit, exact execution
        // Include:
        // - Output capture installation
        // - Library loading
        // - Function call
        // - Result extraction
        todo!()
    }
}
```

**OutputCapture Implementation**:
```rust
// Based on runtime audit findings

pub struct OutputCapture {
    // Exact fields based on what evcxr uses
}

impl OutputCapture {
    pub fn install() -> Result<Self> {
        // Exact implementation from audit
        todo!()
    }
    
    pub fn take(&self) -> (String, String) {
        // Exact implementation from audit
        todo!()
    }
}
```

**Provide complete, ready-to-implement code** for:
- CachedCompiler
- OutputCapture  
- ErrorTranslator (rustc → Oxur source)
- CodeGenerator (Core Forms → Rust wrapper)
- SessionState

### 5. rustc Invocation Reference Card

**The Exact Command**:
```bash
# Copy-paste ready rustc invocation for Oxur REPL
rustc \
  --crate-type dylib \
  --edition 2021 \
  -C opt-level=2 \
  -C incremental=/path/to/session/incremental \
  --out-dir /path/to/session/target \
  # ... every flag with explanation of why we need it
  input.rs
```

**Platform Variations**:
- Linux-specific
- macOS-specific  
- Windows-specific

**Environment Setup**:
```bash
export RUSTC_WRAPPER=...
export CARGO_HOME=...
# All necessary env vars
```

### 6. File Organization Specification

**Exact Directory Structure**:
```
/tmp/oxur-repl/
├── sessions/
│   └── session-{id}/
│       ├── incremental/     # rustc incremental cache
│       ├── source/          # Generated .rs files
│       │   ├── eval_001.rs
│       │   └── eval_002.rs
│       ├── target/          # Compiled .so/.dylib files
│       │   ├── libeval_001.so
│       │   └── libeval_002.so
│       └── metadata.json    # Session metadata
└── global-cache/            # Shared dependencies (future)
```

**Lifecycle**:
- When created
- When cleaned up
- How to handle crashes
- Disk space management

### 7. Performance Expectations

**Benchmarks** (based on audit findings + estimates):

| Operation | Target | Notes |
|-----------|--------|-------|
| Tier 1 eval (calc) | <1ms | Pure Rust arithmetic |
| Tier 2 first compile | 50-200ms | Cold, no cache |
| Tier 2 cached | <10ms | Library already loaded |
| Tier 2 incremental | 40-80ms | Warm incremental cache |
| Session startup | <100ms | Initialize runtime |
| Session cleanup | <50ms | Remove temp files |

**Optimization Strategy**:
- What makes compilation fast?
- What's the cache hit rate we expect?
- When to show progress indicators?

### 8. Error Handling Strategy

**Error Translation Pipeline**:
```
rustc error (generated.rs:42)
    ↓ Parse error message
rustc error structure (file, line, col, message)
    ↓ Map file:line to Node ID
Rust AST Node (from Stage 4)
    ↓ Source map lookup
Core Form Node (from Stage 3)
    ↓ Source map lookup
Surface Form Node (from Stage 2)
    ↓ Source map lookup
Original Oxur source (test.ox:5:10)
```

**Implementation**:
```rust
pub struct ErrorTranslator {
    source_map: Arc<SourceMap>,
    generated_files: HashMap<PathBuf, GeneratedSource>,
}

impl ErrorTranslator {
    pub fn translate_rustc_error(&self, error: &RustcError) -> OxurError {
        // Exact implementation based on audit + source map design
        todo!()
    }
}
```

### 9. Testing Strategy

**Unit Tests**:
```rust
#[test]
fn test_calculator_mode() {
    // Test Tier 1 evaluation
}

#[test]
fn test_compile_simple_expr() {
    // Test Tier 2 compilation
}

#[test]
fn test_output_capture() {
    // Test stdout/stderr capture
}

#[test]
fn test_error_translation() {
    // Test rustc error → Oxur source
}
```

**Integration Tests**:
```rust
#[tokio::test]
async fn test_full_repl_session() {
    // End-to-end REPL workflow
    let mut client = connect_repl().await;
    
    // Create session
    let session = client.clone_session().await?;
    
    // Calculator mode (fast)
    let r1 = client.eval("(+ 1 2)").await?;
    assert_eq!(r1.value, Some(json!(3)));
    
    // Compilation (first time slow)
    let r2 = client.eval("(defn double [x] (* x 2))").await?;
    
    // Cached (fast)
    let r3 = client.eval("(double 21)").await?;
    assert_eq!(r3.value, Some(json!(42)));
}
```

**Benchmarks**:
```rust
#[bench]
fn bench_calculator_eval(b: &mut Bencher) {
    // Measure Tier 1 performance
}

#[bench]
fn bench_first_compile(b: &mut Bencher) {
    // Measure cold compilation
}

#[bench]
fn bench_cached_compile(b: &mut Bencher) {
    // Measure cache hit performance
}
```

### 10. Implementation Roadmap

**Phase 0: Foundation** (Week 1)
- [ ] Set up project structure
- [ ] Add evcxr_runtime dependency
- [ ] Create basic types (CachedCompiler, etc.)
- [ ] Write failing tests

**Phase 1: Minimal Compilation** (Week 2)
- [ ] Implement rustc invocation
- [ ] Compile simple expressions
- [ ] Load and execute dynamic libraries
- [ ] Verify basic functionality

**Phase 2: Output Capture** (Week 3)
- [ ] Implement OutputCapture
- [ ] Integrate with evcxr_runtime
- [ ] Test stdout/stderr capture
- [ ] Handle edge cases

**Phase 3: Caching** (Week 4)
- [ ] Implement compilation cache
- [ ] Add incremental compilation
- [ ] Measure performance improvements
- [ ] Tune cache eviction

**Phase 4: Error Handling** (Week 5)
- [ ] Parse rustc errors
- [ ] Integrate source maps
- [ ] Translate to Oxur source
- [ ] Test error reporting

**Phase 5: Calculator Mode** (Week 6)
- [ ] Implement Tier 1 interpreter
- [ ] Add fast path check
- [ ] Benchmark improvements
- [ ] Integration tests

**Phase 6: Protocol Integration** (Week 7)
- [ ] Connect to REPL server
- [ ] Implement session management
- [ ] Add dual-mode support
- [ ] End-to-end testing

**Phase 7: Polish** (Week 8)
- [ ] Performance tuning
- [ ] Documentation
- [ ] User testing
- [ ] v1.0 release preparation

### 11. Dependencies and Versioning

**Direct Dependencies**:
```toml
[dependencies]
evcxr_runtime = "X.Y.Z"  # Based on audit recommendation
libloading = "0.8"
tempfile = "3.8"
# ... all dependencies with versions and justifications
```

**Feature Flags**:
```toml
[features]
default = []
incremental-compilation = []
# ... based on what we actually need
```

### 12. Risk Mitigation

For each risk identified in audits:

**Risk**: [Specific risk from audit]
**Likelihood**: High | Medium | Low
**Impact**: High | Medium | Low
**Mitigation Strategy**: [Concrete steps]
**Fallback Plan**: [What if mitigation fails]

### 13. Open Questions

**Technical Questions**:
- [ ] Question 1 from audits
- [ ] Question 2 from audits
- **Decision needed by**: [Date]
- **Owner**: [Who will research]

**Architecture Questions**:
- [ ] Question 1
- [ ] Question 2

**Performance Questions**:
- [ ] Question 1
- [ ] Question 2

### 14. Appendix: Audit Summary Tables

**Pattern Adoption Matrix**:

| Pattern | Source | Priority | Status | Notes |
|---------|--------|----------|--------|-------|
| Incremental compilation | evcxr | P0 | ✅ Adopt | Critical for perf |
| Output capture | evcxr_runtime | P0 | ✅ Adopt | Essential |
| ... | ... | ... | ... | ... |

**API Integration Checklist**:

| API | Purpose | Usage | Priority | Status |
|-----|---------|-------|----------|--------|
| Runtime::new() | Initialize | CachedCompiler::new() | P0 | ✅ |
| ... | ... | ... | ... | ... |

**Performance Baseline**:

| Metric | evcxr Measured | Oxur Target | Gap Analysis |
|--------|----------------|-------------|--------------|
| Cold compile | 200ms | 200ms | ✅ Match |
| ... | ... | ... | ... |

## Analysis Guidelines

### Do:
- ✅ Make clear, actionable decisions backed by audit evidence
- ✅ Provide complete, compilable code examples
- ✅ Create ADRs for all major architecture decisions
- ✅ Specify exact commands, flags, and configurations
- ✅ Think about integration with Oxur's existing design (docs 0013, 0018, 0019)
- ✅ Consider maintenance burden and long-term evolution
- ✅ Identify gaps where audits don't provide answers
- ✅ Be specific about what to adopt vs. adapt vs. skip

### Don't:
- ❌ Just summarize audit findings without synthesis
- ❌ Ignore conflicts between audit recommendations
- ❌ Provide vague guidance like "consider using X"
- ❌ Copy patterns blindly without adapting for Oxur
- ❌ Skip difficult decisions or leave them "TBD"
- ❌ Ignore performance implications
- ❌ Forget about source map integration
- ❌ Lose sight of the dual-mode REPL architecture

## Success Criteria

This synthesis is successful if:
1. ✅ We can start implementing Oxur REPL immediately with confidence
2. ✅ All major architectural decisions have clear ADRs
3. ✅ We have exact rustc commands and file organization
4. ✅ We have compilable code examples for key components
5. ✅ We know exactly what to use from evcxr and what to build
6. ✅ We understand all risks and have mitigation strategies
7. ✅ The roadmap is realistic and achievable
8. ✅ Nothing critical is left as "TBD" or "needs research"

## Output Format

Produce a single comprehensive markdown document titled:

**"Oxur REPL Implementation Specification - Synthesized from evcxr Audits"**

Save as: `oxur-repl-implementation-spec.md`

This document should be ~30-50 pages and serve as the **definitive reference** for REPL implementation.

## Final Note

You're not just reporting on audits - you're **making architectural decisions** that will shape Oxur's REPL for years. Be thoughtful, be specific, be decisive.

The audit reports are data. Your job is to turn that data into wisdom.

Good luck! 🦀
