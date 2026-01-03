# REPL Component Proto-Plans

**Purpose:** Rough implementation sketches for core REPL components to guide detailed design and planning.

**Status:** Proto-plans - NOT full design documents. Each component will need its own detailed design doc or implementation plan.

---

## Table of Contents

### Can Build Independently (No Blockers)

1. [VariableStore](#1-variablestore)
2. [SessionDir](#2-sessiondir)
3. [Cargo Integration](#3-cargo-integration)
4. [Subprocess Runtime](#4-subprocess-runtime)

### Blocked on oxur-lang/oxur-comp

5. [Core Forms Definition](#5-core-forms-definition)
2. [Lowering to Rust AST](#6-lowering-to-rust-ast)
3. [CodeGenerator](#7-codegenerator)
4. [Full Evaluation](#8-full-evaluation)
5. [Source Maps](#9-source-maps)

### Integration Component

10. [CachedCompiler](#10-cachedcompiler)

---

## 1. VariableStore

**Status:** ✅ Can implement now
**Dependencies:** None
**Complexity:** Low (~50 lines)
**Timeline:** 1 day

### Description

Type-erased variable storage using `Box<dyn Any>` pattern from evcxr. Allows persisting arbitrary user-defined types across REPL evaluations without serialization.

### Key Requirements

1. Store variables with `Box<dyn Any + 'static>`
2. Type-safe retrieval with runtime checking
3. Put, check, and take operations
4. No trait bounds on stored types

### Main Components

```rust
pub struct VariableStore {
    variables: HashMap<String, Box<dyn Any + 'static>>,
}

// Three core operations:
impl VariableStore {
    pub fn put_variable<T: 'static>(&mut self, name: &str, value: T);
    pub fn check_variable<T: 'static>(&self, name: &str) -> bool;
    pub fn take_variable<T: 'static>(&mut self, name: &str) -> T;
}
```

### Implementation Tasks

1. **Create module structure**
   - Location: `crates/oxur-repl/src/runtime/`
   - Files: `mod.rs`, `variable_store.rs`

2. **Implement VariableStore struct**
   - HashMap storage
   - Put operation with type erasure
   - Check operation with type validation
   - Take operation with downcast

3. **Add error handling**
   - Variable not found
   - Type mismatch
   - Custom error types

4. **Write comprehensive tests**
   - Store and retrieve primitives (i32, f64, String)
   - Store and retrieve user types
   - Type mismatch detection
   - Variable lifecycle

5. **Document with examples**
   - Doc comments on public API
   - Usage examples in module docs

### Critical Decisions

1. **Panic vs Result?**
   - evcxr panics on type mismatch
   - We could return Result for better error handling
   - Decision: Follow evcxr for v1.0, consider Result in v1.1

2. **Thread safety?**
   - Not needed (single session = single thread)
   - Can add Arc<RwLock> wrapper later if needed

3. **Variable shadowing?**
   - Allow or error?
   - evcxr allows (overwrites)
   - Decision: Allow for v1.0

### Relevant Design Docs

- **ODD-0030** (Section 3.1, ADR-001): Complete specification from evcxr audit
- **ODD-0027**: evcxr_repl audit (variable storage pattern)
- **ODD-0018**: Remote REPL protocol (how variables fit in session state)

### Testing Strategy

```rust
#[test]
fn test_basic_storage() {
    let mut store = VariableStore::new();
    store.put_variable("x", 42i32);
    assert!(store.check_variable::<i32>("x"));
    assert_eq!(store.take_variable::<i32>("x"), 42);
}

#[test]
#[should_panic]
fn test_type_mismatch() {
    let mut store = VariableStore::new();
    store.put_variable("x", 42i32);
    let _ = store.take_variable::<f64>("x"); // Wrong type!
}

#[test]
fn test_user_types() {
    struct Point { x: i32, y: i32 }
    let mut store = VariableStore::new();
    store.put_variable("p", Point { x: 10, y: 20 });
    let p = store.take_variable::<Point>("p");
    assert_eq!(p.x, 10);
}
```

### Success Criteria

- [ ] Can store and retrieve primitives
- [ ] Can store and retrieve user-defined types
- [ ] Type mismatches detected at runtime
- [ ] All tests pass
- [ ] Documentation complete

---

## 2. SessionDir

**Status:** ✅ Can implement now
**Dependencies:** None
**Complexity:** Medium (~200 lines)
**Timeline:** 1 day

### Description

Manages per-session temporary directories containing Cargo project, source files, and compilation artifacts. Handles creation, cleanup, and platform-specific concerns.

### Key Requirements

1. Create unique directory per session
2. Generate Cargo.toml with correct config
3. Manage src/lib.rs generation
4. Handle cleanup on session close
5. Platform-specific file operations (Windows DLL locking)
6. Stale directory cleanup on startup

### Main Components

```rust
pub struct SessionDir {
    root: PathBuf,              // /tmp/oxur-repl/session-{uuid}
    session_id: SessionId,
}

impl SessionDir {
    pub fn new(session_id: SessionId) -> Result<Self>;
    pub fn root(&self) -> &Path;
    pub fn src_path(&self) -> PathBuf;        // root/src/lib.rs
    pub fn cargo_toml_path(&self) -> PathBuf; // root/Cargo.toml
    pub fn target_dir(&self) -> PathBuf;      // root/target

    // Initialization
    pub fn init_cargo_project(&self) -> Result<()>;

    // File operations
    pub fn write_source(&self, code: &str) -> Result<()>;
    pub fn read_artifact(&self, name: &str) -> Result<PathBuf>;

    // Cleanup
    pub fn cleanup(self) -> Result<()>;
    pub fn cleanup_stale_dirs(base: &Path) -> Result<usize>;
}
```

### Implementation Tasks

1. **Directory structure setup**
   - Create base directory: `/tmp/oxur-repl/` or platform equivalent
   - Generate unique session subdirectory using UUID
   - Create `src/` subdirectory

2. **Cargo.toml generation**
   - Template from ODD-0030 section 4.2
   - Package name: "ctx"
   - cdylib crate type
   - opt-level 2, incremental true

3. **Platform-specific handling**
   - Linux: Standard operations, .so extension
   - macOS: .dylib extension, possible timestamp workarounds
   - Windows: .dll extension, copy instead of rename (DLL locking)

4. **Cleanup logic**
   - Normal: Delete entire directory on Drop
   - Stale: On startup, remove dirs >24h old
   - Error handling: Log but don't fail if cleanup fails

5. **Testing**
   - Create and verify directory structure
   - Cargo.toml generation correctness
   - Platform-specific tests
   - Cleanup verification
   - Stale directory detection

### Critical Decisions

1. **Base directory location?**
   - Linux/macOS: `/tmp/oxur-repl/`
   - Windows: `%TEMP%\oxur-repl\`
   - Use `std::env::temp_dir()` for portability

2. **Cleanup strategy?**
   - Implement Drop for automatic cleanup
   - Also provide explicit cleanup() method
   - Log errors, don't panic

3. **Concurrent access?**
   - Each session = separate directory
   - No locking needed between sessions
   - Within session: single-threaded access

### Relevant Design Docs

- **ODD-0030** (Section 5): Complete file system organization spec
- **ODD-0030** (Section 4): Cargo.toml template
- **ODD-0028**: evcxr compilation audit (platform specifics)

### File Structure Example

```
/tmp/oxur-repl/
├── session-550e8400-e29b-41d4-a716-446655440000/
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs
│   └── target/
│       └── debug/
│           ├── libctx.so
│           ├── libeval_001.so
│           ├── libeval_002.so
│           └── incremental/
└── session-6ba7b810-9dad-11d1-80b4-00c04fd430c8/
    └── ...
```

### Testing Strategy

```rust
#[test]
fn test_session_dir_creation() {
    let session_id = SessionId::new();
    let dir = SessionDir::new(session_id).unwrap();

    assert!(dir.root().exists());
    assert!(dir.root().join("src").exists());
}

#[test]
fn test_cargo_toml_generation() {
    let dir = SessionDir::new(SessionId::new()).unwrap();
    dir.init_cargo_project().unwrap();

    let toml = fs::read_to_string(dir.cargo_toml_path()).unwrap();
    assert!(toml.contains("crate-type = [\"cdylib\"]"));
    assert!(toml.contains("opt-level = 2"));
}

#[test]
fn test_cleanup() {
    let dir = SessionDir::new(SessionId::new()).unwrap();
    let root = dir.root().to_path_buf();

    assert!(root.exists());
    dir.cleanup().unwrap();
    assert!(!root.exists());
}
```

### Success Criteria

- [ ] Creates proper directory structure
- [ ] Generates valid Cargo.toml
- [ ] Handles all three platforms correctly
- [ ] Cleanup works reliably
- [ ] Stale directory detection works
- [ ] All tests pass

---

## 3. Cargo Integration

**Status:** ✅ Can implement now
**Dependencies:** SessionDir
**Complexity:** Medium (~300 lines)
**Timeline:** 1-2 days

### Description

Wrapper around cargo invocation for compiling generated Rust code. Parses JSON output, handles errors, manages environment, and returns artifact paths.

### Key Requirements

1. Invoke cargo with correct arguments
2. Set environment variables (CARGO_TARGET_DIR, RUSTFLAGS)
3. Parse JSON message format output
4. Extract compilation errors with spans
5. Find compiled artifact paths
6. Handle platform-specific linker flags
7. Timeout handling for long compilations

### Main Components

```rust
pub struct CargoBuilder {
    session_dir: Arc<SessionDir>,
    rustflags: String,
}

pub struct BuildResult {
    pub artifact_path: PathBuf,
    pub warnings: Vec<CompilerMessage>,
}

pub struct CompilerMessage {
    pub message: String,
    pub level: String,  // "error", "warning", "note"
    pub spans: Vec<Span>,
    pub code: Option<ErrorCode>,
}

impl CargoBuilder {
    pub async fn build(&self) -> Result<BuildResult>;
    fn detect_fast_linker() -> Option<String>;
    fn parse_cargo_output(output: &str) -> Vec<CompilerMessage>;
}
```

### Implementation Tasks

1. **Cargo invocation**
   - Command: `cargo build --message-format=json`
   - Working directory: session root
   - Environment variables setup
   - Async execution with tokio::process

2. **Environment configuration**
   - CARGO_TARGET_DIR: Point to session target dir
   - RUSTFLAGS: Auto-detect mold/lld, set linker
   - Platform-specific target triple

3. **JSON output parsing**
   - Use serde_json for line-by-line parsing
   - Filter for "compiler-message" reasons
   - Extract artifact paths from "compiler-artifact" messages
   - Build structured CompilerMessage types

4. **Error handling**
   - Distinguish errors vs warnings
   - Collect all errors before failing
   - Preserve spans for source mapping
   - Handle cargo invocation failures

5. **Fast linker detection**
   - Check for mold (Linux)
   - Fall back to lld (cross-platform)
   - Use system linker as last resort
   - Cache detection result

6. **Testing**
   - Successful compilation
   - Compilation with warnings
   - Compilation with errors
   - Artifact path extraction
   - Environment variable handling
   - Timeout scenarios

### Critical Decisions

1. **Synchronous vs Async?**
   - Use async (tokio::process::Command)
   - Allows timeout and cancellation
   - Fits with overall async architecture

2. **Error collection strategy?**
   - Collect all errors before returning
   - Return first error immediately?
   - Decision: Collect all (better UX)

3. **Caching compiled artifacts?**
   - Incremental compilation handles this
   - Just ensure CARGO_TARGET_DIR is session-specific
   - No additional caching layer needed

4. **Timeout value?**
   - First compile: 30 seconds
   - Incremental: 10 seconds
   - Configurable per call

### Relevant Design Docs

- **ODD-0030** (Section 4): Complete rustc invocation reference
- **ODD-0030** (Section 7): Error handling strategy
- **ODD-0028**: evcxr compilation audit (cargo usage patterns)

### Cargo Command Example

```bash
# What we're wrapping:
cargo build \
  --message-format=json \
  --target x86_64-unknown-linux-gnu

# With environment:
CARGO_TARGET_DIR=/tmp/oxur-repl/session-abc/target
RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### JSON Parsing Example

```rust
#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    message: Option<CompilerMessage>,
    target: Option<Target>,
}

fn parse_cargo_output(output: &str) -> Result<Vec<CompilerMessage>> {
    output.lines()
        .filter_map(|line| serde_json::from_str::<CargoMessage>(line).ok())
        .filter(|msg| msg.reason == "compiler-message")
        .filter_map(|msg| msg.message)
        .collect()
}
```

### Testing Strategy

```rust
#[tokio::test]
async fn test_successful_build() {
    let session_dir = SessionDir::new(SessionId::new()).unwrap();
    session_dir.write_source("pub fn foo() -> i32 { 42 }").unwrap();

    let builder = CargoBuilder::new(Arc::new(session_dir));
    let result = builder.build().await.unwrap();

    assert!(result.artifact_path.exists());
}

#[tokio::test]
async fn test_compilation_error() {
    let session_dir = SessionDir::new(SessionId::new()).unwrap();
    session_dir.write_source("pub fn foo() -> i32 { \"not an int\" }").unwrap();

    let builder = CargoBuilder::new(Arc::new(session_dir));
    let result = builder.build().await;

    assert!(result.is_err());
    let errors = result.unwrap_err().compiler_messages();
    assert!(!errors.is_empty());
}
```

### Success Criteria

- [ ] Successfully compiles valid Rust code
- [ ] Detects and uses fast linker
- [ ] Parses JSON output correctly
- [ ] Returns proper error messages
- [ ] Finds artifact paths
- [ ] Handles timeouts
- [ ] All tests pass

---

## 4. Subprocess Runtime

**Status:** ✅ Can implement now
**Dependencies:** VariableStore
**Complexity:** Medium-High (~400 lines)
**Timeline:** 2 days

### Description

Separate binary that loads compiled dynamic libraries and executes user code. Communicates via stdin/stdout with main process. Provides isolation so user code crashes don't corrupt REPL state.

### Key Requirements

1. Load dynamic libraries via libloading
2. Execute exported functions with variable store
3. Command protocol (LOAD, PING, etc.)
4. Capture stdout/stderr
5. Signal execution completion
6. Handle panics gracefully
7. Memory safety with FFI

### Main Components

```rust
// Binary: oxur-subprocess

struct Runtime {
    libraries: Vec<Library>,
    variable_store: Box<VariableStore>,
}

// Command protocol
enum Command {
    Load { lib_path: String, fn_name: String },
    Ping,
    Shutdown,
}

// Response protocol
enum Response {
    ExecutionComplete,
    Pong,
    Error(String),
}
```

### Implementation Tasks

1. **Create subprocess binary crate**
   - New crate: `crates/oxur-subprocess/`
   - Binary target, not library
   - Dependencies: libloading, oxur-repl (for VariableStore)

2. **Command loop implementation**
   - Read lines from stdin
   - Parse commands
   - Execute and respond
   - Loop until shutdown or EOF

3. **Library loading**
   - Use libloading::Library
   - Keep libraries loaded (don't unload)
   - Platform-specific symbol lookup
   - Error handling for missing symbols

4. **Function execution**
   - Get function pointer from library
   - Cast to correct signature: `fn(*mut c_void) -> *mut c_void`
   - Pass variable store pointer
   - Execute function
   - Handle panics with catch_unwind

5. **Output capture**
   - Subprocess stdout/stderr goes to parent automatically
   - Parent process captures via pipes
   - No special handling needed in subprocess

6. **Communication protocol**
   - Simple line-based text protocol
   - Format: `COMMAND arg1 arg2 ...`
   - Responses: Single-line confirmations
   - Completion marker: `EVCXR_EXECUTION_COMPLETE`

7. **Testing**
   - Unit tests for command parsing
   - Integration tests with real libraries
   - Panic recovery tests
   - Communication protocol tests

### Critical Decisions

1. **Communication protocol?**
   - Text lines (simple, debuggable)
   - vs Binary (more efficient)
   - Decision: Text for v1.0 (matches evcxr)

2. **Panic handling?**
   - Catch with std::panic::catch_unwind
   - Report error to parent
   - Continue running (don't exit)

3. **Library unloading?**
   - evcxr keeps all libraries loaded
   - Safer (avoids segfaults from dangling references)
   - Accept memory growth
   - Decision: Never unload in v1.0

4. **Variable store location?**
   - Owned by subprocess
   - Passed to each function
   - Persists across executions

### Relevant Design Docs

- **ODD-0030** (Section 3.1): Subprocess binary implementation
- **ODD-0027**: evcxr_repl audit (subprocess pattern)
- **ODD-0018**: Remote REPL protocol (how subprocess fits in architecture)

### Protocol Example

```
# Parent sends:
LOAD /tmp/session-abc/target/debug/libeval_001.so run_user_code_5

# Subprocess executes function, then responds:
EVCXR_EXECUTION_COMPLETE

# Health check:
Parent: PING
Subprocess: PONG
```

### Function Signature

```rust
// What we're calling:
#[no_mangle]
pub extern "C" fn run_user_code_5(
    store_ptr: *mut c_void
) -> *mut c_void {
    let store = unsafe { &mut *(store_ptr as *mut VariableStore) };

    // User code here

    store_ptr
}
```

### Testing Strategy

```rust
#[test]
fn test_command_parsing() {
    let cmd = Command::parse("LOAD /path/to/lib.so func_name").unwrap();
    assert!(matches!(cmd, Command::Load { .. }));
}

#[tokio::test]
async fn test_execution() {
    // Compile simple library
    let lib = compile_test_lib("pub fn test() -> i32 { 42 }");

    // Start subprocess
    let mut subprocess = Subprocess::spawn().await.unwrap();

    // Load and execute
    subprocess.send_load(&lib, "test").await.unwrap();
    let result = subprocess.wait_complete().await.unwrap();

    assert_eq!(result, "EVCXR_EXECUTION_COMPLETE");
}

#[tokio::test]
async fn test_panic_recovery() {
    let lib = compile_test_lib("pub fn test() { panic!(); }");
    let mut subprocess = Subprocess::spawn().await.unwrap();

    subprocess.send_load(&lib, "test").await.unwrap();
    let result = subprocess.wait_complete().await;

    // Should get error, not crash
    assert!(result.is_err());

    // Subprocess still alive
    subprocess.send_ping().await.unwrap();
}
```

### Success Criteria

- [ ] Loads and executes libraries correctly
- [ ] Handles panics without crashing
- [ ] Communication protocol works
- [ ] Variable store persists
- [ ] All tests pass
- [ ] Works on Linux/macOS/Windows

---

## 5. Core Forms Definition

**Status:** ❌ Blocked - needs oxur-lang team
**Dependencies:** oxur-lang design
**Complexity:** High (design + implementation)
**Timeline:** 2-3 weeks (with oxur-lang team)

### Description

Canonical intermediate representation for Oxur code after macro expansion. Bridge between surface syntax and code generation. Must be simple enough for REPL but complete enough for full language.

### Key Requirements

1. Represent all Oxur constructs after macro expansion
2. Typed (or at least type-checkable)
3. Serializable for caching
4. Maps 1:1 to Rust concepts
5. Source position tracking
6. Minimal set for REPL v1.0

### Main Components (Tentative)

```rust
pub enum CoreForm {
    // Literals
    Literal(Literal),

    // Variables
    Var(Symbol),

    // Definition
    Def { name: Symbol, value: Box<CoreForm> },

    // Function application
    Apply { func: Box<CoreForm>, args: Vec<CoreForm> },

    // Primitive operations (for calculator mode)
    PrimOp { op: PrimOp, args: Vec<CoreForm> },

    // Conditionals
    If { cond: Box<CoreForm>, then: Box<CoreForm>, else_: Box<CoreForm> },

    // Sequences
    Do { exprs: Vec<CoreForm> },

    // Function definitions
    Fn { params: Vec<Symbol>, body: Box<CoreForm> },
}

pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

pub enum PrimOp {
    Add, Sub, Mul, Div,  // Arithmetic
    Eq, Lt, Gt,          // Comparison
    // ... more as needed
}
```

### Critical Questions for oxur-lang Team

1. **Scope of v1.0?**
   - What's the minimal set for REPL?
   - Can we defer: structs, traits, macros, modules?
   - Start with: literals, variables, functions, arithmetic?

2. **Type representation?**
   - Fully typed Core Forms?
   - Or type inference layer separate?
   - How do we handle Rust's type system?

3. **Relationship to Surface Forms?**
   - What comes before Core Forms?
   - What macros need to expand?
   - How does parser fit in?

4. **Source position tracking?**
   - Every Core Form has Span?
   - How granular?
   - How to preserve through transformations?

5. **Rust semantics?**
   - Ownership/borrowing in Core Forms?
   - Or only in lowering?
   - How to represent mut vs immut?

### Relevant Design Docs

- **ODD-0030** (Section 3): Mentions Core Forms as input to CodeGenerator
- **ODD-0001**: Oxur Letter of Intent (language philosophy)
- **ODD-0013**: Compilation chain (where Core Forms fit)
- **Need:** Design doc specifically for Core Forms IR

### Integration Points

```rust
// How REPL will use Core Forms:

// 1. Parser produces Core Form
let core_form = oxur_lang::parse(source)?;

// 2. REPL passes to CodeGenerator
let rust_code = code_generator.generate(&core_form)?;

// 3. Compile and execute
let result = compiler.compile_and_run(&rust_code)?;
```

### Minimal Set for REPL v1.0

To enable basic REPL functionality:

```oxur
; These should work in v1.0:
(def x 42)              ; Variable definition
(+ x 1)                 ; Arithmetic with variables
(if (> x 0) "pos" "neg") ; Conditionals
(fn [a b] (+ a b))      ; Function definitions
```

### Testing Strategy

```rust
#[test]
fn test_literal_forms() {
    let form = CoreForm::Literal(Literal::Int(42));
    // Serialize/deserialize
    // Lower to Rust
}

#[test]
fn test_def_and_use() {
    let def = CoreForm::Def {
        name: "x".into(),
        value: Box::new(CoreForm::Literal(Literal::Int(42))),
    };
    let use_ = CoreForm::Var("x".into());
    // Test lowering together
}
```

### Success Criteria

- [ ] Designed in collaboration with oxur-lang
- [ ] Covers minimal REPL use cases
- [ ] Can be lowered to Rust AST
- [ ] Source positions preserved
- [ ] Documented with examples
- [ ] Integrated into oxur-lang crate

### Recommendation

**Create dedicated design doc:** "ODD-0032: Core Forms Intermediate Representation"

Should cover:

- Complete grammar
- Type system integration
- Relationship to surface syntax
- Lowering strategy to Rust
- Evolution path to full language

---

## 6. Lowering to Rust AST

**Status:** ❌ Blocked - needs oxur-comp team
**Dependencies:** Core Forms, oxur-ast
**Complexity:** High
**Timeline:** 3-4 weeks

### Description

Transformation layer that converts Core Forms to Rust AST. The "compiler" part of the Oxur compiler. Must handle Oxur semantics while generating valid, idiomatic Rust.

### Key Requirements

1. Core Form → Rust AST conversion
2. Variable scoping and lifetime management
3. Type inference/checking
4. Error messages with source positions
5. Generate idiomatic Rust
6. Support REPL-specific patterns (variable persistence)

### Main Components

```rust
pub struct Lowerer {
    context: LoweringContext,
    type_ctx: TypeContext,
}

pub struct LoweringContext {
    // Track variables in scope
    variables: HashMap<Symbol, VarInfo>,

    // Track types
    types: HashMap<Symbol, Type>,

    // Current function context
    function_depth: usize,
}

impl Lowerer {
    pub fn lower(&mut self, form: &CoreForm) -> Result<syn::Expr>;

    fn lower_literal(&self, lit: &Literal) -> syn::Expr;
    fn lower_def(&mut self, name: Symbol, value: &CoreForm) -> Result<syn::Stmt>;
    fn lower_apply(&mut self, func: &CoreForm, args: &[CoreForm]) -> Result<syn::Expr>;
    fn lower_if(&mut self, cond: &CoreForm, then: &CoreForm, else_: &CoreForm) -> Result<syn::Expr>;
}
```

### Implementation Tasks

1. **Literal lowering**
   - Int → `syn::ExprLit` with i64
   - Float → f64
   - String → String
   - Bool → bool
   - Simple, straightforward

2. **Variable definitions**
   - `(def x 42)` → `let mut x = 42;`
   - Track in context
   - Handle shadowing
   - Type inference

3. **Variable references**
   - `x` → reference the variable
   - Check if in scope
   - Type checking

4. **Function application**
   - `(f a b)` → `f(a, b)`
   - Handle method calls
   - Handle operators specially

5. **Primitive operations**
   - `(+ a b)` → `a + b`
   - Map PrimOp → Rust operators
   - Type coercion if needed

6. **Control flow**
   - `(if cond then else)` → `if cond { then } else { else_ }`
   - Blocks for sequences
   - Return values

7. **Function definitions**
   - `(fn [x] body)` → closure or function
   - Parameter handling
   - Capture analysis

8. **REPL-specific**
   - Variable persistence across evals
   - Top-level expressions
   - Statement vs expression context

### Critical Decisions

1. **Type inference strategy?**
   - Fully infer types?
   - Require type annotations?
   - Gradual typing?
   - Decision: Start with simple inference, expand later

2. **Variable mutability?**
   - All `mut` for REPL simplicity?
   - Track actual mutations?
   - Decision: All `mut` in v1.0 (matches evcxr)

3. **Error handling?**
   - Oxur exceptions → Rust Result?
   - Panic-based (simpler)?
   - Decision: Defer to v1.1

4. **Ownership model?**
   - How to represent in Core Forms?
   - Explicit annotations?
   - Inferred from usage?
   - Decision: CRITICAL - needs design

### Relevant Design Docs

- **ODD-0030** (Section 3): Code generation strategy (shows expected output)
- **ODD-0013**: Compilation chain architecture
- **ODD-0003**: AST representation (target format)
- **Need:** Design doc for lowering semantics

### Example Lowerings

```rust
// Input Core Form:
Def {
    name: "x",
    value: Literal(Int(42))
}

// Output Rust AST:
syn::Stmt::Local(Local {
    pat: syn::Pat::Ident("x"),
    init: Some(syn::Expr::Lit(42)),
    attrs: vec![],
})

// Generated Rust:
let mut x = 42;
```

```rust
// Input:
Apply {
    func: Var("+"),
    args: [Var("x"), Literal(Int(1))]
}

// Output:
syn::Expr::Binary(Binary {
    left: syn::Expr::Path("x"),
    op: syn::BinOp::Add,
    right: syn::Expr::Lit(1),
})

// Generated:
x + 1
```

### Integration with REPL

```rust
// How REPL will use lowering:

// 1. Parse to Core Form
let core_form = parser.parse("(+ x 1)")?;

// 2. Lower to Rust AST
let rust_ast = lowerer.lower(&core_form)?;

// 3. Generate Rust source (using oxur-ast)
let rust_source = printer.print_expr(&rust_ast);

// 4. Wrap in function, compile, execute
```

### Testing Strategy

```rust
#[test]
fn test_lower_literal() {
    let form = CoreForm::Literal(Literal::Int(42));
    let ast = Lowerer::new().lower(&form).unwrap();
    // Verify it's syn::ExprLit with value 42
}

#[test]
fn test_lower_arithmetic() {
    let form = CoreForm::PrimOp {
        op: PrimOp::Add,
        args: vec![
            CoreForm::Literal(Literal::Int(1)),
            CoreForm::Literal(Literal::Int(2)),
        ],
    };
    let ast = Lowerer::new().lower(&form).unwrap();
    // Verify it's syn::ExprBinary with Add op
}

#[test]
fn test_variable_scoping() {
    // Define x, use x, shadow x
    // Verify scoping rules
}
```

### Success Criteria

- [ ] Lowers all minimal Core Forms
- [ ] Generates valid Rust AST
- [ ] Type inference works
- [ ] Variable scoping correct
- [ ] Error messages useful
- [ ] Integration tests pass

### Recommendation

**Create dedicated design doc:** "ODD-0033: Oxur to Rust Lowering Semantics"

Should cover:

- Lowering rules for each Core Form
- Type inference algorithm
- Variable scoping rules
- Ownership representation
- Error handling strategy
- REPL-specific considerations

---

## 7. CodeGenerator

**Status:** ❌ Blocked - needs Core Forms
**Dependencies:** Core Forms, Lowering, VariableStore
**Complexity:** Medium-High (~500 lines)
**Timeline:** 1-2 weeks (after dependencies ready)

### Description

Wraps the lowering process and generates complete Rust library source with variable persistence, source map comments, and wrapper functions for REPL execution.

### Key Requirements

1. Generate complete lib.rs from Core Form
2. Add VariableStore integration
3. Insert source map comments (`/* oxur_node=N */`)
4. Create wrapper function for execution
5. Handle variable check/load/store
6. Track what variables are in scope
7. Generate unique function names per eval

### Main Components

```rust
pub struct CodeGenerator {
    lowerer: Lowerer,
    eval_counter: u64,
}

pub struct GeneratedCode {
    pub source: String,         // Complete lib.rs
    pub fn_name: String,        // "run_user_code_42"
    pub node_map: NodeMap,      // For source mapping
    pub variables_used: Vec<Symbol>,
}

impl CodeGenerator {
    pub fn generate(
        &mut self,
        form: &CoreForm,
        variables: &VariableContext,
    ) -> Result<GeneratedCode>;
}
```

### Implementation Tasks

1. **Template generation**
   - Start with template from ODD-0030 section 3.1
   - Include VariableStore module
   - Generate wrapper function structure

2. **Core Form lowering**
   - Use Lowerer to get Rust AST
   - Convert AST to source string (oxur-ast printer)
   - Preserve structure for source mapping

3. **Source map insertion**
   - Track Core Form Node IDs
   - Insert comments: `/* oxur_node=42 */`
   - Map to original source positions
   - Build NodeMap for later error translation

4. **Variable handling**
   - Analyze Core Form for variable usage
   - Generate check statements
   - Generate load statements
   - Generate store statements
   - Handle type information

5. **Function wrapper**
   - Unique name per eval: `run_user_code_{N}`
   - Signature: `fn(store_ptr) -> store_ptr`
   - Proper extern "C" declaration

6. **Testing**
   - Simple expressions
   - Variable definitions
   - Variable usage
   - Source map correctness
   - Round-trip compilation

### Critical Decisions

1. **Source map granularity?**
   - Every Core Form node?
   - Only top-level?
   - Expression boundaries?
   - Decision: Every node for best error messages

2. **Variable type tracking?**
   - Static type inference?
   - Dynamic type checking?
   - Type annotations in Core Forms?
   - Decision: Depends on Core Forms design

3. **Template vs builder?**
   - String template with placeholders?
   - Build syn nodes programmatically?
   - Decision: Build with syn (type-safe)

### Relevant Design Docs

- **ODD-0030** (Section 3.1, ADR-006): Complete code generation strategy
- **ODD-0030** (Section 7): Error translation (source maps)
- **ODD-0028**: evcxr code generation patterns

### Generated Code Example

```rust
// Generated lib.rs

mod evcxr_variable_store {
    use std::any::Any;
    use std::collections::HashMap;

    pub struct VariableStore {
        variables: HashMap<String, Box<dyn Any>>,
    }

    impl VariableStore {
        pub fn put_variable<T: 'static>(&mut self, name: &str, value: T) {
            self.variables.insert(name.to_owned(), Box::new(value));
        }

        pub fn check_variable<T: 'static>(&self, name: &str) -> bool {
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

#[no_mangle]
pub extern "C" fn run_user_code_42(
    mut store_ptr: *mut evcxr_variable_store::VariableStore
) -> *mut evcxr_variable_store::VariableStore {
    let store = unsafe { &mut *store_ptr };

    // Check and load variable x
    if !store.check_variable::<i32>("x") {
        return store_ptr;
    }
    let mut x = store.take_variable::<i32>("x");

    // User code with source map
    /* oxur_node=42 */ let result = /* oxur_node=43 */ x + /* oxur_node=44 */ 1;

    // Store variables
    store.put_variable("x", x);
    store.put_variable("result", result);

    store_ptr
}
```

### Testing Strategy

```rust
#[test]
fn test_generate_simple_literal() {
    let form = CoreForm::Literal(Literal::Int(42));
    let vars = VariableContext::new();

    let code = CodeGenerator::new().generate(&form, &vars).unwrap();

    assert!(code.source.contains("42"));
    assert!(code.source.contains("run_user_code_"));
}

#[test]
fn test_variable_usage() {
    let form = CoreForm::Var("x".into());
    let mut vars = VariableContext::new();
    vars.define("x", Type::Int);

    let code = CodeGenerator::new().generate(&form, &vars).unwrap();

    assert!(code.source.contains("check_variable::<i32>(\"x\")"));
    assert!(code.source.contains("take_variable::<i32>(\"x\")"));
    assert!(code.variables_used.contains(&"x".into()));
}

#[test]
fn test_source_map_insertion() {
    let form = CoreForm::PrimOp {
        op: PrimOp::Add,
        args: vec![
            CoreForm::Literal(Literal::Int(1)),
            CoreForm::Literal(Literal::Int(2)),
        ],
    };

    let code = CodeGenerator::new().generate(&form, &VariableContext::new()).unwrap();

    // Should have source map comments
    assert!(code.source.contains("/* oxur_node="));
}
```

### Success Criteria

- [ ] Generates valid Rust code
- [ ] Includes VariableStore integration
- [ ] Source maps inserted correctly
- [ ] Variable handling works
- [ ] Compiles successfully
- [ ] Round-trip tests pass

---

## 8. Full Evaluation

**Status:** ❌ Blocked - needs all above
**Dependencies:** All previous components
**Complexity:** High (integration)
**Timeline:** 2-3 weeks (after dependencies)

### Description

Integration of all components into complete evaluation pipeline. Handles Tier 1 (calculator), Tier 2 (compilation), caching, and error recovery.

### Key Requirements

1. Tier 1: Fast calculator mode (<1ms)
2. Tier 2: Full compilation path
3. Tier selection heuristic
4. Result caching
5. Error recovery (don't corrupt state)
6. Clone-try-commit pattern
7. Performance optimization

### Main Components

```rust
pub struct Evaluator {
    session_id: SessionId,
    session_dir: Arc<SessionDir>,
    subprocess: Option<Subprocess>,

    compiler: CachedCompiler,
    calculator: Calculator,

    state: EvalState,
}

pub struct EvalState {
    variables: VariableContext,
    eval_counter: u64,
    loaded_libraries: Vec<String>,
}

impl Evaluator {
    pub async fn eval(&mut self, source: &str) -> Result<EvalResult>;

    fn try_tier1(&self, form: &CoreForm) -> Option<Value>;
    async fn eval_tier2(&mut self, form: &CoreForm) -> Result<EvalResult>;
}
```

### Implementation Tasks

1. **Tier 1 Calculator**
   - Pattern match on Core Forms
   - Direct evaluation for:
     - Literals
     - Arithmetic with known values
     - Simple variable lookups
   - Return None if can't handle

2. **Tier 2 Compilation**
   - Use CachedCompiler
   - Full code generation
   - Compilation and execution
   - Result extraction

3. **Tier selection**
   - Try Tier 1 first (always)
   - Fall back to Tier 2
   - Log tier used for debugging

4. **Clone-try-commit**
   - Clone state before eval
   - Update tentative state
   - Compile with tentative state
   - Only commit if successful
   - Rollback on error

5. **Caching strategy**
   - Cache compiled libraries
   - Reuse if code unchanged
   - Invalidate on variable type change
   - Track cache hits/misses

6. **Error handling**
   - Compilation errors
   - Runtime errors
   - Subprocess crashes
   - State corruption detection

7. **Integration testing**
   - Full evaluation pipeline
   - All error paths
   - Performance benchmarks
   - Stress tests

### Critical Decisions

1. **When to use Tier 1?**
   - Only literals and simple arithmetic?
   - Also simple variable lookups?
   - Decision: Conservative in v1.0, expand based on data

2. **Caching granularity?**
   - Per expression?
   - Per session?
   - Decision: Per compiled library (matches evcxr)

3. **Subprocess restart policy?**
   - On crash: always restart?
   - On error: keep running?
   - Decision: Restart on crash, continue on error

4. **Performance targets?**
   - Tier 1: <1ms (strict)
   - Tier 2 first: 200-300ms (acceptable)
   - Tier 2 warm: 50-100ms (target)

### Relevant Design Docs

- **ODD-0030** (Section 6): Performance expectations
- **ODD-0030** (Section 3): Full implementation spec
- **ODD-0026**: Tiered execution strategy design
- **ODD-0018**: REPL evaluation strategy (from original spec)

### Evaluation Flow

```
User input: "(+ 1 2)"
    ↓
Parse → Core Form
    ↓
Try Tier 1 Calculator
    ↓ (success: <1ms)
Return Value(3)

User input: "(def x 42)"
    ↓
Parse → Core Form
    ↓
Try Tier 1 Calculator
    ↓ (fail: needs compilation)
Tier 2: Clone state
    ↓
Generate code
    ↓
Compile (200ms first time, 50ms later)
    ↓
Execute in subprocess
    ↓
Commit state
    ↓
Return Success
```

### Testing Strategy

```rust
#[tokio::test]
async fn test_tier1_arithmetic() {
    let mut eval = Evaluator::new(session_id);

    let result = eval.eval("(+ 1 2)").await.unwrap();

    // Should be fast
    assert_eq!(result.value, Value::Int(3));
    assert!(result.duration < Duration::from_millis(1));
    assert_eq!(result.tier, Tier::One);
}

#[tokio::test]
async fn test_tier2_variable_def() {
    let mut eval = Evaluator::new(session_id);

    let result = eval.eval("(def x 42)").await.unwrap();

    assert!(result.duration > Duration::from_millis(50)); // Compiled
    assert_eq!(result.tier, Tier::Two);

    // Variable should persist
    let result2 = eval.eval("x").await.unwrap();
    assert_eq!(result2.value, Value::Int(42));
}

#[tokio::test]
async fn test_error_recovery() {
    let mut eval = Evaluator::new(session_id);

    eval.eval("(def x 42)").await.unwrap();

    // This should fail
    let result = eval.eval("(def x \"not an int\")").await;
    assert!(result.is_err());

    // Original state should be intact
    let result2 = eval.eval("x").await.unwrap();
    assert_eq!(result2.value, Value::Int(42));
}
```

### Success Criteria

- [ ] Tier 1 works for simple cases
- [ ] Tier 2 compiles and executes
- [ ] Tier selection is correct
- [ ] Clone-try-commit works
- [ ] Errors don't corrupt state
- [ ] Performance targets met
- [ ] All integration tests pass

---

## 9. Source Maps

**Status:** ❌ Blocked - needs parser integration
**Dependencies:** Parser, Core Forms, CodeGenerator
**Complexity:** Medium (~300 lines)
**Timeline:** 1 week

### Description

Multi-stage source mapping that translates Rust compiler errors back to original Oxur source positions. Critical for good developer experience.

### Key Requirements

1. Track source positions through pipeline
2. Oxur source → Surface Forms → Core Forms → Rust AST
3. Node ID assignment at each stage
4. Bidirectional lookup
5. Fuzzy matching for robustness
6. Error position translation

### Main Components

```rust
pub struct SourceMap {
    // Original source
    oxur_source: String,

    // Node ID mappings
    surface_map: HashMap<NodeId, SourcePos>,
    core_to_surface: HashMap<NodeId, NodeId>,
    rust_to_core: HashMap<NodeId, NodeId>,
}

pub struct SourcePos {
    file: String,
    line: usize,
    col: usize,
    len: usize,
}

impl SourceMap {
    pub fn lookup(&self, node_id: NodeId) -> Option<SourcePos>;
    pub fn translate_error(&self, rustc_error: &CompilerMessage) -> Option<OxurError>;
}
```

### Implementation Tasks

1. **Node ID generation**
   - Assign unique IDs during parsing
   - Preserve through transformations
   - Include in AST nodes

2. **Position tracking**
   - Parser records Oxur positions
   - Core Forms preserve Node IDs
   - CodeGenerator emits comments

3. **Map building**
   - Three-level mapping structure
   - Efficient lookup (HashMap)
   - Serialize for caching

4. **Error translation**
   - Parse rustc error JSON
   - Extract span and message
   - Find Node ID from generated source
   - Lookup original position
   - Rebuild error with Oxur positions

5. **Fuzzy matching**
   - If exact Node ID not found
   - Try nearby lines
   - Return best guess + confidence
   - Show both Rust and Oxur if uncertain

6. **Testing**
   - Round-trip position mapping
   - Error translation accuracy
   - Edge cases (macros, generated code)

### Critical Decisions

1. **Node ID assignment strategy?**
   - Global counter?
   - Hash-based?
   - Decision: Global counter (simpler, debuggable)

2. **Granularity?**
   - Every subexpression?
   - Only top-level forms?
   - Decision: Every Core Form node

3. **Multi-line expressions?**
   - Start position only?
   - Start + length?
   - Decision: Start + length for better highlighting

4. **Fallback strategy?**
   - Show Rust error if translation fails?
   - Show both?
   - Decision: Show both with note about translation

### Relevant Design Docs

- **ODD-0030** (Section 7): Error handling and translation strategy
- **ODD-0030** (ADR-005): Source mapping architecture
- **ODD-0028**: evcxr error handling patterns

### Source Map Flow

```
Oxur: (def x (+ 1 "not a number"))
      ^       ^  ^ ^              ^
   NodeId:    5  6 7              8

Parse → Surface → Core → Rust

Generated Rust:
/* oxur_node=5 */ let mut x = /* oxur_node=6 */ 1
                               /* oxur_node=7 */ +
                               /* oxur_node=8 */ "not a number";

rustc error: lib.rs:10:15 "cannot add String to i32"

Extract line 10, find /* oxur_node=7 */
Lookup node 7 → SourcePos { line: 1, col: 14 }

Translate: "test.ox:1:14 cannot add String to i32"
```

### Error Translation Example

```rust
impl SourceMap {
    pub fn translate_error(&self, msg: &CompilerMessage) -> Option<OxurError> {
        // 1. Get span
        let span = msg.primary_span()?;

        // 2. Read line from generated Rust
        let line = read_line(&span.file_name, span.line_start)?;

        // 3. Extract node ID
        let node_id = extract_node_id_near_column(&line, span.column_start)?;

        // 4. Lookup original position
        let pos = self.lookup(node_id)?;

        // 5. Build Oxur error
        Some(OxurError {
            message: msg.message.clone(),
            file: pos.file.clone(),
            line: pos.line,
            column: pos.col,
            code: msg.code.clone(),
            original_rust_error: Some(msg.clone()),
        })
    }
}

fn extract_node_id_near_column(line: &str, col: usize) -> Option<NodeId> {
    // Find /* oxur_node=N */ near column
    // Search backward from column
    // Parse N and return
}
```

### Testing Strategy

```rust
#[test]
fn test_position_tracking() {
    let source = "(def x (+ 1 2))";
    let parsed = Parser::parse(source).unwrap();

    // Check that positions are recorded
    let map = parsed.source_map();
    let plus_pos = map.lookup(get_plus_node_id(&parsed));

    assert_eq!(plus_pos.line, 1);
    assert_eq!(plus_pos.col, 8); // Position of '+'
}

#[test]
fn test_error_translation() {
    let source = "(def x (+ 1 \"string\"))";
    let result = eval_and_get_error(source);

    // Should point to the addition, not generated Rust
    assert_eq!(result.file, "test.ox");
    assert_eq!(result.line, 1);
    assert_eq!(result.column, 8); // The '+' operator
}
```

### Success Criteria

- [ ] Positions tracked through pipeline
- [ ] Node IDs preserved
- [ ] rustc errors translate correctly
- [ ] Fuzzy matching works
- [ ] Error messages clear
- [ ] All tests pass

---

## 10. CachedCompiler

**Status:** ❌ Blocked - needs ALL above
**Dependencies:** Everything
**Complexity:** High (orchestration)
**Timeline:** 2-3 weeks (final integration)

### Description

Main orchestrator that ties everything together. Manages session state, coordinates compilation, executes code, handles errors, and provides the public API for REPL evaluation.

### Key Requirements

1. Coordinate all components
2. Session lifecycle management
3. Clone-try-commit state management
4. Subprocess management
5. Error recovery
6. Performance monitoring
7. Public REPL API

### Main Components

```rust
pub struct CachedCompiler {
    session_id: SessionId,
    session_dir: Arc<SessionDir>,

    state: SessionState,
    subprocess: Option<Subprocess>,

    code_gen: CodeGenerator,
    cargo: CargoBuilder,
    source_map: Arc<SourceMap>,
}

pub struct SessionState {
    variables: VariableContext,
    eval_counter: u64,
    loaded_libs: Vec<PathBuf>,
}

impl CachedCompiler {
    pub async fn new(session_id: SessionId) -> Result<Self>;
    pub async fn eval(&mut self, form: CoreForm) -> Result<Response>;
    pub async fn shutdown(self) -> Result<()>;

    async fn compile_to_dylib(&self, code: &GeneratedCode) -> Result<PathBuf>;
    async fn execute(&mut self, artifact: &PathBuf, fn_name: &str) -> Result<ExecResult>;
}
```

### Implementation Tasks

1. **Initialization**
   - Create SessionDir
   - Initialize Cargo project
   - Spawn subprocess
   - Setup state

2. **Evaluation pipeline**
   - Clone state (tentative)
   - Generate code
   - Compile to dylib
   - Execute in subprocess
   - Commit state on success

3. **Compilation**
   - Use CodeGenerator
   - Invoke CargoBuilder
   - Handle errors with source maps
   - Track artifacts

4. **Execution**
   - Send LOAD to subprocess
   - Capture output
   - Wait for completion
   - Extract result

5. **Error handling**
   - Compilation errors → translate
   - Runtime errors → report
   - Subprocess crash → restart
   - State rollback on failure

6. **Resource management**
   - Cleanup on shutdown
   - Subprocess lifecycle
   - File cleanup

7. **Testing**
   - Full integration tests
   - Error scenarios
   - Performance tests
   - Resource leak detection

### Critical Decisions

1. **State cloning strategy?**
   - Deep clone everything?
   - Only clone variable metadata?
   - Decision: Clone metadata only (cheap)

2. **Subprocess restart policy?**
   - When to restart?
   - How to detect need?
   - Decision: Restart on panic/crash, keep on error

3. **Compilation caching?**
   - When to reuse artifacts?
   - Cache invalidation?
   - Decision: Rely on incremental compilation

4. **Concurrency?**
   - One eval at a time?
   - Allow concurrent?
   - Decision: Sequential in v1.0 (simpler)

### Relevant Design Docs

- **ODD-0030** (Section 3): Complete implementation specification
- **ODD-0030** (ADR-008): Session state management
- **ODD-0026**: Tiered execution strategy
- **ODD-0018**: Remote REPL protocol integration

### Evaluation Flow

```rust
pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
    // 1. Clone state
    let mut tentative = self.state.clone();
    tentative.eval_counter += 1;

    // 2. Generate code
    let code = self.code_gen.generate(&form, &tentative.variables)?;

    // 3. Compile
    let artifact = self.compile_to_dylib(&code).await
        .map_err(|e| self.translate_error(e))?;

    // 4. Execute
    let result = self.execute(&artifact, &code.fn_name).await?;

    // 5. Commit
    self.state = tentative;

    // 6. Return
    Ok(Response {
        value: result.value,
        out: result.stdout,
        err: result.stderr,
        status: vec![],
    })
}
```

### Integration with Server

```rust
// In MessageHandler::handle()

match request.operation {
    Operation::Eval { code, mode } => {
        // 1. Get or create CachedCompiler for session
        let compiler = self.get_compiler(&request.session_id).await?;

        // 2. Parse code to Core Form
        let form = oxur_lang::parse(&code)?;

        // 3. Evaluate
        let response = compiler.eval(form).await?;

        // 4. Return result
        Ok(Response {
            request_id: request.id,
            session_id: request.session_id,
            result: OperationResult::Success {
                value: response.value,
                output: response.out,
            },
        })
    }
}
```

### Testing Strategy

```rust
#[tokio::test]
async fn test_full_pipeline() {
    let compiler = CachedCompiler::new(SessionId::new()).await.unwrap();

    // Define variable
    let form1 = CoreForm::Def {
        name: "x".into(),
        value: Box::new(CoreForm::Literal(Literal::Int(42))),
    };
    compiler.eval(form1).await.unwrap();

    // Use variable
    let form2 = CoreForm::PrimOp {
        op: PrimOp::Add,
        args: vec![CoreForm::Var("x".into()), CoreForm::Literal(Literal::Int(1))],
    };
    let result = compiler.eval(form2).await.unwrap();

    assert_eq!(result.value, Some(DisplayValue::Text("43")));
}

#[tokio::test]
async fn test_compilation_error_recovery() {
    let compiler = CachedCompiler::new(SessionId::new()).await.unwrap();

    // Define x
    compiler.eval(parse("(def x 42)")).await.unwrap();

    // This should fail
    let bad = parse("(+ x \"not an int\")");
    let result = compiler.eval(bad).await;
    assert!(result.is_err());

    // State should be unchanged
    let good = parse("x");
    let result = compiler.eval(good).await.unwrap();
    assert_eq!(result.value, Some(DisplayValue::Text("42")));
}

#[tokio::test]
async fn test_subprocess_crash_recovery() {
    let compiler = CachedCompiler::new(SessionId::new()).await.unwrap();

    // Force subprocess to crash
    let crash = parse("(panic \"crash\")");
    let result = compiler.eval(crash).await;
    assert!(result.is_err());

    // Next eval should work (subprocess restarted)
    let good = parse("(+ 1 2)");
    let result = compiler.eval(good).await.unwrap();
    assert_eq!(result.value, Some(DisplayValue::Text("3")));
}
```

### Success Criteria

- [ ] Full evaluation pipeline works
- [ ] State management correct
- [ ] Error recovery works
- [ ] Subprocess lifecycle managed
- [ ] Integration with server works
- [ ] Performance targets met
- [ ] All tests pass

---

## Summary

### Implementation Phases

**Phase A: Foundation (Can start now)**

1. VariableStore - 1 day
2. SessionDir - 1 day
3. Cargo Integration - 1-2 days
4. Subprocess Runtime - 2 days

**Total: ~1 week of independent work**

**Phase B: Language Integration (Needs oxur-lang/oxur-comp)**
5. Core Forms Definition - 2-3 weeks (with team)
6. Lowering to Rust AST - 3-4 weeks
7. CodeGenerator - 1-2 weeks

**Total: ~6-9 weeks, requires team collaboration**

**Phase C: Full Integration**
8. Full Evaluation - 2-3 weeks
9. Source Maps - 1 week
10. CachedCompiler - 2-3 weeks

**Total: ~5-7 weeks**

### Critical Path

```
Start
  ↓
Phase A: Foundation (parallel, no blockers)
  ├─ VariableStore ──┐
  ├─ SessionDir ─────┤
  ├─ Cargo ──────────┤
  └─ Subprocess ─────┘
         ↓
    BLOCKED UNTIL:
    Core Forms defined (Phase B.5)
         ↓
    Lowering implemented (Phase B.6)
         ↓
    CodeGenerator (Phase B.7)
         ↓
Phase C: Integration
    Full Evaluation (C.8)
         ↓
    Source Maps (C.9)
         ↓
    CachedCompiler (C.10)
         ↓
    DONE!
```

### Total Timeline Estimate

- **Optimistic:** 12-15 weeks
- **Realistic:** 15-20 weeks
- **Conservative:** 20-25 weeks

**Critical dependency:** Core Forms and Lowering from oxur-lang/oxur-comp teams

---

## Next Steps

1. **Immediate:** Start Phase A (Foundation)
   - Create tasks for VariableStore, SessionDir, Cargo, Subprocess
   - Can work in parallel with oxur-lang team

2. **Parallel:** Design Core Forms
   - Work with oxur-lang team
   - Create ODD-0032 for Core Forms specification
   - Define minimal set for REPL v1.0

3. **Plan:** Lowering strategy
   - Create ODD-0033 for Lowering semantics
   - Work with oxur-comp team
   - Prototype with minimal Core Forms

4. **Schedule:** Regular sync meetings
   - Coordinate between oxur-repl, oxur-lang, oxur-comp
   - Review progress on dependencies
   - Adjust timeline as needed

---

**End of Proto-Plans Document**
