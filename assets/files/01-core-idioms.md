# Core Rust Idioms

> Essential patterns that should appear in virtually every Rust codebase. These are the "social norms" of Rust programming.

---

## ID-01: Use Borrowed Types for Function Arguments

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

**Clippy**: `clippy::ptr_arg`

---

## ID-02: Constructors via `new` and `Default`

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

**Rationale**: 
- `new` is the conventional constructor name in Rust
- `Default` enables `..Default::default()` syntax and works with generic code
- If `new()` takes no arguments, it should behave identically to `Default::default()`

**See also**: Builder pattern for complex construction

---

## ID-03: Derive Common Traits

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

**Common derive combinations**:

| Type Kind | Typical Derives |
|-----------|-----------------|
| Simple data | `Debug, Clone, PartialEq` |
| Copy-able data | `Debug, Clone, Copy, PartialEq` |
| HashMap keys | `Debug, Clone, PartialEq, Eq, Hash` |
| Ordered data | `Debug, Clone, PartialEq, Eq, PartialOrd, Ord` |
| Config/DTOs | `Debug, Clone, PartialEq, Serialize, Deserialize` |
| Errors | `Debug, thiserror::Error` |

**Rationale**: These traits enable debugging, collections, testing, and serialization. Deriving them costs nothing if unused but enables many use cases.

---

## ID-04: Use `format!` for String Concatenation

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

**When to use alternatives**:
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

## ID-05: `mem::take` and `mem::replace` for Owned Values in Enums

**Strength**: SHOULD (when applicable)

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

**Key functions**:
- `mem::take(&mut value)` — Replaces with `Default::default()`, returns old value
- `mem::replace(&mut value, new)` — Replaces with `new`, returns old value

**Rationale**: Avoids cloning when you need to move a value out of a mutable reference. Zero-cost for types where `Default` is cheap (String, Vec, Option).

---

## ID-06: Use Iterators and Combinators

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

**Common combinators**:

| Combinator | Purpose |
|------------|---------|
| `map` | Transform each element |
| `filter` | Keep elements matching predicate |
| `filter_map` | Filter + map in one (for `Option` results) |
| `flat_map` | Map then flatten nested iterators |
| `fold` | Accumulate into single value |
| `find` | First element matching predicate |
| `any` / `all` | Boolean checks |
| `take` / `skip` | Limit iteration |
| `enumerate` | Add indices |
| `zip` | Pair with another iterator |

**Rationale**: Iterator chains are often more readable, enable lazy evaluation, and can be better optimized by the compiler.

---

## ID-07: Option and Result Combinators

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

**Key combinators**:

| On `Option<T>` | Purpose |
|----------------|---------|
| `?` | Return `None` early |
| `map(f)` | `Some(x)` → `Some(f(x))` |
| `and_then(f)` | `Some(x)` → `f(x)` (where f returns Option) |
| `unwrap_or(v)` | `Some(x)` → `x`, `None` → `v` |
| `ok_or(e)` | Convert to `Result` |

| On `Result<T, E>` | Purpose |
|-------------------|---------|
| `?` | Return `Err` early |
| `map(f)` | `Ok(x)` → `Ok(f(x))` |
| `map_err(f)` | `Err(e)` → `Err(f(e))` |
| `and_then(f)` | `Ok(x)` → `f(x)` |
| `unwrap_or_else(f)` | `Ok(x)` → `x`, `Err(e)` → `f(e)` |

---

## ID-08: Temporary Mutability

**Strength**: SHOULD

**Summary**: Limit mutable binding scope, then rebind as immutable.

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

**Rationale**: Signals that mutation is complete. Prevents accidental modification later. Enables compiler optimizations.

---

## ID-09: Destructors for Finalization (RAII)

**Strength**: SHOULD (for resource management)

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

**Use cases**:
- File handles, sockets, database connections
- Locks (MutexGuard already does this)
- Temporary resources
- Logging/metrics for scope duration

**Warning**: Drop is *not* guaranteed to run (infinite loop, `mem::forget`, process abort). Don't rely on it for critical data persistence.

---

## ID-10: `#[non_exhaustive]` for Public Enums and Structs

**Strength**: SHOULD (for libraries)

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

## ID-11: Prefer `Option<T>` over Sentinel Values

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

## ID-12: Use Type Aliases for Complex Types

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

## ID-13: Iterate Over `Option` When Useful

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

## ID-14: Return Consumed Arguments on Error

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

## ID-15: Use `let else` for Early Returns

**Strength**: SHOULD (Rust 1.65+)

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

## Summary: Essential Idioms Checklist

When writing Rust code, verify:

- [ ] Function parameters use `&str`, `&[T]` not `&String`, `&Vec<T>`
- [ ] Types implement `Debug`, `Clone`, and other appropriate derives
- [ ] `Default` implemented for types with sensible defaults
- [ ] Constructors named `new` (or `with_*`, `from_*` for variants)
- [ ] `format!` used for string concatenation (unless perf-critical)
- [ ] Iterator combinators preferred over manual loops
- [ ] `?` and combinators used instead of verbose `match`
- [ ] Mutable bindings scoped minimally
- [ ] Resources use RAII pattern (`Drop` for cleanup)
- [ ] Optional values use `Option`, not sentinel values
- [ ] Library types marked `#[non_exhaustive]` where appropriate

---

*See also: [11-anti-patterns.md](11-anti-patterns.md) for what to avoid.*
