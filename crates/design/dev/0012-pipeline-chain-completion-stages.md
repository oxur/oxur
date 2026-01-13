# Pipeline Chain Completion - Stage Breakdown

**Date:** 2026-01-12
**Purpose:** Stage-by-stage breakdown of dev plan 0011 for context-sized work sessions
**Status:** Work breakdown structure for implementation

---

## About This Document

This document breaks each phase from `0011-pipeline-chain-completion.md` into **stages** - small, focused units of work that can be completed in a single session (potentially with 1-2 compactions).

**Stage Characteristics:**
- Single clear deliverable
- Fits in one context window
- Can be tested independently
- Has clear entry/exit criteria
- Takes 2-4 hours of focused work

---

## Phase 1: Source Mapping (2 weeks)

### Week 1: Position Tracking in Parser

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 1.1 | Span types | Define `Span`, `SourcePos` structs in oxur-smap | 1h | None | Types compile, basic tests pass |
| 1.2 | SurfaceForm with Span | Update all SurfaceForm variants to include Span | 2h | 1.1 | All parser tests pass with new structure |
| 1.3 | Position tracking foundation | Implement `current_position()` in Parser | 2h | 1.2 | Line/column calculation correct |
| 1.4 | Position tracking in parse methods | Update `parse_form()`, `parse_list()`, `parse_atom()` to record spans | 3h | 1.3 | All forms have accurate spans |
| 1.5 | Position tracking tests | Test suite for span accuracy | 2h | 1.4 | 100% test coverage for position tracking |

### Week 2: Mapping Chains Through Pipeline

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 1.6 | SourceMap recording API | Implement `record_span()`, `record_transform()` in SourceMap | 2h | 1.5 | API documented and tested |
| 1.7 | Surface → Core mapping | Update Expander to record mappings | 3h | 1.6 | All Core Forms have source provenance |
| 1.8 | Core → syn mapping | Update Lowerer to record mappings (temporary until Stage 4) | 3h | 1.7 | Mappings preserved through lowering |
| 1.9 | rustc diagnostic parser | Parse rustc JSON output to extract positions | 2h | None | Can extract file:line:col from rustc errors |
| 1.10 | Error translator | Implement `translate_rustc_error()` | 3h | 1.8, 1.9 | Errors point to Oxur source |
| 1.11 | Error translation tests | End-to-end error reporting tests | 2h | 1.10 | Error messages show Oxur source context |

---

## Phase 2: Stage 4 Integration (3 weeks)

### Week 1: Update Stage 3 to Output S-expressions

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 2.1 | S-expression builders design | Design API for building Oxur AST S-expressions | 1h | None | API documented |
| 2.2 | Change Lowerer signature | Update `lower()` return type to `Result<Vec<SExp>>` | 1h | 2.1 | Compiles (will break tests) |
| 2.3 | Implement Item S-expression builder | `lower_function_to_sexp()` for DefineFunc | 3h | 2.2 | Generates `(Item :kind (Fn ...))` |
| 2.4 | Implement Expr S-expression builders | S-expression builders for expressions | 3h | 2.3 | All expression types covered |
| 2.5 | Implement Stmt S-expression builders | S-expression builders for statements | 2h | 2.4 | Macro calls work |
| 2.6 | Verify S-expression format | Compare output against ODD-0003 spec | 2h | 2.5 | Format matches specification |
| 2.7 | Update tests for S-expression output | Fix all broken tests | 2h | 2.6 | All oxur-comp tests pass |

### Week 2: Create Stage 4 Processor

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 2.8 | Create de_sexpr module | New file `oxur-comp/src/de_sexpr.rs` | 1h | 2.7 | Module structure in place |
| 2.9 | DeSExprProcessor struct | Basic processor using oxur_ast::Builder | 2h | 2.8 | Processor compiles |
| 2.10 | Integrate into Compiler | Wire Stage 4 into compilation pipeline | 2h | 2.9 | Pipeline: Stage 3 → Stage 4 → Stage 5 |
| 2.11 | End-to-end compilation test | Test Oxur source → binary via new pipeline | 2h | 2.10 | Hello world compiles and runs |
| 2.12 | Regression tests | Verify all existing examples still work | 2h | 2.11 | No functionality regressions |
| 2.13 | Performance comparison | Benchmark vs. old direct-to-syn approach | 1h | 2.12 | Document any performance changes |

### Week 3: Remove syn Dependency from oxur-comp

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 2.14 | Update Cargo.toml | Remove syn/quote/proc-macro2, add oxur-ast | 1h | 2.13 | oxur-comp compiles without syn |
| 2.15 | Remove unused imports | Clean up all syn imports from oxur-comp | 1h | 2.14 | Clippy clean |
| 2.16 | Verify dependency isolation | Check dependency tree | 1h | 2.15 | Only oxur-ast depends on syn |
| 2.17 | Update buffer zone docs | Document architecture in design docs | 2h | 2.16 | Architecture clearly explained |
| 2.18 | Create architectural diagram | Visual representation of buffer zone | 1h | 2.17 | Diagram in docs/ |

---

## Phase 3: Core Forms Expansion (3 weeks)

### Week 1: Operators and Calls

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 3.1 | Define operator Core Forms | Add BinaryOp, UnaryOp to CoreForm enum | 2h | None | Types compile |
| 3.2 | Define call Core Forms | Add Call, MethodCall to CoreForm enum | 1h | 3.1 | Types compile |
| 3.3 | Expand binary operators | Recognize `+`, `-`, `*`, `/` in Expander | 3h | 3.2 | `(+ 1 2)` → BinaryOp |
| 3.4 | Expand function calls | Recognize `(func args...)` in Expander | 3h | 3.3 | `(add 1 2)` → Call |
| 3.5 | Expand method calls | Recognize `(obj:method args...)` in Expander | 2h | 3.4 | `(x:pow 2)` → MethodCall |
| 3.6 | Lower BinaryOp to S-expressions | Generate Oxur AST for operators | 3h | 3.5 | Operators lower correctly |
| 3.7 | Lower Call to S-expressions | Generate Oxur AST for calls | 2h | 3.6 | Function calls lower correctly |
| 3.8 | Lower MethodCall to S-expressions | Generate Oxur AST for method calls | 2h | 3.7 | Method calls lower correctly |
| 3.9 | Operator tests | Test arithmetic operations end-to-end | 2h | 3.8 | Can compile `(+ 1 2)` |
| 3.10 | Call tests | Test function calls end-to-end | 2h | 3.9 | Can compile `(add 1 2)` |
| 3.11 | Integration test | Compile function with operators | 2h | 3.10 | `(deffn add (a b) (+ a b))` works |

### Week 2: Local Bindings and Type Annotations

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 3.12 | Define binding Core Forms | Add Let, Def to CoreForm enum | 2h | 3.11 | Types compile |
| 3.13 | Define Type struct | Create Type representation for annotations | 2h | 3.12 | Type struct defined |
| 3.14 | Parse type annotations | Recognize `name:type` syntax in Parser | 3h | 3.13 | `x:i32` parsed correctly |
| 3.15 | Expand let bindings | Implement `expand_let()` | 3h | 3.14 | `(let ((x 42)) ...)` → Let |
| 3.16 | Expand def | Implement `expand_def()` | 2h | 3.15 | `(def x:i32 42)` → Def |
| 3.17 | Update DefineFunc for typed params | Store parameter types in DefineFunc | 2h | 3.16 | `(deffn f (x:i32) ...)` stores type |
| 3.18 | Lower Let to S-expressions | Generate Oxur AST for let bindings | 3h | 3.17 | Let bindings lower correctly |
| 3.19 | Lower Def to S-expressions | Generate Oxur AST for def | 2h | 3.18 | Def lowers correctly |
| 3.20 | Lower typed parameters | Include type annotations in lowered functions | 2h | 3.19 | Types preserved in Rust output |
| 3.21 | Type annotation tests | Test typed functions compile | 2h | 3.20 | `(deffn add (a:i32 b:i32) ...)` works |

### Week 3: Conditionals and Pattern Matching

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 3.22 | Expand IfExpr | Implement `expand_if()` (currently stubbed) | 3h | 3.21 | `(if cond then else)` → IfExpr |
| 3.23 | Expand MatchExpr | Implement `expand_match()` (currently stubbed) | 3h | 3.22 | `(match x ...)` → MatchExpr |
| 3.24 | Lower IfExpr to S-expressions | Generate Oxur AST for conditionals | 3h | 3.23 | If expressions lower correctly |
| 3.25 | Lower MatchExpr to S-expressions | Generate Oxur AST for pattern matching | 3h | 3.24 | Match expressions lower correctly |
| 3.26 | Conditional tests | Test if expressions end-to-end | 2h | 3.25 | `(if (> x 0) "pos" "neg")` works |
| 3.27 | Pattern matching tests | Test match expressions end-to-end | 2h | 3.26 | `(match x (Some v) v None 0)` works |
| 3.28 | Integration test: Fibonacci | Compile recursive Fibonacci function | 2h | 3.27 | Full example with all features works |

---

## Phase 5: Core Macros Library (2 weeks)

### Week 1: Core Macro Framework

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 5.1 | Design macro registry | Design MacroRegistry and CoreMacro trait | 2h | None | API documented |
| 5.2 | Implement macro registry | Create registry infrastructure | 2h | 5.1 | Registry works |
| 5.3 | Integrate registry with Expander | Wire macro lookup into expansion | 2h | 5.2 | Expander uses registry |
| 5.4 | Implement `when` macro | `when` → if without else | 2h | 5.3 | `(when cond body)` works |
| 5.5 | Implement `unless` macro | `unless` → negated if | 1h | 5.4 | `(unless cond body)` works |
| 5.6 | Implement `cond` macro | `cond` → multi-way conditional | 3h | 5.5 | `(cond ...)` works |
| 5.7 | Macro tests | Test each macro independently | 2h | 5.6 | All macros pass tests |
| 5.8 | Nested macro tests | Test macro composition | 2h | 5.7 | Nested macros work |

### Week 2: Threading and Let Variants

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 5.9 | Implement `->` macro | Thread-first transformation | 3h | 5.8 | `(-> x f g h)` works |
| 5.10 | Implement `->>` macro | Thread-last transformation | 2h | 5.9 | `(->> x f g h)` works |
| 5.11 | Implement `as->` macro | Thread-as transformation | 2h | 5.10 | `(as-> x $ ...)` works |
| 5.12 | Implement `when-let` macro | Conditional binding | 2h | 5.11 | `(when-let [x val] ...)` works |
| 5.13 | Implement `if-let` macro | Conditional with else | 2h | 5.12 | `(if-let [x val] ...)` works |
| 5.14 | Threading tests | Test all threading macros | 2h | 5.13 | Threading transformations correct |
| 5.15 | Let variant tests | Test all let variants | 2h | 5.14 | Let variants work |
| 5.16 | Document core macros | Write usage guide for all macros | 2h | 5.15 | Documentation complete |

---

## Phase 6: REPL Subprocess IPC (2 weeks)

### Week 1: Design and Implement IPC Protocol

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 6.1 | Design IPC message format | Define SubprocessMessage enum | 2h | None | Protocol documented |
| 6.2 | Implement message serialization | JSON or postcard encoding | 2h | 6.1 | Messages serialize correctly |
| 6.3 | Implement stdin/stdout communication | Message send/receive over pipes | 3h | 6.2 | Bidirectional communication works |
| 6.4 | Update SubprocessExecutor::load_library | Implement library loading via IPC | 3h | 6.3 | Can load dynamic libraries |
| 6.5 | Update SubprocessExecutor::run_function | Implement function execution via IPC | 3h | 6.4 | Can execute functions |
| 6.6 | Implement subprocess binary | Create src/bin/subprocess.rs | 3h | 6.5 | Subprocess receives and processes messages |
| 6.7 | Handle subprocess crashes | Graceful error handling | 2h | 6.6 | Crashes don't hang REPL |
| 6.8 | Implement Ctrl-C support | SIGTERM/SIGKILL handling | 2h | 6.7 | Ctrl-C kills subprocess |
| 6.9 | IPC tests | Test message protocol | 2h | 6.8 | All message types work |

### Week 2: Integration with REPL Tiers

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 6.10 | Wire Tier 2 into EvalContext | Use cached library + subprocess | 3h | 6.9 | Tier 2 execution works |
| 6.11 | Wire Tier 3 into EvalContext | Compile + load + execute | 3h | 6.10 | Tier 3 execution works |
| 6.12 | Test tier selection logic | Verify correct tier chosen | 2h | 6.11 | Tiers selected correctly |
| 6.13 | Test cache hit/miss flow | Verify Tier 2 on cache hit | 2h | 6.12 | Caching works |
| 6.14 | Performance benchmarking | Measure Tier 1/2/3 latencies | 2h | 6.13 | Meets performance targets |
| 6.15 | End-to-end REPL test | Full REPL session test | 2h | 6.14 | Can define and call functions |
| 6.16 | REPL documentation | Document three-tier architecture | 2h | 6.15 | Documentation complete |

---

## Phase 7: CLI & Tooling (1 week)

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 7.1 | Audit CLI flags | Review all command-line options | 1h | Phase 6 complete | Flags documented |
| 7.2 | Add missing CLI features | Implement any critical missing features | 3h | 7.1 | CLI feature-complete |
| 7.3 | Improve error messages | Polish error output formatting | 2h | 7.2 | Errors are clear and helpful |
| 7.4 | Add progress indicators | Show compilation progress | 2h | 7.3 | User sees what's happening |
| 7.5 | CLI documentation | Update --help and user guide | 2h | 7.4 | CLI documented |
| 7.6 | CLI tests | Test all CLI commands | 2h | 7.5 | All commands work |

---

## Phase 8: v1.0 Release (2 weeks)

| Stage | Deliverable | Features/Issues | Est. | Dependencies | Success Criteria |
|-------|-------------|-----------------|------|--------------|------------------|
| 8.1 | Audit documentation | Review all docs for accuracy | 3h | Phase 7 complete | Docs up to date |
| 8.2 | Write tutorial | Getting started guide | 4h | 8.1 | Tutorial works |
| 8.3 | Create example programs | 5-10 example programs | 4h | 8.2 | Examples compile and run |
| 8.4 | Write architecture guide | Deep dive on compilation pipeline | 3h | 8.3 | Architecture explained |
| 8.5 | Performance benchmarks | Benchmark vs. other compilers | 3h | 8.4 | Benchmarks documented |
| 8.6 | Write release notes | Changelog and features | 2h | 8.5 | Release notes complete |
| 8.7 | Package for distribution | Create release artifacts | 2h | 8.6 | Binaries available |
| 8.8 | Write blog post | Announce v1.0 | 3h | 8.7 | Blog post ready |
| 8.9 | Final testing | Smoke test all features | 3h | 8.8 | Everything works |
| 8.10 | Release! | Tag v1.0 and publish | 1h | 8.9 | v1.0 released 🎉 |

---

## Usage Notes

**For Implementation:**
1. Each stage should be treated as a separate work session
2. Before starting a stage, read its dependencies
3. After completing a stage, verify success criteria
4. Document any deviations or issues discovered

**For Tracking:**
- Use checkboxes: `- [ ]` for incomplete, `- [x]` for complete
- Add notes for any stage that takes longer than estimated
- Track blockers and dependencies carefully

**For AI Assistants:**
- Each stage should fit comfortably in context
- Dependencies ensure stages are done in order
- Success criteria provide clear completion signals
- Estimated times help with planning

---

## Summary Statistics

| Phase | Stages | Total Est. | Focus Area |
|-------|--------|------------|------------|
| Phase 1 | 11 | 25h | Source mapping and error translation |
| Phase 2 | 18 | 31h | Stage 4 integration and buffer zone |
| Phase 3 | 28 | 62h | Core Forms expansion |
| Phase 5 | 16 | 32h | Core macro library |
| Phase 6 | 16 | 37h | REPL subprocess IPC |
| Phase 7 | 6 | 12h | CLI polish |
| Phase 8 | 10 | 28h | Release preparation |
| **Total** | **105** | **227h** | **~15 weeks at 15h/week** |

---

## Next Steps

1. **Review this breakdown** - Does stage granularity feel right?
2. **Start with Stage 1.1** - Define Span and SourcePos types
3. **Create tracking system** - GitHub project, issues, or checkboxes in this doc
4. **Begin implementation** - One stage at a time!
