# Traits

Guidelines for designing traits, implementation patterns, and using trait objects effectively.

## Trait Design

### Sealed Traits Protect Against Downstream Implementations

**Strength**: CONSIDER

**Summary**: Use the sealed trait pattern to prevent external implementations while preserving the ability to add methods.

**Examples**:

```rust
// Good - sealed trait pattern
/// This trait is sealed and cannot be implemented outside this crate.
pub trait Operation: private::Sealed {
    fn execute(&self) -> Result<(), Error>;
    
    // Can add methods in minor versions without breaking changes
    fn validate(&self) -> bool {
        true
    }
    
    // Private methods not shown in public docs
    #[doc(hidden)]
    fn internal_id(&self) -> u64;
}

// Implementations in this crate
pub struct Add;
pub struct Subtract;

impl Operation for Add {
    fn execute(&self) -> Result<(), Error> {
        // ...
    }
    
    fn internal_id(&self) -> u64 { 1 }
}

impl Operation for Subtract {
    fn execute(&self) -> Result<(), Error> {
        // ...
    }
    
    fn internal_id(&self) -> u64 { 2 }
}

// Private module prevents external implementation
mod private {
    pub trait Sealed {}
    
    impl Sealed for super::Add {}
    impl Sealed for super::Subtract {}
}

// External crates cannot implement Operation:
// impl Operation for MyType {}  // Error: private::Sealed not in scope

// Benefits:
// 1. Can add methods to Operation without breaking change
// 2. Can change method signatures (if not public)
// 3. Can add private methods
// 4. Exhaustive matching is sound

fn process(op: &dyn Operation) {
    match op {
        // Compiler knows all implementations are in this crate
        _ if op.internal_id() == 1 => println!("Add"),
        _ if op.internal_id() == 2 => println!("Subtract"),
        _ => unreachable!(),
    }
}
```

**When to use sealed traits**:
- ✅ Trait has exhaustive set of implementations
- ✅ Need to add methods in future versions
- ✅ Want to maintain implementation invariants
- ✅ Trait is used for internal polymorphism

**When NOT to seal**:
- ❌ Trait is meant to be implemented by users
- ❌ Trait defines a general capability (Read, Write, Iterator)
- ❌ Extensibility is a key feature

**Documentation**: Always document that a trait is sealed:

```rust
/// Represents a database operation.
///
/// This trait is sealed and cannot be implemented outside of this crate.
/// See the [list of implementors](#implementors) for available operations.
pub trait Operation: private::Sealed {
    // ...
}
```

**Rationale**: Sealed traits enable non-breaking evolution while maintaining backwards compatibility.

**See also**: C-SEALED

---

## Common Trait Implementations

### Implement Common Derive Traits

**Strength**: MUST

**Summary**: Types should derive or implement Clone, Debug, PartialEq, and other common traits where applicable.

**Examples**:

```rust
// Good - comprehensive trait implementations
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    x: i32,
    y: i32,
}

// Good - selective traits for types with interior mutability
use std::cell::RefCell;

#[derive(Debug)]  // Debug but not Clone/PartialEq
pub struct Counter {
    count: RefCell<u32>,
}

impl Counter {
    pub fn increment(&self) {
        *self.count.borrow_mut() += 1;
    }
}

// Good - custom implementations when derive doesn't work
use std::fmt;

pub struct CustomPoint {
    x: f64,
    y: f64,
}

impl Clone for CustomPoint {
    fn clone(&self) -> Self {
        CustomPoint {
            x: self.x,
            y: self.y,
        }
    }
}

impl fmt::Debug for CustomPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CustomPoint")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

// PartialEq for f64 needs custom logic
impl PartialEq for CustomPoint {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < f64::EPSILON &&
        (self.y - other.y).abs() < f64::EPSILON
    }
}

// Good - Default implementation
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    timeout: u64,
    retries: u32,
}

// Or manual Default
impl Default for Config {
    fn default() -> Self {
        Config {
            timeout: 30,
            retries: 3,
        }
    }
}
```

**Essential traits checklist**:

| Trait | When to implement |
|-------|-------------------|
| `Clone` | Unless type manages unique resources |
| `Copy` | For small, trivial types only |
| `Debug` | Always (except maybe for private types) |
| `PartialEq` | For types that can be compared |
| `Eq` | When equality is reflexive (a == a) |
| `PartialOrd` | For types with meaningful ordering |
| `Ord` | When ordering is total |
| `Hash` | For types used as HashMap keys |
| `Default` | For types with a sensible default value |
| `Display` | For types with user-facing representation |

**When NOT to implement Clone**:
```rust
// Don't clone file handles
pub struct FileHandle {
    fd: RawFd,
}

// Don't clone database connections
pub struct DbConnection {
    conn: *mut sqlite3,
}

// Don't clone mutex guards
// (MutexGuard doesn't implement Clone by design)
```

**See also**: C-COMMON-TRAITS

---

### Implement Display for User-Facing Types

**Strength**: SHOULD

**Summary**: Types that users will see should implement Display with clear, helpful messages.

**Examples**:

```rust
use std::fmt;

// Good - Display for error types
#[derive(Debug)]
pub enum ValidationError {
    TooShort { min: usize, actual: usize },
    TooLong { max: usize, actual: usize },
    InvalidCharacter { ch: char, position: usize },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::TooShort { min, actual } => {
                write!(f, "input too short: expected at least {} characters, got {}", min, actual)
            }
            ValidationError::TooLong { max, actual } => {
                write!(f, "input too long: expected at most {} characters, got {}", max, actual)
            }
            ValidationError::InvalidCharacter { ch, position } => {
                write!(f, "invalid character '{}' at position {}", ch, position)
            }
        }
    }
}

// Good - Display for domain types
pub struct EmailAddress(String);

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmailAddress(\"{}\")", self.0)
    }
}

// Usage shows the difference:
let email = EmailAddress("user@example.com".to_string());
println!("{}", email);     // user@example.com
println!("{:?}", email);   // EmailAddress("user@example.com")

// Good - Display for IDs
pub struct UserId(u64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User#{}", self.0)
    }
}

impl fmt::Debug for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserId({})", self.0)
    }
}

// Bad - Display same as Debug
impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)  // Don't do this
    }
}
```

**Display vs Debug**:
- **Display** - User-facing, no delimiters, clean output
- **Debug** - Developer-facing, shows structure, includes type info

```rust
let value = EmailAddress("test@example.com".to_string());

// Display: clean for users
assert_eq!(format!("{}", value), "test@example.com");

// Debug: structural for developers
assert_eq!(format!("{:?}", value), "EmailAddress(\"test@example.com\")");
```

**Rationale**: Display provides clean output for users while Debug is for developers.

---

## Trait Object Patterns

### Use dyn Trait for Runtime Polymorphism

**Strength**: SHOULD

**Summary**: Use trait objects (dyn Trait) for heterogeneous collections or when type erasure is needed.

**Examples**:

```rust
// Good - trait object for plugins
pub trait Plugin {
    fn name(&self) -> &str;
    fn execute(&mut self) -> Result<(), Error>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    pub fn run_all(&mut self) -> Result<(), Error> {
        for plugin in &mut self.plugins {
            plugin.execute()?;
        }
        Ok(())
    }
}

// Good - trait object for rendering
pub trait Drawable {
    fn draw(&self, canvas: &mut Canvas);
    fn bounds(&self) -> Rect;
}

pub struct Scene {
    objects: Vec<Box<dyn Drawable>>,
}

impl Scene {
    pub fn render(&self, canvas: &mut Canvas) {
        for obj in &self.objects {
            obj.draw(canvas);
        }
    }
    
    pub fn add<T: Drawable + 'static>(&mut self, obj: T) {
        self.objects.push(Box::new(obj));
    }
}

// Usage
let mut scene = Scene { objects: Vec::new() };
scene.add(Circle { radius: 10.0 });
scene.add(Rectangle { width: 20.0, height: 15.0 });
scene.add(Triangle { /* ... */ });
scene.render(&mut canvas);

// Good - trait object in return position
pub fn load_config(path: &str) -> Result<(), Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    // Can return different error types
    Ok(())
}

// Good - trait object for callbacks
pub struct EventHandler {
    handlers: Vec<Box<dyn FnMut(&Event) + Send>>,
}

impl EventHandler {
    pub fn register<F>(&mut self, handler: F)
    where
        F: FnMut(&Event) + Send + 'static,
    {
        self.handlers.push(Box::new(handler));
    }
    
    pub fn trigger(&mut self, event: &Event) {
        for handler in &mut self.handlers {
            handler(event);
        }
    }
}
```

**When to use trait objects**:
- ✅ Need heterogeneous collections
- ✅ Type not known at compile time (plugins, dynamic dispatch)
- ✅ Reduce binary size (less monomorphization)
- ✅ Across API boundaries where generic would leak implementation

**When to use generics instead**:
- ✅ Performance critical (static dispatch)
- ✅ Need generic methods on trait
- ✅ Homogeneous collections
- ✅ Trait has associated types (usually)

**Trait object requirements**:
```rust
// Must be object-safe
// Must specify lifetime if needed
let handler: &dyn Handler;           // Lifetime inferred
let handler: &'static dyn Handler;   // Explicit lifetime
let handler: Box<dyn Handler>;       // Owned

// Common bounds for trait objects
Box<dyn Error>                       // Basic
Box<dyn Error + Send>                // Thread-safe
Box<dyn Error + Send + Sync>         // Fully thread-safe
Box<dyn Error + Send + Sync + 'static>  // Can downcast
```

**Rationale**: Trait objects provide runtime polymorphism when compile-time polymorphism (generics) isn't sufficient.

---

### Provide From Conversions for Related Types

**Strength**: SHOULD

**Summary**: Implement From<T> to enable ergonomic conversions and interoperability.

**Examples**:

```rust
// Good - From for owned conversions
impl From<String> for EmailAddress {
    fn from(s: String) -> Self {
        EmailAddress(s)
    }
}

// Now works with .into() and ?
fn create_user(email: String) -> Result<User, Error> {
    let email_addr: EmailAddress = email.into();
    Ok(User { email: email_addr })
}

// Good - From for copying conversions
impl From<u32> for u64 {
    fn from(small: u32) -> u64 {
        small as u64
    }
}

let big: u64 = 100u32.into();

// Good - From for enums wrapping types
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl From<Ipv4Addr> for IpAddr {
    fn from(addr: Ipv4Addr) -> IpAddr {
        IpAddr::V4(addr)
    }
}

impl From<Ipv6Addr> for IpAddr {
    fn from(addr: Ipv6Addr) -> IpAddr {
        IpAddr::V6(addr)
    }
}

// Usage with ?
fn parse_ip(s: &str) -> Result<IpAddr, ParseError> {
    if s.contains(':') {
        let v6: Ipv6Addr = s.parse()?;
        Ok(v6.into())  // Automatically converts to IpAddr
    } else {
        let v4: Ipv4Addr = s.parse()?;
        Ok(v4.into())
    }
}

// Good - From for error types
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> AppError {
        AppError::Io(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> AppError {
        AppError::Json(err)
    }
}

// Now ? works seamlessly
fn load_data(path: &str) -> Result<Data, AppError> {
    let contents = std::fs::read_to_string(path)?;  // io::Error → AppError
    let data = serde_json::from_str(&contents)?;     // json::Error → AppError
    Ok(data)
}

// Bad - requiring explicit conversion
fn load_data_bad(path: &str) -> Result<Data, AppError> {
    let contents = std::fs::read_to_string(path)
        .map_err(AppError::Io)?;  // Manual conversion
    let data = serde_json::from_str(&contents)
        .map_err(AppError::Json)?;  // Manual conversion
    Ok(data)
}
```

**From vs TryFrom**:
```rust
// Use From for infallible conversions
impl From<u16> for u32 {
    fn from(small: u16) -> u32 {
        small as u32  // Always succeeds
    }
}

// Use TryFrom for fallible conversions
impl TryFrom<u32> for u16 {
    type Error = TryFromIntError;
    
    fn try_from(big: u32) -> Result<u16, Self::Error> {
        if big <= u16::MAX as u32 {
            Ok(big as u16)
        } else {
            Err(/* ... */)
        }
    }
}
```

**AsRef vs From**:
```rust
// AsRef - cheap reference conversion
impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        &self
    }
}

// From - owned conversion
impl From<&str> for String {
    fn from(s: &str) -> String {
        s.to_owned()
    }
}
```

**Rationale**: From enables implicit conversion with `.into()` and the `?` operator, making APIs more ergonomic.

**See also**: C-CONV-TRAITS

---

## Operator Overloading

### Operator Overloads Are Unsurprising

**Strength**: MUST

**Summary**: Only implement operator traits when the operation naturally corresponds to the operator.

**Examples**:

```rust
use std::ops::{Add, Mul, Neg};

// Good - Vector addition
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector2D {
    x: f64,
    y: f64,
}

impl Add for Vector2D {
    type Output = Vector2D;
    
    fn add(self, other: Vector2D) -> Vector2D {
        Vector2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// Good - Scalar multiplication
impl Mul<f64> for Vector2D {
    type Output = Vector2D;
    
    fn mul(self, scalar: f64) -> Vector2D {
        Vector2D {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

// Good - Negation
impl Neg for Vector2D {
    type Output = Vector2D;
    
    fn neg(self) -> Vector2D {
        Vector2D {
            x: -self.x,
            y: -self.y,
        }
    }
}

// Usage is intuitive
let v1 = Vector2D { x: 1.0, y: 2.0 };
let v2 = Vector2D { x: 3.0, y: 4.0 };
let v3 = v1 + v2;          // Vector addition
let v4 = v1 * 2.0;         // Scalar multiplication
let v5 = -v1;              // Negation

// Bad - misleading operator overload
use std::ops::Add;

pub struct Logger {
    messages: Vec<String>,
}

// DON'T DO THIS - + doesn't mean "log"
impl Add<String> for Logger {
    type Output = Logger;
    
    fn add(mut self, message: String) -> Logger {
        self.messages.push(message);
        self
    }
}

// Confusing usage
let logger = Logger::new() + "message".to_string();

// Good - use a clear method instead
impl Logger {
    pub fn log(&mut self, message: String) {
        self.messages.push(message);
    }
}

// Bad - unexpected behavior
impl Add for Configuration {
    type Output = Configuration;
    
    fn add(self, other: Configuration) -> Configuration {
        // Merging configs with + is unexpected
        // Use a merge() method instead
    }
}
```

**Guidelines for operator overloading**:

| Operator | Use for | Avoid for |
|----------|---------|-----------|
| `+` | Addition, concatenation | Logging, merging, unrelated operations |
| `*` | Multiplication, repetition | Unrelated operations |
| `-` | Subtraction, difference | Removal from collections |
| `/` | Division | Path operations (use methods) |
| `%` | Remainder, modulo | Formatting |
| `&` | Bitwise AND, set intersection | Boolean AND (use `&&`) |
| `\|` | Bitwise OR, set union | Boolean OR (use `\|\|`) |
| `^` | Bitwise XOR, symmetric difference | |
| `<<` | Bit shift, stream insertion | |
| `>>` | Bit shift, stream extraction | |

**Properties operators should maintain**:

```rust
// Addition should be commutative (where sensible)
assert_eq!(a + b, b + a);

// Addition should be associative
assert_eq!((a + b) + c, a + (b + c));

// Multiplication should distribute over addition
assert_eq!(a * (b + c), a * b + a * c);

// Identity elements
assert_eq!(a + 0, a);
assert_eq!(a * 1, a);
```

**Rationale**: Operators carry strong semantic expectations. Violating these expectations makes code confusing and error-prone.

**See also**: C-OVERLOAD

---

## Advanced Trait Patterns

### Use Extension Traits for Optional Functionality

**Strength**: CONSIDER

**Summary**: Provide optional functionality through extension traits to avoid tight coupling.

**Examples**:

```rust
// Good - extension trait pattern
pub trait AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>>;
}

// Extension trait for convenience methods
pub trait AsyncReadExt: AsyncRead {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Read<'a, Self>
    where
        Self: Unpin,
    {
        Read { reader: self, buf }
    }
    
    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadExact<'a, Self>
    where
        Self: Unpin,
    {
        ReadExact { reader: self, buf }
    }
}

// Blanket implementation
impl<T: AsyncRead + ?Sized> AsyncReadExt for T {}

// Users get extension methods automatically
async fn example<R: AsyncRead + Unpin>(reader: &mut R) {
    let mut buf = [0u8; 1024];
    reader.read(&mut buf).await?;  // From AsyncReadExt
}

// Good - serde extension pattern
#[cfg(feature = "serde")]
pub trait SerializeExt: Serialize {
    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> SerializeExt for T {}

// Good - iterator extension pattern
pub trait IteratorExt: Iterator {
    fn collect_vec(self) -> Vec<Self::Item>
    where
        Self: Sized,
    {
        self.collect()
    }
    
    fn join(mut self, sep: &str) -> String
    where
        Self: Sized,
        Self::Item: std::fmt::Display,
    {
        let mut result = String::new();
        if let Some(first) = self.next() {
            result.push_str(&first.to_string());
            for item in self {
                result.push_str(sep);
                result.push_str(&item.to_string());
            }
        }
        result
    }
}

impl<T: Iterator> IteratorExt for T {}
```

**Benefits**:
1. Separates core trait from convenience methods
2. Enables feature-gated functionality
3. Allows users to opt-in to additional dependencies
4. Maintains clean core trait

**Rationale**: Extension traits provide optional ergonomics without coupling core functionality to convenience methods.
