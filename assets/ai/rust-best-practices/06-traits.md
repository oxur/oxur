# Trait Design and Implementation

Guidelines for designing traits, implementing them effectively, and using trait objects.


## TR-01: `impl Trait` in Argument Position

**Strength**: SHOULD

**Summary**: Use `impl Trait` for simpler function signatures.

```rust
// ✅ CLEANER: impl Trait
fn process(iter: impl Iterator<Item = i32>) -> i32 {
    iter.sum()
}

// EQUIVALENT but verbose:
fn process<I: Iterator<Item = i32>>(iter: I) -> i32 {
    iter.sum()
}

// ✅ Multiple bounds
fn send_all(items: impl IntoIterator<Item = impl AsRef<str>>) {
    for item in items {
        send(item.as_ref());
    }
}

// ⚠️ LIMITATION: Can't specify the concrete type
fn process(iter: impl Iterator<Item = i32>) {
    // Caller can't use turbofish: process::<std::vec::IntoIter<i32>>(...)
}

// Use generics when callers need to specify type:
fn process<I: Iterator<Item = i32>>(iter: I) {
    // Caller CAN use: process::<std::vec::IntoIter<i32>>(...)
}
```

---

## TR-02: Blanket Implementations

**Strength**: CONSIDER

**Summary**: Implement traits for all types meeting certain bounds.

```rust
use std::fmt::Display;

// ✅ BLANKET IMPL: Any Display type can be logged
trait Loggable {
    fn log(&self);
}

impl<T: Display> Loggable for T {
    fn log(&self) {
        println!("[LOG] {}", self);
    }
}

// Now any Display type has .log():
"hello".log();
42.log();

// ✅ BLANKET IMPL: References implement trait if T does
impl<T: MyTrait + ?Sized> MyTrait for &T {
    fn method(&self) {
        (**self).method()
    }
}

// ✅ BLANKET IMPL: Box<T> implements trait if T does
impl<T: MyTrait + ?Sized> MyTrait for Box<T> {
    fn method(&self) {
        (**self).method()
    }
}

// ⚠️ CAUTION: Blanket impls can conflict
// If you have `impl<T: A> MyTrait for T` and `impl<T: B> MyTrait for T`
// they conflict for any T that implements both A and B
```

---

## TR-03: Coherence and Orphan Rules

**Strength**: MUST

**Summary**: You can only implement traits when you "own" either the trait or the type.

```rust
// ✅ CAN: Your trait on foreign type
pub trait MyTrait { }
impl MyTrait for String { }  // OK: You own MyTrait

// ✅ CAN: Foreign trait on your type
pub struct MyType;
impl std::fmt::Display for MyType { }  // OK: You own MyType

// ❌ CANNOT: Foreign trait on foreign type
impl std::fmt::Display for String { }  // ERROR: Orphan rule

// ✅ WORKAROUND: Newtype wrapper
pub struct MyString(String);
impl std::fmt::Display for MyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Custom formatting
        write!(f, "MyString({})", self.0)
    }
}

// ✅ SPECIAL CASE: Blanket with local trait
pub trait MyExt {
    fn my_method(&self);
}
impl<T: std::fmt::Display> MyExt for T {  // OK: You own MyExt
    fn my_method(&self) {
        println!("Extended: {}", self);
    }
}
```

---

## TR-04: Essential Functionality Should be Inherent

**Strength**: MUST

**Summary**: Types should implement core functionality inherently; trait implementations should forward to inherent methods.

```rust
// Bad - essential functionality only in traits
struct HttpClient {}

impl Download for HttpClient {
    fn download_file(&self, url: impl AsRef<str>) {
        // Core logic buried in trait implementation
        // Users must `use Download` to access this!
    }
}

// Good - essential functionality is inherent
struct HttpClient {}

impl HttpClient {
    // Core functionality available without imports
    pub fn download_file(&self, url: impl AsRef<str>) {
        // ... download logic
    }
}

// Trait implementation forwards to inherent method
impl Download for HttpClient {
    fn download_file(&self, url: impl AsRef<str>) {
        // Simply forward to the inherent implementation
        Self::download_file(self, url)
    }
}
```

**Rationale**: Offloading essential functionality into traits means users must discover and import the right traits to use your type. Inherent methods are immediately discoverable and don't require trait imports.

**See also**: M-ESSENTIAL-FN-INHERENT, Rust API Guidelines C-CONV

---

## TR-05: Extension Traits

**Strength**: CONSIDER

**Summary**: Add methods to foreign types without newtype wrappers.

```rust
// ✅ Extension trait for Option<String>
pub trait OptionStringExt {
    fn or_empty(self) -> String;
    fn is_blank(&self) -> bool;
}

impl OptionStringExt for Option<String> {
    fn or_empty(self) -> String {
        self.unwrap_or_default()
    }
    
    fn is_blank(&self) -> bool {
        self.as_ref().map_or(true, |s| s.trim().is_empty())
    }
}

// ✅ Extension trait for iterators
pub trait IteratorExt: Iterator {
    fn try_collect_vec(self) -> Result<Vec<Self::Item>, Error>
    where
        Self: Sized,
        Self::Item: TryInto<Output, Error = Error>;
}

impl<I: Iterator> IteratorExt for I {
    fn try_collect_vec(self) -> Result<Vec<Self::Item>, Error>
    where
        Self: Sized,
        Self::Item: TryInto<Output, Error = Error>,
    {
        self.collect()
    }
}

// Usage requires importing the trait:
use my_crate::OptionStringExt;

let name: Option<String> = None;
println!("{}", name.or_empty());
```

---

## TR-06: Forwarding Implementations

**Strength**: SHOULD

**Summary**: When implementing traits on wrapper types, forward to the inner implementation.

```rust
use std::fmt;

struct Logged<T> {
    inner: T,
    name: String,
}

// Forward Debug to inner type
impl<T: fmt::Debug> fmt::Debug for Logged<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Logged")
            .field("name", &self.name)
            .field("inner", &self.inner)
            .finish()
    }
}

// Forward Clone when inner is Clone
impl<T: Clone> Clone for Logged<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            name: self.name.clone(),
        }
    }
}
```

**Rationale**: Wrapper types should behave like their inner types where appropriate. Forwarding implementations maintains expected behavior and allows wrappers to compose.

---

---

## TR-07: Generic Traits vs Associated Types

**Strength**: SHOULD

**Summary**: Use associated types for "output" types, generics for "input" types.

```rust
// ✅ ASSOCIATED TYPE: One output type per implementation
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Vec<i32> has ONE iterator item type: i32
// You can't implement Iterator twice for different Item types

// ✅ GENERIC PARAMETER: Multiple implementations possible
trait From<T> {
    fn from(value: T) -> Self;
}

// String can implement From<&str>, From<char>, From<Vec<u8>>, etc.
// Multiple implementations for different T

// Decision guide:
// Q: "For a given type, is there ONE X or MANY X?"
// ONE → Associated type
// MANY → Generic parameter

// ✅ COMBINED: Both can be used together
trait Converter<Input> {  // Generic: many input types
    type Output;          // Associated: one output per Input
    type Error;           // Associated: one error per Input
    
    fn convert(&self, input: Input) -> Result<Self::Output, Self::Error>;
}
```

---

## TR-08: Implement Common Derive Traits

**Strength**: MUST

**Summary**: Types should derive or implement Clone, Debug, PartialEq, and other common traits where applicable.

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

## TR-09: Implement Display for User-Facing Types

**Strength**: SHOULD

**Summary**: Types that users will see should implement Display with clear, helpful messages.

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

```rust
let value = EmailAddress("test@example.com".to_string());

// Display: clean for users
assert_eq!(format!("{}", value), "test@example.com");

// Debug: structural for developers
assert_eq!(format!("{:?}", value), "EmailAddress(\"test@example.com\")");
```

**Rationale**: Display provides clean output for users while Debug is for developers.

---

---

## TR-10: Keep Trait Bounds Minimal

**Strength**: SHOULD

**Summary**: Only require trait bounds that are actually needed for the implementation.

```rust
// Bad - unnecessary bounds
struct Container<T: Clone + Debug> {
    items: Vec<T>,
}

impl<T: Clone + Debug> Container<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn add(&mut self, item: T) {
        // Doesn't use Clone or Debug!
        self.items.push(item);
    }
}

// Good - bounds only where needed
struct Container<T> {
    items: Vec<T>,
}

impl<T> Container<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }
}

// Add bounds only for specific methods
impl<T: Clone> Container<T> {
    pub fn duplicate_last(&mut self) {
        if let Some(last) = self.items.last() {
            self.items.push(last.clone());
        }
    }
}

impl<T: Debug> Container<T> {
    pub fn debug_print(&self) {
        for item in &self.items {
            println!("{:?}", item);
        }
    }
}
```

**Rationale**: Overly restrictive trait bounds limit your type's usability. Users can't use `Container<NotClone>` even if they never call `duplicate_last()`. Place bounds on implementations, not type definitions.

**See also**: Generic Programming best practices

---

## TR-11: Marker Traits

**Strength**: CONSIDER

**Summary**: Traits with no methods that mark type properties.

```rust
// Standard marker traits:
// - Send: Safe to transfer between threads
// - Sync: Safe to share references between threads
// - Copy: Implicitly copied on assignment
// - Sized: Has known size at compile time
// - Unpin: Can be moved after being pinned

// ✅ Custom marker trait
trait ThreadSafeCache: Send + Sync {}

// Blanket impl for qualifying types
impl<T: Send + Sync> ThreadSafeCache for T {}

// ✅ Marker for type-level flags
trait Validated {}

struct Email<V> {
    value: String,
    _marker: std::marker::PhantomData<V>,
}

struct Unvalidated;
struct ValidatedMarker;

impl Email<Unvalidated> {
    fn validate(self) -> Result<Email<ValidatedMarker>, ValidationError> {
        // validation logic
        Ok(Email {
            value: self.value,
            _marker: std::marker::PhantomData,
        })
    }
}

impl Email<ValidatedMarker> {
    fn send(&self) {
        // Can only send validated emails
    }
}
```

---

## TR-12: Narrow Traits Over Wide Traits

**Strength**: SHOULD

**Summary**: When designing trait hierarchies, prefer multiple narrow traits over one wide trait.

```rust
// Bad - one wide trait forces users to implement everything
trait Database {
    async fn store_object(&self, id: Id, obj: Object);
    async fn load_object(&self, id: Id) -> Object;
    async fn delete_object(&self, id: Id);
    async fn update_config(&self, file: PathBuf);
}

// Good - narrow traits allow selective implementation
trait StoreObject {
    async fn store_object(&self, id: Id, obj: Object);
}

trait LoadObject {
    async fn load_object(&self, id: Id) -> Object;
}

trait DeleteObject {
    async fn delete_object(&self, id: Id);
}

// Combine via supertrait when needed
trait DataAccess: StoreObject + LoadObject + DeleteObject {}

// Users can depend on just what they need
async fn read_database(x: impl LoadObject) { 
    // Only requires LoadObject, not full Database
}
```

**Rationale**: Narrow traits give users flexibility to implement only what they need and allow code to depend on minimal interfaces. They compose better and are easier to mock for testing.

**See also**: M-DI-HIERARCHY, Interface Segregation Principle

---

## TR-13: Negative Trait Bounds (Unstable Pattern)

**Strength**: CONSIDER

**Summary**: Rust doesn't have negative bounds, but you can work around it.

```rust
// Can't write: impl<T: !Copy> MyTrait for T

// ✅ WORKAROUND: Use auto traits (limited)
// Some types automatically implement Send/Sync/Unpin
// If your type contains !Send, it's !Send

// ✅ WORKAROUND: Sealed helper trait
mod private {
    pub trait NotCopy {}
}

impl private::NotCopy for String {}
impl private::NotCopy for Vec<u8> {}
// Don't impl for Copy types

trait OnlyForNonCopy: private::NotCopy {
    fn move_out(self);
}
```

---

## TR-14: Object Safety

**Strength**: MUST

**Summary**: Traits must be "object-safe" to use as `dyn Trait`.

```rust
// ✅ OBJECT-SAFE: Can be used as dyn Trait
trait Draw {
    fn draw(&self, canvas: &mut Canvas);
    fn bounding_box(&self) -> Rect;
}

fn draw_all(shapes: &[Box<dyn Draw>]) {
    for shape in shapes {
        shape.draw(&mut canvas);
    }
}

// ❌ NOT OBJECT-SAFE: Generic method
trait Serialize {
    fn serialize<W: Write>(&self, writer: W);  // Generic!
}
// Error: cannot use `dyn Serialize`

// ❌ NOT OBJECT-SAFE: Returns Self
trait Clone {
    fn clone(&self) -> Self;  // Returns Self!
}
// Error: cannot use `dyn Clone`

// ❌ NOT OBJECT-SAFE: Associated const or type with bounds
trait BadTrait {
    const SIZE: usize;  // Associated const
    type Output: Clone; // Bounded associated type
}
```

```rust
trait Serializable {
    // Object-safe version
    fn serialize_to(&self, writer: &mut dyn Write) -> Result<(), Error>;
}

trait SerializableExt: Serializable {
    // Non-object-safe convenience method
    fn serialize<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.serialize_to(&mut writer)
    }
}

impl<T: Serializable> SerializableExt for T {}
```

---

## TR-15: Object Safety Considerations

**Strength**: MUST

**Summary**: Traits used as trait objects must be object-safe: no generic methods, no `Self: Sized` bounds, methods return simple types.

```rust
// Object-safe trait
trait Drawable {
    fn draw(&self);
    fn bounds(&self) -> Rect;
}

// Can use as trait object
let objects: Vec<Box<dyn Drawable>> = vec![
    Box::new(Circle { radius: 10 }),
    Box::new(Rectangle { width: 20, height: 30 }),
];

// Not object-safe - generic method
trait Container {
    fn add<T>(&mut self, item: T); // ❌ generic method
}

// Not object-safe - returns Self
trait Cloneable {
    fn clone_box(&self) -> Self; // ❌ returns Self
}

// Object-safe version
trait CloneableObject {
    fn clone_box(&self) -> Box<dyn CloneableObject>;
}
```

**Rationale**: Object safety rules ensure trait objects can be used with dynamic dispatch. The compiler cannot generate vtables for generic methods or methods that return `Self` in unknown sizes.

**See also**: [Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

---

## TR-16: Operator Overloads Are Unsurprising

**Strength**: MUST

**Summary**: Only implement operator traits when the operation naturally corresponds to the operator.

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

## TR-17: Prefer Concrete Types > Generics > dyn Trait

**Strength**: SHOULD

**Summary**: Use concrete types when possible, generics when flexibility is needed, and trait objects (`dyn Trait`) only when necessary to avoid excessive nesting.

```rust
// Best - concrete type
struct MyService {
    db: PostgresDatabase,
}

// Good - generic for flexibility without nesting issues
struct MyService<T: LoadObject> {
    db: T,
}

async fn read_database(x: impl LoadObject) {
    // Generic parameter, compiles to monomorphized code
}

// Consider - when generics cause excessive nesting
// Instead of Service<Backend<Store<Config>>>
struct DynamicDataAccess {
    inner: Arc<dyn DataAccess>,
}

impl DynamicDataAccess {
    pub fn new<T: DataAccess + 'static>(db: T) -> Self {
        Self {
            inner: Arc::new(db),
        }
    }
}

// Then use concrete type
struct MyService {
    db: DynamicDataAccess,
}
```

**Rationale**: - Concrete types are simple and have zero runtime overhead
- Generics enable flexibility and are zero-cost abstractions (monomorphized)
- Trait objects (`dyn Trait`) have small runtime cost but prevent type nesting explosion

Use trait objects when:
- Generic type parameters would nest excessively (3+ levels)
- Runtime polymorphism is genuinely needed
- Compile times are becoming problematic from monomorphization

**See also**: M-DI-HIERARCHY, M-SIMPLE-ABSTRACTIONS

---

## TR-18: Provide From Conversions for Related Types

**Strength**: SHOULD

**Summary**: Implement From<T> to enable ergonomic conversions and interoperability.

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

## TR-19: Sealed Traits Protect Against Downstream Implementations

**Strength**: CONSIDER

**Summary**: Use the sealed trait pattern to prevent external implementations while preserving the ability to add methods.

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

## TR-20: Supertraits for Trait Composition

**Strength**: SHOULD

**Summary**: Use supertraits when a trait requires another trait's functionality.

```rust
use std::fmt::Debug;
use std::hash::Hash;

// ✅ GOOD: Require Debug for better error messages
trait Repository: Debug {
    fn save(&mut self, item: &Item) -> Result<(), SaveError>;
}

// ✅ GOOD: Compose multiple requirements
trait CacheKey: Clone + Hash + Eq + Debug {}

// Blanket impl: anything meeting requirements is a CacheKey
impl<T: Clone + Hash + Eq + Debug> CacheKey for T {}

// ✅ GOOD: Standard supertrait pattern
trait Error: Debug + std::fmt::Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> { None }
}

// Usage:
fn log_error<E: Error>(e: &E) {
    // Can use both Debug and Display
    println!("Error: {}", e);
    println!("Debug: {:?}", e);
}
```

---

## TR-21: The `Deref` Trait (Use Sparingly)

**Strength**: CONSIDER

**Summary**: `Deref` is for smart pointer types, not inheritance.

```rust
use std::ops::Deref;

// ✅ CORRECT: Smart pointer pattern
struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

// Enables: *my_box, my_box.method_on_t()

// ✅ CORRECT: String is a smart pointer to str
// String implements Deref<Target = str>
let s = String::from("hello");
let len = s.len();  // Calls str::len() via Deref

// ❌ WRONG: Deref for "inheritance"
struct Dog {
    animal: Animal,
}

impl Deref for Dog {
    type Target = Animal;
    fn deref(&self) -> &Animal {
        &self.animal  // DON'T DO THIS
    }
}
// This is anti-pattern! Use composition + delegation instead.
```

---

## TR-22: Trait Definition Basics

**Strength**: SHOULD

**Summary**: Design traits around behavior, not data.

```rust
// ❌ BAD: Trait as data interface (getter/setter)
trait HasName {
    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: &str);
}

// ✅ GOOD: Trait as behavior interface
trait Named {
    fn name(&self) -> &str;
}

trait Greet {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
    
    fn name(&self) -> &str;  // Required method
}

// ✅ GOOD: Trait with associated types
trait Parser {
    type Output;
    type Error;
    
    fn parse(&self, input: &str) -> Result<Self::Output, Self::Error>;
}

// ✅ GOOD: Trait with default implementations
trait Drawable {
    fn draw(&self, canvas: &mut Canvas);
    
    fn draw_with_offset(&self, canvas: &mut Canvas, x: i32, y: i32) {
        canvas.translate(x, y);
        self.draw(canvas);
        canvas.translate(-x, -y);
    }
}
```

---

## TR-23: Trait Objects vs Generics

**Strength**: SHOULD

**Summary**: Generics for performance, trait objects for flexibility.

```rust
// ✅ GENERIC: Monomorphized, zero-cost
fn process_generic<T: Process>(items: &[T]) {
    for item in items {
        item.process();
    }
}
// Compiler generates specialized code for each T

// ✅ TRAIT OBJECT: Dynamic dispatch, one function
fn process_dynamic(items: &[Box<dyn Process>]) {
    for item in items {
        item.process();  // Virtual call
    }
}
// One function handles all types

// ✅ TRAIT OBJECT: Heterogeneous collection
let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle::new()),
    Box::new(Rectangle::new()),
    Box::new(Triangle::new()),
];
// Can't do this with generics!

// ✅ GENERIC: Homogeneous collection
let circles: Vec<Circle> = vec![
    Circle::new(),
    Circle::new(),
];
```

---

## TR-24: Types Eagerly Implement Common Traits

**Strength**: SHOULD

**Summary**: Public types should derive or implement common traits where applicable: `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Default`, `Debug`, `Display`.

```rust
// Good - comprehensive trait implementations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Pending,
    Active,
    Completed,
}

// Default when it makes sense
impl Default for Status {
    fn default() -> Self {
        Status::Pending
    }
}

// Display for user-facing types
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::Active => write!(f, "active"),
            Status::Completed => write!(f, "completed"),
        }
    }
}
```

**Rationale**: Users expect common traits to be implemented. Missing implementations force users to work around limitations or implement wrappers. These traits enable use in standard collections, comparison operations, and debugging.

**See also**: C-COMMON-TRAITS, M-PUBLIC-DEBUG

---

## TR-25: Use dyn Trait for Runtime Polymorphism

**Strength**: SHOULD

**Summary**: Use trait objects (dyn Trait) for heterogeneous collections or when type erasure is needed.

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

---

## TR-26: Use Extension Traits for Optional Functionality

**Strength**: CONSIDER

**Summary**: Provide optional functionality through extension traits to avoid tight coupling.

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

**Rationale**: Extension traits provide optional ergonomics without coupling core functionality to convenience methods.

---

## Summary Table

| Pattern | Strength | Key Insight |
|---------|----------|-------------|
| Essential functionality inherent | MUST | Don't hide core methods in trait impls |
| Narrow traits | SHOULD | Prefer `StoreObject + LoadObject` over `Database` |
| Implement common traits | SHOULD | `Debug`, `Clone`, `Eq`, `Hash`, etc. |
| Concrete > Generic > dyn | SHOULD | Avoid trait objects unless needed |
| Object safety | MUST | Know the rules when using `dyn Trait` |
| Forward trait impls | SHOULD | Wrappers should behave like their inner type |
| Minimal trait bounds | SHOULD | Bound implementations, not type definitions |

---

## Related Guidelines

- **API Design**: See `02-api-design.md` for API composition patterns
- **Type Design**: See `05-type-design.md` for when to use traits vs concrete types
- **Anti-patterns**: See `11-anti-patterns.md` for trait-related mistakes

---

## External References

- [Rust API Guidelines - C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#c-common-traits)
- [Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)
- Pragmatic Rust Guidelines: M-ESSENTIAL-FN-INHERENT, M-DI-HIERARCHY