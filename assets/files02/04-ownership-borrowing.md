# Ownership & Borrowing Guidelines

Patterns for ownership, borrowing, Send/Sync, and making types testable.

## Table of Contents

- [Send and Sync](#send-and-sync)
- [Mockable System Calls](#mockable-system-calls)
- [Type Families](#type-families)
- [Leaking Types](#leaking-types)

---

## Send and Sync

### Types Must Be Send

**Strength**: MUST

**Summary**: All public types should be `Send` for compatibility with async runtimes; futures must always be `Send`.

**Example**:
```rust
use std::rc::Rc;
use std::sync::Arc;

// Bad - not Send, breaks async
pub struct Service {
    data: Rc<String>,  // Rc is !Send
}

async fn process(service: &Service) {
    // Holding service across await makes future !Send
    do_work().await;
    service.use_data();
}

// Good - Send-compatible
pub struct Service {
    data: Arc<String>,  // Arc is Send
}

// Verify Send requirement at compile time
const fn assert_send<T: Send>() {}
const _: () = assert_send::<Service>();

// For futures, verify they're Send
async fn process_send(service: &Service) {
    do_work().await;
    service.use_data();
}

#[test]
fn verify_future_is_send() {
    fn assert_send<T: Send>(_: T) {}
    let service = Service::new();
    assert_send(process_send(&service));
}
```

**Rationale**: Tokio and most async runtimes require `Send` futures. Types that are !Send infect all futures that hold them across await points, making them unusable in most runtime contexts.

**The cost of Send**:
- Atomics vs non-atomics have minimal overhead when uncontended (~64 words between accesses)
- Ideally we'd have `!Send` types for thread-per-core models
- Practically, ecosystem compatibility requires `Send` for wide adoption
- Occasional atomic operations from thread-per-core code has no measurable impact

**When !Send is acceptable**:
- Type is used instantaneously (not held across await)
- Type is explicitly designed for single-threaded use
- Document clearly that type is !Send

**See also**: M-TYPES-SEND

---

## Mockable System Calls

### I/O and System Calls Are Mockable

**Strength**: MUST

**Summary**: Any type doing I/O or system calls with side effects must be mockable for testing.

**Example**:
```rust
// Bad - hardcoded system calls, untestable
pub struct FileProcessor {
    path: PathBuf,
}

impl FileProcessor {
    pub fn process(&self) -> Result<Data, Error> {
        // Direct filesystem access - can't test edge cases!
        let content = std::fs::read_to_string(&self.path)?;
        parse_content(&content)
    }
}

// Good - mockable via enum pattern
pub struct FileProcessor {
    io: IoCore,
}

enum IoCore {
    Native,
    #[cfg(feature = "test-util")]
    Mock(MockCtrl),
}

impl FileProcessor {
    pub fn new() -> Self {
        Self {
            io: IoCore::Native,
        }
    }
    
    #[cfg(feature = "test-util")]
    pub fn new_mocked() -> (Self, MockCtrl) {
        let ctrl = MockCtrl::new();
        let processor = Self {
            io: IoCore::Mock(ctrl.clone()),
        };
        (processor, ctrl)
    }
    
    pub fn process(&self, path: &Path) -> Result<Data, Error> {
        let content = self.io.read_file(path)?;
        parse_content(&content)
    }
}

impl IoCore {
    fn read_file(&self, path: &Path) -> Result<String, Error> {
        match self {
            IoCore::Native => {
                Ok(std::fs::read_to_string(path)?)
            }
            #[cfg(feature = "test-util")]
            IoCore::Mock(ctrl) => {
                ctrl.read_file(path)
            }
        }
    }
}

#[cfg(feature = "test-util")]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    
    #[derive(Clone)]
    pub struct MockCtrl {
        inner: Arc<Mutex<MockState>>,
    }
    
    struct MockState {
        files: HashMap<PathBuf, String>,
    }
    
    impl MockCtrl {
        pub fn new() -> Self {
            Self {
                inner: Arc::new(Mutex::new(MockState {
                    files: HashMap::new(),
                }))
            }
        }
        
        pub fn set_file_content(&self, path: PathBuf, content: String) {
            self.inner.lock().unwrap().files.insert(path, content);
        }
        
        pub(crate) fn read_file(&self, path: &Path) -> Result<String, Error> {
            self.inner.lock().unwrap()
                .files.get(path)
                .cloned()
                .ok_or_else(|| Error::not_found(path))
        }
    }
}

// Usage in tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_not_found() {
        let (processor, _mock) = FileProcessor::new_mocked();
        
        let result = processor.process(Path::new("missing.txt"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_malformed_content() {
        let (processor, mock) = FileProcessor::new_mocked();
        
        mock.set_file_content(
            PathBuf::from("data.txt"),
            "invalid content".to_string()
        );
        
        let result = processor.process(Path::new("data.txt"));
        assert!(result.is_err());
    }
}
```

**Rationale**: Direct system calls make edge cases (file not found, network timeout, permission denied) hard to test. Mocking enables comprehensive testing of error handling paths.

**What needs to be mockable**:
- File and network I/O
- Clocks and time
- Random number generation
- Any operation that is:
  - Non-deterministic
  - Reliant on external state
  - Hardware or environment dependent
  - Not universally reproducible

**Implementation pattern**:
1. Create internal `IoCore` enum with `Native` and `Mock` variants
2. Provide `new()` for production, `new_mocked()` for testing
3. Return `(Self, MockCtrl)` tuple from mock constructor
4. Mock controller implements Clone (Arc<Inner> pattern)
5. Gate mocking behind `test-util` feature

**See also**: M-MOCKABLE-SYSCALLS, M-TEST-UTIL

---

### Test Utilities Must Be Feature-Gated

**Strength**: MUST

**Summary**: Mocking functionality and test utilities must be behind a feature flag.

**Example**:
```rust
// In Cargo.toml:
// [features]
// test-util = []

// Bad - test utilities always compiled
impl HttpClient {
    pub fn bypass_certificate_checks(&mut self) {
        self.skip_verification = true;
    }
}

// Good - gated behind feature
impl HttpClient {
    #[cfg(feature = "test-util")]
    pub fn bypass_certificate_checks(&mut self) {
        self.skip_verification = true;
    }
    
    #[cfg(feature = "test-util")]
    pub fn new_mocked() -> (Self, MockCtrl) {
        // ...
    }
}

// Mock types also gated
#[cfg(feature = "test-util")]
pub mod mock {
    pub struct MockCtrl { /* ... */ }
}
```

**Rationale**: Test utilities can bypass safety checks and shouldn't be available in production builds. Feature gates ensure they're only compiled when explicitly requested.

**What should be feature-gated**:
- Mocking functionality
- Ability to inspect sensitive data
- Safety check overrides
- Fake data generation
- Functions that return `MockCtrl` or similar

**See also**: M-TEST-UTIL

---

## Type Families

### Use the Proper Type Family

**Strength**: MUST

**Summary**: Use the strongest, most appropriate standard library type for your domain.

**Example**:
```rust
use std::path::{Path, PathBuf};

// Bad - wrong type family
pub struct FileConfig {
    path: String,  // Should be PathBuf!
}

pub fn open_file(path: String) -> File {
    // String doesn't handle OS paths correctly
}

// Good - correct type family
pub struct FileConfig {
    path: PathBuf,  // Correct!
}

pub fn open_file(path: impl AsRef<Path>) -> File {
    let path = path.as_ref();
    // Path handles OS-specific concerns
}

// Bad - numeric types in APIs without semantic types
pub fn set_window_size(width: usize, height: usize) {
    // What if someone passes (height, width)?
}

// Good - but keep it simple for obvious cases
pub fn set_window_size(width: usize, height: usize) {
    // This is OK - the names make it clear
}

// For more complex cases, consider newtypes
pub struct Width(usize);
pub struct Height(usize);

pub fn set_window_size(width: Width, height: Height) {
    // Can't swap these at compile time
}
```

**Rationale**: Using the correct type family prevents bugs and leverages Rust's type system. Files and OS paths have platform-specific behavior that `String` doesn't handle.

**Common type families**:

| Domain | Don't Use | Use Instead | Reason |
|--------|-----------|-------------|---------|
| File paths | `String` | `PathBuf` / `Path` | OS path handling |
| OS strings | `String` | `OsString` / `OsStr` | Non-UTF8 support |
| Byte data | `String` | `Vec<u8>` / `&[u8]` | Not always UTF-8 |

**Numeric types**: For simple APIs, regular numeric types are fine. Don't over-engineer with `Saturating<usize>` or `NonZero<usize>` unless there's a clear benefit.

**See also**: M-STRONG-TYPES, C-NEWTYPE

---

## Leaking Types

### Don't Leak External Types

**Strength**: SHOULD

**Summary**: Minimize exposing third-party crate types in public APIs; prefer std types.

**Example**:
```rust
// Bad - leaking external types
use third_party::SpecialString;

pub struct Config {
    pub name: SpecialString,  // Forces all users to depend on third_party
}

pub fn process(data: third_party::Data) -> third_party::Result {
    // Locks users into this crate's version
}

// Good - use std types
pub struct Config {
    pub name: String,  // std type, always available
}

pub fn process(data: Data) -> Result<Output, Error> {
    // Internally convert to/from third_party types
}

// Acceptable - behind feature flag
#[cfg(feature = "serde")]
impl serde::Serialize for Config {
    // OK - users opt-in to serde dependency
}

// Acceptable - part of umbrella crate
// my-runtime crate exports my-runtime-core types
pub use my_runtime_core::Task;  // OK - same ecosystem
```

**Rationale**: Every leaked type becomes part of your API contract. Only std types have stability guarantees. Third-party types create version conflicts and maintenance burden.

**When leaking is acceptable**:
- Behind a feature flag (e.g., `serde`, `tokio`)
- Part of an umbrella crate (sibling crates in same ecosystem)
- Significant benefit for ecosystem interoperability (e.g., `http` crate types)
- The external type is universally used and stable

**Heuristic**:
- Avoid if you can
- OK within umbrella crates
- Behind feature flags for opt-in interop
- Only without feature flag if substantial benefit

**See also**: M-DONT-LEAK-TYPES

---

### Avoid Global Statics

**Strength**: MUST (for consistency-critical state)

**Summary**: Avoid static and thread-local items when a consistent view is required for correctness.

**Example**:
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

// Bad - global state that can be duplicated
static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn increment() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

// Problem: If multiple versions of this crate are linked,
// there are multiple COUNTER instances!
// main:       core v0.4, library_a v0.4 -> core v0.4 ✓ same
// library_b:  core v0.5 -> different COUNTER! ✗

// Good - pass state explicitly
pub struct Counter {
    value: AtomicUsize,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: AtomicUsize::new(0) }
    }
    
    pub fn increment(&self) -> usize {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
}

// Usage - explicit state management
fn main() {
    let counter = Counter::new();
    let count1 = counter.increment();
    let count2 = counter.increment();
    assert_eq!(count2, count1 + 1);  // Always true
}
```

**Rationale**: Rust can link multiple versions of the same crate, each with its own static variables. This "secret duplication" breaks assumptions about global state.

**When statics are OK**:
- Performance optimization (caches, lookup tables)
- State where duplication doesn't affect correctness
- Configuration that's truly global and immutable

**When to avoid statics**:
- Counters, IDs, or sequences
- State that must be consistent across the crate
- Before 1.0 (during 0.x, each minor version can be separate)

**See also**: M-AVOID-STATICS

---

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| Types are Send | MUST | Use Arc, not Rc for shared ownership |
| Futures are Send | MUST | Test with assert_send() |
| I/O is mockable | MUST | Enum pattern with Mock variant |
| Test utils feature-gated | MUST | Use `test-util` feature |
| Proper type families | MUST | PathBuf for paths, not String |
| Don't leak external types | SHOULD | Prefer std types in public APIs |
| Avoid statics | MUST* | *For consistency-critical state |

## Testing Patterns

### Mockable I/O Template

```rust
pub struct Service {
    io: IoCore,
}

enum IoCore {
    Native,
    #[cfg(feature = "test-util")]
    Mock(MockCtrl),
}

impl Service {
    pub fn new() -> Self {
        Self { io: IoCore::Native }
    }
    
    #[cfg(feature = "test-util")]
    pub fn new_mocked() -> (Self, MockCtrl) {
        let ctrl = MockCtrl::new();
        (Self { io: IoCore::Mock(ctrl.clone()) }, ctrl)
    }
}

impl IoCore {
    fn operation(&self) -> Result<Data, Error> {
        match self {
            IoCore::Native => {
                // Real system call
            }
            #[cfg(feature = "test-util")]
            IoCore::Mock(ctrl) => ctrl.operation(),
        }
    }
}

#[cfg(feature = "test-util")]
pub mod mock {
    #[derive(Clone)]
    pub struct MockCtrl {
        inner: Arc<Mutex<MockState>>,
    }
    
    impl MockCtrl {
        pub fn new() -> Self { /* ... */ }
        pub fn set_behavior(&self, /* ... */) { /* ... */ }
        pub(crate) fn operation(&self) -> Result<Data, Error> { /* ... */ }
    }
}
```

## Related Guidelines

- **API Design**: See `02-api-design.md` for services and builders
- **Type Design**: See `05-type-design.md` for newtypes
- **Concurrency**: See `07-concurrency-async.md` for async patterns

## External References

- [Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html)
- Pragmatic Rust: M-TYPES-SEND, M-MOCKABLE-SYSCALLS, M-TEST-UTIL, M-STRONG-TYPES
