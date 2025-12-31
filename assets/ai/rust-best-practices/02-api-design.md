# API Design Guidelines

Comprehensive guidelines for designing public APIs in Rust, based on the official Rust API Guidelines from the Rust library team.

---


## API-01: Abstractions Don't Visibly Nest

**Strength**: MUST

**Summary**: Avoid exposing nested or complex parameterized types in primary API surface.

```rust
// Bad - excessive nesting and parameters
struct Matrix<T, const R: usize, const C: usize, S: Storage<T, R, C>> {
    data: S,
    _phantom: PhantomData<T>,
}

pub struct App {
    // Users must name this complex type!
    matrix: Matrix<f32, 4, 4, ArrayStorage<f32, 4, 4>>,
}

// Good - hide complexity
pub struct Matrix4x4 {
    // Implementation uses generics internally
    inner: MatrixImpl<f32, 4, 4>,
}

pub struct App {
    matrix: Matrix4x4,  // Simple!
}

// Good - container allows user-provided nesting
pub struct List<T> {
    items: Vec<T>,
}

pub struct App {
    // User chose this nesting, it's not forced by our API
    lists: List<Rc<RefCell<Item>>>,
}
```

**Rationale**: Type parameters create cognitive load, especially when nested. Service-level types should not nest on their own volition. Limit nesting to 1 level deep for primary APIs.

**See also**: M-SIMPLE-ABSTRACTIONS, M-ABSTRACTIONS-DONT-NEST

---

## API-02: Accept Borrowed, Return Owned

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

---

## API-03: Accept impl AsRef<> Where Feasible

**Strength**: SHOULD

**Summary**: Function parameters should accept `impl AsRef<T>` for types with clear reference hierarchies.

```rust
// Bad - forces caller to convert
pub fn read_file(path: &Path) -> Result<String, Error> {
    // Caller must call .as_ref() on String
}

pub fn print_message(msg: &str) {
    // Caller must call .as_ref() on String
}

// Good - accept AsRef
pub fn read_file(path: impl AsRef<Path>) -> Result<String, Error> {
    let path = path.as_ref();
    std::fs::read_to_string(path)
}

pub fn print_message(msg: impl AsRef<str>) {
    println!("{}", msg.as_ref());
}

// Usage - works with both owned and borrowed
read_file("config.toml");           // &str
read_file(String::from("data.json")); // String
read_file(&PathBuf::from("./file")); // &PathBuf

// Don't infect struct fields with AsRef
pub struct Config {
    // Bad
    path: impl AsRef<Path>,  // Doesn't work!
    
    // Good
    path: PathBuf,  // Store owned data
}
```

**Rationale**: `impl AsRef<T>` provides ergonomics without runtime cost. The compiler monomorphizes each version, so there's no dynamic dispatch.

**See also**: M-IMPL-ASREF

---

## API-04: Accept impl IO Traits Where Feasible

**Strength**: SHOULD

**Summary**: Functions needing one-shot I/O should accept trait objects (`impl Read`, `impl AsyncRead`) rather than concrete types.

```rust
use std::io::Read;

// Bad - forces file I/O even if data is in memory
pub fn parse_config(file: File) -> Result<Config, Error> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    // ...
}

// Good - accepts any Read source
pub fn parse_config(mut source: impl Read) -> Result<Config, Error> {
    let mut content = String::new();
    source.read_to_string(&mut content)?;
    // ...
}

// Works with:
parse_config(File::open("config.toml")?);       // File
parse_config(TcpStream::connect("...")?);        // Network
parse_config(std::io::stdin());                  // Stdin  
parse_config(&b"key=value"[..]);                 // In-memory bytes
parse_config(std::io::Cursor::new(vec));         // Cursor

// Async version
use futures::io::AsyncRead;

pub async fn parse_config_async(
    mut source: impl AsyncRead + Unpin
) -> Result<Config, Error> {
    let mut content = String::new();
    source.read_to_string(&mut content).await?;
    // ...
}
```

**Rationale**: Separating business logic from I/O provides N×M composability—any parser works with any I/O source. Makes testing easier (use in-memory data) and supports more use cases without code changes.

**See also**: M-IMPL-IO, sans-io pattern

---

## API-05: Accept impl RangeBounds<> Where Feasible

**Strength**: MUST

**Summary**: Functions accepting ranges must use `RangeBounds<T>` or `Range<T>`, not separate low/high parameters.

```rust
// Bad - hand-rolled range parameters
pub fn select_range(low: usize, high: usize) { /* ... */ }
pub fn select_range(range: (usize, usize)) { /* ... */ }

// Acceptable - forces specific syntax
pub fn select_range(range: Range<usize>) { /* ... */ }
// Caller must use: select_range(1..3)

// Best - accepts any range type
use std::ops::RangeBounds;

pub fn select_any(range: impl RangeBounds<usize>) { /* ... */ }

// Caller can use any of:
select_any(1..3);      // Range
select_any(1..=3);     // RangeInclusive
select_any(1..);       // RangeFrom
select_any(..3);       // RangeTo
select_any(..);        // RangeFull
```

**Rationale**: Rust's range types are expressive and standard. Using `RangeBounds` provides maximum flexibility while maintaining clear semantics.

**See also**: M-IMPL-RANGEBOUNDS

---

## API-06: All Public Types Implement Debug

**Strength**: MUST

**Summary**: Every public type should implement Debug for debugging and error reporting.

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

## API-07: Avoid Smart Pointers in APIs

**Strength**: MUST

**Summary**: Don't expose Arc<T>, Rc<T>, Box<T>, or RefCell<T> in public APIs—use clean interfaces with &T, &mut T, or T.

```rust
// Bad - exposing implementation details
pub fn process_shared(data: Arc<Mutex<Shared>>) -> Box<Processed> {
    // Forces all callers to use Arc<Mutex<>>!
}

pub fn initialize(config: Rc<RefCell<Config>>) -> Arc<Server> {
    // Infectious complexity
}

// Good - simple API boundaries
pub fn process_data(data: &Data) -> Processed {
    // Internally we might use Arc, but don't expose it
}

pub fn store_config(config: Config) -> Result<(), Error> {
    // Clean ownership transfer
}

// If internal Arc is needed, hide it
pub struct Server {
    inner: Arc<ServerInner>,  // Hidden implementation detail
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(ServerInner::new(config))
        }
    }
    
    // Clean methods, Arc is invisible
    pub fn start(&self) -> Result<(), Error> {
        self.inner.start()
    }
}

// Clone gives a new handle, not a deep copy
impl Clone for Server {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner)
        }
    }
}
```

**Rationale**: Smart pointers are implementation details that create friction for callers. Multiple crates disagreeing about wrapper types makes composition impossible. Hide wrappers behind clean APIs.

**See also**: M-AVOID-WRAPPERS

---

## API-08: Avoid Stringly-Typed APIs

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

## API-09: Binary Number Types Provide Formatting Traits

**Strength**: SHOULD

**Summary**: Implement `Binary`, `Octal`, `LowerHex`, `UpperHex` for types representing binary data or bitflags.

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

**Rationale**: Binary number formatting is essential for debugging bit-level operations but inappropriate for semantic numeric types.

---

---

## API-10: Builder Pattern for Complex Construction

**Strength**: CONSIDER

**Summary**: Use the builder pattern for types that have many optional configuration parameters.

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

## API-11: Builders with Required Parameters

**Strength**: SHOULD

**Summary**: Required parameters should be passed when creating the builder, not as setter methods.

```rust
// Bad - required params as setters
impl ConfigBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn logger(mut self, logger: Logger) -> Self { /* required! */ }
    pub fn timeout(mut self, timeout: Duration) -> Self { /* optional */ }
}

// Good - required params in builder constructor
#[derive(Debug, Clone)]
pub struct ConfigDeps {
    pub logger: Logger,
    pub config_path: PathBuf,
}

// Support convenient conversions
impl From<Logger> for ConfigDeps {
    fn from(logger: Logger) -> Self {
        Self {
            logger,
            config_path: PathBuf::from("config.toml"),
        }
    }
}

impl From<(Logger, PathBuf)> for ConfigDeps {
    fn from((logger, config_path): (Logger, PathBuf)) -> Self {
        Self { logger, config_path }
    }
}

impl Config {
    pub fn builder(deps: impl Into<ConfigDeps>) -> ConfigBuilder {
        let deps = deps.into();
        ConfigBuilder::new(deps)
    }
}

// Usage - multiple ergonomic options
let cfg1 = Config::builder(logger).build();
let cfg2 = Config::builder((logger, path)).build();
let cfg3 = Config::builder(ConfigDeps { logger, config_path }).build();
```

**Rationale**: Required parameters in constructors prevent incomplete builders at compile time. The `impl Into<Deps>` pattern provides ergonomics while maintaining type safety.

**See also**: M-INIT-BUILDER, fundle crate

---

## API-12: Cascaded Initialization for Complex Hierarchies

**Strength**: SHOULD

**Summary**: Types requiring 4+ parameters should cascade initialization through helper types rather than long parameter lists.

```rust
// Bad - primitive obsession and long parameter list
impl Deposit {
    pub fn new(
        bank_name: &str,
        customer_name: &str,
        currency_name: &str,
        currency_amount: u64,
    ) -> Self {
        // Easy to confuse parameters!
    }
}

// Good - cascaded through strong types
pub struct Account {
    bank: Bank,
    customer: Customer,
}

pub struct Currency {
    name: String,
    amount: u64,
}

impl Deposit {
    pub fn new(account: Account, amount: Currency) -> Self {
        Self { account, amount }
    }
}

impl Account {
    pub fn new(bank: Bank, customer: Customer) -> Self {
        Self { bank, customer }
    }
}
```

**Rationale**: Grouping related parameters semantically prevents parameter confusion and creates reusable types. Combines well with the newtype pattern.

**See also**: M-INIT-CASCADED, C-NEWTYPE

---

## API-13: Collections Implement FromIterator and Extend

**Strength**: MUST

**Summary**: Collection types should support construction from and extension by iterators. Collection types should implement FromIterator for construction and Extend for adding items.

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

**Rationale**: These traits enable collections to work seamlessly with iterator methods like `collect()`, `partition()`, and `unzip()`, making them first-class citizens in the Rust ecosystem. These traits enable collections to work seamlessly with iterators, unlocking powerful composition patterns.

**See also**: C-COLLECT guideline

---

## API-14: Complex Type Construction Has Builders

**Strength**: SHOULD

**Summary**: Types with 4+ optional initialization parameters should provide builders instead of multiple constructors.

```rust
// Bad - too many permutations
impl Config {
    pub fn new() -> Self { /* ... */ }
    pub fn with_a(a: A) -> Self { /* ... */ }
    pub fn with_b(b: B) -> Self { /* ... */ }
    pub fn with_a_b(a: A, b: B) -> Self { /* ... */ }
    pub fn with_a_c(a: A, c: C) -> Self { /* ... */ }
    // 16 permutations for 4 optional params!
}

// Good - builder pattern
impl Config {
    pub fn new() -> Self { /* minimal config */ }
    pub fn builder() -> ConfigBuilder { 
        ConfigBuilder::default() 
    }
}

impl ConfigBuilder {
    pub fn a(mut self, a: A) -> Self { 
        self.a = Some(a); 
        self 
    }
    
    pub fn b(mut self, b: B) -> Self { 
        self.b = Some(b); 
        self 
    }
    
    pub fn c(mut self, c: C) -> Self { 
        self.c = Some(c); 
        self 
    }
    
    pub fn build(self) -> Config {
        Config {
            a: self.a,
            b: self.b,
            c: self.c,
        }
    }
}

// Usage
let config = Config::builder()
    .a(value_a)
    .c(value_c)
    .build();
```

**Rationale**: Builders prevent combinatorial explosion of constructors and provide clear, chainable initialization. The threshold is 4+ optional parameters (types with 2 optional params can use inherent methods).

**See also**: M-INIT-BUILDER, dependency injection pattern

---

## API-15: Constructors Are Static Inherent Methods

**Strength**: MUST

**Summary**: Constructors should be static inherent methods named `new` or following specific patterns. Use static methods named `new`, `with_*`, or `from_*` for constructors.

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

**Rationale**: Static constructors work well with Rust's type imports and provide clear, discoverable API surface. Static inherent methods combine well with type name imports, creating concise and readable construction syntax.

**See also**: C-CTOR, C-COMMON-TRAITS, C-CTOR guideline, builder pattern

---

## API-16: Conversion Method Naming (as_, to_, into_)

**Strength**: MUST

**Summary**: Use `as_` for free borrowed-to-borrowed, `to_` for expensive conversions, and `into_` for consuming conversions.

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

## API-17: Conversions Live on the Most Specific Type

**Strength**: SHOULD

**Summary**: Place conversion methods on the more specific type in the conversion pair. Place conversion methods on the more specific of the two types involved.

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

**Rationale**: This prevents cluttering general types with specialized conversions while keeping related conversions discoverable on the type that provides the guarantees. Keeping conversions on the specific type avoids polluting general types with countless conversion methods and makes the API more discoverable.

**See also**: C-CONV-SPECIFIC, C-CONV-SPECIFIC guideline

---

## API-18: Conversions Use Standard Traits

**Strength**: MUST

**Summary**: Use `From`, `TryFrom`, `AsRef`, `AsMut` traits for conversions, never implement `Into` or `TryInto` directly.

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

**Rationale**: `From` and `TryFrom` provide blanket implementations of `Into` and `TryInto` automatically. Implementing the latter directly would be redundant and could cause confusion.

**See also**: C-CONV-TRAITS, C-CONV for method naming conventions

---

## API-19: Data Structures Implement Serde Traits

**Strength**: SHOULD

**Summary**: Types representing data structures should implement `Serialize` and `Deserialize`, typically behind a feature flag.

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

**Rationale**: Serde is the de facto standard for serialization in Rust. Optional implementation allows users who don't need serialization to avoid the compile-time cost.

---

---

## API-20: Destructors Never Fail

**Strength**: MUST

**Summary**: Drop implementations must not panic. Provide separate cleanup methods for fallible operations.

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

## API-21: Document Public APIs

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

---

## API-22: Error Types Are Meaningful

**Strength**: MUST

**Summary**: Error types must implement Error, Send, Sync and should never be `()`. Error messages should be lowercase without trailing punctuation.

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

## API-23: Error Types Are Meaningful and Well-Behaved

**Strength**: MUST

**Summary**: Error types must implement `Error`, `Send`, `Sync`, and have good `Display` messages.

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

**Rationale**: Well-behaved errors integrate with error handling libraries, work across threads, can be used in `async` contexts, and provide good developer experience.

**See also**: C-GOOD-ERR

---

## API-24: Error Types for Public APIs

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

## API-25: Essential Functionality Should Be Inherent

**Strength**: MUST

**Summary**: Core functionality must be implemented as inherent methods; trait implementations should forward to inherent methods.

```rust
// Bad - essential functionality only in trait
pub struct HttpClient {
    client: reqwest::Client,
}

trait Download {
    fn download_file(&self, url: &str) -> Result<Vec<u8>, Error>;
}

impl Download for HttpClient {
    fn download_file(&self, url: &str) -> Result<Vec<u8>, Error> {
        // Core logic here - not discoverable!
    }
}

// Users must know to import the trait
use crate::Download; // Not obvious!
client.download_file(url);

// Good - inherent methods for core functionality
impl HttpClient {
    pub fn new() -> Self { /* ... */ }
    
    // Core functionality is inherent - easily discoverable
    pub fn download_file(&self, url: &str) -> Result<Vec<u8>, Error> {
        // Core logic here
    }
}

// Trait forwards to inherent impl
impl Download for HttpClient {
    fn download_file(&self, url: &str) -> Result<Vec<u8>, Error> {
        Self::download_file(self, url)
    }
}

// Users can use directly without trait imports
let client = HttpClient::new();
client.download_file(url); // Just works!
```

**Rationale**: Inherent methods appear in IDE autocomplete and don't require trait imports. Users can discover functionality naturally. Traits should extend types, not replace their core API.

**See also**: M-ESSENTIAL-FN-INHERENT, C-METHOD

---

## API-26: Expose Intermediate Results

**Strength**: CONSIDER

**Summary**: When computing a result, expose useful intermediate values to avoid duplicate work.

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

## API-27: Extension Traits for Foreign Types

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

---

## API-28: Functions Do Not Take Out-Parameters

**Strength**: MUST

**Summary**: Return multiple values as tuples or structs, not via mutable out-parameters.

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

## API-29: Functions Take R: Read and W: Write by Value

**Strength**: SHOULD

**Summary**: Generic reader/writer functions should take by value, leveraging the blanket impls that allow passing `&mut` references.

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

## API-30: Functions With Clear Receiver Are Methods

**Strength**: SHOULD

**Summary**: If a function clearly operates on a specific type, make it a method rather than a free function.

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

**Rationale**: Methods provide better ergonomics, discoverability, and clarity about ownership.

**See also**: C-METHOD

---

## API-31: Functions with Clear Receivers Are Methods

**Strength**: SHOULD

**Summary**: When a function operates on a specific type, make it a method rather than a free function.

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

## API-32: Generic Reader/Writer Functions Take by Value

**Strength**: MUST

**Summary**: Functions accepting `R: Read` or `W: Write` should take them by value, not by reference.

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

## API-33: Getter Naming

**Strength**: SHOULD

**Summary**: Omit the `get_` prefix for getters unless there's a compelling reason.

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

## API-34: Implement Common Traits Eagerly

**Strength**: MUST

**Summary**: Implement Copy, Clone, Debug, Default, and other common traits for public types whenever semantically appropriate.

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

## API-35: Implement Standard Traits

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

---

## API-36: Iterator Method Naming

**Strength**: MUST

**Summary**: Use `iter()`, `iter_mut()`, and `into_iter()` for iterator-producing methods on collections.

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

## API-37: Method Receiver Guidelines

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

---

## API-38: Naming Conventions

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

---

## API-39: Naming Conventions (RFC 430)

**Strength**: MUST

**Summary**: Follow Rust's established naming conventions for all public items.

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

## API-40: Newtypes Provide Static Distinctions

**Strength**: SHOULD

**Summary**: Use newtype pattern to create distinct types that prevent mixing up similar values.

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

## API-41: No Out-Parameters

**Strength**: MUST

**Summary**: Return multiple values via tuples or structs, not out-parameters.

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

## API-42: Only Smart Pointers Implement Deref

**Strength**: MUST

**Summary**: `Deref` and `DerefMut` should only be implemented for smart pointer types. Deref and DerefMut should only be implemented for smart pointer types.

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

**See also**: C-DEREF, C-DEREF guideline, 11-anti-patterns.md (Deref polymorphism)

---

## API-43: Operator Overloads Are Unsurprising

**Strength**: MUST

**Summary**: Only implement operator traits when the operation genuinely resembles the operator's mathematical or logical meaning.

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

## API-44: Prefer Concrete Types over Generics

**Strength**: SHOULD

**Summary**: When designing dependencies, prefer concrete types > generics > dyn Trait.

```rust
// Bad - dyn Trait in API
pub async fn start_service(db: Rc<dyn Database>) {
    // Problems: not object-safe, requires wrapper, infectious
}

// Bad - trait with generic not needed
trait Database {
    async fn store(&self, obj: Object);  // Not object-safe!
}

// Good - concrete type with inherent methods
pub struct MyDatabase {
    connection: Connection,
}

impl MyDatabase {
    pub fn new(url: &str) -> Self { /* ... */ }
    
    pub async fn store(&self, obj: Object) {
        // Direct implementation
    }
    
    pub async fn load(&self, id: Id) -> Option<Object> {
        // Direct implementation  
    }
}

// Usage
async fn start_service(db: MyDatabase) {
    // Simple and direct!
}

// If trait abstraction is needed, provide narrow traits
pub trait StoreObject {
    fn store(&self, obj: Object) -> impl Future<Output = ()>;
}

pub trait LoadObject {
    fn load(&self, id: Id) -> impl Future<Output = Option<Object>>;
}

impl StoreObject for MyDatabase { /* ... */ }
impl LoadObject for MyDatabase { /* ... */ }

// Use generics when needed, not dyn
async fn process<D: StoreObject + LoadObject>(db: D) {
    // Works with any implementation
}
```

**Rationale**: Concrete types are easier to use and understand. Generics are better than trait objects for flexibility. Use trait objects only when generics cause excessive nesting.

**See also**: M-DI-HIERARCHY, M-MOCKABLE-SYSCALLS

---

## API-45: Provide `Default` Implementations

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

## API-46: Sealed Traits for Extensible-but-Closed APIs

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

---

## API-47: Services Are Cloneable

**Strength**: MUST

**Summary**: Service types and thread singletons must implement shared-ownership `Clone` semantics using the Arc<Inner> pattern.

```rust
// Internal state
struct ServiceInner {
    config: Config,
    connection: Connection,
}

// Public service
#[derive(Clone)]
pub struct Service {
    inner: Arc<ServiceInner>,
}

impl Service {
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(ServiceInner {
                config,
                connection: Connection::new(),
            })
        }
    }
    
    // Methods forward to inner
    pub fn process(&self, data: &Data) -> Result<(), Error> {
        self.inner.process(data)
    }
}

// Usage - can be freely cloned and shared
struct App {
    service_a: ServiceA,
    service_b: ServiceB,
}

impl App {
    fn init() -> Self {
        let common = CommonService::new();
        
        // Both services get a clone (cheap Arc clone)
        let service_a = ServiceA::new(&common);
        let service_b = ServiceB::new(&common);
        
        Self { service_a, service_b }
    }
}

impl ServiceA {
    pub fn new(common: &CommonService) -> Self {
        Self {
            common: common.clone(), // Cheap!
        }
    }
}
```

**Rationale**: Services are typically heavyweight and shared. Clone semantics let them be passed around ergonomically without actual duplication. The Arc is hidden from users.

**See also**: M-SERVICES-CLONE

---

## API-48: Smart Pointers Do Not Add Inherent Methods

**Strength**: MUST

**Summary**: Smart pointer types should not have inherent methods (except constructors) that could be confused with methods on the inner type.

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

## API-49: Smart Pointers Don't Add Inherent Methods

**Strength**: MUST

**Summary**: Smart pointer types should not have inherent methods that could be confused with methods on the pointed-to type.

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

## API-50: Traits Should Be Object-Safe When Useful

**Strength**: SHOULD

**Summary**: If a trait might be used as a trait object, ensure it's object-safe. Use `where Self: Sized` to exclude specific methods from trait objects.

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

## API-51: Types Are Send and Sync Where Possible

**Strength**: MUST

**Summary**: Types should be `Send` and `Sync` unless they genuinely can't be safely used across threads.

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

**Rationale**: `Send` and `Sync` are fundamental to Rust's concurrency story. Types that can't be sent across threads or shared between threads are severely limited in their usefulness.

**See also**: C-SEND-SYNC

---

## API-52: Types Eagerly Implement Common Traits

**Strength**: MUST

**Summary**: Implement all applicable common traits from `std` to maximize interoperability with the ecosystem.

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

**See also**: C-COMMON-TRAITS in API Guidelines, C-DEBUG for Debug implementation requirements, C-SEND-SYNC for thread safety traits

---

## API-53: Use `impl Trait` in Return Position Judiciously

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

---

## API-54: Use bitflags for Flag Sets

**Strength**: MUST

**Summary**: Use the `bitflags` crate for sets of boolean flags, not enums.

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

## API-55: Use Custom Types, Not bool or Option

**Strength**: SHOULD

**Summary**: Prefer explicit enums or structs over bool/Option parameters when the meaning isn't immediately obvious.

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

## API-56: Use Generics to Minimize Assumptions

**Strength**: SHOULD

**Summary**: Prefer generic type parameters with trait bounds over concrete types to maximize reusability.

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

## API-57: Use Standard Conversion Traits

**Strength**: MUST

**Summary**: Implement From/TryFrom, AsRef/AsMut for conversions. Never implement Into/TryInto directly.

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

## API-58: Validate Arguments

**Strength**: MUST

**Summary**: Functions should validate their inputs and enforce invariants.

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

## API-59: Validate Early, Fail Fast

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

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| Builders for 4+ optional params | SHOULD | Prevent constructor explosion |
| Required params in builder constructor | SHOULD | Compile-time completeness checking |
| Cascaded initialization | SHOULD | Group related parameters |
| Don't nest abstractions | MUST | Keep type signatures simple |
| Avoid smart pointers in APIs | MUST | Hide implementation details |
| Concrete > Generic > dyn | SHOULD | Escalate abstraction as needed |
| Accept impl AsRef | SHOULD | Ergonomic without cost |
| Accept impl RangeBounds | MUST | Use standard range types |
| Services are cloneable | MUST | Arc<Inner> pattern |
| Accept impl Read/Write | SHOULD | Sans-IO for composability |
| Essential methods inherent | MUST | Discoverability |

## Related Guidelines

- **Type Design**: See `05-type-design.md` for newtype pattern
- **Error Handling**: See `03-error-handling.md` for Result patterns
- **Testing**: See `04-ownership-borrowing.md` for mockable patterns

## External References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Pragmatic Rust: M-INIT-BUILDER, M-SIMPLE-ABSTRACTIONS, M-SERVICES-CLONE