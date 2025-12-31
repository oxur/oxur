# Anti-Patterns

Critical patterns to AVOID in Rust code. AI models frequently generate these mistakes.

## Type System Anti-Patterns

### ❌ Using () as Error Type

**Problem**: `()` provides no information about what went wrong and doesn't integrate with error handling.

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

**Why it's wrong**: Error types must be informative and implement `Error` trait.

---

### ❌ Adding Unnecessary Trait Bounds to Structs

**Problem**: Trait bounds on struct definitions that are only needed for derived traits limit the type's usability.

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

**Why it's wrong**: Derives automatically add trait bounds to `impl` blocks. Adding them to the struct is redundant and limiting.

---

### ❌ Public Fields Without Invariants

**Problem**: Public mutable fields prevent adding validation and make refactoring impossible.

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

**Why it's wrong**: Public fields lock you into a representation and prevent maintaining invariants.

---

### ❌ Using Deref for Type Conversions

**Problem**: `Deref` should only be used for smart pointers, not for type conversions.

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

**Why it's wrong**: `Deref` is meant for smart pointers like `Box`, `Rc`, `Arc`. Using it for conversions causes confusing behavior.

---

### ❌ Inherent Methods on Smart Pointers

**Problem**: Smart pointer types shouldn't have inherent methods that could conflict with methods on the inner type.

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

**Why it's wrong**: Smart pointers implement `Deref`, so methods would be ambiguous with the inner type's methods.

---

## Error Handling Anti-Patterns

### ❌ Silently Ignoring Errors

**Problem**: Using `.ok()`, `.unwrap_or()`, or `let _ =` to discard errors hides bugs.

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

**Why it's wrong**: Silently ignoring errors masks bugs and makes debugging impossible.

---

### ❌ Using unwrap() in Library Code

**Problem**: `unwrap()` panics instead of returning errors, forcing caller to handle panics.

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

**Why it's wrong**: Library code should return `Result` to let callers decide how to handle errors. Only application code (`main`, tests) should panic.

---

### ❌ Using Option When Error Info Needed

**Problem**: `Option` loses information about why the operation failed.

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

**Why it's wrong**: `Result` conveys _why_ something failed, not just _that_ it failed.

---

## Naming Anti-Patterns

### ❌ Using get_ Prefix Unnecessarily

**Problem**: Rust getters don't use `get_` prefix except for specific cases.

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

**Why it's wrong**: Rust convention is to omit `get_` prefix for simple getters.

---

### ❌ Inconsistent Word Order

**Problem**: Inconsistent naming makes APIs harder to discover and remember.

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

**Why it's wrong**: Consistency aids discoverability and reduces cognitive load.

---

### ❌ Using Placeholder Words in Feature Names

**Problem**: Words like `use-`, `with-`, or `enable-` add no meaning.

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

**Why it's wrong**: Cargo's implicit features don't use prefixes, so explicit features shouldn't either.

---

## Iterator Anti-Patterns

### ❌ Collecting to Vec Unnecessarily

**Problem**: Collecting to `Vec` when the iterator could be used directly.

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

**Why it's wrong**: Collecting allocates memory and does extra work when the iterator could be consumed directly.

---

### ❌ Manual Index Loops Instead of Iterators

**Problem**: Using manual indexing is more error-prone and less idiomatic than iterators.

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

**Why it's wrong**: Iterators are safer (no index out of bounds), clearer, and often more efficient.

---

## Clone Anti-Patterns

### ❌ Cloning to Satisfy Borrow Checker

**Problem**: Cloning to work around borrow checker instead of fixing the actual issue.

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

**Why it's wrong**: Cloning has a performance cost. Understanding borrowing produces better code.

---

### ❌ Cloning in Loops

**Problem**: Cloning inside a loop when a reference would work.

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

**Why it's wrong**: Unnecessary clones waste memory and CPU time.

---

## String Anti-Patterns

### ❌ Using String When &str Works

**Problem**: Requiring `String` when a string slice would work forces allocation.

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

**Why it's wrong**: `String` forces allocation. `&str` is more flexible and efficient.

---

### ❌ Using format! for Simple Concatenation

**Problem**: `format!` for simple concatenation is slower than alternatives.

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

**Why it's wrong**: `format!` has overhead. Simple concatenation is faster for basic cases.

---

## Memory Anti-Patterns

### ❌ Boxing Unnecessarily

**Problem**: Using `Box` when stack allocation would work.

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

**Why it's wrong**: Boxing has allocation overhead. Stack allocation is faster.

---

This anti-patterns chapter covers the most common mistakes that AI models and developers make. Understanding these helps avoid them in the first place.
