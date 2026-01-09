# Oxur Feature Progress

*Last updated: 2026-01-07*
*Based on: Taxonomy v2.0 + Claude Code reconciliation*

## Summary by Category

| Category | Status | Notes |
|----------|--------|-------|
| 1. Oxur Reader | 35% | Basic parsing works; quote forms and reader macros missing |
| 2. Rust AST Bridge ⭐ | 95% | Crown jewel — nearly complete |
| 3. Symbols & Namespaces | 20% | Basic types exist; no interning or modules |
| 4. Core Evaluation | 25% | `deffn` works; most special forms missing |
| 5. Functions & Closures | 20% | Named functions only; no lambdas/closures |
| 6. Macro System | 0% | Not started (planned v1.0 for core, v2.0 for user) |
| 7. Type System | 10% | Minimal; relies on Rust interop |
| 8. Collections | 15% | Basic lists only |
| 9. Error Handling | 50% | Source mapping good; Lisp-style conditions missing |
| 10. Concurrency | 0% | Not started (will come via Rust interop) |
| 11. Standard Library | 0% | Not started (will come via Rust interop) |
| 12. REPL | 60% | Infrastructure solid; interaction features missing |
| 13. Tooling | 45% | Formatter complete; LSP and others missing |
| 14. Oxur VM 💭 | 0% | Aspirational |

**Overall Progress: ~30%** (weighted average, excluding aspirational VM)

---

## Legend

| Status | Meaning |
|--------|---------|
| 90-100% | ✅ Complete |
| 75-89% | 🟢 Nearly complete |
| 50-74% | 🚧 In progress |
| 25-49% | 📋 Started |
| 1-24% | 🌱 Early |
| 0% | 💭 Not started / Aspirational |

---

## Detailed Progress

*By category and sub-categories*

| **Category** | **Sub-category** | **Description** | **Status** |
|:-----|:-----|:-----|:-----:|
| **1. Oxur Reader** | | Parsing Oxur's surface syntax into internal representation | **35%** |
| | Tokenizer | Breaking source text into tokens | 70% |
| | S-expression parser | Building nested list structures | 80% |
| | Numeric literals | Integers, floats, ratios, complex | 25% |
| | String literals | Basic strings, multi-line, interpolation | 45% |
| | Symbol literals | Identifiers and their representation | 90% |
| | Keyword literals | `:keyword` self-evaluating forms | 90% |
| | Collection literals | `[]` vectors, `{}` maps, `#{}` sets | 0% |
| | Quote forms | `'quote`, `` `quasiquote ``, `~unquote`, `~@splice` | 0% |
| | Reader macros | Dispatch characters (`#`, `@`, `^`) | 0% |
| | Comments | `;` line comments, `#\| \|#` block comments | 50% |
| | Source positions | Line/column tracking for error messages | 90% |
| **2. Rust AST Bridge ⭐** | | Bidirectional conversion between Rust AST and S-expressions | **95%** |
| | S-expression lexer | Tokenizing canonical S-expression format | 100% |
| | S-expression parser | Parsing S-expressions to SExp tree | 100% |
| | S-expression printer | Pretty-printing SExp back to text | 100% |
| | Items (S-exp → Rust) | Functions, structs, enums, traits, impls | 100% |
| | Expressions (S-exp → Rust) | All Rust expression types | 100% |
| | Statements (S-exp → Rust) | All Rust statement types | 100% |
| | Patterns (S-exp → Rust) | All pattern matching syntax | 100% |
| | Types (S-exp → Rust) | All type syntax including generics | 100% |
| | Generics & Lifetimes | Type parameters, bounds, lifetime annotations | 100% |
| | Attributes | `#[derive]`, `#[cfg]`, etc. | 100% |
| | Items (Rust → S-exp) | Reverse: generating S-expressions from AST | 100% |
| | Expressions (Rust → S-exp) | Reverse direction | 100% |
| | Round-trip verification | Rust → S-exp → Rust produces equivalent code | 95% |
| | CLI tools (aster) | `to-ast`, `to-rust`, `verify` commands | 100% |
| **3. Symbols & Namespaces** | | How names are represented, resolved, and organized | **20%** |
| | Symbol representation | Internal symbol data structure | 80% |
| | Symbol interning | Efficient symbol identity via intern table | 0% |
| | Keyword handling | `:keyword` as distinct self-evaluating type | 80% |
| | Lisp-1 resolution | Single namespace for functions and values | 20% |
| | Qualified symbols | `namespace::name` or `namespace/name` syntax | 0% |
| | Module system | Organizing code into modules | 0% |
| | Imports/exports | `use`, visibility controls | 0% |
| | Dynamic variables | `*earmuff*` convention, dynamic scope | 0% |
| **4. Core Evaluation** | | The fundamental evaluation semantics | **25%** |
| | Environment model | Lexical scoping, environment data structure | 10% |
| | Special form: `if` | Conditional evaluation | 20% |
| | Special form: `let` | Local bindings | 0% |
| | Special form: `do` | Sequential evaluation | 0% |
| | Special form: `def` | Top-level definitions | 25% |
| | Special form: `set!` | Mutation | 0% |
| | Function application | Calling functions with arguments | 50% |
| | Tail call optimization | Proper tail recursion | 0% |
| | Multiple values | Returning multiple values (via Rust tuples) | 0% |
| **5. Functions & Closures** | | Defining and using functions | **20%** |
| | Named functions (`deffn`) | Top-level function definitions | 60% |
| | Anonymous functions (`fn`) | Lambda expressions | 0% |
| | Closures | Capturing lexical environment | 0% |
| | Parameter syntax | Basic parameters | 45% |
| | Type-annotated parameters | `name:type` syntax | 0% |
| | Return type annotations | `-> type` syntax | 0% |
| | Multi-arity | Functions with multiple arities | 0% |
| | Variadic functions | Rest parameters (`& rest`) | 0% |
| | Pattern matching in heads | LFE-style clause dispatch | 0% |
| | Guards | `when` constraints on function clauses | 0% |
| | Docstrings | Documentation attached to functions | 0% |
| **6. Macro System** | | Compile-time metaprogramming | **0%** |
| | `defmacro` | Defining macros | 0% |
| | Macro expansion | Recursive expansion at compile time | 0% |
| | Quote (`'`) | Preventing evaluation | 0% |
| | Quasiquote (`` ` ``) | Template construction | 0% |
| | Unquote (`~`) | Inserting values into templates | 0% |
| | Splice (`~@`) | Splicing lists into templates | 0% |
| | Gensym | Generating unique symbols | 0% |
| | Hygiene | Avoiding unintended capture | 0% |
| | Compile-time evaluation | Running code during compilation | 0% |
| | Core macros | Pre-compiled standard macros | 0% |
| **7. Type System** | | Oxur's flavour of Rust type semantics | **10%** |
| | Runtime type tags | Dynamic type discrimination | 20% |
| | Type predicates | `number?`, `string?`, `list?`, etc. | 0% |
| | Type annotations | Optional static type hints in Oxur syntax | 0% |
| | Type inference | Inferring types from context | 30% |
| | Ownership syntax | Expressing `borrow`, `move`, `clone` in Oxur | 0% |
| | Lifetime syntax | Expressing lifetimes in Oxur | 0% |
| | Trait syntax | Defining/implementing traits in Oxur | 0% |
| | Generic functions | Parametric polymorphism | 0% |
| **8. Collections** | | Built-in data structures | **15%** |
| | Cons cells / Lists | Traditional Lisp linked lists | 40% |
| | Vectors | Indexed sequential collections (`[]` syntax) | 0% |
| | Hash maps | Key-value associations (`{}` syntax) | 0% |
| | Sets | Unique element collections (`#{}` syntax) | 0% |
| | Sequences | Lazy sequence abstraction | 0% |
| | Iterators | Rust iterator integration | 0% |
| | Persistent variants | Immutable with structural sharing | 0% |
| **9. Error Handling** | | How errors are signaled and handled | **50%** |
| | Result integration | Working with Rust `Result<T, E>` | 30% |
| | Option integration | Working with Rust `Option<T>` | 30% |
| | `?` operator | Error propagation syntax | 0% |
| | Panic handling | Catching Rust panics | 40% |
| | Conditions (Lisp-style) | Signaling conditions | 0% |
| | Restarts (Lisp-style) | Resumable error handling | 0% |
| | Stack traces | Readable error context | 60% |
| | Source mapping | Errors point to Oxur source, not generated Rust | 85% |
| **10. Concurrency** | | Writing concurrent and parallel code | **0%** |
| | Async functions | `async`/`await` syntax | 0% |
| | Futures | Working with Rust futures | 0% |
| | Threads | `std::thread` integration | 0% |
| | Channels | Message passing via channels | 0% |
| | Actors | LFE-style lightweight processes | 0% |
| | Message passing | Send (`!`) and `receive` | 0% |
| | Process linking | Supervision trees | 0% |
| | Shared state | `Mutex`, `Arc`, etc. | 0% |
| **11. Standard Library** | | Built-in functions and utilities | **0%** |
| | List operations | `car`, `cdr`, `cons`, `map`, `filter`, `reduce`, etc. | 0% |
| | Numeric operations | Arithmetic, math functions | 0% |
| | String operations | Manipulation, formatting, conversion | 0% |
| | I/O operations | File, console, streams | 0% |
| | System interface | Environment, processes, paths | 0% |
| | Rust stdlib access | Seamless access to `std::*` | 0% |
| **12. REPL & Interactive Dev** | | The interactive programming experience | **60%** |
| | Basic REPL loop | Read, eval, print, loop | 70% |
| | Multi-line input | Handling incomplete expressions | 100% |
| | History | Command recall, `*`, `**`, `***` | 0% |
| | Command Completion | Tab completion for non-language, REPL-specific commands | 100% |
| | Symbol Completion | Tab completion for symbols | 0% |
| | Live reload | Redefine functions without restart | 0% |
| | Inspector | Examining values interactively | 0% |
| | Debugger | Stepping, breakpoints | 0% |
| | Tiered evaluation | Calculator → Cache → JIT strategy | 40% |
| | Artifact caching | SHA256-keyed compilation cache | 95% |
| | Error display | Friendly error formatting | 80% |
| | Server/client | Network REPL protocol | 90% |
| | Session management | Multiple isolated sessions | 95% |
| | REPL stats | Metrics for the REPL as a whole | 95% |
| **13. Tooling & Distribution** | | Developer tools and ecosystem | **45%** |
| | Build system (`cargo-oxur`) | Compiling Oxur projects | 25% |
| | Project structure | `Oxur.toml` configuration | 0% |
| | Formatter (`oxur-pretty`) | Code formatting | 95% |
| | Linter | Style and error checking | 0% |
| | Documentation generator | Generating docs from docstrings | 0% |
| | Test framework | Writing and running tests | 0% |
| | Package manager | Dependencies and distribution | 0% |
| | Language server (LSP) | Editor integration | 0% |
| | Debugger integration | IDE debugging support | 0% |
| **14. Oxur VM 💭** | | Native bytecode interpreter (aspirational) | **0%** |
| | Bytecode design | Opcode set, instruction format | 0% |
| | Serialization | Binary bytecode format | 0% |
| | Interpreter loop | Dispatch mechanism | 0% |
| | Object representation | Runtime value layout | 0% |
| | Garbage collector | Memory management | 0% |
| | JIT compilation | Hot path optimization | 0% |
| | Debugger hooks | VM-level debugging | 0% |

---
