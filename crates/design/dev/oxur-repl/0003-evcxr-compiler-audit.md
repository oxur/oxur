# Claude Code Prompt: evcxr (Compiler) Audit

## Your Mission

You are auditing the `evcxr` crate (the core compiler integration library) to understand how it invokes rustc, manages compilation artifacts, handles incremental compilation, and generates code suitable for REPL evaluation. This audit focuses on the **compilation mechanics** that Oxur will need to implement for Tier 2 (cached compilation).

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
- **Does NOT invoke rustc**

**Tier 2: Cached Compilation (Compile Everything Else)**

- Variables, functions, IO, control flow - all compile through Rust
- First time: 50-200ms (full rustc compilation to dynamic library)
- Cached: ~0ms (reuse previously compiled .so/.dylib)
- **This is where we need evcxr's patterns**

### How Oxur Will Compile REPL Code

```rust
// Conceptual compilation flow
pub struct CachedCompiler {
    cache: HashMap<CodeHash, CompiledCode>,
    temp_dir: TempDir,
}

impl CachedCompiler {
    pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
        let code_hash = hash(&form);

        // Check cache
        if let Some(compiled) = self.cache.get(&code_hash) {
            return self.execute(compiled);  // Instant!
        }

        // Not cached - need to compile
        // 1. Lower Core Forms → Rust AST (we do this ourselves)
        let rust_ast = lower(form)?;

        // 2. Generate Rust source (we do this ourselves)
        let rust_code = generate(&rust_ast)?;

        // 3. Compile to dynamic library (LEARN FROM EVCXR)
        let lib_path = self.compile_to_dylib(&rust_code).await?;

        // 4. Load library (LEARN FROM EVCXR)
        let compiled = CompiledCode::load(lib_path)?;

        // 5. Cache for future use
        self.cache.insert(code_hash, compiled.clone());

        // 6. Execute
        self.execute(&compiled)
    }

    async fn compile_to_dylib(&self, rust_code: &str) -> Result<PathBuf> {
        // THIS IS WHERE WE NEED EVCXR'S EXPERTISE
        // - How to invoke rustc?
        // - What flags to use?
        // - How to manage temp files?
        // - How to handle errors?
        // - How to optimize for REPL use case?
    }
}
```

### What We Need from evcxr

1. **rustc Invocation Patterns**: Exact command-line flags, options, environment
2. **Temporary File Management**: Safe handling of generated .rs files and artifacts
3. **Dynamic Library Compilation**: How to produce .so/.dylib suitable for loading
4. **Incremental Compilation**: Leverage rustc's incremental mode for speed
5. **Error Parsing**: Extract useful errors from rustc output
6. **Dependency Handling**: Manage external crates in REPL context
7. **Code Generation Patterns**: How to wrap REPL code for execution

## Your Specific Focus Areas

When auditing `evcxr`, pay special attention to:

### 1. rustc Invocation Mechanics

**Questions:**

- What's the exact rustc command used?
- What flags are essential vs. optional?
- How are target directories specified?
- What edition is used (2018? 2021?)?
- Are there optimization flags for REPL use?
- How is sysroot/toolchain specified?

**Look for:**

- `Command::new("rustc")` or similar
- Flag construction logic
- Environment variable setup
- Working directory management

**Capture:**

```rust
// Example of what we want to extract
let rustc_command = Command::new("rustc")
    .arg("--crate-type").arg("dylib")
    .arg("--edition").arg("2021")
    .arg("-C").arg("opt-level=2")
    // ... what else?
```

### 2. Temporary File and Artifact Management

**Questions:**

- Where are temporary .rs files written?
- How are compilation artifacts organized?
- What cleanup happens and when?
- Are temp files reused or recreated?
- How are collisions avoided?
- What happens on crashes (cleanup)?

**Look for:**

- TempDir usage
- File path construction
- Cleanup logic (Drop implementations?)
- File naming schemes

### 3. Dynamic Library Compilation Strategy

**Questions:**

- What crate type is used (cdylib? dylib? rlib?)?
- How is the library ABI managed?
- What symbols are exported?
- How are dependencies linked?
- Are there platform-specific differences (Linux vs. macOS vs. Windows)?

**Look for:**

- `--crate-type` flags
- Symbol export mechanisms
- Linking flags
- Platform-specific code

### 4. Code Generation and Wrapping

**Questions:**

- How is REPL input wrapped for compilation?
- What boilerplate is added around user code?
- How are values returned from compiled code?
- How is the evcxr_runtime integrated?
- Are there entry point conventions?

**Look for:**

- Code template generation
- Wrapper function patterns
- Return value handling
- Runtime integration points

**Example pattern to find:**

```rust
// How does evcxr wrap this:
let x = 42;

// Into compilable Rust:
pub extern "C" fn repl_eval() -> i32 {
    let x = 42;
    x
}
```

### 5. Incremental Compilation Strategy

**Questions:**

- Is incremental compilation used?
- How are incremental artifacts managed?
- What's the cache invalidation strategy?
- How much speedup does incremental provide?
- Are there trade-offs (disk space vs. speed)?

**Look for:**

- `-C incremental=` flags
- Cache directory management
- Dependency tracking
- Performance measurements

### 6. Error Handling and Parsing

**Questions:**

- How are rustc errors captured?
- How are they parsed/formatted?
- How are error positions mapped?
- What error information is preserved?
- How are errors vs. warnings distinguished?

**Look for:**

- Stderr capture from rustc
- Error parsing logic
- Diagnostic formatting
- Source position extraction

### 7. Dependency Management

**Questions:**

- How does `:dep` work under the hood?
- How are external crates added?
- How is Cargo.toml managed?
- Are dependencies compiled incrementally?
- How are version conflicts handled?

**Look for:**

- Cargo.toml generation/modification
- Dependency resolution
- Crate download/caching
- Version handling

### 8. Performance Optimizations

**Questions:**

- What optimizations make REPL compilation fast?
- Are there compilation level flags (-C opt-level)?
- Is there a dev/release mode distinction?
- What's the compilation time for typical REPL expressions?
- Any parallel compilation strategies?

**Look for:**

- Optimization flags
- Benchmarks or performance tests
- Caching strategies
- Time measurements

## Deliverables

Please produce a markdown report at ./workbench/evcxr-compiler-audit-report.md with the following sections:

### 1. Executive Summary

- High-level overview of evcxr compilation architecture
- 3-5 key compilation strategies for REPL
- Estimated compilation times (if available)
- Biggest surprises or insights

### 2. rustc Invocation Reference

Provide the **complete, exact** rustc invocation pattern:

**Minimal Working Example**:

```bash
# The minimal rustc command that produces a working REPL dylib
rustc \
  --crate-type dylib \
  --edition 2021 \
  -o /tmp/output.so \
  input.rs
```

**Full Production Command**:

```bash
# What evcxr actually uses with all flags explained
rustc \
  --crate-type dylib \
  --edition 2021 \
  -C opt-level=2 \          # Why this optimization level?
  -C incremental=/tmp/inc \ # Incremental compilation
  --out-dir /tmp/out \      # Artifact location
  # ... every flag with explanation
  input.rs
```

**Platform-Specific Variations**:

- Linux-specific flags
- macOS-specific flags
- Windows-specific flags

**Environment Variables**:

```bash
RUSTC_WRAPPER=...  # Any wrappers used?
CARGO_HOME=...     # Dependency location?
# ... all relevant env vars
```

### 3. Compilation Pattern Catalog

For each compilation pattern, provide:

**Pattern Name**: Clear, descriptive name

**Description**: What it does and why (2-3 paragraphs)

**Implementation**:

```rust
// Concrete code showing the pattern
// 20-50 lines with comments
```

**Relevance to Oxur**:

- High: We'll definitely use this
- Medium: Might use depending on needs
- Low: Interesting but not applicable

**Complexity**:

- Simple: Easy to implement
- Moderate: Some complexity
- Complex: Significant effort

**Priority**:

- P0: Must have for v1.0
- P1: Should have for v1.0
- P2: Nice to have for v1.0
- P3: Consider for v2.0+

**Integration Notes**: How to adapt for Oxur

**Performance Impact**: Speed/memory/disk trade-offs

---

### Example Pattern Entry

**Pattern Name**: Incremental Compilation with Shared Cache

**Description**:
evcxr maintains a shared incremental compilation cache across all REPL evaluations. Each compilation specifies the same incremental directory, allowing rustc to reuse previously compiled dependencies and metadata. This dramatically speeds up compilation after the first evaluation.

The cache is organized by target directory and never cleared during a REPL session. This trades disk space for compilation speed - acceptable for interactive development.

**Implementation**:

```rust
use std::path::PathBuf;
use std::process::Command;

pub struct Compiler {
    incremental_dir: PathBuf,
    target_dir: PathBuf,
}

impl Compiler {
    pub fn new() -> Self {
        let incremental_dir = std::env::temp_dir()
            .join("oxur-repl-incremental");
        let target_dir = std::env::temp_dir()
            .join("oxur-repl-target");

        // Create directories if they don't exist
        std::fs::create_dir_all(&incremental_dir).unwrap();
        std::fs::create_dir_all(&target_dir).unwrap();

        Self { incremental_dir, target_dir }
    }

    pub fn compile(&self, source_file: &Path) -> Result<PathBuf> {
        let output = Command::new("rustc")
            .arg("--crate-type").arg("dylib")
            .arg("--edition").arg("2021")
            .arg("-C").arg(format!(
                "incremental={}",
                self.incremental_dir.display()
            ))
            .arg("--out-dir").arg(&self.target_dir)
            .arg("-C").arg("opt-level=2")
            .arg(source_file)
            .output()?;

        if !output.status.success() {
            return Err(parse_rustc_error(&output.stderr)?);
        }

        // Find the generated .so/.dylib
        let lib_path = self.find_dylib(&self.target_dir)?;
        Ok(lib_path)
    }
}
```

**Relevance to Oxur**: High - Speeds up compilation significantly

**Complexity**: Simple - Just specify incremental directory

**Priority**: P0 - Essential for acceptable REPL performance

**Integration Notes**:
We should create per-session incremental directories rather than shared global. This provides:

- Better isolation between sessions
- Easier cleanup (delete session dir when session closes)
- No cache poisoning between unrelated evaluations

**Performance Impact**:

- First compilation: No change (~200ms)
- Subsequent compilations: 3-5x faster (~40-60ms)
- Disk space: ~50-100MB per session
- Trade-off: Speed vs. disk space (worth it for REPL)

---

### 4. File Organization Pattern

Document how evcxr organizes temporary files and artifacts:

```
/tmp/evcxr-session-abc123/
├── incremental/              # Incremental compilation cache
│   ├── s-TIMESTAMP/          # Session-specific cache
│   └── ...
├── source/                   # Generated .rs files
│   ├── input_001.rs
│   ├── input_002.rs
│   └── ...
├── target/                   # Compilation artifacts
│   ├── deps/                 # Dependencies
│   ├── librepl_001.so        # Compiled libraries
│   ├── librepl_002.so
│   └── ...
└── Cargo.toml                # Dependency manifest (if used)
```

**Explain:**

- Why this structure?
- What gets created when?
- What gets cleaned up when?
- How to adapt for Oxur?

### 5. Error Handling Deep Dive

**rustc Error Output Format**:

```
error[E0425]: cannot find value `x` in this scope
 --> /tmp/input.rs:2:5
  |
2 |     x
  |     ^ not found in this scope
```

**evcxr's Parsing Strategy**:

```rust
// How evcxr extracts error information
struct RustcError {
    kind: ErrorKind,      // error vs warning
    code: String,         // E0425
    message: String,      // "cannot find value `x`..."
    file: PathBuf,
    line: usize,
    column: usize,
    snippet: String,      // Source context
}
```

**Integration for Oxur**:

- How to map back to original Oxur source?
- Use source maps from compilation chain
- Translate generated.rs:42 → test.ox:5

### 6. Code Generation Templates

Document the patterns evcxr uses to wrap REPL input:

**Template for Expression Evaluation**:

```rust
// User input:
2 + 2

// Generated wrapper:
pub extern "C" fn eval_N() -> Box<dyn std::any::Any> {
    Box::new(2 + 2)
}
```

**Template for Statement Execution**:

```rust
// User input:
let x = 42;

// Generated wrapper:
pub extern "C" fn eval_N() -> Box<dyn std::any::Any> {
    let x = 42;
    Box::new(())
}
```

**Template with Output Capture**:

```rust
// User input:
println!("hello");

// Generated wrapper:
pub extern "C" fn eval_N() -> Box<dyn std::any::Any> {
    evcxr_runtime::install_output_capture();
    println!("hello");
    Box::new(())
}
```

### 7. Performance Benchmarks

If available in tests or docs, extract:

**Compilation Times**:

| Scenario | Cold (no cache) | Warm (cached) |
|----------|----------------|---------------|
| Simple expr (2+2) | 200ms | 50ms |
| Function def | 250ms | 60ms |
| With deps | 500ms+ | 80ms |

**Cache Size**:

| Scenario | Disk Usage |
|----------|-----------|
| Empty cache | 0 MB |
| After 10 evals | 50 MB |
| After 100 evals | 200 MB |

**Optimization Impact**:

| Flag | Time | Binary Size | Notes |
|------|------|-------------|-------|
| opt-level=0 | 150ms | 2MB | Debug |
| opt-level=1 | 180ms | 1MB | Balanced |
| opt-level=2 | 200ms | 500KB | Release |

### 8. Recommendations

#### Must Adopt (P0)

- Incremental compilation strategy
- Dynamic library compilation flags
- Error parsing approach
- [Any other critical patterns]

#### Should Consider (P1)

- Code generation templates
- Temporary file organization
- Performance optimizations
- [Other useful patterns]

#### Can Skip (P2-P3)

- [Patterns not applicable to Oxur]
- [Features we'll implement differently]

#### Oxur-Specific Needs

- **Source map integration**: Map rustc errors to Oxur source
- **Session isolation**: Per-session compilation directories
- **Core Forms → Rust**: Our own lowering, not evcxr's
- [Other unique requirements]

### 9. Integration Roadmap

**Phase 1: Minimal Compilation** (Week 1)

- [ ] Implement basic rustc invocation
- [ ] Get simple expressions compiling
- [ ] Load and execute dynamic libraries
- [ ] Prove the concept works

**Phase 2: Add Incremental** (Week 2)

- [ ] Set up incremental directories
- [ ] Measure speedup
- [ ] Implement cache cleanup

**Phase 3: Error Handling** (Week 3)

- [ ] Parse rustc errors
- [ ] Integrate with source maps
- [ ] Translate to Oxur source positions

**Phase 4: Optimize** (Week 4)

- [ ] Tune compilation flags
- [ ] Benchmark and profile
- [ ] Implement caching strategy

### 10. Code Hotspots

List specific files/functions worth detailed study:

```
evcxr/src/compile.rs:150-200 - rustc invocation
evcxr/src/compile.rs:300-350 - Error parsing
evcxr/src/module.rs:100-150 - Code generation
```

### 11. Questions for Further Investigation

List uncertainties or areas needing deeper research:

- How does incremental compilation interact with multiple sessions?
- What's the memory overhead of keeping libraries loaded?
- Can we safely unload old libraries to free memory?
- How do we handle rustc version differences?
- Are there undocumented rustc flags we should know about?

## Analysis Guidelines

### Do

- ✅ Provide complete, runnable rustc commands
- ✅ Test compilation patterns to verify they work
- ✅ Measure compilation times if possible
- ✅ Note platform differences (Linux, macOS, Windows)
- ✅ Explain *why* each flag/pattern is used
- ✅ Consider integration with Oxur's compilation chain

### Don't

- ❌ Just list flags without explaining their purpose
- ❌ Ignore platform-specific considerations
- ❌ Assume patterns work without testing
- ❌ Focus on evcxr-specific features we won't use
- ❌ Miss critical error handling logic

## Success Criteria

Your audit is successful if:

1. ✅ We can implement `compile_to_dylib()` for Oxur using your findings
2. ✅ We know the exact rustc command and flags to use
3. ✅ We understand temporary file management strategy
4. ✅ We can integrate incremental compilation
5. ✅ We can parse and translate rustc errors
6. ✅ We have performance baselines and expectations

## Repository Location

The evcxr repository has been cloned to your workspace. Focus your analysis on:

```
workbench/evcxr/
├── evcxr/               # Main focus of this audit
│   ├── src/
│   │   ├── lib.rs
│   │   ├── compile.rs   # rustc invocation
│   │   ├── module.rs    # Code generation
│   │   ├── errors.rs    # Error parsing
│   │   └── ...
│   └── Cargo.toml
├── evcxr_repl/          # For usage examples
└── evcxr_runtime/       # For integration patterns
```

## Final Notes

The compilation mechanics are **critical** for Oxur REPL performance. If compilation is too slow, the REPL is unusable. Your job is to extract the patterns that make evcxr's compilation fast enough for interactive use.

**Focus on concrete, reproducible patterns we can implement immediately.**

Good luck! 🦀
