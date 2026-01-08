# Oxur Feature Taxonomy Reconciliation Prompt

**Purpose**: Map the current Oxur implementation to the conceptual language feature taxonomy, filling in gaps and confirming what exists.

---

## Context

You have two documents:

1. **Feature Taxonomy** (`crates/design/dev/0007-feature-taxonomy-for-tracking-oxur-progress.md`) — A conceptual framework of language features organized by what *users and language enthusiasts* care about: "Can I write closures?", "Does it have pattern matching?", "How's the macro system?"

2. **Implementation Report** (`crates/design/dev/0008-oxur-progress-report.md`) — A detailed crate-by-crate implementation status that you previously generated. This is useful for contributors but doesn't answer more general, non-developer/non-contributor, user-facing questions about language capabilities.

**The gap**: The taxonomy describes *what a Lisp-on-Rust should have*. The implementation report describes *what modules exist in the codebase*. We need to bridge these.

## Your Task

Create a reconciled version of the feature taxonomy that:

1. **Confirms** which sub-categories from the taxonomy are actually implemented (even partially)
2. **Updates** sub-category names or descriptions to match Oxur's actual implementation approach
3. **Adds** any sub-categories that exist in the code but weren't anticipated in the taxonomy
4. **Removes** or marks as "N/A" any sub-categories that don't apply to Oxur's design
5. **Maps** each sub-category to its implementation location(s) in the codebase

## The 13 Categories to Reconcile

For each category below, you will:

- Read the taxonomy's suggested sub-categories
- Scan the relevant code to see what actually exists
- Produce a reconciled sub-category list with implementation mappings

---

### 1. Reader and S-expressions

**Taxonomy suggests**: Tokenizer/lexer, S-expression parser, Data literals, Quote forms, Reader macros, Source location tracking

**Scan these locations**:

- `crates/oxur-lang/src/parser.rs`
- `crates/oxur-ast/src/lexer.rs`
- `crates/oxur-ast/src/parser.rs`
- `crates/oxur-ast/src/sexp_types.rs`

**Questions to answer**:

- Is there a separate lexer, or is tokenization integrated into the parser?
- What data literals are supported? (numbers, strings, keywords, symbols, vectors, maps, sets)
- Are quote forms (`'`, `` ` ``, `~`, `~@`) implemented?
- Are there reader macros (dispatch characters like `#`)?
- Is source location tracked through parsing?

---

### 2. Symbols and Namespaces

**Taxonomy suggests**: Symbol interning, Namespace resolution, Keyword handling, Qualified symbols, Dynamic variables

**Scan these locations**:

- `crates/oxur-lang/src/` — look for symbol handling
- `crates/oxur-ast/src/sexp_types.rs` — SExp type definitions

**Questions to answer**:

- How are symbols represented? Is there interning?
- Is this a true Lisp-1 implementation (single namespace)?
- Are keywords (`:keyword`) distinct from symbols?
- Is there support for qualified names (`namespace/name`)?
- Any convention for dynamic/special variables (`*earmuffs*`)?

---

### 3. Core Evaluation

**Taxonomy suggests**: Environment/scope model, Special forms, Function application, Tail call optimization, Multiple return values

**Scan these locations**:

- `crates/oxur-lang/src/expander.rs`
- `crates/oxur-lang/src/core_forms.rs`
- `crates/oxur-comp/src/lowering.rs`

**Questions to answer**:

- Is there an explicit environment/scope data structure?
- What special forms are implemented? (`if`, `let`, `fn`, `do`, `def`, `set!`, `deffn`)
- How does function application work?
- Is TCO implemented or planned?
- Multiple return values — via Rust tuples?

---

### 4. Functions and Closures

**Taxonomy suggests**: Lambda definitions, Named functions, Closures, Multi-arity functions, Pattern matching in heads, Guards

**Scan these locations**:

- `crates/oxur-lang/src/core_forms.rs` — DefineFunc, etc.
- `crates/oxur-comp/src/lowering.rs` — function lowering
- `crates/design/docs/` — look for function/syntax design docs

**Questions to answer**:

- Is `lambda` / anonymous functions supported?
- How are named functions defined? (`deffn`?)
- Are closures capturing lexical scope?
- Multi-arity (variadic, optional args)?
- LFE-style pattern matching in function heads?
- Guard clauses (`when`)?

---

### 5. Macro System

**Taxonomy suggests**: defmacro, Macro expansion, Gensym generation, Syntax quoting, Hygiene approach, Compile-time evaluation

**Scan these locations**:

- `crates/oxur-lang/src/expander.rs`
- `crates/oxur-ast/` — macro-related tests
- `crates/design/docs/` — look for macro design docs

**Questions to answer**:

- Is `defmacro` implemented?
- How does macro expansion work?
- Is there gensym for unique symbols?
- Syntax quoting (template construction)?
- Hygiene strategy?
- Can macros run arbitrary code at compile time?

**Note from Letter of Intent**: Core macros are planned for v1.0 (pre-compiled), user macros for v2.0.

---

### 6. Type System and Rust Interop

**Taxonomy suggests**: Runtime type tags, Rust type mapping, Ownership integration, Trait bridging, FFI calls, Type annotations

**Scan these locations**:

- `crates/oxur-ast/` — entire crate (this is the Rust AST bridge)
- `crates/oxur-comp/src/lowering.rs`
- `crates/design/docs/` — look for type/interop design docs

**Questions to answer**:

- How are Lisp values represented at runtime?
- How do Lisp types map to Rust types?
- How is Rust's ownership model exposed? (`borrow`, `move`, `clone`)
- Can Rust traits be implemented from Oxur?
- FFI: Can Oxur call arbitrary Rust functions?
- Optional type annotations in Oxur syntax?

**This is a key differentiator for Oxur** — the bidirectional AST conversion is central.

---

### 7. Collections and Data Structures

**Taxonomy suggests**: Cons cells/lists, Vectors, Hash maps, Sets, Sequences/iterators, Persistent/immutable variants

**Scan these locations**:

- `crates/oxur-lang/` — data literal handling
- `crates/oxur-ast/src/sexp_types.rs`

**Questions to answer**:

- Traditional cons cells, or different list representation?
- Vector literals `[]`?
- Hash map literals `{}`?
- Set literals `#{}`?
- Lazy sequences / iterator protocol?
- Persistent data structures (Clojure-style)?

---

### 8. Error Handling

**Taxonomy suggests**: Exceptions/conditions, Restarts, Result integration, Stack traces, Panic handling

**Scan these locations**:

- `crates/oxur-repl/src/compiler/error_translator.rs`
- `crates/oxur-smap/` — source mapping for errors
- `crates/oxur-comp/` — error handling

**Questions to answer**:

- Traditional exception model, or Rust-style Result?
- Lisp-style restarts/conditions?
- How are Rust `Result<T,E>` values handled in Oxur?
- Stack trace generation?
- How are Rust panics surfaced?

---

### 9. Concurrency

**Taxonomy suggests**: Lightweight processes/actors, Message passing, Pattern matching in receive, Process linking, Rust async bridging

**Scan these locations**:

- `crates/oxur-repl/` — may have async patterns
- `crates/design/docs/` — look for concurrency design

**Questions to answer**:

- Any actor/process model planned? (LFE influence)
- Message passing primitives?
- Erlang-style receive with pattern matching?
- Process linking / supervision?
- Integration with Rust async/await?

**Note**: This may be largely aspirational — check design docs.

---

### 10. Standard Library

**Taxonomy suggests**: List operations, Numeric operations, String operations, I/O operations, System interface

**Scan these locations**:

- `crates/oxur-lang/src/core_forms.rs`
- `crates/oxur-lang/test-data/examples/`

**Questions to answer**:

- Core list ops: `car`, `cdr`, `cons`, `map`, `filter`, `reduce`?
- Numeric: arithmetic, math functions?
- String: manipulation, formatting?
- I/O: file, console?
- System: environment, processes, paths?

**Note**: Much of this may come "for free" via Rust interop.

---

### 11. REPL and Interactive Development

**Taxonomy suggests**: Read-eval-print loop, History and recall, Live code reload, Debugger/inspector, Completion, Tiered evaluation

**Scan these locations**:

- `crates/oxur-repl/` — entire crate
- `crates/design/dev/oxur-repl/` — development docs

**Questions to answer**:

- Core REPL loop working?
- History variables (`*`, `**`, `***`)?
- Hot reload / redefine without restart?
- Debugger / stack inspection?
- Tab completion?
- Tiered evaluation (calculator → cache → JIT)?

**This crate is well-developed** — map carefully.

---

### 12. Tooling and Distribution

**Taxonomy suggests**: Build system, Package manager, Formatter, Documentation generator, Test framework, Language server (LSP)

**Scan these locations**:

- `crates/cargo-oxur/` — Cargo integration
- `crates/oxur-pretty/` — formatter
- `crates/design/` (`oxd`) — documentation tool
- `crates/oxur-cli/` — CLI infrastructure

**Questions to answer**:

- Build system: `cargo-oxur`?
- Package manager: planned?
- Formatter: `oxur-pretty` (what does it format?)
- Doc generator: beyond `oxd` for design docs?
- Test framework: built-in or via Rust?
- LSP: any work started?

---

### 13. Oxur VM (Aspirational) 💭

**Taxonomy suggests**: Bytecode design, Serialization format, Interpreter loop, Garbage collector, Object representation, JIT compilation, Debugger hooks

**Scan these locations**:

- `crates/design/docs/0042-*` — may have VM design doc

**Questions to answer**:

- Is there any VM design documentation?
- Any code started?
- What's the vision?

**Mark all as 0%** unless you find actual implementation work.

---

## Output Format

Create a file called `TAXONOMY-RECONCILED.md` with this structure:

```markdown
# Oxur Feature Taxonomy — Reconciled

*Reconciled: [DATE]*
*Based on: Feature taxonomy (dev/0007) + codebase scan*

## How to Read This Document

This document maps Oxur's conceptual language features to their implementation status.
Each sub-category shows:
- **Status**: ✅ Implemented | 🚧 In Progress | 📋 Planned | 💭 Aspirational | — N/A
- **Location**: Where in the codebase this is implemented
- **Notes**: Implementation approach, deviations from taxonomy, blockers

---

## 1. Reader and S-expressions

*Foundation: parsing source text into data structures*

| Sub-category | Status | Location | Notes |
|--------------|--------|----------|-------|
| Tokenizer/lexer | 🚧 | `oxur-lang/src/parser.rs` | Integrated into parser |
| S-expression parser | ✅ | `oxur-lang/src/parser.rs`, `oxur-ast/src/parser.rs` | Two parsers: Oxur surface + AST S-exp |
| Data literals: numbers | ✅ | ... | Integer, float |
| Data literals: strings | ✅ | ... | ... |
| Data literals: keywords | 📋 | ... | `:keyword` syntax planned |
| ... | ... | ... | ... |

**Category summary**: [Brief prose about overall state]

---

## 2. Symbols and Namespaces

...

[Continue for all 13 categories]

---

## Deviations from Original Taxonomy

List any significant differences between the original taxonomy and Oxur's actual design:

1. **[Category]: [Sub-category]** — [Explanation of deviation]
2. ...

## Discovered Sub-categories

Sub-categories found in the codebase that weren't in the original taxonomy:

1. **[Category]**: [New sub-category] — [What it does]
2. ...

## Recommendations

Based on this reconciliation:

1. [Any taxonomy categories that should be renamed/restructured]
2. [Any implementation gaps that are critical path]
3. [Any sub-categories that should be promoted/demoted]
```

---

## Process

1. **Read the taxonomy document first**: `cat crates/design/dev/0007-feature-taxonomy-for-tracking-oxur-progress.md`

2. **For each category**, scan the relevant code locations listed above

3. **Cross-reference with design docs**: `ls crates/design/docs/` — many ODDs have detailed status

4. **Cross-reference with your previous PROGRESS.md**: Use your implementation knowledge

5. **Be specific about locations**: Use actual file paths, not vague references

6. **Note deviations honestly**: If Oxur does something differently than the taxonomy suggests, explain why

7. **Preserve the 13-category structure**: Even if some are mostly empty/aspirational

---

## Critical Guidance

- **This is about language features, not implementation modules**: "Pattern matching" not "builder/pattern.rs"
- **User-facing perspective**: Would someone evaluating Oxur care about this?
- **Be conservative**: If something is scaffolded but doesn't work end-to-end, mark it 🚧 not ✅
- **Preserve aspirational features**: They show the project's vision
- **The Rust interop story is key**: oxur-ast is the crown jewel — make sure Category 6 reflects this

---

**Run from the project root directory.**
