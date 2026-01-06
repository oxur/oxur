---
number: 26
title: "Oxur REPL Evaluation Strategy"
author: "Duncan McGreggor and Claude"
created: 2026-01-02
updated: 2026-01-05
state: Draft
supersedes: null
superseded-by: null
version: 1.2
---


# Oxur REPL Evaluation Strategy

## Overview

This document establishes the evaluation strategy for Oxur's REPL, addressing the fundamental question: **when should we interpret vs. compile Oxur code?**

The answer: **Interpret almost nothing. Compile almost everything.**

## Executive Summary

### Core Decision

Oxur REPL will use a **minimal interpretation** strategy:

- ✅ **Interpret:** Literal arithmetic expressions only (calculator mode)
- ✅ **Compile:** Everything else through Rust pipeline
- ❌ **Never:** Build a full interpreter that duplicates Rust semantics

### Rationale

1. **Safety:** Rust's type system, borrow checker, and compiler catch errors
2. **Consistency:** REPL behavior matches compiled code exactly
3. **Simplicity:** ~100 lines of calculator code vs. ~5000+ for full interpreter
4. **Performance:** Caching makes compilation instant after first use
5. **Maintenance:** No divergent execution paths to maintain

### Integration with Existing Designs

This strategy builds upon:

- [Design Doc 0013: Compilation Chain Architecture](./0013-oxur-compilation-chain-architecture.md) - Full compilation pipeline
- [Design Doc 0018: REPL Protocol Design](./0018-oxur-remote-repl-protocol-design.md) - Network protocol and session management
- [Design Doc 0038: REPL Architecture Overview](./0038-oxur-repl-architecture.md) - Complete system architecture with component locations and data flow (v1.2)

---

## 1. The Interpretation vs. Compilation Tradeoff

### The Slippery Slope

Building an interpreter for Lisp-family languages is tempting but dangerous:

```lisp
;; It starts innocently:
oxur> (+ 1 2)
;; "We could interpret this! It's just addition!"

;; Then you need variables:
oxur> (+ x 2)
;; "Okay, we need an environment for variable lookup..."

;; Then control flow:
oxur> (if (> x 10) (println x) (println "small"))
;; "We need if-expr evaluation, side effects, IO..."

;; Then functions:
oxur> (defn factorial [n] (if (<= n 1) 1 (* n (factorial (- n 1)))))
;; "Now we're building a complete evaluation engine with recursion..."
```

**Every interpreted feature becomes:**

- Code that doesn't benefit from rustc's type checking
- Code that doesn't benefit from rustc's borrow checker
- Code that doesn't benefit from rustc's optimization
- Code you must test and maintain separately
- Code that might behave subtly differently than compiled code

### Lessons from evcxr

The evcxr project initially tried more interpretation. They backed off because:

1. **Semantic divergence:** "Works in REPL but fails when compiled"
2. **Maintenance burden:** Two execution paths to maintain and test
3. **Marginal gains:** Performance benefit was small for real work
4. **Safety loss:** Lost Rust's safety guarantees

They settled on: compile everything, optimize the compilation pipeline.

---

## 2. Proposed Evaluation Strategy

### Three-Tier Execution Model

```
┌─────────────────────────────────────────┐
│      Tier 1: Calculator Mode            │
│                                         │
│  Interpret: Literal arithmetic only     │
│  Examples: (+ 1 2), (* 3 4)             │
│  Time: <1ms                             │
│  Code: ~100 lines                       │
│  Risk: Minimal                          │
└─────────────────────────────────────────┘
                 ↓ (if not literal arithmetic)
┌─────────────────────────────────────────┐
│      Tier 2: Cached Compilation         │
│                                         │
│  Execute: Previously compiled code      │
│  Examples: Repeat evaluations           │
│  Time: 1-5ms (just function call)       │
│  Code: Reuse loaded library             │
│  Risk: None - Rust safety guaranteed    │
└─────────────────────────────────────────┘
                 ↓ (if not cached)
┌─────────────────────────────────────────┐
│      Tier 3: JIT Compilation            │
│                                         │
│  Compile: First-time code               │
│  Examples: New functions, complex forms │
│  Time: 50-300ms (compile + load)        │
│  Code: Full compilation pipeline        │
│  Risk: None - Rust safety guaranteed    │
└─────────────────────────────────────────┘
```

**Why Three Tiers?**

Tier 2 and Tier 3 have significantly different performance characteristics:

- **Tier 2 (Cached):** Library already loaded in subprocess, just call the function - very fast (~1-5ms)
- **Tier 3 (JIT):** Must invoke cargo, compile Rust code, load dylib into subprocess - much slower (~50-300ms)

This distinction is critical for user experience:

- First evaluation of `(defn square [x] (* x x))` → Tier 3 (200ms)
- Subsequent `(square 5)` calls → Tier 2 (2ms)

The tier decision logic requires cache checking and affects whether we show compilation progress indicators.

### 2.1 Component Placement

The three-tier evaluation logic is distributed across components:

```
EvalContext (oxur-repl/src/eval/context.rs)
  ├─ Tier 1 decision: is_simple_arithmetic()
  ├─ Tier 1 execution: eval_calculator()
  └─ Delegates to CachedCompiler for Tier 2/3
       ↓
CachedCompiler (oxur-repl/src/compiler/cached.rs)
  ├─ Tier 2/3 decision: check ArtifactCache
  ├─ Tier 2 execution: execute from loaded library
  └─ Tier 3 execution: compile, cache, load, execute
       ↓
SubprocessExecutor (oxur-repl/src/executor/subprocess.rs)
  └─ Executes all compiled code in isolated process
```

### Tier 1: Calculator Mode

**Only interpret literal arithmetic:**

```rust
// Location: oxur-repl/src/eval/context.rs
// Part of: EvalContext

pub enum InterpretableForm {
    Literal(Literal),
    BinaryOp {
        op: BinOp,
        left: Box<InterpretableForm>,
        right: Box<InterpretableForm>,
    },
}

impl EvalContext {
    fn eval_calculator(&self, form: &CoreForm) -> Result<Value> {
        match form {
            CoreForm::Literal(Literal::Integer(n)) => Ok(Value::Int(*n)),
            CoreForm::BinaryOp { op, left, right, .. } => {
                let l = self.eval_calculator(left)?;
                let r = self.eval_calculator(right)?;
                self.apply_op(*op, l, r)
            }
            _ => Err(Error::NotInterpretable),
        }
    }

    fn is_simple_arithmetic(&self, form: &CoreForm) -> bool {
        match form {
            CoreForm::Literal(_) => true,
            CoreForm::BinaryOp { left, right, .. } => {
                self.is_simple_arithmetic(left) && self.is_simple_arithmetic(right)
            }
            _ => false,
        }
    }
    
    pub fn eval(&mut self, code: &str) -> Result<Value> {
        // Parse code via oxur-lang
        let surface_form = self.parse(code)?;
        let core_form = self.expand(surface_form)?;
        
        // Tier 1: Try calculator mode first
        if self.is_simple_arithmetic(&core_form) {
            return self.eval_calculator(&core_form);
        }
        
        // Tier 2/3: Delegate to compiler (cache check inside)
        self.compiler.eval(core_form).await
    }
}
```

**Interpreted examples:**

```lisp
oxur> 42
42

oxur> (+ 1 2)
3

oxur> (* (+ 2 3) 4)
20

oxur> (/ 100 (- 15 5))
10
```

**Not interpreted (compile instead):**

```lisp
oxur> x
;; Variable reference - needs environment

oxur> (+ x 2)
;; Variable in expression - needs environment

oxur> (println "hello")
;; Side effect - needs Rust runtime

oxur> (if true 1 2)
;; Control flow - compile for consistency

oxur> (defn add [x y] (+ x y))
;; Function definition - always compiled
```

### Tier 2/3: Compilation

**Compile everything else through the full pipeline:**

```rust
// Location: oxur-repl/src/compiler/cached.rs
// Component: CachedCompiler (owned by EvalContext)

pub struct CachedCompiler {
    session_dir: SessionDir,
    state: SessionState,
    executor: SubprocessExecutor,              // MANDATORY - not optional
    rust_ast_wrapper: RustAstWrapper,          // RENAMED from CodeGenerator
    source_map: Arc<SourceMap>,                // from oxur-smap crate
    type_inference: TypeInference,             // NEW - uses rust-analyzer
    cache: Arc<ArtifactCache>,                 // NEW - Phase 0 mandatory
}

impl CachedCompiler {
    pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
        // Generate cache key from form content
        let cache_key = self.generate_cache_key(&form)?;

        // Tier 2/3 decision: Check global artifact cache
        if let Some(artifact_path) = self.cache.get(&cache_key).await? {
            // Tier 2: Artifact exists on disk, load if not in subprocess
            if !self.executor.is_loaded(&cache_key) {
                self.executor.load_library(&artifact_path).await?;
            }
            return self.executor.execute(&cache_key).await;
        }

        // Tier 3: Compile from scratch
        // (with progress indicator for slow compiles >200ms)
        let artifact_path = self.compile(form, &cache_key).await?;
        
        // Store in global cache for future sessions
        self.cache.insert(&cache_key, &artifact_path).await?;
        
        // Load into subprocess and execute
        self.executor.load_library(&artifact_path).await?;
        self.executor.execute(&cache_key).await
    }

    async fn compile(&self, form: CoreForm) -> Result<PathBuf> {
        // Full pipeline from compilation chain doc:
        // 1. Lower Core Forms → Rust AST (via oxur-comp)
        let rust_ast = oxur_comp::lower_to_rust(&form, &self.source_map)?;
        
        // 2. Wrap with REPL scaffolding (RustAstWrapper)
        let wrapped_ast = self.rust_ast_wrapper.wrap(
            rust_ast,
            &self.state,
            &self.source_map
        )?;
        
        // 3. Generate Rust source (via oxur-ast)
        let rust_source = oxur_ast::generate_source(&wrapped_ast)?;
        
        // 4. Write to session directory
        let source_path = self.session_dir.write_source(&rust_source)?;
        
        // 5. Compile to dynamic library (cargo)
        let artifact_path = self.invoke_cargo(&source_path).await?;
        
        Ok(artifact_path)
    }
    
    fn generate_cache_key(&self, form: &CoreForm) -> Result<String> {
        // Content-based addressing (ODD-0038 Decision 5)
        // Cache key: SHA256(source + deps + opt_level + source_map)
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", form).as_bytes());
        hasher.update(self.get_dependencies()?.as_bytes());
        hasher.update(b"opt-level=0");  // Current optimization level
        hasher.update(format!("{:?}", self.source_map).as_bytes());
        
        Ok(format!("{:x}", hasher.finalize()))
    }
}
```

---

## 3. Integration with evcxr Patterns

Based on completed audits (ODD-0027, ODD-0028, ODD-0029), we adopt these proven patterns:

### 3.1 VariableStore Pattern (Type-Erased Storage)

Variables are stored in a type-erased store that lives in the subprocess:

```rust
// Location: oxur-repl/src/subprocess/variable_store.rs
// Module: oxur_variable_store (rebranded from evcxr)

use std::any::Any;
use std::collections::HashMap;

pub struct VariableStore {
    variables: HashMap<String, Box<dyn Any + 'static>>,
}

impl VariableStore {
    pub fn get<T: 'static>(&self, name: &str) -> Option<&T> {
        self.variables
            .get(name)
            .and_then(|v| v.downcast_ref::<T>())
    }

    pub fn set<T: 'static>(&mut self, name: String, value: T) {
        self.variables.insert(name, Box::new(value));
    }
}

// Global static store accessed by generated code
static mut STORE: Option<VariableStore> = None;

pub fn with_store<F, R>(f: F) -> R
where
    F: FnOnce(&mut VariableStore) -> R,
{
    unsafe {
        let store = STORE.get_or_insert_with(VariableStore::new);
        f(store)
    }
}
```

**Key constraints (ODD-0038 Decision 7):**

- `Box<dyn Any + 'static>` requires owned data
- No inter-variable references possible
- Aligns with Lisp semantics (immutable data structures)

### 3.2 Subprocess Execution (Isolation)

All user code executes in an isolated subprocess for safety and interruption support:

```rust
// Location: oxur-repl/src/executor/subprocess.rs
// Component: SubprocessExecutor (MANDATORY - ODD-0038 Decision 3)

pub struct SubprocessExecutor {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    loaded_libraries: HashSet<String>,
}

impl SubprocessExecutor {
    pub async fn load_library(&mut self, path: &Path) -> Result<()> {
        // Send command to subprocess via stdin (ODD-0038 Decision 3a)
        writeln!(self.stdin, "LOAD {}", path.display())?;
        self.stdin.flush()?;
        
        // Wait for acknowledgment
        let mut response = String::new();
        self.stdout.read_line(&mut response)?;
        
        if response.trim() == "LOADED" {
            Ok(())
        } else {
            Err(Error::LoadFailed(response))
        }
    }
    
    pub async fn execute(&mut self, cache_key: &str) -> Result<Response> {
        // Send execution command
        writeln!(self.stdin, "RUN {}", cache_key)?;
        self.stdin.flush()?;
        
        // Collect output until completion marker
        let mut output = String::new();
        loop {
            let mut line = String::new();
            self.stdout.read_line(&mut line)?;
            
            if line.starts_with("OXUR_EXECUTION_COMPLETE") {
                break;
            } else if line.starts_with("OXUR_RUNTIME_ERROR") {
                return Err(Error::RuntimeError(output));
            } else if line.starts_with("OXUR_PANIC_LOCATION") {
                // Parse panic location for better error messages
                return Err(Error::Panic(line));
            } else {
                output.push_str(&line);
            }
        }
        
        Ok(Response::Success { output })
    }
}
```

**Why subprocess is MANDATORY (ODD-0038 Decision 3):**

- Rust threads cannot be interrupted (no pthread_cancel equivalent)
- Ctrl-C support requires process isolation
- Crashes in user code don't corrupt REPL state
- evcxr evidence: Subprocess from day one, unchanged for 6+ years

### 3.3 Subprocess Runtime Binary

The subprocess is a separate binary target within the `oxur-repl` crate:

```toml
# Location: oxur-repl/Cargo.toml
# Binary target configuration (ODD-0038 v1.2)

[[bin]]
name = "oxur-repl-subprocess"
path = "src/bin/subprocess.rs"
```

```rust
// Location: oxur-repl/src/bin/subprocess.rs
// Binary: oxur-repl-subprocess

use oxur_variable_store::VariableStore;
use std::io::{BufRead, BufReader};

fn main() {
    let stdin = BufReader::new(std::io::stdin());
    
    for line in stdin.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        match parts[0] {
            "LOAD" => {
                // Load dynamic library using libloading
                let path = parts[1];
                unsafe {
                    let lib = libloading::Library::new(path).unwrap();
                    // Store lib handle...
                }
                println!("LOADED");
            }
            "RUN" => {
                // Execute function from loaded library
                let cache_key = parts[1];
                let result = execute_function(cache_key);
                println!("{}", result);
                println!("OXUR_EXECUTION_COMPLETE");
            }
            _ => eprintln!("Unknown command: {}", parts[0]),
        }
    }
}
```

**IPC Protocol (ODD-0038 Decision 3a):**

- Uses stdin/stdout text protocol (proven stable in evcxr for 6+ years)
- Simple commands: `LOAD <path>`, `RUN <cache_key>`
- Response markers: `OXUR_EXECUTION_COMPLETE`, `OXUR_RUNTIME_ERROR`, `OXUR_PANIC_LOCATION`
- Unix sockets deferred to v1.1+ (if needed for performance)

### 3.4 Generated Code Structure

Every evaluation generates a small Rust library:

```rust
// Generated by RustAstWrapper (oxur-repl/src/wrapper.rs)
// Wraps lowered Rust AST with REPL scaffolding

use oxur_variable_store::{self, VariableStore};

// User's code (lowered from Core Forms)
fn user_code() -> i32 {
    let x = oxur_variable_store::with_store(|store| {
        store.get::<i32>("x").cloned().unwrap_or(0)
    });
    
    let result = x + 2;
    
    oxur_variable_store::with_store(|store| {
        store.set("result".to_string(), result);
    });
    
    result
}

// Entry point called by subprocess
#[no_mangle]
pub extern "C" fn oxur_eval() -> i32 {
    user_code()
}
```

**Compilation:**

```toml
# Generated Cargo.toml for each evaluation
# Location: {session_dir}/eval_{N}/Cargo.toml

[package]
name = "oxur_eval_42"
version = "0.1.0"
edition = "2021"  # Updated in ODD-0038 v1.2

[lib]
crate-type = ["cdylib"]

[profile.dev]
opt-level = 0  # Fastest REPL iteration (ODD-0038 v1.2)

[dependencies]
oxur-variable-store = { path = "../../../oxur-variable-store" }
```

**Compilation invocation:**

```rust
async fn invoke_cargo(&self, source_path: &Path) -> Result<PathBuf> {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")  // Even with opt-level=0, release mode for cdylib
        .current_dir(source_path.parent().unwrap())
        .output()
        .await?;
    
    if !output.status.success() {
        // Translate rustc errors to Oxur source locations
        let errors = self.translate_errors(&output.stderr)?;
        return Err(Error::CompilationFailed(errors));
    }
    
    Ok(source_path.with_extension("so"))  // Or .dylib on macOS, .dll on Windows
}
```

---

## 4. Handling REPL Modes

The REPL supports two input modes, both using the three-tier strategy:

```rust
// Location: oxur-repl/src/eval/context.rs

pub enum ReplMode {
    Lisp,   // Full Lisp syntax
    Sexpr,  // S-expression only (for testing)
}

impl EvalContext {
    pub fn eval(&mut self, code: &str) -> Result<Value> {
        // Parse according to mode
        let surface_form = match self.mode {
            ReplMode::Lisp => oxur_lang::parse_lisp(code)?,
            ReplMode::Sexpr => oxur_lang::parse_sexpr(code)?,
        };
        
        // Expand to Core Forms (mode-independent from here)
        let core_form = oxur_lang::expand(surface_form)?;
        
        // Apply three-tier strategy (same for both modes)
        if self.is_simple_arithmetic(&core_form) {
            // Tier 1: Calculator
            self.eval_calculator(&core_form)
        } else {
            // Tier 2/3: Check cache, compile if needed
            self.compiler.eval(core_form).await
        }
    }
}
```

**Both modes use same execution tiers:**

- **Tier 1:** Literal arithmetic (mode-independent)
- **Tier 2:** Cached execution (mode-independent)
- **Tier 3:** Full compilation (mode-independent)

---

## 5. Two-Level Caching Strategy

The REPL uses two caches for optimal performance (ODD-0038 Decision 5):

### 5.1 Global Artifact Cache (Disk-Based)

Persistent cache shared across all sessions:

```rust
// Location: oxur-repl/src/cache/artifact.rs
// Component: ArtifactCache (Phase 0 mandatory)

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ArtifactCache {
    cache_dir: PathBuf,  // Default: ~/.cache/oxur/artifacts/
    index: HashMap<String, PathBuf>,
}

pub type SharedCache = Arc<Mutex<ArtifactCache>>;

impl ArtifactCache {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or(Error::NoCacheDir)?
            .join("oxur")
            .join("artifacts");
        
        fs::create_dir_all(&cache_dir)?;
        
        // Load index from disk
        let index = Self::load_index(&cache_dir)?;
        
        Ok(Self { cache_dir, index })
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<PathBuf>> {
        Ok(self.index.get(key).cloned())
    }
    
    pub async fn insert(&mut self, key: &str, artifact_path: &Path) -> Result<()> {
        // Copy artifact to cache directory
        let cache_path = self.cache_dir.join(key);
        fs::create_dir_all(&cache_path)?;
        fs::copy(artifact_path, cache_path.join("lib.so"))?;
        
        // Update index
        self.index.insert(key.to_string(), cache_path);
        self.save_index()?;
        
        Ok(())
    }
}
```

**Cache key algorithm (ODD-0038 Decision 5):**

```
SHA256(
    source_code +
    dependencies +
    optimization_level +
    source_map_config
)
```

**Why disk-based cache matters:**

- Persists across sessions (major performance win)
- Shared between multiple REPL instances
- evcxr's biggest regret: waited 5 years to add caching
- Day-one requirement for Oxur (learn from evcxr's mistake)

### 5.2 Session Library Cache (In-Memory)

Tracks which libraries are loaded in the current subprocess:

```rust
// Location: oxur-repl/src/executor/subprocess.rs
// Part of: SubprocessExecutor

pub struct SubprocessExecutor {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    loaded_libraries: HashSet<String>,  // Cache keys of loaded libs
}

impl SubprocessExecutor {
    pub fn is_loaded(&self, cache_key: &str) -> bool {
        self.loaded_libraries.contains(cache_key)
    }
    
    pub async fn load_library(&mut self, path: &Path) -> Result<()> {
        // ... load library via subprocess protocol ...
        self.loaded_libraries.insert(cache_key.to_string());
        Ok(())
    }
}
```

**Performance impact:**

- **Tier 2 (library loaded):** ~1-5ms (just function call)
- **Tier 3 (cache hit, load needed):** ~10-20ms (load dylib)
- **Tier 3 (cache miss):** ~50-300ms (compile + load)

---

## 6. Preparatory Work Status

### Completed ✅

#### 1. Audit evcxr Projects

**Status:** COMPLETE

**Deliverables:**

- ODD-0027: evcxr_repl Audit - Session management, subprocess pattern, compilation workflow
- ODD-0028: evcxr (Compiler) Audit - Cargo integration, incremental compilation, rustc invocation
- ODD-0029: evcxr_runtime Audit - Discovery that it's just MIME formatting (75 lines)

**Key Findings:**

- Type-erased storage pattern (adopted)
- Subprocess isolation (adopted)
- Cargo-based compilation (adopted)
- evcxr_runtime is minimal (build our own)

**Results synthesized in:** ODD-0030 (REPL Implementation Specification) and ODD-0038 (REPL Architecture)

### Remaining 🔧

#### 2. Define Prototype Oxur Lisp Syntax

**Status:** IN PROGRESS

**Tasks:**

- [ ] Define syntax for 2-3 simple forms (arithmetic, let bindings, functions)
- [ ] Write example `.ox` files using these forms
- [ ] Document expected Rust output for each example
- [ ] Create test suite with expected results

**Blocker:** Requires ODD-0021 (Syntax Design) to be finalized

**Example:**

```lisp
;; test-cases/arithmetic.ox
(+ 1 2)

;; test-cases/let-binding.ox
(let [x 10
      y 20]
  (+ x y))

;; test-cases/function.ox
(defn double [n]
  (* n 2))
```

**Deliverable:** `test-cases/` directory with prototype syntax examples

#### 3. Build Prototype Compiler

**Status:** NOT STARTED

**Dependencies:**

- Prototype syntax definition (above)
- Core Forms specification
- Basic parser implementation in oxur-lang
- Basic lowering in oxur-comp

**Tasks:**

- [ ] Implement parser for prototype syntax → Surface Forms
- [ ] Implement expander for prototype forms → Core Forms
- [ ] Implement lowerer Core Forms → Rust AST (for test cases)
- [ ] Generate Rust source from AST
- [ ] Verify generated code compiles with rustc

**Deliverable:** Working prototype that compiles test cases to Rust

**Success Criteria:**

```bash
$ oxur-proto compile test-cases/arithmetic.ox
Generated: output/arithmetic.rs
Compiled: output/arithmetic

$ ./output/arithmetic
3
```

---

## 7. Why This Preparatory Work Matters

### Avoid Premature Design Decisions

Without hands-on experience:

- We might design APIs that don't fit actual usage
- We might miss important integration points
- We might overcommit to patterns that don't work

### Validate Assumptions

The prototype will reveal:

- Whether our three-tier strategy actually works in practice
- How well the VariableStore pattern integrates with our compiler
- What compilation performance really looks like
- What error messages users actually see
- Cache hit rates and effectiveness

### Build Intuition

Working code provides:

- Concrete examples for documentation
- Test cases for the real implementation
- Muscle memory for Oxur idioms
- Confidence in design decisions
- Early detection of architectural issues

---

## 8. Implementation Phases (Tentative)

These phases are **subject to revision** after remaining preparatory work:

### Phase 0: Foundation (Weeks 1-4)

**Prerequisites (must complete first):**

- ✅ Audit evcxr projects (COMPLETE)
- 🔧 Define prototype syntax (IN PROGRESS)
- 🔧 Build prototype compiler (BLOCKED)
- ⬜ Build oxur-smap crate (NEW - Phase 0 prerequisite per ODD-0038 Decision 2)
- ⬜ Build ArtifactCache infrastructure (NEW - Phase 0 mandatory per ODD-0038 Decision 5)

**Core infrastructure:**

- SessionDir with tmpfs strategy (ODD-0038 Decision 4)
- VariableStore implementation
- Subprocess binary target
- Global artifact cache
- Source mapping foundation

**Validate:** Can compile 3 test cases end-to-end with caching

**Timeline:** 3-4 weeks (was 2-3 weeks in v1.1, +1 week for caching infrastructure)

### Phase 1: Tier 3 Only (Weeks 5-6)

- Implement compilation-only REPL (no cache hit optimization, no calculator)
- Integrate VariableStore pattern
- Get output capture working
- Get subprocess execution working
- Add TypeInference with rust-analyzer (ODD-0038 Decision 6)
- **Validate:** Can evaluate all test cases via compilation

### Phase 2: Add Tier 2 (Week 7)

- Implement session library tracking (is_loaded check)
- Optimize cache hits (skip reload if already loaded)
- Add global artifact cache integration
- Measure cache hit rates
- **Validate:** Repeat evaluations are instant

### Phase 3: Add Tier 1 (Week 8)

- Implement calculator mode
- Add fast path for literal arithmetic
- Add tier decision logic to EvalContext
- **Validate:** Calculator <1ms, Tier 3 <300ms, Tier 2 <5ms

### Phase 4: Protocol Integration (Week 9)

- Connect EvalContext to MessageHandler
- Test via protocol (TCP clients)
- Multi-session testing
- **Validate:** Remote clients can connect and evaluate

### Phase 5: Polish (Week 10)

- Error translation with source maps (oxur-smap)
- Progress indicators for slow compiles (>200ms)
- Comprehensive testing
- Documentation
- Performance tuning

**Total Timeline:** 10 weeks (was 9 weeks in v1.1, adjusted for oxur-smap prerequisite)

---

## 9. Temp Directory Strategy

### Implementation (ODD-0038 Decision 4)

Best-effort tmpfs with graceful fallback:

```rust
// Location: oxur-repl/src/session/dir.rs

pub struct SessionDir {
    path: PathBuf,
    is_tmpfs: bool,
}

impl SessionDir {
    pub fn new(session_id: &SessionId) -> Result<Self> {
        // Try user override first
        if let Ok(dir) = env::var("OXUR_REPL_TEMP_DIR") {
            return Self::create_in(PathBuf::from(dir), session_id);
        }
        
        // Try tmpfs on Linux
        #[cfg(target_os = "linux")]
        if let Ok(tmpfs) = Self::try_tmpfs(session_id) {
            return Ok(tmpfs);
        }
        
        // Fall back to OS temp directory
        let temp_dir = env::temp_dir().join("oxur-repl").join(session_id.to_string());
        Self::create_in(temp_dir, session_id)
    }
    
    #[cfg(target_os = "linux")]
    fn try_tmpfs(session_id: &SessionId) -> Result<Self> {
        let tmpfs_dir = PathBuf::from("/dev/shm")
            .join("oxur-repl")
            .join(session_id.to_string());
        
        fs::create_dir_all(&tmpfs_dir)?;
        
        Ok(Self {
            path: tmpfs_dir,
            is_tmpfs: true,
        })
    }
}
```

**Strategy details:**

- **Linux:** Try `/dev/shm` (RAM-backed tmpfs, ~2-3% faster)
- **macOS/Windows:** Use OS temp directory (good enough with OS caching)
- **Override:** `OXUR_REPL_TEMP_DIR` environment variable
- **Zero configuration:** Works everywhere, optimizes where possible

**Performance impact:**

- Compilation time: ~2-3% faster on tmpfs
- Not dramatic, but free optimization
- More important: Reduces disk wear on SSDs

---

## 10. Success Criteria

The REPL evaluation strategy is successful when:

1. ✅ Calculator mode handles literal arithmetic in <1ms
2. ✅ Tier 2 (cached, loaded) execution is near-instant (<5ms)
3. ✅ Tier 3 (compilation) completes in <300ms (cold), <100ms (warm with cache hit)
4. ✅ Compilation produces identical results to compiled code
5. ✅ Error messages trace back to original Oxur source (via oxur-smap)
6. ✅ Users never encounter "works in REPL but fails when compiled"
7. ✅ Calculator code is <200 lines
8. ✅ No divergent execution paths to maintain
9. ✅ Cache hit rate >80% for typical REPL usage
10. ✅ Subprocess crashes don't lose session state
11. ✅ Global artifact cache persists across sessions
12. ✅ Subprocess isolation enables Ctrl-C interruption

---

## 11. Open Questions

These will be answered during remaining preparatory work and implementation:

1. **Cache Size:** How many compiled libraries should we keep in memory per session?
2. **Cache Eviction:** When should we evict compiled code from global cache? (LRU? Size limit? TTL?)
3. **Progress Indicators:** At what threshold should we show "Compiling..."? (200ms? 500ms?)
4. **Error Translation:** How accurate can we make rustc → Oxur error mapping with oxur-smap?
5. **Memory Management:** How do we safely unload old dynamic libraries? (Or do we need to?)
6. **Subprocess Restart:** When should we restart a crashed subprocess? (Immediate? After N failures?)
7. **Incremental Compilation:** Can we measure the actual speedup in our use case?
8. **Type Inference Accuracy:** How accurate is rust-analyzer for REPL variable types?

---

## 12. References

- [Design Doc 0013: Compilation Chain Architecture](./0013-oxur-compilation-chain-architecture.md)
- [Design Doc 0018: REPL Protocol Design](./0018-oxur-remote-repl-protocol-design.md)
- [Design Doc 0027: evcxr_repl Audit](./0027-evcxr-repl-audit.md)
- [Design Doc 0028: evcxr Compiler Audit](./0028-evcxr-compiler-audit.md)
- [Design Doc 0029: evcxr_runtime Audit](./0029-evcxr-runtime-audit.md)
- [Design Doc 0030: REPL Implementation Specification](./0030-oxur-repl-implementation-specification.md)
- [Design Doc 0038: REPL Architecture Overview](./0038-oxur-repl-architecture.md) - v1.2
- [evcxr Project](https://github.com/evcxr/evcxr)

---

## Conclusion

Oxur's REPL will succeed by **embracing Rust's strengths** rather than reimplementing them:

- **Minimal interpretation** keeps code simple and safe
- **Three-tier execution** optimizes for common patterns (calculator, cached, compile)
- **Compilation through Rust** ensures consistency and correctness
- **Two-level caching** (disk + memory) makes repeated compilation feel instant
- **Subprocess isolation** protects session state from user code crashes
- **VariableStore pattern** enables variable persistence without serialization
- **Source mapping** (oxur-smap) enables rustc-quality error messages for Oxur code
- **rust-analyzer integration** avoids 4 years of compiler error hacks

The slippery slope of building a full interpreter is real and dangerous. By resisting it, we get a simpler, safer, more maintainable REPL that leverages Rust's guarantees instead of bypassing them.

**Critical architectural decisions from ODD-0038 v1.2:**

- ✅ Subprocess execution is MANDATORY (not optional)
- ✅ ArtifactCache is MANDATORY Phase 0 (not deferred)
- ✅ oxur-smap is Phase 0 prerequisite (foundation crate)
- ✅ TypeInference uses rust-analyzer from day one (no compiler hacks)
- ✅ RustAstWrapper clarifies component responsibility (wrapping, not lowering)
- ✅ All protocol markers and modules use OXUR branding (not EVCXR)
- ✅ Subprocess binary is a target within oxur-repl crate (not separate)
- ✅ Rust edition 2021 (stable, widely supported)

**Next Step:** Complete prototype syntax definition and build prototype compiler to validate these design decisions.

---

## Version History

### Version 2.0 (2026-01-05)

**Major Update:** Complete alignment with ODD-0038 v1.2 architecture specification.

**Critical Changes:**

1. **Component Naming Updates**
   - Renamed `CodeGenerator` → `RustAstWrapper` throughout (ODD-0038 Decision 1)
   - Location: `oxur-repl/src/wrapper.rs` (was `src/codegen/generator.rs`)
   - Clarifies responsibility: wraps Rust AST, doesn't do lowering

2. **Protocol Branding Consistency**
   - Updated all markers: `EVCXR_*` → `OXUR_*`
   - `OXUR_EXECUTION_COMPLETE`, `OXUR_RUNTIME_ERROR`, `OXUR_PANIC_LOCATION`
   - Updated module: `evcxr_variable_store` → `oxur_variable_store`

3. **Missing Components Added**
   - Added `cache: Arc<ArtifactCache>` to CachedCompiler (ODD-0038 Decision 5)
   - Added `type_inference: TypeInference` to CachedCompiler (ODD-0038 Decision 6)
   - Location: `oxur-repl/src/type_inference.rs`

4. **Subprocess Architecture Clarified**
   - Changed `subprocess: Option<ChildProcess>` → `executor: SubprocessExecutor`
   - Emphasized MANDATORY status (ODD-0038 Decision 3)
   - Reason: Rust threads cannot be interrupted, subprocess required for Ctrl-C

5. **Rust Edition Updated**
   - Changed `edition = "2024"` → `edition = "2021"`
   - Rationale: Stable, widely supported (ODD-0038 v1.2)

**Major Additions:**

6. **Two-Level Caching Strategy**
   - NEW Section 5: Disk-based artifact cache + session library cache
   - Global cache: `~/.cache/oxur/artifacts/<sha256>/`
   - Session cache: Tracks loaded libraries in subprocess
   - Cache key: SHA256(source + deps + opt_level + source_map)

7. **Temp Directory Strategy**
   - NEW Section 9: Best-effort tmpfs with graceful fallback
   - Linux: `/dev/shm` (RAM-backed)
   - macOS/Windows: OS temp directory
   - Override: `OXUR_REPL_TEMP_DIR` environment variable

8. **Component Locations**
   - Added explicit file paths throughout:
   - EvalContext: `oxur-repl/src/eval/context.rs`
   - CachedCompiler: `oxur-repl/src/compiler/cached.rs`
   - RustAstWrapper: `oxur-repl/src/wrapper.rs`
   - SubprocessExecutor: `oxur-repl/src/executor/subprocess.rs`
   - TypeInference: `oxur-repl/src/type_inference.rs`
   - VariableStore: `oxur-repl/src/subprocess/variable_store.rs`
   - Subprocess binary: `oxur-repl/src/bin/subprocess.rs`

9. **Source Mapping Integration**
   - Added oxur-smap crate references (ODD-0038 Decision 2)
   - Phase 0 prerequisite foundation crate
   - Multi-stage tracking: Surface → Core → Rust → Error translation

10. **Subprocess Binary Packaging**
    - Added Cargo.toml `[[bin]]` configuration example
    - Binary name: `oxur-repl-subprocess`
    - Location: `src/bin/subprocess.rs`

**Implementation Timeline Changes:**

- Phase 0: 3-4 weeks (was 2-3 weeks)
- Added oxur-smap as Phase 0 prerequisite
- Added ArtifactCache as Phase 0 mandatory
- Total: 10 weeks (was 9 weeks)

**Updated Success Criteria:**

- Added: Global artifact cache persists across sessions
- Added: Subprocess isolation enables Ctrl-C interruption
- Added: Error messages trace back via oxur-smap

**Documentation:**

- Updated all references to ODD-0038 (was referencing ODD-0030)
- Added optimization level defaults: `opt-level = 0`
- Clarified IPC protocol uses stdin/stdout text (ODD-0038 Decision 3a)

**Impact:** Complete architectural alignment - evaluation strategy now accurately reflects implementation architecture from ODD-0038 v1.2.

---

### Version 1.1 (2026-01-04)

**Date:** January 3-4, 2026

#### Critical Corrections

- **Section 3: Three-Tier Execution Model** - Corrected from two-tier to three-tier (Calculator, Cached, JIT), removed incorrect note claiming no distinction
- **Section 8: Integration with evcxr Patterns** - Replaced evcxr_runtime usage with actual patterns: VariableStore + Subprocess isolation

#### Major Additions

- **Section 3.1: Component Placement (NEW)** - Shows where evaluation logic lives in architecture (EvalContext, CachedCompiler locations)
- **Architecture integration** - Added context about component ownership and data flow

#### Updates

- **Section 4: Handling REPL Modes** - Added component location context, updated code to show tier decision logic
- **Section 6: Preparatory Work Status** - Marked evcxr audits as complete (ODD-0027, 0028, 0029), updated remaining tasks

#### Key Improvements

- Alignment with implemented architecture from REPL Architecture Overview
- Corrected understanding of evcxr patterns based on completed audits
- Clear component placement in codebase

---

### Version 1.0 (2026-01-02)

Initial evaluation strategy specification.

---

**"Interpret the trivial. Cache the compiled. Trust Rust."**

---

*End of Document*