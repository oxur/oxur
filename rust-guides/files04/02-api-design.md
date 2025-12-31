# API Design Guidelines

Guidelines for designing public Rust APIs that are ergonomic, composable, and maintainable.

## Table of Contents

- [Builder Patterns](#builder-patterns)
- [Type Simplicity](#type-simplicity)
- [Smart Pointers](#smart-pointers)
- [Accept Traits](#accept-traits)
- [Service Design](#service-design)
- [Sans-IO Pattern](#sans-io-pattern)
- [Essential Functionality](#essential-functionality)

---

## Builder Patterns

### Complex Type Construction Has Builders

**Strength**: SHOULD

**Summary**: Types with 4+ optional initialization parameters should provide builders instead of multiple constructors.

**Example**:
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

**Builder conventions**:
- Builder for `Foo` is named `FooBuilder`
- Methods are chainable, final method is `.build()`
- Shortcut: `Foo::builder()` (not `FooBuilder::new()`)
- Method to set `x` is called `.x()`, not `.set_x()`

**See also**: M-INIT-BUILDER, dependency injection pattern

---

### Builders with Required Parameters

**Strength**: SHOULD

**Summary**: Required parameters should be passed when creating the builder, not as setter methods.

**Example**:
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

### Cascaded Initialization for Complex Hierarchies

**Strength**: SHOULD

**Summary**: Types requiring 4+ parameters should cascade initialization through helper types rather than long parameter lists.

**Example**:
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

## Type Simplicity

### Abstractions Don't Visibly Nest

**Strength**: MUST

**Summary**: Avoid exposing nested or complex parameterized types in primary API surface.

**Example**:
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

**When type parameters are acceptable**:
- Containers (List<T>, Map<K, V>)
- Utility types not expected to be named (iterators)
- When parameters don't have complex bounds
- When they don't affect inference in other functions

**See also**: M-SIMPLE-ABSTRACTIONS, M-ABSTRACTIONS-DONT-NEST

---

### Avoid Smart Pointers in APIs

**Strength**: MUST

**Summary**: Don't expose Arc<T>, Rc<T>, Box<T>, or RefCell<T> in public APIs—use clean interfaces with &T, &mut T, or T.

**Example**:
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

**Acceptable cases**:
- The smart pointer is fundamental to the API's purpose (new container type)
- Significant proven performance benefit
- Internal use (never in public signatures)

**See also**: M-AVOID-WRAPPERS

---

### Prefer Concrete Types over Generics

**Strength**: SHOULD

**Summary**: When designing dependencies, prefer concrete types > generics > dyn Trait.

**Example**:
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

**Escalation ladder**:
1. Start with concrete types
2. If multiple implementations needed for testing, use enum (see mockable pattern)
3. If users need custom implementations, introduce narrow traits
4. Accept traits as generic bounds: `fn process(db: impl StoreObject)`
5. Only if generics cause nesting problems, use trait objects with custom wrapper

**See also**: M-DI-HIERARCHY, M-MOCKABLE-SYSCALLS

---

## Accept Traits

### Accept impl AsRef<> Where Feasible

**Strength**: SHOULD

**Summary**: Function parameters should accept `impl AsRef<T>` for types with clear reference hierarchies.

**Example**:
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

**Common patterns**:

| Instead of... | Use... |
|---------------|--------|
| `&str`, `String` | `impl AsRef<str>` |
| `&Path`, `PathBuf` | `impl AsRef<Path>` |
| `&[u8]`, `Vec<u8>` | `impl AsRef<[u8]>` |

**When NOT to use**:
- Function takes ownership and the type is "low frequency, low volume"
- Type fields (use owned types like `String`, `PathBuf`)

**See also**: M-IMPL-ASREF

---

### Accept impl RangeBounds<> Where Feasible

**Strength**: MUST

**Summary**: Functions accepting ranges must use `RangeBounds<T>` or `Range<T>`, not separate low/high parameters.

**Example**:
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

## Service Design

### Services Are Cloneable

**Strength**: MUST

**Summary**: Service types and thread singletons must implement shared-ownership `Clone` semantics using the Arc<Inner> pattern.

**Example**:
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

## Sans-IO Pattern

### Accept impl IO Traits Where Feasible

**Strength**: SHOULD

**Summary**: Functions needing one-shot I/O should accept trait objects (`impl Read`, `impl AsyncRead`) rather than concrete types.

**Example**:
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

**Common I/O traits**:
- Sync: `std::io::Read`, `std::io::Write`
- Async (runtime-agnostic): `futures::io::AsyncRead`, `futures::io::AsyncWrite`

**When NOT to use**:
- Type needs runtime-specific continuous I/O (use runtime abstraction pattern)
- Type manages long-lived I/O state

**See also**: M-IMPL-IO, sans-io pattern

---

## Essential Functionality

### Essential Functionality Should Be Inherent

**Strength**: MUST

**Summary**: Core functionality must be implemented as inherent methods; trait implementations should forward to inherent methods.

**Example**:
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
