# Anti-Patterns in Rust

> Critical patterns to avoid when writing Rust code

**Purpose**: This document catalogs common mistakes that AI models frequently generate. Understanding these anti-patterns is essential for producing correct, idiomatic Rust code.

## Table of Contents

1. [Error Handling Anti-Patterns](#error-handling-anti-patterns)
2. [Ownership and Borrowing Anti-Patterns](#ownership-and-borrowing-anti-patterns)
3. [Performance Anti-Patterns](#performance-anti-patterns)
4. [Type System Anti-Patterns](#type-system-anti-patterns)
5. [API Design Anti-Patterns](#api-design-anti-patterns)
6. [Complexity Anti-Patterns](#complexity-anti-patterns)
7. [Safety Anti-Patterns](#safety-anti-patterns)

---

## Error Handling Anti-Patterns

### Unwrapping in Production Code

**Strength**: AVOID

**Summary**: Using `.unwrap()` or `.expect()` in production code causes panics instead of graceful error handling.

**Example**:
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

**Clippy Lints**: `unwrap_used`, `expect_used` (restriction category)

**Exceptions**: 
- Test code: `#[cfg(test)]` modules can freely use `unwrap()`
- Constants: Compile-time unwrapping is safe
- Prototypes: Early development code

**See Also**: [03-error-handling.md](03-error-handling.md)

---

### Ignoring Errors with `let _ =`

**Strength**: AVOID

**Summary**: Silently discarding Result values masks errors that should be handled.

**Example**:
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

**Clippy Lints**: `let_underscore_must_use`, `unused_must_use`

---

### Using `panic!()` for Error Conditions

**Strength**: AVOID

**Summary**: Use `Result` for expected error conditions; reserve `panic!()` for programmer errors and invariant violations.

**Example**:
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

**Clippy Lints**: `panic`, `unreachable`, `todo`, `unimplemented` (restriction category)

---

### Nested Result/Option Unwrapping

**Strength**: AVOID

**Summary**: Multiple unwraps create fragile code that's hard to debug when it fails.

**Example**:
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

**Clippy Lints**: `unwrap_used`, `option_map_unwrap_or`

---

## Ownership and Borrowing Anti-Patterns

### Unnecessary Cloning

**Strength**: AVOID  

**Summary**: Cloning data when references would suffice wastes memory and CPU.

**Example**:
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

**Clippy Lints**: `clone_on_copy`, `clone_double_ref`, `redundant_clone`

---

### Fighting the Borrow Checker with Clones

**Strength**: AVOID

**Summary**: Adding clones to satisfy the borrow checker usually indicates a design problem.

**Example**:
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

**Clippy Lints**: `redundant_clone`, `unnecessary_to_owned`

---

### Overusing `String` Instead of `&str`

**Strength**: AVOID

**Summary**: Taking owned `String` when `&str` suffices forces unnecessary allocations.

**Example**:
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

**Clippy Lints**: `unnecessary_to_owned`, `str_to_string`

---

### Returning References to Local Variables

**Strength**: MUST NOT (won't compile)

**Summary**: Cannot return references to stack-allocated data.

**Example**:
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

**Clippy Lints**: This is caught by the compiler, not Clippy.

---

## Performance Anti-Patterns

### Using `Vec` When Array Would Suffice

**Strength**: CONSIDER

**Summary**: Fixed-size collections can use stack-allocated arrays instead of heap-allocated vectors.

**Example**:
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

**Clippy Lints**: `useless_vec`

---

### Collecting Iterator Just to Iterate Again

**Strength**: AVOID

**Summary**: Unnecessary intermediate collections waste memory and time.

**Example**:
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

**Clippy Lints**: `needless_collect`, `unnecessary_collect`

---

### String Concatenation in Loops

**Strength**: AVOID

**Summary**: Using `+` to concatenate strings in loops performs O(n²) allocations.

**Example**:
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

**Clippy Lints**: `string_add`, `string_add_assign`

---

### Needless `collect()` Before `join()`

**Strength**: AVOID

**Summary**: Many iterators can be joined directly without collecting first.

**Example**:
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

**Clippy Lints**: `unnecessary_collect`

---

## Type System Anti-Patterns

### Using `Box` Without Needing Indirection

**Strength**: AVOID

**Summary**: Boxing adds heap allocation overhead without benefit unless you need indirection or trait objects.

**Example**:
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

**Clippy Lints**: `box_default`, `unnecessary_box`

---

### Option\<Option\<T\>\>

**Strength**: AVOID

**Summary**: Nested Options usually indicate a design problem.

**Example**:
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

**Clippy Lints**: `option_option`

---

### Using `String` as an Error Type

**Strength**: AVOID

**Summary**: `String` errors lack structure and type safety.

**Example**:
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

**Clippy Lints**: `string_slice` (indirectly related)

**See Also**: [03-error-handling.md](03-error-handling.md)

---

## API Design Anti-Patterns

### Public API with Implementation Details

**Strength**: AVOID

**Summary**: Exposing internal types in public APIs creates tight coupling.

**Example**:
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

**Clippy Lints**: Not directly linted, but against API guidelines

**See Also**: [02-api-design.md](02-api-design.md)

---

### Taking Concrete Types Instead of Traits

**Strength**: CONSIDER

**Summary**: Generic trait bounds make APIs more flexible.

**Example**:
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

**Clippy Lints**: `ptr_arg`

---

### Builder Pattern Without Consuming Self

**Strength**: CONSIDER

**Summary**: Builder methods should consume `self` to prevent reuse bugs.

**Example**:
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

**Clippy Lints**: `wrong_self_convention`

---

## Complexity Anti-Patterns

### Needless Boolean Comparisons

**Strength**: AVOID

**Summary**: Comparing booleans to `true` or `false` is redundant.

**Example**:
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

**Clippy Lints**: `bool_comparison`, `needless_bool`

---

### Match on Boolean

**Strength**: AVOID

**Summary**: Use `if` for boolean conditions, not `match`.

**Example**:
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

**Clippy Lints**: `match_bool`

---

### Single Match to If Let

**Strength**: CONSIDER

**Summary**: Single-arm matches are better expressed as `if let`.

**Example**:
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

**Clippy Lints**: `single_match`, `single_match_else`

---

### Redundant Field Names in Struct Literals

**Strength**: AVOID

**Summary**: Use field init shorthand when variable name matches field name.

**Example**:
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

**Clippy Lints**: `redundant_field_names`

---

## Safety Anti-Patterns

### Transmuting Between Incompatible Types

**Strength**: MUST NOT

**Summary**: `transmute` bypasses type safety and is almost always wrong.

**Example**:
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

**Clippy Lints**: `transmute_bytes_to_str`, `transmute_ptr_to_ptr`, `wrong_transmute`

---

### Dereferencing Raw Pointers Without Safety Comments

**Strength**: MUST (when using unsafe)

**Summary**: Every unsafe block should have a comment explaining why it's safe.

**Example**:
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

**Clippy Lints**: `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`

---

### Leaking Resources with `mem::forget`

**Strength**: AVOID

**Summary**: `mem::forget` prevents destructors from running, causing resource leaks.

**Example**:
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

**Clippy Lints**: `mem_forget`

---

## Summary Checklist

When reviewing Rust code, check for these categories:

### Error Handling
- [ ] No `unwrap()`/`expect()` in production code
- [ ] Results are not ignored with `let _ =`
- [ ] Errors use structured types, not `String`
- [ ] Panic only for programmer errors

### Ownership & Borrowing
- [ ] No unnecessary `clone()`
- [ ] Use `&str` instead of `String` in function parameters
- [ ] References used where ownership isn't needed

### Performance
- [ ] Arrays used instead of Vec for fixed sizes
- [ ] Iterators not collected unnecessarily
- [ ] Strings concatenated with `push_str`, not `+`

### Type System
- [ ] No unnecessary `Box`
- [ ] No nested `Option<Option<T>>`
- [ ] Concrete error types instead of `String`

### API Design
- [ ] Implementation details are private
- [ ] Generic trait bounds used for flexibility
- [ ] Builders consume `self`

### Complexity
- [ ] `if` used for booleans, not `match`
- [ ] Field init shorthand used
- [ ] `if let` preferred over single-arm `match`

### Safety
- [ ] Unsafe blocks have SAFETY comments
- [ ] No transmute without careful justification
- [ ] Resources cleaned up via RAII

---

**Note**: This document focuses on patterns AI models frequently generate incorrectly. It's not exhaustive but covers the most critical issues.
