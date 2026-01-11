# Treebeard Architectural Research: A Tree-Walking Interpreter for Oxur

**Oxur requires a fundamentally new interpreter design that marries Lisp's REPL-first development with Rust's ownership semantics.** This research reveals that while no production interpreter has previously combined these properties, the pieces exist: Miri proves runtime ownership tracking is possible, RefCell demonstrates minimal viable borrow checking, and established patterns from Clojure, BEAM, and LuaJIT show how hybrid interpretation/compilation architectures can work. The recommended architecture centers on a **hybrid ownership model**—runtime enforcement of ownership and simple borrowing with complex lifetime analysis deferred to compilation.

---

## 1. Tree-walking interpreter design patterns

### Summary of findings

Tree-walking interpreters prove well-suited for REPL-focused use cases despite their **10-100x performance overhead** compared to bytecode. Research into Boa, Rune, and several Rust-based Lisp interpreters reveals a consistent pattern: while production JavaScript/Rust interpreters have evolved to bytecode compilation for performance, pure tree-walking remains viable within Treebeard's performance budget.

The canonical environment model from SICP—a chain of frames where each frame maps symbols to values with a parent pointer—dominates implementations. Rust interpreters consistently use `Rc<RefCell<Environment>>` for the frame chain, enabling interior mutability needed for closures that capture mutable state. Persistent data structures (rpds, hamt-rs) offer an alternative with ~2x lookup overhead but better support for backtracking and parallel evaluation.

For closures, tree-walking interpreters capture the **entire enclosing environment** at definition time, then extend it with parameter bindings at call time. This differs from bytecode VMs which typically use flat closures with explicit upvalue tracking.

Tail call optimization in tree-walking requires explicit techniques since host language recursion consumes stack. **Trampolining** emerges as the cleanest approach—returning a "continue" value that the main loop iterates rather than recursing.

### Key design decisions with tradeoffs

| Decision | Option A | Option B | Tradeoff |
|----------|----------|----------|----------|
| Environment structure | HashMap + parent pointer | Indexed environments (Crafting Interpreters) | Simplicity vs O(1) lookups |
| Mutability | `Rc<RefCell<...>>` | Persistent data structures | Rust-idiomatic vs functional purity |
| Closure capture | Capture environment reference | Flat closure with explicit captures | Simple vs memory-efficient |
| Tail calls | Trampolining | CPS transform | Easy implementation vs full continuations |

### Recommended approach for Oxur

**Environment representation:**
```
Frame {
    bindings: HashMap<Symbol, Binding>,
    parent: Option<Rc<RefCell<Frame>>>,
}

Binding {
    value: Value,
    mode: BindingMode,  // Immutable | Mutable | Moved
    borrow_state: BorrowState,
}
```

Use HashMap chains with O(n) lookup through the chain. For Treebeard's ~50k line budget, the simpler implementation wins. Add indexed lookup optimization only if profiling shows environment lookup as a bottleneck.

Implement trampolining from the start—retrofitting TCO is painful. Evaluation returns `enum EvalResult { Value(Value), TailCall { func, args } }` and the main loop handles unwinding.

### Open questions needing prototyping

- Should environments use interned symbols (faster comparison) or Strings (simpler debugging)?
- What's the actual performance of persistent vs mutable environments for typical REPL sessions?
- How does environment capture interact with Rust's move semantics at interpretation time?

---

## 2. Macro expansion in an interpreter

### Summary of findings

Lisp macro systems divide into two camps: **hygienic** (Racket, Scheme) and **unhygienic with conventions** (Common Lisp, Clojure). Racket's approach provides the strongest guarantees—syntax objects track lexical context, ensuring macro-introduced identifiers never accidentally capture or are captured—but at significant implementation cost (syntax objects, phase separation, and complex expansion algorithms).

Clojure demonstrates a pragmatic middle ground: namespace-qualified symbols via syntax-quote (backtick) prevent most capture issues, while explicit `gensym` or `#` suffix handles the remaining cases. This approach works well for a JVM-hosted Lisp and aligns with Oxur's goals.

**Expansion timing** varies: Racket enforces strict phase separation where macros expand before runtime, while traditional Lisp interpreters often interleave expansion and evaluation. For REPL use, lazy expansion (re-expanding on each use) provides better ergonomics when macros are redefined, at the cost of repeated expansion work.

The interaction between macros and REPL is subtle. When a macro is redefined, **previously-expanded code retains the old expansion**. No mainstream system automatically re-expands dependents; SLIME provides `slime-who-macroexpands` to find uses for manual recompilation.

### Key design decisions with tradeoffs

| Decision | Option A | Option B | Tradeoff |
|----------|----------|----------|----------|
| Hygiene | Full (syntax objects) | gensym + namespacing | Safety vs simplicity |
| Expansion timing | Compile-time | Per-use in REPL | Predictability vs REPL flexibility |
| Dependency tracking | None | Invalidate on redefinition | Simple vs correct |
| Macro environment | Shared with runtime | Separate phase | Simple vs powerful |

### Recommended approach for Oxur

Start with **Clojure-style macros**:
- `defmacro` creates compile-time transformers
- Backtick (syntax-quote) auto-qualifies symbols with current namespace
- `gensym` for macro-introduced local bindings
- `macroexpand` and `macroexpand-1` for debugging

Expand at definition time but **store unexpanded forms** alongside expanded for potential re-expansion. Track which macros each expansion used; emit warnings (not automatic re-expansion) when dependencies change.

Since Oxur's AST mirrors syn, macro expansion is essentially syn→syn transformation. Provide quasiquote syntax for constructing AST fragments in macro bodies:

```
(defmacro when [test \u0026 body]
  `(if ~test (do ~@body) nil))
```

### Open questions needing prototyping

- How do Rust's ownership semantics interact with macro expansion? Can macros introduce moves/borrows?
- What's the right error message when expansion produces AST that fails type checking?
- Should macros have access to type information for conditional expansion?

---

## 3. BEAM VM lessons for hot code loading and REPL

### Summary of findings

BEAM provides the most sophisticated hot code loading in production use. Its **two-version system** maintains "current" and "old" code simultaneously—loading new code makes current→old, and a third load terminates processes still in old code. The critical insight: **fully-qualified function calls** (Module:function()) switch to new code, while local calls stay in the current version. This gives programmers explicit control over upgrade timing.

LFE (Lisp Flavored Erlang) demonstrates effective Lisp-on-VM layering: LFE handles syntax, macros, and some pattern matching, while BEAM handles all runtime behavior, code loading, and GC. The REPL inherits hot code loading "for free" because it operates at module granularity.

For REPL state management, **nREPL's session model** provides the best reference: sessions are persistent contexts with unique IDs, maintaining thread-local bindings (`*ns*`, `*1`, `*2`, `*e`), persisting across connections. Common Lisp's image-based development offers complementary insights—the debugger preserves the call stack on error, enabling inspection and even redefinition before continuing.

### Key design decisions with tradeoffs

| Decision | Option A | Option B | Tradeoff |
|----------|----------|----------|----------|
| Redefinition unit | Symbol (CL-style) | Module (BEAM-style) | Flexibility vs controlled updates |
| Version coexistence | None | Two versions | Simplicity vs graceful updates |
| Session state | Global | Per-session | Simple vs multi-REPL support |
| Error handling | Unwind stack | Preserve for inspection | Simple vs powerful debugging |

### Recommended approach for Oxur

**Symbol-level redefinition** (Common Lisp style) fits tree-walking better than BEAM's module-level:
- Functions stored in symbol's function cell
- `defn` replaces immediately; all callers see new definition through late binding
- No explicit versioning initially

**Session model** (nREPL-compatible, since Treebeard already has nREPL protocol):
- Session = persistent evaluation context with unique ID
- Per-session state: current namespace, `*1`/`*2`/`*3` (last values), `*e` (last exception), dynamic bindings
- Operations: clone, close, list-sessions

**Error resilience**: Catch all evaluation errors, return as structured data `{:status :error, :ex ...}`, preserve all definitions from before the error. Bind `*e` to last exception for inspection.

**Interruption**: Check interrupt flag at start of each form evaluation. Use atomic boolean that signal handler can set. On interrupt, throw `InterruptException`, catch at REPL loop, report "Interrupted", continue.

### Open questions needing prototyping

- How does REPL state interact with Rust lifetimes? Can a REPL session hold references across evaluations?
- What granularity should namespace/module use—Rust-style crate/mod or Clojure-style namespace?
- How to handle cyclic dependencies in incremental definition?

---

## 4. Hybrid interpretation/compilation architectures

### Summary of findings

Four systems provide critical patterns for Treebeard's compilation escape hatches:

**Julia** uses a hybrid where simple expressions are interpreted while complex code compiles via LLVM. The decision happens in `jl_toplevel_eval_flex()` using heuristics. Julia's REPL makes compilation transparent—users don't distinguish interpreted from compiled, and introspection macros (`@code_llvm`, `@code_native`) reveal what happened.

**LuaJIT** demonstrates trace compilation: hotcounts track iterations (threshold ~56), hot paths trigger recording, recorded traces compile to native code with guards for speculation. When guards fail, execution "side exits" back to interpreter. Trace linking chains multiple compiled paths together.

**Common Lisp** shows interpreted/compiled duality in language semantics. SBCL compiles everything (even REPL input), while the standard allows mixing. The `compile` function can compile individual lambdas at runtime: `(compile nil '(lambda (x) (* x x)))`.

**GraalVM/Truffle** achieves the most ambitious goal: write a simple AST interpreter, and partial evaluation automatically derives efficient native code. The key mechanism is **Assumptions**—objects tracking compilation invariants that trigger deoptimization when invalidated.

For state sharing between interpreter and compiled code, systems converge on **FFI-compatible representations**. HotSpot's OSR passes an "OSR buffer" with stack state; LuaJIT snapshots restore state on side exits; Truffle uses `VirtualFrame` objects that become scalars after compilation.

### Key design decisions with tradeoffs

| Decision | Option A | Option B | Tradeoff |
|----------|----------|----------|----------|
| When to compile | Explicit `(compile fn)` | Automatic hot paths | Control vs convenience |
| Representation | Boxed values everywhere | Unboxed hot paths | Simplicity vs performance |
| Invalidation | Dependency tracking | Timestamp/version checks | Accuracy vs overhead |
| Compilation target | Rust source → rustc | Direct codegen | Leverage rustc vs compilation speed |

### Recommended approach for Oxur

**Phased implementation:**

**Phase 1—Manual escape hatch:**
- Implement `(compile fn)` special form
- Generate Rust source code from AST
- Invoke rustc with `--crate-type=cdylib`
- Load via `libloading`, call through function pointers

**Phase 2—Automatic compilation:**
- Add call counters to interpreter (increment at function entry)
- Threshold (~200 calls) triggers compilation
- Background compilation (don't block interpreter)

**Calling convention** between interpreted and compiled:
```
// All compiled functions have this signature
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
- Lazy recompilation when threshold hit again

### Open questions needing prototyping

- What's the practical rustc invocation overhead? Does incremental compilation help for REPL use?
- How to handle generic functions—monomorphize per call site or generate dispatch table?
- Can we share some compilation artifacts across REPL sessions?

---

## 5. Runtime ownership and borrowing enforcement (CRITICAL)

### Summary of findings

**This is Oxur's most novel challenge—and research reveals it's tractable.** Miri (Rust's official interpreter) proves runtime ownership tracking works, and the Stacked Borrows (POPL 2020) and Tree Borrows (PLDI 2025) papers provide rigorous operational semantics.

**Miri's approach**: Every allocation gets a unique ID and per-location borrow stack. Each reference carries a unique tag. Operations maintain stack invariants—new borrows push, using a reference pops items above it. Permission states include Unique, SharedRO, SharedRW, and Disabled. Cost: **~1000x slowdown** versus native.

**RefCell's approach**: Two counters per value—shared borrow count and mutable borrow flag. `borrow()` increments shared count (panics if mutable borrowed), `borrow_mut()` sets exclusive flag (panics if any borrows exist). Cost: **~2 integer operations** per borrow.

**Vale's generational references** offer a third path: each allocation has a 64-bit generation number, references store expected generation, dereference checks for match. This catches use-after-free but not aliasing violations.

**The hybrid insight**: Runtime can enforce ownership (moves) and simple borrowing (aliasing violations) cheaply, while complex lifetime analysis is deferred to compilation. This gives **90% of Rust's safety** with **10% of Miri's complexity**.

### Key design decisions with tradeoffs

| Decision | Option A | Option B | Tradeoff |
|----------|----------|----------|----------|
| Ownership tracking | Per-value state | Miri-style per-location | Simple vs complete |
| Borrowing | RefCell-style counters | Full Stacked Borrows | Fast vs precise |
| Lifetime enforcement | Scope-based | Full lifetime inference | Simple vs complete |
| Dangling detection | Generational refs | None | Safety vs overhead |

### Recommended approach for Oxur

**Minimum viable ownership model:**

```
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

**Scope-based lifetime enforcement:**
- Track active borrows per lexical scope
- On scope exit, invalidate borrows by incrementing generation
- Catches most "outlives" violations without full lifetime inference

**What's deferred to compilation:**
- Lifetime parameters (`&'a T`)
- Complex reborrowing patterns
- Field-level borrowing (runtime checks whole values)
- `'static` guarantees

### Open questions needing prototyping

- What's the actual overhead of per-value ownership tracking in typical REPL use?
- How do closures capture ownership? Does `move ||` require copying or can we transfer tracking?
- Can we skip checks for obviously-safe patterns (local values never aliased)?

---

## 6. Environment and binding representation

### Summary of findings

Research confirms the **SICP environment model** remains canonical: environments are chains of frames, frames are symbol→binding mappings. For Treebeard's requirement to represent Rust's different binding modes, each binding must carry metadata beyond just the value.

Persistent data structures (rpds, hamt-rs) offer structural sharing beneficial for functional patterns but add complexity. For mutable REPLs with sequential evaluation, standard `HashMap` with mutation proves simpler and fast enough.

**Shadowing** works naturally with frame chains—inner frames shadow outer frames with same name. **Mutual recursion** requires either forward declarations or letrec semantics (all bindings visible to all bodies simultaneously). Common Lisp uses explicit forward declaration; Scheme's letrec evaluates all bodies in environment containing all bindings (with undefined value initially).

### Recommended approach for Oxur

```
Frame {
    bindings: HashMap<Symbol, Binding>,
    parent: Option<Rc<RefCell<Frame>>>,
}

Binding {
    value: Value,
    mode: Let | LetMut | Static | Const,
    ownership: OwnershipState,
}

Environment {
    current: Rc<RefCell<Frame>>,
    globals: Rc<RefCell<Frame>>,  // Separate for REPL persistence
}
```

For mutual recursion, use **forward declaration** pattern:
1. Define placeholder bindings for all functions in group
2. Create closures capturing environment (which contains placeholders)
3. Replace placeholders with actual closures
4. Closures now reference each other through shared environment

### Open questions needing prototyping

- Should symbol interning be eager (all symbols interned) or lazy (on first lookup)?
- What's the memory overhead of per-binding ownership state for large environments?

---

## 7. REPL state management

### Summary of findings

Clojure's nREPL provides the most complete model for modern REPL state:
- **Sessions**: Persistent contexts with unique IDs, thread bindings, cross-connection state
- **Middleware**: Composable handlers for evaluation, printing, error handling
- **Interruption**: Thread-based—kill evaluation thread, spawn new one (bindings preserved, ThreadLocals lost)

Common Lisp's image-based development shows the power of **preserving state on error**: SLIME's debugger holds the stack open, allows inspection, even allows redefinition before continuing.

Racket's XREPL demonstrates **namespace management**: multiple workspaces, `switch-namespace` for isolation, `enter!` to work inside a module's namespace.

### Recommended approach for Oxur

Since nREPL protocol is already implemented, maintain compatibility:
- Sessions own evaluation contexts
- Per-session: namespace, history (`*1` `*2` `*3`), last error (`*e`), dynamic bindings
- Error isolation: catch all exceptions, return as structured response, preserve definitions
- Interruption: atomic flag checked at form boundaries, throw on interrupt

**History and introspection essentials:**
- `*1`, `*2`, `*3`: Last three values
- `*e`: Last exception
- `(macroexpand '(form))`: Show expansion
- `(type-of x)`: Show inferred type
- `(disassemble fn)`: Show compiled code (when compiled)

---

## 8. Calling external/compiled code (crate loading)

### Summary of findings

Rust-hosted languages (Rhai, Rune, Gluon) demonstrate patterns for calling native Rust:
- **Rhai**: `Engine::register_fn` binds Rust functions, `Dynamic` type for values
- **Rune**: `#[derive(Any)]` exposes types, module system for organization
- **Gluon**: Trait-based marshalling (`VmType`, `Getable`, `Pushable`)

For dynamic loading, **libloading** is the standard: load `.so`/`.dll`/`.dylib`, get symbols, call through function pointers. **Critical**: Use `cdylib` crate type with `extern "C"` functions—Rust's ABI is unstable across compiler versions.

**abi_stable** crate provides FFI-safe std alternatives (`RVec`, `RString`, `RArc`) with runtime type checking via `StableAbi` trait. This handles complex types across the FFI boundary.

### Recommended approach for Oxur

**Architecture:**
```
oxur-interface/     # Shared FFI types (cdylib-safe)
user-crate/         # Dependency being loaded
wrapper-crate/      # Generated: links user-crate, exports via FFI
treebeard/          # Loads wrapper-crate at runtime
```

**Loading flow:**
1. Parse `(require "some-crate")` in Oxur
2. Generate wrapper crate with `[lib] crate-type = ["cdylib"]`
3. Generate shims exporting crate's API via `extern "C"`
4. `Command::new("cargo").args(["build", "--release"])`
5. `libloading::Library::new(&cdylib_path)`
6. Call init function to get function pointer table
7. Register functions in interpreter environment

**Function pointer table:**
```
#[repr(C)]
OxurModule {
    name: *const c_char,
    functions: *const OxurFunction,
    function_count: usize,
}

#[repr(C)]
OxurFunction {
    name: *const c_char,
    fn_ptr: extern "C" fn(*const Value, usize) -> Value,
    // ... type metadata
}
```

### Open questions needing prototyping

- What's the practical overhead of wrapper generation + cargo build for REPL use?
- Can we cache artifacts across REPL restarts?
- How to handle crate features in the loading API?

---

## Architectural sketch

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                                    TREEBEARD                                    │
├────────────────────────────────────────────────────────────────────────────────┤
│  nREPL Server (existing)                                                       │
│    ├── Session Manager (session state, bindings, history)                      │
│    ├── Message Router (ops: eval, interrupt, describe)                         │
│    └── Middleware Stack (print, caught, interruptible-eval)                    │
├────────────────────────────────────────────────────────────────────────────────┤
│  FRONT END                                                                      │
│    ├── Reader (S-expr → syn-like AST)                                          │
│    ├── Macro Expander (defmacro + gensym, tracks dependencies)                 │
│    └── Namespace Manager (symbol tables, imports, current ns)                  │
├────────────────────────────────────────────────────────────────────────────────┤
│  CORE INTERPRETER                                                               │
│    ├── Environment                                                              │
│    │     ├── Frame Chain (HashMap<Symbol, Binding> + parent pointer)           │
│    │     └── Global Frame (persists across evaluations)                        │
│    ├── Evaluator (tree-walking, trampolined for TCO)                           │
│    │     ├── eval_expr(Expr, Env) → EvalResult                                 │
│    │     └── Main loop: while TailCall { recurse without stack growth }        │
│    └── Ownership Tracker                                                        │
│          ├── Per-value: Owned/Moved/Borrowed state                             │
│          ├── Borrow counters (shared count, mutable flag)                      │
│          ├── Generation numbers (dangling detection)                           │
│          └── Scope tracker (invalidate on exit)                                │
├────────────────────────────────────────────────────────────────────────────────┤
│  COMPILATION ESCAPE HATCH                                                       │
│    ├── Hot Path Detector (call counters, threshold ~200)                       │
│    ├── Rust Codegen (AST → Rust source)                                        │
│    ├── rustc Invoker (cargo build --release, cdylib output)                    │
│    ├── Dynamic Loader (libloading, function pointer extraction)                │
│    └── Invalidation Tracker (dependency graph, staleness flags)                │
├────────────────────────────────────────────────────────────────────────────────┤
│  CRATE LOADER                                                                   │
│    ├── Wrapper Generator (shim crates for dependencies)                        │
│    ├── Cargo Invoker (build dependencies as cdylib)                            │
│    ├── Symbol Resolver (function pointer tables)                               │
│    └── Type Bridge (oxur-interface: #[repr(C)] FFI types)                      │
└────────────────────────────────────────────────────────────────────────────────┘
```

**Data flow for `(defn add [a b] (+ a b))` followed by `(add 1 2)`:**
1. Reader: S-expr → AST (DefnExpr with params [a, b] and body (+ a b))
2. Macro Expander: no macros, pass through
3. Evaluator: Create closure, store in current namespace under symbol `add`
4. Reader: `(add 1 2)` → CallExpr
5. Evaluator: Look up `add` → Closure, evaluate args → [1, 2]
6. Ownership Tracker: Check args are Owned, create new bindings in new frame
7. Evaluator: Evaluate body in extended environment → 3
8. Return Value(3) to REPL

**Data flow for compilation escape (after `add` is hot):**
1. Hot Path Detector: `add` call count exceeds 200
2. Rust Codegen: Generate `fn add(a: i64, b: i64) -> i64 { a + b }`
3. rustc Invoker: `cargo build --release` → `libadd.so`
4. Dynamic Loader: `Library::new("libadd.so")`, extract function pointer
5. Update `add` binding: Closure → CompiledFn(fn_ptr)
6. Subsequent calls: Call through fn_ptr instead of tree-walking

---

## Risk assessment

### High risk

**Runtime ownership overhead may be unacceptable.** Even with the simplified RefCell-style model, per-operation checks accumulate. Miri's 1000x slowdown is instructive—the goal of 10-100x requires careful implementation.

*Mitigation*: Implement tiered checking. "Strict mode" for development catches all violations; "fast mode" skips checks for inner loops. Profile early to identify hot spots.

**rustc compilation latency for escape hatches.** Each compile invokes cargo → rustc, typically 1-5 seconds even for trivial functions. This may make automatic hot-path compilation feel sluggish.

*Mitigation*: Compile in background threads, continue interpreting until ready. Cache compilation artifacts across sessions. Consider whole-module compilation batching.

**Semantic drift between interpreted and compiled code.** The interpreter's simplified ownership model may accept programs that rustc rejects (or vice versa).

*Mitigation*: Define a clear "Oxur subset" that both modes support. Test interpreter behavior against Miri for key patterns. Provide clear error messages when compilation reveals issues.

### Medium risk

**Macro redefinition complexity.** Tracking which compiled code depends on which macros, and invalidating correctly, adds significant bookkeeping.

*Mitigation*: Start with no dependency tracking (require manual recompilation). Add tracking incrementally as pain points emerge.

**ABI stability for loaded crates.** If a loaded crate is compiled with a different rustc version, layout mismatches cause UB.

*Mitigation*: Compile all crates with same rustc as Treebeard. Use `abi_stable` for types crossing FFI boundary. Verify layouts at load time where possible.

### Low risk

**Environment lookup performance.** O(n) frame chain walking is theoretically concerning but REPL sessions typically have shallow nesting.

*Mitigation*: Profile before optimizing. If needed, add indexed lookup as enhancement.

**REPL state management complexity.** Session isolation, error recovery, and interruption handling are well-understood patterns.

---

## Suggested prototyping order

### Phase 1: Core interpreter without ownership (2-3 weeks)

**Goal**: Validate tree-walking architecture with basic Lisp semantics.

1. **Reader/Parser**: S-expressions to AST (subset: symbols, numbers, lists, function calls)
2. **Environment**: Frame chains with HashMap, no ownership tracking yet
3. **Evaluator**: Tree-walking for core forms: `def`, `fn`, `let`, `if`, function application
4. **Basic REPL loop**: Read-eval-print, persist definitions across inputs

**Validation**: Can define and call recursive functions (fibonacci, factorial). TCO works (deep recursion doesn't overflow stack).

### Phase 2: Ownership tracking (2-3 weeks)

**Goal**: Prove minimum viable ownership model is practical.

1. **Binding modes**: Add `Let | LetMut | Moved` to bindings
2. **Move semantics**: Use-after-move detection
3. **Borrow tracking**: RefCell-style counters per value
4. **Scope invalidation**: Track borrows per scope, invalidate on exit

**Validation**: Correctly rejects double-mutable-borrow, use-after-move. Accepts valid patterns like sequential borrows.

### Phase 3: Macro system (1-2 weeks)

**Goal**: Enable user-defined syntax.

1. **defmacro**: Store macro transformers in separate table
2. **gensym**: Generate unique symbols
3. **Expansion loop**: Walk AST, expand macro calls, repeat until stable
4. **macroexpand**: Debugging utility

**Validation**: Can implement `when`, `cond`, `->` threading macro.

### Phase 4: Compilation escape hatch (3-4 weeks)

**Goal**: Prove hybrid architecture works.

1. **Rust codegen**: AST → Rust source for simple functions
2. **Cargo invoker**: Shell out to `cargo build`, capture cdylib path
3. **Dynamic loader**: libloading integration
4. **Calling convention**: FFI function pointers with Value marshalling
5. **Manual trigger**: `(compile fn)` special form

**Validation**: A compiled function produces same results as interpreted, with measurable speedup.

### Phase 5: External crate loading (2-3 weeks)

**Goal**: Use real Rust ecosystem from REPL.

1. **Wrapper generation**: Template for shim crate exposing target crate's API
2. **Build pipeline**: cargo build for wrapper crate
3. **Symbol extraction**: Function pointer table loading
4. **Type bridging**: Handle primitives first, then simple structs

**Validation**: Can `(require "regex")` and match patterns from REPL.

### Phase 6: Production hardening (ongoing)

- Automatic hot-path compilation with dependency invalidation
- Full nREPL compatibility
- Error messages with source locations
- Comprehensive test suite comparing interpreter to rustc behavior

---

## Conclusion

Treebeard's design is unprecedented but achievable. **The key insight is layered enforcement**: runtime catches the common ownership violations (use-after-move, aliasing) cheaply, while compilation handles the complex cases (lifetimes, field borrowing). This matches the REPL use case—exploratory code can run with dynamic checks, production code gets full static verification.

The riskiest unknowns center on performance: ownership tracking overhead and compilation latency. Early prototyping should specifically measure these. The highest-value validation is Phase 2 (ownership tracking)—if the minimum viable model proves too slow or imprecise, the architecture needs revision before investing in later phases.

With ~50k lines of budget and the patterns identified here, Treebeard can deliver a genuinely novel development experience: Rust semantics with Lisp's interactive workflow.