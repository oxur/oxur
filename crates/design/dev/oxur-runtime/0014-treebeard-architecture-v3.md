# Treebeard Implementation Guide v3

**A Tree-Walking Interpreter for Rust's `syn` AST**

**Date:** 2026-01-10  
**Version:** 3.0  
**Status:** Final Architecture Specification

---

## Executive Summary

**What is Treebeard?**

Treebeard is a tree-walking interpreter for Rust's `syn` AST—a general-purpose execution engine that any language compiling to Rust can leverage. It provides immediate execution of Rust code without compilation, with an escape hatch to `rustc` for performance-critical paths.

**Why this architecture?**

The research validates the "thin layer" principle: LFE succeeds by doing ONE thing well (syntax transformation) and delegating everything else to BEAM. Treebeard follows the same pattern—interpret `syn` AST, delegate type checking and optimization to `rustc`. This keeps the codebase under 15K lines while achieving 100% Rust interoperability.

**What's the critical path?**

Given Oxur's current state (95% AST Bridge, 60% REPL, 25% Evaluation, 0% Macros):

1. **Phase 1:** Core evaluator for `syn` AST (builds on 95% AST Bridge)
2. **Phase 2:** Oxur macro system (the 0% → 100% gap)
3. **Phase 3:** REPL integration (leverages 60% infrastructure)
4. **Phase 4:** Compilation escape hatch (performance path)

**What are the biggest risks?**

1. **`syn` AST complexity** — hundreds of types to implement (mitigated by incremental approach)
2. **Ownership tracking performance** — per-operation checks accumulate (mitigated by tiered checking)
3. **`rustc` compilation latency** — 1-5 seconds per compile (mitigated by caching and background compilation)

**What's the timeline?**

~16-20 weeks to production-ready system, with useful milestones at weeks 4 (basic evaluation), 8 (macros), 12 (REPL), and 16 (compilation).

---

## Part 1: Architecture Validation

### 1.1 Is the `LanguageFrontend` Trait the Right Abstraction Boundary?

**Verdict: YES, with modifications.**

The proposed trait:

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
```

**Evidence supporting this design:**

1. **LFE's pattern:** LFE separates syntax parsing (`lfe_scan`, `lfe_parse`) from macro expansion (`lfe_macro`) from code generation (`lfe_codegen`). The `LanguageFrontend` trait captures this same separation.

2. **Rhai's approach:** Rhai uses a similar abstraction for embedding—the `Engine` type provides parse + eval, allowing custom syntax extensions via registered functions.

3. **Elixir's architecture:** Elixir separates parsing (`elixir_parser`) from expansion (`elixir_expand`) from compilation, with well-defined boundaries.

**Recommended modifications:**

```rust
pub trait LanguageFrontend {
    /// Parse source into syn AST items
    fn parse(&self, source: &str) -> Result<Vec<syn::Item>, ParseError>;
    
    /// Expand macros in context of environment
    /// Returns (expanded_items, updated_macro_env)
    fn expand_macros(
        &self, 
        items: Vec<syn::Item>, 
        macro_env: &MacroEnvironment
    ) -> Result<(Vec<syn::Item>, MacroEnvironment), MacroError>;
    
    /// Format an evaluation error for display
    fn format_error(&self, error: &EvalError, source: &str) -> String;
    
    /// Format a value for REPL display
    fn format_value(&self, value: &Value, depth: usize) -> String;
    
    /// Language metadata
    fn name(&self) -> &str;
    fn file_extension(&self) -> &str;
    
    /// Custom REPL commands
    fn repl_commands(&self) -> Vec<ReplCommand> { vec![] }
    
    /// Syntax highlighting hints (for REPL)
    fn syntax_categories(&self) -> SyntaxCategories { SyntaxCategories::default() }
}
```

**Key change:** `expand_macros` now takes and returns a `MacroEnvironment`, following LFE's pattern where macros can define more macros. This supports incremental macro definition in the REPL.

### 1.2 Is `syn` AST the Right Intermediate Representation?

**Verdict: YES, with caveats.**

**Evidence for `syn` AST:**

1. **Ecosystem standard:** `syn` is the canonical Rust AST, used by all proc-macro authors. Over 20,000 crates depend on it.

2. **Well-documented:** Comprehensive documentation with examples for every node type.

3. **Round-trip capable:** Oxur's AST Bridge is 95% complete with verified round-trip (Rust → S-exp → Rust).

4. **Direct compilation path:** `syn` AST can be converted to `TokenStream` via `quote!`, enabling direct `rustc` compilation.

**Caveats and mitigations:**

| Caveat | Impact | Mitigation |
|--------|--------|------------|
| `syn` has hundreds of types | Large implementation surface | Implement incrementally; "not yet implemented" errors for esoteric features |
| `syn` types are large | Memory overhead | Use `Arc<syn::Expr>` for sharing; implement custom compact representation for hot paths |
| `syn` doesn't track semantic info | Need separate type info | Maintain parallel `TypeInfo` map keyed by node ID |
| `syn` version coupling | Breaking changes possible | Re-export `syn` from `treebeard-core`; pin to specific version |

**Alternative considered: Custom IR**

```
S-expressions → Custom IR → Evaluation
                    ↓
                syn AST → rustc
```

**Rejected because:**
- Adds translation layer (more code, more bugs)
- Duplicates `syn`'s functionality
- Complicates compilation path (IR → syn → tokens)
- LFE doesn't do this—it goes directly to Erlang AST

### 1.3 How Does Oxur's Existing 95% AST Bridge Fit?

**The AST Bridge is the foundation.**

Current Oxur AST Bridge capabilities:
- ✅ S-expression lexer (100%)
- ✅ S-expression parser (100%)
- ✅ S-expression printer (100%)
- ✅ Items S-exp → Rust (100%)
- ✅ Expressions S-exp → Rust (100%)
- ✅ Round-trip verification (95%)
- ✅ CLI tools (100%)

**Integration architecture:**

```
┌──────────────────────────────────────────────────────────────┐
│                     Oxur (oxur-vm)                            │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │  oxur-reader   │→│  oxur-macros   │→│  ast-bridge    │  │
│  │  (S-exp parse) │  │  (expand)      │  │  (95% done!)   │  │
│  └────────────────┘  └────────────────┘  └───────┬────────┘ │
│                                                   │          │
│           Implements LanguageFrontend trait       │          │
└───────────────────────────────────────────────────┼──────────┘
                                                    │
════════════════════════════════════════════════════╪══════════
                        syn AST boundary            │
════════════════════════════════════════════════════╪══════════
                                                    │
┌───────────────────────────────────────────────────┼──────────┐
│                    Treebeard                       │          │
│  ┌────────────────┐  ┌────────────────┐  ┌───────▼────────┐ │
│  │ treebeard-core │  │ treebeard-repl │  │ treebeard-     │ │
│  │  (evaluator)   │  │  (session mgmt)│  │ interface      │ │
│  └────────────────┘  └────────────────┘  └────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

**What changes in the AST Bridge:**

1. **Nothing major** — the bridge already produces `syn::Item`, `syn::Expr`, etc.
2. **Add node IDs** — for ownership tracking and error reporting
3. **Add span mapping** — bridge currently tracks this, need to expose it

---

## Part 2: Critical Path Analysis

### 2.1 Current Oxur Status Assessment

| Category | Status | Implication for Treebeard |
|----------|--------|---------------------------|
| **Rust AST Bridge** | 95% | Foundation is ready—Treebeard can use it directly |
| **REPL Infrastructure** | 60% | Session management, multi-line input, completion exist |
| **Core Evaluation** | 25% | `deffn` works; needs replacement with proper evaluator |
| **Macro System** | 0% | Must be built from scratch—critical gap |
| **Functions & Closures** | 20% | Named functions only; closures needed |

### 2.2 Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         DEPENDENCY GRAPH                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────┐                                                    │
│  │  syn AST        │ ← Already exists (Rust ecosystem)                  │
│  │  (foundation)   │                                                    │
│  └────────┬────────┘                                                    │
│           │                                                              │
│           ▼                                                              │
│  ┌─────────────────┐                                                    │
│  │  AST Bridge     │ ← Already 95% (Oxur)                               │
│  │  (S-exp ↔ syn) │                                                    │
│  └────────┬────────┘                                                    │
│           │                                                              │
│           ▼                                                              │
│  ┌─────────────────────────────────────────────────────┐               │
│  │              PHASE 1: Core Evaluator                 │               │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐       │               │
│  │  │   Value   │  │Environment│  │ Evaluator │       │               │
│  │  │   repr    │→│  bindings │→│  syn::*   │       │               │
│  │  └───────────┘  └───────────┘  └───────────┘       │               │
│  └──────────────────────────┬──────────────────────────┘               │
│                             │                                           │
│           ┌─────────────────┼─────────────────┐                        │
│           │                 │                 │                         │
│           ▼                 ▼                 ▼                         │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                  │
│  │  PHASE 2:   │   │  PHASE 3:   │   │  PHASE 4:   │                  │
│  │   Macros    │   │    REPL     │   │  Compiler   │                  │
│  │  (Oxur-     │   │(Integration)│   │ (Escape     │                  │
│  │  specific)  │   │             │   │  hatch)     │                  │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘                  │
│         │                 │                 │                          │
│         └─────────────────┴─────────────────┘                          │
│                           │                                             │
│                           ▼                                             │
│                  ┌─────────────────┐                                   │
│                  │   PHASE 5:      │                                   │
│                  │   Ownership     │                                   │
│                  │   (Optional)    │                                   │
│                  └─────────────────┘                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Minimum Viable Treebeard (MVT)

**The smallest Treebeard that unblocks meaningful progress:**

```rust
// MVT: Can evaluate simple expressions
let env = Environment::new();
let ctx = EvalContext::default();

// Parse Oxur → syn AST (via existing bridge)
let items: Vec<syn::Item> = oxur_bridge::parse("(deffn add [a b] (+ a b))")?;

// Evaluate with Treebeard
let result = treebeard::eval_items(&items, &mut env, &ctx)?;

// Call defined function
let call = syn::parse_quote! { add(1, 2) };
let value = treebeard::eval_expr(&call, &env, &ctx)?;
assert_eq!(value, Value::Integer(3));
```

**MVT requirements:**
1. Value representation (primitives + functions)
2. Environment (bindings + scopes)
3. Evaluate: `syn::ExprLit`, `syn::ExprBinary`, `syn::ExprPath`, `syn::ExprCall`
4. Evaluate: `syn::StmtLocal`, `syn::ItemFn`
5. Error handling with source positions

**MVT does NOT need:**
- Ownership tracking (can add later)
- Closures (named functions sufficient initially)
- Macros (frontend responsibility)
- Compilation escape hatch (interpreter-only initially)

**Estimated effort:** 3-4 weeks

---

## Part 3: Design Decisions Matrix

| Decision | Options | Recommendation | Rationale | Evidence |
|----------|---------|----------------|-----------|----------|
| **Environment representation** | HashMap chain vs indexed vs persistent | **Flat scope with frame boundaries** | Simple O(n) lookup acceptable for REPL; index caching in AST for hot paths | Rhai achieves 2x Python with this approach; fits 50K line budget |
| **Value boxing** | Enum vs tagged pointer vs NaN boxing | **Three-tier enum** (Inline/Heap/Native) | Balances simplicity with performance; 16-byte Value size acceptable | Rune uses this pattern; Book's tagged pointers too complex |
| **Ownership tracking** | Full Miri vs RefCell-style vs none | **Minimal per-value tags** (10 bytes) | Catches use-after-move and double-borrow without Miri's complexity | Simplified Stacked Borrows; can add complexity later |
| **Closure capture** | Environment reference vs flat capture | **Explicit upvalues** with share-on-capture | Rust semantics require knowing what's captured; upvalues enable optimization | Book's Lua-style + Rhai's automatic detection |
| **TCO mechanism** | Trampolining vs CPS vs none | **Trampolining** for self-tail-calls | Essential for Lisp recursion patterns; CPS too invasive | Standard technique; ~200 LOC |
| **Compilation trigger** | Manual vs hotcount vs never | **Manual initially**, hotcount later | Complexity of profiling not worth it initially | Start simple; add profiling when needed |
| **Function lookup** | Early-bound vs late-bound | **Late-bound** (always lookup) | Enables trivial hot reload; tree-walker advantage over BEAM | LFE analysis: late binding is simpler AND better |
| **Macro timing** | Eager vs lazy expansion | **Eager expansion** before evaluation | LFE pattern: expand all macros, then evaluate/compile | `lfe_macro.erl` always expands before `lfe_eval.erl` |

### 3.1 Environment Representation Details

**Recommended: Flat scope with frame boundaries**

```rust
#[derive(Clone)]
pub struct Environment {
    /// Variable bindings (flat array for cache-friendly access)
    bindings: Vec<Binding>,
    
    /// Frame boundaries (indices into bindings)
    frames: Vec<usize>,
    
    /// Current module context
    current_module: Option<ModulePath>,
}

#[derive(Clone)]
pub struct Binding {
    pub name: Ident,
    pub value: Value,
    pub mode: BindingMode,
    pub source_span: Option<Span>,
}

#[derive(Clone, Copy)]
pub enum BindingMode {
    Let,      // Immutable
    LetMut,   // Mutable
    Const,    // Compile-time constant
    Static,   // Global static
}
```

**Lookup algorithm (reverse search with frame boundaries):**

```rust
impl Environment {
    pub fn lookup(&self, name: &Ident) -> Option<&Value> {
        // Search backwards (most recent binding first)
        for binding in self.bindings.iter().rev() {
            if &binding.name == name {
                return Some(&binding.value);
            }
        }
        None
    }
    
    pub fn push_frame(&mut self) {
        self.frames.push(self.bindings.len());
    }
    
    pub fn pop_frame(&mut self) {
        if let Some(boundary) = self.frames.pop() {
            self.bindings.truncate(boundary);
        }
    }
}
```

**Evidence:** Rhai uses exactly this pattern (`src/types/scope.rs:62-74`) and achieves good performance for scripting use cases.

### 3.2 Value Representation Details

**Recommended: Three-tier enum**

```rust
pub enum Value {
    // Tier 1: Inline primitives (no allocation)
    Unit,
    Bool(bool),
    Char(char),
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128), Isize(isize),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128), Usize(usize),
    F32(f32), F64(f64),
    
    // Tier 2: Heap-allocated Rust types (Arc-wrapped)
    String(Arc<String>),
    Vec(Arc<Vec<Value>>),
    HashMap(Arc<HashMap<Value, Value>>),
    Tuple(Arc<Vec<Value>>),
    Struct(Arc<StructValue>),
    Enum(Arc<EnumValue>),
    
    // Tier 3: Callable
    Closure(Arc<Closure>),
    Function(Arc<FunctionDef>),
    BuiltinFn(BuiltinFn),
    CompiledFn(CompiledFn),
    
    // References (for ownership tracking)
    Ref(ValueRef),
    RefMut(ValueRefMut),
}

pub struct StructValue {
    pub type_name: Ident,
    pub fields: HashMap<Ident, Value>,
}

pub struct Closure {
    pub params: Vec<Ident>,
    pub body: Arc<syn::Block>,
    pub captured: Vec<Upvalue>,
}
```

**Size analysis:**
- Enum discriminant: 1 byte
- Largest inline variant: `I128`/`U128` at 16 bytes
- Total Value size: ~24 bytes (with padding)

**Evidence:** Rune uses a similar three-tier approach (`crates/rune/src/runtime/value.rs:67-88`), achieving good balance between performance and flexibility.

### 3.3 Ownership Tracking Details

**Recommended: Minimal per-value tags (opt-in)**

```rust
pub struct OwnershipTracker {
    /// Next tag to allocate
    next_tag: u32,
    
    /// Per-value ownership state
    states: HashMap<ValueId, OwnershipState>,
    
    /// Active protectors (function arguments)
    protectors: Vec<Protector>,
}

#[derive(Clone, Copy)]
pub struct OwnershipState {
    pub tag: u32,                    // 4 bytes
    pub permission: Permission,      // 1 byte
    pub protected: bool,             // 1 byte
}
// Total: 6 bytes (8 with padding)

#[derive(Clone, Copy)]
pub enum Permission {
    Unique,      // Owned or &mut
    SharedRW,    // &mut (reborrowed)
    SharedRO,    // & (shared reference)
    Disabled,    // Moved or dropped
}
```

**Checking algorithm (simplified Stacked Borrows):**

```rust
impl OwnershipTracker {
    /// Called when creating a borrow
    pub fn retag(&mut self, value_id: ValueId, kind: BorrowKind) -> Result<u32, OwnershipError> {
        let tag = self.next_tag;
        self.next_tag += 1;
        
        let state = self.states.get_mut(&value_id)
            .ok_or(OwnershipError::UnknownValue)?;
        
        match (kind, state.permission) {
            // Can create shared ref from unique or shared
            (BorrowKind::Shared, Permission::Unique | Permission::SharedRO) => {
                // Don't change permission, just record borrow
                Ok(tag)
            }
            // Can create unique ref only from unique
            (BorrowKind::Unique, Permission::Unique) => {
                state.permission = Permission::SharedRW;
                state.tag = tag;
                Ok(tag)
            }
            // Can't borrow disabled value
            (_, Permission::Disabled) => {
                Err(OwnershipError::UseAfterMove { value_id })
            }
            // Can't create unique from shared
            (BorrowKind::Unique, Permission::SharedRO | Permission::SharedRW) => {
                Err(OwnershipError::BorrowConflict { value_id })
            }
        }
    }
    
    /// Called when accessing through a reference
    pub fn check_access(&self, value_id: ValueId, tag: u32, write: bool) -> Result<(), OwnershipError> {
        let state = self.states.get(&value_id)
            .ok_or(OwnershipError::UnknownValue)?;
        
        if state.permission == Permission::Disabled {
            return Err(OwnershipError::UseAfterMove { value_id });
        }
        
        if write && state.permission == Permission::SharedRO {
            return Err(OwnershipError::WriteToShared { value_id });
        }
        
        Ok(())
    }
}
```

**Evidence:** Miri's full Stacked Borrows (`src/borrow_tracker/stacked_borrows/`) uses 50+ bytes per location. Our simplified model uses 8 bytes per value—sufficient for REPL use cases where we care about catching obvious errors, not full UB detection.

---

## Part 4: Module Specifications

### 4.1 treebeard-core

```rust
// ═══════════════════════════════════════════════════════════════════════
// KEY TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Runtime value representation
pub enum Value { /* see Part 3.2 */ }

/// Variable/function bindings
pub struct Environment { /* see Part 3.1 */ }

/// Ownership state tracking (optional)
pub struct OwnershipTracker { /* see Part 3.3 */ }

/// Evaluation configuration
pub struct EvalContext {
    /// Ownership checking level
    pub ownership_mode: OwnershipMode,
    
    /// Auto-compile threshold (None = manual only)
    pub compile_threshold: Option<u32>,
    
    /// Interrupt flag for long-running evaluation
    pub interrupt: Arc<AtomicBool>,
    
    /// Call depth limit (stack overflow protection)
    pub max_call_depth: usize,
}

#[derive(Clone, Copy)]
pub enum OwnershipMode {
    /// Full checking (catches all violations)
    Strict,
    /// Check only explicit borrows
    Permissive,
    /// No checking (maximum performance)
    Off,
}

// ═══════════════════════════════════════════════════════════════════════
// KEY TRAITS
// ═══════════════════════════════════════════════════════════════════════

/// Evaluation trait for syn AST nodes
pub trait Evaluate {
    fn eval(&self, env: &mut Environment, ctx: &EvalContext) -> Result<Value, EvalError>;
}

// Implementations for all syn expression types
impl Evaluate for syn::Expr { ... }
impl Evaluate for syn::ExprLit { ... }
impl Evaluate for syn::ExprBinary { ... }
impl Evaluate for syn::ExprCall { ... }
impl Evaluate for syn::ExprClosure { ... }
impl Evaluate for syn::ExprIf { ... }
impl Evaluate for syn::ExprMatch { ... }
impl Evaluate for syn::ExprBlock { ... }
// ... (~50 expression types total)

impl Evaluate for syn::Stmt { ... }
impl Evaluate for syn::Item { ... }

/// Conversion traits for Rust interop
pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self, ConversionError>;
}

pub trait ToValue {
    fn to_value(self) -> Value;
}

// ═══════════════════════════════════════════════════════════════════════
// KEY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════

/// Evaluate a sequence of items (top-level forms)
pub fn eval_items(
    items: &[syn::Item], 
    env: &mut Environment, 
    ctx: &EvalContext
) -> Result<Value, EvalError>;

/// Evaluate a single expression
pub fn eval_expr(
    expr: &syn::Expr, 
    env: &mut Environment, 
    ctx: &EvalContext
) -> Result<Value, EvalError>;

/// Register a native Rust function
pub fn register_fn<F, Args, Ret>(
    env: &mut Environment,
    name: &str,
    func: F,
) where
    F: IntoBuiltinFn<Args, Ret>,
    Args: FromValueTuple,
    Ret: ToValue;
```

**Responsibilities:**
- Evaluate `syn` AST nodes to values
- Manage variable/function bindings
- Track ownership state (optional)
- Provide hooks for language frontends

**Non-responsibilities:**
- Parsing (frontend responsibility)
- Macro expansion (frontend responsibility)
- Compilation (treebeard-loader responsibility)
- REPL UI (treebeard-repl responsibility)

**Dependencies:**
- `syn` (AST types)
- `quote` (for compilation path)
- `proc-macro2` (for span handling)

**Dependents:**
- `treebeard-repl`
- `treebeard-loader`
- `oxur-vm`

### 4.2 treebeard-repl

```rust
// ═══════════════════════════════════════════════════════════════════════
// KEY TYPES
// ═══════════════════════════════════════════════════════════════════════

/// REPL session state
pub struct Session {
    pub id: SessionId,
    pub env: Environment,
    pub history: History,
    pub module_registry: Arc<ModuleRegistry>,
    pub state: ReplState,
}

/// Three-environment pattern (from LFE)
pub struct ReplState {
    /// Base environment (prelude, never changes)
    pub base: Environment,
    
    /// Saved environment (snapshot before slurp)
    pub save: Option<Environment>,
    
    /// Current working environment
    pub curr: Environment,
    
    /// Whether a file is slurped
    pub slurped: bool,
}

/// REPL history tracking
pub struct History {
    /// Previous forms (+, ++, +++)
    pub forms: VecDeque<String>,
    
    /// Previous values (*, **, ***)
    pub values: VecDeque<Value>,
    
    /// Maximum history size
    pub max_size: usize,
}

/// Session manager for multiple concurrent sessions
pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
    next_id: SessionId,
}

// ═══════════════════════════════════════════════════════════════════════
// KEY TRAITS
// ═══════════════════════════════════════════════════════════════════════

/// Language frontend interface
pub trait LanguageFrontend {
    fn parse(&self, source: &str) -> Result<Vec<syn::Item>, ParseError>;
    fn expand_macros(
        &self, 
        items: Vec<syn::Item>, 
        env: &MacroEnvironment
    ) -> Result<(Vec<syn::Item>, MacroEnvironment), MacroError>;
    fn format_error(&self, error: &EvalError, source: &str) -> String;
    fn format_value(&self, value: &Value, depth: usize) -> String;
    fn name(&self) -> &str;
    fn file_extension(&self) -> &str;
    fn repl_commands(&self) -> Vec<ReplCommand> { vec![] }
}

// ═══════════════════════════════════════════════════════════════════════
// KEY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════

/// Create a new REPL with given frontend
pub fn new_repl<F: LanguageFrontend>(frontend: F) -> Repl<F>;

/// Run REPL main loop
impl<F: LanguageFrontend> Repl<F> {
    pub fn run(&mut self) -> Result<(), ReplError>;
    pub fn eval_line(&mut self, input: &str) -> Result<Value, EvalError>;
    pub fn slurp(&mut self, path: &Path) -> Result<String, LoadError>;
    pub fn unslurp(&mut self) -> Result<(), ReplError>;
    pub fn reset(&mut self);
}
```

**Responsibilities:**
- Session lifecycle management
- History tracking (+, *, etc.)
- Slurp/unslurp file loading
- Error recovery (panic-safe evaluation)
- nREPL protocol implementation (optional)

**Non-responsibilities:**
- Parsing (delegated to frontend)
- Evaluation (delegated to treebeard-core)
- UI rendering (delegated to client)

**Dependencies:**
- `treebeard-core`

**Dependents:**
- `oxur-vm` (uses REPL infrastructure)

### 4.3 treebeard-loader

```rust
// ═══════════════════════════════════════════════════════════════════════
// KEY TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Module registry for hot code loading
pub struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, Module>>>,
    versions: HashMap<String, u64>,
}

/// Loaded module representation
pub struct Module {
    pub name: String,
    pub version: u64,
    pub functions: HashMap<(String, usize), FunctionDef>,
    pub types: HashMap<String, TypeDef>,
    pub loaded_at: Instant,
}

/// Crate loader for Rust ecosystem integration
pub struct CrateLoader {
    cache_dir: PathBuf,
    loaded: HashMap<CrateName, LoadedCrate>,
}

/// Compilation escape hatch
pub struct Compiler {
    cache: CompilationCache,
    rustc_path: PathBuf,
}

// ═══════════════════════════════════════════════════════════════════════
// KEY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════

impl ModuleRegistry {
    /// Load or reload a module
    pub fn load_module(&self, module: Module);
    
    /// Look up a function by name and arity
    pub fn get_function(&self, module: &str, name: &str, arity: usize) -> Option<FunctionDef>;
    
    /// Reload module from source file
    pub fn reload_from_file(&self, path: &Path) -> Result<String, LoadError>;
}

impl CrateLoader {
    /// Load a crate from crates.io
    pub fn require(&mut self, spec: CrateSpec) -> Result<(), LoadError>;
    
    /// Get function from loaded crate
    pub fn get_function(&self, crate_name: &str, fn_name: &str) -> Option<CompiledFn>;
}

impl Compiler {
    /// Compile a function to native code
    pub fn compile(&mut self, func: &syn::ItemFn) -> Result<CompiledFn, CompileError>;
    
    /// Check if function should be compiled (hotspot detection)
    pub fn should_compile(&self, fn_id: FnId) -> bool;
}
```

**Responsibilities:**
- Module registry for hot code loading
- Crate loading from Rust ecosystem
- Compilation escape hatch (`rustc` invocation)
- Caching compiled artifacts

**Non-responsibilities:**
- Evaluation (treebeard-core)
- Source parsing (frontend)

**Dependencies:**
- `treebeard-core`
- `libloading` (dynamic library loading)
- `cargo` (for building crates)

### 4.4 treebeard-interface

```rust
// ═══════════════════════════════════════════════════════════════════════
// FFI-SAFE TYPES FOR COMPILED CODE
// ═══════════════════════════════════════════════════════════════════════

/// ABI-stable value representation for FFI boundary
#[repr(C)]
pub struct FfiValue {
    pub tag: u8,
    pub data: FfiValueData,
}

#[repr(C)]
pub union FfiValueData {
    pub unit: (),
    pub boolean: bool,
    pub integer: i64,
    pub float: f64,
    pub pointer: *mut (),
}

/// Function table exported by compiled code
#[repr(C)]
pub struct FunctionTable {
    pub version: u32,
    pub num_functions: u32,
    pub functions: *const FunctionEntry,
}

#[repr(C)]
pub struct FunctionEntry {
    pub name: *const c_char,
    pub arity: u32,
    pub ptr: extern "C" fn(*const FfiValue, u32) -> FfiValue,
}
```

**Responsibilities:**
- ABI-stable types for crossing FFI boundary
- Function table format for compiled libraries
- Conversion between `Value` and `FfiValue`

**Non-responsibilities:**
- Actual compilation (treebeard-loader)
- Evaluation (treebeard-core)

---

## Part 5: Integration Plan with Oxur

### 5.1 Code Reuse Assessment

**What Oxur code can be reused:**

| Component | Status | Reuse Strategy |
|-----------|--------|----------------|
| **S-expression lexer** | 100% | Direct use—no changes needed |
| **S-expression parser** | 100% | Direct use—no changes needed |
| **S-expression printer** | 100% | Direct use—no changes needed |
| **Items S-exp → syn** | 100% | Direct use—this IS the bridge |
| **Expressions S-exp → syn** | 100% | Direct use—this IS the bridge |
| **Round-trip verification** | 95% | Useful for testing |
| **CLI tools (aster)** | 100% | Keep as standalone tool |
| **REPL multi-line input** | 100% | Direct use in treebeard-repl |
| **REPL completion** | 100% | Adapt for Treebeard |
| **REPL server/client** | 90% | Adapt for Treebeard protocol |
| **Session management** | 95% | Adapt for Treebeard |
| **Artifact caching** | 95% | Useful for compilation cache |

**Estimated code reuse: ~8,000 lines** (out of current Oxur codebase)

### 5.2 Code Replacement Assessment

**What Oxur code needs replacement:**

| Component | Status | Replacement |
|-----------|--------|-------------|
| **Core evaluation** | 25% | Replace with `treebeard-core` |
| **Function definitions** | 60% (`deffn`) | Keep syntax, use Treebeard eval |
| **Partial special forms** | 20% (`if`) | Replace with Treebeard |

**Why replace rather than extend:**
- Current evaluation is ad-hoc, not based on `syn` traversal
- Treebeard provides proper environment handling
- Ownership tracking requires fresh implementation

### 5.3 New Code Needed in oxur-vm

```rust
// oxur-vm/src/lib.rs

/// Oxur language frontend for Treebeard
pub struct OxurFrontend {
    /// S-expression parser
    reader: OxurReader,
    
    /// Macro expander (THE NEW CODE)
    macro_expander: MacroExpander,
    
    /// S-exp to syn AST bridge (existing)
    ast_bridge: AstBridge,
}

impl LanguageFrontend for OxurFrontend {
    fn parse(&self, source: &str) -> Result<Vec<syn::Item>, ParseError> {
        // 1. Parse S-expressions (existing code)
        let sexps = self.reader.read(source)?;
        
        // 2. Convert to syn AST (existing 95% bridge)
        let items = self.ast_bridge.sexps_to_items(&sexps)?;
        
        Ok(items)
    }
    
    fn expand_macros(
        &self,
        items: Vec<syn::Item>,
        env: &MacroEnvironment,
    ) -> Result<(Vec<syn::Item>, MacroEnvironment), MacroError> {
        // NEW CODE: Macro expansion
        self.macro_expander.expand_all(items, env)
    }
    
    fn format_error(&self, error: &EvalError, source: &str) -> String {
        // Map syn spans back to S-expression positions
        // Use existing source mapping infrastructure
        self.ast_bridge.format_error(error, source)
    }
    
    fn format_value(&self, value: &Value, depth: usize) -> String {
        // S-expression pretty printer for values
        self.ast_bridge.value_to_sexp(value).pretty_print(depth)
    }
    
    fn name(&self) -> &str { "Oxur" }
    fn file_extension(&self) -> &str { "oxur" }
    
    fn repl_commands(&self) -> Vec<ReplCommand> {
        vec![
            ReplCommand::new("defmacro", "Define a macro", self.handle_defmacro),
            ReplCommand::new("macroexpand", "Expand a macro", self.handle_macroexpand),
            ReplCommand::new("macroexpand-1", "Expand once", self.handle_macroexpand_1),
        ]
    }
}
```

**New code needed:**

| Component | Lines (est.) | Notes |
|-----------|-------------|-------|
| `MacroExpander` | ~1,500 | The 0% → 100% gap |
| `MacroEnvironment` | ~300 | Macro definition storage |
| `OxurFrontend` impl | ~500 | Glue code |
| REPL commands | ~200 | Oxur-specific commands |
| **Total** | ~2,500 | New Oxur-specific code |

### 5.4 Integration Sequence

**Phase 1: Parallel Operation (Week 1-4)**

```
                     Source
                        │
            ┌───────────┴───────────┐
            │                       │
            ▼                       ▼
    ┌───────────────┐       ┌───────────────┐
    │ Existing Oxur │       │   Treebeard   │
    │  Evaluation   │       │   (new)       │
    └───────────────┘       └───────────────┘
            │                       │
            ▼                       ▼
        Result A                Result B
            │                       │
            └───────────┬───────────┘
                        │
                    COMPARE
```

Run both evaluators on same input, verify identical results.

**Phase 2: Gradual Migration (Week 5-8)**

```rust
impl OxurEvaluator {
    fn eval(&self, sexp: &SExp) -> Result<Value, Error> {
        // Convert to syn first
        let expr = self.bridge.sexp_to_expr(sexp)?;
        
        // Try Treebeard
        match treebeard::eval_expr(&expr, &mut self.env, &self.ctx) {
            Ok(value) => Ok(value),
            Err(EvalError::NotImplemented(_)) => {
                // Fall back to old evaluator for unimplemented features
                self.old_eval(sexp)
            }
            Err(e) => Err(e.into()),
        }
    }
}
```

**Phase 3: Full Cutover (Week 9+)**

Remove old evaluator, Treebeard handles everything.

### 5.5 Macro System Design for Oxur

**This is the critical 0% → 100% gap.**

```rust
/// Oxur macro expander (following LFE's pattern)
pub struct MacroExpander {
    /// Core forms that are never expanded
    core_forms: HashSet<Symbol>,
    
    /// Predefined macros (built-in)
    predefined: HashMap<Symbol, PredefinedMacro>,
}

/// Macro environment (separate from runtime environment)
pub struct MacroEnvironment {
    /// User-defined macros
    user_macros: HashMap<Symbol, UserMacro>,
    
    /// Variable counter for gensym
    gensym_counter: u64,
}

/// User-defined macro
pub struct UserMacro {
    pub name: Symbol,
    pub params: Vec<Symbol>,
    pub body: SExp,  // Stored as S-expression, not syn
}

impl MacroExpander {
    /// Expand all macros in items (recursive)
    pub fn expand_all(
        &self,
        items: Vec<syn::Item>,
        env: &MacroEnvironment,
    ) -> Result<(Vec<syn::Item>, MacroEnvironment), MacroError> {
        let mut new_env = env.clone();
        let mut expanded = Vec::new();
        
        for item in items {
            match self.expand_item(&item, &mut new_env)? {
                ExpandResult::Item(i) => expanded.push(i),
                ExpandResult::MacroDef => { /* absorbed into env */ }
                ExpandResult::Multiple(items) => expanded.extend(items),
            }
        }
        
        Ok((expanded, new_env))
    }
    
    /// Expand a single form (from LFE's exp_macro pattern)
    fn expand_form(
        &self,
        form: &SExp,
        env: &MacroEnvironment,
    ) -> Result<Option<SExp>, MacroError> {
        // 1. Never expand core forms
        if let SExp::List(items) = form {
            if let Some(SExp::Symbol(name)) = items.first() {
                if self.core_forms.contains(name) {
                    return Ok(None);
                }
            }
        }
        
        // 2. Check user-defined macros
        if let Some(macro_def) = self.lookup_user_macro(form, env) {
            let expanded = self.apply_macro(macro_def, form, env)?;
            return Ok(Some(expanded));
        }
        
        // 3. Check predefined macros
        if let Some(expanded) = self.expand_predefined(form)? {
            return Ok(Some(expanded));
        }
        
        // 4. Not a macro
        Ok(None)
    }
    
    /// Recursive expansion until fixed point
    fn expand_recursive(
        &self,
        form: &SExp,
        env: &MacroEnvironment,
    ) -> Result<SExp, MacroError> {
        match self.expand_form(form, env)? {
            Some(expanded) => {
                // Recursively expand the result
                self.expand_recursive(&expanded, env)
            }
            None => {
                // Not a macro, expand subforms
                self.expand_subforms(form, env)
            }
        }
    }
}
```

**Predefined macros to implement:**

| Macro | Expansion | Priority |
|-------|-----------|----------|
| `defn` | `(def name (fn [args] body))` | High |
| `defmacro` | Register in environment | High |
| `when` | `(if test (do body) nil)` | High |
| `unless` | `(if test nil (do body))` | High |
| `cond` | Nested `if` | High |
| `->` | Thread-first | Medium |
| `->>` | Thread-last | Medium |
| `let` | `(block (let! x v) body)` | High |
| `and` | Short-circuit `if` | Medium |
| `or` | Short-circuit `if` | Medium |

---

## Part 6: Ownership Model Specification

### 6.1 What Ownership Violations Are Caught at Runtime?

| Violation | Caught? | How? |
|-----------|---------|------|
| **Use after move** | ✅ Yes | Value marked `Disabled` after move; access check fails |
| **Double mutable borrow** | ✅ Yes | Second `&mut` creation fails if permission not `Unique` |
| **Borrow outlives scope** | ✅ Yes | Protectors released on scope exit; protected access fails |
| **Mutable borrow during shared** | ✅ Yes | `&mut` creation fails if permission is `SharedRO` |
| **Write through shared ref** | ✅ Yes | Write check fails if permission is `SharedRO` |

### 6.2 What Is Deferred to Compilation?

| Feature | Deferred? | Reason |
|---------|-----------|--------|
| **Lifetime parameters** | ✅ Yes | Too complex for runtime; rustc handles perfectly |
| **Generic lifetime bounds** | ✅ Yes | Compile-time only concept |
| **NLL (non-lexical lifetimes)** | ✅ Yes | Requires control-flow analysis |
| **Variance** | ✅ Yes | Type system feature |
| **Drop order** | ✅ Yes | Static analysis |

### 6.3 Data Structures

```rust
/// Global ownership tracker
pub struct OwnershipTracker {
    /// Next tag to allocate
    next_tag: u32,
    
    /// Per-value ownership state (keyed by ValueId)
    states: HashMap<ValueId, OwnershipState>,
    
    /// Stack of active protectors
    protector_stack: Vec<ProtectorFrame>,
}

/// Per-value ownership state
#[derive(Clone, Copy, Debug)]
pub struct OwnershipState {
    /// Unique tag for this borrow (4 bytes)
    pub tag: u32,
    
    /// Current permission level (1 byte)
    pub permission: Permission,
    
    /// Is this value protected by a function call? (1 byte)
    pub protected: bool,
}
// Size: 6 bytes (8 with alignment)

/// Permission levels (simplified Stacked Borrows)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    /// Owned or unique mutable borrow
    Unique,
    
    /// Mutable reborrow (can read and write)
    SharedRW,
    
    /// Shared reference (read only)
    SharedRO,
    
    /// Moved or dropped (cannot access)
    Disabled,
}

/// Protector frame (for function arguments)
pub struct ProtectorFrame {
    /// Scope this protector was created in
    scope_id: ScopeId,
    
    /// Protected values
    protected: Vec<(ValueId, u32)>,  // (value_id, tag)
}

/// Unique identifier for values
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ValueId(u64);
```

### 6.4 Performance Budget

| Aspect | Budget | Rationale |
|--------|--------|-----------|
| **Per-operation overhead** | < 100 ns | Hash lookup + enum check |
| **Memory per value** | 8 bytes | OwnershipState size |
| **Acceptable slowdown** | 2-5x | Comparable to RefCell |

**Optimization strategies:**

1. **Opt-in checking:** `OwnershipMode::Off` skips all checks
2. **Fast path:** Most values never borrowed; early exit
3. **Batched release:** Release protectors at frame boundary, not individually

---

## Part 7: Risk Mitigation

| Risk | Likelihood | Impact | Mitigation Strategy | Validation Approach |
|------|------------|--------|---------------------|---------------------|
| **`syn` AST complexity** | High | High | Implement incrementally; "not yet implemented" errors for rare features | Track coverage of `syn` types; target 80% for MVP |
| **Ownership tracking perf** | Medium | Medium | Opt-in checking (`OwnershipMode`); profile and optimize hot paths | Benchmark against RefCell-based code; target < 5x slowdown |
| **rustc compilation latency** | High | Medium | Background compilation; caching; only compile hot functions | Measure latency in CI; target < 2s for cached compiles |
| **Semantic drift** | Medium | High | Comprehensive test suite comparing Treebeard to rustc output | Fuzz testing; property-based testing on expression evaluation |
| **Frontend API stability** | Medium | Medium | Version the trait; use extension traits for optional features | Freeze v1.0 API after Oxur integration complete |
| **Macro system complexity** | Medium | High | Follow LFE's proven pattern exactly; defer hygiene initially | Test against LFE's macro test suite (adapted) |
| **Memory usage** | Low | Medium | Use `Arc` for sharing; implement custom compact representation for hot paths | Monitor memory in CI; target < 2x Rhai for equivalent programs |

---

## Part 8: Revised Prototyping Order

### Phase 1: Core Evaluator MVP (4 weeks)

**Goal:** Evaluate basic `syn::Expr` and `syn::Stmt` without ownership tracking.

**Deliverables:**
- [ ] `Value` enum (all primitive types)
- [ ] `Environment` struct (flat scope with frames)
- [ ] `Evaluate` impl for `syn::ExprLit` (literals)
- [ ] `Evaluate` impl for `syn::ExprBinary` (arithmetic, comparison)
- [ ] `Evaluate` impl for `syn::ExprPath` (variable lookup)
- [ ] `Evaluate` impl for `syn::ExprCall` (function calls)
- [ ] `Evaluate` impl for `syn::StmtLocal` (let bindings)
- [ ] `Evaluate` impl for `syn::ExprBlock` (blocks)
- [ ] `Evaluate` impl for `syn::ExprIf` (conditionals)
- [ ] `Evaluate` impl for `syn::ItemFn` (function definitions)
- [ ] Basic error types with span tracking
- [ ] CLI REPL for testing

**Duration:** 4 weeks

**Dependencies:** `syn`, `proc-macro2`

**Success criteria:**
```rust
// Can evaluate this:
let items: Vec<syn::Item> = syn::parse_str(r#"
    fn factorial(n: i64) -> i64 {
        if n <= 1 {
            1
        } else {
            n * factorial(n - 1)
        }
    }
"#)?;
let result = eval_items(&items, &mut env, &ctx)?;
// factorial(5) == 120
```

**Oxur integration:** None yet—pure Treebeard testing

### Phase 2: Frontend Trait + OxurFrontend (2 weeks)

**Goal:** Prove split architecture works with Oxur.

**Deliverables:**
- [ ] `LanguageFrontend` trait definition
- [ ] `RustFrontend` (trivial impl for testing)
- [ ] `OxurFrontend` struct (uses existing AST bridge)
- [ ] Run same test suite through both frontends
- [ ] Source span mapping (syn spans → Oxur positions)

**Duration:** 2 weeks

**Dependencies:** Phase 1, Oxur AST Bridge

**Success criteria:**
```rust
// Same program, two syntaxes, same result:
let rust_result = rust_frontend.eval("fn add(a: i32, b: i32) -> i32 { a + b }")?;
let oxur_result = oxur_frontend.eval("(defn add [a:i32 b:i32] -> i32 (+ a b))")?;
assert_eq!(rust_result, oxur_result);
```

**Oxur integration:** Uses existing AST Bridge (95%)

### Phase 3: Oxur Macro System (3 weeks)

**Goal:** Lisp macros work, producing syn AST.

**Deliverables:**
- [ ] `MacroExpander` struct
- [ ] `MacroEnvironment` (macro definitions)
- [ ] `defmacro` registration
- [ ] `gensym` implementation
- [ ] Quasiquote expansion (`, `, ~, ~@)
- [ ] Predefined macros: `defn`, `when`, `unless`, `cond`, `let`
- [ ] Recursive expansion with fixed-point detection
- [ ] Macro error reporting with span info

**Duration:** 3 weeks

**Dependencies:** Phase 2

**Success criteria:**
```lisp
;; Define a macro
(defmacro when [test & body]
  `(if ~test (do ~@body) nil))

;; Use it
(when (> x 0)
  (println! "positive")
  (+ x 1))

;; Expands to:
(if (> x 0) (do (println! "positive") (+ x 1)) nil)
```

**Oxur integration:** This is the 0% → 100% gap

### Phase 4: REPL Integration (2 weeks)

**Goal:** Full-featured REPL using existing Oxur infrastructure.

**Deliverables:**
- [ ] `Session` struct (three-environment pattern)
- [ ] History variables (+, ++, +++, *, **, ***)
- [ ] `slurp` / `unslurp` commands
- [ ] Error recovery (panic-safe eval)
- [ ] Integrate with existing Oxur REPL server
- [ ] Pretty printing with depth limits

**Duration:** 2 weeks

**Dependencies:** Phase 3, Oxur REPL infrastructure (60%)

**Success criteria:**
```
oxur> (defn add [a b] (+ a b))
#'add
oxur> (add 1 2)
3
oxur> *
3
oxur> (slurp "mylib.oxur")
✓ Slurped module: mylib
oxur> (unslurp)
✓ Reverted slurp
```

**Oxur integration:** Leverages existing REPL infrastructure

### Phase 5: Closures and Ownership (3 weeks)

**Goal:** Closures work; optional ownership checking.

**Deliverables:**
- [ ] `Closure` struct with upvalues
- [ ] Capture analysis (detect captured variables)
- [ ] `Evaluate` impl for `syn::ExprClosure`
- [ ] `OwnershipTracker` struct
- [ ] Retag on borrow creation
- [ ] Access checking
- [ ] Protectors for function arguments
- [ ] `OwnershipMode` configuration

**Duration:** 3 weeks

**Dependencies:** Phase 4

**Success criteria:**
```rust
// Closures capture correctly:
let x = 5;
let f = |y| x + y;  // captures x
f(3)  // => 8

// Ownership violations detected:
let x = vec![1, 2, 3];
let y = x;  // move
let z = x;  // ERROR: use after move
```

**Oxur integration:** Benefits Oxur immediately

### Phase 6: Compilation Escape Hatch (3 weeks)

**Goal:** Hot functions can be compiled to native code.

**Deliverables:**
- [ ] `Compiler` struct
- [ ] Rust codegen (syn AST → Rust source via `quote!`)
- [ ] rustc invocation with cdylib output
- [ ] Dynamic library loading via `libloading`
- [ ] `compile(fn)` REPL command
- [ ] Execution counter for hotspot detection
- [ ] Auto-compile threshold configuration

**Duration:** 3 weeks

**Dependencies:** Phase 5

**Success criteria:**
```
oxur> (defn fib [n] ...)
oxur> (time (fib 35))
Elapsed: 1500ms
oxur> (compile fib)
✓ Compiled fib to native code
oxur> (time (fib 35))
Elapsed: 15ms  ;; 100x speedup
```

**Oxur integration:** Major performance milestone

### Phase 7: Crate Loading (2 weeks)

**Goal:** Use Rust ecosystem crates from REPL.

**Deliverables:**
- [ ] `CrateLoader` struct
- [ ] Wrapper generation for crate functions
- [ ] Cargo invocation for building
- [ ] Symbol extraction from cdylib
- [ ] Type bridging (Value ↔ Rust types)
- [ ] `require` REPL command

**Duration:** 2 weeks

**Dependencies:** Phase 6

**Success criteria:**
```
oxur> (require "regex")
✓ Loaded crate: regex
oxur> (def re (regex/Regex:new "\\d+"))
oxur> (regex/Regex:is-match re "123")
true
```

**Oxur integration:** Full Rust ecosystem access

### Timeline Summary

| Phase | Duration | Cumulative | Milestone |
|-------|----------|------------|-----------|
| Phase 1: Core Evaluator | 4 weeks | Week 4 | Basic evaluation works |
| Phase 2: Frontend Trait | 2 weeks | Week 6 | Split architecture proven |
| Phase 3: Macro System | 3 weeks | Week 9 | Oxur macros work |
| Phase 4: REPL Integration | 2 weeks | Week 11 | Usable REPL |
| Phase 5: Closures + Ownership | 3 weeks | Week 14 | Full language coverage |
| Phase 6: Compilation | 3 weeks | Week 17 | Performance path exists |
| Phase 7: Crate Loading | 2 weeks | Week 19 | Rust ecosystem access |

**Total: ~19 weeks to full implementation**

---

## Part 9: Open Questions

### 9.1 Questions Requiring Prototyping

1. **How expensive is ownership tracking in practice?**
   - Can only measure with real implementation
   - Phase 5 will answer this

2. **What's the optimal Value size?**
   - Trade-off between inline capacity and enum size
   - Phase 1 will establish baseline; can optimize later

3. **Is index caching in AST worth it?**
   - Rhai uses it, but we have `Arc<syn::Expr>`
   - Phase 1 will benchmark with/without

4. **How much `syn` AST do we actually need?**
   - Track coverage in Phase 1-4
   - May find 80% of code uses 20% of types

### 9.2 Questions Requiring User Feedback

1. **Should ownership checking be on by default?**
   - Safer but slower
   - Need user feedback on acceptable slowdown

2. **What macro semantics do users expect?**
   - Hygienic by default? Opt-in hygiene?
   - LFE is non-hygienic; Elixir is hygienic
   - Survey Oxur users

3. **What REPL commands are most important?**
   - History, slurp, compile—what else?
   - Phase 4 can add more based on feedback

4. **How important is async/await support?**
   - Complex to implement in interpreter
   - May be better to compile async functions

### 9.3 Questions That Can Be Deferred

1. **Should we support Rust 2024 edition?**
   - Defer until Rust 2024 stabilizes
   - `syn` will handle it

2. **Should there be a bytecode compilation tier?**
   - Complexity doesn't justify benefit for REPL use
   - Defer until clear performance need

3. **Should we support debugging/stepping?**
   - Nice to have, not essential
   - Phase 8+ feature

4. **Should we support distributed evaluation?**
   - Way out of scope
   - Use Rust crates if needed

---

## Part 10: Appendices

### Appendix A: `syn` Type Coverage Plan

**High Priority (MVP):**
- `syn::Expr`: Lit, Binary, Unary, Path, Call, MethodCall, Block, If, Match, Closure, Return, Break, Continue
- `syn::Stmt`: Local, Expr, Semi
- `syn::Item`: Fn, Const, Static, Struct, Enum, Impl

**Medium Priority (Phase 5-6):**
- `syn::Expr`: Assign, AssignOp, Index, Field, Range, Reference, Try, Await
- `syn::Item`: Trait, Type, Mod, Use

**Low Priority (Phase 7+):**
- `syn::Expr`: Array, Tuple, Repeat, Cast, Let, While, Loop, ForLoop, Unsafe, Yield
- `syn::Item`: ExternCrate, Macro, Union, TraitAlias

**Probably Never:**
- `syn::Expr`: Verbatim, Infer, TryBlock (unstable)
- `syn::Item`: ForeignMod (FFI—use crate loader instead)

### Appendix B: Error Message Design

**Good error messages include:**

1. **What went wrong** (concise)
2. **Where it happened** (source location)
3. **Why it's wrong** (explanation)
4. **How to fix it** (suggestion)

**Example:**

```
error[E0382]: use of moved value: `x`
  --> src/main.rs:4:13
   |
2  |     let y = x;
   |             - value moved here
3  |     ...
4  |     let z = x;
   |             ^ value used after move
   |
   = note: move occurs because `x` has type `Vec<i32>`, which does not implement `Copy`
help: consider cloning the value
   |
2  |     let y = x.clone();
   |              ++++++++
```

**Treebeard should produce similar quality.**

### Appendix C: Benchmark Targets

| Benchmark | Target | Rationale |
|-----------|--------|-----------|
| Fibonacci(35) interpreted | < 2s | Reasonable for REPL |
| Fibonacci(35) compiled | < 50ms | Near-native |
| Simple REPL eval | < 10ms | Responsive feel |
| Hot reload | < 100ms | Seamless development |
| Startup time | < 500ms | Quick iteration |
| Memory (idle REPL) | < 50MB | Reasonable footprint |

### Appendix D: Glossary

- **AST Bridge:** Oxur's bidirectional converter between S-expressions and `syn` AST
- **Frontend:** A language that compiles to `syn` AST (e.g., Oxur, Rust)
- **Hot code loading:** Replacing function definitions without restarting
- **MVT:** Minimum Viable Treebeard
- **Retag:** Creating a new ownership tag when a borrow is created
- **Slurp:** Loading a file's definitions into the REPL environment
- **syn:** The canonical Rust AST library
- **Thin layer:** A language implementation that delegates to an underlying runtime
- **Treebeard:** The tree-walking interpreter for `syn` AST
- **Upvalue:** A captured variable in a closure

---

## Conclusion

This guide provides a concrete, actionable path from Oxur's current state to a production-ready Treebeard implementation. The key insights from the research are:

1. **Thin layer principle:** Do syntax transformation, delegate everything else to Rust/rustc
2. **Late binding advantage:** Tree-walkers get better hot reload than BEAM's bytecode
3. **Flat scope simplicity:** Rhai's pattern works well enough for REPL use cases
4. **Minimal ownership:** 8 bytes per value catches common errors without Miri's complexity
5. **LFE's macro pattern:** Expand before evaluation, separate compile-time and runtime environments

The architecture is designed to:
- Build on Oxur's existing 95% AST Bridge
- Leverage the 60% REPL infrastructure
- Fill the 0% macro system gap
- Replace the 25% evaluation with proper `syn` traversal

**Total estimated codebase:**
- Treebeard: ~10,000 lines
- Oxur additions: ~2,500 lines
- Combined: ~12,500 lines (under 15K target)

**This is achievable.** The research validates the approach, the evidence supports the decisions, and the path is clear. Time to build.

---

**End of Document**
