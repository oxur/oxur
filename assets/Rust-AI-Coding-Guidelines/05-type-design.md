# Type Design

> Patterns for designing structs, enums, newtypes, and generics.

---

## TD-01: Use Newtypes for Type Safety

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

## TD-02: Use Enums for State Machines

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

## TD-03: Builder Pattern for Complex Construction

**Strength**: SHOULD

**Summary**: Use a builder when construction has many optional parameters.

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

**Variations**:
- Return `Self` by value for chaining (shown above)
- Return `&mut Self` for reuse: `builder.foo(1); builder.bar(2); builder.build()`
- Typestate builder: encode required fields in the type system

---

## TD-04: Use `#[non_exhaustive]` for Future Compatibility

**Strength**: SHOULD (for libraries)

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

## TD-05: Prefer Structs Over Tuples for Named Data

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

## TD-06: Generic Type Parameters

**Strength**: SHOULD (use appropriately)

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

**Generic vs Concrete**:
- Generic: Same logic works for multiple types
- Concrete: Simpler, faster compilation, clearer errors

---

## TD-07: Associated Types vs Generic Parameters

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

## TD-08: The Typestate Pattern

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

## TD-09: Phantom Data for Unused Type Parameters

**Strength**: MUST (when needed)

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

**PhantomData variations**:
- `PhantomData<T>` - Acts like you own a `T`
- `PhantomData<*const T>` - Covariant, doesn't imply ownership
- `PhantomData<fn() -> T>` - Covariant in `T`

---

## TD-10: Encapsulation with Private Fields

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

## TD-11: Zero-Sized Types (ZSTs)

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

## TD-12: Sealed Traits

**Strength**: CONSIDER (for libraries)

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

## Summary: Type Design Checklist

- [ ] Use newtypes to distinguish semantically different values
- [ ] Model states as enum variants, not boolean flags
- [ ] Use builder pattern for > 3 optional parameters
- [ ] Mark public types `#[non_exhaustive]` in libraries
- [ ] Prefer structs over tuples for named data
- [ ] Use generics only when behavior is truly generic
- [ ] Associated types for "one per impl", generics for "many per impl"
- [ ] Keep fields private; validate in constructors
- [ ] Consider typestate for compile-time state enforcement

---

*See also: [06-traits.md](06-traits.md) for trait design patterns.*
