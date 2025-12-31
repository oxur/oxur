# Anti-Patterns

Common mistakes and what NOT to do in Rust. **This section is critical for AI code generation.**

## Table of Contents

- [Type System Misuse](#type-system-misuse)
- [Error Handling](#error-handling)
- [Ownership and Borrowing](#ownership-and-borrowing)
- [API Design](#api-design)
- [Performance](#performance)
- [Safety](#safety)

---

## Type System Misuse

### Don't Use String for Everything

**Strength**: AVOID

**Problem**: Using `String` for file paths, IDs, or data that has specific types.

**Example**:
```rust
// WRONG
pub struct Config {
    path: String,  // Should be PathBuf
    user_id: String,  // Should be UserId newtype
    timeout: String,  // Should be Duration
}

// CORRECT
use std::path::PathBuf;
use std::time::Duration;

pub struct UserId(u64);

pub struct Config {
    path: PathBuf,
    user_id: UserId,
    timeout: Duration,
}
```

**Why this is wrong**: `String` loses type safety, enables parameter confusion, and doesn't handle platform-specific concerns (like path separators).

**See also**: M-STRONG-TYPES, primitive obsession

---

### Don't Expose Smart Pointers in APIs

**Strength**: AVOID

**Problem**: Forcing callers to use `Arc<T>`, `Rc<T>`, `Box<T>` in function signatures.

**Example**:
```rust
// WRONG - exposes implementation details
pub fn process(data: Arc<Mutex<Data>>) -> Box<Result> {
    // Forces all callers to use Arc<Mutex<>>
}

// CORRECT - clean interface
pub fn process(data: &Data) -> Result {
    // Callers pass simple reference
}

// If you need Arc internally, hide it
pub struct Service {
    data: Arc<Mutex<Data>>,  // Hidden
}

impl Service {
    pub fn process(&self) -> Result {
        let data = self.data.lock().unwrap();
        // ...
    }
}
```

**Why this is wrong**: Smart pointers are implementation details. Exposing them makes APIs inflexible and forces infectious type complexity on all callers.

**See also**: M-AVOID-WRAPPERS

---

### Don't Nest Generics Deeply

**Strength**: AVOID

**Problem**: Creating types like `Service<Backend<Store<Data>>>` in APIs.

**Example**:
```rust
// WRONG - excessive nesting
pub struct App {
    service: Service<Backend<Store<Config, Data>>>,
}

// Users must name this monster type!

// CORRECT - hide complexity
pub struct Service {
    backend: Backend,  // Concrete type or hidden generic
}

pub struct Backend {
    store: Store,  // Hide generic parameters
}

// Or provide type alias at module level
pub type AppService = Service<DefaultBackend>;
```

**Why this is wrong**: Nested generics create cognitive load, confusing error messages, and make types hard to name. Service-level types should not nest more than 1 level deep.

**See also**: M-SIMPLE-ABSTRACTIONS

---

## Error Handling

### Don't Use Panic for Error Handling

**Strength**: AVOID

**Problem**: Using `panic!()`, `unwrap()`, or `expect()` for recoverable errors.

**Example**:
```rust
// WRONG - panic on recoverable error
pub fn load_config(path: &Path) -> Config {
    let content = std::fs::read_to_string(path)
        .unwrap();  // DON'T DO THIS
    
    toml::from_str(&content)
        .expect("invalid config")  // OR THIS
}

// CORRECT - return Result
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;
    Ok(config)
}

// unwrap() is OK in:
// - Tests
// - Prototypes
// - When the error literally cannot happen
fn example() {
    let x = Some(42);
    let value = x.unwrap();  // OK - we know it's Some
}
```

**Why this is wrong**: Panics cannot be reliably caught and can cause program termination. Users compiling with `panic = "abort"` can't recover. Use `Result` for recoverable errors.

**See also**: M-PANIC-IS-STOP

---

### Don't Return Error When You Should Panic

**Strength**: AVOID

**Problem**: Returning `Result` for programming errors that should panic.

**Example**:
```rust
// WRONG - contract violation returns error
pub fn divide(x: u32, y: u32) -> Result<u32, MathError> {
    if y == 0 {
        return Err(MathError::DivideByZero);
    }
    Ok(x / y)
}

// CORRECT - contract violation panics
pub fn divide(x: u32, y: u32) -> u32 {
    if y == 0 {
        panic!("divide: divisor cannot be zero");
    }
    x / y
}

// OR make it impossible to construct invalid input
pub struct NonZero(u32);

impl NonZero {
    pub fn new(value: u32) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }
}

pub fn divide(x: u32, y: NonZero) -> u32 {
    x / y.0
}
```

**Why this is wrong**: Programming errors have no valid recovery path. Returning `Result` creates impossible error handling code. Use types to prevent invalid states.

**See also**: M-PANIC-ON-BUG

---

### Don't Expose ErrorKind Enums

**Strength**: AVOID

**Problem**: Making error kind enums public, forcing exhaustive matching.

**Example**:
```rust
// WRONG - public enum breaks when adding variants
pub enum ErrorKind {
    Io(std::io::Error),
    Parse(ParseError),
}

pub struct Error {
    pub kind: ErrorKind,  // Public!
}

// User code breaks when you add variants
match error.kind {
    ErrorKind::Io(_) => { },
    ErrorKind::Parse(_) => { },
    // Adding ErrorKind::Network is breaking change!
}

// CORRECT - private enum, public methods
pub struct Error {
    kind: ErrorKind,  // Private
}

enum ErrorKind {
    Io(std::io::Error),
    Parse(ParseError),
}

impl Error {
    pub fn is_io(&self) -> bool {
        matches!(self.kind, ErrorKind::Io(_))
    }
    
    pub fn is_parse(&self) -> bool {
        matches!(self.kind, ErrorKind::Parse(_))
    }
}
```

**Why this is wrong**: Public enums require exhaustive matching, making adding new variants a breaking change. Helper methods provide stable API.

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

## Ownership and Borrowing

### Don't Clone When You Can Borrow

**Strength**: AVOID

**Problem**: Cloning data unnecessarily instead of using references.

**Example**:
```rust
// WRONG - unnecessary clone
fn process_user(user: User) -> String {
    let name = user.name.clone();  // Why clone?
    format!("Hello, {}", name)
}

// CORRECT - borrow instead
fn process_user(user: &User) -> String {
    format!("Hello, {}", user.name)
}

// WRONG - clone in loop
fn process_all(users: &[User]) {
    for user in users {
        let user_copy = user.clone();  // Cloning every iteration!
        process(user_copy);
    }
}

// CORRECT - use references
fn process_all(users: &[User]) {
    for user in users {
        process(user);  // Just borrow
    }
}
```

**Why this is wrong**: Unnecessary clones waste CPU and memory. Borrow when you just need to read data.

**When cloning is necessary**:
- Moving into spawned thread/task
- Building a collection from borrowed data
- Explicitly requested by user (`.to_owned()`, `.clone()`)

---

### Don't Fight the Borrow Checker with unsafe

**Strength**: AVOID

**Problem**: Using `unsafe` to work around borrow checker limitations.

**Example**:
```rust
// WRONG - unsafe to bypass borrow checker
fn get_mut_twice(v: &mut Vec<i32>) -> (&mut i32, &mut i32) {
    unsafe {
        let ptr = v.as_mut_ptr();
        (&mut *ptr, &mut *ptr.add(1))  // UNSOUND!
    }
}

// CORRECT - use split_at_mut
fn get_mut_twice(v: &mut [i32]) -> (&mut i32, &mut i32) {
    let (left, right) = v.split_at_mut(1);
    (&mut left[0], &mut right[0])
}

// Or restructure your code
fn process_separately(v: &mut Vec<i32>) {
    process_first(&mut v[0]);
    process_second(&mut v[1]);
}
```

**Why this is wrong**: The borrow checker is usually right. Using `unsafe` to bypass it often introduces undefined behavior. Restructure your code or use safe APIs like `split_at_mut`.

---

### Don't Use Rc in Async Code

**Strength**: AVOID

**Problem**: Using `Rc<T>` in async functions, making futures !Send.

**Example**:
```rust
use std::rc::Rc;
use std::sync::Arc;

// WRONG - Rc makes future !Send
async fn process_data(data: Rc<String>) {
    fetch_from_db().await;
    println!("{}", data);
}
// Can't be used with Tokio!

// CORRECT - use Arc
async fn process_data(data: Arc<String>) {
    fetch_from_db().await;
    println!("{}", data);
}
```

**Why this is wrong**: `Rc` is !Send, which makes any future holding it across `.await` also !Send. Most async runtimes require Send futures.

**See also**: M-TYPES-SEND

---

## API Design

### Don't Use Associated Functions for Everything

**Strength**: AVOID

**Problem**: Putting unrelated functions under a type as associated functions.

**Example**:
```rust
// WRONG - unrelated logic as associated function
struct Database;

impl Database {
    pub fn new() -> Self { Self }  // OK
    
    pub fn query(&self) { }  // OK
    
    // NOT OK - doesn't need to be under Database
    pub fn validate_sql(query: &str) -> bool {
        // This is a free function masquerading as associated fn
    }
}

// CORRECT - regular function
fn validate_sql(query: &str) -> bool {
    // Just a function!
}

impl Database {
    pub fn new() -> Self { Self }
    pub fn query(&self) { }
}
```

**Why this is wrong**: Associated functions should be for constructors or methods. Unrelated logic should be regular functions for better discoverability.

**See also**: M-REGULAR-FN

---

### Don't Use Builder for Simple Types

**Strength**: AVOID

**Problem**: Creating builders for types with 1-2 parameters.

**Example**:
```rust
// WRONG - overkill for simple type
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn builder() -> PointBuilder { }
}

// CORRECT - just use regular constructor
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

// Builders are for 4+ optional parameters
pub struct Config {
    // Many optional fields...
}

impl Config {
    pub fn builder() -> ConfigBuilder { }  // OK
}
```

**Why this is wrong**: Builders add complexity. Use them only when you have 4+ optional parameters or complex initialization.

**See also**: M-INIT-BUILDER

---

### Don't Glob Re-export

**Strength**: AVOID

**Problem**: Using `pub use module::*` to re-export items.

**Example**:
```rust
// WRONG - hard to review, may export unintended items
pub use internal::*;

// CORRECT - explicit exports
pub use internal::{Foo, Bar, Baz};

// Exception: platform-specific HAL modules
#[cfg(target_os = "windows")]
pub use windows_impl::*;  // OK for platform abstraction
```

**Why this is wrong**: Glob exports are hard to review in PRs and can accidentally expose internal items. Explicit exports make the public API clear.

**See also**: M-NO-GLOB-REEXPORTS

---

## Performance

### Don't Prematurely Optimize

**Strength**: AVOID

**Problem**: Optimizing without profiling or benchmarking first.

**Example**:
```rust
// WRONG - complex "optimization" without measurement
pub fn process(items: &[Item]) -> Vec<Result> {
    // "Optimized" with unsafe and manual memory management
    unsafe {
        let mut results = Vec::with_capacity(items.len());
        let ptr = results.as_mut_ptr();
        for (i, item) in items.iter().enumerate() {
            ptr.add(i).write(process_one(item));
        }
        results.set_len(items.len());
        results
    }
}

// CORRECT - simple, safe, probably fast enough
pub fn process(items: &[Item]) -> Vec<Result> {
    items.iter().map(process_one).collect()
}

// If profiling shows this is slow, THEN optimize
```

**Why this is wrong**: Premature optimization wastes time and adds complexity. Profile first, optimize hot paths, benchmark improvements.

**See also**: M-HOTPATH

---

### Don't Allocate in Loops

**Strength**: AVOID

**Problem**: Creating new `String` or `Vec` every iteration when buffer can be reused.

**Example**:
```rust
// WRONG - allocates every iteration
for i in 0..1000 {
    let msg = format!("Item {}", i);
    process(&msg);
}

// CORRECT - reuse buffer
let mut msg = String::with_capacity(50);
for i in 0..1000 {
    use std::fmt::Write;
    msg.clear();
    write!(&mut msg, "Item {}", i).unwrap();
    process(&msg);
}

// WRONG - collect intermediate
let results: Vec<_> = items.iter()
    .map(|x| expensive_transform(x))
    .collect();
for result in results {
    // ...
}

// CORRECT - process directly
for item in items {
    let result = expensive_transform(item);
    // ...
}
```

**Why this is wrong**: Repeated allocations are expensive. Reuse buffers when possible.

---

## Safety

### Don't Write Unsound Code

**Strength**: AVOID

**Problem**: Writing safe code that can cause undefined behavior.

**Example**:
```rust
// WRONG - unsound transmute
pub fn as_u128<T>(x: &T) -> &u128 {
    unsafe { std::mem::transmute(x) }
    // UB if T is not exactly 16 bytes!
}

// WRONG - unsound Send impl
struct NotSend {
    ptr: *const u8,  // Raw pointer is !Send for a reason
}

unsafe impl Send for NotSend { }  // UNSOUND!

// WRONG - violating aliasing rules
fn get_two_mut<T>(slice: &mut [T]) -> (&mut T, &mut T) {
    unsafe {
        let ptr = slice.as_mut_ptr();
        (&mut *ptr, &mut *ptr)  // ALIASING VIOLATION!
    }
}

// CORRECT - use safe alternatives
fn get_two_mut<T>(slice: &mut [T]) -> Option<(&mut T, &mut T)> {
    if slice.len() < 2 {
        return None;
    }
    let (first, rest) = slice.split_first_mut().unwrap();
    let second = &mut rest[0];
    Some((first, second))
}
```

**Why this is wrong**: Unsound code can cause undefined behavior even when called from safe code. This violates Rust's safety guarantees.

**See also**: M-UNSOUND

---

### Don't Use unsafe Without Clear Safety Comments

**Strength**: AVOID

**Problem**: Using `unsafe` without documenting why it's safe.

**Example**:
```rust
// WRONG - no safety documentation
pub fn from_raw_parts(ptr: *const u8, len: usize) -> Vec<u8> {
    unsafe {
        Vec::from_raw_parts(ptr as *mut u8, len, len)
    }
}

// CORRECT - document safety requirements
/// Creates a Vec from raw parts.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is valid for reads of `len` bytes
/// - `ptr` points to `len` consecutive properly initialized `u8` values  
/// - The memory pointed to by `ptr` is not accessed after this function
/// - The memory was allocated with the global allocator
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Vec<u8> {
    unsafe {
        // SAFETY: Caller guarantees all requirements above
        Vec::from_raw_parts(ptr as *mut u8, len, len)
    }
}
```

**Why this is wrong**: Unsafe code requires careful reasoning. Without documentation, reviewers can't verify safety and future maintainers may violate invariants.

**See also**: M-UNSAFE

---

## Summary of Anti-Patterns

| Anti-Pattern | Instead Do | Why |
|-------------|------------|-----|
| String for paths | PathBuf | Platform-specific handling |
| String for IDs | Newtype | Type safety |
| Smart pointers in APIs | &T or T | Hide implementation |
| Deep generic nesting | Type aliases | Reduce complexity |
| Panic for errors | Result | Recoverability |
| Result for bugs | Panic | No recovery path |
| Public ErrorKind | is_xxx() methods | Stability |
| Clone in loops | Borrow | Performance |
| Rc in async | Arc | Send requirement |
| Associated fn for unrelated logic | Regular fn | Discoverability |
| Builder for simple types | new() | Simplicity |
| Glob re-exports | Explicit use | Clarity |
| Premature optimization | Profile first | Wasted effort |
| Allocate in hot loop | Reuse buffer | Performance |
| Unsound unsafe | Safe alternatives | Correctness |
| Undocumented unsafe | Safety comments | Reviewability |

## Critical Reminders for AI

These patterns are **especially common in AI-generated code**:

1. ❌ Using `String` everywhere instead of proper types
2. ❌ `unwrap()` and `expect()` for error handling
3. ❌ Exposing `Arc<Mutex<T>>` in function signatures
4. ❌ Cloning unnecessarily
5. ❌ Using `Rc` in async code
6. ❌ Creating builders for simple types
7. ❌ Putting unrelated logic in `impl` blocks
8. ❌ Using `unsafe` without safety comments

## Related Guidelines

- **Core Idioms**: See `01-core-idioms.md` for proper panic usage
- **Error Handling**: See `03-error-handling.md` for error design
- **Type Design**: See `05-type-design.md` for newtypes
- **Performance**: See `08-performance.md` for optimization

## External References

- [Rust Anti-Patterns](https://rust-unofficial.github.io/patterns/anti_patterns/index.html)
- [Common Rust Lifetime Misconceptions](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md)
