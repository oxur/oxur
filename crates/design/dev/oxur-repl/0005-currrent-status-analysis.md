# Oxur REPL: Current Status vs. Specification Requirements

## What We've Built (Phases 1-3 Complete)

### ✅ Phase 1: Protocol & Transport Layer
- **Binary protocol** with Postcard serialization
- **TCP transport** with async I/O
- **Length-prefixed framing** for reliable delivery
- **Request/Response types** (Operation, OperationResult, Status)
- **Codec implementation** (PostcardCodec)

### ✅ Phase 2: Basic Evaluation Layer  
- **EvalContext** with session state
- **Placeholder evaluation** (Tier 1 & Tier 2 stubs)
- **Output capture** infrastructure
- **Lisp/Sexpr mode** hooks (placeholder implementations)

### ✅ Phase 3: Server Layer
- **SessionManager** with thread-safe storage
- **MessageHandler** for request/response routing
- **ReplServer** with concurrent connections
- **Graceful shutdown** with connection tracking
- **ShutdownHandle** for controlled termination

### Test Coverage
- **141 tests passing** (134 existing + 7 new)
- Full protocol integration tests
- Server lifecycle tests
- Error handling tests

---

## What's Missing (Per ODD-0030)

### ❌ Core REPL Components (Not Implemented)

#### 1. **CachedCompiler** (Critical)
The heart of the REPL - compiles Oxur to Rust and executes it.
- **Status:** Not implemented
- **Location:** Should be in `crates/oxur-repl/src/compiler/`
- **Dependencies:** CodeGenerator, VariableStore, SessionDir, Subprocess

#### 2. **VariableStore** (Critical)
Type-erased storage using `Box<dyn Any>` for variable persistence.
- **Status:** Not implemented
- **Pattern:** From evcxr
- **Lines:** ~50 lines
- **Location:** `crates/oxur-repl/src/runtime/variable_store.rs`

#### 3. **CodeGenerator** (Critical)
Lowers Oxur Core Forms to Rust code with source maps.
- **Status:** Not implemented
- **Dependencies:** Needs Core Forms from oxur-lang
- **Location:** `crates/oxur-repl/src/codegen/`

#### 4. **Subprocess Runtime** (Critical)
Separate binary that loads and executes compiled libraries.
- **Status:** Not implemented
- **Location:** `crates/oxur-subprocess/`
- **Dependencies:** libloading, VariableStore

#### 5. **Source Map Tracking** (High Priority)
Maps generated Rust back to Oxur source positions.
- **Status:** Not implemented
- **Location:** `crates/oxur-repl/src/source_map.rs`

#### 6. **SessionDir Management** (High Priority)
Manages temporary directories and Cargo projects per session.
- **Status:** Not implemented
- **Location:** `crates/oxur-repl/src/session/dir.rs`

#### 7. **Cargo Integration** (High Priority)
Invokes cargo, parses JSON output, handles errors.
- **Status:** Not implemented
- **Location:** `crates/oxur-repl/src/compiler/cargo.rs`

#### 8. **Error Translation** (High Priority)
Translates rustc errors back to Oxur source positions.
- **Status:** Not implemented
- **Dependencies:** Source maps, cargo JSON parsing

#### 9. **Calculator Mode / Tier 1** (Medium Priority)
Fast path for literal arithmetic (<1ms).
- **Status:** Placeholder exists in EvalContext
- **Needs:** Actual implementation

---

## Prerequisite Analysis

### What We HAVE ✅

1. **Network Protocol** - Complete, tested, production-ready
2. **Session Management** - SessionManager with isolation
3. **Server Infrastructure** - ReplServer with graceful shutdown
4. **Output Capture** - Infrastructure in place
5. **Message Routing** - MessageHandler connects everything

### What We NEED ❌

#### From oxur-lang (NOT in oxur-repl):
1. **Core Forms** - Canonical representation after macro expansion
   - Status: Likely needs design
   - Location: `crates/oxur-lang/src/core_forms/`
   
2. **Parser** - Oxur syntax → Surface Forms
   - Status: May exist in placeholder form
   - Location: `crates/oxur-lang/src/parser/`

3. **Macro Expander** - Surface → Core Forms
   - Status: Needs implementation
   - Location: `crates/oxur-lang/src/expander/`

#### From oxur-comp (NOT in oxur-repl):
1. **Lowering** - Core Forms → Rust AST
   - Status: Planned but not implemented
   - Location: `crates/oxur-comp/src/lower/`
   
2. **Code Generation** - Rust AST → Rust source
   - Status: Can use oxur-ast's printer
   - Location: Could use existing infrastructure

---

## Critical Path Analysis

### Immediate Blockers

**CANNOT implement CachedCompiler without:**
1. ✅ Session management (WE HAVE THIS)
2. ❌ Core Forms definition (FROM oxur-lang)
3. ❌ Lowering to Rust AST (FROM oxur-comp)
4. ❌ VariableStore (NEED TO BUILD)
5. ❌ Subprocess runtime (NEED TO BUILD)

**CAN implement independently:**
1. ✅ VariableStore - No dependencies
2. ✅ SessionDir - No dependencies  
3. ✅ Cargo integration - No dependencies
4. ✅ Subprocess runtime - Only depends on VariableStore

**MUST wait for oxur-lang/oxur-comp:**
1. ❌ CodeGenerator - Needs Core Forms
2. ❌ Full evaluation - Needs lowering
3. ❌ Source maps - Needs parser integration

---

## Recommended Implementation Order

### Immediate (Can Start Now):

**Phase A: Foundation Components**
1. VariableStore implementation (~1 day)
2. SessionDir management (~1 day)
3. Subprocess runtime binary (~2 days)
4. Cargo invocation wrapper (~1 day)

These have NO dependencies on oxur-lang or oxur-comp.

**Deliverable:** Basic compilation infrastructure

### Short-term (Need Core Forms):

**Phase B: Code Generation**
1. Define minimal Core Forms (with oxur-lang team)
2. Implement CodeGenerator
3. Add source map tracking
4. Error translation

**Deliverable:** Can compile simple forms

### Medium-term (Full Integration):

**Phase C: Full REPL**
1. Calculator mode (Tier 1)
2. Integration with existing server
3. Testing and optimization
4. Platform-specific handling

**Deliverable:** Production REPL

---

## Key Questions for oxur-lang/oxur-comp

1. **Core Forms Status?**
   - Are Core Forms defined?
   - What types exist?
   - Can we get a minimal set for REPL v1.0?

2. **Lowering Status?**
   - Can we lower Core Forms to Rust AST?
   - Is this implemented or designed?
   - Timeline?

3. **Integration Points?**
   - Where does REPL fit in the pipeline?
   - Who owns what?

---

## Gap Summary

### We Have (oxur-repl):
- ✅ Complete network protocol (Phases 1-3)
- ✅ Session management
- ✅ Server infrastructure
- ✅ 141 tests passing

### We Need (Before full REPL):
- ❌ Core Forms (from oxur-lang)
- ❌ Lowering (from oxur-comp)
- ❌ VariableStore (build in oxur-repl)
- ❌ Subprocess runtime (build in oxur-subprocess)
- ❌ Cargo integration (build in oxur-repl)
- ❌ CodeGenerator (build in oxur-repl, needs Core Forms)

### Can Build Independently:
1. VariableStore ✅
2. SessionDir ✅
3. Subprocess runtime ✅
4. Cargo integration ✅
5. Error parsing ✅

### Blocked Until oxur-lang/oxur-comp:
1. CodeGenerator ❌
2. Full evaluation ❌
3. Source maps ❌

---

## Recommendation

**START NOW with Phase A (Foundation):**

Build the infrastructure that doesn't depend on oxur-lang/oxur-comp:
- VariableStore
- SessionDir
- Subprocess runtime
- Cargo wrapper

**PARALLEL EFFORT needed:**

Work with oxur-lang team to define:
- Minimal Core Forms for REPL v1.0
- Lowering strategy
- Integration points

**This gives us ~1-2 weeks of independent work while unblocking dependencies.**
