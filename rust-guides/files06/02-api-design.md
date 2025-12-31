# API Design Guidelines

Comprehensive guidelines for designing public APIs in Rust, based on the official Rust API Guidelines from the Rust library team.

---

## Naming Conventions (RFC 430)

**Strength**: MUST

**Summary**: Follow Rust's established naming conventions for all public items.

**Example**:
```rust
// Good - proper casing for different items
pub struct HttpClient { /* ... */ }          // UpperCamelCase for types
pub trait Serialize { /* ... */ }             // UpperCamelCase for traits
pub enum Color { Red, Green, Blue }           // UpperCamelCase for types and variants
pub fn parse_config() -> Config { /* ... */ } // snake_case for functions
pub const MAX_SIZE: usize = 1024;             // SCREAMING_SNAKE_CASE for constants

// Good - acronyms are treated as one word
pub struct Uuid;  // not UUID
pub fn is_xid_start() -> bool { /* ... */ }  // not is_XID_start

// Bad - incorrect casing
pub struct HTTP_Client { /* ... */ }  // should be HttpClient
pub fn ParseConfig() { /* ... */ }    // should be parse_config
```

**Rationale**: Consistent naming conventions make Rust code instantly recognizable and reduce cognitive load when reading code from different sources.

**See also**: RFC 430, 01-core-idioms.md

---

## Conversion Method Naming (as_, to_, into_)

**Strength**: MUST

**Summary**: Use `as_` for free borrowed-to-borrowed, `to_` for expensive conversions, and `into_` for consuming conversions.

**Example**:
```rust
// Good - as_ for free, non-consuming conversions
impl str {
    pub fn as_bytes(&self) -> &[u8] { /* ... */ }
}

// Good - to_ for expensive conversions (may allocate)
impl str {
    pub fn to_lowercase(&self) -> String { /* ... */ }
    pub fn to_owned(&self) -> String { /* ... */ }
}

// Good - into_ for consuming conversions
impl String {
    pub fn into_bytes(self) -> Vec<u8> { /* ... */ }
}

// Good - into_inner for wrapper types
impl BufReader<R> {
    pub fn into_inner(self) -> R { /* ... */ }
}

// Bad - misleading names
impl str {
    pub fn to_bytes(&self) -> &[u8] { /* ... */ }  // Should be as_bytes (free)
    pub fn as_lowercase(&self) -> String { /* ... */ }  // Should be to_lowercase (expensive)
}
```

**Rationale**: These prefixes provide clear expectations about cost and ownership. Users can optimize based on whether a conversion is free (`as_`), expensive (`to_`), or consuming (`into_`).

**See also**: C-CONV guideline, From/Into traits

---

## Getter Naming

**Strength**: SHOULD

**Summary**: Omit the `get_` prefix for getters unless there's a compelling reason.

**Example**:
```rust
// Good - simple, clean getters
pub struct Person {
    name: String,
    age: u32,
}

impl Person {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn age(&self) -> u32 {
        self.age
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }
}

// Good - use get_ when there's a single obvious thing to "get"
use std::cell::Cell;
let cell = Cell::new(5);
let value = cell.get();  // Obvious what we're getting

// Good - get_unchecked for unsafe variants
fn get(&self, index: usize) -> Option<&T>;
unsafe fn get_unchecked(&self, index: usize) -> &T;

// Bad - unnecessary get_ prefix
impl Person {
    pub fn get_name(&self) -> &str { /* ... */ }  // Just use name()
    pub fn get_age(&self) -> u32 { /* ... */ }     // Just use age()
}
```

**Rationale**: Rust's method syntax makes it clear you're accessing a field. The `get_` prefix adds verbosity without clarity.

**See also**: C-GETTER guideline

---

## Iterator Method Naming

**Strength**: MUST

**Summary**: Use `iter()`, `iter_mut()`, and `into_iter()` for iterator-producing methods on collections.

**Example**:
```rust
// Good - standard iterator naming
impl<T> Vec<T> {
    pub fn iter(&self) -> Iter<'_, T> { /* ... */ }
    pub fn iter_mut(&mut self) -> IterMut<'_, T> { /* ... */ }
    pub fn into_iter(self) -> IntoIter<T> { /* ... */ }
}

// Good - iterator type names match methods
pub struct Iter<'a, T> { /* ... */ }
pub struct IterMut<'a, T> { /* ... */ }
pub struct IntoIter<T> { /* ... */ }

// Good - specialized iterators have descriptive names
impl<K, V> HashMap<K, V> {
    pub fn keys(&self) -> Keys<'_, K, V> { /* ... */ }
    pub fn values(&self) -> Values<'_, K, V> { /* ... */ }
}

// Bad - non-standard iterator names
impl<T> MyCollection<T> {
    pub fn get_iterator(&self) -> Iter<T> { /* ... */ }  // Should be iter()
    pub fn to_iter(self) -> IntoIter<T> { /* ... */ }    // Should be into_iter()
}
```

**Rationale**: Consistent iterator naming makes collections immediately familiar and enables generic code to work across different collection types.

**See also**: C-ITER, C-ITER-TY guidelines

---

## Implement Common Traits Eagerly

**Strength**: MUST

**Summary**: Implement Copy, Clone, Debug, Default, and other common traits for public types whenever semantically appropriate.

**Example**:
```rust
// Good - eagerly implement common traits
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// Good - implement both Default and new()
#[derive(Default)]
pub struct Config {
    timeout: u64,    // defaults to 0
    retries: u32,    // defaults to 0
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}

// Good - custom Default for better defaults
pub struct ServerOptions {
    pub host: String,
    pub port: u16,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 8080,
        }
    }
}

// Bad - missing common traits makes type hard to use
pub struct Data {
    value: i32,
}
// No Clone, Debug, PartialEq, etc. - users can't easily work with this type
```

**Rationale**: Due to the orphan rule, downstream crates cannot add these implementations. Providing them upfront maximizes interoperability.

**See also**: C-COMMON-TRAITS guideline

---

## Use Standard Conversion Traits

**Strength**: MUST

**Summary**: Implement From/TryFrom, AsRef/AsMut for conversions. Never implement Into/TryInto directly.

**Example**:
```rust
// Good - implement From, get Into for free
impl From<u16> for u32 {
    fn from(small: u16) -> Self {
        small as u32
    }
}

// Good - TryFrom for fallible conversions
use std::convert::TryFrom;

impl TryFrom<u32> for u16 {
    type Error = std::num::TryFromIntError;

    fn try_from(large: u32) -> Result<Self, Self::Error> {
        u16::try_from(large)
    }
}

// Good - AsRef for flexible APIs
fn open_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
    File::open(path.as_ref())
}

// Now can call with &str, String, Path, PathBuf, etc.
open_file("file.txt");
open_file(String::from("file.txt"));

// Bad - implementing Into directly
impl Into<u32> for MyType {  // Don't do this!
    fn into(self) -> u32 { /* ... */ }
}

// Instead, implement From:
impl From<MyType> for u32 {
    fn from(value: MyType) -> u32 { /* ... */ }
}
```

**Rationale**: From/Into have blanket implementations. Implementing From automatically provides Into for free.

**See also**: C-CONV-TRAITS guideline

---

## Collections Implement FromIterator and Extend

**Strength**: SHOULD

**Summary**: Collection types should implement FromIterator for construction and Extend for adding items.

**Example**:
```rust
use std::iter::FromIterator;

// Good - implement FromIterator
pub struct MyCollection<T> {
    items: Vec<T>,
}

impl<T> FromIterator<T> for MyCollection<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        MyCollection {
            items: iter.into_iter().collect(),
        }
    }
}

// Good - implement Extend
impl<T> Extend<T> for MyCollection<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.items.extend(iter);
    }
}

// Usage becomes ergonomic
let items: MyCollection<i32> = vec![1, 2, 3].into_iter().collect();
let more: MyCollection<i32> = (0..10).collect();

let mut collection = MyCollection::from_iter(vec![1, 2, 3]);
collection.extend(vec![4, 5, 6]);
```

**Rationale**: These traits enable collections to work seamlessly with iterators, unlocking powerful composition patterns.

**See also**: C-COLLECT guideline

---

## Error Types Are Meaningful

**Strength**: MUST

**Summary**: Error types must implement Error, Send, Sync and should never be `()`. Error messages should be lowercase without trailing punctuation.

**Example**:
```rust
use std::fmt;
use std::error::Error;

// Good - proper error type
#[derive(Debug)]
pub struct ParseError {
    line: usize,
    column: usize,
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error at {}:{}: {}",
               self.line, self.column, self.message)
    }
}

impl Error for ParseError {}

// Error messages: lowercase, no trailing punctuation
// Good: "unexpected end of file"
// Good: "invalid UTF-8 sequence of 2 bytes from index 5"
// Bad: "Unexpected end of file."
// Bad: "Invalid UTF-8!"

// Good - error with source tracking
#[derive(Debug)]
pub struct ConfigError {
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}

// Bad - unit type as error
fn parse(input: &str) -> Result<Data, ()> { /* ... */ }  // Never use () as error!

// Bad - error type not Send + Sync
struct BadError {
    handle: Rc<RefCell<File>>,  // Not Send or Sync!
}
```

**Rationale**: Meaningful error types enable proper error handling, chaining, and debugging. Send + Sync requirements enable multithreaded error handling.

**See also**: C-GOOD-ERR guideline, 03-error-handling.md

---

## Functions Take R: Read and W: Write by Value

**Strength**: SHOULD

**Summary**: Generic reader/writer functions should take by value, leveraging the blanket impls that allow passing `&mut` references.

**Example**:
```rust
use std::io::{Read, Write};

// Good - takes by value, can still be called with &mut
pub fn copy<R: Read, W: Write>(reader: R, writer: W) -> io::Result<u64> {
    // Because of impls: impl<R: Read> Read for &mut R
    //                   impl<W: Write> Write for &mut W
    // Users can pass &mut file if they want to reuse it
}

// Usage
let mut file = File::open("data.txt")?;
copy(&mut file, &mut stdout())?;  // Can pass &mut
copy(file, stdout())?;             // Or consume

// In docs, remind users they can pass &mut:
/// Copies data from reader to writer.
///
/// This function takes ownership of both reader and writer. To reuse
/// them afterward, pass them by mutable reference (`&mut file`).
```

**Rationale**: Taking by value is more flexible—callers can choose to pass owned values or `&mut` references as needed.

**See also**: C-RW-VALUE guideline

---

## Smart Pointers Don't Add Inherent Methods

**Strength**: MUST

**Summary**: Smart pointer types should not have inherent methods that could be confused with methods on the pointed-to type.

**Example**:
```rust
// Good - associated function, not method
impl<T> Box<T> {
    pub fn into_raw(b: Box<T>) -> *mut T { /* ... */ }
}

// Usage is unambiguous
let boxed = Box::new("hello");
let ptr = Box::into_raw(boxed);  // Clearly a Box method

// Bad - if this were an inherent method
impl<T> Box<T> {
    pub fn into_raw(self) -> *mut T { /* ... */ }  // Don't do this!
}

// Would be confusing:
let boxed = Box::new(my_struct);
boxed.some_method();  // Is this on Box or my_struct?
boxed.into_raw();     // Is this on Box or my_struct?
```

**Rationale**: Smart pointers use Deref to transparently access the inner type. Inherent methods would be ambiguous with methods on the inner type.

**See also**: C-SMART-PTR guideline

---

## Conversions Live on the Most Specific Type

**Strength**: SHOULD

**Summary**: Place conversion methods on the more specific of the two types involved.

**Example**:
```rust
// Good - str is more specific than &[u8]
impl str {
    pub fn as_bytes(&self) -> &[u8] { /* ... */ }
    pub fn from_utf8(bytes: &[u8]) -> Result<&str, Utf8Error> { /* ... */ }
}

// Bad - pollutes the less specific type
impl [u8] {
    pub fn to_str(&self) -> Result<&str, Utf8Error> { /* ... */ }  // Don't do this
}

// Good - Path is more specific than &OsStr
impl Path {
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Path { /* ... */ }
    pub fn as_os_str(&self) -> &OsStr { /* ... */ }
}
```

**Rationale**: Keeping conversions on the specific type avoids polluting general types with countless conversion methods and makes the API more discoverable.

**See also**: C-CONV-SPECIFIC guideline

---

## Functions with Clear Receivers Are Methods

**Strength**: SHOULD

**Summary**: When a function operates on a specific type, make it a method rather than a free function.

**Example**:
```rust
// Good - methods for type-specific operations
impl Configuration {
    pub fn load(&mut self, path: &Path) -> Result<()> { /* ... */ }
    pub fn save(&self, path: &Path) -> Result<()> { /* ... */ }
    pub fn validate(&self) -> Result<()> { /* ... */ }
}

// Bad - free functions when methods would be clearer
pub fn load_configuration(config: &mut Configuration, path: &Path) -> Result<()> { /* ... */ }
pub fn save_configuration(config: &Configuration, path: &Path) -> Result<()> { /* ... */ }

// Usage comparison
config.load(path)?;  // Clear and concise
load_configuration(&mut config, path)?;  // Verbose and awkward
```

**Rationale**: Methods don't need imports, support auto-borrowing, appear in rustdoc for the type, and use clean `self` notation.

**See also**: C-METHOD guideline

---

## No Out-Parameters

**Strength**: MUST

**Summary**: Return multiple values via tuples or structs, not out-parameters.

**Example**:
```rust
// Good - return tuple
pub fn parse_header(data: &[u8]) -> Result<(Header, usize), ParseError> {
    // Returns both the parsed header and bytes consumed
}

// Good - return struct for many values
pub struct ParseResult {
    pub header: Header,
    pub bytes_consumed: usize,
    pub warnings: Vec<String>,
}

pub fn parse_with_details(data: &[u8]) -> Result<ParseResult, ParseError> { /* ... */ }

// Bad - out-parameter pattern
pub fn parse_header_bad(data: &[u8], header: &mut Header) -> Result<usize, ParseError> {
    // Modifies header, returns bytes consumed
}

// Exception: reusing buffers for performance
pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    // This is OK - the buffer is owned by caller for reuse
}
```

**Rationale**: Rust's tuple and struct types are efficiently compiled. Out-parameters are less ergonomic and less idiomatic than returning compound values.

**See also**: C-NO-OUT guideline

---

## Operator Overloads Are Unsurprising

**Strength**: MUST

**Summary**: Only implement operator traits when the operation genuinely resembles the operator's mathematical or logical meaning.

**Example**:
```rust
use std::ops::{Add, Mul, BitOr};

// Good - clear mathematical meaning
#[derive(Copy, Clone)]
pub struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Add for Vector3 {
    type Output = Vector3;

    fn add(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// Good - BitOr for flag combination
bitflags! {
    struct Permissions: u32 {
        const READ = 0b001;
        const WRITE = 0b010;
        const EXECUTE = 0b100;
    }
}

let perms = Permissions::READ | Permissions::WRITE;  // Makes sense!

// Bad - surprising operator use
impl Add for HttpRequest {
    // Adding HTTP requests? What does that even mean?
}

impl Mul for Logger {
    // Multiplying loggers? Confusing!
}
```

**Rationale**: Operator overloading should follow the principle of least surprise. Operators come with strong mathematical and logical expectations.

**See also**: C-OVERLOAD guideline

---

## Only Smart Pointers Implement Deref

**Strength**: MUST

**Summary**: Deref and DerefMut should only be implemented for smart pointer types.

**Example**:
```rust
use std::ops::Deref;

// Good - Deref for smart pointers
impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T { /* ... */ }
}

impl<T> Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &T { /* ... */ }
}

impl Deref for String {
    type Target = str;
    fn deref(&self) -> &str { /* ... */ }
}

// Bad - Deref for non-smart-pointer types
struct User {
    name: String,
}

impl Deref for User {
    type Target = String;  // Don't do this!
    fn deref(&self) -> &String {
        &self.name
    }
}
```

**Rationale**: Deref is used implicitly by the compiler in many contexts. It's designed specifically for smart pointers to provide transparent access to the contained value.

**See also**: C-DEREF guideline, 11-anti-patterns.md (Deref polymorphism)

---

## Constructors Are Static Inherent Methods

**Strength**: SHOULD

**Summary**: Use static methods named `new`, `with_*`, or `from_*` for constructors.

**Example**:
```rust
// Good - primary constructor
impl<T> Vec<T> {
    pub fn new() -> Vec<T> { /* ... */ }
    pub fn with_capacity(capacity: usize) -> Vec<T> { /* ... */ }
}

// Good - domain-specific constructor names
impl File {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File> { /* ... */ }
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<File> { /* ... */ }
}

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> { /* ... */ }
}

// Good - conversion constructors
impl String {
    pub fn from_utf8(vec: Vec<u8>) -> Result<String, FromUtf8Error> { /* ... */ }
}

impl Error {
    pub fn from_raw_os_error(code: i32) -> Error { /* ... */ }
}

// Good - both new() and Default
impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
```

**Rationale**: Static inherent methods combine well with type name imports, creating concise and readable construction syntax.

**See also**: C-CTOR guideline, builder pattern

---

## Expose Intermediate Results

**Strength**: CONSIDER

**Summary**: When computing a result, expose useful intermediate values to avoid duplicate work.

**Example**:
```rust
// Good - returns index for insertion point if not found
pub fn binary_search<T>(vec: &[T], item: &T) -> Result<usize, usize>
where
    T: Ord,
{
    // Ok(index) if found
    // Err(index) where item should be inserted if not found
}

// Good - returns intermediate parsing info on error
pub struct FromUtf8Error {
    bytes: Vec<u8>,
    error: Utf8Error,
}

impl FromUtf8Error {
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes  // Return the original bytes
    }

    pub fn utf8_error(&self) -> Utf8Error {
        self.error  // Expose parsing details
    }
}

// Good - returns previous value
impl<K, V> HashMap<K, V> {
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        // Returns the old value if key existed
    }
}
```

**Rationale**: Exposing intermediate results prevents users from having to redo expensive computations and provides more context for error handling.

**See also**: C-INTERMEDIATE guideline

---

## Use Generics to Minimize Assumptions

**Strength**: SHOULD

**Summary**: Prefer generic type parameters with trait bounds over concrete types to maximize reusability.

**Example**:
```rust
use std::io::Write;

// Good - generic over any writer
pub fn write_header<W: Write>(writer: W, header: &Header) -> io::Result<()> {
    // Works with files, network sockets, in-memory buffers, etc.
}

// Bad - assumes specific type
pub fn write_header_bad(writer: &mut File, header: &Header) -> io::Result<()> {
    // Only works with File, can't use with Vec<u8> or TcpStream
}

// Good - generic over iteration
pub fn process<I>(items: I)
where
    I: IntoIterator<Item = Data>,
{
    for item in items {
        // Process item
    }
}

// Now works with Vec, slice, HashSet, custom collections, etc.

// Good - AsRef for path-like arguments
pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path = path.as_ref();
    // Works with &str, String, Path, PathBuf, etc.
}
```

**Rationale**: Generics enable reuse across multiple types while maintaining type safety and performance through monomorphization.

**See also**: C-GENERIC guideline

---

## Traits Should Be Object-Safe When Useful

**Strength**: SHOULD

**Summary**: If a trait might be used as a trait object, ensure it's object-safe. Use `where Self: Sized` to exclude specific methods from trait objects.

**Example**:
```rust
// Good - object-safe trait
pub trait Draw {
    fn draw(&self, canvas: &mut Canvas);
    fn bounds(&self) -> Rectangle;
}

// Can use as trait object
let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle { /* ... */ }),
    Box::new(Rectangle { /* ... */ }),
];

// Good - mixed object-safe and generic methods
pub trait Iterator {
    type Item;

    // Object-safe
    fn next(&mut self) -> Option<Self::Item>;

    // Not object-safe, but excluded from trait object
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        // ...
    }
}

// Bad - unnecessarily not object-safe
pub trait Process {
    fn process<T: Data>(&self, data: T);  // Generic method prevents trait objects
}
```

**Rationale**: Trait objects enable heterogeneous collections and dynamic dispatch. Making traits object-safe when appropriate provides flexibility.

**See also**: C-OBJECT guideline

---

## Newtypes Provide Static Distinctions

**Strength**: SHOULD

**Summary**: Use newtype pattern to create distinct types that prevent mixing up similar values.

**Example**:
```rust
// Good - newtype prevents mixing units
pub struct Celsius(pub f64);
pub struct Fahrenheit(pub f64);

impl Celsius {
    pub fn to_fahrenheit(self) -> Fahrenheit {
        Fahrenheit(self.0 * 9.0 / 5.0 + 32.0)
    }
}

// Type system prevents errors
fn boil_water(temp: Celsius) {
    if temp.0 >= 100.0 {
        println!("Water is boiling!");
    }
}

boil_water(Celsius(100.0));  // OK
// boil_water(Fahrenheit(212.0));  // Compile error!

// Good - newtype for validation
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        if email.contains('@') {
            Ok(EmailAddress(email))
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**Rationale**: Newtypes provide type safety at compile time with zero runtime cost, preventing entire classes of bugs.

**See also**: C-NEWTYPE guideline, 05-type-design.md

---

## Use Custom Types, Not bool or Option

**Strength**: SHOULD

**Summary**: Prefer explicit enums or structs over bool/Option parameters when the meaning isn't immediately obvious.

**Example**:
```rust
// Good - explicit types convey meaning
pub enum Size {
    Small,
    Medium,
    Large,
}

pub enum Shape {
    Round,
    Square,
}

pub fn create_widget(size: Size, shape: Shape) -> Widget {
    // Clear what each parameter means
}

let w = create_widget(Size::Large, Shape::Round);

// Bad - unclear booleans
pub fn create_widget_bad(large: bool, round: bool) -> Widget {
    // What does each bool mean? Have to check docs
}

let w = create_widget_bad(true, false);  // What does this mean?

// Good - enum instead of Option when meaning is specific
pub enum Validation {
    Strict,
    Lenient,
}

pub fn parse(input: &str, validation: Validation) -> Result<Data> { /* ... */ }

// Bad - Option doesn't convey the meaning clearly
pub fn parse_bad(input: &str, strict: Option<bool>) -> Result<Data> { /* ... */ }
```

**Rationale**: Custom types make code self-documenting and easier to extend. Adding a third size or shape is straightforward with enums.

**See also**: C-CUSTOM-TYPE guideline

---

## Use bitflags for Flag Sets

**Strength**: MUST

**Summary**: Use the `bitflags` crate for sets of boolean flags, not enums.

**Example**:
```rust
use bitflags::bitflags;

// Good - bitflags for multiple flags
bitflags! {
    pub struct OpenOptions: u32 {
        const READ = 0b0001;
        const WRITE = 0b0010;
        const CREATE = 0b0100;
        const TRUNCATE = 0b1000;
    }
}

pub fn open(path: &Path, options: OpenOptions) -> Result<File> {
    if options.contains(OpenOptions::READ) {
        // Handle read
    }
    if options.contains(OpenOptions::WRITE) {
        // Handle write
    }
    // ...
}

// Usage - combine flags with |
open(path, OpenOptions::READ | OpenOptions::WRITE)?;

// Bad - enum can only represent one choice
pub enum OpenOption {
    Read,
    Write,
    Create,
    Truncate,
}

// Can't represent "read AND write"
```

**Rationale**: Bitflags efficiently represent combinations of boolean flags and provide a type-safe, ergonomic API.

**See also**: C-BITFLAG guideline

---

## Builder Pattern for Complex Construction

**Strength**: CONSIDER

**Summary**: Use the builder pattern for types that have many optional configuration parameters.

**Example**:
```rust
// Good - builder for complex type
pub struct Server {
    host: String,
    port: u16,
    timeout: Duration,
    max_connections: usize,
}

pub struct ServerBuilder {
    host: String,
    port: u16,
    timeout: Option<Duration>,
    max_connections: Option<usize>,
}

impl ServerBuilder {
    pub fn new(host: impl Into<String>) -> Self {
        ServerBuilder {
            host: host.into(),
            port: 8080,
            timeout: None,
            max_connections: None,
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn build(self) -> Server {
        Server {
            host: self.host,
            port: self.port,
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            max_connections: self.max_connections.unwrap_or(100),
        }
    }
}

// Usage - clean chaining
let server = ServerBuilder::new("localhost")
    .port(3000)
    .timeout(Duration::from_secs(60))
    .build();
```

**Rationale**: Builders provide a clean API for constructing complex objects with many optional parameters and defaults.

**See also**: C-BUILDER guideline

---

## Validate Arguments

**Strength**: MUST

**Summary**: Functions should validate their inputs and enforce invariants.

**Example**:
```rust
// Good - static validation through types
pub struct NonZeroU32(u32);

impl NonZeroU32 {
    pub fn new(value: u32) -> Option<Self> {
        if value != 0 {
            Some(NonZeroU32(value))
        } else {
            None
        }
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

// Good - dynamic validation with clear errors
pub fn connect(addr: &str, port: u16) -> Result<Connection, ConnectError> {
    if port == 0 {
        return Err(ConnectError::InvalidPort);
    }

    if addr.is_empty() {
        return Err(ConnectError::InvalidAddress);
    }

    // Proceed with connection
}

// Good - opt-out validation with _unchecked
pub fn get(&self, index: usize) -> Option<&T> {
    if index < self.len() {
        Some(&self.items[index])
    } else {
        None
    }
}

pub unsafe fn get_unchecked(&self, index: usize) -> &T {
    // No bounds checking - caller must ensure validity
    &self.items[index]
}
```

**Rationale**: Rust enforces correctness. Validating inputs catches bugs early and prevents invalid states.

**See also**: C-VALIDATE guideline

---

## Destructors Never Fail

**Strength**: MUST

**Summary**: Drop implementations must not panic. Provide separate cleanup methods for fallible operations.

**Example**:
```rust
// Good - Drop never fails, separate close() for fallible cleanup
pub struct Connection {
    socket: TcpStream,
    closed: bool,
}

impl Connection {
    // Explicit close that can fail
    pub fn close(mut self) -> io::Result<()> {
        if !self.closed {
            self.socket.shutdown(Shutdown::Both)?;
            self.closed = true;
        }
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if !self.closed {
            // Best-effort cleanup, ignore errors
            let _ = self.socket.shutdown(Shutdown::Both);
        }
    }
}

// Bad - Drop that can panic
impl Drop for BadConnection {
    fn drop(&mut self) {
        self.socket.shutdown(Shutdown::Both).unwrap();  // Can panic!
    }
}
```

**Rationale**: Drop is called during panicking. A failing destructor during panic causes the program to abort.

**See also**: C-DTOR-FAIL guideline

---

## All Public Types Implement Debug

**Strength**: MUST

**Summary**: Every public type should implement Debug for debugging and error reporting.

**Example**:
```rust
// Good - derived Debug
#[derive(Debug)]
pub struct User {
    pub name: String,
    pub email: String,
}

// Good - custom Debug for sensitive data
pub struct Credentials {
    username: String,
    password: String,
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

// Good - Debug representation is never empty
let empty: Vec<i32> = vec![];
assert_eq!(format!("{:?}", empty), "[]");

let empty_str = "";
assert_eq!(format!("{:?}", empty_str), "\"\"");
```

**Rationale**: Debug is essential for error messages, logging, and development. It should never be omitted from public types.

**See also**: C-DEBUG, C-DEBUG-NONEMPTY guidelines

---

## Summary

These API design guidelines ensure Rust crates are:
- **Consistent** with ecosystem conventions
- **Interoperable** through common traits
- **Ergonomic** with method syntax and smart generics
- **Type-safe** using newtypes and custom types
- **Flexible** supporting diverse use cases
- **Debuggable** with comprehensive Debug impls
- **Future-proof** with sealed traits and private fields

Cross-references:
- 01-core-idioms.md (foundational patterns)
- 03-error-handling.md (error design)
- 04-ownership-borrowing.md (ownership patterns)
- 05-type-design.md (advanced type patterns)
- 06-traits.md (trait design)
- 11-anti-patterns.md (what to avoid)
