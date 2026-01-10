# Treebeard Architectural Research: A Tree-Walking Interpreter for Rust `syn` AST

**Treebeard is a general-purpose tree-walking interpreter for Rust's `syn` AST with compilation escape hatches.** By interpreting `syn` directly, Treebeard becomes infrastructure that any language compiling to Rust can leverage—not just Oxur. This document details the layered architecture separating the general-purpose interpreter (Treebeard) from language-specific frontends (like oxur-vm).

---

## The Layered Architecture

### Core Insight

`syn` is already the Rust ecosystem's canonical AST representation. By building an interpreter for `syn::Expr`, `syn::Stmt`, `syn::Item`, etc., Treebeard becomes useful to:

- **DSL authors** building embedded languages that compile to Rust
- **Macro developers** wanting to test proc-macro output interactively
- **Language experimenters** prototyping Rust-like languages
- **Educational tools** teaching Rust semantics interactively
- **Oxur** (and similar Lisp-to-Rust projects)

### Repository Structure

```
treebeard/                          # Standalone repository
├── treebeard-core/                 # The interpreter engine
│   ├── src/
│   │   ├── eval/                   # syn AST evaluation
│   │   ├── env/                    # Environment/bindings
│   │   ├── ownership/              # Runtime ownership tracking
│   │   ├── types/                  # Runtime type representation
│   │   ├── compile/                # Compilation escape hatch
│   │   └── loader/                 # Dynamic crate loading
│   └── Cargo.toml
├── treebeard-repl/                 # Generic REPL infrastructure
│   ├── src/
│   │   ├── session.rs              # Session management
│   │   ├── protocol.rs             # Wire protocol (nREPL-compatible)
│   │   └── middleware.rs           # Extensible middleware
│   └── Cargo.toml
└── treebeard-interface/            # FFI types for compiled code
    └── ...

oxur/                               # Separate repository
├── oxur-reader/                    # S-expression → syn AST
├── oxur-macros/                    # Lisp macro system
├── oxur-vm/                        # Thin layer over treebeard
│   ├── src/
│   │   ├── lib.rs                  # Integrates reader + macros + treebeard
│   │   └── repl.rs                 # Oxur-specific REPL features
│   └── Cargo.toml                  # depends on treebeard-core, treebeard-repl
└── oxur-compiler/                  # Full compilation to .rs files
```

### The Abstraction Boundary

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        LANGUAGE FRONTENDS                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   oxur-vm   │  │  rust-repl  │  │  dsl-foo    │  │  edu-rust   │    │
│  │ (S-expr →   │  │ (Rust src → │  │ (Custom →   │  │ (Simplified │    │
│  │  syn AST)   │  │  syn AST)   │  │  syn AST)   │  │  syn AST)   │    │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘    │
│         │                │                │                │            │
│         └────────────────┴────────────────┴────────────────┘            │
│                                   │                                      │
│                        ┌──────────▼──────────┐                          │
│                        │   Frontend Trait    │                          │
│                        │  ─────────────────  │                          │
│                        │  parse() → syn AST  │                          │
│                        │  expand_macros()    │                          │
│                        │  format_error()     │                          │
│                        └──────────┬──────────┘                          │
└───────────────────────────────────┼─────────────────────────────────────┘
                                    │
════════════════════════════════════╪══════════════════════════════════════
                    ABSTRACTION BOUNDARY (treebeard crate API)
════════════════════════════════════╪══════════════════════════════════════
                                    │
┌───────────────────────────────────┼─────────────────────────────────────┐
│                              TREEBEARD                                   │
│                        ┌──────────▼──────────┐                          │
│                        │   treebeard-core    │                          │
│                        │  ─────────────────  │                          │
│                        │  eval(syn::Expr)    │                          │
│                        │  Environment        │                          │
│                        │  OwnershipTracker   │                          │
│                        │  CompileEscapeHatch │                          │
│                        └──────────┬──────────┘                          │
│                                   │                                      │
│         ┌─────────────────────────┼─────────────────────────┐           │
│         │                         │                         │           │
│  ┌──────▼──────┐          ┌───────▼───────┐         ┌───────▼───────┐  │
│  │treebeard-   │          │ treebeard-    │         │ treebeard-    │  │
│  │repl         │          │ interface     │         │ loader        │  │
│  │─────────────│          │───────────────│         │───────────────│  │
│  │Session mgmt │          │FFI types      │         │libloading     │  │
│  │Protocol     │          │Value repr     │         │Cargo invoke   │  │
│  │Middleware   │          │ABI-stable     │         │Symbol resolve │  │
│  └─────────────┘          └───────────────┘         └───────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## What Lives Where: The Split in Detail

### Treebeard (general-purpose, reusable)

#### treebeard-core

**Evaluator for `syn` AST nodes:**
```rust
pub trait Evaluate {
    fn eval(&self, env: &mut Environment, ctx: &EvalContext) -> Result<Value, EvalError>;
}

impl Evaluate for syn::Expr { ... }
impl Evaluate for syn::Stmt { ... }
impl Evaluate for syn::Item { ... }
// etc.
```

**Environment and bindings:**
```rust
pub struct Environment {
    frames: Vec<Frame>,
    globals: Frame,
}

pub struct Frame {
    bindings: HashMap<Ident, Binding>,
}

pub struct Binding {
    pub value: Value,
    pub mode: BindingMode,           // Let, LetMut, Static, Const
    pub ownership: OwnershipState,   // Owned, Moved, Borrowed { ... }
    pub type_info: Option<TypeInfo>, // Runtime type metadata
}
```

**Runtime value representation:**
```rust
pub enum Value {
    // Primitives
    Unit,
    Bool(bool),
    Char(char),
    Integer(IntValue),    // Handles i8..i128, u8..u128
    Float(FloatValue),    // f32, f64
    
    // Compound
    String(TrackedString),
    Tuple(Vec<Value>),
    Array(TrackedArray),
    Struct(TrackedStruct),
    Enum(TrackedEnum),
    
    // Callable
    Closure(Closure),
    CompiledFn(CompiledFn),
    BuiltinFn(BuiltinFn),
    
    // References (with ownership tracking)
    Ref(TrackedRef),
    RefMut(TrackedRefMut),
}
```

**Ownership tracking:**
```rust
pub struct OwnershipTracker {
    // Per-value state
    states: HashMap<ValueId, OwnershipState>,
    // Scope-based borrow invalidation
    scope_stack: Vec<ScopeId>,
    active_borrows: HashMap<ScopeId, Vec<BorrowId>>,
}

pub enum OwnershipState {
    Owned,
    Moved { moved_at: Span },
    Borrowed { shared: u32, mutable: bool },
}
```

**Compilation escape hatch:**
```rust
pub struct CompileEscapeHatch {
    pub fn compile(&self, item: &syn::ItemFn) -> Result<CompiledFn, CompileError>;
    pub fn is_hot(&self, fn_id: FnId) -> bool;
    pub fn invalidate(&mut self, fn_id: FnId);
}
```

**Evaluation context (configuration):**
```rust
pub struct EvalContext {
    // Ownership checking level
    pub ownership_mode: OwnershipMode,  // Strict, Permissive, Off
    
    // Compilation settings
    pub auto_compile_threshold: Option<u32>,  // None = manual only
    
    // Hooks for language frontends
    pub on_undefined_ident: Option<Box<dyn Fn(&Ident) -> Option<Value>>>,
    pub on_call: Option<Box<dyn Fn(&Ident, &[Value]) -> Option<Result<Value, EvalError>>>>,
    
    // Interruption
    pub interrupt_flag: Arc<AtomicBool>,
}
```

#### treebeard-repl

**Session management:**
```rust
pub struct Session {
    pub id: SessionId,
    pub env: Environment,
    pub history: VecDeque<HistoryEntry>,  // *1, *2, *3
    pub last_error: Option<EvalError>,     // *e
    pub current_module: ModulePath,
}

pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
    pub fn create(&mut self) -> SessionId;
    pub fn clone_session(&mut self, from: SessionId) -> SessionId;
    pub fn eval(&mut self, session: SessionId, input: EvalInput) -> EvalOutput;
}
```

**Protocol layer (nREPL-compatible):**
```rust
pub trait ReplProtocol {
    fn handle_message(&mut self, msg: Message) -> Response;
}

pub struct NReplProtocol {
    session_manager: SessionManager,
    middleware: Vec<Box<dyn Middleware>>,
}
```

**Middleware for extensibility:**
```rust
pub trait Middleware {
    fn handle(&self, msg: &Message, next: &dyn Fn(&Message) -> Response) -> Response;
}

// Built-in middleware
pub struct PrintMiddleware;      // Pretty-print results
pub struct CaughtMiddleware;     // Error handling
pub struct InterruptMiddleware;  // Ctrl+C handling
```

#### treebeard-interface

**FFI-safe types for compiled code:**
```rust
#[repr(C)]
pub struct FfiValue {
    tag: u8,
    data: FfiValueData,
}

#[repr(C)]
pub union FfiValueData {
    unit: (),
    boolean: bool,
    integer: i64,
    float: f64,
    string: FfiString,
    // ...
}

// Using abi_stable for complex types
use abi_stable::std_types::{RVec, RString, RBox};
```

#### treebeard-loader

**Dynamic crate loading:**
```rust
pub struct CrateLoader {
    cache_dir: PathBuf,
    loaded: HashMap<CrateName, LoadedCrate>,
}

impl CrateLoader {
    pub fn load(&mut self, spec: &CrateSpec) -> Result<LoadedCrate, LoadError>;
    pub fn call(&self, crate_name: &str, fn_name: &str, args: &[Value]) -> Result<Value, CallError>;
}
```

---

### Language Frontends (language-specific)

#### The Frontend Trait

```rust
/// Trait that language frontends implement to integrate with Treebeard
pub trait LanguageFrontend {
    /// Parse source text into syn AST
    fn parse(&self, source: &str) -> Result<Vec<syn::Item>, ParseError>;
    
    /// Expand any language-specific macros (returns syn AST)
    fn expand_macros(&self, items: Vec<syn::Item>, env: &Environment) 
        -> Result<Vec<syn::Item>, MacroError>;
    
    /// Format errors for display (language-specific source locations)
    fn format_error(&self, error: &EvalError, source: &str) -> String;
    
    /// Language name for REPL prompts, etc.
    fn name(&self) -> &str;
    
    /// File extension
    fn extension(&self) -> &str;
    
    /// Optional: custom REPL commands
    fn repl_commands(&self) -> Vec<ReplCommand> { vec![] }
}
```

#### oxur-vm Implementation

```rust
// oxur/oxur-vm/src/lib.rs

use treebeard_core::{Environment, EvalContext, Value};
use treebeard_repl::{Session, ReplProtocol};
use oxur_reader::Reader;
use oxur_macros::MacroExpander;

pub struct OxurFrontend {
    reader: Reader,
    macro_expander: MacroExpander,
}

impl LanguageFrontend for OxurFrontend {
    fn parse(&self, source: &str) -> Result<Vec<syn::Item>, ParseError> {
        // S-expression → syn AST
        let sexpr = self.reader.read(source)?;
        oxur_reader::sexpr_to_syn(sexpr)
    }
    
    fn expand_macros(&self, items: Vec<syn::Item>, env: &Environment) 
        -> Result<Vec<syn::Item>, MacroError> 
    {
        // Oxur's defmacro system operates here
        self.macro_expander.expand(items, env)
    }
    
    fn format_error(&self, error: &EvalError, source: &str) -> String {
        // Map syn spans back to S-expression positions
        oxur_reader::format_error_with_sexpr_context(error, source)
    }
    
    fn name(&self) -> &str { "Oxur" }
    fn extension(&self) -> &str { "oxur" }
    
    fn repl_commands(&self) -> Vec<ReplCommand> {
        vec![
            ReplCommand::new("macroexpand", "Show macro expansion", cmd_macroexpand),
            ReplCommand::new("defmacro", "Define a macro", cmd_defmacro),
        ]
    }
}
```

#### Other Potential Frontends

**rust-repl** (direct Rust syntax):
```rust
pub struct RustFrontend;

impl LanguageFrontend for RustFrontend {
    fn parse(&self, source: &str) -> Result<Vec<syn::Item>, ParseError> {
        // Just use syn directly!
        syn::parse_str(source).map_err(|e| e.into())
    }
    
    fn expand_macros(&self, items: Vec<syn::Item>, _env: &Environment) 
        -> Result<Vec<syn::Item>, MacroError> 
    {
        // No macro expansion for basic Rust frontend
        // (Could integrate with proc-macro2 for advanced use)
        Ok(items)
    }
    
    fn name(&self) -> &str { "Rust" }
    fn extension(&self) -> &str { "rs" }
}
```

**edu-rust** (simplified Rust for teaching):
```rust
pub struct EduRustFrontend {
    // Restricts to safe subset, provides better error messages
}

impl LanguageFrontend for EduRustFrontend {
    fn parse(&self, source: &str) -> Result<Vec<syn::Item>, ParseError> {
        let items = syn::parse_str(source)?;
        self.validate_safe_subset(&items)?;  // No unsafe, no raw pointers, etc.
        Ok(items)
    }
    // ...
}
```

---

## Research Findings by Area

### 1. Tree-Walking Interpreter Design Patterns

#### Summary of Findings

Tree-walking interpreters prove well-suited for REPL-focused use cases despite their **10-100x performance overhead** compared to bytecode. Research into Boa, Rune, and several Rust-based Lisp interpreters reveals a consistent pattern: while production JavaScript/Rust interpreters have evolved to bytecode compilation for performance, pure tree-walking remains viable within Treebeard's performance budget.

The canonical environment model from SICP—a chain of frames where each frame maps symbols to values with a parent pointer—dominates implementations. Rust interpreters consistently use `Rc<RefCell<Environment>>` for the frame chain, enabling interior mutability needed for closures that capture mutable state. Persistent data structures (rpds, hamt-rs) offer an alternative with ~2x lookup overhead but better support for backtracking and parallel evaluation.

For closures, tree-walking interpreters capture the **entire enclosing environment** at definition time, then extend it with parameter bindings at call time. This differs from bytecode VMs which typically use flat closures with explicit upvalue tracking.

Tail call optimization in tree-walking requires explicit techniques since host language recursion consumes stack. **Trampolining** emerges as the cleanest approach—returning a "continue" value that the main loop iterates rather than recursing.

#### Key Design Decisions with Tradeoffs

| Decision | Option A | Option B | Tradeoff |
|----------|----------|----------|----------|
| Environment structure | HashMap + parent pointer | Indexed environments | Simplicity vs O(1) lookups |
| Mutability | `Rc<RefCell<...>>` | Persistent data structures | Rust-idiomatic vs functional purity |
| Closure capture | Capture environment reference | Flat closure with explicit captures | Simple vs memory-efficient |
| Tail calls | Trampolining | CPS transform | Easy implementation vs full continuations |

#### Key Update for the Split

The evaluator should be generic over AST node types via the `Evaluate` trait:

```rust
// Core evaluation loop in treebeard-core
pub fn eval_items(items: &[syn::Item], env: &mut Environment, ctx: &EvalContext) 
    -> Result<Value, EvalError> 
{
    let mut result = Value::Unit;
    for item in items {
        result = item.eval(env, ctx)?;
    }
    Ok(result)
}

// Language frontends call this with their parsed syn AST
let syn_items = frontend.parse(source)?;
let expanded = frontend.expand_macros(syn_items, &env)?;
let result = treebeard_core::eval_items(&expanded, &mut env, &ctx)?;
```

#### Recommended Approach

Implement `Evaluate` for all `syn` types incrementally:
1. Start with expressions: `syn::Expr` variants (Lit, Binary, Path, Call, etc.)
2. Add statements: `syn::Stmt` variants (Let, Expr, Item)
3. Add items: `syn::ItemFn`, `syn::ItemStruct`, `syn::ItemEnum`
4. Add patterns: `syn::Pat` for match expressions

Use HashMap chains with O(n) lookup. Implement trampolining from the start—retrofitting TCO is painful.

---

### 2. Macro Expansion — The Critical Split Point

#### Summary of Findings

Lisp macro systems divide into two camps: **hygienic** (Racket, Scheme) and **unhygienic with conventions** (Common Lisp, Clojure). Clojure demonstrates a pragmatic middle ground: namespace-qualified symbols via syntax-quote prevent most capture issues, while explicit `gensym` handles the remaining cases.

#### The Critical Insight: Macros are Language-Specific

**Treebeard does NOT handle macro expansion.** It receives already-expanded `syn` AST. This is essential because:

1. **Different frontends have different macro systems:**
   - Oxur: Lisp-style `defmacro` with quasiquote
   - Rust frontend: proc-macro2 integration
   - DSLs: Custom macro systems
   - Educational: No macros at all

2. **Macro expansion is inherently language-specific:**
   - Syntax of macro definitions varies
   - Hygiene strategies vary
   - Expansion timing varies

**The contract:**
```rust
/// Frontend responsibility: expand all macros BEFORE calling Treebeard
/// Treebeard receives "macro-free" syn AST
fn expand_macros(&self, items: Vec<syn::Item>, env: &Environment) 
    -> Result<Vec<syn::Item>, MacroError>;
```

**What Treebeard provides to help frontends:**
```rust
// Access to current bindings (for macro-time evaluation)
impl Environment {
    pub fn lookup(&self, ident: &Ident) -> Option<&Binding>;
    pub fn defined_names(&self) -> impl Iterator<Item = &Ident>;
}

// Ability to evaluate syn AST at macro-expansion time
pub fn eval_at_compile_time(expr: &syn::Expr, env: &Environment) -> Result<Value, EvalError>;
```

---

### 3. BEAM VM Lessons — Recontextualized

#### What Treebeard Learns from BEAM

BEAM provides the most sophisticated hot code loading in production use. Its **two-version system** maintains "current" and "old" code simultaneously. The critical insight: **fully-qualified function calls** switch to new code, while local calls stay in the current version.

1. **Hot code loading pattern** → Treebeard's compilation escape hatch with invalidation
2. **Module as unit of code organization** → Treebeard supports `syn::ItemMod`
3. **Two-version coexistence** → Treebeard's `CompiledFn` with validity flags

#### What Frontends Learn from LFE

LFE demonstrates effective Lisp-on-VM layering: LFE handles syntax, macros, and some pattern matching, while BEAM handles all runtime behavior.

1. **Thin layer over VM** → oxur-vm is thin over treebeard-core
2. **Macro expansion before VM** → oxur-macros runs before treebeard evaluation
3. **REPL inherits VM features** → oxur-vm gets hot reloading "for free"

---

### 4. Hybrid Interpretation/Compilation

#### Summary of Findings

Four systems provide critical patterns:

**Julia** uses a hybrid where simple expressions are interpreted while complex code compiles via LLVM. Julia's REPL makes compilation transparent—users don't distinguish interpreted from compiled.

**LuaJIT** demonstrates trace compilation: hotcounts track iterations, hot paths trigger recording, recorded traces compile to native code with guards for speculation.

**Common Lisp** shows interpreted/compiled duality in language semantics. The `compile` function can compile individual lambdas at runtime.

**GraalVM/Truffle** uses **Assumptions**—objects tracking compilation invariants that trigger deoptimization when invalidated.

#### Treebeard Owns the Compilation Escape Hatch

**Frontends don't need to know about compilation**—they just call `eval()` and Treebeard handles whether to interpret or call compiled code:

```rust
pub struct CompileEscapeHatch {
    rustc_path: PathBuf,
    cache_dir: PathBuf,
    compiled: HashMap<FnId, CompiledFn>,
    call_counts: HashMap<FnId, u32>,
    dependencies: HashMap<FnId, HashSet<FnId>>,
}

impl CompileEscapeHatch {
    pub fn compile(&mut self, item: &syn::ItemFn) -> Result<CompiledFn, CompileError> {
        // 1. Generate Rust source from syn AST (via quote!)
        let rust_source = quote::quote!(#item).to_string();
        
        // 2. Write to temp file with wrapper
        let wrapper = self.generate_cdylib_wrapper(&rust_source)?;
        
        // 3. Invoke cargo
        self.invoke_cargo(&wrapper)?;
        
        // 4. Load resulting .so/.dll
        let lib = self.load_cdylib(&wrapper.output_path)?;
        
        // 5. Extract function pointer
        let fn_ptr = self.extract_fn_ptr(&lib, &item.sig.ident)?;
        
        Ok(CompiledFn { fn_ptr, lib })
    }
}
```

**Calling convention** between interpreted and compiled:
```rust
extern "C" fn compiled_fn(
    env: *mut Environment,
    args: *const Value,
    nargs: usize
) -> Value
```

**Invalidation strategy:**
- Track dependencies: `{compiled_fn_X: [depends_on_a, depends_on_b]}`
- On redefinition of `a`: mark `compiled_fn_X` as stale
- Next call checks validity flag, falls back to interpreter if stale

---

### 5. Runtime Ownership Enforcement — Shared Infrastructure

#### Summary of Findings

**This is Treebeard's most novel contribution—and research reveals it's tractable.**

**Miri's approach**: Every allocation gets a unique ID and per-location borrow stack. Cost: **~1000x slowdown**.

**RefCell's approach**: Two counters per value—shared borrow count and mutable borrow flag. Cost: **~2 integer operations** per borrow.

**Vale's generational references**: Each allocation has a 64-bit generation number, references store expected generation, dereference checks for match.

**The hybrid insight**: Runtime can enforce ownership (moves) and simple borrowing (aliasing violations) cheaply, while complex lifetime analysis is deferred to compilation. This gives **90% of Rust's safety** with **10% of Miri's complexity**.

#### This is Purely Treebeard's Domain

Frontends don't need to implement ownership tracking:

```rust
pub struct OwnershipTracker {
    value_states: HashMap<ValueId, OwnershipState>,
    borrow_stack: Vec<BorrowScope>,
    generation: u64,
}

impl OwnershipTracker {
    pub fn use_value(&mut self, id: ValueId, usage: Usage) -> Result<(), OwnershipError> {
        let state = self.value_states.get(&id).ok_or(OwnershipError::Unknown)?;
        
        match (state, usage) {
            (OwnershipState::Moved { moved_at }, _) => {
                Err(OwnershipError::UseAfterMove { moved_at: *moved_at })
            }
            (OwnershipState::Borrowed { mutable: true, .. }, Usage::Move) => {
                Err(OwnershipError::MoveWhileBorrowed)
            }
            (OwnershipState::Borrowed { shared, .. }, Usage::BorrowMut) if *shared > 0 => {
                Err(OwnershipError::MutBorrowWhileShared)
            }
            (OwnershipState::Borrowed { mutable: true, .. }, Usage::BorrowMut) => {
                Err(OwnershipError::DoubleMutBorrow)
            }
            _ => self.perform_usage(id, usage),
        }
    }
    
    pub fn exit_scope(&mut self) {
        if let Some(scope) = self.borrow_stack.pop() {
            for borrow_id in scope.borrows {
                self.release_borrow(borrow_id);
            }
        }
        self.generation += 1;  // Invalidate dangling refs
    }
}
```

#### Minimum Viable Ownership Model

```rust
Value {
    data: T,
    ownership_state: Owned | Moved | Borrowed { shared: u32, mutable: bool },
    generation: u64,
}
```

**Runtime rules:**
- **Move**: Check not-Moved and not-borrowed, set state to Moved
- **Shared borrow**: Check not-Moved and not-mutably-borrowed, increment shared count
- **Mutable borrow**: Check not-Moved and no borrows exist, set mutable flag
- **End borrow**: Decrement counter or clear flag (via RAII guard)
- **Use**: Check not-Moved, check generation matches for references

**What's deferred to compilation:**
- Lifetime parameters (`&'a T`)
- Complex reborrowing patterns
- Field-level borrowing (runtime checks whole values)
- `'static` guarantees

---

### 6. Environment and Binding — Treebeard Core

The environment is purely Treebeard's concern:

```rust
pub struct Environment {
    frames: Vec<Frame>,
    globals: Frame,
    modules: HashMap<ModulePath, Module>,
}

pub struct Binding {
    pub value: Value,
    pub mode: BindingMode,
    pub value_id: ValueId,
    pub type_info: Option<syn::Type>,
}

#[derive(Clone, Copy)]
pub enum BindingMode {
    Let,       // Immutable binding
    LetMut,    // Mutable binding
    Static,    // Static variable
    Const,     // Compile-time constant
}
```

**Frontends can query but not directly modify:**

```rust
impl Environment {
    pub fn lookup(&self, ident: &Ident) -> Option<&Binding>;
    pub fn is_defined(&self, ident: &Ident) -> bool;
    pub fn current_module(&self) -> &ModulePath;
    
    pub(crate) fn bind(&mut self, ident: Ident, binding: Binding);
    pub(crate) fn push_scope(&mut self);
    pub(crate) fn pop_scope(&mut self);
}
```

---

### 7. REPL State Management — Split Between Layers

#### treebeard-repl Provides

- Session management (creation, cloning, destruction)
- Generic protocol handling (nREPL ops)
- Middleware infrastructure
- History tracking (*1, *2, *3)
- Error handling (*e)
- Interruption

#### Frontends Provide

- Custom REPL commands
- Language-specific formatting
- Source mapping for errors
- Custom prompts

```rust
pub struct Session {
    pub id: SessionId,
    pub env: Environment,
    pub history: History,
    pub last_error: Option<EvalError>,
    pub frontend_state: Box<dyn Any>,  // Frontend-specific state
}

// Oxur example
pub struct OxurSession {
    pub macros: MacroExpander,
    pub reader_state: ReaderState,
}
```

---

### 8. Calling External Code — Treebeard Infrastructure

The crate loader is entirely Treebeard's responsibility:

```rust
pub struct CrateLoader {
    cache_dir: PathBuf,
    loaded: HashMap<CrateName, LoadedCrate>,
    functions: HashMap<(CrateName, FnName), FnPtr>,
}

impl CrateLoader {
    pub fn require(&mut self, spec: CrateSpec) -> Result<(), LoadError> {
        if self.loaded.contains_key(&spec.name) {
            return Ok(());
        }
        
        let wrapper = self.generate_wrapper(&spec)?;
        self.build_wrapper(&wrapper)?;
        let lib = self.load_cdylib(&wrapper.output_path)?;
        
        let init: Symbol<fn() -> &'static FunctionTable> = lib.get(b"treebeard_init")?;
        let table = init();
        
        for func in table.functions() {
            self.functions.insert((spec.name.clone(), func.name.clone()), func.ptr);
        }
        
        self.loaded.insert(spec.name, LoadedCrate { lib, table });
        Ok(())
    }
}
```

**Frontends just use the loader through Environment:**

```rust
fn handle_require(&self, crate_name: &str, env: &mut Environment) -> Result<(), Error> {
    env.loader().require(CrateSpec::from_name(crate_name))?;
    Ok(())
}
```

---

## Public API Surface

### For Language Frontend Authors

```rust
pub trait LanguageFrontend {
    fn parse(&self, source: &str) -> Result<Vec<syn::Item>, ParseError>;
    fn expand_macros(&self, items: Vec<syn::Item>, env: &Environment) 
        -> Result<Vec<syn::Item>, MacroError>;
    fn format_error(&self, error: &EvalError, source: &str) -> String;
    fn name(&self) -> &str;
    fn extension(&self) -> &str;
    fn repl_commands(&self) -> Vec<ReplCommand> { vec![] }
}

pub fn eval_items(items: &[syn::Item], env: &mut Environment, ctx: &EvalContext) 
    -> Result<Value, EvalError>;

pub struct EvalContext {
    pub ownership_mode: OwnershipMode,
    pub auto_compile_threshold: Option<u32>,
    pub interrupt_flag: Arc<AtomicBool>,
    pub hooks: EvalHooks,
}

pub struct Repl<F: LanguageFrontend> {
    pub fn new(frontend: F) -> Self;
    pub fn run(&mut self) -> Result<(), ReplError>;
    pub fn eval(&mut self, input: &str) -> Result<Value, EvalError>;
}
```

### For Embedding in Applications

```rust
// Minimal embedding
let mut env = Environment::new();
let ctx = EvalContext::default();
let items: Vec<syn::Item> = syn::parse_str("fn main() { 42 }")?;
let result = treebeard::eval_items(&items, &mut env, &ctx)?;

// With a frontend
let oxur = OxurFrontend::new();
let mut repl = Repl::new(oxur);
repl.eval("(defn add [a b] (+ a b))")?;
let result = repl.eval("(add 1 2)")?;
```

---

## Risk Assessment

### High Risk

**1. `syn` AST complexity.** `syn` has hundreds of types. Implementing `Evaluate` for all of them is substantial work.

*Mitigation:* Implement incrementally. Many esoteric Rust features can be "not yet implemented" errors initially.

**2. Semantic fidelity.** Treebeard must match rustc's semantics closely enough.

*Mitigation:* Extensive test suite comparing Treebeard output to rustc. Use Miri as reference.

**3. Ownership tracking performance.** Even with simplified model, per-operation checks accumulate.

*Mitigation*: Tiered checking. "Strict mode" for development; "fast mode" skips inner loop checks.

**4. rustc compilation latency.** Each compile invokes cargo → rustc, typically 1-5 seconds.

*Mitigation*: Compile in background threads. Cache across sessions.

### Medium Risk

**1. Frontend API stability.** The `LanguageFrontend` trait needs to be stable.

*Mitigation:* Version the trait. Use extension traits for optional features.

**2. `syn` version coupling.** Treebeard depends on specific `syn` version.

*Mitigation:* Re-export `syn` from `treebeard-core`. Document supported version range.

### Low Risk

**1. Market size.** "Tree-walking interpreter for syn" is niche.

*Mitigation:* Oxur is the primary customer. External adoption is bonus.

---

## Prototyping Order

### Phase 1: Minimal treebeard-core (3-4 weeks)

**Goal:** Evaluate basic `syn::Expr` and `syn::Stmt` without ownership.

1. Define `Value` enum
2. Define `Environment`
3. Implement `Evaluate` for `syn::ExprLit`, `syn::ExprBinary`, `syn::ExprPath`
4. Implement `Evaluate` for `syn::ExprCall`, `syn::ExprClosure`
5. Implement `Evaluate` for `syn::StmtLocal`, `syn::StmtExpr`
6. Implement `Evaluate` for `syn::ItemFn`
7. Basic REPL loop

**Validation:** Can define and call functions, basic arithmetic, closures work.

### Phase 2: Frontend trait + oxur-vm proof-of-concept (2 weeks)

**Goal:** Prove the split architecture works.

1. Define `LanguageFrontend` trait
2. Implement trivial `RustFrontend`
3. Implement basic `OxurFrontend` (reader, no macros)
4. Run same test suite through both frontends

**Validation:** Same Treebeard core serves two different syntaxes.

### Phase 3: Ownership tracking (2-3 weeks)

**Goal:** Prove minimum viable ownership model.

1. Add `ValueId` to all values
2. Implement `OwnershipTracker` with move detection
3. Add borrow counting
4. Add scope-based borrow invalidation

**Validation:** Correctly rejects double-mutable-borrow, use-after-move.

### Phase 4: Oxur macro system (2 weeks)

**Goal:** Lisp macros work, producing syn AST.

1. `defmacro` registration
2. `gensym` 
3. Quasiquote → syn AST generation
4. Expansion loop with fixed-point detection

**Validation:** Can implement `when`, `cond`, `->` in Oxur.

### Phase 5: treebeard-repl + protocol (2 weeks)

**Goal:** Proper REPL infrastructure.

1. Session management
2. nREPL protocol implementation
3. Middleware
4. Frontend hook integration

**Validation:** Works with existing Oxur editor tooling.

### Phase 6: Compilation escape hatch (3-4 weeks)

**Goal:** Prove hybrid architecture works.

1. Rust codegen (syn AST → Rust source via `quote!`)
2. Cargo invoker
3. Dynamic loader (libloading)
4. Calling convention
5. Manual trigger: `compile(fn)` function

**Validation:** Compiled function produces same results as interpreted, with measurable speedup.

### Phase 7: treebeard-loader (2-3 weeks)

**Goal:** Use real Rust ecosystem from REPL.

1. Wrapper generation
2. Build pipeline
3. Symbol extraction
4. Type bridging

**Validation:** Can `require("regex")` and match patterns from REPL.

---

## Conclusion

The split architecture transforms Treebeard from "Oxur's interpreter" into "the interpreter for Rust-targeting languages." This is a better product:

1. **Broader audience:** Anyone building a Rust DSL, educational tool, or compile-to-Rust language
2. **Cleaner separation:** Syntax/macros (frontend) vs semantics/execution (Treebeard)
3. **Shared infrastructure:** Ownership tracking, compilation, crate loading benefit all frontends
4. **Oxur stays lean:** oxur-vm becomes a thin layer doing only Oxur-specific work

The abstraction boundary at "syn AST" is the key insight — it's already a well-designed, well-documented interface that the Rust ecosystem understands. Treebeard interprets that interface; frontends produce it.

**Total estimated effort:** ~18-22 weeks for full implementation, with useful milestones at phases 2 (proof of split), 3 (ownership), and 5 (usable REPL).
