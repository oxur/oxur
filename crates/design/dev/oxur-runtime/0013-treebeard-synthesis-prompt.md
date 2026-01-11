# Treebeard Architecture Synthesis Prompt

## Your Role

You are a senior systems architect specializing in programming language implementation. You have been given comprehensive research on interpreter design, VM architectures, and the current state of the Oxur project. Your task is to synthesize this information into concrete, actionable architectural recommendations for **Treebeard** — a tree-walking interpreter for Rust's `syn` AST, a v3 (potentially a complete rewrite) for a v2 provided.

---

## Input Documents You Will Receive

1. **Treebeard Architecture Research v2** (`treebeard-architecture-v2.md`)
   - The proposed split architecture (Treebeard as general `syn` interpreter, oxur-vm as thin frontend)
   - Research findings on tree-walking interpreters, macro expansion, ownership tracking, etc.
   - Preliminary prototyping order and risk assessment

2. **Rust VM/Interpreter Analysis** (findings from analyzing Rhai, Miri, Rune, Gluon, Ketos)
   - Environment/binding patterns from real Rust interpreters
   - Value representation approaches
   - Rust interop mechanisms
   - Ownership tracking implementations (especially Miri)

3. **BEAM/LFE/Elixir Analysis** (findings from analyzing LFE, Erlang/OTP, Elixir)
   - Hot code loading patterns
   - Macro expansion architecture
   - REPL implementation patterns
   - Lisp-on-VM layering principles

4. **Oxur Feature Status** (`STATUS.md`)
   - Current implementation progress by category
   - What already works (Rust AST Bridge at 95%!)
   - What's missing (Macro System at 0%, VM at 0%)
   - Where Treebeard needs to plug in

---

## Your Task

Produce a comprehensive **Treebeard Implementation Guide** that answers:

### Part 1: Architecture Validation

Review the proposed split architecture and either:

- **Validate** it with supporting evidence from the research
- **Revise** it based on patterns found in the analyzed codebases
- **Identify gaps** that the research didn't address

Specifically address:

1. Is the `LanguageFrontend` trait the right abstraction boundary?
2. Is `syn` AST the right intermediate representation, or should there be another layer?
3. How does Oxur's existing 95% Rust AST Bridge fit into the architecture?

### Part 2: Critical Path Analysis

Given Oxur's current status, what is the **minimum viable Treebeard** that unblocks the most progress?

Consider:

- Oxur's Macro System is at 0% — Treebeard doesn't handle macros, but needs to support frontends that do
- Oxur's Core Evaluation is at 25% — this is what Treebeard replaces/enhances
- Oxur's REPL is at 60% with solid infrastructure — Treebeard needs to integrate
- Oxur's Rust AST Bridge is at 95% — this is the foundation Treebeard builds on

Produce a **dependency graph** showing what must be built in what order.

### Part 3: Design Decisions Matrix

For each major design decision, provide:

| Decision | Options | Recommendation | Rationale | Evidence |
|----------|---------|----------------|-----------|----------|
| Environment representation | HashMap chain vs indexed vs persistent | ? | ? | From [codebase] |
| Value boxing | Enum vs tagged pointer vs NaN boxing | ? | ? | From [codebase] |
| Ownership tracking | Full Miri vs RefCell-style vs none | ? | ? | From [research] |
| Closure capture | Environment reference vs flat capture | ? | ? | From [codebase] |
| TCO mechanism | Trampolining vs CPS vs none | ? | ? | From [research] |
| Compilation trigger | Manual vs hotcount vs never | ? | ? | From [research] |
| Function lookup | Early-bound vs late-bound | ? | ? | From BEAM |
| Macro timing | Eager vs lazy expansion | ? | ? | From LFE/Elixir |

### Part 4: Module Specifications

For each Treebeard crate, provide:

#### treebeard-core

```rust
// Key types with signatures
// Key traits with methods
// Key functions with signatures
```

- Responsibilities (what it does)
- Non-responsibilities (what it explicitly doesn't do)
- Dependencies (what it needs)
- Dependents (what needs it)

#### treebeard-repl

*Same format*

#### treebeard-loader

*Same format*

#### treebeard-interface

*Same format*

### Part 5: Integration Plan with Oxur

How does Treebeard integrate with Oxur's existing codebase?

1. **What Oxur code can be reused?**
   - The Rust AST Bridge (95% complete)
   - The REPL infrastructure (60% complete)
   - What else?

2. **What Oxur code needs to be replaced?**
   - Current evaluation (25% complete)
   - What's the migration path?

3. **What new code is needed in oxur-vm?**
   - The `OxurFrontend` implementation
   - Macro expansion (the 0% that needs to become 100%)
   - What else?

4. **What's the integration sequence?**
   - Can we run Treebeard alongside existing Oxur evaluation?
   - How do we incrementally migrate?

### Part 6: Ownership Model Specification

This is Treebeard's most novel aspect. Based on the Miri research and the constraints of REPL use, specify:

1. **What ownership violations are caught at runtime?**
   - Use-after-move: yes/no, how?
   - Double mutable borrow: yes/no, how?
   - Borrow outlives scope: yes/no, how?

2. **What is deferred to compilation?**
   - Lifetime parameters
   - What else?

3. **What's the data structure?**

   ```rust
   // Specify the actual types
   struct OwnershipTracker { ... }
   enum OwnershipState { ... }
   ```

4. **What's the performance budget?**
   - Overhead per operation
   - Memory overhead per value
   - Acceptable slowdown factor

### Part 7: Risk Mitigation

For each high/medium risk from the research, provide:

| Risk | Likelihood | Impact | Mitigation Strategy | Validation Approach |
|------|------------|--------|---------------------|---------------------|
| syn AST complexity | ? | ? | ? | ? |
| Ownership tracking perf | ? | ? | ? | ? |
| rustc compilation latency | ? | ? | ? | ? |
| Semantic drift | ? | ? | ? | ? |
| Frontend API stability | ? | ? | ? | ? |

### Part 8: Revised Prototyping Order

Based on Oxur's current state and the research findings, revise the prototyping phases:

For each phase:

- **Goal**: What we're validating
- **Deliverables**: Concrete artifacts
- **Duration**: Time estimate
- **Dependencies**: What must exist first
- **Success criteria**: How we know it works
- **Oxur integration**: How it connects to existing code

### Part 9: Open Questions

List questions that:

1. **Require prototyping** to answer (can't be resolved by more research)
2. **Require user feedback** (design choices that affect Oxur's feel)
3. **Can be deferred** (don't block initial implementation)

### Part 10: Executive Summary

A 1-page summary for someone who won't read the full document:

- What is Treebeard?
- Why this architecture?
- What's the critical path?
- What are the biggest risks?
- What's the timeline?

---

## Constraints to Honor

1. **Budget**: ~50k lines of Rust for Treebeard
2. **Performance**: 10-100x native is acceptable for interpreted code
3. **Oxur compatibility**: Must work with existing Oxur AST Bridge
4. **REPL-first**: Interactive development is the primary use case
5. **Rust semantics**: Must faithfully represent Rust's ownership model (even if simplified)

---

## Output Format

Produce a single comprehensive Markdown document with all sections above. Use:

- Clear headers for navigation
- Code blocks for type definitions
- Tables for comparisons
- Diagrams (ASCII art) where helpful
- Cross-references between sections

The document should be:

- **Actionable**: Someone could start implementing from it
- **Justified**: Every recommendation traces to research evidence
- **Prioritized**: Clear on what matters most
- **Honest**: Clear about uncertainties and risks

---

## How to Approach This

1. **Read all inputs first** before making recommendations
2. **Look for patterns** that appear across multiple sources
3. **Identify contradictions** between sources and resolve them
4. **Ground recommendations in evidence** from the analyzed codebases
5. **Consider Oxur's current state** — don't recommend rebuilding what works
6. **Think about the developer experience** — this is for interactive Lisp development
7. **Be specific** — vague recommendations aren't useful

---

## Example of Good vs Bad Recommendations

**Bad**: "The environment should be efficient."

**Good**: "Use a HashMap-based frame chain (like Rhai's `Scope`) rather than persistent data structures. Evidence: Rhai achieves 2x Python performance with this approach (benchmarks.html), and the simpler implementation fits Treebeard's 50k line budget. Persistent structures add ~2x lookup overhead (rpds benchmarks) without benefits for sequential REPL evaluation. Implementation: see `rhai/src/types/scope.rs` lines 45-120 for the pattern."

---

## Final Note

The goal is not a perfect design, but a **sound foundation** that can evolve. Treebeard will change as Oxur matures. The architecture should make future changes tractable, not prevent them.

Remember: LFE succeeded by embracing Erlang's semantics rather than fighting them. Treebeard should embrace Rust's semantics (via `syn`) rather than inventing a parallel type system. The research supports this approach — now make it concrete.
