# Oxur Session Bootstrap Template

**Instructions:** Copy everything below this line, attach the relevant design documents, fill in the bracketed sections, and paste into a new Claude conversation.

---

# Oxur Development Session

I'm working on **Oxur** - a Lisp dialect that compiles to Rust with 100% bidirectional interoperability. We have comprehensive design documents that establish the architecture, specifications, and implementation approach.

## Please Read These Documents First

I'm attaching the following design documents. Please read them to understand the project:

**[ATTACH FILES - check the boxes for what's relevant to this session]:**
- [ ] `oxur-001-letter-of-intent.md` - Project vision and philosophy (ALWAYS include this)
- [ ] `oxur-002-rast-spec.md` - S-expression format specification (for oxur-ast work)
- [ ] `oxur-003-phase-0-sexp.md` - S-expression infrastructure implementation
- [ ] `oxur-004-phase-1-ast-builder.md` - AST types and builder implementation
- [ ] `oxur-005-phase-2-generator.md` - Generator implementation (AST → S-expr)
- [ ] `oxur-006-phase-3-integration.md` - Integration, testing, and CLI
- [ ] `oxur-007-phase-4-complete.md` - Complete AST coverage and code generation
- [ ] [Other relevant design docs]

## Quick Context

**Project:** Oxur - Lisp dialect targeting Rust with complete interop  
**Repository Structure:** github.com/oxur/* (multiple repositories)  
**Current Component:** [FILL IN: oxur-ast | oxur-lang | oxur-repl | oxur-cli | design | etc]  
**Current Phase:** [FILL IN: Phase 0-4 of oxur-ast | Initial design | Implementation | Testing | etc]  

## Architecture Overview

**Two-Stage Compilation:**
```
Oxur Syntax → Core Forms (IR) → Rust AST → Rust Code → Binary
  (Stage 1)         (IR)          (Stage 2)
```

**oxur-ast (Stage 2) - Rust AST ↔ S-expression:**
- Bidirectional conversion between Rust AST and canonical S-expressions
- Integration with syn crate for parsing real Rust code
- Complete coverage of Rust AST nodes
- Code generation back to valid Rust

**oxur-lang (Stage 1) - The Lisp Compiler:**
- Parses Oxur Lisp syntax
- Expands macros
- Type checking
- Compiles to S-expressions (Core Forms)

**Key Design Decisions:**
- ✓ Crate naming: hyphenated (oxur-ast, oxur-lang, NOT rast)
- ✓ S-expression format is canonical (see oxur-002 spec)
- ✓ Using syn for Rust integration
- ✓ Round-trip testing is critical
- ✓ Rust semantics with Lisp syntax (NOT Lisp semantics)
- ✓ 100% interop from day one
- ✓ No plugin memory leaks (Rust advantage over Go-based Zylisp)

## Current Status

**Completed:**
- [FILL IN: e.g., "Phases 0-3 of oxur-ast design complete"]
- [FILL IN: e.g., "S-expression infrastructure specified"]
- [FILL IN: e.g., "AST builder architecture defined"]

**In Progress:**
- [FILL IN: e.g., "Implementing Phase 1 with Claude Code"]
- [FILL IN: e.g., "Setting up workspace structure"]

**Not Started:**
- [FILL IN: e.g., "oxur-lang compiler"]
- [FILL IN: e.g., "REPL implementation"]

## Key Conventions & Patterns

**Workspace Structure:**
```
github.com/oxur/
├── oxur-ast/       # Rust AST ↔ S-expr (Stage 2)
├── oxur-lang/      # Oxur compiler (Stage 1)
├── oxur-repl/      # REPL server/client
├── oxur-cli/       # CLI tool
├── rely-rs/        # Supervision library
└── design/         # Design docs (where we are now)
```

**Code Conventions:**
- Rust workspace with multiple crates
- Comprehensive testing (unit, integration, round-trip)
- Error handling with thiserror
- CLI with clap
- Documentation with examples

**Testing Philosophy:**
- Round-trip verification: X → Y → X must be equivalent
- Test with real Rust code from rust-lang/rust repository
- Benchmarking with criterion
- Regression test suite

## Today's Focus

**Primary Goal:**
[FILL IN: e.g., "Help Claude Code implement Phase 1 of oxur-ast - the AST builder"]

**Specific Tasks:**
- [FILL IN: e.g., "1. Review the phase implementation document"]
- [FILL IN: e.g., "2. Create the file structure"]
- [FILL IN: e.g., "3. Implement core types"]
- [FILL IN: e.g., "4. Write tests"]

**Success Criteria:**
[FILL IN: e.g., "Phase 1 complete when all builder tests pass and Hello World AST can be constructed"]

## What I Need Help With

[FILL IN: Specific questions or guidance needed, e.g.:]
- "Review my implementation approach for X"
- "Help debug why Y isn't working"
- "Suggest best practices for Z"
- "Code review for implementation of W"

## Files Already in Context

[OPTIONAL - if you've already created files in this conversation:]
- [FILL IN: e.g., "src/ast/types.rs - AST type definitions"]
- [FILL IN: e.g., "tests/builder_tests.rs - Builder test suite"]

## Important Notes

**Things to Remember:**
- [FILL IN: Any session-specific context or constraints]
- [FILL IN: e.g., "We're using Rust stable, not nightly"]
- [FILL IN: e.g., "Focus on Hello World support first, expand later"]

**Decisions Made in This Session:**
[FILL IN as the conversation progresses - keep a running log]
- 
- 
- 

---

## Quick Reference

**Key Design Documents:**
1. **Letter of Intent (001)** - Overall vision and philosophy
2. **S-expr Spec (002)** - Canonical format for Rust AST
3. **Phase Documents (003-007)** - Implementation guides for oxur-ast

**Critical Concepts:**
- **Core Forms** - Canonical S-expression representation of code
- **Round-trip** - X → transform → X must preserve meaning
- **Bidirectional** - Must work both ways (Rust ↔ S-expr)
- **100% Interop** - Can call any Rust code, Rust can call any Oxur code

**When Stuck:**
- Check the relevant design document first
- Look for similar patterns in completed phases
- Consider: "If this was in Zetalisp or LFE, how would it work?"
- Remember: Rust semantics, Lisp syntax

---

**After you've read the attached documents, please confirm your understanding and let me know you're ready to help with today's focus!**
