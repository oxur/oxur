# Treebeard Codebase Analysis Report

**Date:** 2026-01-10
**Analyst:** Claude Sonnet 4.5
**Purpose:** Inform the design of Treebeard, a tree-walking interpreter for Rust's `syn` AST

---

## Executive Summary

This report analyzes six Rust VM/interpreter implementations (Rhai, Miri, Rune, Gluon, Ketos, rust-hosted-langs/book) to extract patterns and recommendations for Treebeard. The key findings are:

1. **Environment/Binding:** Flat scope with reverse-order shadowing (Rhai) is simpler than frame chains
2. **Value Representation:** Three-tier approach (inline primitives + dynamic objects + native FFI) balances performance and flexibility (Rune)
3. **Ownership Tracking:** Minimal viable model needs only 10 bytes per value (per-value tags + permission + protected flag) vs Miri's 50+ bytes
4. **Rust Interop:** Declarative macro-based registration (Rune) is cleaner than imperative APIs (Rhai)
5. **Closure Capture:** Lua-style upvalues (book) combined with share-on-capture (Rhai) provides explicit control

---

## 1. Comparison Matrix

### 1.1 Architecture Overview

| Codebase | LOC | Execution Model | Bytecode? | Type System | GC Strategy | Primary Use Case |
|----------|-----|-----------------|-----------|-------------|-------------|------------------|
| **Rhai** | 77k | Tree-walking | No | Dynamic | Rc-based | Embedded scripting |
| **Miri** | 106k | MIR interpreter | Yes (MIR) | Static (Rust) | N/A (analysis) | UB detection |
| **Rune** | 200k | Bytecode VM | Yes | Dynamic + RTTI | Arc-based | Async scripting |
| **Gluon** | 90k | Bytecode VM | Yes | Static (HM) | Mark-sweep gen. | Functional lang |
| **Ketos** | 25k | Bytecode VM | Yes | Dynamic | Rc-based | Lisp dialect |
| **Book** | 8.5k | Bytecode VM | Yes | Monomorphic | stickyimmix | Educational |

**Key Insight:** Rhai is the only major tree-walking interpreter; all others use bytecode for performance.

### 1.2 Value Representation

| Codebase | Primary Type | Inline Types | Boxed Types | Custom Types | Size (bytes) |
|----------|--------------|--------------|-------------|--------------|--------------|
| **Rhai** | `Dynamic(Union)` | bool, int, float, char | String, Array, Map | Box\<Box\<dyn Variant\>\> | 32-40 |
| **Rune** | `Value(Repr)` | bool, int, uint, float, char | Dynamic, Any | AnyObj vtable | 9-16 |
| **Gluon** | `Value(ValueRepr)` | byte, int, float | String, Data, Array | GcPtr\<dyn Userdata\> | 16+ |
| **Ketos** | `Value` (enum) | bool, float, int | List, String, Struct | Rc\<dyn ForeignValue\> | 24+ |
| **Book** | `Value<'guard>` | Nil, Number (isize) | All others | ScopedPtr\<T\> | 8 (tagged) |

**Performance Ranking (fastest to slowest):**
1. **Book** - Tagged pointers (8 bytes, inline integers)
2. **Rune** - Inline enum (9-16 bytes, inline primitives)
3. **Gluon** - GcPtr enum (16+ bytes, typed allocations)
4. **Ketos** - Rc enum (24+ bytes, all Rc-wrapped)
5. **Rhai** - Dynamic union (32-40 bytes, includes Arc/Tag/AccessMode)

### 1.3 Environment/Scope Model

| Codebase | Structure | Lookup | Shadowing | Frame Isolation | Notes |
|----------|-----------|--------|-----------|-----------------|-------|
| **Rhai** | Flat arrays (names, values, aliases) | O(n) reverse search | Reverse iteration | Single Scope, push/pop | Index caching in AST |
| **Rune** | Stack with frame offsets | O(1) address + offset | Frame-local addressing | CallFrame stores `top` | Stack isolation via `top` |
| **Gluon** | N/A (compiled) | Compile-time resolution | N/A | N/A | Static scoping |
| **Ketos** | GlobalScope + Namespace | HashMap lookup | Namespace layering | Module boundaries | Separate constants/macros/values |
| **Book** | Thread.globals + upvalues | Dict lookup | Dict layering | CallFrame + stack_base | Upvalues bridge stack/heap |

**Key Insight:** Rhai's flat scope is simple but O(n); Rune's stack-based addressing is O(1) but requires compilation.

### 1.4 Closure Capture Mechanisms

| Codebase | Capture Model | Storage | Sharing | Mutation | Notes |
|----------|---------------|---------|---------|----------|-------|
| **Rhai** | Share-on-capture | FnPtr.curry: Vec\<Dynamic\> | Rc\<RefCell\> or Arc\<Mutex\> | Via Shared variant | Automatic via Stmt::Share |
| **Rune** | By-value capture | Closure instruction | Arc-wrapped | Via protocols | Captured at closure creation |
| **Gluon** | Upvars array | ClosureData.upvars: Array\<Value\> | GcPtr | GC-tracked | Homogeneous array |
| **Ketos** | Scope reference | Lambda holds scope | Rc\<GlobalScope\> | Via scope mutation | Dynamic lookup |
| **Book** | Upvalues (Lua-style) | Function.nonlocal_refs | Closed upvalues | Via Upvalue.set | Late binding to stack |

**Best Pattern:** Combine Book's explicit Upvalue pattern with Rhai's automatic share detection.

### 1.5 Rust Function Registration

| Codebase | API Style | Type Safety | Overhead | Async Support | Notes |
|----------|-----------|-------------|----------|---------------|-------|
| **Rhai** | Imperative (`register_fn`) | Compile-time generics | Minimal (hash lookup) | Limited | Macro-generated trait impls |
| **Rune** | Declarative (`#[rune::function]`) | Compile-time bounds | Minimal (hash + vtable) | First-class | Protocol system |
| **Gluon** | Trait-based (`Userdata`) | Compile-time | GC overhead | Limited | Type-safe FFI |
| **Ketos** | `ForeignValue` trait | Runtime | HashMap lookup | No | Dynamic typing |

**Best Pattern:** Rune's declarative macros with protocol system (cleaner API, better discoverability).

### 1.6 Ownership/Borrow Tracking

Only Miri implements this (by design). Key characteristics:

| Aspect | Implementation | Overhead | Simplification Opportunities |
|--------|----------------|----------|------------------------------|
| **Per-Value Tag** | BorTag (u64, non-zero) | 8 bytes | Could use u32 for REPL |
| **Per-Location State** | Stack or Tree (8-20 bytes) | 40-50 bytes | Flat per-value tags (10 bytes) |
| **Protected Tags** | SmallVec\<[(AllocId, Tag); 2]\> | 16 bytes | Keep (essential) |
| **Data Race Tracking** | GlobalDataRaceHandler | 30-50% overhead | Remove for REPL |
| **History/Diagnostics** | AllocHistory | 20+ bytes | Remove (just "violated by X") |

**Minimal Model for Treebeard:**
```rust
struct MinimalOwnership {
    tag: u32,              // 4 bytes (sufficient for REPL)
    permission: u8,        // 1 byte (Unique/Shared/Disabled)
    protected: bool,       // 1 byte
}
// Total: 6 bytes (+ 2 padding = 8 bytes aligned)
```

---

## 2. Detailed Analysis by Codebase

### 2.1 Rhai (77k LOC) - Tree-Walking Interpreter

**Repository:** https://github.com/rhaiscript/rhai

#### Architecture
- **Execution:** Direct AST traversal with recursive evaluation
- **Scope:** Flat arrays (names, values, aliases) with reverse-order shadowing
- **Caching:** Pre-calculated variable indices in AST nodes
- **Dynamic type:** `Dynamic(Union)` with Tag and AccessMode

#### Key Strengths
- Simple, straightforward codebase
- Excellent Rust interop via `register_fn`
- Good performance for embedded use (no compilation overhead)
- SmartString optimization (inline to 23 chars)

#### Key Weaknesses
- No tail call optimization
- O(n) variable lookup (mitigated by index caching)
- Large value size (32-40 bytes)
- No first-class async support

#### Patterns to Adopt for Treebeard
1. **Flat scope with reverse shadowing** - simpler than frame chains
2. **Index caching in AST** - pre-calculated offsets for hot variables
3. **SmartString inlining** - most identifiers fit in 23 bytes
4. **Share-on-capture closures** - automatic Stmt::Share before closure creation
5. **Tag system** - metadata per value (useful for source location tracking)

#### Patterns to Avoid
- Large Dynamic enum (32-40 bytes) - use smaller representation
- O(n) scope search without bytecode compilation
- No TCO - Treebeard should at least detect tail position

**File Reference:**
- Environment: `src/types/scope.rs:62-74`
- Value: `src/types/dynamic.rs:54-103`
- Closures: `src/parser.rs:3700-3774`, `src/eval/stmt.rs:978-1023`
- Registration: `src/func/register.rs:127-245`

---

### 2.2 Miri (106k LOC) - Runtime Ownership Verification

**Repository:** https://github.com/rust-lang/miri

#### Architecture
- **Execution:** MIR (mid-level IR) interpreter
- **Ownership:** Dual implementation (Stacked Borrows / Tree Borrows)
- **Per-value tags:** BorTag (NonZero\<u64\>)
- **Per-location state:** Permission stacks or tree nodes

#### Key Strengths
- Full fidelity Rust semantics (catches all UB)
- Efficient bit-packing (8-byte stack items)
- LRU cache optimization (32-entry)
- Provenance tracking via tags

#### Key Weaknesses
- High overhead (8-40% slowdown minimum)
- Complex diagnostics infrastructure
- Assumes static compilation (not REPL)
- Data race tracking adds 30-50% overhead

#### Patterns to Adopt for Treebeard
1. **BorTag system** - sequential u64 tags (could use u32 for REPL)
2. **Permission enum** - Unique/Shared/Disabled (2 bits)
3. **Protectors** - WeakProtector/StrongProtector for noalias
4. **Frame-based release** - protected tags released on scope exit
5. **Retag protocol** - create tags on borrow, validate on access

#### Patterns to Avoid
- Full Stacked Borrows (too complex for Treebeard)
- Data race detection (not needed for single-threaded REPL)
- Comprehensive diagnostics (just "violated by tag X")

**Minimal Adaptation for Treebeard:**
```rust
struct ValueOwnership {
    tag: u32,                    // Unique tag per borrow
    permission: Permission,      // Unique/SharedRW/SharedRO/Disabled
    protected: bool,             // In protected frame?
}

enum Permission {
    Unique,           // &mut - exclusive
    SharedReadWrite,  // UnsafeCell-like
    SharedReadOnly,   // & - shared
    Disabled,         // Moved or invalidated
}
```

**File Reference:**
- Machine: `src/machine.rs:484-649`
- Borrow Tracker: `src/borrow_tracker/mod.rs:101-127`
- Stacked Borrows: `src/borrow_tracker/stacked_borrows/mod.rs:30-39`
- Item Packing: `src/borrow_tracker/stacked_borrows/item.rs:6-61`

---

### 2.3 Rune (200k LOC) - Bytecode VM with Async

**Repository:** https://github.com/rune-rs/rune

#### Architecture
- **Execution:** Bytecode dispatch loop (70+ instructions)
- **Value:** Three-tier (Inline/Dynamic/Any)
- **Stack:** Frame-isolated with address offsets
- **Type system:** Runtime hash-based + RTTI

#### Key Strengths
- Fast bytecode dispatch
- Excellent async/await support
- Clean protocol system
- Declarative function registration
- Pattern matching instructions

#### Key Weaknesses
- Large codebase (200k LOC)
- Requires compilation (not pure tree-walking)
- Complex module system
- Higher memory usage for bytecode

#### Patterns to Adopt for Treebeard
1. **Three-tier value model** - Inline (Copy) + Dynamic + Any (FFI)
2. **Hash-based type identity** - O(1) type checking
3. **Declarative registration** - `#[treebeard::function]` macros
4. **Protocol system** - Overloadable operations (Display, Index, etc.)
5. **Pattern matching support** - MatchType/MatchSequence/MatchObject
6. **Stack frame isolation** - `top` pointer prevents cross-frame access

#### Patterns to Avoid
- Full bytecode compilation (if Treebeard is tree-walking)
- Complex RTTI system (keep minimal for REPL)

**Hybrid Approach for Treebeard:**
Could use Rune-style bytecode for hot paths (compilation escape hatch) while tree-walking for cold paths.

**File Reference:**
- VM: `crates/rune/src/runtime/vm.rs:103-116`
- Value: `crates/rune/src/runtime/value.rs:67-88`
- Inline: `crates/rune/src/runtime/value/inline.rs:17-52`
- Instructions: `crates/rune/src/runtime/inst.rs:17-52`
- Pattern matching: `crates/rune/src/runtime/vm.rs:2531-2650`

---

### 2.4 Gluon (90k LOC) - Statically-Typed Functional Language

**Repository:** https://github.com/gluon-lang/gluon

#### Key Contributions
- **DataDef trait** - Custom allocation with size calculation
- **Generational GC** - Efficient for long-lived closures
- **Type-first design** - Full type information through compilation
- **ClosureData model** - Function + upvars array

**Pattern to Adopt:**
```rust
pub trait DataDef {
    type Value;
    fn size(&self) -> usize;
    fn initialize<'w>(self, result: WriteOnly<'w, Self::Value>) -> &'w mut Self::Value;
}
```

This allows Treebeard to allocate variable-sized values efficiently.

**File Reference:**
- Value: `vm/src/value.rs:35-461`
- Closure: Found in value.rs implementation
- GC: `vm/src/gc.rs:1-250+`

---

### 2.5 Ketos (25k LOC) - Dynamic Lisp

**Repository:** https://github.com/murarth/ketos

#### Key Contributions
- **GlobalScope + Namespace** - Separate constants/macros/values
- **Comprehensive bytecode** - 50+ instruction types
- **Simple Rc model** - No complex GC

**Pattern to Adopt:**
Separate namespaces for different binding kinds (useful if Treebeard supports macros or const evaluation).

**File Reference:**
- Value: `src/ketos/value.rs:1-150`
- Scope: `src/ketos/scope.rs:1-120+`
- Bytecode: `src/ketos/bytecode.rs:1-120`

---

### 2.6 rust-hosted-langs/book (8.5k LOC) - Educational Interpreter

**Repository:** https://github.com/rust-hosted-langs/book

#### Key Contributions
- **Tagged pointer optimization** - 8-byte values
- **Upvalue pattern** - Lua-style late binding
- **Partial application** - Explicit currying support
- **Lifetime guards** - Safe pointer manipulation

**Pattern to Adopt:**
```rust
pub struct Upvalue {
    value: TaggedCellPtr,    // Closed value or nil
    closed: Cell<bool>,      // Is this closed?
    location: ArraySize,     // Stack location if open
}

impl Upvalue {
    fn close(&self, guard, stack) {
        let ptr = stack.get(self.location).get_ptr();
        self.value.set_to_ptr(ptr);
        self.closed.set(true);
    }
}
```

This provides explicit control over when closures capture by value vs by reference.

**File Reference:**
- Tagged pointers: `interpreter/src/taggedptr.rs:1-250+`
- Upvalues: `interpreter/src/function.rs:1-200+`
- VM: `interpreter/src/vm.rs:1-250+`

---

## 3. Synthesis: Answering Treebeard Design Questions

### 3.1 What's the best environment representation for Treebeard?

**Given:** Tree-walking (not bytecode), syn AST, need to track ownership

**Recommendation:** **Hybrid Rhai/Rune model**

```rust
pub struct Environment {
    // Flat scope like Rhai for simplicity
    names: Vec<Ident>,
    values: Vec<Value>,
    ownership: Vec<ValueOwnership>,  // Parallel array for ownership state

    // Frame markers for proper cleanup
    frame_boundaries: Vec<usize>,  // Stack of frame start indices
}

impl Environment {
    // O(n) lookup with reverse shadowing (like Rhai)
    pub fn lookup(&self, name: &Ident) -> Option<(usize, &Value)> {
        self.names.iter().enumerate().rev()
            .find(|(_, n)| *n == name)
            .map(|(i, _)| (i, &self.values[i]))
    }

    // Push new frame boundary
    pub fn push_frame(&mut self) {
        self.frame_boundaries.push(self.names.len());
    }

    // Pop frame and release protectors
    pub fn pop_frame(&mut self) {
        let boundary = self.frame_boundaries.pop().unwrap();
        // Release protected tags for values >= boundary
        for ownership in &mut self.ownership[boundary..] {
            if ownership.protected {
                // Implicit read access, then unprotect
                ownership.protected = false;
            }
        }
        // Truncate to boundary
        self.names.truncate(boundary);
        self.values.truncate(boundary);
        self.ownership.truncate(boundary);
    }
}
```

**Why this works:**
- Simple flat structure (easy to implement and debug)
- O(n) lookup is acceptable for tree-walking (already slow)
- Parallel ownership array tracks borrow state per value
- Frame boundaries enable proper cleanup on scope exit
- Can optimize later with index caching in AST (like Rhai)

### 3.2 What's the minimum viable ownership tracking model?

**Given:** REPL use case, 10-100x slowdown acceptable, full checking at compile time

**Recommendation:** **Simplified Stacked Borrows with 8-byte per-value overhead**

```rust
#[derive(Copy, Clone)]
pub struct ValueOwnership {
    tag: u32,              // Unique borrow tag (4 bytes)
    permission: Permission, // Current permission (1 byte)
    protected: bool,       // In protected scope (1 byte)
    _padding: u16,         // Alignment (2 bytes)
}  // Total: 8 bytes

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Permission {
    Unique = 0,           // &mut - exclusive access
    SharedReadWrite = 1,  // UnsafeCell - shared mutable
    SharedReadOnly = 2,   // & - shared immutable
    Disabled = 3,         // Moved or invalidated
}

pub struct OwnershipTracker {
    next_tag: u32,  // Monotonic tag counter
    protected_tags: Vec<(usize, u32)>,  // (env_index, tag) pairs in current frame
}

impl OwnershipTracker {
    // Called on every borrow operation (& or &mut)
    pub fn retag(&mut self, env_index: usize, borrow_kind: BorrowKind) -> ValueOwnership {
        let new_tag = self.next_tag;
        self.next_tag += 1;

        let permission = match borrow_kind {
            BorrowKind::Shared => Permission::SharedReadOnly,
            BorrowKind::Mut => Permission::Unique,
        };

        ValueOwnership {
            tag: new_tag,
            permission,
            protected: false,  // Set by caller if needed
            _padding: 0,
        }
    }

    // Called on every memory access (read or write)
    pub fn check_access(
        &self,
        ownership: &ValueOwnership,
        access_kind: AccessKind,
    ) -> Result<(), BorrowError> {
        match (ownership.permission, access_kind) {
            (Permission::Disabled, _) => Err(BorrowError::UseAfterMove),
            (Permission::SharedReadOnly, AccessKind::Write) => Err(BorrowError::WriteToShared),
            (Permission::Unique | Permission::SharedReadWrite, _) => Ok(()),
            (Permission::SharedReadOnly, AccessKind::Read) => Ok(()),
        }
    }

    // Called on scope exit
    pub fn release_protectors(&mut self, env: &mut Environment) {
        for (env_index, tag) in &self.protected_tags {
            // Implicit read access through protected tag
            let ownership = &env.ownership[*env_index];
            self.check_access(ownership, AccessKind::Read).unwrap();

            // Unprotect
            env.ownership[*env_index].protected = false;
        }
        self.protected_tags.clear();
    }
}
```

**What's simplified vs Miri:**
- **No per-location stacks** - just per-value tags (saves 40 bytes per value)
- **No data race tracking** - single-threaded REPL (saves 30-50% overhead)
- **No comprehensive diagnostics** - just "borrowed value used after move at line X"
- **No Tree Borrows** - simpler Stacked Borrows sufficient
- **No exposed tags** - no int-to-ptr casts in REPL

**What's kept:**
- **Tag-based provenance** - every borrow gets a unique tag
- **Permission tracking** - Unique/Shared/Disabled states
- **Protectors** - function arguments marked protected
- **Frame-based cleanup** - release protectors on scope exit

**Estimated overhead:** ~10-20% vs Miri's 40-100% (for full checking)

### 3.3 How should Treebeard call compiled Rust code?

**Given:** Need to invoke rustc-compiled functions, pass/return Values

**Recommendation:** **Rune-style declarative macros + Rhai-style type conversion**

```rust
// 1. Define Value type with conversion traits
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Rc<String>),
    // ... other variants
    Native(Box<dyn Any>),  // For Rust types
}

// 2. FromValue/ToValue traits (like Rhai's Variant)
pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self, TypeError>;
}

pub trait ToValue {
    fn to_value(self) -> Value;
}

// Implement for primitives
impl FromValue for i64 {
    fn from_value(value: &Value) -> Result<Self, TypeError> {
        match value {
            Value::Int(i) => Ok(*i),
            _ => Err(TypeError::expected("Int", value.type_name())),
        }
    }
}

impl ToValue for i64 {
    fn to_value(self) -> Value {
        Value::Int(self)
    }
}

// 3. Declarative registration macro (like Rune)
#[treebeard::function]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

// Expands to:
impl TreebeardFunction for add {
    fn call(args: &[Value]) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::ArgCount { expected: 2, got: args.len() });
        }
        let a = i64::from_value(&args[0])?;
        let b = i64::from_value(&args[1])?;
        let result = add(a, b);
        Ok(result.to_value())
    }
}

// 4. Compilation escape hatch
pub struct CompiledFunction {
    func_ptr: fn(&[Value]) -> Result<Value, RuntimeError>,
    signature: FunctionSignature,
}

impl Context {
    pub fn register_compiled<F>(&mut self, name: &str, func: CompiledFunction) {
        self.functions.insert(name.to_string(), FunctionKind::Compiled(func));
    }

    pub fn compile_and_register(&mut self, name: &str, expr: &syn::Expr) -> Result<(), CompileError> {
        // 1. Generate Rust source from syn::Expr
        let rust_source = syn_to_rust(expr)?;

        // 2. Compile to .so/.dylib via rustc
        let lib_path = rustc_compile(&rust_source)?;

        // 3. Load symbol and register
        let func_ptr = unsafe { load_symbol(&lib_path, name)? };
        self.register_compiled(name, CompiledFunction { func_ptr, signature });

        Ok(())
    }
}
```

**Why this works:**
- **Type-safe at compile time** - macro checks F: FromValue for all args
- **Zero overhead** - direct function call, no dynamic dispatch
- **Familiar syntax** - Rust functions just work with `#[treebeard::function]`
- **Compilation escape hatch** - can JIT hot functions to native code

**Comparison with other codebases:**
- **Better than Rhai:** Declarative (vs imperative `register_fn` calls)
- **Better than Rune:** Simpler (no complex protocol system needed initially)
- **Better than Ketos:** Type-safe (vs runtime type errors)

### 3.4 What value representation gives the best tradeoff?

**Given:** Need to track ownership state per-value, support Rust's types

**Recommendation:** **Hybrid Rune + Book model with ownership tracking**

```rust
#[derive(Clone)]
pub struct Value {
    repr: ValueRepr,
    ownership: ValueOwnership,  // 8 bytes inline
}

#[derive(Clone)]
pub enum ValueRepr {
    // Inline primitives (Copy) - like Rune
    Inline(InlineValue),

    // Heap-allocated types - like Rune
    Heap(Rc<HeapValue>),

    // Rust native types - like Rune's Any
    Native(Box<dyn Any>),
}

#[derive(Copy, Clone)]
pub enum InlineValue {
    Unit,
    Bool(bool),
    Char(char),
    Int(i64),
    Float(f64),
    // Could add more if needed (e.g., small strings)
}

pub enum HeapValue {
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Closure(ClosureData),
    Struct(StructData),
}

pub struct ClosureData {
    func: Rc<syn::ItemFn>,         // AST of function
    env: Vec<Value>,                // Captured upvalues
    upvalue_names: Vec<Ident>,     // For debugging
}
```

**Memory layout:**
```
Value = 24 bytes
├─ ValueRepr = 16 bytes
│  ├─ Discriminant = 1 byte
│  ├─ Payload = 15 bytes
│  │  ├─ Inline: 8 bytes (i64/f64)
│  │  ├─ Heap: 8 bytes (Rc pointer)
│  │  └─ Native: 8 bytes (Box pointer)
│  └─ Padding = 7 bytes
└─ ValueOwnership = 8 bytes
   ├─ tag = 4 bytes
   ├─ permission = 1 byte
   ├─ protected = 1 byte
   └─ padding = 2 bytes
```

**Why this works:**
- **Small for primitives** - 24 bytes total, inline storage for int/float
- **Ownership inline** - no separate lookup, cache-friendly
- **Rc for heap** - simple, predictable memory management
- **Extensible** - Native variant for user-defined Rust types

**Comparison:**
| Model | Size | Pros | Cons |
|-------|------|------|------|
| **This model** | 24 bytes | Balanced; inline ownership | Larger than Book |
| **Rhai** | 32-40 bytes | Proven in production | Too large |
| **Rune** | 9-16 bytes | Smallest repr | Would need +8 for ownership |
| **Book** | 8 bytes | Fastest | Requires unsafe; no ownership |

### 3.5 How should closures work with ownership?

**Given:** Closures may capture by reference or by move

**Recommendation:** **Combine Rhai's share-on-capture + Book's explicit upvalues**

```rust
pub struct ClosureData {
    func: Rc<syn::ItemFn>,
    upvalues: Vec<Upvalue>,  // Explicit upvalue list (like Book)
}

pub enum Upvalue {
    // Captured by immutable reference - shared ownership
    Shared {
        value: Value,           // Shared clone of original
        source_tag: u32,        // Tag of original borrow
    },

    // Captured by mutable reference - moved ownership
    Owned {
        value: Value,           // Moved value
    },

    // Captured by reference to stack (open upvalue, like Book)
    Open {
        env_index: usize,       // Index into environment
        borrow_tag: u32,        // Tag of borrow
    },
}

impl Interpreter {
    // Called when analyzing closure captures
    fn create_closure(&mut self, func: &syn::ItemFn, captures: &[Ident]) -> Result<Value, Error> {
        let mut upvalues = Vec::new();

        for capture in captures {
            let (env_index, value) = self.env.lookup(capture)
                .ok_or_else(|| Error::VariableNotFound)?;

            let ownership = &self.env.ownership[env_index];

            // Determine capture mode based on usage in closure body
            let capture_mode = self.analyze_capture_mode(func, capture)?;

            let upvalue = match capture_mode {
                CaptureMode::SharedRef => {
                    // Clone value with shared ownership
                    let shared_value = value.clone();

                    // Update ownership: new tag, SharedReadOnly permission
                    let new_ownership = self.ownership_tracker.retag(env_index, BorrowKind::Shared);

                    Upvalue::Shared {
                        value: shared_value,
                        source_tag: ownership.tag,
                    }
                }

                CaptureMode::MutRef => {
                    // For mutable capture, use open upvalue (like Book)
                    let borrow_tag = self.ownership_tracker.retag(env_index, BorrowKind::Mut).tag;

                    // Mark original as borrowed mutably
                    self.env.ownership[env_index].permission = Permission::Disabled;

                    Upvalue::Open {
                        env_index,
                        borrow_tag,
                    }
                }

                CaptureMode::Move => {
                    // Move value (disable original)
                    let owned_value = value.clone();  // Shallow clone
                    self.env.ownership[env_index].permission = Permission::Disabled;

                    Upvalue::Owned {
                        value: owned_value,
                    }
                }
            };

            upvalues.push(upvalue);
        }

        Ok(Value {
            repr: ValueRepr::Heap(Rc::new(HeapValue::Closure(ClosureData {
                func: Rc::new(func.clone()),
                upvalues,
            }))),
            ownership: ValueOwnership::default(),  // Closure itself is owned
        })
    }

    // Close open upvalues when environment is popped
    fn close_upvalues(&mut self, env_index: usize) {
        // Find all closures with Open upvalues referencing env_index
        // Convert them to Owned or Shared
    }
}
```

**Why this works:**
- **Explicit capture semantics** - clear which variables are captured and how
- **Ownership tracking** - disabled source when moved, shared when borrowed
- **Open upvalues** - efficient for closures that escape their scope
- **Hybrid model** - combines Rhai's simplicity with Book's control

**Closure call sequence:**
1. Lookup closure value
2. Push new environment frame
3. Load upvalues into environment (respecting ownership)
4. Evaluate function body
5. Pop frame (releasing protectors)

---

## 4. Recommended Patterns for Treebeard

### 4.1 Core Architecture

```rust
pub struct Interpreter {
    // Environment
    env: Environment,                  // Flat scope (Rhai-style)

    // Ownership tracking
    ownership_tracker: OwnershipTracker,  // Simplified Stacked Borrows

    // Context
    context: Context,                  // Functions, types, modules

    // AST
    ast: syn::File,                    // Parsed Rust AST
}

pub struct Environment {
    names: Vec<Ident>,
    values: Vec<Value>,
    ownership: Vec<ValueOwnership>,
    frame_boundaries: Vec<usize>,
}

pub struct Value {
    repr: ValueRepr,                   // Rune-style three-tier
    ownership: ValueOwnership,         // 8-byte inline tracking
}

pub struct OwnershipTracker {
    next_tag: u32,
    protected_tags: Vec<(usize, u32)>,
}
```

### 4.2 Evaluation Pattern

```rust
impl Interpreter {
    pub fn eval_expr(&mut self, expr: &syn::Expr) -> Result<Value, Error> {
        match expr {
            syn::Expr::Lit(lit) => self.eval_lit(lit),
            syn::Expr::Path(path) => self.eval_path(path),
            syn::Expr::Binary(bin) => self.eval_binary(bin),
            syn::Expr::Call(call) => self.eval_call(call),
            syn::Expr::Closure(closure) => self.eval_closure(closure),
            syn::Expr::Reference(reference) => self.eval_reference(reference),
            // ... other expression types
        }
    }

    fn eval_reference(&mut self, reference: &syn::ExprReference) -> Result<Value, Error> {
        // Evaluate inner expression
        let inner_value = self.eval_expr(&reference.expr)?;

        // Retag for borrow
        let borrow_kind = if reference.mutability.is_some() {
            BorrowKind::Mut
        } else {
            BorrowKind::Shared
        };

        // Find environment index (for ownership tracking)
        let env_index = self.find_env_index_for_value(&inner_value)?;

        // Create new borrow tag
        let new_ownership = self.ownership_tracker.retag(env_index, borrow_kind);

        // Return borrowed value with new ownership
        Ok(Value {
            repr: inner_value.repr.clone(),  // Shallow clone
            ownership: new_ownership,
        })
    }

    fn eval_call(&mut self, call: &syn::ExprCall) -> Result<Value, Error> {
        // Evaluate function expression
        let func_value = self.eval_expr(&call.func)?;

        // Evaluate arguments
        let args: Result<Vec<_>, _> = call.args.iter()
            .map(|arg| self.eval_expr(arg))
            .collect();
        let args = args?;

        // Dispatch based on function kind
        match &func_value.repr {
            ValueRepr::Heap(Rc::HeapValue::Closure(closure)) => {
                self.call_closure(closure, &args)
            }
            ValueRepr::Native(native) => {
                self.call_native(native, &args)
            }
            _ => Err(Error::NotCallable),
        }
    }

    fn call_closure(&mut self, closure: &ClosureData, args: &[Value]) -> Result<Value, Error> {
        // 1. Push frame
        self.env.push_frame();

        // 2. Bind parameters
        let params = extract_function_params(&closure.func)?;
        if params.len() != args.len() {
            return Err(Error::ArgCount);
        }

        for (param, arg) in params.iter().zip(args) {
            // Mark parameter as protected (function argument)
            let mut arg_ownership = arg.ownership;
            arg_ownership.protected = true;

            self.env.push(param.ident.clone(), arg.clone(), arg_ownership);
            self.ownership_tracker.protected_tags.push((self.env.values.len() - 1, arg_ownership.tag));
        }

        // 3. Load upvalues
        for (i, upvalue) in closure.upvalues.iter().enumerate() {
            let (name, value) = match upvalue {
                Upvalue::Shared { value, .. } => (format!("__upvalue_{}", i), value.clone()),
                Upvalue::Owned { value } => (format!("__upvalue_{}", i), value.clone()),
                Upvalue::Open { env_index, .. } => {
                    (format!("__upvalue_{}", i), self.env.values[*env_index].clone())
                }
            };
            self.env.push(name, value.clone(), value.ownership);
        }

        // 4. Evaluate body
        let result = self.eval_block(&closure.func.block)?;

        // 5. Pop frame (releases protectors)
        self.ownership_tracker.release_protectors(&mut self.env);
        self.env.pop_frame();

        Ok(result)
    }
}
```

### 4.3 Compilation Escape Hatch

```rust
pub enum FunctionKind {
    Interpreted(ClosureData),
    Compiled(CompiledFunction),
}

pub struct CompiledFunction {
    func_ptr: libloading::Symbol<fn(&[Value]) -> Result<Value, RuntimeError>>,
    lib: libloading::Library,  // Keep library loaded
}

impl Interpreter {
    pub fn compile_hot_function(&mut self, name: &str, expr: &syn::Expr) -> Result<(), Error> {
        // 1. Generate Rust source
        let rust_source = format!(
            r#"
            use treebeard::{{Value, RuntimeError}};

            #[no_mangle]
            pub extern "C" fn {name}(args: &[Value]) -> Result<Value, RuntimeError> {{
                {body}
            }}
            "#,
            name = name,
            body = syn_to_rust(expr)?,
        );

        // 2. Compile with rustc
        let lib_path = self.rustc_compile(&rust_source)?;

        // 3. Load library
        let lib = unsafe { libloading::Library::new(&lib_path)? };
        let func_ptr = unsafe {
            lib.get::<fn(&[Value]) -> Result<Value, RuntimeError>>(name.as_bytes())?
        };

        // 4. Register
        self.context.functions.insert(name.to_string(), FunctionKind::Compiled(CompiledFunction {
            func_ptr,
            lib,
        }));

        Ok(())
    }

    fn rustc_compile(&self, source: &str) -> Result<PathBuf, Error> {
        // Write source to temp file
        let temp_dir = tempfile::tempdir()?;
        let src_path = temp_dir.path().join("compiled.rs");
        std::fs::write(&src_path, source)?;

        // Invoke rustc
        let output = std::process::Command::new("rustc")
            .args(&[
                "--crate-type", "cdylib",
                "-C", "opt-level=3",
                "-o", "compiled.so",
                src_path.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(Error::CompilationFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(temp_dir.path().join("compiled.so"))
    }
}
```

---

## 5. Implementation Roadmap

### Phase 1: Core Interpreter (MVP)
- [ ] Value representation (InlineValue + HeapValue)
- [ ] Environment (flat scope with push/pop)
- [ ] Basic expression evaluation (literals, variables, binary ops)
- [ ] Simple function calls (no closures)
- [ ] Error handling

**Goal:** Can evaluate `let x = 1 + 2; x * 3` → `9`

### Phase 2: Ownership Tracking
- [ ] ValueOwnership struct
- [ ] OwnershipTracker (tag allocation)
- [ ] Retag on borrow creation
- [ ] Check on memory access
- [ ] Protectors for function arguments

**Goal:** Catches `let x = 5; let y = &x; let z = x;` → error (use after move)

### Phase 3: Closures
- [ ] ClosureData struct
- [ ] Capture analysis (which variables are captured?)
- [ ] Upvalue creation (Shared/Owned/Open)
- [ ] Closure call with upvalue loading
- [ ] Open upvalue closing

**Goal:** Can execute `let x = 5; let f = |y| x + y; f(3)` → `8`

### Phase 4: Rust Interop
- [ ] FromValue/ToValue traits
- [ ] #[treebeard::function] macro
- [ ] Native function registration
- [ ] Type-safe argument extraction

**Goal:** Can call `println!("hello")` from Treebeard code

### Phase 5: Compilation Escape Hatch
- [ ] rustc invocation
- [ ] Dynamic library loading
- [ ] Hotspot detection (execution counters)
- [ ] Automatic compilation of hot functions

**Goal:** Hot loop automatically JIT-compiled to native code

### Phase 6: REPL
- [ ] nREPL protocol implementation
- [ ] Multi-line input handling
- [ ] Error recovery (don't crash on bad input)
- [ ] Auto-completion
- [ ] Source location tracking for errors

**Goal:** Interactive REPL with good UX

---

## 6. Risk Assessment

### 6.1 Risks That Might Not Work

| Risk | Mitigation |
|------|------------|
| **Ownership tracking overhead too high** | Start with opt-in checking; make it a feature flag |
| **Tree-walking too slow** | Use compilation escape hatch for hot paths |
| **Closure capture analysis too complex** | Start with explicit `move` keyword; infer later |
| **syn AST too large in memory** | Use Arc\<syn::Expr\> and share subtrees |
| **rustc compilation too slow** | Cache compiled libraries; only recompile on change |
| **nREPL protocol mismatch** | Implement subset first; extend as needed |

### 6.2 Complexity Budget

**Target:** ~50k lines of Rust

| Component | LOC Estimate | Justification |
|-----------|--------------|---------------|
| **Value + Environment** | 2k | Simple enums and flat scope |
| **Ownership Tracker** | 3k | Simplified Stacked Borrows |
| **Expression Evaluator** | 8k | ~50 expression types in syn |
| **Statement Evaluator** | 5k | ~20 statement types |
| **Closures** | 4k | Capture analysis + upvalues |
| **Rust Interop** | 6k | FromValue/ToValue + macros |
| **Compilation Escape** | 5k | rustc invocation + loading |
| **REPL** | 4k | nREPL protocol |
| **Error Handling** | 3k | Error types and formatting |
| **Testing** | 10k | Comprehensive test suite |
| **Total** | **50k** | Fits within budget |

---

## 7. Conclusion

Based on analysis of six Rust VM/interpreter implementations, the recommended architecture for Treebeard is:

1. **Environment:** Flat scope with reverse shadowing (Rhai-style) + frame boundaries
2. **Value Representation:** Three-tier model (Inline/Heap/Native) with 8-byte inline ownership (Rune + Miri fusion)
3. **Ownership Tracking:** Simplified Stacked Borrows with per-value tags (10-byte overhead)
4. **Closures:** Explicit upvalues (Book-style) with share-on-capture analysis (Rhai-style)
5. **Rust Interop:** Declarative macros (Rune-style) with type-safe conversion (Rhai-style)
6. **Compilation Escape:** rustc JIT for hot functions with dynamic library loading

This architecture balances **simplicity** (tree-walking), **performance** (compilation escape hatch), and **correctness** (ownership tracking) while staying within the 50k LOC budget and achieving 10-100x native performance.

---

## 8. Appendix: Code Locations Reference

### Rhai
- Environment: `src/types/scope.rs:62-74`
- Value: `src/types/dynamic.rs:54-103`
- Closures: `src/parser.rs:3700-3774`
- Registration: `src/func/register.rs:127-245`

### Miri
- Machine: `src/machine.rs:484-649`
- Borrow Tracker: `src/borrow_tracker/mod.rs:101-127`
- Stacked Borrows: `src/borrow_tracker/stacked_borrows/mod.rs:30-39`
- Item Packing: `src/borrow_tracker/stacked_borrows/item.rs:6-61`

### Rune
- VM: `crates/rune/src/runtime/vm.rs:103-116`
- Value: `crates/rune/src/runtime/value.rs:67-88`
- Inline: `crates/rune/src/runtime/value/inline.rs:17-52`
- Instructions: `crates/rune/src/runtime/inst.rs:17-52`

### Gluon
- Value: `vm/src/value.rs:35-461`
- GC: `vm/src/gc.rs:1-250+`

### Ketos
- Value: `src/ketos/value.rs:1-150`
- Scope: `src/ketos/scope.rs:1-120+`

### Book
- Tagged Pointers: `interpreter/src/taggedptr.rs:1-250+`
- Upvalues: `interpreter/src/function.rs:1-200+`

---

**End of Report**
