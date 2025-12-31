# Anti-Patterns: What NOT to Do

> This section is **critical for AI code generation**. These are common mistakes that compile but produce suboptimal, confusing, or incorrect code.

---

## AP-01: Clone to Satisfy the Borrow Checker

**Strength**: AVOID

**Summary**: Using `.clone()` to make borrow checker errors disappear without understanding why.

```rust
// ❌ BAD: Cloning to avoid borrow issues
let mut x = expensive_data();
let y = &mut (x.clone());  // Clone just to get a mutable reference
process(y);
println!("{:?}", x);  // x and y are now desynchronized!

// ✅ GOOD: Restructure to avoid the need for clone
let mut x = expensive_data();
process(&mut x);
println!("{:?}", x);

// ✅ GOOD: If you truly need independent copies, be explicit about why
let x = expensive_data();
let mut y = x.clone();  // Intentional: y needs independent mutation
process(&mut y);
// x remains unchanged, y is modified — this is the intended behavior
```

**Rationale**: Cloning silently creates independent copies. Changes to one don't affect the other, which may not be intended. It also has performance cost. If you need to clone, ensure it's deliberate.

**Exceptions**: 
- `Rc<T>` and `Arc<T>` clones are cheap (just increment reference count)
- Prototyping where correctness matters more than performance
- When the borrow checker issue is genuinely complex and clone is the clearest solution

**See also**: `mem::take`, `mem::replace` for zero-cost alternatives in enums

---

## AP-02: `#![deny(warnings)]` in Library Code

**Strength**: AVOID (in libraries), CONSIDER (in binaries with CI)

**Summary**: Using `#![deny(warnings)]` can break builds when Rust adds new lints or deprecates APIs.

```rust
// ❌ BAD: In a library crate
#![deny(warnings)]  // Future Rust versions may add warnings, breaking downstream users

// ✅ GOOD: Deny specific lints you care about
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]

// ✅ GOOD: Use RUSTFLAGS in CI instead
// In CI: RUSTFLAGS="-D warnings" cargo build
```

**Rationale**: Rust's stability guarantee means code that compiles today will compile tomorrow. But `deny(warnings)` opts out of this — a new warning in a future Rust version becomes a hard error, breaking your users' builds.

**Exceptions**: Binary crates where you control the entire build pipeline.

---

## AP-03: Deref Polymorphism (Fake Inheritance)

**Strength**: AVOID

**Summary**: Misusing `Deref` to simulate OOP inheritance.

```rust
// ❌ BAD: Using Deref for inheritance-like behavior
use std::ops::Deref;

struct Animal { name: String }
impl Animal {
    fn speak(&self) { println!("{} makes a sound", self.name); }
}

struct Dog { animal: Animal }
impl Deref for Dog {
    type Target = Animal;
    fn deref(&self) -> &Animal { &self.animal }
}

fn main() {
    let dog = Dog { animal: Animal { name: "Rex".into() } };
    dog.speak();  // Works via Deref, but this is NOT idiomatic
}

// ✅ GOOD: Use composition with explicit delegation or traits
trait Speaker {
    fn speak(&self);
}

struct Animal { name: String }
struct Dog { name: String }

impl Speaker for Animal {
    fn speak(&self) { println!("{} makes a sound", self.name); }
}

impl Speaker for Dog {
    fn speak(&self) { println!("{} barks", self.name); }
}
```

**Rationale**: `Deref` is designed for smart pointers (like `Box<T>` → `T`), not for type relationships. Using it for inheritance:
- Confuses readers expecting pointer-like behavior
- Doesn't create proper subtyping (traits won't auto-implement)
- Breaks generic programming expectations

---

## AP-04: Returning `impl Trait` When Concrete Type Would Work

**Strength**: CONSIDER (avoiding)

**Summary**: Using `-> impl Trait` when the concrete type is simple and public.

```rust
// ❌ QUESTIONABLE: Hiding a simple public type
fn get_numbers() -> impl Iterator<Item = i32> {
    vec![1, 2, 3].into_iter()
}

// ✅ BETTER: Return concrete type when it's simple and stable
fn get_numbers() -> std::vec::IntoIter<i32> {
    vec![1, 2, 3].into_iter()
}

// ✅ GOOD USE: When the type is complex or an implementation detail
fn get_filtered_numbers(data: &[i32]) -> impl Iterator<Item = &i32> {
    data.iter().filter(|&&x| x > 0).map(|x| x)
    // The actual type here is deeply nested and unstable
}
```

**Rationale**: `impl Trait` hides the concrete type, preventing callers from:
- Naming the type in their own signatures
- Using type-specific methods not in the trait
- Storing in structs without boxing

Use it when the concrete type is complex, unstable, or truly an implementation detail.

---

## AP-05: Stringly Typed APIs

**Strength**: AVOID

**Summary**: Using `String` or `&str` where an enum or newtype would provide type safety.

```rust
// ❌ BAD: Stringly typed
fn set_log_level(level: &str) {
    match level {
        "debug" | "DEBUG" => { /* ... */ }
        "info" | "INFO" => { /* ... */ }
        _ => panic!("Unknown level"),  // Runtime error!
    }
}

// ✅ GOOD: Type-safe enum
#[derive(Debug, Clone, Copy)]
enum LogLevel { Debug, Info, Warn, Error }

fn set_log_level(level: LogLevel) {
    match level {
        LogLevel::Debug => { /* ... */ }
        LogLevel::Info => { /* ... */ }
        // Compiler ensures all variants handled
    }
}

// ✅ GOOD: Newtype for validated strings
struct Username(String);

impl Username {
    fn new(s: &str) -> Result<Self, ValidationError> {
        if s.len() >= 3 && s.chars().all(|c| c.is_alphanumeric()) {
            Ok(Username(s.to_string()))
        } else {
            Err(ValidationError::InvalidUsername)
        }
    }
}
```

**Rationale**: Strings bypass the type system. Typos become runtime errors. Enums and newtypes catch errors at compile time.

---

## AP-06: `unwrap()` in Library Code

**Strength**: AVOID (in libraries), CONSIDER carefully (in application code)

**Summary**: Using `.unwrap()` propagates `None`/`Err` as panics instead of proper error handling.

```rust
// ❌ BAD: Panics on invalid input
fn parse_config(input: &str) -> Config {
    let value: i32 = input.parse().unwrap();  // Panics on bad input!
    Config { value }
}

// ✅ GOOD: Return Result, let caller decide
fn parse_config(input: &str) -> Result<Config, ParseIntError> {
    let value: i32 = input.parse()?;
    Ok(Config { value })
}

// ✅ ACCEPTABLE: When you can prove it won't fail
fn get_first_char(s: &str) -> char {
    debug_assert!(!s.is_empty(), "precondition: s must not be empty");
    s.chars().next().unwrap()  // Safe: we verified precondition
}

// ✅ BETTER: Use expect() with explanation
fn known_safe_operation() -> Value {
    STATIC_MAP.get("known_key")
        .expect("known_key is always present in STATIC_MAP")
}
```

**Rationale**: `unwrap()` in libraries forces a panic on callers who may want to handle errors gracefully. Use `?` to propagate errors. Use `expect()` with a message when you're certain it won't fail.

**Clippy**: `clippy::unwrap_used`, `clippy::expect_used`

---

## AP-07: `collect()` Without Type Annotation

**Strength**: AVOID

**Summary**: Calling `.collect()` without specifying the target type.

```rust
// ❌ BAD: Type inference fails or is unclear
let numbers = vec![1, 2, 3];
let doubled = numbers.iter().map(|x| x * 2).collect();  // Error or unclear

// ✅ GOOD: Turbofish syntax
let doubled = numbers.iter().map(|x| x * 2).collect::<Vec<_>>();

// ✅ GOOD: Type annotation on binding
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

// ✅ GOOD: Type annotation with inference
let doubled: Vec<_> = numbers.iter().map(|x| x * 2).collect();
```

**Rationale**: `collect()` can produce many types (`Vec`, `HashSet`, `String`, `Result<Vec<_>, E>`, etc.). Without annotation, the compiler can't infer which you want.

---

## AP-08: `&String` and `&Vec<T>` in Function Parameters

**Strength**: AVOID

**Summary**: Accepting `&String` instead of `&str`, or `&Vec<T>` instead of `&[T]`.

```rust
// ❌ BAD: Overly restrictive parameter types
fn process_name(name: &String) { /* ... */ }
fn sum_values(values: &Vec<i32>) -> i32 { /* ... */ }

// Callers forced to have String/Vec:
let s = "hello";  // &str
process_name(&s.to_string());  // Unnecessary allocation!

// ✅ GOOD: Use borrowed slices
fn process_name(name: &str) { /* ... */ }
fn sum_values(values: &[i32]) -> i32 { /* ... */ }

// Now accepts both:
process_name("hello");           // &str works directly
process_name(&String::from("x")); // &String coerces to &str
sum_values(&[1, 2, 3]);          // Array slice
sum_values(&vec![1, 2, 3]);      // Vec coerces to &[T]
```

**Rationale**: `&str` and `&[T]` are strictly more general. `&String` adds an unnecessary layer of indirection and restricts callers.

**Clippy**: `clippy::ptr_arg`

---

## AP-09: Manual `Drop` for Non-Resource Types

**Strength**: AVOID

**Summary**: Implementing `Drop` when there's no resource to clean up.

```rust
// ❌ BAD: Drop for side effects on plain data
struct Counter { value: i32 }

impl Drop for Counter {
    fn drop(&mut self) {
        println!("Counter dropped with value {}", self.value);
    }
}

// ✅ GOOD: Drop only for resource management
struct FileHandle { fd: RawFd }

impl Drop for FileHandle {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd); }
    }
}

// ✅ GOOD: Use a guard pattern for scoped behavior
struct LogOnDrop<'a>(&'a str);

impl Drop for LogOnDrop<'_> {
    fn drop(&mut self) {
        log::debug!("Exiting scope: {}", self.0);
    }
}
```

**Rationale**: `Drop` implies resource ownership (files, sockets, locks). Using it for logging or side effects is surprising and can cause issues with `std::mem::forget`.

---

## AP-10: Boolean Parameters

**Strength**: AVOID (for public APIs)

**Summary**: Using `bool` parameters whose meaning isn't clear at the call site.

```rust
// ❌ BAD: What do these booleans mean?
process_file("data.txt", true, false, true);

// ✅ GOOD: Use enums for clarity
enum Overwrite { Yes, No }
enum CreateDirs { Yes, No }
enum Verbose { Yes, No }

process_file("data.txt", Overwrite::Yes, CreateDirs::No, Verbose::Yes);

// ✅ GOOD: Use builder pattern for many options
ProcessFile::new("data.txt")
    .overwrite(true)
    .create_dirs(false)
    .verbose(true)
    .run();
```

**Rationale**: `true` and `false` at call sites tell readers nothing about what they control. Enums are self-documenting.

---

## AP-11: `match` on `Option`/`Result` Instead of Combinators

**Strength**: CONSIDER (avoiding excessive match)

**Summary**: Using verbose `match` when combinators are cleaner.

```rust
// ❌ VERBOSE: Match when combinators work
fn get_username(id: i32) -> Option<String> {
    let user = match find_user(id) {
        Some(u) => u,
        None => return None,
    };
    match user.name {
        Some(n) => Some(n.to_uppercase()),
        None => None,
    }
}

// ✅ CONCISE: Combinators
fn get_username(id: i32) -> Option<String> {
    find_user(id)?
        .name
        .map(|n| n.to_uppercase())
}

// ✅ ALSO GOOD: When match adds clarity for complex logic
fn process(value: Option<i32>) -> String {
    match value {
        Some(x) if x > 100 => format!("Large: {x}"),
        Some(x) if x > 0 => format!("Positive: {x}"),
        Some(x) => format!("Non-positive: {x}"),
        None => "No value".to_string(),
    }
}
```

**Rationale**: Combinators like `map`, `and_then`, `unwrap_or`, and `?` often express intent more clearly than `match`. But `match` is better when you have complex conditions or multiple patterns.

---

## AP-12: `Box<dyn Error>` Without `Send + Sync`

**Strength**: AVOID (in async/threaded code)

**Summary**: Using `Box<dyn Error>` which isn't thread-safe.

```rust
// ❌ BAD: Won't work across thread/async boundaries
fn fetch_data() -> Result<Data, Box<dyn std::error::Error>> {
    // ...
}

// ✅ GOOD: Add Send + Sync for thread safety
fn fetch_data() -> Result<Data, Box<dyn std::error::Error + Send + Sync>> {
    // ...
}

// ✅ BETTER: Use anyhow for applications
fn fetch_data() -> anyhow::Result<Data> {
    // ...
}

// ✅ BETTER: Use thiserror for libraries
#[derive(Debug, thiserror::Error)]
enum FetchError {
    #[error("network error: {0}")]
    Network(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

fn fetch_data() -> Result<Data, FetchError> {
    // ...
}
```

**Rationale**: Async runtimes and thread pools require `Send + Sync`. Plain `Box<dyn Error>` can't cross these boundaries.

---

## AP-13: Ignoring `#[must_use]` Warnings

**Strength**: MUST NOT (ignore)

**Summary**: Discarding `Result` or other `#[must_use]` values.

```rust
// ❌ BAD: Ignoring Result
fn write_config(config: &Config) {
    std::fs::write("config.json", serde_json::to_string(config).unwrap());
    // Warning: unused Result! Write might have failed!
}

// ✅ GOOD: Handle the Result
fn write_config(config: &Config) -> std::io::Result<()> {
    std::fs::write("config.json", serde_json::to_string(config)?)?;
    Ok(())
}

// ✅ ACCEPTABLE: Explicitly ignore when you truly don't care
fn try_delete_temp_file(path: &Path) {
    let _ = std::fs::remove_file(path);  // Explicit ignore
}
```

**Rationale**: `#[must_use]` indicates the return value contains important information (success/failure, a computed value). Ignoring it usually means losing error information.

**Clippy**: `clippy::let_underscore_must_use`

---

## AP-14: `to_string()` in Hot Loops

**Strength**: AVOID

**Summary**: Repeatedly allocating strings in performance-critical loops.

```rust
// ❌ BAD: Allocating in every iteration
fn find_user(users: &[User], target: &str) -> Option<&User> {
    for user in users {
        if user.name.to_lowercase() == target.to_lowercase() {
            return Some(user);
        }
    }
    None
}

// ✅ GOOD: Allocate once outside loop
fn find_user(users: &[User], target: &str) -> Option<&User> {
    let target_lower = target.to_lowercase();
    users.iter().find(|u| u.name.to_lowercase() == target_lower)
}

// ✅ BETTER: Use case-insensitive comparison without allocation
fn find_user(users: &[User], target: &str) -> Option<&User> {
    users.iter().find(|u| u.name.eq_ignore_ascii_case(target))
}
```

**Rationale**: `to_string()`, `to_lowercase()`, `format!()` all allocate. In hot loops, this causes many small allocations which hurt performance.

---

## AP-15: `pub` Fields When Invariants Exist

**Strength**: AVOID

**Summary**: Making struct fields public when they have invariants that must be maintained.

```rust
// ❌ BAD: Public field with implicit invariant
pub struct PositiveInt {
    pub value: i32,  // Should be > 0, but anyone can set it to -5!
}

// ✅ GOOD: Private field with accessor
pub struct PositiveInt {
    value: i32,
}

impl PositiveInt {
    pub fn new(value: i32) -> Option<Self> {
        if value > 0 { Some(Self { value }) } else { None }
    }
    
    pub fn get(&self) -> i32 { self.value }
    
    pub fn set(&mut self, value: i32) -> Result<(), InvalidValue> {
        if value > 0 {
            self.value = value;
            Ok(())
        } else {
            Err(InvalidValue)
        }
    }
}
```

**Rationale**: Public fields allow anyone to put the struct into an invalid state. Encapsulation ensures invariants are always maintained.

---

## AP-16: Overly Generic Functions

**Strength**: CONSIDER (avoiding)

**Summary**: Using generics when concrete types would be clearer and compile faster.

```rust
// ❌ QUESTIONABLE: Generic for no reason
fn print_value<T: std::fmt::Display>(value: T) {
    println!("{}", value);
}

// ✅ SIMPLER: If you only ever pass strings
fn print_value(value: &str) {
    println!("{}", value);
}

// ✅ GOOD USE OF GENERICS: When flexibility is needed
fn find_in_slice<T: PartialEq>(slice: &[T], target: &T) -> Option<usize> {
    slice.iter().position(|x| x == target)
}
```

**Rationale**: Generics increase compile time (monomorphization) and can make error messages harder to understand. Use them when you need the flexibility, not by default.

---

## AP-17: Nested `Result<Option<Result<...>>>`

**Strength**: AVOID

**Summary**: Deeply nested `Result`/`Option` types that are hard to work with.

```rust
// ❌ BAD: Triple-nested nightmare
fn fetch_user_age(id: i32) -> Result<Option<Result<i32, ParseError>>, DbError> {
    // What does None mean? What about the inner Result?
}

// ✅ GOOD: Flatten with custom error type
#[derive(Debug, thiserror::Error)]
enum FetchAgeError {
    #[error("database error: {0}")]
    Db(#[from] DbError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("user not found")]
    NotFound,
}

fn fetch_user_age(id: i32) -> Result<i32, FetchAgeError> {
    let user = db.find_user(id)?.ok_or(FetchAgeError::NotFound)?;
    let age = user.age_str.parse()?;
    Ok(age)
}
```

**Rationale**: Nested wrappers are hard to destructure and unclear in meaning. Custom error types document what can go wrong.

---

## AP-18: Synchronous I/O in Async Functions

**Strength**: MUST NOT

**Summary**: Using blocking I/O (`std::fs`, `std::net`) inside async functions.

```rust
// ❌ BAD: Blocks the async runtime!
async fn read_config() -> Config {
    let contents = std::fs::read_to_string("config.json").unwrap();  // BLOCKS!
    serde_json::from_str(&contents).unwrap()
}

// ✅ GOOD: Use async I/O
async fn read_config() -> Result<Config, Error> {
    let contents = tokio::fs::read_to_string("config.json").await?;
    Ok(serde_json::from_str(&contents)?)
}

// ✅ GOOD: Use spawn_blocking for unavoidable sync I/O
async fn compute_hash(data: Vec<u8>) -> Hash {
    tokio::task::spawn_blocking(move || {
        expensive_hash_function(&data)  // CPU-bound, OK to block
    }).await.unwrap()
}
```

**Rationale**: Async runtimes multiplex many tasks on few threads. Blocking I/O prevents the thread from running other tasks, destroying concurrency benefits.

---

## AP-19: `Rc<RefCell<T>>` When `Cell<T>` Suffices

**Strength**: CONSIDER (simpler alternatives)

**Summary**: Using `Rc<RefCell<T>>` for interior mutability when simpler types work.

```rust
// ❌ OVERKILL: RefCell for a simple counter
use std::cell::RefCell;
use std::rc::Rc;

struct Counter {
    value: Rc<RefCell<i32>>,
}

impl Counter {
    fn increment(&self) {
        *self.value.borrow_mut() += 1;
    }
}

// ✅ SIMPLER: Cell for Copy types
use std::cell::Cell;

struct Counter {
    value: Cell<i32>,
}

impl Counter {
    fn increment(&self) {
        self.value.set(self.value.get() + 1);
    }
}
```

**Rationale**: 
- `Cell<T>` (for `Copy` types): No runtime borrow checking, just copies values
- `RefCell<T>`: Runtime borrow checking, can panic
- `Rc<T>`: Reference counting overhead

Use the simplest type that works.

---

## AP-20: Magic Numbers and Strings

**Strength**: AVOID

**Summary**: Using literal values without named constants.

```rust
// ❌ BAD: Magic numbers
fn calculate_tax(amount: f64) -> f64 {
    amount * 0.0825  // What is 0.0825?
}

fn retry_request() {
    for _ in 0..3 {  // Why 3?
        // ...
    }
}

// ✅ GOOD: Named constants
const TAX_RATE: f64 = 0.0825;
const MAX_RETRIES: usize = 3;

fn calculate_tax(amount: f64) -> f64 {
    amount * TAX_RATE
}

fn retry_request() {
    for _ in 0..MAX_RETRIES {
        // ...
    }
}
```

**Rationale**: Named constants document intent and make changes easier (change in one place vs. find-and-replace).

---

## Summary Table

| ID | Anti-Pattern | Key Issue |
|----|--------------|-----------|
| AP-01 | Clone for borrow checker | Performance, desync |
| AP-02 | `deny(warnings)` | Fragile builds |
| AP-03 | Deref polymorphism | Misuse of trait |
| AP-04 | Unnecessary `impl Trait` | Hides useful types |
| AP-05 | Stringly typed | No type safety |
| AP-06 | `unwrap()` in libraries | Forces panics |
| AP-07 | `collect()` without type | Inference failure |
| AP-08 | `&String`/`&Vec<T>` params | Overly restrictive |
| AP-09 | `Drop` for non-resources | Surprising behavior |
| AP-10 | Boolean parameters | Unclear call sites |
| AP-11 | `match` over combinators | Verbose code |
| AP-12 | `Box<dyn Error>` not Send | Thread-unsafe |
| AP-13 | Ignoring `#[must_use]` | Lost errors |
| AP-14 | Allocations in hot loops | Performance |
| AP-15 | `pub` fields with invariants | Broken encapsulation |
| AP-16 | Over-generic functions | Complexity, compile time |
| AP-17 | Nested `Result<Option<...>>` | Unreadable types |
| AP-18 | Sync I/O in async | Blocks runtime |
| AP-19 | `Rc<RefCell>` overkill | Unnecessary overhead |
| AP-20 | Magic numbers | Poor documentation |

---

*See also: [01-core-idioms.md](01-core-idioms.md) for what TO do.*
