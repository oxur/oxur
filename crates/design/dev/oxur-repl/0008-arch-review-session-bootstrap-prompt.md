# Oxur REPL Architecture Review - Session Bootstrap Prompt

## Context & Purpose

This prompt provides context for reviewing and refining the Oxur REPL architecture. The REPL implementation has reached a critical juncture where foundational protocol work (ODD-0018) is complete, but before proceeding with full implementation (ODD-0030), we need to ensure the architecture is sound and well-documented.

## Current Situation

### What's Been Completed
- **ODD-0018**: Oxur Remote REPL Protocol Design - FULLY IMPLEMENTED
  - Binary protocol with Postcard serialization
  - TCP transport with async I/O
  - SessionManager with thread-safe isolation
  - ReplServer with graceful shutdown
  - 141 passing tests

### The Problem
Claude Code has been working incrementally and may be operating in a "narrow canyon" without full visibility of the big picture. Recent discoveries from analyzing the evcxr crates have significantly adjusted our architecture, but these insights may not be fully reflected in our design docs or implementation plans.

## Key Concerns

1. **Missing Big Picture Documentation**: Need a comprehensive architecture doc that shows how all pieces fit together
2. **Cross-Crate Dependencies**: Significant work needed in oxur-lang and oxur-comp before full REPL implementation
3. **Architecture Drift**: Recent evcxr analysis revealed new components not explicitly called out in ODD-0030
4. **Planning Disconnect**: Foundation work plans may not align with the complete architectural vision
5. **Session Continuity**: Need to maintain context across multiple Claude sessions

## Critical Questions to Answer

### About Design Documents

**ODD-0026 (Evaluation Strategy)**:
- Does it reflect our current understanding from evcxr analysis?
- Does it account for the three-tier compilation strategy?
- Are the CachedCompiler, VariableStore, Subprocess patterns documented?

**ODD-0030 (Implementation Specification)**:
- Does it describe the actual architecture (client, server, protocols)?
- Are all components from evcxr analysis explicitly listed?
- Does it show what gets compiled when and by whom?
- Is the relationship between oxur-repl, oxur-lang, and oxur-comp clear?

### About Architecture

1. **Component Boundaries**: What lives in oxur-repl vs oxur-lang vs oxur-comp vs oxur-subprocess?
2. **Compilation Pipeline**: Who owns each stage? (Parse → Expand → Lower → Codegen → Compile → Execute)
3. **Data Flow**: How do Core Forms flow from oxur-lang to CachedCompiler?
4. **Critical Path**: What's the minimum viable set of components for a working REPL?

## Documents to Review

### Design Documents
- `crates/design/docs/06-final/0026-oxur-repl-evaluation-strategy.md`
- `crates/design/docs/05-active/0030-oxur-repl-implementation-specification.md`
- `crates/design/docs/06-final/0018-oxur-remote-repl-protocol-design.md`

### Related evcxr Analysis Documents
(These contain our discoveries about REPL architecture)
- Any documents analyzing evcxr's CachedCompiler
- Any documents analyzing evcxr's VariableStore
- Any documents analyzing the subprocess pattern
- Any documents about compilation strategies

### Current Implementation Plans
- `oxur-repl-status-analysis.md` (uploaded)
- `oxur-repl-component-proto-plans.md` (uploaded)
- `oxur-repl-phase1-foundation-impl-plan.md` (uploaded)

### Source Code
- `crates/oxur-repl/src/` - Current implementation
- Relevant parts of oxur-lang and oxur-comp (as needed)

## Goals for This Review

### Primary Goal
Create a **REPL Architecture Overview** document that:
- Shows the complete system architecture (all components, all crates)
- Defines clear boundaries and responsibilities
- Documents the compilation pipeline end-to-end
- Provides enough detail for Claude Code to make correct implementation decisions
- Serves as the single source of truth for REPL architecture

### Secondary Goals
1. **Update ODD-0026** if evaluation strategy needs refinement
2. **Update ODD-0030** to explicitly call out all architectural components
3. **Review Foundation Plans** against the complete architecture
4. **Identify Gaps** between current plans and architectural requirements
5. **Create Implementation Roadmap** with correct sequencing and dependencies

## What "Just Enough Detail" Means

For each major component, document:
- **Purpose**: What problem does it solve?
- **Location**: Which crate/module?
- **Dependencies**: What does it need to work?
- **Interface**: What's its API contract?
- **Data Flow**: What goes in, what comes out?

**Avoid**:
- Implementation details that constrain the developer
- Premature optimization discussions
- Overly prescriptive code patterns

**Include**:
- Enough context to make correct architectural decisions
- Critical patterns that MUST be followed (e.g., the subprocess isolation)
- Integration points between components

## Discovered Components from evcxr Analysis

These components were identified through evcxr analysis but may not be fully documented:

1. **CachedCompiler** - Core REPL engine that manages compilation caching
2. **VariableStore** - Type-erased storage using `Box<dyn Any>`
3. **Subprocess Runtime** - Separate binary for safe code execution
4. **SessionDir** - Manages temporary Cargo projects per session
5. **CodeGenerator** - Lowers Oxur Core Forms to Rust source
6. **Source Maps** - Maps generated Rust back to Oxur positions
7. **Cargo Integration** - Wrapper around cargo with JSON parsing
8. **Error Translation** - Translates rustc errors to Oxur positions

Are all of these explicitly documented in ODD-0030? Do they have clear crate assignments?

## Key Architectural Decisions to Validate

1. **Three-Tier Strategy**: Calculator mode → Direct compilation → Cached compilation
2. **Subprocess Isolation**: Code executes in separate process for safety
3. **Core Forms as Interface**: oxur-lang provides Core Forms, oxur-repl lowers them
4. **Variable Persistence**: VariableStore pattern from evcxr
5. **Source Map Round-Tripping**: Preserve position info through Rust compilation

## Success Criteria

This review succeeds when:

1. ✅ We have a single REPL Architecture Overview document
2. ✅ ODD-0026 and ODD-0030 accurately reflect our current understanding
3. ✅ All components have clear crate assignments
4. ✅ The compilation pipeline is fully documented
5. ✅ Foundation work plans align with the complete architecture
6. ✅ Claude Code can start a new session with just the architecture doc and make correct decisions

## How to Use This Prompt

**Starting a new session:**
1. Upload this bootstrap prompt
2. Upload current versions of relevant design docs
3. Upload any new analysis or implementation documents
4. State what specific aspect you want to focus on

**Continuing work:**
1. Reference previous session conclusions
2. Upload updated documents
3. Focus on next architectural area

**Before implementation:**
- Review the architecture doc
- Verify all dependencies are clear
- Confirm component boundaries

## Current State Summary

**Phase 1-3 Complete** ✅:
- Binary protocol, transport, server infrastructure
- 141 tests passing
- Production-ready network layer

**Blocked on** ⏸️:
- Core Forms definition (oxur-lang)
- Lowering implementation (oxur-comp)
- Architecture documentation updates

**Can Build Now** 🚀:
- VariableStore
- SessionDir
- Subprocess runtime
- Cargo wrapper

**Needs Architectural Clarity** ❓:
- Where does each component live?
- How do Core Forms flow through the system?
- What's the critical path to a working REPL?

## Next Steps

1. Review ODD-0026 for accuracy and completeness
2. Review ODD-0030 for explicit component documentation
3. Create REPL Architecture Overview document
4. Validate foundation work plans against architecture
5. Update design docs as needed
6. Proceed with implementation in correct order

---

## Template for Session Start

When starting a new session, say:

> I'm continuing the Oxur REPL architecture review. I've uploaded the bootstrap prompt and [list other documents]. I want to focus on [specific area: design doc review / architecture documentation / implementation planning]. 
>
> The key context is: [brief summary of where we are in the review process].

This ensures continuity and focus across sessions.
