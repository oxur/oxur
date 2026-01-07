# Feature taxonomy for tracking a Lisp-on-Rust language

Oxur's unique position—a Lisp-1 compiling to Rust with Zetalisp and LFE influences—requires a tracking taxonomy that spans three domains: **language implementation mechanics**, **Lisp-specific semantics**, and **Rust interoperability**. The recommended structure uses **12 top-level categories** with **3-7 sub-categories each**, balancing meaningful progress granularity against at-a-glance readability.

## Recommended category structure

The categories below are organized by implementation priority (core → tooling → aspirational), with Lisp-specific features elevated to first-class status rather than buried under generic compiler terminology.

### 1. Reader and S-expressions
The foundation of any Lisp—parsing source text into data structures.

| Sub-category | Tracks |
|--------------|--------|
| Tokenizer/lexer | Character stream → tokens |
| S-expression parser | Nested list construction |
| Data literals | Numbers, strings, keywords, symbols, vectors, maps, sets |
| Quote forms | `'quote`, `` `quasiquote ``, `~unquote`, `~@splice` |
| Reader macros | Dispatch characters (`#`, `@`, `^`) |
| Source location tracking | Span info for error messages |

### 2. Symbols and namespaces
Lisp-1's single namespace requires careful symbol handling.

| Sub-category | Tracks |
|--------------|--------|
| Symbol interning | Unique symbol identity |
| Namespace resolution | Lisp-1 single namespace lookup |
| Keyword handling | Self-evaluating `:keyword` forms |
| Qualified symbols | `namespace/name` resolution |
| Dynamic variables | Special variable declarations (`*var*` convention) |

### 3. Core evaluation
The semantic heart—how expressions become values.

| Sub-category | Tracks |
|--------------|--------|
| Environment/scope model | Lexical scoping with closures |
| Special forms | `if`, `let`, `fn`, `do`, `def`, `set!` |
| Function application | First-class functions, higher-order calls |
| Tail call optimization | Proper tail recursion |
| Multiple return values | Destructuring returns |

### 4. Functions and closures
Lisp-1's unified function/value treatment is central.

| Sub-category | Tracks |
|--------------|--------|
| Lambda definitions | Anonymous function creation |
| Named functions | `defn` with docstrings |
| Closures | Lexical environment capture |
| Multi-arity functions | Variable argument counts |
| Pattern matching in heads | LFE-style clause dispatch |
| Guards | `when` constraints on clauses |

### 5. Macro system
Homoiconicity powers Lisp's extensibility.

| Sub-category | Tracks |
|--------------|--------|
| `defmacro` | Basic macro definitions |
| Macro expansion | Recursive expansion phases |
| Gensym generation | Unique symbol creation |
| Syntax quoting | Template construction with `\`` |
| Hygiene approach | Capture avoidance strategy |
| Compile-time evaluation | Macro-time computation |

### 6. Type system and Rust interop
Bridging Lisp dynamism with Rust's static types.

| Sub-category | Tracks |
|--------------|--------|
| Runtime type tags | Dynamic type discrimination |
| Rust type mapping | Lisp values ↔ Rust types |
| Ownership integration | How Lisp values interact with Rust borrowing |
| Trait bridging | Exposing Rust traits to Lisp |
| FFI calls | Calling Rust functions from Lisp |
| Type annotations | Optional static type hints |

### 7. Collections and data structures
Standard Lisp and extended collection types.

| Sub-category | Tracks |
|--------------|--------|
| Cons cells/lists | Traditional linked lists |
| Vectors | Indexed sequential access |
| Hash maps | Key-value associations |
| Sets | Unique element collections |
| Sequences/iterators | Lazy sequence protocol |
| Persistent/immutable variants | Structural sharing (Clojure-style) |

### 8. Error handling
Merging Lisp conditions with Rust's Result model.

| Sub-category | Tracks |
|--------------|--------|
| Exceptions/conditions | Throwable error types |
| Restarts | Resumable error handling |
| Result integration | Rust `Result<T,E>` interop |
| Stack traces | Readable error context |
| Panic handling | Unrecoverable errors |

### 9. Concurrency (LFE-influenced)
Actor-style concurrency inherited from LFE's Erlang model.

| Sub-category | Tracks |
|--------------|--------|
| Lightweight processes | Spawn/actor creation |
| Message passing | Send (`!`) and `receive` |
| Pattern matching in receive | Erlang-style mailbox dispatch |
| Process linking | Supervisor relationships |
| Rust async bridging | Integration with Rust async/await |

### 10. Standard library
The built-in function repertoire.

| Sub-category | Tracks |
|--------------|--------|
| List operations | `car`, `cdr`, `cons`, `map`, `filter`, `reduce` |
| Numeric operations | Arithmetic, math functions |
| String operations | Manipulation, formatting |
| I/O operations | File, console, streams |
| System interface | Environment, processes, paths |

### 11. REPL and interactive development
The interactive experience differentiating Lisp from batch-compiled languages.

| Sub-category | Tracks |
|--------------|--------|
| Read-eval-print loop | Core REPL functionality |
| History and recall | `*`, `**`, `***` variables |
| Live code reload | Redefine without restart |
| Debugger/inspector | Stack inspection, variable examination |
| Completion | Symbol and path completion |

### 12. Tooling and distribution
Developer experience and ecosystem support.

| Sub-category | Tracks |
|--------------|--------|
| Build system | Compilation orchestration |
| Package manager | Dependencies, versioning |
| Formatter | Code style enforcement |
| Documentation generator | Docstring extraction |
| Test framework | Unit test discovery and execution |
| Language server (LSP) | Editor integration |

---

## Aspirational: Oxur VM

If the VM is a future goal, track it as a separate section with clear "aspirational" status.

| Sub-category | Tracks |
|--------------|--------|
| Bytecode design | Opcode set, instruction format |
| Serialization format | Binary encoding, versioning |
| Interpreter loop | Dispatch mechanism (switch/threaded) |
| Garbage collector | Algorithm (mark-sweep, generational) |
| Object representation | Tagged pointers, headers |
| JIT compilation | Hot path optimization (future) |
| Debugger hooks | Breakpoints, stepping |

---

## Granularity guidelines

The balance between too-coarse and too-fine tracking determines whether the table provides actionable insight or becomes maintenance overhead.

**Recommended principles:**

- **Feature = independently testable unit**: If a sub-category can have its own test suite, it's the right granularity. "Quote forms" is testable; "handles parentheses" is too fine.
- **5-7 sub-categories per category**: Fewer than 3 suggests the category should merge with another; more than 8 suggests splitting.
- **Completion % thresholds matter**: Define what 25%, 50%, 75%, 100% mean. For example: 25% = basic cases work; 50% = most common patterns; 75% = edge cases and errors; 100% = production-ready with documentation.
- **Test coverage % signals confidence**: Track this separately from code completion—high completion with low test coverage indicates technical debt.
- **Mark aspirational clearly**: Use visual separation (different section, gray styling, or 📋/💭 icons) for features that aren't actively being implemented.

**Visual status vocabulary:**

| Symbol | Meaning |
|--------|---------|
| ✓ or 🟢 | Complete (90%+) |
| 🚧 or 🟡 | In progress |
| 📋 | Planned/started |
| 💭 | Aspirational/future |
| — | Not applicable |

---

## Example table structure

```
┌─────────────────────────────────────┬──────────────┬─────────────────┐
│ Category → Sub-category             │ Code Complete│ Testing Coverage│
├─────────────────────────────────────┼──────────────┼─────────────────┤
│ **Reader and S-expressions**        │              │                 │
│   Tokenizer/lexer                   │ 95%          │ 88%             │
│   S-expression parser               │ 90%          │ 85%             │
│   Data literals                     │ 80%          │ 70%             │
│   Quote forms                       │ 100%         │ 95%             │
│   Reader macros                     │ 40%          │ 30%             │
│   Source location tracking          │ 75%          │ 60%             │
├─────────────────────────────────────┼──────────────┼─────────────────┤
│ **Macro system**                    │              │                 │
│   defmacro                          │ 85%          │ 80%             │
│   Macro expansion                   │ 70%          │ 65%             │
│   ...                               │              │                 │
└─────────────────────────────────────┴──────────────┴─────────────────┘
```

---

## Sources of inspiration

Several projects demonstrate effective feature tracking patterns:

- **Rust RFC tracking**: Uses GitHub issues with standardized labels (`B-RFC-approved`, `S-tracking-impl-incomplete`) and checkbox progressions from RFC → implementation → stabilization
- **GCC C++ status pages**: HTML tables grouped by standard version with version numbers indicating when features shipped
- **PostgreSQL feature matrix**: 26 categories with checkmarks showing version introduction, filterable by version range
- **"Are we X yet" sites**: Simple status indicators (Yes/Almost/No) with links to relevant libraries

The PostgreSQL model—categories with sub-features, visual indicators, and version tracking—most closely matches Oxur's needs for a two-level hierarchy with percentage columns.

---

## Final recommendations

For a Lisp-1 on Rust with Zetalisp/LFE influences, the **12 categories above** provide complete coverage:

1. **Reader and S-expressions** — Lisp's textual foundation
2. **Symbols and namespaces** — Lisp-1's core semantic model  
3. **Core evaluation** — Expression semantics
4. **Functions and closures** — First-class functions
5. **Macro system** — Homoiconic metaprogramming
6. **Type system and Rust interop** — Bridging dynamic Lisp with static Rust
7. **Collections and data structures** — Built-in data types
8. **Error handling** — Conditions, restarts, and Rust Result integration
9. **Concurrency** — LFE-style actors and Rust async
10. **Standard library** — Built-in functions
11. **REPL and interactive development** — Live coding experience
12. **Tooling and distribution** — Developer experience

Add **Oxur VM** as a 13th aspirational category when the project matures. This structure provides **~60 trackable sub-features**—enough granularity for meaningful progress indication without overwhelming the at-a-glance view.