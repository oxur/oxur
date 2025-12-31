# API Design Guidelines

> Patterns for designing public APIs that are ergonomic, flexible, and stable.

---

## API-01: Naming Conventions

**Strength**: MUST

**Summary**: Follow Rust's standard naming conventions consistently.

```rust
// ✅ CORRECT naming conventions

// Types: UpperCamelCase
struct HttpRequest { }
enum ConnectionState { }
trait Serialize { }
type UserId = u64;

// Functions/methods: snake_case
fn process_request() { }
fn is_empty(&self) -> bool { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS: usize = 100;
static DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// Modules: snake_case
mod http_client;
mod error_handling;

// Type parameters: single uppercase or CamelCase
fn parse<T: FromStr>(s: &str) -> T { }
fn convert<Source, Target>(s: Source) -> Target { }

// Lifetimes: short lowercase, 'a is conventional
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { }
```

**Common method name conventions**:

| Pattern | Usage | Example |
|---------|-------|---------|
| `new` | Primary constructor | `Vec::new()` |
| `with_*` | Constructor with config | `Vec::with_capacity(10)` |
| `from_*` | Conversion from specific type | `String::from_utf8()` |
| `into_*` | Consuming conversion | `String::into_bytes()` |
| `as_*` | Cheap reference conversion | `str::as_bytes()` |
| `to_*` | Expensive conversion | `str::to_uppercase()` |
| `is_*` | Boolean query | `Option::is_some()` |
| `has_*` | Boolean containment | `str::has_pattern()` |
| `get_*` | Getter (usually just use field name) | `self.len()` not `self.get_len()` |
| `set_*` | Setter | `self.set_capacity(n)` |
| `*_mut` | Mutable variant | `slice::iter_mut()` |
| `try_*` | Fallible operation | `str::try_reserve()` |

---

## API-02: Method Receiver Guidelines

**Strength**: SHOULD

**Summary**: Choose the right `self` receiver for each method.

```rust
impl MyType {
    // &self: Read-only access, most common
    fn len(&self) -> usize {
        self.data.len()
    }
    
    // &mut self: Mutates in place
    fn push(&mut self, item: T) {
        self.data.push(item);
    }
    
    // self: Consumes the value, transforms it
    fn into_inner(self) -> Vec<T> {
        self.data
    }
    
    // No self: Associated function (constructor, utility)
    fn new() -> Self {
        Self { data: Vec::new() }
    }
}
```

**Guidelines**:
- Default to `&self` (non-consuming, shareable)
- Use `&mut self` only when mutation is needed
- Use `self` (consuming) for transformations (`into_*`) or when semantically consumed
- Avoid `mut self` (consuming + mutable) — usually `self` suffices

---

## API-03: Accept Borrowed, Return Owned

**Strength**: SHOULD

**Summary**: Functions should borrow inputs and return owned outputs by default.

```rust
// ✅ GOOD: Borrow input, return owned
fn process(input: &str) -> String {
    input.to_uppercase()
}

// ✅ GOOD: Return reference when returning part of input
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

// ✅ GOOD: Accept generic for flexibility
fn greet(name: impl AsRef<str>) -> String {
    format!("Hello, {}!", name.as_ref())
}

// Works with both:
greet("world");              // &str
greet(String::from("world")); // String

// ❌ AVOID: Forcing ownership when not needed
fn process(input: String) -> String {  // Caller must give up String
    input.to_uppercase()
}
```

**Flexibility traits for parameters**:

| Accept | Via Trait | Accepts |
|--------|-----------|---------|
| String-like | `impl AsRef<str>` | `&str`, `String`, `&String` |
| Path-like | `impl AsRef<Path>` | `&Path`, `PathBuf`, `&str` |
| Bytes | `impl AsRef<[u8]>` | `&[u8]`, `Vec<u8>`, `&str` |
| Any iterable | `impl IntoIterator<Item=T>` | `Vec<T>`, arrays, iterators |

---

## API-04: Implement Standard Traits

**Strength**: SHOULD

**Summary**: Implement common traits to make types work with the ecosystem.

```rust
// Essential traits for most types
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserId(u64);

// Additional traits based on usage:
#[derive(
    Debug,      // Required for {:?} formatting
    Clone,      // Explicit duplication
    Copy,       // Implicit copy (for small types)
    PartialEq,  // == and != comparison
    Eq,         // Marker: equality is reflexive
    PartialOrd, // <, >, <=, >= comparison
    Ord,        // Total ordering (for sorting, BTreeMap)
    Hash,       // For HashMap/HashSet keys
    Default,    // T::default() construction
)]
pub struct Point { x: i32, y: i32 }

// Display for user-facing output
use std::fmt;

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

// FromStr for parsing
use std::str::FromStr;

impl FromStr for UserId {
    type Err = ParseUserIdError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.strip_prefix("user:")
            .ok_or(ParseUserIdError::MissingPrefix)?
            .parse()
            .map_err(ParseUserIdError::InvalidNumber)?;
        Ok(UserId(id))
    }
}
```

**Trait checklist**:

| Trait | Implement when |
|-------|---------------|
| `Debug` | Always (use `#[derive]`) |
| `Clone` | Type can be duplicated |
| `Copy` | Type is small, trivially copyable |
| `PartialEq` | Type can be compared for equality |
| `Eq` | Equality is total (not NaN-like) |
| `Hash` | Used as HashMap/HashSet key |
| `Default` | Meaningful zero/empty value exists |
| `Display` | User-facing string representation |
| `FromStr` | Can be parsed from string |
| `From`/`Into` | Lossless conversions exist |
| `TryFrom`/`TryInto` | Fallible conversions exist |
| `Deref` | Smart pointer / wrapper types only |

---

## API-05: Error Types for Public APIs

**Strength**: MUST

**Summary**: Public functions should return specific error types, not strings or boxes.

```rust
// ❌ BAD: Opaque error
pub fn connect(addr: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    todo!()
}

// ❌ BAD: String error
pub fn connect(addr: &str) -> Result<Connection, String> {
    todo!()
}

// ✅ GOOD: Specific error type
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("invalid address: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),
    
    #[error("connection refused")]
    ConnectionRefused,
    
    #[error("timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("TLS error: {0}")]
    Tls(#[from] TlsError),
}

pub fn connect(addr: &str) -> Result<Connection, ConnectError> {
    todo!()
}

// Users can now match on specific errors:
match connect("localhost:8080") {
    Ok(conn) => use_connection(conn),
    Err(ConnectError::Timeout(d)) => retry_with_longer_timeout(d),
    Err(ConnectError::ConnectionRefused) => try_fallback_server(),
    Err(e) => return Err(e.into()),
}
```

---

## API-06: Use `impl Trait` in Return Position Judiciously

**Strength**: CONSIDER

**Summary**: `-> impl Trait` hides the concrete type — use when that's desirable.

```rust
// ✅ GOOD: Complex iterator type hidden
pub fn items(&self) -> impl Iterator<Item = &Item> {
    self.data.iter().filter(|i| i.is_active())
}

// ✅ GOOD: Closure type (unnameable)
pub fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

// ❌ QUESTIONABLE: Simple type hidden unnecessarily
pub fn get_names(&self) -> impl Iterator<Item = &str> {
    self.names.iter().map(String::as_str)
}

// ✅ BETTER: Return concrete type when simple
pub fn get_names(&self) -> std::slice::Iter<'_, String> {
    self.names.iter()
}

// Or document the hidden type:
/// Returns an iterator over active items.
/// 
/// The concrete iterator type is an implementation detail.
pub fn items(&self) -> impl Iterator<Item = &Item> {
    // ...
}
```

**When to use `impl Trait`**:
- Complex/nested types that would be painful to name
- Types that might change (implementation detail)
- Closure return types (unnameable)

**When NOT to use**:
- Simple types like `Vec<T>`, `Option<T>`
- When users need to name the type (store in struct)
- When users need type-specific methods

---

## API-07: Provide `Default` Implementations

**Strength**: SHOULD

**Summary**: Implement `Default` when there's a sensible "empty" or "default" value.

```rust
#[derive(Debug)]
pub struct Config {
    pub timeout: Duration,
    pub retries: u32,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retries: 3,
            verbose: false,
        }
    }
}

// Enables convenient patterns:
let config = Config::default();
let config = Config { verbose: true, ..Default::default() };

// Works with Option::unwrap_or_default()
fn get_config() -> Option<Config> { None }
let config = get_config().unwrap_or_default();

// For simple cases, derive it:
#[derive(Debug, Default)]
pub struct Counter {
    value: u64,  // Defaults to 0
}
```

---

## API-08: Document Public APIs

**Strength**: MUST

**Summary**: Every public item should have documentation.

```rust
/// A thread-safe counter with atomic operations.
/// 
/// # Examples
/// 
/// ```
/// use my_crate::Counter;
/// 
/// let counter = Counter::new();
/// counter.increment();
/// assert_eq!(counter.get(), 1);
/// ```
#[derive(Debug)]
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    /// Creates a new counter starting at zero.
    pub fn new() -> Self {
        Self { value: AtomicU64::new(0) }
    }
    
    /// Increments the counter by one.
    /// 
    /// Returns the previous value.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use my_crate::Counter;
    /// let counter = Counter::new();
    /// let prev = counter.increment();
    /// assert_eq!(prev, 0);
    /// assert_eq!(counter.get(), 1);
    /// ```
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Returns the current value.
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }
}
```

**Documentation checklist**:
- [ ] One-line summary for every public item
- [ ] Longer description for complex items
- [ ] `# Examples` section with runnable code
- [ ] `# Panics` if the function can panic
- [ ] `# Errors` describing when `Result::Err` is returned
- [ ] `# Safety` for unsafe functions

---

## API-09: Validate Early, Fail Fast

**Strength**: SHOULD

**Summary**: Check invariants at API boundaries, not deep in implementation.

```rust
// ❌ BAD: Error discovered deep in call stack
pub fn process_batch(items: Vec<Item>) -> Result<(), ProcessError> {
    for item in items {
        self.process_one(item)?;  // Might fail on item 500
    }
    Ok(())
}

// ✅ GOOD: Validate upfront
pub fn process_batch(items: Vec<Item>) -> Result<(), ProcessError> {
    // Validate all items first
    for (i, item) in items.iter().enumerate() {
        item.validate()
            .map_err(|e| ProcessError::InvalidItem { index: i, source: e })?;
    }
    
    // Now process (won't fail validation)
    for item in items {
        self.process_one_unchecked(item);
    }
    Ok(())
}

// ✅ GOOD: Use types that enforce validity
pub struct ValidatedItem { /* private fields */ }

impl ValidatedItem {
    pub fn new(item: Item) -> Result<Self, ValidationError> {
        // Validate here, once
        todo!()
    }
}

pub fn process_batch(items: Vec<ValidatedItem>) {
    // Can't receive invalid items!
    for item in items {
        self.process_one(item);
    }
}
```

---

## API-10: Sealed Traits for Extensible-but-Closed APIs

**Strength**: CONSIDER

**Summary**: Seal traits when you want to add methods without breaking changes.

```rust
// Public trait that external code can't implement
mod private {
    pub trait Sealed {}
}

/// A database backend.
/// 
/// This trait is sealed and cannot be implemented outside this crate.
pub trait Backend: private::Sealed {
    fn execute(&self, query: &str) -> Result<(), Error>;
    
    // Can add new methods in future versions without breaking changes
    fn execute_batch(&self, queries: &[&str]) -> Result<(), Error> {
        for q in queries {
            self.execute(q)?;
        }
        Ok(())
    }
}

// Only your types can implement it:
pub struct PostgresBackend { /* ... */ }

impl private::Sealed for PostgresBackend {}
impl Backend for PostgresBackend {
    fn execute(&self, query: &str) -> Result<(), Error> {
        todo!()
    }
}
```

**When to seal**:
- Trait methods might be added in future
- You provide all implementations
- External implementations would be incorrect/unsafe

---

## API-11: Avoid Stringly-Typed APIs

**Strength**: SHOULD

**Summary**: Use enums and types instead of strings for finite choices.

```rust
// ❌ BAD: String for finite options
pub fn set_log_level(level: &str) {
    match level {
        "debug" | "DEBUG" => { /* ... */ }
        "info" | "INFO" => { /* ... */ }
        _ => panic!("unknown level"),  // Runtime error!
    }
}

// ✅ GOOD: Enum for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub fn set_log_level(level: LogLevel) {
    match level {
        LogLevel::Debug => { /* ... */ }
        LogLevel::Info => { /* ... */ }
        LogLevel::Warn => { /* ... */ }
        LogLevel::Error => { /* ... */ }
    }
}

// Provide FromStr if users need to parse from config:
impl std::str::FromStr for LogLevel {
    type Err = ParseLogLevelError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(ParseLogLevelError(s.to_string())),
        }
    }
}
```

---

## API-12: Extension Traits for Foreign Types

**Strength**: CONSIDER

**Summary**: Add methods to external types via extension traits.

```rust
/// Extension methods for `Option<String>`.
pub trait OptionStringExt {
    /// Returns the string or an empty string if None.
    fn unwrap_or_empty(self) -> String;
    
    /// Returns true if the option contains a non-empty string.
    fn is_non_empty(&self) -> bool;
}

impl OptionStringExt for Option<String> {
    fn unwrap_or_empty(self) -> String {
        self.unwrap_or_default()
    }
    
    fn is_non_empty(&self) -> bool {
        self.as_ref().map_or(false, |s| !s.is_empty())
    }
}

// Usage (after `use OptionStringExt`):
let name: Option<String> = None;
let display = name.unwrap_or_empty();
```

**Naming convention**: `{Type}Ext` or `{Capability}Ext`

---

## Summary: API Design Checklist

**Naming**:
- [ ] Types are `UpperCamelCase`
- [ ] Functions are `snake_case`
- [ ] Constants are `SCREAMING_SNAKE_CASE`
- [ ] Methods follow `is_*`, `as_*`, `to_*`, `into_*` conventions

**Signatures**:
- [ ] Accept `&T` or `impl AsRef<T>` for inputs
- [ ] Return owned types (or references into input)
- [ ] Use specific error types, not `Box<dyn Error>`

**Traits**:
- [ ] `Debug` implemented (always)
- [ ] `Clone`, `PartialEq` where appropriate
- [ ] `Default` for types with sensible defaults
- [ ] `Display` for user-facing types
- [ ] `From`/`TryFrom` for conversions

**Ergonomics**:
- [ ] Builder pattern for complex construction
- [ ] `#[non_exhaustive]` for public enums/structs
- [ ] Extension traits for adding methods to foreign types

**Documentation**:
- [ ] Every public item documented
- [ ] Examples for complex APIs
- [ ] Panics/Errors sections where applicable

---

*See also: [13-documentation.md](13-documentation.md) for documentation patterns.*
