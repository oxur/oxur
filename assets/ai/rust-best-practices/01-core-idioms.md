# Core Rust Idioms

Essential Rust idioms that every Rust programmer should know. These patterns represent fundamental best practices for writing idiomatic Rust code.

---


## ID-01: `#[non_exhaustive]` for Public Enums and Structs

**Strength**: SHOULD

**Summary**: Mark public types with `#[non_exhaustive]` to allow future additions.

```rust
// In a library crate:

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    PermissionDenied,
    // Future versions can add variants without breaking changes
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Config {
    pub timeout: Duration,
    pub retries: u32,
    // Future versions can add fields
}

// Users of the library must handle unknown variants:
match error {
    Error::NotFound => { /* ... */ }
    Error::PermissionDenied => { /* ... */ }
    _ => { /* Handle future variants */ }  // Required!
}

// Users cannot construct the struct directly:
let config = Config { timeout, retries };  // ERROR
// Must use constructor:
let config = Config::new(timeout, retries);
```

**Rationale**: Adding enum variants or struct fields is normally a breaking change. `#[non_exhaustive]` allows evolution without major version bumps.

---

---

## ID-02: `mem::take` and `mem::replace`

**Strength**: SHOULD

**Summary**: Use `mem::take` or `mem::replace` to move values out of mutable references, particularly when working with enums.

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

## ID-03: `mem::take` and `mem::replace` for Owned Values in Enums

**Strength**: SHOULD

**Summary**: Use `mem::take` or `mem::replace` to move values out of mutable references.

```rust
use std::mem;

enum State {
    Loading { url: String },
    Ready { data: Vec<u8> },
    Error { message: String },
}

impl State {
    // ❌ BAD: Clone to satisfy borrow checker
    fn transition_to_ready(&mut self, data: Vec<u8>) {
        if let State::Loading { url } = self {
            let url_clone = url.clone();  // Unnecessary clone!
            *self = State::Ready { data };
            log::info!("Loaded from {}", url_clone);
        }
    }

    // ✅ GOOD: mem::take to move out
    fn transition_to_ready(&mut self, data: Vec<u8>) {
        if let State::Loading { url } = self {
            let url = mem::take(url);  // Takes ownership, leaves empty String
            *self = State::Ready { data };
            log::info!("Loaded from {}", url);
        }
    }
}
```

**Rationale**: Avoids cloning when you need to move a value out of a mutable reference. Zero-cost for types where `Default` is cheap (String, Vec, Option).

---

---

## ID-04: Ad-hoc Conversions Follow as_/to_/into_ Conventions

**Strength**: MUST

**Summary**: Use prefix conventions that communicate cost and ownership semantics.

```rust
// as_* - free, borrowed to borrowed
impl str {
    pub fn as_bytes(&self) -> &[u8] {
        // Zero cost - just reinterprets the reference
        unsafe { self.as_bytes() }
    }
}

// to_* - expensive, creates owned data
impl str {
    pub fn to_lowercase(&self) -> String {
        // Allocates new String, iterates through chars
        // Unicode-aware conversion
        self.chars()
            .flat_map(|c| c.to_lowercase())
            .collect()
    }
    
    pub fn to_string(&self) -> String {
        // Allocates and copies
        String::from(self)
    }
}

// into_* - consumes self, transfers ownership
impl String {
    pub fn into_bytes(self) -> Vec<u8> {
        // Takes ownership, no allocation
        // Just transfers the Vec<u8> inside String
        self.into_bytes()
    }
}

impl<T> Vec<T> {
    pub fn into_boxed_slice(self) -> Box<[T]> {
        // Consumes Vec, returns Box
        self.into_boxed_slice()
    }
}

// Bad examples
impl Path {
    // Bad - should be as_os_str (free operation)
    pub fn to_os_str(&self) -> &OsStr { /* ... */ }
    
    // Bad - should be to_path_buf (expensive clone)
    pub fn as_path_buf(&self) -> PathBuf { /* ... */ }
}

// Good - f64 conversions
impl f64 {
    // to_* is correct - input is Copy type being converted
    pub fn to_radians(self) -> f64 {
        self * (PI / 180.0)
    }
    
    pub fn to_degrees(self) -> f64 {
        self * (180.0 / PI)
    }
}
```

**Rationale**: These prefixes provide immediate clarity about the cost and ownership implications of a conversion, helping developers write efficient code without consulting documentation.

**See also**: C-CONV-TRAITS for `From`/`Into`/`AsRef`/`AsMut` trait implementations, C-GETTER for getter naming conventions

---

## ID-05: All Magic Values Must Be Documented

**Strength**: MUST

**Summary**: Hardcoded constants must have comments explaining why the value was chosen, side effects of changing it, and external dependencies.

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

**See also**: M-DOCUMENTED-MAGIC

---

## ID-06: Avoid Weasel Words in Names

**Strength**: MUST

**Summary**: Symbol names should be free of uninformative words like `Service`, `Manager`, `Factory`.

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

## ID-07: Casing Conforms to RFC 430

**Strength**: MUST

**Summary**: Follow Rust's standard casing conventions for all identifiers.

```rust
// Good - proper casing
pub struct HttpResponse {
    status_code: u16,
    body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16) -> Self {
        HttpResponse {
            status_code,
            body: String::new(),
        }
    }
    
    pub fn with_body(status_code: u16, body: String) -> Self {
        HttpResponse { status_code, body }
    }
}

const MAX_RETRIES: u32 = 3;
const DEFAULT_TIMEOUT_MS: u64 = 5000;

// Bad - incorrect casing
pub struct HTTPResponse { // Should be HttpResponse
    StatusCode: u16,      // Should be status_code
    Body: String,         // Should be body
}
```

**Rationale**: Consistent naming makes code instantly recognizable as idiomatic Rust and improves cross-project readability.

---

## ID-08: Collections as Smart Pointers

**Strength**: CONSIDER

**Summary**: Understand that `Vec<T>` and `String` are smart pointers that own heap data; use them to avoid explicit lifetime annotations.

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

## ID-09: Constructor Conventions

**Strength**: SHOULD

**Summary**: Use `new()` as the canonical constructor name; use `with_capacity()`, `from_*()`, or other descriptive names for alternate constructors.

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

## ID-10: Constructors via `new` and `Default`

**Strength**: SHOULD

**Summary**: Use `fn new() -> Self` for primary construction, implement `Default` when zero/empty makes sense.

```rust
pub struct Config {
    timeout_ms: u64,
    retries: u32,
    verbose: bool,
}

impl Config {
    /// Creates a new Config with the given timeout.
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            timeout_ms,
            retries: 3,
            verbose: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            retries: 3,
            verbose: false,
        }
    }
}

// Usage:
let config = Config::new(1000);
let default_config = Config::default();
let partial = Config { verbose: true, ..Default::default() };
```

**Rationale**: - `new` is the conventional constructor name in Rust
- `Default` enables `..Default::default()` syntax and works with generic code
- If `new()` takes no arguments, it should behave identically to `Default::default()`

**See also**: Builder pattern for complex construction

---

## ID-11: Derive Common Traits

**Strength**: SHOULD

**Summary**: Derive `Debug`, `Clone`, `PartialEq`, etc. when semantically appropriate.

```rust
// ❌ INCOMPLETE: Missing useful derives
struct Point {
    x: f64,
    y: f64,
}

// ✅ COMPLETE: Derive what makes sense
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

// For types that can be hashed (require Eq for HashMap keys):
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UserId(u64);

// For serialization (with serde):
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Config {
    name: String,
    value: i32,
}
```

**Rationale**: These traits enable debugging, collections, testing, and serialization. Deriving them costs nothing if unused but enables many use cases.

---

---

## ID-12: Destructors for Finalization (RAII)

**Strength**: SHOULD

**Summary**: Use `Drop` to ensure cleanup happens regardless of exit path.

```rust
struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new() -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::File::create(&path)?;
        Ok(Self { path })
    }
    
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // Cleanup runs even on panic or early return
        let _ = std::fs::remove_file(&self.path);
    }
}

fn process() -> Result<(), Error> {
    let temp = TempFile::new()?;
    write_data(temp.path())?;
    process_data(temp.path())?;
    // TempFile automatically deleted when `temp` goes out of scope
    Ok(())
}
```

---

## ID-13: Detected Programming Bugs are Panics, Not Errors

**Strength**: MUST

**Summary**: When an unrecoverable programming error is detected, panic immediately—don't return a Result.

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

## ID-14: Easy Documentation Initialization

**Strength**: SHOULD

**Summary**: Structure code examples in documentation to allow easy copy-paste testing; include all necessary imports.

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

## ID-15: Feature Names Are Free of Placeholder Words

**Strength**: MUST

**Summary**: Don't use words like `use-` or `with-` in feature names. Name features directly after what they enable.

```rust
// Good - Cargo.toml
[features]
default = ["std"]
std = []
serde = ["dep:serde"]

[dependencies]
serde = { version = "1.0", optional = true }

// Bad - Cargo.toml
[features]
default = ["use-std"]  // Don't add "use-"
use-std = []           // Just call it "std"
with-serde = ["dep:serde"]  // Just call it "serde"
```

```rust
// Good - enabling std library support
// In lib.rs
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub fn read_file(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

// Usage in dependent crate's Cargo.toml
[dependencies]
my-crate = { version = "1.0", features = ["std"] }  // Clean and simple
```

**Rationale**: Cargo automatically creates implicit features for optional dependencies using the dependency name. Explicit features should follow the same convention for consistency.

---

## ID-16: FFI Error Handling

**Strength**: MUST

**Summary**: FFI functions should be `unsafe` and return error codes or use out-parameters; never panic or unwind across FFI boundaries.

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

## ID-17: FFI String Handling

**Strength**: MUST

**Summary**: Use `CStr`/`CString` for C string interop; always validate UTF-8 when converting to Rust strings.

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

## ID-18: Finalization in Destructors (RAII)

**Strength**: MUST

**Summary**: Use the `Drop` trait to ensure resources are properly cleaned up; rely on RAII for resource management.

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

## ID-19: First Sentence is One Line, ~15 Words

**Strength**: MUST

**Summary**: The first sentence of documentation becomes the summary—keep it to one line and approximately 15 words.

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

## ID-20: Getter Names Follow Rust Convention

**Strength**: MUST

**Summary**: Avoid `get_` prefix except for specific cases; getters are just named after the field.

```rust
// Good - idiomatic getters
pub struct Connection {
    timeout: Duration,
    address: SocketAddr,
}

impl Connection {
    // Simple field access - no get_ prefix
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
    
    // Mutable access follows same pattern
    pub fn timeout_mut(&mut self) -> &mut Duration {
        &mut self.timeout
    }
    
    // Returns reference to field
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

// Bad - unnecessary get_ prefix
impl Connection {
    pub fn get_timeout(&self) -> Duration { // Don't do this
        self.timeout
    }
    
    pub fn get_address(&self) -> &SocketAddr { // Don't do this
        &self.address
    }
}

// Good - get_ is appropriate for Cell/RefCell
use std::cell::Cell;

impl Cell<T> {
    pub fn get(&self) -> T where T: Copy {
        // Only one obvious thing to "get" from a Cell
        // get_ prefix makes sense here
    }
}

// Good - unchecked variants
impl<T> [T] {
    pub fn get(&self, index: usize) -> Option<&T> { /* ... */ }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> { /* ... */ }
    
    pub unsafe fn get_unchecked(&self, index: usize) -> &T { /* ... */ }
    
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T { /* ... */ }
}
```

**Rationale**: The `get_` prefix is redundant in most cases. Rust's type system makes getters obvious without the prefix, reducing verbosity.

---

## ID-21: Iterate Over `Option` When Useful

**Strength**: CONSIDER

**Summary**: `Option` implements `IntoIterator`, enabling elegant compositions.

```rust
// Extend a vec with an optional element:
let extra: Option<i32> = Some(42);
let mut numbers = vec![1, 2, 3];
numbers.extend(extra);  // numbers is now [1, 2, 3, 42]

// Chain with other iterators:
let required = vec!["a", "b"];
let optional: Option<&str> = Some("c");
for item in required.iter().chain(optional.iter()) {
    println!("{item}");
}

// Filter map pattern:
let values: Vec<Option<i32>> = vec![Some(1), None, Some(2)];
let sum: i32 = values.into_iter().flatten().sum();  // 3
```

**Rationale**: Treating `Option` as a zero-or-one element iterator enables composition with standard iterator methods.

---

---

## ID-22: Iterating Over Option

**Strength**: CONSIDER

**Summary**: Use `Option::iter()` or `Option::iter_mut()` to convert an Option into a 0 or 1 element iterator.

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

## ID-23: Iterator Methods Follow iter/iter_mut/into_iter Pattern

**Strength**: MUST

**Summary**: For collections containing elements of type `U`, provide three standard iterator methods.

```rust
impl<T> Container<T> {
    // Borrows elements immutably
    fn iter(&self) -> Iter<'_, T> 
    // where Iter: Iterator<Item = &T>
    
    // Borrows elements mutably  
    fn iter_mut(&mut self) -> IterMut<'_, T>
    // where IterMut: Iterator<Item = &mut T>
    
    // Consumes container, transfers ownership
    fn into_iter(self) -> IntoIter<T>
    // where IntoIter: Iterator<Item = T>
}
```

```rust
// Good - standard collection iterator pattern
pub struct MyCollection<T> {
    items: Vec<T>,
}

pub struct Iter<'a, T> {
    inner: std::slice::Iter<'a, T>,
}

pub struct IterMut<'a, T> {
    inner: std::slice::IterMut<'a, T>,
}

pub struct IntoIter<T> {
    inner: std::vec::IntoIter<T>,
}

impl<T> MyCollection<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.items.iter(),
        }
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: self.items.iter_mut(),
        }
    }
    
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner: self.items.into_iter(),
        }
    }
}

// Implement Iterator for each type
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

// Usage examples
let mut collection = MyCollection { items: vec![1, 2, 3] };

// Borrow elements immutably
for item in collection.iter() {
    println!("{}", item);
}

// Borrow elements mutably
for item in collection.iter_mut() {
    *item *= 2;
}

// Consume collection
for item in collection.into_iter() {
    println!("{}", item);
}
// collection is now moved and can't be used
```

```rust
// str is NOT a simple homogeneous collection
impl str {
    // Not iter(), because str has nuanced interpretation
    pub fn bytes(&self) -> Bytes<'_> { /* ... */ }
    pub fn chars(&self) -> Chars<'_> { /* ... */ }
    pub fn lines(&self) -> Lines<'_> { /* ... */ }
}
```

**Rationale**: This pattern is so ubiquitous that developers expect it. It provides consistency across all collections and enables generic code.

**See also**: C-ITER-TY for naming the iterator types themselves

---

## ID-24: Iterator Type Names Match Producing Methods

**Strength**: SHOULD

**Summary**: An `into_iter()` method should return an `IntoIter` type, `iter()` returns `Iter`, etc.

```rust
// Good - consistent naming
impl<T> Vec<T> {
    pub fn iter(&self) -> Iter<'_, T> { /* ... */ }
    pub fn iter_mut(&mut self) -> IterMut<'_, T> { /* ... */ }
    pub fn into_iter(self) -> IntoIter<T> { /* ... */ }
}

impl<K, V> BTreeMap<K, V> {
    pub fn keys(&self) -> Keys<'_, K, V> { /* ... */ }
    pub fn values(&self) -> Values<'_, K, V> { /* ... */ }
    pub fn iter(&self) -> Iter<'_, K, V> { /* ... */ }
}

// Good - function returning iterator
use url::percent_encoding;

fn percent_encode(input: &str) -> PercentEncode<'_> {
    percent_encoding::percent_encode(input.as_bytes())
}

// The type name matches the function name pattern
pub struct PercentEncode<'a> { /* ... */ }
```

**Rationale**: When type names match method names, documentation and error messages are more intuitive. The pattern `module::TypeName` makes it clear where the iterator came from.

---

## ID-25: Names Use Consistent Word Order

**Strength**: SHOULD

**Summary**: Within a crate (and ideally ecosystem-wide), use consistent word ordering in type and function names.

```rust
// Good - consistent verb-object-error order (from std)
pub struct ParseBoolError;
pub struct ParseCharError;
pub struct ParseFloatError;
pub struct ParseIntError;
pub struct ParseAddrError;  // Hypothetical, consistent with others

// Bad - inconsistent ordering
pub struct ParseBoolError;
pub struct CharParseError;   // Wrong - breaks pattern
pub struct ErrorParseFloat;  // Wrong - breaks pattern

// Good - consistent noun-direction order
pub struct IntoIter<T>;
pub struct IntoKeys<K, V>;
pub struct IntoValues<K, V>;

// Bad - inconsistent  
pub struct IntoIter<T>;
pub struct KeysInto<K, V>;   // Wrong - breaks pattern

// Good - error type patterns
pub enum Error {
    Io(io::Error),
    Parse(ParseError),
    Network(NetworkError),
    Database(DatabaseError),
}

// Good - consistent builder method order
impl RequestBuilder {
    pub fn with_header(self, key: &str, value: &str) -> Self { /* ... */ }
    pub fn with_timeout(self, duration: Duration) -> Self { /* ... */ }
    pub fn with_body(self, body: String) -> Self { /* ... */ }
    // All use "with_" prefix consistently
}
```

**Rationale**: Consistent word order makes APIs more predictable and easier to discover through autocomplete. When adding new items, developers can guess their names correctly.

**See also**: Standard library error types for examples, C-CASE for overall naming conventions

---

## ID-26: On-Stack Dynamic Dispatch

**Strength**: CONSIDER

**Summary**: Use enums instead of trait objects when the set of types is closed and you want stack allocation.

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

## ID-27: Option and Result Combinators

**Strength**: SHOULD

**Summary**: Use `?`, `map`, `and_then`, `unwrap_or` instead of verbose `match`.

```rust
// ❌ VERBOSE: Nested matches
fn get_user_email(id: i32) -> Option<String> {
    match find_user(id) {
        Some(user) => {
            match user.email {
                Some(email) => Some(email.to_lowercase()),
                None => None,
            }
        }
        None => None,
    }
}

// ✅ CONCISE: Combinators
fn get_user_email(id: i32) -> Option<String> {
    find_user(id)?
        .email
        .map(|e| e.to_lowercase())
}

// Common patterns:
let value = opt.unwrap_or(default);           // Provide default
let value = opt.unwrap_or_else(|| compute()); // Lazy default
let value = opt.ok_or(Error::NotFound)?;      // Convert to Result
let mapped = result.map_err(|e| wrap(e))?;    // Transform error
```

---

## ID-28: Panic Means 'Stop the Program'

**Strength**: MUST

**Summary**: Panics are not exceptions—they signal immediate program termination and should not be caught for error handling.

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

**See also**: M-PANIC-IS-STOP, M-PANIC-ON-BUG

---

## ID-29: Pass Variables to Closures

**Strength**: SHOULD

**Summary**: Use `move` closures to transfer ownership into the closure; clone before the closure if you need to keep the original.

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

## ID-30: Prefer `Option<T>` over Sentinel Values

**Strength**: MUST

**Summary**: Use `Option` to represent absence, not magic values like `-1` or `""`.

```rust
// ❌ BAD: Sentinel values
struct User {
    name: String,
    age: i32,        // -1 means "unknown"
    email: String,   // "" means "not provided"
}

// ✅ GOOD: Option for optional values
struct User {
    name: String,
    age: Option<u32>,        // None means "unknown"
    email: Option<String>,   // None means "not provided"
}

// Usage is explicit:
match user.age {
    Some(age) => println!("Age: {age}"),
    None => println!("Age unknown"),
}
```

**Rationale**: Sentinel values require documentation and can be forgotten. `Option` makes absence explicit in the type system and forces handling.

---

---

## ID-31: Prefer Regular Functions Over Associated Functions

**Strength**: SHOULD

**Summary**: Use regular functions for general computation; reserve associated functions primarily for instance creation.

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

## ID-32: Privacy for Extensibility (`non_exhaustive`)

**Strength**: SHOULD

**Summary**: Use `#[non_exhaustive]` on enums and structs in public APIs to allow adding variants/fields without breaking changes.

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

## ID-33: Return Consumed Arguments on Error

**Strength**: CONSIDER

**Summary**: When a function consumes an argument and can fail, return it in the error.

```rust
// ❌ LOSSY: Argument consumed even on failure
fn send_message(msg: Message) -> Result<(), SendError> {
    // If this fails, `msg` is lost!
    network.send(&msg)?;
    Ok(())
}

// ✅ RECOVERABLE: Return argument in error
struct SendError {
    message: Message,
    cause: std::io::Error,
}

fn send_message(msg: Message) -> Result<(), SendError> {
    match network.send(&msg) {
        Ok(()) => Ok(()),
        Err(cause) => Err(SendError { message: msg, cause }),
    }
}

// Caller can retry:
let mut msg = create_message();
for attempt in 0..3 {
    match send_message(msg) {
        Ok(()) => break,
        Err(e) => {
            msg = e.message;  // Recover the message
            log::warn!("Retry {}: {}", attempt, e.cause);
        }
    }
}
```

**Rationale**: Lets callers recover from errors without cloning arguments preemptively. See `String::from_utf8` for a stdlib example.

---

---

## ID-34: String Concatenation with `format!`

**Strength**: SHOULD

**Summary**: Use `format!` macro for string concatenation instead of manual `push_str` calls when readability matters.

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

## ID-35: Temporary Mutability

**Strength**: SHOULD

**Summary**: Limit mutable binding scope, then rebind as immutable. Use block scope to limit the lifetime of mutable bindings when you need temporary mutation.

```rust
// ✅ PATTERN 1: Nested scope
let data = {
    let mut temp = fetch_data();
    temp.sort();
    temp.dedup();
    temp  // Moved out as immutable
};
// `data` is now immutable

// ✅ PATTERN 2: Rebinding
let mut data = fetch_data();
data.sort();
data.dedup();
let data = data;  // Shadow as immutable
// `data` is now immutable

// ✅ PATTERN 3: Builder/method chain
let data = fetch_data()
    .into_iter()
    .sorted()
    .dedup()
    .collect::<Vec<_>>();
```

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

**Rationale**: Signals that mutation is complete. Prevents accidental modification later. Enables compiler optimizations.

--- Limiting the scope of mutability makes code easier to reason about and prevents accidental mutation. Rust's ownership system makes this pattern safe and efficient.

**See also**: Immutability by default, functional patterns in 08-performance.md

---

## ID-36: The Default Trait

**Strength**: SHOULD

**Summary**: Implement `Default` for types that have a sensible default value; use `#[derive(Default)]` when possible.

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

## ID-37: Use `format!` for String Concatenation

**Strength**: SHOULD

**Summary**: Prefer `format!` over manual string building for readability.

```rust
// ❌ VERBOSE: Manual concatenation
fn greeting(name: &str, age: u32) -> String {
    let mut result = "Hello, ".to_string();
    result.push_str(name);
    result.push_str("! You are ");
    result.push_str(&age.to_string());
    result.push_str(" years old.");
    result
}

// ✅ CLEAR: format! macro
fn greeting(name: &str, age: u32) -> String {
    format!("Hello, {name}! You are {age} years old.")
}
```

```rust
// For maximum performance with known capacity:
fn build_csv_row(fields: &[&str]) -> String {
    let mut result = String::with_capacity(fields.iter().map(|s| s.len() + 1).sum());
    for (i, field) in fields.iter().enumerate() {
        if i > 0 { result.push(','); }
        result.push_str(field);
    }
    result
}

// For simple two-string concatenation (slightly faster):
let full = first.to_string() + &second;
```

**Rationale**: `format!` is readable and handles Display/Debug formatting. Performance difference is negligible for most use cases.

---

---

## ID-38: Use `let else` for Early Returns

**Strength**: SHOULD

**Summary**: `let ... else` combines pattern matching with early return.

```rust
// ❌ VERBOSE: Match for early return
fn process_user(id: Option<i32>) -> Result<User, Error> {
    let id = match id {
        Some(id) => id,
        None => return Err(Error::MissingId),
    };
    // ... use id
}

// ✅ CONCISE: let-else
fn process_user(id: Option<i32>) -> Result<User, Error> {
    let Some(id) = id else {
        return Err(Error::MissingId);
    };
    // ... use id
}

// Works with any pattern:
fn parse_point(s: &str) -> Option<(i32, i32)> {
    let [x, y] = s.split(',').collect::<Vec<_>>()[..] else {
        return None;
    };
    Some((x.parse().ok()?, y.parse().ok()?))
}
```

**Rationale**: Reduces nesting and clearly separates the "happy path" from error handling.

---

---

## ID-39: Use Borrowed Types for Arguments

**Strength**: MUST

**Summary**: Prefer `&str` over `&String`, `&[T]` over `&Vec<T>`, and `&T` over `&Box<T>` for function parameters.

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

## ID-40: Use Borrowed Types for Function Arguments

**Strength**: SHOULD

**Summary**: Accept `&str` over `&String`, `&[T]` over `&Vec<T>`, `&T` over `&Box<T>`.

```rust
// ❌ AVOID: Overly specific parameter types
fn process(data: &Vec<i32>) { /* ... */ }
fn greet(name: &String) { /* ... */ }

// ✅ PREFER: Borrowed slices accept more input types
fn process(data: &[i32]) { /* ... */ }
fn greet(name: &str) { /* ... */ }

// Now both work:
greet("literal");                    // &str
greet(&String::from("owned"));       // &String coerces to &str
process(&[1, 2, 3]);                 // array
process(&vec![1, 2, 3]);             // Vec coerces to &[T]
```

**Rationale**: `&str` and `&[T]` are strictly more flexible. They accept the owned types via deref coercion while also accepting literals and slices. Using `&String` forces callers to allocate unnecessarily.

**Clippy**: clippy::ptr_arg

---

## ID-41: Use Iterators and Combinators

**Strength**: SHOULD

**Summary**: Prefer iterator chains over manual loops for transformations.

```rust
// ❌ IMPERATIVE: Manual loop
fn get_adult_names(people: &[Person]) -> Vec<String> {
    let mut names = Vec::new();
    for person in people {
        if person.age >= 18 {
            names.push(person.name.clone());
        }
    }
    names
}

// ✅ FUNCTIONAL: Iterator chain
fn get_adult_names(people: &[Person]) -> Vec<String> {
    people.iter()
        .filter(|p| p.age >= 18)
        .map(|p| p.name.clone())
        .collect()
}

// ✅ ALSO GOOD: Early return with find
fn find_admin(users: &[User]) -> Option<&User> {
    users.iter().find(|u| u.is_admin)
}

// ✅ ALSO GOOD: Combining iterators
fn merge_sorted(a: &[i32], b: &[i32]) -> Vec<i32> {
    let mut result: Vec<_> = a.iter().chain(b).copied().collect();
    result.sort();
    result
}
```

**Rationale**: Iterator chains are often more readable, enable lazy evaluation, and can be better optimized by the compiler.

---

---

## ID-42: Use Type Aliases for Complex Types

**Strength**: CONSIDER

**Summary**: Create type aliases when types become unwieldy.

```rust
// ❌ REPETITIVE: Same complex type everywhere
fn process(
    callback: Box<dyn Fn(&str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>
) { /* ... */ }

// ✅ CLEARER: Type alias
type ProcessResult = Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;
type ProcessCallback = Box<dyn Fn(&str) -> ProcessResult + Send + Sync>;

fn process(callback: ProcessCallback) { /* ... */ }

// Also useful for domain concepts:
type UserId = u64;
type Timestamp = i64;

fn get_user_activity(user: UserId, since: Timestamp) -> Vec<Activity> {
    // ...
}
```

**Rationale**: Reduces repetition, improves readability, creates vocabulary for your domain. Note: type aliases are transparent (unlike newtypes).

---

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