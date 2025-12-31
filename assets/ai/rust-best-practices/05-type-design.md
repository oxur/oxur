# Type Design Guidelines

Patterns for designing types, including newtypes, enums, and strong typing strategies.


## TD-01: Arguments Use Types Not Bool or Option

**Strength**: SHOULD

**Summary**: Use custom types instead of `bool` or `Option` to convey meaning clearly.

```rust
// Bad - unclear bool arguments
let widget = Widget::new(true, false);
// What do true and false mean? Must check documentation

// Good - explicit enum types
pub enum Size {
    Small,
    Medium,
    Large,
}

pub enum Shape {
    Round,
    Square,
}

let widget = Widget::new(Size::Small, Shape::Round);
// Clear intent without checking docs

// Bad - Option<bool> is confusing
fn configure(enable_caching: Option<bool>) {
    match enable_caching {
        Some(true) => { /* ... */ }
        Some(false) => { /* ... */ }
        None => { /* ... */ }  // What does None mean?
    }
}

// Good - explicit three-state enum
pub enum CacheMode {
    Enabled,
    Disabled,
    Default,
}

fn configure(cache_mode: CacheMode) {
    match cache_mode {
        CacheMode::Enabled => { /* ... */ }
        CacheMode::Disabled => { /* ... */ }
        CacheMode::Default => { /* ... */ }
    }
}

// Bad - multiple bools
fn create_user(
    admin: bool,
    verified: bool,
    send_email: bool,
    premium: bool,
) -> User {
    // Hard to understand calls:
    // create_user(true, false, true, false)
}

// Good - struct or builder pattern
pub struct UserFlags {
    pub admin: bool,
    pub verified: bool,
    pub send_email: bool,
    pub premium: bool,
}

fn create_user(flags: UserFlags) -> User {
    // Clear at call site:
    // create_user(UserFlags {
    //     admin: true,
    //     verified: false,
    //     send_email: true,
    //     premium: false,
    // })
}

// Even better - bitflags for flags
use bitflags::bitflags;

bitflags! {
    pub struct UserPermissions: u32 {
        const ADMIN = 0b00000001;
        const VERIFIED = 0b00000010;
        const PREMIUM = 0b00000100;
    }
}

fn create_user(permissions: UserPermissions) -> User {
    // create_user(UserPermissions::ADMIN | UserPermissions::PREMIUM)
}
```

```rust
// OK - meaning is crystal clear
fn set_visible(visible: bool) { }
fn is_empty() -> bool { }
```

**Rationale**: Explicit types make code self-documenting and prevent passing arguments in the wrong order.

**See also**: C-CUSTOM-TYPE

---

## TD-02: Associated Types vs Generic Parameters

**Strength**: SHOULD

**Summary**: Use associated types when there's one logical type; generics when multiple make sense.

```rust
// ✅ ASSOCIATED TYPE: One logical output per implementor
trait Iterator {
    type Item;  // Each iterator has ONE item type
    fn next(&mut self) -> Option<Self::Item>;
}

struct Counter { value: u32 }

impl Iterator for Counter {
    type Item = u32;  // Counter always yields u32
    fn next(&mut self) -> Option<u32> { 
        self.value += 1;
        Some(self.value)
    }
}

// ✅ GENERIC PARAMETER: Multiple valid implementations per type
trait From<T> {
    fn from(value: T) -> Self;
}

// String can be From<&str>, From<char>, etc.
// impl From<&str> for String { /* ... */ }
// impl From<char> for String { /* ... */ }

// Decision guide:
// - "A type has ONE X" → Associated type
// - "A type can be X for many T" → Generic parameter
```

---

## TD-03: Builder Pattern for Complex Construction

**Strength**: SHOULD

**Summary**: Use a builder when construction has many optional parameters. Use the builder pattern when types have many optional parameters or complex construction requirements.

```rust
// ❌ UNWIELDY: Many constructor parameters
impl Server {
    pub fn new(
        host: String,
        port: u16,
        max_connections: Option<usize>,
        timeout: Option<Duration>,
        tls_config: Option<TlsConfig>,
    ) -> Self { todo!() }
}

// ✅ GOOD: Builder pattern
pub struct ServerBuilder {
    host: String,
    port: u16,
    max_connections: usize,
    timeout: Duration,
    tls_config: Option<TlsConfig>,
}

impl ServerBuilder {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            max_connections: 100,      // sensible default
            timeout: Duration::from_secs(30),
            tls_config: None,
        }
    }
    
    pub fn max_connections(mut self, n: usize) -> Self {
        self.max_connections = n;
        self
    }
    
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = d;
        self
    }
    
    pub fn tls(mut self, config: TlsConfig) -> Self {
        self.tls_config = Some(config);
        self
    }
    
    pub fn build(self) -> Result<Server, BuildError> {
        // Validate and construct
        Ok(Server {
            host: self.host,
            port: self.port,
            max_connections: self.max_connections,
            timeout: self.timeout,
            tls_config: self.tls_config,
        })
    }
}

// Usage:
let server = ServerBuilder::new("localhost", 8080)
    .max_connections(1000)
    .timeout(Duration::from_secs(60))
    .build()?;
```

```rust
// Good - builder for complex configuration
pub struct HttpClient {
    timeout: Duration,
    user_agent: String,
    max_redirects: u32,
    proxy: Option<String>,
    cookies: bool,
}

pub struct HttpClientBuilder {
    timeout: Duration,
    user_agent: String,
    max_redirects: u32,
    proxy: Option<String>,
    cookies: bool,
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        HttpClientBuilder {
            timeout: Duration::from_secs(30),
            user_agent: "rust-client/1.0".to_string(),
            max_redirects: 10,
            proxy: None,
            cookies: true,
        }
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }
    
    pub fn max_redirects(mut self, max_redirects: u32) -> Self {
        self.max_redirects = max_redirects;
        self
    }
    
    pub fn proxy(mut self, proxy: String) -> Self {
        self.proxy = Some(proxy);
        self
    }
    
    pub fn enable_cookies(mut self, enable: bool) -> Self {
        self.cookies = enable;
        self
    }
    
    pub fn build(self) -> HttpClient {
        HttpClient {
            timeout: self.timeout,
            user_agent: self.user_agent,
            max_redirects: self.max_redirects,
            proxy: self.proxy,
            cookies: self.cookies,
        }
    }
}

// Usage - one-liner
let client = HttpClientBuilder::new()
    .timeout(Duration::from_secs(60))
    .user_agent("my-app/2.0".to_string())
    .build();

// Usage - complex configuration
let mut builder = HttpClientBuilder::new();
if let Ok(proxy) = std::env::var("HTTP_PROXY") {
    builder = builder.proxy(proxy);
}
if debug_mode {
    builder = builder.timeout(Duration::from_secs(300));
}
let client = builder.build();

// Good - validating builder
impl HttpClientBuilder {
    pub fn build(self) -> Result<HttpClient, BuildError> {
        if self.timeout.as_secs() == 0 {
            return Err(BuildError::InvalidTimeout);
        }
        
        if self.max_redirects > 100 {
            return Err(BuildError::TooManyRedirects);
        }
        
        Ok(HttpClient {
            timeout: self.timeout,
            user_agent: self.user_agent,
            max_redirects: self.max_redirects,
            proxy: self.proxy,
            cookies: self.cookies,
        })
    }
}

// Good - consuming builder for side effects
pub struct Command {
    program: String,
    args: Vec<String>,
}

impl Command {
    pub fn new(program: String) -> Self {
        Command {
            program,
            args: Vec::new(),
        }
    }
    
    pub fn arg(mut self, arg: String) -> Self {
        self.args.push(arg);
        self
    }
    
    // Terminal method that consumes self
    pub fn spawn(self) -> io::Result<Child> {
        // Actually spawn the process
        // Can't use self after this
    }
}
```

```rust
// Non-consuming builder (preferred when possible)
pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
    self.timeout = timeout;
    self
}

// Consuming builder (necessary when building transfers ownership)
pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
}
```

**Rationale**: Builders make complex construction ergonomic, support optional parameters, and enable validation before construction.

**See also**: C-BUILDER

---

## TD-04: Eagerly Implement Common Traits

**Strength**: SHOULD

**Summary**: Public types should implement standard traits when semantically appropriate.

```rust
// Implement all applicable traits
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(u64);

#[derive(Debug, Clone, PartialEq)]
pub struct UserData {
    name: String,
    email: String,
}

// Implement Default when there's a sensible default
#[derive(Debug, Clone, Default)]
pub struct Config {
    timeout: Option<Duration>,
    retry_count: Option<u32>,
}

// Manual implementation when needed
impl PartialOrd for Temperature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Temperature {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
```

**Rationale**: Implementing standard traits makes types composable and usable in generic contexts. Users expect common types to work with standard library collections and algorithms.

**See also**: C-COMMON-TRAITS

---

## TD-05: Encapsulation with Private Fields

**Strength**: SHOULD

**Summary**: Keep fields private; expose via methods to maintain invariants.

```rust
// ❌ BAD: Public fields allow invalid state
pub struct EmailAddress {
    pub value: String,  // Anyone can set this to "not-an-email"
}

// ✅ GOOD: Private field with validated constructor
pub struct EmailAddress {
    value: String,  // private
}

#[derive(Debug)]
pub struct InvalidEmail;

impl EmailAddress {
    pub fn new(value: &str) -> Result<Self, InvalidEmail> {
        if value.contains('@') && value.len() > 3 {
            Ok(Self { value: value.to_string() })
        } else {
            Err(InvalidEmail)
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

// Now EmailAddress is ALWAYS valid
fn send_email(to: &EmailAddress) {
    // No need to validate - the type guarantees validity
    println!("Sending to {}", to.as_str());
}
```

---

## TD-06: Generic Type Parameters

**Strength**: SHOULD

**Summary**: Use generics when behavior is identical across types.

```rust
// ✅ GOOD: Generic when behavior is the same
fn first<T>(slice: &[T]) -> Option<&T> {
    slice.first()
}

// ✅ GOOD: Bounded generics for required behavior
fn largest<T: Ord>(list: &[T]) -> Option<&T> {
    list.iter().max()
}

// ✅ GOOD: Multiple bounds
use std::fmt::{Debug, Display};

fn debug_print<T: Debug + Display>(value: &T) {
    println!("Debug: {:?}", value);
    println!("Display: {}", value);
}

// ❌ OVER-GENERIC: When you only use one type
fn process_string<T: AsRef<str>>(s: T) {
    let _ = s.as_ref();
    // If you only ever pass &str or String, just use &str
}

// ✅ SIMPLER: Concrete when appropriate
fn process_string(s: &str) {
    let _ = s;
}
```

---

## TD-07: Generics Minimize Assumptions

**Strength**: SHOULD

**Summary**: Use generic parameters to minimize assumptions about types, making functions more reusable.

```rust
// Good - accepts any iterator
fn process_items<I>(items: I) 
where
    I: IntoIterator<Item = i64>
{
    for item in items {
        println!("{}", item);
    }
}

// Can be called with:
process_items(vec![1, 2, 3]);
process_items(&[1, 2, 3]);
process_items(0..10);
process_items(some_hashset);

// Bad - too specific
fn process_items_bad(items: Vec<i64>) {
    for item in items {
        println!("{}", item);
    }
}
// Only works with Vec, not other collections

// Good - generic over string types
fn log_message<S: AsRef<str>>(message: S) {
    println!("{}", message.as_ref());
}

log_message("hello");                    // &str
log_message(String::from("hello"));      // String
log_message(&String::from("hello"));     // &String

// Bad - requires specific type
fn log_message_bad(message: String) {
    println!("{}", message);
}
// Requires String, forcing allocation for string literals

// Good - generic reader/writer
use std::io::{Read, Write};

fn copy_data<R, W>(reader: &mut R, writer: &mut W) -> io::Result<u64>
where
    R: Read,
    W: Write,
{
    let mut buffer = [0u8; 8192];
    let mut total = 0;
    
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buffer[..n])?;
        total += n as u64;
    }
    
    Ok(total)
}

// Works with files, network sockets, in-memory buffers, etc.
```

**Rationale**: Generics provide maximum flexibility and performance through static dispatch and monomorphization.

**See also**: C-GENERIC

---

## TD-08: Newtype Best Practices

**Strength**: SHOULD

**Summary**: Newtypes should provide accessor methods and implement common traits.

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Temperature(i32);

impl Temperature {
    /// Creates a new temperature in Celsius
    pub fn celsius(value: i32) -> Self {
        Self(value)
    }
    
    /// Creates a temperature from Fahrenheit
    pub fn fahrenheit(value: i32) -> Self {
        Self((value - 32) * 5 / 9)
    }
    
    /// Gets the temperature in Celsius
    pub fn as_celsius(&self) -> i32 {
        self.0
    }
    
    /// Gets the temperature in Fahrenheit
    pub fn as_fahrenheit(&self) -> i32 {
        self.0 * 9 / 5 + 32
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°C", self.0)
    }
}

// Good trait implementations
impl Default for Temperature {
    fn default() -> Self {
        Self::celsius(20)  // Room temperature
    }
}

// Allow conversion from inner type when unambiguous
impl From<i32> for Temperature {
    fn from(celsius: i32) -> Self {
        Self::celsius(celsius)
    }
}

// Usage
let temp = Temperature::celsius(25);
println!("{}", temp);  // "25°C"

let temp2 = Temperature::fahrenheit(77);
assert_eq!(temp, temp2);

let temp3: Temperature = 25.into();
```

**Rationale**: Good accessor methods and trait implementations make newtypes ergonomic to use while maintaining encapsulation.

**See also**: C-COMMON-TRAITS

---

## TD-09: Newtypes Hide Implementation Details

**Strength**: SHOULD

**Summary**: Use newtypes to hide complex implementation types from public API.

```rust
use std::iter::{Enumerate, Skip};

// Bad - exposing complex iterator type
pub fn my_transform<I: Iterator>(input: I) -> Enumerate<Skip<I>> {
    input.skip(3).enumerate()
}
// Users see: Enumerate<Skip<SomeIterator>>
// Can't change implementation without breaking API

// Good - newtype hides implementation
pub struct MyTransformResult<I>(Enumerate<Skip<I>>);

impl<I: Iterator> Iterator for MyTransformResult<I> {
    type Item = (usize, I::Item);
    
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub fn my_transform<I: Iterator>(input: I) -> MyTransformResult<I> {
    MyTransformResult(input.skip(3).enumerate())
}
// Users see: MyTransformResult<SomeIterator>
// Can change from Enumerate<Skip<I>> to something else

// Modern alternative - impl Trait
pub fn my_transform<I: Iterator>(input: I) -> impl Iterator<Item = (usize, I::Item)> {
    input.skip(3).enumerate()
}
// Even more opaque, but can't name the type

// Good - newtype for complex hash map value
use std::collections::HashMap;

pub struct Cache<K, V> {
    inner: HashMap<K, Vec<(V, Instant)>>,
}

impl<K, V> Cache<K, V> 
where
    K: Eq + Hash,
{
    pub fn insert(&mut self, key: K, value: V) {
        self.inner
            .entry(key)
            .or_insert_with(Vec::new)
            .push((value, Instant::now()));
    }
    
    // Can change from Vec to VecDeque or custom structure
}

// Bad - exposing HashMap directly
pub struct Cache<K, V> {
    pub inner: HashMap<K, Vec<(V, Instant)>>,
}
// Now locked into this exact structure
```

**Rationale**: Hiding implementation details allows you to optimize or refactor without breaking API compatibility.

**See also**: C-NEWTYPE-HIDE

---

## TD-10: Newtypes Provide Static Distinctions

**Strength**: SHOULD

**Summary**: Use newtype pattern to create distinct types that prevent mixing incompatible values.

```rust
// Good - newtypes distinguish units
pub struct Miles(pub f64);
pub struct Kilometers(pub f64);

impl Miles {
    pub fn to_kilometers(self) -> Kilometers {
        Kilometers(self.0 * 1.60934)
    }
}

impl Kilometers {
    pub fn to_miles(self) -> Miles {
        Miles(self.0 / 1.60934)
    }
}

// Type system prevents mixing units
fn distance_to_destination(miles: Miles) -> bool {
    miles.0 > 100.0
}

let km = Kilometers(200.0);
// distance_to_destination(km);  // Compile error!
distance_to_destination(km.to_miles());  // Must convert

// Bad - using raw types
fn distance_to_destination_bad(miles: f64) -> bool {
    miles > 100.0
}

// Easy to accidentally pass kilometers
let km = 200.0;
distance_to_destination_bad(km);  // Compiles, but wrong!

// Good - newtypes for IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

// Can't accidentally use wrong ID type
fn get_user(id: UserId) -> Option<User> { /* ... */ }
fn get_order(id: OrderId) -> Option<Order> { /* ... */ }

let user_id = UserId(123);
let order_id = OrderId(456);

// get_user(order_id);  // Compile error!
get_user(user_id);  // OK

// Good - newtypes for validated data
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: String) -> Result<Self, ValidationError> {
        if s.is_empty() {
            Err(ValidationError::Empty)
        } else {
            Ok(NonEmptyString(s))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Once constructed, you know it's valid
fn process_name(name: NonEmptyString) {
    // No need to check if empty - type system guarantees it
    println!("Processing: {}", name.as_str());
}
```

**Rationale**: Newtypes use the type system to prevent bugs at compile time rather than runtime, with zero performance cost.

**See also**: C-NEWTYPE

---

## TD-11: Phantom Data for Unused Type Parameters

**Strength**: MUST

**Summary**: Use `PhantomData` when a type parameter isn't used in fields.

```rust
use std::marker::PhantomData;

// Without PhantomData, this errors: "parameter T is never used"
struct Identifier<T> {
    id: u64,
    _marker: PhantomData<T>,  // "Uses" T without storing it
}

// Now different identifiers are different types:
struct User;
struct Order;

type UserId = Identifier<User>;
type OrderId = Identifier<Order>;

fn process_user(id: UserId) { 
    println!("Processing user {}", id.id);
}

fn example() {
    let user_id: UserId = Identifier { id: 1, _marker: PhantomData };
    let order_id: OrderId = Identifier { id: 1, _marker: PhantomData };

    process_user(user_id);   // OK
    // process_user(order_id);  // ERROR: expected UserId, found OrderId
}
```

---

## TD-12: Prefer Structs Over Tuples for Named Data

**Strength**: SHOULD

**Summary**: Use structs when the meaning of fields isn't obvious from context.

```rust
// ❌ UNCLEAR: What do these numbers mean?
fn parse_point(s: &str) -> Option<(f64, f64, f64)> {
    todo!()
}

let point = parse_point("1,2,3")?;
println!("x={}, y={}, z={}", point.0, point.1, point.2);

// ✅ CLEAR: Named fields are self-documenting
#[derive(Debug, Clone, Copy)]
struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

fn parse_point(s: &str) -> Option<Point3D> {
    todo!()
}

let point = parse_point("1,2,3")?;
println!("x={}, y={}, z={}", point.x, point.y, point.z);

// ✅ TUPLES ARE OK: When meaning is obvious
fn min_max(values: &[i32]) -> Option<(i32, i32)> {  // Clearly (min, max)
    Some((*values.iter().min()?, *values.iter().max()?))
}
```

---

## TD-13: Public Types Implement Debug

**Strength**: MUST

**Summary**: All public types must implement `Debug`; types with sensitive data must use custom implementations.

```rust
use std::fmt::{self, Debug, Formatter};

// Good - simple derived Debug
#[derive(Debug)]
pub struct Endpoint {
    url: String,
    timeout: Duration,
}

// Good - custom Debug for sensitive data
pub struct UserCredentials {
    username: String,
    password: String,
    api_key: String,
}

impl Debug for UserCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserCredentials")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("api_key", &"<redacted>")
            .finish()
    }
}

// Test that sensitive data doesn't leak
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_credentials_debug_redacts_secrets() {
        let creds = UserCredentials {
            username: "alice".to_string(),
            password: "super_secret_123".to_string(),
            api_key: "sk-1234567890abcdef".to_string(),
        };
        
        let debug_output = format!("{:?}", creds);
        
        // Verify sensitive data is not present
        assert!(!debug_output.contains("super_secret"));
        assert!(!debug_output.contains("sk-1234"));
        assert!(debug_output.contains("redacted"));
        assert!(debug_output.contains("alice"));  // Username is OK
    }
}
```

**Rationale**: `Debug` is essential for development and logging. Custom implementations prevent accidental leakage of secrets in debug output, logs, or error messages.

**See also**: M-PUBLIC-DEBUG

---

## TD-14: Readable Types Implement Display

**Strength**: MUST

**Summary**: Types intended to be read by users must implement `Display` following Rust conventions.

```rust
use std::fmt::{self, Display, Formatter};

// Good - Display for user-facing type
pub struct UserId(u64);

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

// Good - Display for error type
pub struct ValidationError {
    field: String,
    message: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Validation failed for '{}': {}", self.field, self.message)
    }
}

// Display handles newlines and escape sequences
pub struct MultilineMessage {
    lines: Vec<String>,
}

impl Display for MultilineMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

// Usage
let user_id = UserId(42);
println!("User: {}", user_id);  // "User: user:42"

let error = ValidationError {
    field: "email".to_string(),
    message: "Invalid email format".to_string(),
};
eprintln!("{}", error);  // User-friendly error message
```

**Rationale**: `Display` is for user-facing output. It should produce clean, readable text. Error types must implement it (required by `std::error::Error`).

**See also**: M-PUBLIC-DISPLAY

---

## TD-15: Sealed Traits

**Strength**: CONSIDER

**Summary**: Prevent external implementations of a trait.

```rust
// In your library:
mod private {
    pub trait Sealed {}
}

pub trait MyTrait: private::Sealed {
    fn method(&self);
}

pub struct MyType;

// Only types in your crate can implement Sealed
impl private::Sealed for MyType {}
impl MyTrait for MyType {
    fn method(&self) { 
        println!("MyType::method");
    }
}

// External crates can USE MyTrait but can't IMPLEMENT it
// This lets you add methods to MyTrait without breaking changes
```

---

## TD-16: Structs Don't Duplicate Derived Trait Bounds

**Strength**: MUST

**Summary**: Don't add trait bounds to struct definitions that are only needed by derived implementations.

```rust
// Good - no redundant bounds
#[derive(Clone, Debug, PartialEq)]
pub struct Container<T> {
    items: Vec<T>,
}

// The derive automatically generates:
// impl<T: Clone> Clone for Container<T> { }
// impl<T: Debug> Debug for Container<T> { }
// impl<T: PartialEq> PartialEq for Container<T> { }

// Bad - redundant bounds
#[derive(Clone, Debug, PartialEq)]
pub struct Container<T: Clone + Debug + PartialEq> {
    items: Vec<T>,
}
// Problems:
// 1. Redundant - derive already adds these bounds to impls
// 2. Breaking change to add more derives
// 3. Prevents using Container with non-Clone T when you don't need clone

// Example of the problem:
// Good version allows this:
let container: Container<Rc<String>> = Container { items: vec![] };
// Can use Container with Rc even though Rc is only Clone, not Copy

// Bad version forces Clone even when not needed:
struct Data<T: Clone> { inner: T }
fn store_data<T>(data: T) -> Data<T> { }  // Error: T doesn't implement Clone
// Even though we never clone T in this function!

// Good - bounds only on impl blocks where needed
pub struct Container<T> {
    items: Vec<T>,
}

impl<T: Clone> Container<T> {
    pub fn duplicate_items(&self) -> Container<T> {
        Container {
            items: self.items.clone(),
        }
    }
}

impl<T> Container<T> {
    pub fn len(&self) -> usize {
        self.items.len()
    }
    // No Clone bound needed here
}

// Exceptions where bounds ARE needed:
pub struct Sorted<T: Ord> {
    items: Vec<T>,
}
// Ord bound is required by the type's invariant (items are always sorted)

impl<T: Ord> Sorted<T> {
    pub fn insert(&mut self, item: T) {
        // Maintains sorted order
        let pos = self.items.binary_search(&item).unwrap_or_else(|e| e);
        self.items.insert(pos, item);
    }
}
```

**Rationale**: Unnecessary bounds are a backward compatibility hazard and limit type usage unnecessarily.

**See also**: C-STRUCT-BOUNDS

---

## TD-17: Structs Have Private Fields

**Strength**: MUST

**Summary**: Make struct fields private by default; provide accessor methods for controlled access.

```rust
// Good - private fields, public API
pub struct User {
    id: UserId,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: UserId, name: String, email: String) -> Self {
        User {
            id,
            name,
            email,
            created_at: Utc::now(),
        }
    }
    
    // Controlled access
    pub fn id(&self) -> UserId {
        self.id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    // Can add validation
    pub fn set_name(&mut self, name: String) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        self.name = name;
        Ok(())
    }
}

// Bad - public fields
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
}
// Problems:
// - Can't add validation later
// - Can't change internal representation
// - Can't maintain invariants
// - Breaking change to make fields private

// Exception - C-like structs (passive data)
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
// OK because it's just passive coordinate data

// Exception - builder/config structs
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}
// OK because it's configuration data without invariants
```

**Rationale**: Private fields enable evolution of the type without breaking changes and allow maintaining invariants.

**See also**: C-STRUCT-PRIVATE

---

## TD-18: The Typestate Pattern

**Strength**: CONSIDER

**Summary**: Encode state in the type system to make invalid operations impossible.

```rust
use std::marker::PhantomData;

// Type-level state markers (zero-sized types)
struct Unlocked;
struct Locked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        println!("Locking door");
        Door { _state: PhantomData }
    }
    
    fn open(&self) {
        println!("Opening door");
    }
}

impl Door<Locked> {
    fn unlock(self) -> Door<Unlocked> {
        println!("Unlocking door");
        Door { _state: PhantomData }
    }
    // Note: No open() method - can't open a locked door!
}

fn new_door() -> Door<Unlocked> {
    Door { _state: PhantomData }
}

// Usage:
fn example() {
    let door = new_door();
    door.open();           // OK
    let door = door.lock();
    // door.open();        // ERROR: Door<Locked> has no method `open`
    let door = door.unlock();
    door.open();           // OK again
}
```

**Rationale**: Compile-time enforcement of valid state transitions. No runtime cost.

---

---

## TD-19: Traits Are Object-Safe When Useful

**Strength**: SHOULD

**Summary**: If a trait might reasonably be used as a trait object, design it to be object-safe.

```rust
// Good - object-safe trait
pub trait Draw {
    fn draw(&self, canvas: &mut Canvas);
    fn bounds(&self) -> Rect;
}

// Can be used as trait object
let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle { radius: 10.0 }),
    Box::new(Rectangle { width: 20.0, height: 15.0 }),
];

for shape in &shapes {
    shape.draw(&mut canvas);
}

// Bad - not object-safe (generic method)
pub trait Process {
    fn process<T: Serialize>(&self, data: T);
}

// Cannot use as trait object:
// let processor: Box<dyn Process> = ...;  // Error!

// Good - make generic method require Self: Sized
pub trait Iterator {
    type Item;
    
    fn next(&mut self) -> Option<Self::Item>;
    
    // Object-safe methods work
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
    
    // Generic methods excluded from trait object
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,  // Excludes from trait object
        F: FnMut(Self::Item) -> B,
    {
        // ...
    }
}

// Now Iterator can be used as trait object for basic methods
let iter: &mut dyn Iterator<Item = i32> = &mut vec![1, 2, 3].into_iter();
while let Some(x) = iter.next() {
    println!("{}", x);
}

// But generic methods like map() still work on concrete types
vec![1, 2, 3].into_iter().map(|x| x * 2);

// Good - object-safe alternative to generic
pub trait Visitor {
    fn visit_i32(&mut self, value: i32);
    fn visit_string(&mut self, value: &str);
    fn visit_bool(&mut self, value: bool);
}

// Can use as trait object
let visitor: &mut dyn Visitor = &mut MyVisitor::new();
visitor.visit_i32(42);
```

```rust
// Not object-safe
trait Bad {
    fn generic<T>(&self, x: T);
    fn by_value(self);
    fn returns_self(&self) -> Self;
}

// Object-safe version
trait Good {
    fn generic<T>(&self, x: T) where Self: Sized;
    fn by_value(self) where Self: Sized;
    fn returns_self(&self) -> Self where Self: Sized;
}
```

**Rationale**: Object-safety allows runtime polymorphism when needed while still supporting generic methods on concrete types.

**See also**: C-OBJECT

---

## TD-20: Type Design Checklist

**Strength**: SHOULD

**Summary**: 

```rust
// 1. Should this be a newtype?
pub struct UserId(u64);  // vs u64

// 2. What traits make sense?
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

// 3. Does it need validation?
impl UserId {
    pub fn new(id: u64) -> Result<Self, ValidationError> {
        // Validate...
    }
}

// 4. Should it have a Display?
impl Display for UserId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

// 5. What accessors are needed?
impl UserId {
    pub fn get(&self) -> u64 { self.0 }
}

// 6. Are there useful constructors?
impl UserId {
    pub fn from_string(s: &str) -> Result<Self, ParseError> {
        // Parse...
    }
}

// 7. Should it be Copy or Clone?
// Clone if it owns data, Copy if trivial

// 8. Does it need Default?
impl Default for Config {
    fn default() -> Self {
        // Sensible defaults
    }
}
```

---

## TD-21: Use `#[non_exhaustive]` for Future Compatibility

**Strength**: SHOULD

**Summary**: Allow adding variants/fields without breaking changes.

```rust
// In your library:
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum DatabaseError {
    ConnectionFailed,
    QueryFailed,
    Timeout,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows: Vec<Row>,
    pub affected: usize,
}

// Users must handle unknown variants:
match error {
    DatabaseError::ConnectionFailed => { /* ... */ }
    DatabaseError::QueryFailed => { /* ... */ }
    DatabaseError::Timeout => { /* ... */ }
    _ => { /* Handle future variants */ }  // Required!
}

// Users can't construct the struct directly:
// let result = QueryResult { rows, affected };  // ERROR outside crate
// Must use constructor you provide
```

**Rationale**: Without `#[non_exhaustive]`, adding a new enum variant or struct field is a breaking change requiring a major version bump.

---

---

## TD-22: Use bitflags for Flag Sets

**Strength**: SHOULD

**Summary**: For a set of boolean flags, use the `bitflags` crate instead of enums or multiple bools.

```rust
use bitflags::bitflags;

// Good - bitflags for file permissions
bitflags! {
    pub struct Permissions: u32 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
        const EXECUTE = 0b00000100;
        const DELETE = 0b00001000;
    }
}

fn set_permissions(perms: Permissions) {
    if perms.contains(Permissions::READ) {
        println!("Can read");
    }
    if perms.contains(Permissions::WRITE) {
        println!("Can write");
    }
}

// Usage - combine flags with |
let perms = Permissions::READ | Permissions::WRITE;
set_permissions(perms);

// Bad - using enum for flags
pub enum Permission {
    Read,
    Write,
    Execute,
}

fn set_permissions_bad(perms: Vec<Permission>) {
    // Awkward: must use Vec or similar
    // Can't easily combine flags
}

// Good - bitflags with formatting
impl fmt::Binary for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.bits(), f)
    }
}

let perms = Permissions::READ | Permissions::EXECUTE;
println!("{:b}", perms);  // Prints: 101

// Good - checking for specific combinations
let perms = Permissions::READ | Permissions::WRITE | Permissions::EXECUTE;

// Check for any
if perms.intersects(Permissions::READ | Permissions::WRITE) {
    println!("Has read or write");
}

// Check for all
if perms.contains(Permissions::READ | Permissions::WRITE) {
    println!("Has both read and write");
}

// Remove flags
let read_only = perms & !Permissions::WRITE;
```

**Rationale**: Bitflags provide efficient storage, intuitive combination syntax, and are a well-established pattern.

**See also**: C-BITFLAG

---

## TD-23: Use Enums for State Machines

**Strength**: SHOULD

**Summary**: Model mutually exclusive states as enum variants.

```rust
// ❌ BAD: Boolean flags for state
struct Connection {
    is_connected: bool,
    is_authenticated: bool,
    socket: Option<TcpStream>,
    user: Option<User>,
}

// Can represent invalid states:
// is_authenticated = true, is_connected = false (??)

// ✅ GOOD: Enum makes invalid states unrepresentable
enum Connection {
    Disconnected,
    Connected {
        socket: TcpStream,
    },
    Authenticated {
        socket: TcpStream,
        user: User,
    },
}

impl Connection {
    fn authenticate(&mut self, credentials: &Credentials) -> Result<(), AuthError> {
        // Take ownership of self to transform state
        let old = std::mem::replace(self, Connection::Disconnected);
        
        match old {
            Connection::Connected { socket } => {
                let user = perform_auth(&socket, credentials)?;
                *self = Connection::Authenticated { socket, user };
                Ok(())
            }
            Connection::Disconnected => {
                *self = old;
                Err(AuthError::NotConnected)
            }
            Connection::Authenticated { socket, user } => {
                *self = Connection::Authenticated { socket, user };
                Err(AuthError::AlreadyAuthenticated)
            }
        }
    }
}
```

**Rationale**: Enums ensure exactly one state is active. The compiler enforces handling all states. Invalid state combinations are impossible to construct.

---

---

## TD-24: Use Newtypes for Type Safety

**Strength**: SHOULD

**Summary**: Wrap primitive types to prevent mixing up semantically different values.

```rust
// ❌ DANGEROUS: Easy to mix up arguments
fn transfer(from: u64, to: u64, amount: u64) {
    // Which is which?
}

transfer(123, 456, 100);  // Did we get the order right?

// ✅ SAFE: Newtype wrappers prevent mistakes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Amount(u64);

fn transfer(from: UserId, to: UserId, amount: Amount) {
    // Types enforce correct usage
}

// Compiler catches mistakes:
// transfer(UserId(123), Amount(100), UserId(456));  // ERROR: wrong types!

// ✅ GOOD: Implement only the operations that make sense
impl Amount {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
    
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

// Note: Don't implement Add<UserId> for Amount - that doesn't make sense!
```

**Rationale**: Newtypes are zero-cost (same representation as inner type) but provide compile-time type checking. They also let you impl traits differently for the wrapper.

---

---

## TD-25: Use Strong Types

**Strength**: SHOULD

**Summary**: Use the most specific standard library type for your domain; create newtypes when std types aren't sufficient.

```rust
use std::path::{Path, PathBuf};
use std::time::Duration;

// Bad - primitive obsession
pub struct Config {
    timeout_seconds: u64,
    max_retries: u64,
    config_file: String,  // Should be PathBuf
    user_id: u64,
    api_key: String,
}

pub fn create_user(id: u64, timeout: u64) -> User {
    // Easy to swap parameters!
}

// Good - strong types
pub struct Config {
    timeout: Duration,
    max_retries: RetryCount,
    config_file: PathBuf,
    user_id: UserId,
    api_key: ApiKey,
}

pub struct RetryCount(u32);
pub struct UserId(u64);
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(key: String) -> Result<Self, ValidationError> {
        if key.len() < 20 {
            return Err(ValidationError::InvalidApiKey);
        }
        Ok(Self(key))
    }
}

pub fn create_user(id: UserId, timeout: Duration) -> User {
    // Can't swap parameters - compile error!
}

// Usage
let config = Config {
    timeout: Duration::from_secs(30),
    max_retries: RetryCount(3),
    config_file: PathBuf::from("app.toml"),
    user_id: UserId(42),
    api_key: ApiKey::new("sk-very-long-key".to_string())?,
};
```

**Rationale**: Strong types prevent bugs through type safety. Parameters can't be swapped, validation can be enforced at construction, and intent is clearer.

**See also**: M-STRONG-TYPES, C-NEWTYPE

---

## TD-26: When to Use Newtypes

**Strength**: SHOULD

**Summary**: Create newtypes to add semantics, enforce invariants, or prevent parameter confusion.

```rust
// 1. Add semantics
pub struct Meters(f64);
pub struct Seconds(f64);

pub fn calculate_speed(distance: Meters, time: Seconds) -> f64 {
    distance.0 / time.0
}

// Can't accidentally pass (time, distance)

// 2. Enforce invariants
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: String) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Some(Self(s))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// 3. Prevent parameter confusion in builders
pub struct Account {
    bank: Bank,
    customer: Customer,
}

pub struct Bank(String);
pub struct Customer(String);

impl Account {
    // Clear which is which
    pub fn new(bank: Bank, customer: Customer) -> Self {
        Self { bank, customer }
    }
}

// 4. Hide implementation details
pub struct RequestId(Uuid);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    // Don't expose Uuid directly
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// 5. Add trait implementations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(u64);

impl UserId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn get(&self) -> u64 {
        self.0
    }
}

// Can use UserId as HashMap key
let mut users: HashMap<UserId, User> = HashMap::new();
```

**Rationale**: Newtypes are zero-cost abstractions that add type safety. They prevent common bugs without runtime overhead.

**See also**: C-NEWTYPE

---

## TD-27: Zero-Sized Types (ZSTs)

**Strength**: CONSIDER

**Summary**: Types with no data can be useful for type-level programming.

```rust
use std::marker::PhantomData;

// ZSTs take no space at runtime
struct Meters;
struct Feet;

struct Distance<Unit> {
    value: f64,
    _unit: PhantomData<Unit>,
}

impl Distance<Meters> {
    fn new(value: f64) -> Self {
        Self { value, _unit: PhantomData }
    }
    
    fn to_feet(self) -> Distance<Feet> {
        Distance {
            value: self.value * 3.281,
            _unit: PhantomData,
        }
    }
}

// size_of::<Distance<Meters>>() == size_of::<f64>()
// The Unit parameter adds NO runtime cost

// ✅ ZST for capability tokens
struct AdminToken;  // Zero-sized "proof" of admin status

fn delete_all(_token: AdminToken) {  // Must have token to call
    println!("Deleting everything!");
}
```

---

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| Public types have Debug | MUST | Use custom impl for sensitive data |
| User-facing types have Display | MUST | Implement for errors and identifiers |
| Use strong types | SHOULD | PathBuf not String, Duration not u64 |
| Newtype for semantics | SHOULD | Prevent parameter confusion |
| Newtype for invariants | SHOULD | Validate at construction |
| Provide accessors | SHOULD | Methods to get inner value |
| Implement common traits | SHOULD | Make types composable |

## Type Design Checklist

When creating a new type, consider:

```rust
// 1. Should this be a newtype?
pub struct UserId(u64);  // vs u64

// 2. What traits make sense?
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

// 3. Does it need validation?
impl UserId {
    pub fn new(id: u64) -> Result<Self, ValidationError> {
        // Validate...
    }
}

// 4. Should it have a Display?
impl Display for UserId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

// 5. What accessors are needed?
impl UserId {
    pub fn get(&self) -> u64 { self.0 }
}

// 6. Are there useful constructors?
impl UserId {
    pub fn from_string(s: &str) -> Result<Self, ParseError> {
        // Parse...
    }
}

// 7. Should it be Copy or Clone?
// Clone if it owns data, Copy if trivial

// 8. Does it need Default?
impl Default for Config {
    fn default() -> Self {
        // Sensible defaults
    }
}
```

## Related Guidelines

- **Core Idioms**: See `01-core-idioms.md` for Debug trait
- **API Design**: See `02-api-design.md` for type simplicity
- **Error Handling**: See `03-error-handling.md` for error Display

## External References

- [Rust API Guidelines - Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html)
- [Newtype Pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
- Pragmatic Rust: M-PUBLIC-DEBUG, M-PUBLIC-DISPLAY, M-STRONG-TYPES