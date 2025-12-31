# Type Design

Guidelines for designing structs, enums, newtypes, and using generics effectively.

## Newtypes

### Newtypes Provide Static Distinctions

**Strength**: SHOULD

**Summary**: Use newtype pattern to create distinct types that prevent mixing incompatible values.

**Examples**:

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

**Zero-cost abstraction**: Newtypes have no runtime overhead - they compile down to the underlying type.

**Rationale**: Newtypes use the type system to prevent bugs at compile time rather than runtime, with zero performance cost.

**See also**: C-NEWTYPE

---

### Arguments Use Types Not Bool or Option

**Strength**: SHOULD

**Summary**: Use custom types instead of `bool` or `Option` to convey meaning clearly.

**Examples**:

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

**When bool is OK**:
- The meaning is completely obvious from the parameter name
- There's only one parameter
- It's a well-established convention

```rust
// OK - meaning is crystal clear
fn set_visible(visible: bool) { }
fn is_empty() -> bool { }
```

**Rationale**: Explicit types make code self-documenting and prevent passing arguments in the wrong order.

**See also**: C-CUSTOM-TYPE

---

### Use bitflags for Flag Sets

**Strength**: SHOULD

**Summary**: For a set of boolean flags, use the `bitflags` crate instead of enums or multiple bools.

**Examples**:

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

**When to use bitflags**:
- ✅ Multiple boolean properties that can be combined
- ✅ System flags (file permissions, feature flags)
- ✅ Configuration options
- ✅ Hardware register bits

**When NOT to use bitflags**:
- ❌ Mutually exclusive options (use enum)
- ❌ Non-boolean state (use enum with data)
- ❌ Single flag (use bool)

**Rationale**: Bitflags provide efficient storage, intuitive combination syntax, and are a well-established pattern.

**See also**: C-BITFLAG

---

### Builder Pattern for Complex Construction

**Strength**: SHOULD

**Summary**: Use the builder pattern when types have many optional parameters or complex construction requirements.

**Examples**:

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

**Builder method patterns**:

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

## Struct Design

### Structs Have Private Fields

**Strength**: MUST

**Summary**: Make struct fields private by default; provide accessor methods for controlled access.

**Examples**:

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

**Benefits of private fields**:
1. Can change internal representation without breaking API
2. Can add validation when setting fields
3. Can maintain invariants
4. Can compute derived values
5. Can add logging/tracing when fields change

**When public fields are OK**:
- Pure data structures (coordinates, colors)
- Configuration objects without invariants
- Interop with C (repr(C) structs)

**Rationale**: Private fields enable evolution of the type without breaking changes and allow maintaining invariants.

**See also**: C-STRUCT-PRIVATE

---

### Newtypes Hide Implementation Details

**Strength**: SHOULD

**Summary**: Use newtypes to hide complex implementation types from public API.

**Examples**:

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

### Structs Don't Duplicate Derived Trait Bounds

**Strength**: MUST

**Summary**: Don't add trait bounds to struct definitions that are only needed by derived implementations.

**Examples**:

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

**Traits to never use as struct bounds**:
- `Clone`
- `Copy` (though sometimes needed for specific algorithms)
- `Debug`
- `Display`
- `PartialEq` / `Eq`
- `PartialOrd` / `Ord`
- `Hash`
- `Default`
- `Serialize` / `Deserialize`

**Traits that might be legitimate struct bounds**:
- Trait bounds that express semantic requirements
- Bounds needed to ensure type invariants
- `Send` / `Sync` when required for safety

**Rationale**: Unnecessary bounds are a backward compatibility hazard and limit type usage unnecessarily.

**See also**: C-STRUCT-BOUNDS

---

## Generic Types

### Generics Minimize Assumptions

**Strength**: SHOULD

**Summary**: Use generic parameters to minimize assumptions about types, making functions more reusable.

**Examples**:

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

**Benefits of generics**:
1. **Reusability** - Works with many types
2. **Static dispatch** - No runtime cost
3. **Type safety** - Compiler checks constraints
4. **Optimization** - Specialized for each use
5. **Inline layout** - Generic struct fields laid out inline

**Drawbacks of generics**:
1. **Code size** - Monomorphization duplicates code
2. **Compile time** - More code to compile
3. **Signature complexity** - Can be verbose
4. **Binary size** - Each instantiation adds code

**When to use generics**:
- ✅ Maximum flexibility needed
- ✅ Performance critical code
- ✅ Library code
- ✅ Small functions (inlining benefit)

**When to consider trait objects instead**:
- ❌ Binary size is critical
- ❌ Many concrete types will be used
- ❌ Heterogeneous collections needed

**Rationale**: Generics provide maximum flexibility and performance through static dispatch and monomorphization.

**See also**: C-GENERIC

---

### Traits Are Object-Safe When Useful

**Strength**: SHOULD

**Summary**: If a trait might reasonably be used as a trait object, design it to be object-safe.

**Examples**:

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

**Object-safety rules**:
A trait is object-safe if:
1. All methods return `Self` by value are where `Self: Sized`
2. No method has generic type parameters
3. No method has self: Self parameter
4. No associated functions (no methods with no self)

**Making traits object-safe**:
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
