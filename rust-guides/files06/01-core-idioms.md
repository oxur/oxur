# Core Rust Idioms

Essential Rust idioms that every Rust programmer should know. These patterns represent fundamental best practices for writing idiomatic Rust code.

---

## Use Borrowed Types for Arguments

**Strength**: MUST

**Summary**: Prefer `&str` over `&String`, `&[T]` over `&Vec<T>`, and `&T` over `&Box<T>` for function parameters.

**Example**:
```rust
// Good - accepts both String and &str
fn print_message(msg: &str) {
    println!("{}", msg);
}

// Usage flexibility
let owned = String::from("hello");
let borrowed = "world";
print_message(&owned);
print_message(borrowed);

// Bad - only accepts &String
fn print_message_bad(msg: &String) {
    println!("{}", msg);
}

// Good - accepts any slice
fn sum(values: &[i32]) -> i32 {
    values.iter().sum()
}

// Bad - only accepts Vec
fn sum_bad(values: &Vec<i32>) -> i32 {
    values.iter().sum()
}
```

**Rationale**: Using borrowed types allows callers to pass owned or borrowed data without additional conversions. This follows the Deref coercion rules and makes APIs more flexible and ergonomic.

**See also**: Deref coercion, API Guidelines on flexibility

---

## String Concatenation with `format!`

**Strength**: SHOULD

**Summary**: Use `format!` macro for string concatenation instead of manual `push_str` calls when readability matters.

**Example**:
```rust
// Good - clear and readable
fn create_greeting(name: &str, age: u32) -> String {
    format!("Hello, {}! You are {} years old.", name, age)
}

// Bad - verbose and error-prone
fn create_greeting_bad(name: &str, age: u32) -> String {
    let mut result = String::from("Hello, ");
    result.push_str(name);
    result.push_str("! You are ");
    result.push_str(&age.to_string());
    result.push_str(" years old.");
    result
}

// Note: For performance-critical code with many concatenations, consider using a String with push_str
```

**Rationale**: The `format!` macro is more readable, less error-prone, and handles allocations efficiently. While `push_str` might be marginally faster in tight loops, `format!` is preferred for clarity.

**See also**: `write!` macro for writing to strings, performance considerations in 08-performance.md

---

## Constructor Conventions

**Strength**: SHOULD

**Summary**: Use `new()` as the canonical constructor name; use `with_capacity()`, `from_*()`, or other descriptive names for alternate constructors.

**Example**:
```rust
// Good - standard constructor pattern
pub struct Connection {
    host: String,
    port: u16,
}

impl Connection {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub fn with_default_port(host: String) -> Self {
        Self { host, port: 8080 }
    }
}

// Good - constructor that can fail
pub struct Config {
    data: String,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(path)?;
        Ok(Self { data })
    }
}

// Good - multiple constructors with clear names
impl Vec<u8> {
    pub fn new() -> Self { /* ... */ }
    pub fn with_capacity(capacity: usize) -> Self { /* ... */ }
}
```

**Rationale**: The `new()` convention is widely understood in the Rust ecosystem. It's not a language feature but a strong community convention that improves code readability.

**See also**: Builder pattern in RustDesignPatterns.pdf, Default trait

---

## The Default Trait

**Strength**: SHOULD

**Summary**: Implement `Default` for types that have a sensible default value; use `#[derive(Default)]` when possible.

**Example**:
```rust
// Good - derive when all fields are Default
#[derive(Default)]
pub struct ConnectionConfig {
    timeout: u64,        // defaults to 0
    retries: u32,        // defaults to 0
    enabled: bool,       // defaults to false
}

// Good - manual implementation for custom defaults
pub struct ServerConfig {
    host: String,
    port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 8080,
        }
    }
}

// Usage
let config = ServerConfig::default();
let another = ServerConfig { port: 9000, ..Default::default() };
```

**Rationale**: `Default` is used throughout the standard library and enables struct update syntax. It's clearer than having a `new()` that takes no arguments.

**See also**: Constructor conventions, Builder pattern

---

## Collections as Smart Pointers

**Strength**: CONSIDER

**Summary**: Understand that `Vec<T>` and `String` are smart pointers that own heap data; use them to avoid explicit lifetime annotations.

**Example**:
```rust
// Good - Vec owns the data, no lifetime needed
pub struct UserDatabase {
    users: Vec<String>,
}

impl UserDatabase {
    pub fn new() -> Self {
        Self { users: Vec::new() }
    }

    pub fn add_user(&mut self, name: String) {
        self.users.push(name);
    }
}

// Alternative with slice requires lifetime
pub struct UserDatabaseRef<'a> {
    users: &'a [String],
}

// Good - use owned collections to simplify APIs
pub struct DataProcessor {
    buffer: Vec<u8>,
}

impl DataProcessor {
    pub fn process(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        // Process buffer...
    }
}
```

**Rationale**: Using owned collections trades a small heap allocation for simpler code without lifetime parameters. This is often the right trade-off for struct fields.

**See also**: Ownership patterns in 04-ownership-borrowing.md

---

## Finalization in Destructors (RAII)

**Strength**: MUST

**Summary**: Use the `Drop` trait to ensure resources are properly cleaned up; rely on RAII for resource management.

**Example**:
```rust
// Good - RAII pattern ensures cleanup
pub struct FileGuard {
    file: std::fs::File,
    path: std::path::PathBuf,
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        println!("Closing file: {:?}", self.path);
        // File is automatically closed when dropped
    }
}

// Good - lock guard pattern
use std::sync::Mutex;

fn update_shared_data(mutex: &Mutex<Vec<i32>>, value: i32) {
    let mut data = mutex.lock().unwrap();
    data.push(value);
    // Lock automatically released when guard is dropped
}

// Bad - manual cleanup is error-prone
pub struct ResourceBad {
    handle: i32,
}

impl ResourceBad {
    pub fn cleanup(&mut self) {
        // Easy to forget to call!
    }
}
```

**Rationale**: RAII ensures resources are cleaned up even in the presence of panics or early returns. This is a fundamental Rust pattern inherited from C++.

**See also**: RAII Guards pattern in RustDesignPatterns.pdf, 07-concurrency-async.md

---

## `mem::take` and `mem::replace`

**Strength**: SHOULD

**Summary**: Use `mem::take` or `mem::replace` to move values out of mutable references, particularly when working with enums.

**Example**:
```rust
use std::mem;

// Good - using mem::take to transform an enum
enum State {
    Idle,
    Processing { data: Vec<u8> },
    Done,
}

impl State {
    fn start_processing(&mut self, new_data: Vec<u8>) {
        // Take the old state, leaving Idle in its place
        let old_state = mem::take(self);

        match old_state {
            State::Idle => {
                *self = State::Processing { data: new_data };
            }
            State::Processing { mut data } => {
                data.extend(new_data);
                *self = State::Processing { data };
            }
            State::Done => {
                *self = State::Idle;
            }
        }
    }
}

// Good - using mem::replace when you need the old value
fn update_config(config: &mut String, new_value: String) -> String {
    mem::replace(config, new_value)
}

// Bad - doesn't compile without mem::take/replace
enum StateBad {
    Idle,
    Processing { data: Vec<u8> },
}

impl StateBad {
    fn bad_update(&mut self) {
        // Error: cannot move out of `*self`
        // match *self {
        //     StateBad::Processing { data } => { /* ... */ }
        //     _ => {}
        // }
    }
}
```

**Rationale**: `mem::take` allows you to move a value out of a mutable reference by replacing it with `Default::default()`. This is essential for transforming enums and avoiding borrow checker issues.

**See also**: `mem::swap`, Option::take, 11-anti-patterns.md (clone to satisfy borrow checker)

---

## On-Stack Dynamic Dispatch

**Strength**: CONSIDER

**Summary**: Use enums instead of trait objects when the set of types is closed and you want stack allocation.

**Example**:
```rust
// Good - stack-allocated enum dispatch
enum Operation {
    Add(i32),
    Multiply(i32),
    Divide(i32),
}

impl Operation {
    fn apply(&self, value: i32) -> i32 {
        match self {
            Operation::Add(x) => value + x,
            Operation::Multiply(x) => value * x,
            Operation::Divide(x) => value / x,
        }
    }
}

fn process(ops: &[Operation], initial: i32) -> i32 {
    ops.iter().fold(initial, |acc, op| op.apply(acc))
}

// Alternative - heap-allocated trait object
trait OperationTrait {
    fn apply(&self, value: i32) -> i32;
}

fn process_dynamic(ops: &[Box<dyn OperationTrait>], initial: i32) -> i32 {
    ops.iter().fold(initial, |acc, op| op.apply(acc))
}

// Usage shows the difference
let stack_ops = vec![
    Operation::Add(5),
    Operation::Multiply(2),
]; // Allocated on stack (in Vec, but items are stack-sized)

// vs
let heap_ops: Vec<Box<dyn OperationTrait>> = vec![
    Box::new(AddOp(5)),
    Box::new(MulOp(2)),
]; // Each item heap-allocated
```

**Rationale**: Enums provide zero-cost abstraction for closed type sets with better cache locality and no heap allocation. Use trait objects when you need an open set of types or plugin architectures.

**See also**: 06-traits.md (trait objects), 08-performance.md

---

## FFI Error Handling

**Strength**: MUST

**Summary**: FFI functions should be `unsafe` and return error codes or use out-parameters; never panic or unwind across FFI boundaries.

**Example**:
```rust
// Good - FFI function with error handling
#[no_mangle]
pub unsafe extern "C" fn process_data(
    data: *const u8,
    len: usize,
    out: *mut u8,
    out_len: *mut usize,
) -> i32 {
    // Validate pointers
    if data.is_null() || out.is_null() || out_len.is_null() {
        return -1; // Error code for null pointer
    }

    // Use catch_unwind to prevent panics from crossing FFI boundary
    let result = std::panic::catch_unwind(|| {
        let input = std::slice::from_raw_parts(data, len);
        // Process data...
        0 // Success
    });

    result.unwrap_or(-2) // Error code for panic
}

// Bad - can panic across FFI boundary
#[no_mangle]
pub extern "C" fn bad_process(data: *const u8, len: usize) {
    // Missing 'unsafe'
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    slice[1000]; // Can panic! Undefined behavior if called from C
}
```

**Rationale**: Panics/unwinding across FFI boundaries is undefined behavior. Always catch panics and convert them to error codes when crossing language boundaries.

**See also**: 09-unsafe-ffi.md, catch_unwind documentation

---

## FFI String Handling

**Strength**: MUST

**Summary**: Use `CStr`/`CString` for C string interop; always validate UTF-8 when converting to Rust strings.

**Example**:
```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Good - accepting C string from external code
#[no_mangle]
pub unsafe extern "C" fn print_message(msg: *const c_char) -> i32 {
    if msg.is_null() {
        return -1;
    }

    let c_str = CStr::from_ptr(msg);

    match c_str.to_str() {
        Ok(rust_str) => {
            println!("{}", rust_str);
            0
        }
        Err(_) => -2, // Invalid UTF-8
    }
}

// Good - passing Rust string to C
fn create_c_string(s: &str) -> Result<CString, std::ffi::NulError> {
    CString::new(s)
}

// Example usage
fn call_c_function(name: &str) {
    let c_name = CString::new(name).expect("CString creation failed");
    unsafe {
        // some_c_function(c_name.as_ptr());
    }
    // c_name is automatically freed when dropped
}

// Bad - assumes valid UTF-8 without checking
#[no_mangle]
pub unsafe extern "C" fn bad_print(msg: *const c_char) {
    let c_str = CStr::from_ptr(msg);
    let rust_str = c_str.to_str().unwrap(); // Can panic!
    println!("{}", rust_str);
}
```

**Rationale**: C strings are null-terminated and may not be valid UTF-8. Always validate and handle errors when converting between C and Rust strings.

**See also**: 09-unsafe-ffi.md, 03-error-handling.md

---

## Iterating Over Option

**Strength**: CONSIDER

**Summary**: Use `Option::iter()` or `Option::iter_mut()` to convert an Option into a 0 or 1 element iterator.

**Example**:
```rust
// Good - chaining Option with iterators
fn process_optional_items(opt: Option<i32>, items: Vec<i32>) -> Vec<i32> {
    opt.iter()
        .chain(items.iter())
        .copied()
        .collect()
}

// Good - flat mapping over Options
fn get_user_emails(users: Vec<Option<User>>) -> Vec<String> {
    users.into_iter()
        .flat_map(|opt| opt.iter())
        .map(|user| user.email.clone())
        .collect()
}

// Alternative using filter_map
fn get_user_emails_alt(users: Vec<Option<User>>) -> Vec<String> {
    users.into_iter()
        .filter_map(|opt| opt)
        .map(|user| user.email)
        .collect()
}

// Good - for loop over Option
fn print_if_some(value: Option<i32>) {
    for v in value.iter() {
        println!("Value: {}", v);
    }
}

struct User {
    email: String,
}
```

**Rationale**: Treating Options as iterators allows seamless integration with iterator chains. This can be more elegant than explicit `match` or `if let` in certain contexts.

**See also**: `Option::into_iter()`, iterator combinators in 08-performance.md

---

## Pass Variables to Closures

**Strength**: SHOULD

**Summary**: Use `move` closures to transfer ownership into the closure; clone before the closure if you need to keep the original.

**Example**:
```rust
use std::thread;

// Good - move ownership into closure
fn spawn_greeting_thread(name: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        println!("Hello, {}!", name);
    })
}

// Good - clone before moving when you need both
fn spawn_and_keep(name: String) -> thread::JoinHandle<()> {
    let name_clone = name.clone();
    let handle = thread::spawn(move || {
        println!("Thread: {}", name_clone);
    });
    println!("Main: {}", name); // Can still use original
    handle
}

// Good - move multiple values
fn spawn_processor(data: Vec<u8>, config: Config) -> thread::JoinHandle<ProcessResult> {
    thread::spawn(move || {
        process(&data, &config)
    })
}

// Bad - borrowing in a thread won't compile
fn bad_spawn(name: String) -> thread::JoinHandle<()> {
    thread::spawn(|| {
        // println!("Hello, {}!", name); // Error: may outlive borrowed value
    })
}

struct Config;
struct ProcessResult;
fn process(_data: &[u8], _config: &Config) -> ProcessResult { ProcessResult }
```

**Rationale**: The `move` keyword transfers ownership to the closure, which is essential for thread safety and avoiding lifetime issues. Clone explicitly when you need both owned and moved values.

**See also**: 07-concurrency-async.md, closure captures

---

## Privacy for Extensibility (`non_exhaustive`)

**Strength**: SHOULD

**Summary**: Use `#[non_exhaustive]` on enums and structs in public APIs to allow adding variants/fields without breaking changes.

**Example**:
```rust
// Good - non_exhaustive enum allows adding variants
#[non_exhaustive]
pub enum Error {
    Io(std::io::Error),
    Parse(String),
    // Can add more variants in the future without breaking users
}

// Users must use a wildcard pattern
fn handle_error(err: Error) {
    match err {
        Error::Io(e) => println!("IO: {}", e),
        Error::Parse(s) => println!("Parse: {}", s),
        _ => println!("Other error"), // Required because of #[non_exhaustive]
    }
}

// Good - non_exhaustive struct allows adding fields
#[non_exhaustive]
pub struct Config {
    pub timeout: u64,
    pub retries: u32,
}

impl Config {
    pub fn new() -> Self {
        Self {
            timeout: 30,
            retries: 3,
        }
    }
}

// Users cannot construct directly (must use constructor)
// let config = Config { timeout: 10, retries: 5 }; // Error: cannot create non-exhaustive struct

// Bad - exhaustive enum breaks when adding variants
pub enum ErrorBad {
    Io(std::io::Error),
    Parse(String),
}

// If we add a variant later, this match breaks:
fn handle_error_bad(err: ErrorBad) {
    match err {
        ErrorBad::Io(e) => println!("IO: {}", e),
        ErrorBad::Parse(s) => println!("Parse: {}", s),
        // Compiles now, but breaks if we add ErrorBad::Network later
    }
}
```

**Rationale**: `#[non_exhaustive]` allows library authors to add enum variants or struct fields in minor version bumps without breaking API compatibility.

**See also**: 02-api-design.md, semver compatibility

---

## Easy Documentation Initialization

**Strength**: SHOULD

**Summary**: Structure code examples in documentation to allow easy copy-paste testing; include all necessary imports.

**Example**:
```rust
/// A connection to a remote server.
///
/// # Examples
///
/// ```
/// use mylib::Connection;
///
/// let conn = Connection::new("localhost", 8080);
/// conn.send("Hello").expect("send failed");
/// ```
pub struct Connection {
    host: String,
    port: u16,
}

impl Connection {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }

    pub fn send(&self, _msg: &str) -> Result<(), std::io::Error> {
        Ok(())
    }
}

// Good - example with full context
/// Process a configuration file.
///
/// # Examples
///
/// ```
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use mylib::process_config;
///
/// let result = process_config("config.toml")?;
/// println!("Processed: {:?}", result);
/// # Ok(())
/// # }
/// ```
pub fn process_config(_path: &str) -> Result<String, std::io::Error> {
    Ok(String::from("processed"))
}

// Bad - incomplete example that won't compile
/// Does something useful.
///
/// # Examples
///
/// ```
/// do_something();  // Where does this come from? Missing use statement!
/// ```
pub fn do_something() {}
```

**Rationale**: Documentation examples are tested by `cargo test`. Making them complete and copy-paste ready improves both documentation quality and test coverage.

**See also**: 13-documentation.md, rustdoc guidelines

---

## Temporary Mutability

**Strength**: CONSIDER

**Summary**: Use block scope to limit the lifetime of mutable bindings when you need temporary mutation.

**Example**:
```rust
// Good - limit scope of mutability
fn create_sorted_unique_list(items: Vec<i32>) -> Vec<i32> {
    let mut items = items;
    items.sort();
    items.dedup();
    items  // Immutable from here, but we can return it
}

// Good - shadowing for temporary mutation
fn process_data(data: Vec<u8>) -> Vec<u8> {
    let mut data = data;
    data.reverse();

    let data = data; // Shadow with immutable binding

    // Use immutable data from here
    transform(data)
}

// Good - inner scope for temporary mutation
fn build_message(mut parts: Vec<String>) -> String {
    {
        let mut parts = parts;  // Temporary mutable scope
        parts.sort();
        parts.dedup();
        // parts is dropped here
    }
    parts.join(", ")
}

// Alternative - use into_iter to consume and rebuild
fn transform_list(items: Vec<i32>) -> Vec<String> {
    items.into_iter()
        .map(|x| x.to_string())
        .collect()
}

fn transform(_data: Vec<u8>) -> Vec<u8> { vec![] }
```

**Rationale**: Limiting the scope of mutability makes code easier to reason about and prevents accidental mutation. Rust's ownership system makes this pattern safe and efficient.

**See also**: Immutability by default, functional patterns in 08-performance.md

---

## Summary

These idioms represent fundamental Rust patterns that should be second nature to Rust programmers:

1. **Use borrowed types** for maximum API flexibility
2. **Use `format!`** for readable string concatenation
3. **Follow `new()` conventions** for constructors
4. **Implement `Default`** when appropriate
5. **Understand collections as smart pointers** to simplify lifetimes
6. **Use RAII** for resource management
7. **Use `mem::take`** to transform owned values
8. **Prefer enums over trait objects** for closed type sets
9. **Handle FFI errors** without panicking
10. **Validate FFI strings** before use
11. **Use `Option::iter()`** for elegant Option handling
12. **Use `move` closures** for ownership transfer
13. **Use `#[non_exhaustive]`** for API stability
14. **Write complete doc examples** for better testing
15. **Limit mutability scope** for safer code

Cross-references:
- 02-api-design.md (public API patterns)
- 03-error-handling.md (Result and Option)
- 04-ownership-borrowing.md (ownership patterns)
- 06-traits.md (trait design)
- 08-performance.md (iterators and zero-cost abstractions)
- 09-unsafe-ffi.md (FFI safety)
- 11-anti-patterns.md (what to avoid)
