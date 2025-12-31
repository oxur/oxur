# Ownership, Borrowing, and Lifetimes

> Strategies for working with Rust's ownership system effectively.

---


## OB-01: Avoid Global Statics

**Strength**: MUST

**Summary**: Avoid static and thread-local items when a consistent view is required for correctness.

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

**See also**: M-AVOID-STATICS

---

## OB-02: Clone Strategically

**Strength**: SHOULD

**Summary**: Clone intentionally, not to satisfy the borrow checker.

```rust
// ❌ BAD: Cloning to avoid borrow issues (anti-pattern)
fn process(data: &Data) {
    let cloned = data.clone();  // Why?
    // ... use cloned where data would have worked
}

// ✅ GOOD: Clone when you need independent ownership
fn process(data: &Data) -> ProcessHandle {
    let owned = data.clone();  // Need to store in handle
    ProcessHandle { data: owned }
}

// ✅ GOOD: Clone for thread transfer
fn spawn_processor(data: &Data) {
    let owned = data.clone();  // Threads need 'static
    std::thread::spawn(move || {
        process(owned);
    });
}

// ✅ GOOD: Clone with Rc/Arc is cheap
use std::rc::Rc;

let shared = Rc::new(expensive_data());
let handle1 = Rc::clone(&shared);  // Just increments counter
let handle2 = Rc::clone(&shared);  // Still just increments counter
```

---

## OB-03: Don't Leak External Types

**Strength**: SHOULD

**Summary**: Minimize exposing third-party crate types in public APIs; prefer std types.

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

**See also**: M-DONT-LEAK-TYPES

---

## OB-04: I/O and System Calls Are Mockable

**Strength**: MUST

**Summary**: Any type doing I/O or system calls with side effects must be mockable for testing.

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

**See also**: M-MOCKABLE-SYSCALLS, M-TEST-UTIL

---

## OB-05: Interior Mutability Patterns

**Strength**: CONSIDER

**Summary**: Use `Cell`, `RefCell`, `Mutex`, etc. when shared references need mutation.

```rust
use std::cell::{Cell, RefCell};
use std::sync::{Mutex, RwLock, Arc};

// Cell<T>: For Copy types, no runtime cost
struct Counter {
    value: Cell<i32>,
}

impl Counter {
    fn increment(&self) {  // Note: &self, not &mut self
        self.value.set(self.value.get() + 1);
    }
}

// RefCell<T>: Runtime borrow checking, panics on violation
struct Document {
    content: RefCell<String>,
}

impl Document {
    fn append(&self, text: &str) {
        self.content.borrow_mut().push_str(text);
    }
    
    fn read(&self) -> String {
        self.content.borrow().clone()
    }
}

// Mutex<T>: Thread-safe, blocking
struct SharedState {
    data: Mutex<Vec<i32>>,
}

impl SharedState {
    fn add(&self, value: i32) {
        self.data.lock().unwrap().push(value);
    }
}

// RwLock<T>: Multiple readers OR single writer
struct Cache {
    data: RwLock<HashMap<String, String>>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }
    
    fn set(&self, key: String, value: String) {
        self.data.write().unwrap().insert(key, value);
    }
}
```

---

## OB-06: Lifetime Bounds on Structs

**Strength**: MUST

**Summary**: Structs storing references must declare their lifetime parameters.

```rust
// ✅ CORRECT: Lifetime parameter for borrowed data
struct Parser<'input> {
    input: &'input str,
    position: usize,
}

impl<'input> Parser<'input> {
    fn new(input: &'input str) -> Self {
        Self { input, position: 0 }
    }
    
    fn remaining(&self) -> &'input str {
        &self.input[self.position..]
    }
}

// Usage:
let text = String::from("hello world");
let parser = Parser::new(&text);
// parser cannot outlive text

// ✅ ALTERNATIVE: Store owned data (no lifetime needed)
struct OwnedParser {
    input: String,
    position: usize,
}

// ✅ MULTIPLE LIFETIMES: When needed
struct Processor<'a, 'b> {
    input: &'a str,
    output: &'b mut String,
}

// 'a and 'b can be different, allowing more flexible usage
```

---

## OB-07: Mockable I/O Template

**Strength**: SHOULD

**Summary**: 

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

---

## OB-08: Move Closures

**Strength**: SHOULD

**Summary**: Use `move` to transfer ownership into closures.

```rust
// ❌ WON'T COMPILE: Closure borrows, but outlives the data
fn spawn_printer(message: String) {
    std::thread::spawn(|| {
        println!("{}", message);  // ERROR: message borrowed, not moved
    });
}

// ✅ CORRECT: move captures ownership
fn spawn_printer(message: String) {
    std::thread::spawn(move || {
        println!("{}", message);  // OK: message moved into closure
    });
}

// Selective capture with move:
fn example() {
    let owned = String::from("owned");
    let to_clone = String::from("cloned");
    let to_clone = to_clone.clone();  // Clone before closure
    
    std::thread::spawn(move || {
        // Both owned and to_clone are moved in
        println!("{} {}", owned, to_clone);
    });
}

// Rebinding for partial moves:
fn selective_move() {
    let data = ComplexStruct { a: String::new(), b: 42 };
    let a = data.a;  // Move out just `a`
    
    std::thread::spawn(move || {
        println!("{}", a);  // Only `a` captured
    });
    
    // data.b is still accessible (b is Copy)
    println!("{}", data.b);
}
```

---

## OB-09: Prefer Borrowing Over Ownership in Parameters

**Strength**: SHOULD

**Summary**: Functions should borrow (`&T`) unless they need ownership.

```rust
// ❌ UNNECESSARY OWNERSHIP: Forces caller to give up or clone
fn process(data: Vec<i32>) -> i32 {
    data.iter().sum()
}

// Caller must clone if they need the data later:
let data = vec![1, 2, 3];
let sum = process(data.clone());  // Unnecessary allocation
println!("{:?}", data);

// ✅ BORROWING: Caller retains ownership
fn process(data: &[i32]) -> i32 {
    data.iter().sum()
}

// Caller keeps their data:
let data = vec![1, 2, 3];
let sum = process(&data);
println!("{:?}", data);  // Still valid!

// ✅ OWNERSHIP NEEDED: When you must store or modify
fn consume_into_result(data: Vec<i32>) -> ProcessedData {
    ProcessedData { 
        original: data,  // Need to store it
        // ...
    }
}
```

---

## OB-10: Return Owned Types, Not References

**Strength**: SHOULD

**Summary**: Functions typically return owned values; references need lifetime annotation.

```rust
// ✅ SIMPLE: Return owned type
fn create_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

// ❌ WON'T COMPILE: Can't return reference to local
fn bad_greeting(name: &str) -> &str {
    let greeting = format!("Hello, {name}!");
    &greeting  // ERROR: greeting dropped at end of function
}

// ✅ RETURNING REFERENCE: When returning part of input
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

// ✅ RETURNING REFERENCE: From struct field
impl User {
    fn name(&self) -> &str {
        &self.name
    }
}

// ✅ COW: When sometimes owned, sometimes borrowed
use std::borrow::Cow;

fn maybe_modify(s: &str) -> Cow<'_, str> {
    if s.contains("bad") {
        Cow::Owned(s.replace("bad", "good"))
    } else {
        Cow::Borrowed(s)
    }
}
```

---

## OB-11: Smart Pointer Selection

**Strength**: SHOULD

**Summary**: Choose the right smart pointer for your ownership needs.

```rust
// Box<T>: Single owner, heap allocation
let boxed: Box<[i32; 1000]> = Box::new([0; 1000]);  // Large array on heap

// Rc<T>: Multiple owners, single-threaded
use std::rc::Rc;
let shared = Rc::new(data);
let clone1 = Rc::clone(&shared);
let clone2 = Rc::clone(&shared);
// All three point to same data

// Arc<T>: Multiple owners, thread-safe
use std::sync::Arc;
let shared = Arc::new(data);
let for_thread = Arc::clone(&shared);
std::thread::spawn(move || {
    println!("{:?}", for_thread);
});

// Weak<T>: Non-owning reference (prevents cycles)
use std::rc::Weak;
struct Node {
    parent: Option<Weak<Node>>,  // Doesn't prevent parent from being dropped
    children: Vec<Rc<Node>>,     // Owns children
}
```

---

## OB-12: Struct Decomposition for Independent Borrowing

**Strength**: CONSIDER

**Summary**: Split structs to allow borrowing fields independently.

```rust
// ❌ PROBLEM: Can't borrow two fields mutably through &mut self
struct Game {
    player: Player,
    enemies: Vec<Enemy>,
}

impl Game {
    fn update(&mut self) {
        for enemy in &mut self.enemies {
            // ERROR: Can't borrow player while enemies borrowed
            enemy.attack(&mut self.player);
        }
    }
}

// ✅ SOLUTION A: Pass fields separately
impl Game {
    fn update(&mut self) {
        let (player, enemies) = (&mut self.player, &mut self.enemies);
        for enemy in enemies {
            enemy.attack(player);  // Works!
        }
    }
}

// ✅ SOLUTION B: Decompose into sub-structs
struct GameState {
    player: Player,
}

struct GameEntities {
    enemies: Vec<Enemy>,
}

struct Game {
    state: GameState,
    entities: GameEntities,
}

impl Game {
    fn update(&mut self) {
        for enemy in &mut self.entities.enemies {
            enemy.attack(&mut self.state.player);  // Different sub-structs!
        }
    }
}
```

**Rationale**: The borrow checker sees struct fields independently only within the same function. Decomposition helps when methods need concurrent access to different parts.

---

---

## OB-13: Test Utilities Must Be Feature-Gated

**Strength**: MUST

**Summary**: Mocking functionality and test utilities must be behind a feature flag.

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

**See also**: M-TEST-UTIL

---

## OB-14: The `mem::take` and `mem::replace` Pattern

**Strength**: SHOULD

**Summary**: Move values out of mutable references without unsafe.

```rust
use std::mem;

// ✅ mem::take: Replace with Default, return old value
fn drain_queue<T: Default>(queue: &mut Vec<T>) -> Vec<T> {
    mem::take(queue)  // queue is now empty Vec, returns old contents
}

// ✅ mem::replace: Replace with specific value
fn update_state(state: &mut State) -> State {
    mem::replace(state, State::Updated)  // Returns old state
}

// ✅ Use case: Moving out of enum variants
enum Status {
    Pending(Data),
    Complete,
}

fn complete(status: &mut Status) -> Option<Data> {
    match status {
        Status::Pending(data) => {
            let data = mem::take(data);  // Take the data
            *status = Status::Complete;
            Some(data)
        }
        Status::Complete => None,
    }
}

// ✅ Option::take is a convenience method for this
fn extract_value<T>(opt: &mut Option<T>) -> Option<T> {
    opt.take()  // Same as mem::take(opt)
}
```

---

## OB-15: Types Must Be Send

**Strength**: MUST

**Summary**: All public types should be `Send` for compatibility with async runtimes; futures must always be `Send`.

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

**See also**: M-TYPES-SEND

---

## OB-16: Use `'static` Correctly

**Strength**: MUST

**Summary**: `'static` means "can live forever", not "must live forever".

```rust
// MISCONCEPTION: 'static means it lives for entire program
// TRUTH: 'static means it CAN live for the entire program (no borrowed references)

// ✅ String literals are 'static
let s: &'static str = "hello";

// ✅ Owned types satisfy 'static bounds
fn spawn_thread<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::spawn(f);
}

let owned = String::from("hello");
spawn_thread(move || {
    println!("{}", owned);  // Works: String is 'static (no borrowed refs)
});

// ❌ WRONG: Borrowed references don't satisfy 'static
let local = String::from("hello");
let borrowed: &str = &local;
spawn_thread(move || {
    // println!("{}", borrowed);  // ERROR: borrowed has non-static lifetime
});

// ✅ CORRECT understanding for trait objects:
fn takes_static(x: Box<dyn std::fmt::Debug + 'static>) { }

takes_static(Box::new(String::from("owned")));  // OK: String is 'static
// takes_static(Box::new(&local_string));  // ERROR: reference not 'static
```

---

## OB-17: Use `Cow` for Optional Ownership

**Strength**: CONSIDER

**Summary**: `Cow` (Clone-on-Write) delays allocation until mutation is needed.

```rust
use std::borrow::Cow;

// ✅ Function that might need to modify input
fn normalize_path(path: &str) -> Cow<'_, str> {
    if path.contains("//") {
        // Need to allocate
        Cow::Owned(path.replace("//", "/"))
    } else {
        // No allocation needed
        Cow::Borrowed(path)
    }
}

let path1 = normalize_path("/home/user");     // Borrowed, no alloc
let path2 = normalize_path("/home//user");    // Owned, allocated

// ✅ Struct that might own or borrow
struct Config<'a> {
    name: Cow<'a, str>,
}

impl<'a> Config<'a> {
    fn borrowed(name: &'a str) -> Self {
        Self { name: Cow::Borrowed(name) }
    }
    
    fn owned(name: String) -> Self {
        Self { name: Cow::Owned(name) }
    }
    
    fn name(&self) -> &str {
        &self.name  // Works for both variants
    }
}
```

---

## OB-18: Use Lifetime Elision Where Possible

**Strength**: SHOULD

**Summary**: Let the compiler infer lifetimes; annotate only when required.

```rust
// ❌ VERBOSE: Explicit lifetimes where not needed
fn first<'a>(s: &'a str) -> &'a str {
    &s[..1]
}

// ✅ ELIDED: Compiler infers the lifetime
fn first(s: &str) -> &str {
    &s[..1]
}

// The elision rules:
// 1. Each elided input lifetime becomes a distinct parameter
// 2. If there's exactly one input lifetime, it's assigned to all outputs
// 3. If there's &self or &mut self, its lifetime is assigned to outputs

// ✅ EXPLICIT NEEDED: Multiple input lifetimes, ambiguous output
fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() { s1 } else { s2 }
}

// ✅ EXPLICIT NEEDED: Output lifetime differs from inputs
struct Parser<'input> {
    input: &'input str,
}

impl<'input> Parser<'input> {
    fn new(input: &'input str) -> Self {
        Self { input }
    }
}
```

---

## OB-19: Use the Proper Type Family

**Strength**: MUST

**Summary**: Use the strongest, most appropriate standard library type for your domain.

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

**See also**: M-STRONG-TYPES, C-NEWTYPE

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