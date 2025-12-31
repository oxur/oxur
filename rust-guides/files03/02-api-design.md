# API Design

Comprehensive guidelines for designing public Rust APIs that are flexible, predictable, and interoperable.

## Interoperability

### Types Eagerly Implement Common Traits

**Strength**: MUST

**Summary**: Implement all applicable common traits from `std` to maximize interoperability with the ecosystem.

**Essential traits to implement**:

- `Copy` - Bitwise copyable types
- `Clone` - Types that can be explicitly duplicated
- `Eq` - Total equality
- `PartialEq` - Partial equality
- `Ord` - Total ordering
- `PartialOrd` - Partial ordering
- `Hash` - Hashable types
- `Debug` - Debugging representation
- `Display` - User-facing representation
- `Default` - Default values

**Examples**:

```rust
// Good - comprehensive trait implementations
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserId(u64);

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
}

impl Default for User {
    fn default() -> Self {
        User {
            id: UserId(0),
            name: String::new(),
            email: String::new(),
        }
    }
}

// Good - Display for user-facing output
impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "User({})", self.0)
    }
}

// Bad - missing important traits
pub struct Point {
    x: f64,
    y: f64,
}
// Missing: Clone, Debug, PartialEq, Default
// This limits Point's usefulness in collections, testing, etc.

// Good - implementing both Default and new
#[derive(Clone, Debug, Default)]
pub struct Config {
    pub timeout: u64,
    pub retries: u32,
}

impl Config {
    pub fn new() -> Self {
        // Delegates to Default, or vice versa
        Self::default()
    }
}
```

**Rationale**: Without these traits, your types can't be used in common patterns. For example, types need `Clone` for `Vec`, `Debug` for `assert_eq!`, `Hash` for `HashMap` keys.

**Orphan rule consideration**: Traits can only be implemented where either the trait or the type is defined in your crate. Implementing common traits eagerly prevents downstream users from being unable to add them later.

**See also**:
- C-COMMON-TRAITS in API Guidelines
- C-DEBUG for Debug implementation requirements
- C-SEND-SYNC for thread safety traits

---

### Conversions Use Standard Traits

**Strength**: MUST

**Summary**: Use `From`, `TryFrom`, `AsRef`, `AsMut` traits for conversions, never implement `Into` or `TryInto` directly.

**Examples**:

```rust
// Good - implement From, get Into for free
impl From<u16> for u32 {
    fn from(small: u16) -> u32 {
        small as u32
    }
}

// From provides a blanket Into impl automatically:
// let x: u32 = 100u16.into();

// Good - TryFrom for fallible conversions
use std::convert::TryFrom;

impl TryFrom<u32> for u16 {
    type Error = std::num::TryFromIntError;
    
    fn try_from(big: u32) -> Result<u16, Self::Error> {
        if big <= u16::MAX as u32 {
            Ok(big as u16)
        } else {
            Err(/* ... */)
        }
    }
}

// Usage
let big: u32 = 100_000;
match u16::try_from(big) {
    Ok(small) => println!("Converted: {}", small),
    Err(e) => println!("Too large: {}", e),
}

// Good - AsRef for cheap reference conversions
impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        self
    }
}

// This enables generic APIs like:
fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
    File::open(path.as_ref())
}

// Can be called with String, &str, PathBuf, &Path, etc.
open_file("file.txt");
open_file(String::from("file.txt"));
open_file(PathBuf::from("file.txt"));

// Bad - implementing Into directly
impl Into<u32> for MyType {  // DON'T DO THIS
    fn into(self) -> u32 {
        // Implement From instead
    }
}

// Bad - implementing TryInto directly
impl TryInto<u16> for MyType {  // DON'T DO THIS
    type Error = MyError;
    fn try_into(self) -> Result<u16, Self::Error> {
        // Implement TryFrom instead
    }
}
```

**Trait hierarchy**:

```
From<T>     ──provides──→ Into<T>
TryFrom<T>  ──provides──→ TryInto<T>
```

**Rationale**: `From` and `TryFrom` provide blanket implementations of `Into` and `TryInto` automatically. Implementing the latter directly would be redundant and could cause confusion.

**See also**:
- C-CONV-TRAITS
- C-CONV for method naming conventions

---

### Collections Implement FromIterator and Extend

**Strength**: MUST

**Summary**: Collection types should support construction from and extension by iterators.

**Examples**:

```rust
use std::iter::FromIterator;

// Good - collection implements both traits
pub struct MyVec<T> {
    items: Vec<T>,
}

impl<T> FromIterator<T> for MyVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        MyVec {
            items: iter.into_iter().collect(),
        }
    }
}

impl<T> Extend<T> for MyVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.items.extend(iter);
    }
}

// Usage - collect into custom collection
let my_vec: MyVec<i32> = (0..10).collect();

// Usage - extend existing collection
let mut my_vec = MyVec { items: vec![1, 2, 3] };
my_vec.extend(vec![4, 5, 6]);

// Usage - partition
let (evens, odds): (MyVec<_>, MyVec<_>) = 
    (0..10).partition(|n| n % 2 == 0);

// Usage - unzip
let pairs = vec![(1, 'a'), (2, 'b'), (3, 'c')];
let (nums, chars): (MyVec<_>, MyVec<_>) = pairs.into_iter().unzip();
```

**Rationale**: These traits enable collections to work seamlessly with iterator methods like `collect()`, `partition()`, and `unzip()`, making them first-class citizens in the Rust ecosystem.

**Methods enabled**:
- `Iterator::collect()` - Create collection from iterator
- `Iterator::partition()` - Split into two collections
- `Iterator::unzip()` - Separate tuples into two collections

---

### Data Structures Implement Serde Traits

**Strength**: SHOULD

**Summary**: Types representing data structures should implement `Serialize` and `Deserialize`, typically behind a feature flag.

**Examples**:

```rust
// In Cargo.toml
[dependencies]
serde = { version = "1.0", optional = true, features = ["derive"] }

[features]
default = []
serde = ["dep:serde"]

// In lib.rs - with derive
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
}

// In lib.rs - manual implementation
use serde::{Serialize, Deserialize};

pub struct UserId(pub u64);

#[cfg(feature = "serde")]
impl Serialize for UserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for UserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        u64::deserialize(deserializer).map(UserId)
    }
}
```

**When to implement Serde**:

✅ **Do implement for**:
- Configuration types
- Data transfer objects (DTOs)
- Domain models representing data
- Types you'd want to save/load from files
- API request/response types

❌ **Don't implement for**:
- Marker types (like `PhantomData`)
- Compile-time-only types (like byte order markers)
- Builder types
- Types with complex lifetime dependencies

**Feature flag naming**: Always use `"serde"` as the feature name, not `"serde_support"` or `"serialization"`.

**Rationale**: Serde is the de facto standard for serialization in Rust. Optional implementation allows users who don't need serialization to avoid the compile-time cost.

---

### Types Are Send and Sync Where Possible

**Strength**: MUST

**Summary**: Types should be `Send` and `Sync` unless they genuinely can't be safely used across threads.

**Examples**:

```rust
// Good - automatically Send + Sync
pub struct Data {
    values: Vec<i32>,
    count: usize,
}
// Vec and usize are Send + Sync, so Data is too

// Good - explicitly not Send/Sync when necessary
use std::rc::Rc;

pub struct Shared {
    inner: Rc<String>,
}
// Rc is not Send/Sync, so Shared isn't either

// Good - manually implementing Send/Sync for raw pointers
pub struct MyBox<T> {
    ptr: *mut T,
}

unsafe impl<T: Send> Send for MyBox<T> {}
unsafe impl<T: Sync> Sync for MyBox<T> {}

// Testing Send and Sync
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Data>();
    }
    
    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Data>();
    }
}

// Bad - unnecessarily opting out
pub struct Config {
    name: String,
    // _marker: PhantomData<*const ()>,  // DON'T DO THIS
}
// This would make Config not Send/Sync for no reason
```

**When types are NOT Send/Sync**:
- Types containing `Rc<T>` or `Cell<T>`
- Types with raw pointers that don't guarantee thread safety
- Types that wrap thread-local state
- Types that contain `!Send` or `!Sync` fields

**Rationale**: `Send` and `Sync` are fundamental to Rust's concurrency story. Types that can't be sent across threads or shared between threads are severely limited in their usefulness.

**See also**: C-SEND-SYNC

---

### Error Types Are Meaningful and Well-Behaved

**Strength**: MUST

**Summary**: Error types must implement `Error`, `Send`, `Sync`, and have good `Display` messages.

**Examples**:

```rust
use std::error::Error;
use std::fmt;

// Good - comprehensive error type
#[derive(Debug)]
pub enum ParseError {
    InvalidFormat { line: usize, column: usize },
    UnexpectedEof,
    InvalidCharacter(char),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat { line, column } => {
                write!(f, "invalid format at line {}, column {}", line, column)
            }
            ParseError::UnexpectedEof => {
                write!(f, "unexpected end of file")
            }
            ParseError::InvalidCharacter(ch) => {
                write!(f, "invalid character '{}'", ch)
            }
        }
    }
}

impl Error for ParseError {}

// Automatically Send + Sync because all fields are

// Good - unit struct error when no data needed
#[derive(Debug)]
pub struct ConnectionClosed;

impl fmt::Display for ConnectionClosed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "connection closed")
    }
}

impl Error for ConnectionClosed {}

// Bad - using () as error type
fn parse() -> Result<Data, ()> {  // DON'T DO THIS
    // ...
}
// Problems:
// - () doesn't implement Error
// - () doesn't implement Display
// - unhelpful Debug output
// - can't use with error handling libraries
// - can't use with ? operator in functions returning other errors

// Good - trait object errors must be Send + Sync + 'static
fn get_error() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    Err(Box::new(ParseError::UnexpectedEof))
}

// This allows downcasting
fn handle_error(e: Box<dyn Error + Send + Sync + 'static>) {
    if let Some(parse_err) = e.downcast_ref::<ParseError>() {
        match parse_err {
            ParseError::UnexpectedEof => {
                eprintln!("File ended too soon");
            }
            _ => {}
        }
    }
}
```

**Error message style**:
- Lowercase, no trailing punctuation
- Concise and clear
- Examples: 
  - ✅ `"unexpected end of file"`
  - ✅ `"invalid IP address syntax"`
  - ✅ `"environment variable was not valid unicode: {:?}"`
  - ❌ `"An error occurred."`
  - ❌ `"ERROR: Invalid input!"`

**Required traits**:
1. `Error` - Standard error trait
2. `Debug` - For debugging (usually derived)
3. `Display` - Human-readable messages
4. `Send` - Can be sent across threads
5. `Sync` - Can be shared across threads

**Deprecated**: Never implement `Error::description()`. Use `Display` instead.

**Rationale**: Well-behaved errors integrate with error handling libraries, work across threads, can be used in `async` contexts, and provide good developer experience.

**See also**: C-GOOD-ERR

---

### Binary Number Types Provide Formatting Traits

**Strength**: SHOULD

**Summary**: Implement `Binary`, `Octal`, `LowerHex`, `UpperHex` for types representing binary data or bitflags.

**Examples**:

```rust
use std::fmt;

// Good - bitflags with formatting
#[derive(Clone, Copy, Debug)]
pub struct Permissions(u32);

impl Permissions {
    pub const READ: Self = Permissions(0b001);
    pub const WRITE: Self = Permissions(0b010);
    pub const EXECUTE: Self = Permissions(0b100);
}

impl fmt::Binary for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

// Usage
let perms = Permissions::READ | Permissions::EXECUTE;
println!("{:b}", perms);   // Binary: 101
println!("{:o}", perms);   // Octal: 5
println!("{:x}", perms);   // Lowercase hex: 5
println!("{:X}", perms);   // Uppercase hex: 5

// Don't implement for numeric quantities
pub struct Nanoseconds(u64);
// This is a quantity, not bit manipulation data
// Don't implement Binary, Octal, Hex
```

**When to implement**:
- ✅ Bitflags
- ✅ Binary protocols
- ✅ Hardware registers
- ✅ Masks and bit patterns
- ❌ Numeric quantities (meters, seconds, counts)
- ❌ IDs (user IDs, request IDs)

**Rationale**: Binary number formatting is essential for debugging bit-level operations but inappropriate for semantic numeric types.

---

### Generic Reader/Writer Functions Take by Value

**Strength**: MUST

**Summary**: Functions accepting `R: Read` or `W: Write` should take them by value, not by reference.

**Examples**:

```rust
use std::io::{self, Read, Write};

// Good - takes Read by value
pub fn parse_data<R: Read>(mut reader: R) -> io::Result<Data> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    // parse buffer...
    Ok(Data {})
}

// Good - takes Write by value
pub fn write_data<W: Write>(mut writer: W, data: &Data) -> io::Result<()> {
    writer.write_all(data.as_bytes())?;
    writer.flush()?;
    Ok(())
}

// Why this works - standard library implements Read/Write for &mut T
impl<R: Read + ?Sized> Read for &mut R { /* ... */ }
impl<W: Write + ?Sized> Write for &mut W { /* ... */ }

// Usage - can pass file directly
let file = File::open("data.txt")?;
parse_data(file)?;

// Usage - can pass mutable reference when needed
let mut file = File::create("output.txt")?;
write_data(&mut file, &data)?;  // Can reuse file after this

// More writes to the same file
write_data(&mut file, &other_data)?;
write_data(&mut file, &more_data)?;

// Documentation should mention this pattern
/// Parse data from a reader.
///
/// This function takes the reader by value. If you need to reuse
/// the reader after calling this function, pass `&mut reader` instead.
pub fn parse_data<R: Read>(mut reader: R) -> io::Result<Data> {
    // ...
}
```

**Rationale**: Taking by value allows the function to accept both owned values and mutable references through the blanket impl. This provides maximum flexibility to callers.

**See also**: C-RW-VALUE

---

## Predictability

### Smart Pointers Do Not Add Inherent Methods

**Strength**: MUST

**Summary**: Smart pointer types should not have inherent methods (except constructors) that could be confused with methods on the inner type.

**Examples**:

```rust
// Good - associated function, not method
impl<T: ?Sized> Box<T> {
    pub fn into_raw(b: Box<T>) -> *mut T {
        // Takes Box<T>, not &self
        Box::into_raw(b)
    }
}

// Usage is unambiguous
let boxed_str: Box<str> = Box::new("hello");
let ptr = Box::into_raw(boxed_str);  // Clearly operates on Box

// Bad - if it were a method
// impl<T: ?Sized> Box<T> {
//     pub fn into_raw(self) -> *mut T { /* ... */ }
// }
// 
// let boxed_str: Box<str> = /* ... */;
// boxed_str.into_raw()  // Is this a method on str or Box<str>?

// Good - Rc/Arc pattern
use std::rc::Rc;

let shared = Rc::new(String::from("hello"));
let ptr = Rc::into_raw(shared);  // Clearly operates on Rc
let count = Rc::strong_count(&shared);  // Associated function

// Good - constructors as inherent methods are fine
impl<T> Box<T> {
    pub fn new(value: T) -> Box<T> {
        // Constructors don't conflict with inner type
        Box::new(value)
    }
}
```

**Rationale**: Smart pointers implement `Deref`, so method calls are automatically delegated to the inner type. Inherent methods on the smart pointer would create ambiguity about which type's method is being called.

**See also**: C-SMART-PTR, C-DEREF

---

### Conversions Live on the Most Specific Type

**Strength**: SHOULD

**Summary**: Place conversion methods on the more specific type in the conversion pair.

**Examples**:

```rust
// Good - conversions on the more specific type (str)
impl str {
    // str is more specific than &[u8] (adds UTF-8 constraint)
    pub fn as_bytes(&self) -> &[u8] {
        // str → [u8]
        unsafe { self.as_bytes() }
    }
    
    pub fn from_utf8(bytes: &[u8]) -> Result<&str, Utf8Error> {
        // [u8] → str
        // ...
    }
}

// Bad - would pollute [u8] with endless conversions
// impl [u8] {
//     pub fn as_str(&self) -> Result<&str, Utf8Error> { }
//     pub fn as_os_str(&self) -> Result<&OsStr, _> { }
//     pub fn as_c_str(&self) -> Result<&CStr, _> { }
//     // This would be overwhelming
// }

// Good - PathBuf is more specific than OsString
impl PathBuf {
    pub fn from_os_string(s: OsString) -> PathBuf {
        PathBuf { inner: s }
    }
    
    pub fn into_os_string(self) -> OsString {
        self.inner
    }
}

// Good - String is more specific than Vec<u8>
impl String {
    pub fn from_utf8(vec: Vec<u8>) -> Result<String, FromUtf8Error> {
        // ...
    }
    
    pub fn into_bytes(self) -> Vec<u8> {
        // ...
    }
}
```

**Determining specificity**:
- Type with additional invariants > Type without
- `str` > `[u8]` (UTF-8 invariant)
- `PathBuf` > `OsString` (path semantics)
- `CStr` > `[u8]` (null-termination invariant)

**Rationale**: This prevents cluttering general types with specialized conversions while keeping related conversions discoverable on the type that provides the guarantees.

**See also**: C-CONV-SPECIFIC

---

### Functions With Clear Receiver Are Methods

**Strength**: SHOULD

**Summary**: If a function clearly operates on a specific type, make it a method rather than a free function.

**Examples**:

```rust
// Good - method style
impl Foo {
    pub fn process(&self, widget: Widget) -> Result<(), Error> {
        // ...
    }
}

let foo = Foo::new();
foo.process(widget)?;  // Clear, concise

// Bad - free function style
pub fn process_foo(foo: &Foo, widget: Widget) -> Result<(), Error> {
    // ...
}

process_foo(&foo, widget)?;  // Less clear

// Advantages of methods:
// 1. No imports needed (just need the type in scope)
// 2. Autoborrowing (can call on &foo, &mut foo, foo)
// 3. Discoverable via foo.<tab> in editors
// 4. Self notation clarifies ownership

impl Parser {
    // Clear ownership semantics
    pub fn parse(self) -> Result<Ast, Error> { /* consumes parser */ }
    pub fn peek(&self) -> Option<Token> { /* borrows */ }
    pub fn advance(&mut self) -> Option<Token> { /* mutable borrow */ }
}
```

**When to use free functions instead**:
- No clear receiver type
- Operates equally on multiple types
- Constructors for traits (can't be methods)

**Rationale**: Methods provide better ergonomics, discoverability, and clarity about ownership.

**See also**: C-METHOD

---

### Functions Do Not Take Out-Parameters

**Strength**: MUST

**Summary**: Return multiple values as tuples or structs, not via mutable out-parameters.

**Examples**:

```rust
// Good - return tuple
pub fn split_name(full_name: &str) -> (String, String) {
    let parts: Vec<_> = full_name.split_whitespace().collect();
    (parts[0].to_string(), parts[1].to_string())
}

let (first, last) = split_name("John Doe");

// Good - return struct for many values
pub struct ParseResult {
    pub value: f64,
    pub unit: String,
    pub precision: usize,
}

pub fn parse_measurement(input: &str) -> Result<ParseResult, Error> {
    // ...
}

// Bad - out-parameter style
pub fn split_name_bad(full_name: &str, first: &mut String, last: &mut String) {
    // DON'T DO THIS in Rust
}

// Exception - reusing buffers for performance
pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    // This is OK because caller owns the buffer
    // and wants to reuse it across multiple reads
}

// Good - builder pattern for complex output
pub struct QueryBuilder {
    results: Vec<Row>,
}

impl QueryBuilder {
    pub fn execute(mut self, query: &str) -> Result<Self, Error> {
        // Populate self.results
        Ok(self)
    }
    
    pub fn into_results(self) -> Vec<Row> {
        self.results
    }
}
```

**Rationale**: Rust's efficient return value optimization means returning structs or tuples is cheap. Out-parameters would be a C-ism that doesn't align with Rust's ownership model.

**See also**: C-NO-OUT

---

### Only Smart Pointers Implement Deref

**Strength**: MUST

**Summary**: `Deref` and `DerefMut` should only be implemented for smart pointer types.

**Examples**:

```rust
// Good - smart pointers implement Deref
impl<T: ?Sized> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T { /* ... */ }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &T { /* ... */ }
}

impl Deref for String {
    type Target = str;
    fn deref(&self) -> &str { /* ... */ }
}

// String is a smart pointer to str

// Bad - using Deref for conversions
struct Degrees(f64);
struct Radians(f64);

// DON'T DO THIS
// impl Deref for Degrees {
//     type Target = Radians;
//     fn deref(&self) -> &Radians {
//         // This is an abuse of Deref
//     }
// }

// Good - explicit conversion instead
impl Degrees {
    pub fn to_radians(&self) -> Radians {
        Radians(self.0 * PI / 180.0)
    }
}
```

**What is a smart pointer?**
- Provides ownership or reference semantics over an inner value
- Examples: `Box<T>`, `Rc<T>`, `Arc<T>`, `Cow<'a, T>`, `String` (smart pointer to `str`)

**Why not use Deref for conversions?**
- `Deref` interacts with method resolution in complex ways
- Can cause confusing compiler errors
- Should represent zero-cost pointer-like semantics
- Conversions should be explicit

**See also**: C-DEREF

---

### Constructors Are Static Inherent Methods

**Strength**: MUST

**Summary**: Constructors should be static inherent methods named `new` or following specific patterns.

**Examples**:

```rust
// Good - primary constructor
impl<T> Vec<T> {
    pub fn new() -> Vec<T> {
        Vec { /* ... */ }
    }
}

let v = Vec::new();  // Concise with full type import

// Good - secondary constructors with detail
impl Config {
    pub fn new() -> Self {
        Config::default()
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Config {
            items: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }
    
    pub fn with_timeout(timeout: Duration) -> Self {
        Config {
            timeout,
            ..Default::default()
        }
    }
}

// Good - conversion constructors
impl Error {
    pub fn from_raw_os_error(code: i32) -> Error {
        // ...
    }
}

impl String {
    pub fn from_utf8(bytes: Vec<u8>) -> Result<String, FromUtf8Error> {
        // ...
    }
}

// Good - resource constructors use domain names
impl File {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
        // "open" is more appropriate than "new" for files
    }
}

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        // "connect" better describes TCP semantics
    }
}

// Good - both new and Default
#[derive(Default)]
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new() -> Self {
        Self::default()
    }
}
```

**Constructor naming patterns**:
- `new()` - Primary constructor with no/minimal args
- `with_*()` - Secondary constructors with specific configuration
- `from_*()` - Conversion constructors
- Domain-specific - `open()`, `connect()`, `bind()` for resources

**Rationale**: Static constructors work well with Rust's type imports and provide clear, discoverable API surface.

**See also**: C-CTOR, C-COMMON-TRAITS
