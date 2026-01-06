# oxur-repl Enhancement Plan

**Date:** 2026-01-05
**Design Doc:** ODD-0018 (Oxur Remote REPL Protocol Design)
**Audit Scope:** Align code with Rust best practices and design doc

## Executive Summary

This document outlines enhancements needed to bring the oxur-repl codebase into full compliance with:

1. Rust best practices (from `assets/ai/ai-rust/guides/`)
2. Design doc ODD-0018 specifications
3. Oxur project conventions (from `CLAUDE.md`)

## Issues Identified

### 1. Type Safety - Newtype Pattern (Priority: HIGH)

**Pattern Violated:** API-40 (Newtypes Provide Static Distinctions)

**Current State:**

```rust
// crates/oxur-repl/src/protocol/messages.rs:10-11
pub type SessionId = String;
pub type MessageId = u64;
```

**Problem:**

- Type aliases don't prevent mixing SessionId with regular String
- No compile-time safety for domain-specific types
- Can't add domain-specific methods

**Solution:**
Replace type aliases with newtypes:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(u64);

impl MessageId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
```

**Files to Modify:**

- `crates/oxur-repl/src/protocol/messages.rs`
- All files using SessionId/MessageId (update call sites)

---

### 2. Error Handling Logic (Priority: HIGH)

**Pattern Violated:** EH-02 (Option vs Result Decision)

**Current State:**

```rust
// crates/oxur-repl/src/server/session.rs:111
self.context
    .eval(input)
    .map_err(|_| SessionError::NotFound(session_id.to_string()))?;
```

**Problem:**

- Mapping `EvalError` to `SessionError::NotFound` is semantically incorrect
- Loses error information from eval
- Misleading error type

**Solution:**

1. Add `SessionError::EvalFailed` variant:

```rust
#[derive(Debug, Error)]
pub enum SessionError {
    // ... existing variants ...

    #[error("evaluation failed")]
    EvalFailed(#[from] EvalError),
}
```

1. Update error mapping:

```rust
self.context.eval(input)?;  // Auto-converts via From trait
```

**Files to Modify:**

- `crates/oxur-repl/src/server/session.rs`
- Add EvalError to SessionError's From impl

---

### 3. API Design - Generic Parameters (Priority: MEDIUM)

**Pattern Violated:** API-03 (Accept impl AsRef<> Where Feasible)

**Current State:**
Various functions accept `&str` or `String` parameters.

**Problem:**

- Less flexible API
- Forces callers to convert types
- Not following Rust idioms

**Solution:**
Update function signatures to accept `impl AsRef<str>`:

```rust
// Before
pub fn new(session_id: &str, message: String) -> Self

// After
pub fn new(session_id: impl AsRef<str>, message: impl Into<String>) -> Self {
    Self {
        session_id: session_id.as_ref().to_string(),
        message: message.into(),
    }
}
```

**Files to Modify:**

- `crates/oxur-repl/src/protocol/messages.rs` - Message constructors
- Other modules with string parameters

---

### 4. Documentation - Error/Panic Sections (Priority: MEDIUM)

**Pattern Violated:** EH-09 (Document Error Conditions), EH-10 (Document Panic Conditions)

**Current State:**
Many functions lack "Errors" and "Panics" sections in documentation.

**Problem:**

- Users don't know what can fail
- No documentation of panic conditions
- Harder to use API correctly

**Solution:**
Add comprehensive documentation:

```rust
/// Evaluates an Oxur expression in this session's context.
///
/// # Arguments
///
/// * `input` - The Oxur expression to evaluate
///
/// # Errors
///
/// Returns `SessionError::EvalFailed` if:
/// - The expression contains syntax errors
/// - The expression references undefined variables
/// - Evaluation times out (> 30 seconds)
///
/// # Examples
///
/// ```
/// let mut session = Session::new();
/// session.eval("(+ 1 2)")?;
/// # Ok::<(), SessionError>(())
/// ```
pub fn eval(&mut self, input: &str) -> Result<(), SessionError> {
    // ...
}
```

**Files to Modify:**

- All public functions in:
  - `crates/oxur-repl/src/protocol/messages.rs`
  - `crates/oxur-repl/src/server/session.rs`
  - `crates/oxur-repl/src/server/repl_server.rs`
  - `crates/oxur-repl/src/eval/context.rs`

---

### 5. Common Trait Implementations (Priority: MEDIUM)

**Pattern Violated:** API-34 (Implement Common Traits Eagerly)

**Current State:**
Some types missing `Debug`, `Clone`, `PartialEq`, etc.

**Problem:**

- Types can't be used in collections
- Harder to test
- Less ergonomic API

**Solution:**
Add missing trait implementations:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplServer {
    // ... fields ...
}

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    // ... fields ...
}
```

**Files to Modify:**

- Audit all public types and add appropriate derives
- Implement `Default` where sensible

---

### 6. Position Tracking in Errors (Priority: MEDIUM)

**Pattern Followed (Good!):** Oxur-specific error pattern from CLAUDE.md

**Current State:**
Errors don't include position information.

**Recommendation:**
For parse/eval errors, add Position tracking:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Syntax error at {pos}: {message}")]
    SyntaxError {
        message: String,
        pos: Position,
    },
    // ... other variants ...
}
```

**Files to Modify:**

- `crates/oxur-repl/src/eval/context.rs` - Add Position to errors

---

### 7. Test Coverage Enhancement (Priority: LOW)

**Pattern:** CLAUDE-CODE-COVERAGE.md guidance

**Current State:**
Good test coverage overall, but some edge cases missing.

**Recommendations:**

- Add round-trip tests for message serialization
- Add error path tests for all error variants
- Add property-based tests for codec

**Files to Modify:**

- Add more tests to existing test modules

---

## Implementation Checklist

### Phase 1: Type Safety (HIGH Priority)

- [ ] Replace SessionId type alias with newtype
- [ ] Replace MessageId type alias with newtype
- [ ] Update all call sites
- [ ] Add comprehensive tests for newtypes
- [ ] Update documentation

### Phase 2: Error Handling (HIGH Priority)

- [ ] Fix SessionError::NotFound mapping issue
- [ ] Add SessionError::EvalFailed variant
- [ ] Add From impl for EvalError
- [ ] Update error handling in session.rs
- [ ] Add tests for error cases

### Phase 3: API Design (MEDIUM Priority)

- [ ] Audit all public functions for parameter types
- [ ] Update to use impl AsRef<str> where appropriate
- [ ] Update to use impl Into<String> where appropriate
- [ ] Verify backwards compatibility (or document breaking changes)
- [ ] Add tests for new API flexibility

### Phase 4: Documentation (MEDIUM Priority)

- [ ] Add "Errors" sections to all fallible functions
- [ ] Add "Panics" sections where applicable
- [ ] Add examples to complex functions
- [ ] Review and improve module-level documentation
- [ ] Run `cargo doc` and review output

### Phase 5: Trait Implementations (MEDIUM Priority)

- [ ] Audit all public types for missing traits
- [ ] Add Debug, Clone, PartialEq where appropriate
- [ ] Add Default where sensible
- [ ] Add Display for user-facing types
- [ ] Add tests for trait implementations

### Phase 6: Position Tracking (MEDIUM Priority)

- [ ] Add Position struct to error module
- [ ] Update EvalError with Position fields
- [ ] Update error construction to include positions
- [ ] Add tests for position tracking
- [ ] Update error display to show positions

### Phase 7: Testing (LOW Priority)

- [ ] Add round-trip serialization tests
- [ ] Add error path coverage tests
- [ ] Add property-based tests
- [ ] Verify 95%+ coverage with cargo llvm-cov
- [ ] Add integration tests

---

## Success Criteria

1. **All code follows Rust best practices:**
   - ✓ No violations of anti-patterns (11-anti-patterns.md)
   - ✓ Follows core idioms (01-core-idioms.md)
   - ✓ API design guidelines met (02-api-design.md)
   - ✓ Error handling best practices (03-error-handling.md)

2. **Aligns with design doc ODD-0018:**
   - ✓ All message types implemented correctly
   - ✓ Protocol matches specification
   - ✓ Session management as designed
   - ✓ Transport abstraction correct

3. **Meets Oxur project standards:**
   - ✓ Position tracking in errors
   - ✓ Test coverage ≥ 95%
   - ✓ Documentation complete
   - ✓ Naming conventions followed

4. **Build and test success:**
   - ✓ `cargo build --all` succeeds
   - ✓ `cargo test --all` passes
   - ✓ `make lint` passes
   - ✓ `make coverage` shows ≥95%

---

## Notes

- All changes should be made incrementally with tests
- Run `make check` after each phase
- Document any breaking changes
- Consider creating a migration guide if API changes significantly
