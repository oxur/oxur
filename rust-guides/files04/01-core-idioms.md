# Core Rust Idioms

Essential Rust idioms and patterns that every Rust developer should know. These represent the foundational "Rust way" of doing things.

## Table of Contents

- [Naming Conventions](#naming-conventions)
- [Panic Semantics](#panic-semantics)
- [Function Organization](#function-organization)
- [Documentation Basics](#documentation-basics)
- [Magic Values](#magic-values)

---

## Naming Conventions

### Avoid Weasel Words in Names

**Strength**: MUST

**Summary**: Symbol names should be free of uninformative words like `Service`, `Manager`, `Factory`.

**Example**:
```rust
// Bad - weasel words add no information
struct BookingService {
    bookings: Vec<Booking>
}

struct BookingManager {
    bookings: Vec<Booking>
}

// Good - descriptive names
struct Bookings {
    items: Vec<Booking>
}

struct BookingDispatcher {
    queue: Vec<Booking>
}
```

**Rationale**: Terms like "Service", "Manager", and "Factory" are vague and don't convey what the type actually does. Use specific names that describe the type's purpose. For builders, use the `Builder` suffix (e.g., `FooBuilder`, not `FooFactory`).

**See also**: M-CONCISE-NAMES, Builder pattern

---

## Panic Semantics

### Panic Means 'Stop the Program'

**Strength**: MUST

**Summary**: Panics are not exceptions—they signal immediate program termination and should not be caught for error handling.

**Example**:
```rust
// Bad - using panic for error handling
fn parse_config(path: &str) -> Config {
    let content = std::fs::read_to_string(path)
        .unwrap(); // Don't panic on I/O errors!
    // ...
}

// Good - return Result for recoverable errors
fn parse_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    // ...
    Ok(config)
}

// Acceptable panic - programming error detected
fn divide_by(x: u32, y: u32) -> u32 {
    if y == 0 {
        panic!("divide_by called with y=0, this is a programming error");
    }
    x / y
}
```

**Rationale**: Panics cannot be reliably caught (especially with `panic = "abort"`), and assuming they will be caught leads to fragile code. Use `Result` for recoverable errors, panic only for unrecoverable programming errors.

**Valid panic scenarios**:
- Programming errors: `x.expect("must never happen")`
- Const contexts: `const { foo.unwrap() }`
- User-requested unwrapping: providing an `unwrap()` method
- Poisoned locks (another thread already panicked)

**See also**: M-PANIC-IS-STOP, M-PANIC-ON-BUG

---

### Detected Programming Bugs are Panics, Not Errors

**Strength**: MUST

**Summary**: When an unrecoverable programming error is detected, panic immediately—don't return a Result.

**Example**:
```rust
// Bad - contract violation returns an error
fn divide_by(x: u32, y: u32) -> Result<u32, DivisionError> {
    if y == 0 {
        return Err(DivisionError::DivideByZero);
    }
    Ok(x / y)
}

// Good - contract violation panics
fn divide_by(x: u32, y: u32) -> u32 {
    if y == 0 {
        panic!("divide_by: y cannot be zero");
    }
    x / y
}

// Alternative - make it correct by construction
struct NonZeroU32(u32);

impl NonZeroU32 {
    pub fn new(value: u32) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }
    
    pub fn get(&self) -> u32 { self.0 }
}

fn divide_by(x: u32, y: NonZeroU32) -> u32 {
    x / y.get()
}
```

**Rationale**: Programming errors cannot be handled at runtime—there's no valid recovery path. Returning an error for contract violations creates impossible error-handling code. Use the type system to prevent invalid states when possible (correct by construction).

**See also**: M-PANIC-ON-BUG, newtype pattern

---

## Function Organization

### Prefer Regular Functions Over Associated Functions

**Strength**: SHOULD

**Summary**: Use regular functions for general computation; reserve associated functions primarily for instance creation.

**Example**:
```rust
struct Database;

impl Database {
    // Good - constructor as associated function
    pub fn new() -> Self { 
        Self 
    }
    
    // Good - method on instance
    pub fn query(&self) {
        // ...
    }
    
    // Bad - unrelated functionality as associated function
    fn check_parameters(p: &str) {
        // This doesn't need to be under Database!
    }
}

// Good - regular function for general logic
fn check_parameters(p: &str) {
    // ...
}

// Acceptable - trait associated function
trait Default {
    fn default() -> Self;
}
```

**Rationale**: Regular functions are first-class citizens in Rust and reduce unnecessary noise (`Database::check_parameters()` vs `check_parameters()`). Associated functions make sense for constructors and trait implementations, but general logic should be standalone.

**See also**: M-REGULAR-FN

---

## Documentation Basics

### First Sentence is One Line, ~15 Words

**Strength**: MUST

**Summary**: The first sentence of documentation becomes the summary—keep it to one line and approximately 15 words.

**Example**:
```rust
/// Opens a file at the specified path and returns a handle.
///
/// This function will create the file if it doesn't exist and will
/// truncate it if it does. The file is opened in write-only mode.
///
/// # Errors
///
/// Returns an error if the path is invalid or permissions are insufficient.
pub fn open_file(path: &Path) -> Result<File, IoError> {
    // ...
}

// Bad - first sentence too long, breaks visual flow
/// Opens a file at the specified path and returns a handle to that file which can then be used for various I/O operations.
pub fn open_file(path: &Path) -> Result<File, IoError> {
    // ...
}
```

**Rationale**: Rust documentation extracts the first sentence for module summaries. Keeping it to one line (~15 words) makes API documentation easily skimmable and maintains a clean visual hierarchy.

**See also**: M-FIRST-DOC-SENTENCE

---

## Magic Values

### All Magic Values Must Be Documented

**Strength**: MUST

**Summary**: Hardcoded constants must have comments explaining why the value was chosen, side effects of changing it, and external dependencies.

**Example**:
```rust
// Bad - no explanation
const TIMEOUT: u64 = 86400;

// Better - inline comment
// Wait at most a day; based on api.foo.com timeout policies
const TIMEOUT: u64 = 60 * 60 * 24;

// Best - named constant with full documentation
/// How long we wait for the upstream server.
///
/// This value is large enough to ensure the server can finish processing.
/// Setting this too low might cause us to abort valid requests.
/// Based on `api.foo.com` timeout policies.
const UPSTREAM_SERVER_TIMEOUT: Duration = Duration::from_secs(60 * 60 * 24);

wait_timeout(UPSTREAM_SERVER_TIMEOUT).await
```

**Rationale**: Magic values become maintenance hazards when their purpose is unclear. Documentation prevents future developers (including yourself) from accidentally breaking assumptions or external integrations.

**Required information**:
- Why this value was chosen
- Non-obvious side effects of changing it
- External systems that depend on this value

**See also**: M-DOCUMENTED-MAGIC

---

## Common Traits Implementation

### Public Types Implement Debug

**Strength**: MUST

**Summary**: All public types must implement `Debug`; types with sensitive data must use custom implementations.

**Example**:
```rust
// Good - simple derived Debug
#[derive(Debug)]
pub struct Endpoint(String);

// Good - custom Debug for sensitive data
use std::fmt::{Debug, Formatter};

pub struct UserSecret(String);

impl Debug for UserSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserSecret(...)")
    }
}

#[test]
fn test_secret_debug() {
    let key = "552d3454-d0d5-445d-ab9f-ef2ae3a8896a";
    let secret = UserSecret(key.to_string());
    let rendered = format!("{:?}", secret);
    
    assert!(rendered.contains("UserSecret"));
    assert!(!rendered.contains(key));
}
```

**Rationale**: `Debug` is essential for development and debugging. Custom implementations for sensitive data prevent accidental leakage in logs while maintaining debuggability.

**See also**: M-PUBLIC-DEBUG, C-COMMON-TRAITS

---

## Best Practices Summary

### Quick Reference Table

| Pattern | Strength | Key Insight |
|---------|----------|-------------|
| Avoid weasel words | MUST | Use specific names, not "Service", "Manager", "Factory" |
| Panic = stop program | MUST | Never use panic for error handling |
| Programming bugs panic | MUST | Contract violations should panic, not return errors |
| Regular > associated fn | SHOULD | Keep associated functions for constructors/traits |
| First doc sentence ~15 words | MUST | Enables scannable documentation |
| Document magic values | MUST | Explain why, side effects, external deps |
| Public types have Debug | MUST | Use custom impl for sensitive data |

---

## Related Guidelines

- **Error Handling**: See `03-error-handling.md` for Result/Option patterns
- **Type Design**: See `05-type-design.md` for newtypes and strong typing
- **Documentation**: See `13-documentation.md` for comprehensive doc patterns
- **Anti-patterns**: See `11-anti-patterns.md` for common mistakes

---

## External References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Pragmatic Rust Guidelines: M-CONCISE-NAMES, M-PANIC-IS-STOP, M-PANIC-ON-BUG, M-REGULAR-FN, M-DOCUMENTED-MAGIC, M-PUBLIC-DEBUG
