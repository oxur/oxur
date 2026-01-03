# Claude Code Prompt: evcxr_repl Audit

## Your Mission

You are auditing the `evcxr_repl` crate to identify patterns, techniques, and architectural decisions that could benefit the Oxur REPL implementation. The repository has been cloned locally and you should analyze the code in depth.

## Context: What is Oxur?

Oxur is a Lisp dialect that compiles to Rust with 100% interoperability. It has a unique architecture:

### Compilation Pipeline

1. **Parse** - Oxur syntax → Surface Forms (S-expressions with sugar/macros)
2. **Expand** - Surface Forms → Core Forms (canonical S-expressions, the IR)
3. **Lower** - Core Forms → Rust AST
4. **Generate** - Rust AST → Rust source code
5. **Compile** - Rust source → Binary (via rustc)

### REPL Evaluation Strategy

Oxur uses a **two-tier execution model**:

**Tier 1: Calculator Mode (Interpret)**

- Only interpret literal arithmetic: `(+ 1 2)`, `(* 3 4)`
- No variables, no side effects, no control flow
- Target: <1ms response time
- ~100 lines of code

**Tier 2: Cached Compilation (Compile Everything Else)**

- Variables, functions, IO, control flow - all compile through Rust
- First time: 50-200ms (full rustc compilation)
- Cached: ~0ms (reuse compiled dynamic library)
- Use `evcxr_runtime` for value representation and execution

### Dual-Mode REPL

1. **Lisp Syntax Mode** (default) - User-facing Oxur syntax with macros
2. **S-expression Mode** (debug) - Core Forms directly, always compiles (no calculator fast path)

### Key Design Principles

- **Minimal interpretation** - Only trivial calculator math
- **Compilation for everything else** - Leverage Rust's safety guarantees
- **No semantic divergence** - REPL must match compiled code behavior exactly
- **Source map integration** - Errors trace back to original Oxur source
- **Session isolation** - Multiple concurrent sessions with independent state

## Context: Oxur REPL Protocol Design

Oxur has a sophisticated network protocol (unlike evcxr's local-only REPL):

### Protocol Features

- **Multi-transport**: TCP, Unix sockets, named pipes, in-process
- **Session management**: Explicit session IDs, independent evaluation contexts
- **Dual-mode**: Switch between Lisp syntax and S-expression modes
- **Streaming**: Support for partial responses during long evaluations
- **Operations**: `clone` (create session), `eval`, `load-file`, `interrupt`, `close`, `describe`, `history`

### Message Format

- **Serialization**: Postcard (binary, compact) for v1.0, MessagePack future
- **Framing**: Length-delimited with 4-byte big-endian prefix
- **Correlation**: Each request has unique ID echoed in response
- **Output capture**: Separate stdout/stderr from evaluation results
- **Error taxonomy**: Protocol errors vs. evaluation errors

### Architecture Layers

1. **Protocol** - Message types, no I/O dependencies
2. **Transport** - Unified interface across connection types
3. **Evaluation** - Integration with compilation chain
4. **Server** - Connection handling, session management
5. **Client** - Reference implementation

## Your Specific Focus Areas

When auditing `evcxr_repl`, pay special attention to:

### 1. State Management Between Evaluations

- How does evcxr maintain state across REPL interactions?
- How are variables/functions stored and accessed?
- What's the lifecycle of compiled code?
- How do they handle cleanup/garbage collection?

### 2. Error Recovery Mechanisms

- How does evcxr handle compilation errors without crashing?
- How are partial/incomplete inputs handled?
- What happens when rustc fails?
- How are panics in user code isolated?

### 3. Dependency Management

- How does the `:dep` command work?
- How are external crates added dynamically?
- How is `Cargo.toml` managed/updated?
- What's the compilation strategy when deps change?

### 4. Output Capture

- How is stdout/stderr captured during evaluation?
- How are print statements from compiled code intercepted?
- How is output threaded back to the REPL?
- Any special handling for async/multi-threaded output?

### 5. rustc Integration

- What flags/options are used when invoking rustc?
- How is incremental compilation leveraged?
- How are compilation artifacts managed?
- What optimizations are applied?

### 6. User Experience Patterns

- How does evcxr provide feedback during slow operations?
- How are multi-line inputs handled?
- What conveniences exist for interactive development?
- Any syntax sugar or REPL-specific commands?

## Deliverables

Please produce a markdown report at ./workbench/evcxr-repl-audit-report.md with the following sections:

### 1. Executive Summary

- High-level overview of evcxr_repl architecture
- 3-5 key takeaways for Oxur
- Biggest surprises or insights

### 2. Pattern Catalog

For each significant pattern/technique found, provide:

**Pattern Name**: Clear, descriptive name

**Description**: What it does and why (2-3 paragraphs)

**Code Example**: Concrete code snippet showing the pattern (10-30 lines)

**Relevance to Oxur**:

- High: Critical for Oxur's needs
- Medium: Useful but not essential
- Low: Interesting but not applicable

**Complexity**:

- Simple: Straightforward to implement
- Moderate: Some complexity, manageable
- Complex: Significant implementation effort

**Priority**:

- P0: Must have for v1.0
- P1: Should have for v1.0
- P2: Nice to have for v1.0
- P3: Consider for v2.0+

**Integration Notes**: How we'd adapt this for Oxur (1-2 paragraphs)

**Risks/Considerations**: Potential issues or limitations

---

### Example Pattern Entry

**Pattern Name**: Stdout/Stderr Capture with Thread-Local Storage

**Description**:
evcxr captures output by replacing the global stdout/stderr handles with custom writers that store output in thread-local buffers. This allows code executed in the REPL to print normally while the REPL intercepts and displays the output.

The implementation uses `std::io::set_output_capture()` (nightly feature) or manual handle replacement. Output is buffered per evaluation and returned alongside the result value.

**Code Example**:

```rust
// From evcxr_repl/src/output.rs (hypothetical)
use std::sync::{Arc, Mutex};
use std::io::Write;

pub struct OutputCapture {
    stdout: Arc<Mutex<Vec<u8>>>,
    stderr: Arc<Mutex<Vec<u8>>>,
}

impl OutputCapture {
    pub fn install() -> Self {
        let stdout = Arc::new(Mutex::new(Vec::new()));
        let stderr = Arc::new(Mutex::new(Vec::new()));

        // Replace global handles
        unsafe {
            let stdout_writer = Box::new(BufferedWriter {
                buffer: stdout.clone(),
            });
            std::io::set_output(stdout_writer);
        }

        Self { stdout, stderr }
    }

    pub fn take(&self) -> (String, String) {
        let stdout = String::from_utf8_lossy(
            &self.stdout.lock().unwrap()
        ).into_owned();
        self.stdout.lock().unwrap().clear();

        // ... similar for stderr
        (stdout, String::new())
    }
}
```

**Relevance to Oxur**: High - We need output capture for REPL protocol's `out`/`err` fields

**Complexity**: Moderate - Requires careful handle management and thread safety

**Priority**: P0 - Essential for v1.0, users expect to see print output

**Integration Notes**:
We'd adapt this for our `OutputBuffer` type in the evaluation context. Since we're compiling to dynamic libraries, we need to ensure the capture works across library boundaries. May need to pass output handles into compiled code explicitly rather than relying on global state.

**Risks/Considerations**:

- Thread-local capture might not work if user code spawns threads
- Nested REPL evaluations could interfere with capture
- Performance overhead of locking on every write
- Compatibility with async/await if user code uses tokio

---

### 3. Architecture Comparison

Create a table comparing evcxr_repl's architecture to Oxur's design:

| Aspect | evcxr_repl | Oxur REPL | Assessment |
|--------|-----------|-----------|------------|
| Transport | Local only | Multi-transport (TCP, Unix, pipe) | Oxur more flexible |
| Session Model | Implicit (global state) | Explicit (session IDs) | Oxur better isolation |
| ... | ... | ... | ... |

### 4. Recommendations

#### Must Adopt (P0)

List patterns we should definitely use, with brief justification

#### Should Consider (P1)

List patterns worth adapting, with trade-offs

#### Can Skip (P2-P3)

List patterns not applicable to Oxur, with reasons

#### Novel Solutions Needed

List areas where evcxr's approach won't work for Oxur and we need our own solution

### 5. Risk Assessment

Identify potential issues with adopting evcxr patterns:

- **Technical risks**: Implementation challenges
- **Maintenance risks**: Complexity that could cause problems
- **Compatibility risks**: Things that might not work with Oxur's architecture

### 6. Questions for Further Investigation

List any uncertainties or areas needing deeper analysis before implementation

### 7. Code Hotspots

Identify specific files/functions worth studying in detail:

```
evcxr_repl/src/file.rs:123-145 - Output capture installation
evcxr_repl/src/file.rs:200-250 - rustc invocation pattern
```

## Analysis Guidelines

### Do

- ✅ Provide concrete code examples (not just descriptions)
- ✅ Explain *why* patterns are used, not just *what* they do
- ✅ Consider Oxur's specific needs (two-tier execution, dual modes, network protocol)
- ✅ Note both strengths and weaknesses of each pattern
- ✅ Be specific about file paths and line numbers
- ✅ Think about how patterns interact with Oxur's compilation pipeline

### Don't

- ❌ Just list features without analysis
- ❌ Copy large blocks of code without explanation
- ❌ Ignore differences between evcxr's and Oxur's architectures
- ❌ Recommend patterns that conflict with Oxur's design principles
- ❌ Focus on UI/presentation over core functionality

## Success Criteria

Your audit is successful if:

1. ✅ We can implement Oxur REPL Tier 2 (compilation) using your findings
2. ✅ We know exactly what to borrow from evcxr and what to build ourselves
3. ✅ We understand the risks and trade-offs of each pattern
4. ✅ We have concrete code examples to reference during implementation
5. ✅ We can make informed architectural decisions about the REPL

## Repository Location

The evcxr repository has been cloned to your workspace. Focus your analysis on:

```
workbench/evcxr/
├── evcxr_repl/          # Main focus of this audit
│   ├── src/
│   │   ├── lib.rs       # Core REPL logic
│   │   ├── eval.rs      # Evaluation implementation
│   │   └── ...
│   └── Cargo.toml
├── evcxr/               # Compiler integration (secondary focus)
└── evcxr_runtime/       # Will be audited separately
```

## Final Notes

Remember: Oxur is not just copying evcxr. We're building a network-capable, session-isolated, dual-mode REPL with a sophisticated protocol. Your job is to identify what evcxr does well that we can adapt, and where we need to diverge.

**Focus on actionable insights that directly inform implementation decisions.**

Good luck! 🦀
