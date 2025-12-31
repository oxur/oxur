# Anti-Patterns in Rust

> Critical patterns to avoid when writing Rust code

**Purpose**: This document catalogs common mistakes that AI models frequently generate. Understanding these anti-patterns is essential for producing correct, idiomatic Rust code.


## AP-01: `#![deny(warnings)]` in Library Code

**Strength**: AVOID

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

---

## AP-02: `&String` and `&Vec<T>` in Function Parameters

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

**Clippy**: clippy::ptr_arg

---

## AP-03: `Box<dyn Error>` Without `Send + Sync`

**Strength**: AVOID

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

---

## AP-04: `collect()` Without Type Annotation

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

---

## AP-05: `match` on `Option`/`Result` Instead of Combinators

**Strength**: CONSIDER

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

---

## AP-06: `pub` Fields When Invariants Exist

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

---

## AP-07: `Rc<RefCell<T>>` When `Cell<T>` Suffices

**Strength**: CONSIDER

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

**Rationale**: - `Cell<T>` (for `Copy` types): No runtime borrow checking, just copies values
- `RefCell<T>`: Runtime borrow checking, can panic
- `Rc<T>`: Reference counting overhead

Use the simplest type that works.

---

---

## AP-08: `to_string()` in Hot Loops

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

---

## AP-09: `unwrap()` in Library Code

**Strength**: AVOID

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

**Clippy**: clippy::unwrap_used

---

## AP-10: Boolean Parameters

**Strength**: AVOID

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

---

## AP-11: Builder Pattern Without Consuming Self

**Strength**: CONSIDER

**Summary**: Builder methods should consume `self` to prevent reuse bugs.

```rust
// Bad - allows invalid state
struct RequestBuilder {
    url: Option<String>,
    method: Option<String>,
}

impl RequestBuilder {
    fn url(&mut self, url: String) -> &mut Self {
        self.url = Some(url);
        self
    }
    
    fn method(&mut self, method: String) -> &mut Self {
        self.method = Some(method);
        self
    }
    
    fn build(&self) -> Request {
        Request {
            url: self.url.clone().unwrap(),
            method: self.method.clone().unwrap(),
        }
    }
}

// Good - consuming builder prevents reuse
struct RequestBuilder {
    url: Option<String>,
    method: Option<String>,
}

impl RequestBuilder {
    fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }
    
    fn method(mut self, method: String) -> Self {
        self.method = Some(method);
        self
    }
    
    fn build(self) -> Result<Request, BuilderError> {
        Ok(Request {
            url: self.url.ok_or(BuilderError::MissingUrl)?,
            method: self.method.ok_or(BuilderError::MissingMethod)?,
        })
    }
}

// Usage - can't accidentally reuse
let request = RequestBuilder::new()
    .url("https://api.example.com".to_string())
    .method("GET".to_string())
    .build()?;
```

**Rationale**: Consuming builders prevent accidental reuse and partial state bugs.

---

## AP-12: Clone to Satisfy the Borrow Checker

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

**See also**: `mem::take`, `mem::replace` for zero-cost alternatives in enums

---

## AP-13: Collecting Iterator Just to Iterate Again

**Strength**: AVOID

**Summary**: Unnecessary intermediate collections waste memory and time.

```rust
// Bad - collect then iterate
fn sum_evens(numbers: &[i32]) -> i32 {
    let evens: Vec<i32> = numbers.iter()
        .filter(|&n| n % 2 == 0)
        .copied()
        .collect();
    evens.iter().sum()
}

// Good - iterate directly
fn sum_evens(numbers: &[i32]) -> i32 {
    numbers.iter()
        .filter(|&n| n % 2 == 0)
        .sum()
}

// Bad - collect for length
fn count_evens(numbers: &[i32]) -> usize {
    numbers.iter()
        .filter(|&n| n % 2 == 0)
        .collect::<Vec<_>>()
        .len()
}

// Good - use count()
fn count_evens(numbers: &[i32]) -> usize {
    numbers.iter()
        .filter(|&n| n % 2 == 0)
        .count()
}
```

**Rationale**: Iterators are lazy and efficient. Collecting prematurely allocates unnecessarily.

---

## AP-14: Critical Reminders for AI

**Strength**: SHOULD

**Summary**: 

---

## AP-15: Deref Polymorphism (Fake Inheritance)

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

---

## AP-16: Dereferencing Raw Pointers Without Safety Comments

**Strength**: MUST

**Summary**: Every unsafe block should have a comment explaining why it's safe.

```rust
// Bad - no safety explanation
unsafe {
    *ptr = 42;
}

// Good - documented safety invariants
// SAFETY: ptr is guaranteed to be valid and properly aligned because:
// 1. It was obtained from Box::into_raw()
// 2. No other references exist
// 3. The pointee has not been dropped
unsafe {
    *ptr = 42;
}
```

**Rationale**: Unsafe code requires careful reasoning. Document invariants for reviewers and future maintainers.

---

## AP-17: Don't Allocate in Loops

**Strength**: AVOID

**Summary**: 

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

---

## AP-18: Don't Clone When You Can Borrow

**Strength**: AVOID

**Summary**: 

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

---

## AP-19: Don't Expose ErrorKind Enums

**Strength**: AVOID

**Summary**: 

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

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

## AP-20: Don't Expose Smart Pointers in APIs

**Strength**: AVOID

**Summary**: 

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

**See also**: M-AVOID-WRAPPERS

---

## AP-21: Don't Fight the Borrow Checker with unsafe

**Strength**: AVOID

**Summary**: 

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

---

## AP-22: Don't Glob Re-export

**Strength**: AVOID

**Summary**: 

```rust
// WRONG - hard to review, may export unintended items
pub use internal::*;

// CORRECT - explicit exports
pub use internal::{Foo, Bar, Baz};

// Exception: platform-specific HAL modules
#[cfg(target_os = "windows")]
pub use windows_impl::*;  // OK for platform abstraction
```

**See also**: M-NO-GLOB-REEXPORTS

---

## AP-23: Don't Nest Generics Deeply

**Strength**: AVOID

**Summary**: 

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

**See also**: M-SIMPLE-ABSTRACTIONS

---

## AP-24: Don't Prematurely Optimize

**Strength**: AVOID

**Summary**: 

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

**See also**: M-HOTPATH

---

## AP-25: Don't Return Error When You Should Panic

**Strength**: AVOID

**Summary**: 

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

**See also**: M-PANIC-ON-BUG

---

## AP-26: Don't Use Associated Functions for Everything

**Strength**: AVOID

**Summary**: 

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

**See also**: M-REGULAR-FN

---

## AP-27: Don't Use Builder for Simple Types

**Strength**: AVOID

**Summary**: 

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

**See also**: M-INIT-BUILDER

---

## AP-28: Don't Use Panic for Error Handling

**Strength**: AVOID

**Summary**: 

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

**See also**: M-PANIC-IS-STOP

---

## AP-29: Don't Use Rc in Async Code

**Strength**: AVOID

**Summary**: 

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

**See also**: M-TYPES-SEND

---

## AP-30: Don't Use String for Everything

**Strength**: AVOID

**Summary**: 

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

**See also**: M-STRONG-TYPES, primitive obsession

---

## AP-31: Don't Use unsafe Without Clear Safety Comments

**Strength**: AVOID

**Summary**: 

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

**See also**: M-UNSAFE

---

## AP-32: Don't Write Unsound Code

**Strength**: AVOID

**Summary**: 

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

**See also**: M-UNSOUND

---

## AP-33: Fighting the Borrow Checker with Clones

**Strength**: AVOID

**Summary**: Adding clones to satisfy the borrow checker usually indicates a design problem.

```rust
// Bad - cloning to avoid borrow checker
struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    fn get_or_compute(&mut self, key: &str) -> String {
        if let Some(value) = self.data.get(key) {
            value.clone() // Clone just to satisfy borrow checker
        } else {
            let computed = expensive_computation(key);
            self.data.insert(key.to_string(), computed.clone());
            computed
        }
    }
}

// Good - restructure to avoid clone
impl Cache {
    fn get_or_compute(&mut self, key: &str) -> &str {
        use std::collections::hash_map::Entry;
        
        match self.data.entry(key.to_string()) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(expensive_computation(key)),
        }
    }
}

fn expensive_computation(key: &str) -> String {
    format!("computed_{}", key)
}
```

**Rationale**: The borrow checker enforces correctness. If you're fighting it with clones, redesign the API.

---

## AP-34: Ignoring `#[must_use]` Warnings

**Strength**: MUST

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

**Clippy**: clippy::let_underscore_must_use

---

## AP-35: Ignoring Errors with `let _ =`

**Strength**: AVOID

**Summary**: Silently discarding Result values masks errors that should be handled.

```rust
// Bad - error is completely ignored
fn save_data(data: &Data) {
    let _ = std::fs::write("data.json", serde_json::to_string(data).unwrap());
}

// Good - error is propagated
fn save_data(data: &Data) -> std::io::Result<()> {
    let json = serde_json::to_string(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write("data.json", json)?;
    Ok(())
}

// Acceptable - explicitly acknowledge we don't care
fn save_data_best_effort(data: &Data) {
    if let Ok(json) = serde_json::to_string(data) {
        let _ = std::fs::write("data.json", json); // Intentionally ignore file errors
    }
}
```

**Rationale**: `Result` types exist because operations can fail. Ignoring them leads to silent bugs.

---

## AP-36: Leaking Resources with `mem::forget`

**Strength**: AVOID

**Summary**: `mem::forget` prevents destructors from running, causing resource leaks.

```rust
// Bad - file handle never closed
let file = File::create("data.txt")?;
std::mem::forget(file); // Leaks file descriptor

// Good - let RAII handle cleanup
{
    let file = File::create("data.txt")?;
    // file is automatically closed when it goes out of scope
}

// When you truly need to prevent drop (rare)
let file = File::create("data.txt")?;
let raw_fd = file.into_raw_fd(); // Takes ownership, prevents drop
// Now we're responsible for closing raw_fd
```

**Rationale**: RAII is Rust's resource management model. `forget` breaks it and should be avoided.

---

## AP-37: Magic Numbers and Strings

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

---

## AP-38: Manual `Drop` for Non-Resource Types

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

---

## AP-39: Match on Boolean

**Strength**: AVOID

**Summary**: Use `if` for boolean conditions, not `match`.

```rust
// Bad
match is_valid {
    true => println!("Valid"),
    false => println!("Invalid"),
}

// Good
if is_valid {
    println!("Valid");
} else {
    println!("Invalid");
}
```

**Rationale**: `if/else` is clearer for binary conditions. Use `match` for multi-variant enums.

---

## AP-40: Needless `collect()` Before `join()`

**Strength**: AVOID

**Summary**: Many iterators can be joined directly without collecting first.

```rust
// Bad
fn format_numbers(nums: &[i32]) -> String {
    nums.iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

// Good - Itertools provides join on iterators
use itertools::Itertools;
fn format_numbers(nums: &[i32]) -> String {
    nums.iter()
        .map(|n| n.to_string())
        .join(", ")
}

// Good - standard library alternative
fn format_numbers(nums: &[i32]) -> String {
    nums.iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ") // This case actually needs collect
}
```

**Rationale**: Itertools extends iterator functionality without intermediate collections.

---

## AP-41: Needless Boolean Comparisons

**Strength**: AVOID

**Summary**: Comparing booleans to `true` or `false` is redundant.

```rust
// Bad
if is_valid == true {
    do_something();
}

if is_valid == false {
    do_something_else();
}

// Good
if is_valid {
    do_something();
}

if !is_valid {
    do_something_else();
}
```

**Rationale**: Boolean values are already true or false; comparing them is redundant.

---

## AP-42: Nested `Result<Option<Result<...>>>`

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

---

## AP-43: Nested Result/Option Unwrapping

**Strength**: AVOID

**Summary**: Multiple unwraps create fragile code that's hard to debug when it fails.

```rust
// Bad - which unwrap failed?
fn get_user_age(users: &HashMap<String, User>) -> u32 {
    users.get("alice").unwrap().age.unwrap()
}

// Good - clear error handling with context
fn get_user_age(users: &HashMap<String, User>) -> Result<u32, String> {
    let user = users.get("alice")
        .ok_or("User 'alice' not found")?;
    user.age.ok_or("User has no age set")
}

// Good - using and_then for cleaner chaining
fn get_user_age(users: &HashMap<String, User>) -> Option<u32> {
    users.get("alice").and_then(|user| user.age)
}
```

**Rationale**: Multiple unwraps make debugging difficult and provide poor error messages.

---

## AP-44: Option\<Option\<T\>\>

**Strength**: AVOID

**Summary**: Nested Options usually indicate a design problem.

```rust
// Bad - confusing API
struct User {
    name: String,
    email: Option<Option<String>>, // Some(Some("a@b.com")), Some(None), None
}

// What's the difference between Some(None) and None?

// Good - use descriptive types
struct User {
    name: String,
    email: Email,
}

enum Email {
    Verified(String),
    Pending,
    None,
}

// Or simply
struct User {
    name: String,
    email: Option<String>,  // None means no email
    email_verified: bool,
}
```

**Rationale**: `Option<Option<T>>` has three states when two should suffice. Use enums for explicit states.

---

## AP-45: Overly Generic Functions

**Strength**: CONSIDER

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

---

## AP-46: Overusing `String` Instead of `&str`

**Strength**: AVOID

**Summary**: Taking owned `String` when `&str` suffices forces unnecessary allocations.

```rust
// Bad - forces caller to own String
fn greet(name: String) {
    println!("Hello, {}!", name);
}

// Usage requires .to_string() or .to_owned()
greet(user.name.to_string());

// Good - accepts &str (which String can coerce to)  
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

// Usage works with &str or &String
greet(&user.name);
greet("Alice");

// Good - take ownership only when needed
fn store_name(name: String) -> User {
    User { name } // Moves name into User
}
```

**Rationale**: `&str` is more flexible—it accepts string literals, `&String`, and `&str`. Only take ownership when you need to store or modify the string.

---

## AP-47: Public API with Implementation Details

**Strength**: AVOID

**Summary**: Exposing internal types in public APIs creates tight coupling.

```rust
// Bad - implementation detail in public API
pub struct Database {
    pub connection: rusqlite::Connection, // Public field
}

impl Database {
    pub fn execute_raw(&self, sql: &str) -> rusqlite::Result<()> {
        self.connection.execute(sql, [])?;
        Ok(())
    }
}

// Good - hide implementation
pub struct Database {
    connection: rusqlite::Connection, // Private
}

impl Database {
    pub fn new(path: &str) -> Result<Self, DatabaseError> {
        Ok(Database {
            connection: rusqlite::Connection::open(path)?,
        })
    }
    
    pub fn execute(&self, query: &Query) -> Result<(), DatabaseError> {
        // Abstract away rusqlite details
        todo!()
    }
}
```

**Rationale**: Public implementation details prevent changing internals without breaking changes.

---

## AP-48: Redundant Field Names in Struct Literals

**Strength**: AVOID

**Summary**: Use field init shorthand when variable name matches field name.

```rust
// Bad
let user = User {
    name: name,
    age: age,
    email: email,
};

// Good
let user = User {
    name,
    age,
    email,
};
```

**Rationale**: Shorthand reduces noise and is idiomatic Rust.

---

## AP-49: Returning `impl Trait` When Concrete Type Would Work

**Strength**: CONSIDER

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

---

## AP-50: Returning References to Local Variables

**Strength**: MUST

**Summary**: Cannot return references to stack-allocated data.

```rust
// Bad - won't compile
fn create_string() -> &str {
    let s = String::from("hello");
    &s // ERROR: returns reference to local variable
}

// Good - return owned String
fn create_string() -> String {
    String::from("hello")
}

// Good - return static string literal
fn create_string() -> &'static str {
    "hello"
}

// Good - use lifetimes when returning references from input
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```

**Rationale**: Stack values are dropped at function end. Returning references to them would create dangling pointers.

---

## AP-51: Single Match to If Let

**Strength**: CONSIDER

**Summary**: Single-arm matches are better expressed as `if let`.

```rust
// Verbose
match some_option {
    Some(val) => println!("{}", val),
    None => {}
}

// Clear
if let Some(val) = some_option {
    println!("{}", val);
}

// Even better when there's an else
match some_option {
    Some(val) => process(val),
    None => default_action(),
}

// Clearer intent
if let Some(val) = some_option {
    process(val);
} else {
    default_action();
}
```

**Rationale**: `if let` expresses "do something if pattern matches" more clearly than single-arm `match`.

---

## AP-52: String Concatenation in Loops

**Strength**: AVOID

**Summary**: Using `+` to concatenate strings in loops performs O(n²) allocations.

```rust
// Bad - creates new String each iteration
fn join_words(words: &[&str]) -> String {
    let mut result = String::new();
    for word in words {
        result = result + word + " "; // New allocation each time
    }
    result
}

// Good - push_str reuses allocation
fn join_words(words: &[&str]) -> String {
    let mut result = String::new();
    for word in words {
        result.push_str(word);
        result.push(' ');
    }
    result
}

// Better - use join
fn join_words(words: &[&str]) -> String {
    words.join(" ")
}

// Good - pre-allocate capacity
fn join_words(words: &[&str]) -> String {
    let mut result = String::with_capacity(words.iter().map(|w| w.len() + 1).sum());
    for word in words {
        result.push_str(word);
        result.push(' ');
    }
    result
}
```

**Rationale**: String concatenation with `+` creates new allocations. Use `push_str` or `format!` instead.

---

## AP-53: Stringly Typed APIs

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

---

## AP-54: Synchronous I/O in Async Functions

**Strength**: MUST

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

---

## AP-55: Taking Concrete Types Instead of Traits

**Strength**: CONSIDER

**Summary**: Generic trait bounds make APIs more flexible.

```rust
// Less flexible - only works with Vec
fn process_items(items: &Vec<String>) {
    for item in items {
        println!("{}", item);
    }
}

// Better - works with Vec, arrays, slices, etc.
fn process_items(items: &[String]) {
    for item in items {
        println!("{}", item);
    }
}

// Best - works with any iterator of &str-like items
fn process_items<I, S>(items: I) 
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    for item in items {
        println!("{}", item.as_ref());
    }
}
```

**Rationale**: Trait bounds increase reusability without sacrificing performance (monomorphization).

---

## AP-56: Transmuting Between Incompatible Types

**Strength**: MUST

**Summary**: `transmute` bypasses type safety and is almost always wrong.

```rust
// Bad - undefined behavior
unsafe {
    let x: i32 = 42;
    let y: f32 = std::mem::transmute(x); // UB! Sizes happen to match but semantics don't
}

// Good - use explicit conversion
let x: i32 = 42;
let y: f32 = x as f32;

// Bad - transmuting references
unsafe {
    let s = "hello";
    let bytes: &[u8] = std::mem::transmute(s); // Use s.as_bytes() instead!
}

// Good
let s = "hello";
let bytes: &[u8] = s.as_bytes();
```

**Rationale**: Transmute is UB unless you know exactly what you're doing. Use safe alternatives.

---

## AP-57: Unnecessary Cloning

**Strength**: AVOID

**Summary**: Cloning data when references would suffice wastes memory and CPU.

```rust
// Bad - unnecessary clone
fn print_user(user: &User) {
    let user_copy = user.clone();
    println!("{}", user_copy.name);
}

// Good - just use the reference
fn print_user(user: &User) {
    println!("{}", user.name);
}

// Bad - cloning in loop
fn find_user(users: &[User], name: &str) -> Option<User> {
    for user in users {
        if user.name == name {
            return Some(user.clone()); // Unnecessary
        }
    }
    None
}

// Good - return reference
fn find_user(users: &[User], name: &str) -> Option<&User> {
    users.iter().find(|u| u.name == name)
}

// Good - clone only when ownership is truly needed
fn take_user(users: &[User], index: usize) -> User {
    users[index].clone() // Clone IS necessary here
}
```

**Rationale**: Cloning is expensive. Rust's borrowing system lets you avoid most clones through references.

---

## AP-58: Unwrapping in Production Code

**Strength**: AVOID

**Summary**: Using `.unwrap()` or `.expect()` in production code causes panics instead of graceful error handling.

```rust
// Bad - will panic if file doesn't exist
fn read_config() -> Config {
    let contents = std::fs::read_to_string("config.toml").unwrap();
    toml::from_str(&contents).unwrap()
}

// Good - propagates errors for caller to handle
fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string("config.toml")?;
    let config = toml::from_str(&contents)?;
    Ok(config)
}

// Good - provides context with expect in appropriate cases
fn read_config() -> Config {
    let contents = std::fs::read_to_string("config.toml")
        .expect("config.toml must exist in current directory");
    toml::from_str(&contents)
        .expect("config.toml must be valid TOML")
}
```

**Rationale**: Unwrapping causes panics which crash the program. In production, errors should be handled gracefully or at least provide meaningful context.

---

## AP-59: Using `Box` Without Needing Indirection

**Strength**: AVOID

**Summary**: Boxing adds heap allocation overhead without benefit unless you need indirection or trait objects.

```rust
// Bad - unnecessary Box
fn create_user(name: String, age: u32) -> Box<User> {
    Box::new(User { name, age })
}

// Good - return by value
fn create_user(name: String, age: u32) -> User {
    User { name, age }
}

// Good - Box needed for recursive type
enum List {
    Cons(i32, Box<List>),
    Nil,
}

// Good - Box needed for trait object
fn create_logger() -> Box<dyn Logger> {
    Box::new(FileLogger::new("app.log"))
}
```

**Rationale**: Boxing without reason adds overhead. Use it only for recursive types, large types, or trait objects.

---

## AP-60: Using `panic!()` for Error Conditions

**Strength**: AVOID

**Summary**: Use `Result` for expected error conditions; reserve `panic!()` for programmer errors and invariant violations.

```rust
// Bad - user input shouldn't cause panics
fn parse_age(input: &str) -> u8 {
    input.parse().unwrap_or_else(|_| {
        panic!("Invalid age: {}", input)
    })
}

// Good - invalid input is an expected error
fn parse_age(input: &str) -> Result<u8, String> {
    input.parse()
        .map_err(|_| format!("Invalid age: {}", input))
}

// Good - panic for programmer error
fn get_element(vec: &Vec<i32>, index: usize) -> i32 {
    assert!(index < vec.len(), "Index out of bounds: programmer error");
    vec[index]
}
```

**Rationale**: Panics are for unrecoverable programmer errors. User input, I/O failures, and network issues should return `Result`.

---

## AP-61: Using `String` as an Error Type

**Strength**: AVOID

**Summary**: `String` errors lack structure and type safety.

```rust
// Bad - stringly-typed errors
fn parse_config(s: &str) -> Result<Config, String> {
    if s.is_empty() {
        return Err("Config is empty".to_string());
    }
    // ...
    Ok(Config::default())
}

// Good - structured error type
#[derive(Debug)]
enum ConfigError {
    Empty,
    InvalidFormat { line: usize },
    MissingField(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Empty => write!(f, "Config is empty"),
            ConfigError::InvalidFormat { line } => write!(f, "Invalid format at line {}", line),
            ConfigError::MissingField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

impl std::error::Error for ConfigError {}

fn parse_config(s: &str) -> Result<Config, ConfigError> {
    if s.is_empty() {
        return Err(ConfigError::Empty);
    }
    // ...
    Ok(Config::default())
}
```

**Rationale**: Structured error types enable better error handling, pattern matching, and debugging.

---

## AP-62: Using `Vec` When Array Would Suffice

**Strength**: CONSIDER

**Summary**: Fixed-size collections can use stack-allocated arrays instead of heap-allocated vectors.

```rust
// Bad - heap allocation for known size
fn get_rgb() -> Vec<u8> {
    vec![255, 128, 0]
}

// Good - stack allocation
fn get_rgb() -> [u8; 3] {
    [255, 128, 0]
}

// Bad - Vec of fixed size
fn create_matrix() -> Vec<Vec<i32>> {
    vec![
        vec![1, 0, 0],
        vec![0, 1, 0],
        vec![0, 0, 1],
    ]
}

// Good - fixed-size array
fn create_matrix() -> [[i32; 3]; 3] {
    [
        [1, 0, 0],
        [0, 1, 0],
        [0, 0, 1],
    ]
}
```

**Rationale**: Arrays are stack-allocated and have zero overhead. Use Vec only when size is dynamic.

---

## AP-63: ❌ Adding Unnecessary Trait Bounds to Structs

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - redundant bounds
#[derive(Clone, Debug, PartialEq)]
pub struct Container<T: Clone + Debug + PartialEq> {
    value: T,
}

// Problems:
// 1. Can't create Container<Rc<String>> even when you don't need clone
// 2. Adding more derives is a breaking change
// 3. Bounds are redundant (derive adds them to impls automatically)

// GOOD - no bounds on struct
#[derive(Clone, Debug, PartialEq)]
pub struct Container<T> {
    value: T,
}

// Bounds go on impl blocks where needed
impl<T: Clone> Container<T> {
    pub fn duplicate(&self) -> Container<T> {
        Container {
            value: self.value.clone(),
        }
    }
}

impl<T> Container<T> {
    pub fn get(&self) -> &T {
        &self.value
    }
    // No Clone bound needed here
}
```

---

## AP-64: ❌ Boxing Unnecessarily

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - unnecessary boxing
fn create_point() -> Box<Point> {
    Box::new(Point { x: 0, y: 0 })
}

// GOOD - return value directly
fn create_point() -> Point {
    Point { x: 0, y: 0 }
}

// OK - boxing for trait objects
fn create_drawable() -> Box<dyn Drawable> {
    Box::new(Circle { radius: 10.0 })
}

// OK - boxing for large types
pub struct HugeStruct {
    data: [u8; 1_000_000],
}

fn create_huge() -> Box<HugeStruct> {
    // Boxing large types to avoid stack overflow
    Box::new(HugeStruct { data: [0; 1_000_000] })
}
```

---

## AP-65: ❌ Cloning in Loops

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - cloning in loop
fn process_all(template: String, items: &[Item]) {
    for item in items {
        let t = template.clone();  // Cloning every iteration!
        item.process(&t);
    }
}

// GOOD - use reference
fn process_all(template: &str, items: &[Item]) {
    for item in items {
        item.process(template);  // No clone needed
    }
}

// BAD - cloning Arc unnecessarily
fn spawn_workers(data: Arc<Data>) {
    for _ in 0..10 {
        let data_clone = data.clone();  // OK for Arc, but...
        thread::spawn(move || {
            process(&data_clone);
        });
    }
}

// GOOD - clone inside spawn
fn spawn_workers(data: Arc<Data>) {
    for _ in 0..10 {
        let data = Arc::clone(&data);  // More explicit that we're cloning Arc, not Data
        thread::spawn(move || {
            process(&data);
        });
    }
}
```

---

## AP-66: ❌ Cloning to Satisfy Borrow Checker

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - unnecessary clone
fn process_user(users: &mut Vec<User>, id: UserId) {
    let user = users.iter()
        .find(|u| u.id == id)
        .unwrap()
        .clone();  // Cloning to avoid borrow checker!
    
    user.process();
    users.push(user);  // Now users isn't borrowed
}

// GOOD - split borrow
fn process_user(users: &mut Vec<User>, id: UserId) {
    let user_idx = users.iter()
        .position(|u| u.id == id)
        .unwrap();
    
    users[user_idx].process();
    // No clone needed
}

// BAD - cloning string unnecessarily
fn format_message(name: String) -> String {
    let name_copy = name.clone();  // Unnecessary!
    format!("Hello, {}!", name_copy)
}

// GOOD - use reference
fn format_message(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Or consume the string
fn format_message(name: String) -> String {
    format!("Hello, {}!", name)
}
```

---

## AP-67: ❌ Collecting to Vec Unnecessarily

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - unnecessary collection
fn sum_evens(numbers: &[i32]) -> i32 {
    let evens: Vec<i32> = numbers
        .iter()
        .filter(|n| *n % 2 == 0)
        .copied()
        .collect();  // Unnecessary allocation!
    
    evens.iter().sum()
}

// GOOD - use iterator directly
fn sum_evens(numbers: &[i32]) -> i32 {
    numbers
        .iter()
        .filter(|n| *n % 2 == 0)
        .sum()
}

// BAD - collecting then iterating
fn process_items(items: &[Item]) {
    let filtered: Vec<_> = items
        .iter()
        .filter(|item| item.is_active())
        .collect();  // Unnecessary!
    
    for item in filtered {
        process(item);
    }
}

// GOOD - iterate directly
fn process_items(items: &[Item]) {
    for item in items.iter().filter(|item| item.is_active()) {
        process(item);
    }
}
```

---

## AP-68: ❌ Inconsistent Word Order

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - inconsistent ordering
pub struct ParseIntError;
pub struct FloatParseError;     // Should be ParseFloatError
pub struct ErrorParseBool;      // Should be ParseBoolError

// BAD - mixed conventions
pub fn read_to_string() -> String;
pub fn string_from_file() -> String;  // Should be from_file_string or read_file_to_string

// GOOD - consistent word order
pub struct ParseIntError;
pub struct ParseFloatError;
pub struct ParseBoolError;

// GOOD - consistent patterns
pub fn read_to_string() -> String;
pub fn read_to_end() -> Vec<u8>;
```

---

## AP-69: ❌ Inherent Methods on Smart Pointers

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - method on Box
impl<T> Box<T> {
    pub fn process(&self) -> Result<(), Error> {
        // Is this processing the Box or T?
        // Confusing!
    }
}

let boxed_value: Box<MyType> = Box::new(value);
boxed_value.process();  // Which process? Box's or MyType's?

// GOOD - associated function, not method
impl<T> Box<T> {
    pub fn into_raw(b: Box<T>) -> *mut T {
        // Takes Box<T>, not &self
        // No confusion with methods on T
    }
}

let boxed_value = Box::new(value);
Box::into_raw(boxed_value);  // Clearly operates on Box
```

---

## AP-70: ❌ Manual Index Loops Instead of Iterators

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - manual indexing
fn process_all(items: &[Item]) {
    for i in 0..items.len() {
        process(&items[i]);
    }
}

// BAD - counting manually
fn count_active(items: &[Item]) -> usize {
    let mut count = 0;
    for i in 0..items.len() {
        if items[i].is_active() {
            count += 1;
        }
    }
    count
}

// GOOD - use iterator
fn process_all(items: &[Item]) {
    for item in items {
        process(item);
    }
}

// GOOD - use iterator methods
fn count_active(items: &[Item]) -> usize {
    items.iter().filter(|item| item.is_active()).count()
}

// OK - when you actually need the index
fn find_position(items: &[Item], target: &Item) -> Option<usize> {
    items.iter().position(|item| item == target)
}
```

---

## AP-71: ❌ Public Fields Without Invariants

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - public mutable fields
pub struct User {
    pub name: String,
    pub age: u32,
    pub email: String,
}

// Problems:
// 1. Can't validate name (e.g., not empty)
// 2. Can't validate age (e.g., > 0, < 150)
// 3. Can't validate email format
// 4. Can't change internal representation later
// 5. Can't add logging/metrics when fields change

let mut user = User {
    name: String::new(),  // Empty name - should be invalid!
    age: 300,             // Invalid age!
    email: "not-an-email".to_string(),  // Invalid email!
};

// GOOD - private fields with validation
pub struct User {
    name: String,
    age: u32,
    email: String,
}

impl User {
    pub fn new(name: String, age: u32, email: String) -> Result<Self, ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if age == 0 || age > 150 {
            return Err(ValidationError::InvalidAge(age));
        }
        if !email.contains('@') {
            return Err(ValidationError::InvalidEmail);
        }
        
        Ok(User { name, age, email })
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn set_name(&mut self, name: String) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        self.name = name;
        Ok(())
    }
}
```

---

## AP-72: ❌ Silently Ignoring Errors

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - silently ignoring errors
fn save_config(config: &Config) {
    std::fs::write("config.toml", toml::to_string(config).unwrap()).ok();
    // If this fails, program continues silently
}

// BAD - unwrap_or with default
fn load_count() -> u32 {
    std::fs::read_to_string("count.txt")
        .unwrap_or_default()
        .parse()
        .unwrap_or(0)
    // Errors reading or parsing are hidden
}

// BAD - let _ = discards Result
fn process() {
    let _ = some_operation();  // Ignores errors!
}

// GOOD - propagate errors
fn save_config(config: &Config) -> Result<(), Error> {
    let toml_string = toml::to_string(config)?;
    std::fs::write("config.toml", toml_string)?;
    Ok(())
}

// GOOD - handle errors explicitly
fn load_count() -> Result<u32, Error> {
    let contents = std::fs::read_to_string("count.txt")?;
    let count = contents.parse()?;
    Ok(count)
}

// GOOD - if error genuinely doesn't matter, be explicit
fn log_event(msg: &str) {
    // OK to ignore errors in non-critical logging
    let _ = writeln!(std::io::stderr(), "{}", msg);
}
```

---

## AP-73: ❌ Using () as Error Type

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - no error information
fn parse_config(s: &str) -> Result<Config, ()> {
    if s.is_empty() {
        return Err(());  // Why did it fail? Unknown!
    }
    // ...
}

// Can't use with ? operator in functions returning other errors
// Can't display error message
// Can't use with error handling libraries

// GOOD - specific error type
#[derive(Debug)]
pub enum ConfigError {
    Empty,
    InvalidFormat(String),
    MissingField(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Empty => write!(f, "configuration is empty"),
            ConfigError::InvalidFormat(msg) => write!(f, "invalid format: {}", msg),
            ConfigError::MissingField(field) => write!(f, "missing field: {}", field),
        }
    }
}

impl std::error::Error for ConfigError {}

fn parse_config(s: &str) -> Result<Config, ConfigError> {
    if s.is_empty() {
        return Err(ConfigError::Empty);
    }
    // ...
}
```

---

## AP-74: ❌ Using Deref for Type Conversions

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - abusing Deref for conversion
pub struct Celsius(f64);
pub struct Fahrenheit(f64);

impl std::ops::Deref for Celsius {
    type Target = Fahrenheit;
    
    fn deref(&self) -> &Fahrenheit {
        // This is wrong! Can't safely return reference to computed value
        // Would need unsafe or thread-local storage
    }
}

// Problems:
// 1. Deref is for smart pointers, not conversions
// 2. Can cause confusing method resolution
// 3. Violates principle of least surprise

// GOOD - explicit conversion methods
impl Celsius {
    pub fn to_fahrenheit(&self) -> Fahrenheit {
        Fahrenheit(self.0 * 9.0 / 5.0 + 32.0)
    }
}

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Fahrenheit {
        Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
    }
}
```

---

## AP-75: ❌ Using format! for Simple Concatenation

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - format! for simple cases
let full_name = format!("{} {}", first, last);
let message = format!("Error: {}", msg);

// GOOD - direct concatenation
let full_name = first.to_string() + " " + last;
let message = "Error: ".to_string() + msg;

// BETTER - using Vec for multiple concatenations
let mut result = String::new();
result.push_str(prefix);
result.push_str(middle);
result.push_str(suffix);

// OK - format! for actual formatting
let message = format!("User {} has {} points", name, points);
let coord = format!("({:.2}, {:.2})", x, y);
```

---

## AP-76: ❌ Using get_ Prefix Unnecessarily

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - unnecessary get_ prefix
pub struct Config {
    timeout: Duration,
}

impl Config {
    pub fn get_timeout(&self) -> Duration {  // Don't do this
        self.timeout
    }
    
    pub fn get_mut_timeout(&mut self) -> &mut Duration {  // Don't do this
        &mut self.timeout
    }
}

// GOOD - no get_ prefix
impl Config {
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
    
    pub fn timeout_mut(&mut self) -> &mut Duration {
        &mut self.timeout
    }
}

// GOOD - get_ only for special cases
impl<T> [T] {
    pub fn get(&self, index: usize) -> Option<&T> {
        // Special case: runtime validation
    }
    
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        // Pairs with get()
    }
}
```

---

## AP-77: ❌ Using Option When Error Info Needed

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - Option loses error context
fn connect_database(url: &str) -> Option<Connection> {
    // Can fail for many reasons:
    // - Invalid URL
    // - Network error
    // - Authentication failed
    // - Database doesn't exist
    // Which one? Caller doesn't know!
    None
}

// GOOD - Result provides error details
#[derive(Debug)]
pub enum DbError {
    InvalidUrl(String),
    NetworkError(std::io::Error),
    AuthFailed { user: String },
    DatabaseNotFound { name: String },
}

fn connect_database(url: &str) -> Result<Connection, DbError> {
    // Caller can distinguish between error cases
}

// OK - Option when there's only one failure mode
fn first_line(text: &str) -> Option<&str> {
    text.lines().next()
    // Only fails if text is empty - Option is fine
}
```

---

## AP-78: ❌ Using Placeholder Words in Feature Names

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - placeholder words in Cargo.toml
[features]
use-std = []         # Just call it "std"
with-serde = []      # Just call it "serde"
enable-logging = []  # Just call it "logging"

// GOOD - direct names
[features]
std = []
serde = ["dep:serde"]
logging = []

// Usage is cleaner
# In dependent crate
my-crate = { version = "1.0", features = ["std", "serde"] }
```

---

## AP-79: ❌ Using String When &str Works

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - forces allocation
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// Caller must allocate:
greet("Alice".to_string());  // Unnecessary allocation!

// GOOD - accept &str
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Caller can use string literal:
greet("Alice");  // No allocation!
greet(&name);    // Works with String too

// BAD - collecting strings unnecessarily
fn join_names(names: Vec<String>) -> String {
    // Forces caller to allocate all strings
}

// GOOD - accept slices
fn join_names(names: &[&str]) -> String {
    names.join(", ")
}

// Or even better - generic over AsRef<str>
fn join_names<S: AsRef<str>>(names: &[S]) -> String {
    names.iter()
        .map(|s| s.as_ref())
        .collect::<Vec<_>>()
        .join(", ")
}
```

---

## AP-80: ❌ Using unwrap() in Library Code

**Strength**: SHOULD

**Summary**: 

```rust
// BAD - unwrap in library
pub fn parse_json(s: &str) -> Data {
    serde_json::from_str(s).unwrap()  // Panics on invalid JSON!
}

// BAD - expect in library
pub fn read_config() -> Config {
    let contents = std::fs::read_to_string("config.toml")
        .expect("Failed to read config");  // Panics!
    toml::from_str(&contents).expect("Failed to parse config")  // Panics!
}

// GOOD - return Result
pub fn parse_json(s: &str) -> Result<Data, serde_json::Error> {
    serde_json::from_str(s)
}

pub fn read_config() -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string("config.toml")?;
    let config = toml::from_str(&contents)?;
    Ok(config)
}

// OK - unwrap in test code
#[test]
fn test_parse() {
    let data = parse_json(r#"{"name": "test"}"#).unwrap();
    assert_eq!(data.name, "test");
}

// OK - unwrap with documented justification
pub fn process(data: &ValidatedData) -> String {
    // SAFETY: data is validated to be UTF-8 by constructor
    String::from_utf8(data.as_bytes().to_vec()).unwrap()
}
```

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