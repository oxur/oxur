# Core Idioms

Essential Rust conventions and naming patterns that form the foundation of idiomatic Rust code.

## Naming Conventions

### Casing Conforms to RFC 430

**Strength**: MUST

**Summary**: Follow Rust's standard casing conventions for all identifiers.

**Conventions Table**:

| Item | Convention | Example |
|------|------------|---------|
| Crates | `snake_case` (unclear in RFC) | `serde_json` |
| Modules | `snake_case` | `std::collections` |
| Types | `UpperCamelCase` | `Vec`, `HashMap` |
| Traits | `UpperCamelCase` | `Iterator`, `Clone` |
| Enum variants | `UpperCamelCase` | `Some`, `None` |
| Functions | `snake_case` | `fn parse_input()` |
| Methods | `snake_case` | `fn is_empty()` |
| General constructors | `new` or `with_*` | `Vec::new()` |
| Conversion constructors | `from_*` | `String::from_utf8()` |
| Macros | `snake_case!` | `vec!`, `println!` |
| Local variables | `snake_case` | `let user_name` |
| Statics | `SCREAMING_SNAKE_CASE` | `const MAX_SIZE` |
| Constants | `SCREAMING_SNAKE_CASE` | `const PI: f64` |
| Type parameters | Concise `UpperCamelCase` | `T`, `K`, `V` |
| Lifetimes | Short lowercase | `'a`, `'de`, `'src` |

**Examples**:

```rust
// Good - proper casing
pub struct HttpResponse {
    status_code: u16,
    body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16) -> Self {
        HttpResponse {
            status_code,
            body: String::new(),
        }
    }
    
    pub fn with_body(status_code: u16, body: String) -> Self {
        HttpResponse { status_code, body }
    }
}

const MAX_RETRIES: u32 = 3;
const DEFAULT_TIMEOUT_MS: u64 = 5000;

// Bad - incorrect casing
pub struct HTTPResponse { // Should be HttpResponse
    StatusCode: u16,      // Should be status_code
    Body: String,         // Should be body
}
```

**Rationale**: Consistent naming makes code instantly recognizable as idiomatic Rust and improves cross-project readability.

**Special Rules**:

1. **Acronyms**: In `UpperCamelCase`, treat acronyms as single words: `Uuid` not `UUID`, `TcpStream` not `TCPStream`
2. **Single letters**: In `snake_case`, avoid single-letter words except at end: `btree_map` not `b_tree_map`, but `PI_2` is ok
3. **Crate names**: Never use `-rs` or `-rust` suffix (e.g., `serde` not `serde-rs`)

---

## Conversion Methods

### Ad-hoc Conversions Follow as_/to_/into_ Conventions

**Strength**: MUST

**Summary**: Use prefix conventions that communicate cost and ownership semantics.

**Convention Table**:

| Prefix | Cost | Ownership Transfer | Example |
|--------|------|-------------------|---------|
| `as_` | Free | Borrowed → Borrowed | `str::as_bytes()` |
| `to_` | Expensive | Borrowed → Owned (or Owned → Owned for Copy) | `str::to_lowercase()` |
| `into_` | Variable | Owned → Owned | `String::into_bytes()` |

**Examples**:

```rust
// as_* - free, borrowed to borrowed
impl str {
    pub fn as_bytes(&self) -> &[u8] {
        // Zero cost - just reinterprets the reference
        unsafe { self.as_bytes() }
    }
}

// to_* - expensive, creates owned data
impl str {
    pub fn to_lowercase(&self) -> String {
        // Allocates new String, iterates through chars
        // Unicode-aware conversion
        self.chars()
            .flat_map(|c| c.to_lowercase())
            .collect()
    }
    
    pub fn to_string(&self) -> String {
        // Allocates and copies
        String::from(self)
    }
}

// into_* - consumes self, transfers ownership
impl String {
    pub fn into_bytes(self) -> Vec<u8> {
        // Takes ownership, no allocation
        // Just transfers the Vec<u8> inside String
        self.into_bytes()
    }
}

impl<T> Vec<T> {
    pub fn into_boxed_slice(self) -> Box<[T]> {
        // Consumes Vec, returns Box
        self.into_boxed_slice()
    }
}

// Bad examples
impl Path {
    // Bad - should be as_os_str (free operation)
    pub fn to_os_str(&self) -> &OsStr { /* ... */ }
    
    // Bad - should be to_path_buf (expensive clone)
    pub fn as_path_buf(&self) -> PathBuf { /* ... */ }
}

// Good - f64 conversions
impl f64 {
    // to_* is correct - input is Copy type being converted
    pub fn to_radians(self) -> f64 {
        self * (PI / 180.0)
    }
    
    pub fn to_degrees(self) -> f64 {
        self * (180.0 / PI)
    }
}
```

**Rationale**: These prefixes provide immediate clarity about the cost and ownership implications of a conversion, helping developers write efficient code without consulting documentation.

**Special Cases**:

1. **into_inner()**: Use for unwrapping types that add semantics to an inner value
   ```rust
   impl<R: Read> BufReader<R> {
       pub fn into_inner(self) -> R {
           self.inner
       }
   }
   ```

2. **Mut qualifiers**: Place `mut` as it appears in the type
   ```rust
   impl<T> Vec<T> {
       // Correct - returns &mut [T]
       pub fn as_mut_slice(&mut self) -> &mut [T] { /* ... */ }
       
       // Wrong - confusing
       pub fn as_slice_mut(&mut self) -> &mut [T] { /* ... */ }
   }
   ```

**See also**: 
- C-CONV-TRAITS for `From`/`Into`/`AsRef`/`AsMut` trait implementations
- C-GETTER for getter naming conventions

---

## Getter Methods

### Getter Names Follow Rust Convention

**Strength**: MUST

**Summary**: Avoid `get_` prefix except for specific cases; getters are just named after the field.

**Examples**:

```rust
// Good - idiomatic getters
pub struct Connection {
    timeout: Duration,
    address: SocketAddr,
}

impl Connection {
    // Simple field access - no get_ prefix
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
    
    // Mutable access follows same pattern
    pub fn timeout_mut(&mut self) -> &mut Duration {
        &mut self.timeout
    }
    
    // Returns reference to field
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

// Bad - unnecessary get_ prefix
impl Connection {
    pub fn get_timeout(&self) -> Duration { // Don't do this
        self.timeout
    }
    
    pub fn get_address(&self) -> &SocketAddr { // Don't do this
        &self.address
    }
}

// Good - get_ is appropriate for Cell/RefCell
use std::cell::Cell;

impl Cell<T> {
    pub fn get(&self) -> T where T: Copy {
        // Only one obvious thing to "get" from a Cell
        // get_ prefix makes sense here
    }
}

// Good - unchecked variants
impl<T> [T] {
    pub fn get(&self, index: usize) -> Option<&T> { /* ... */ }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> { /* ... */ }
    
    pub unsafe fn get_unchecked(&self, index: usize) -> &T { /* ... */ }
    
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T { /* ... */ }
}
```

**Rationale**: The `get_` prefix is redundant in most cases. Rust's type system makes getters obvious without the prefix, reducing verbosity.

**When to use `get_` prefix**:
1. When there's a single, obvious thing being "gotten" (like `Cell::get`)
2. For runtime-validated access with unchecked variants (`slice::get`, `slice::get_unchecked`)

**When NOT to use `get_` prefix**:
1. Simple field access
2. Computed properties
3. Most builder or configuration types

---

## Iterator Conventions

### Iterator Methods Follow iter/iter_mut/into_iter Pattern

**Strength**: MUST

**Summary**: For collections containing elements of type `U`, provide three standard iterator methods.

**Standard Iterator Method Signatures**:

```rust
impl<T> Container<T> {
    // Borrows elements immutably
    fn iter(&self) -> Iter<'_, T> 
    // where Iter: Iterator<Item = &T>
    
    // Borrows elements mutably  
    fn iter_mut(&mut self) -> IterMut<'_, T>
    // where IterMut: Iterator<Item = &mut T>
    
    // Consumes container, transfers ownership
    fn into_iter(self) -> IntoIter<T>
    // where IntoIter: Iterator<Item = T>
}
```

**Examples**:

```rust
// Good - standard collection iterator pattern
pub struct MyCollection<T> {
    items: Vec<T>,
}

pub struct Iter<'a, T> {
    inner: std::slice::Iter<'a, T>,
}

pub struct IterMut<'a, T> {
    inner: std::slice::IterMut<'a, T>,
}

pub struct IntoIter<T> {
    inner: std::vec::IntoIter<T>,
}

impl<T> MyCollection<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.items.iter(),
        }
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: self.items.iter_mut(),
        }
    }
    
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner: self.items.into_iter(),
        }
    }
}

// Implement Iterator for each type
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

// Usage examples
let mut collection = MyCollection { items: vec![1, 2, 3] };

// Borrow elements immutably
for item in collection.iter() {
    println!("{}", item);
}

// Borrow elements mutably
for item in collection.iter_mut() {
    *item *= 2;
}

// Consume collection
for item in collection.into_iter() {
    println!("{}", item);
}
// collection is now moved and can't be used
```

**Counterexample - When NOT to use this pattern**:

```rust
// str is NOT a simple homogeneous collection
impl str {
    // Not iter(), because str has nuanced interpretation
    pub fn bytes(&self) -> Bytes<'_> { /* ... */ }
    pub fn chars(&self) -> Chars<'_> { /* ... */ }
    pub fn lines(&self) -> Lines<'_> { /* ... */ }
}
```

**Rationale**: This pattern is so ubiquitous that developers expect it. It provides consistency across all collections and enables generic code.

**See also**: C-ITER-TY for naming the iterator types themselves

---

## Iterator Type Naming

### Iterator Type Names Match Producing Methods

**Strength**: SHOULD

**Summary**: An `into_iter()` method should return an `IntoIter` type, `iter()` returns `Iter`, etc.

**Examples**:

```rust
// Good - consistent naming
impl<T> Vec<T> {
    pub fn iter(&self) -> Iter<'_, T> { /* ... */ }
    pub fn iter_mut(&mut self) -> IterMut<'_, T> { /* ... */ }
    pub fn into_iter(self) -> IntoIter<T> { /* ... */ }
}

impl<K, V> BTreeMap<K, V> {
    pub fn keys(&self) -> Keys<'_, K, V> { /* ... */ }
    pub fn values(&self) -> Values<'_, K, V> { /* ... */ }
    pub fn iter(&self) -> Iter<'_, K, V> { /* ... */ }
}

// Good - function returning iterator
use url::percent_encoding;

fn percent_encode(input: &str) -> PercentEncode<'_> {
    percent_encoding::percent_encode(input.as_bytes())
}

// The type name matches the function name pattern
pub struct PercentEncode<'a> { /* ... */ }
```

**Rationale**: When type names match method names, documentation and error messages are more intuitive. The pattern `module::TypeName` makes it clear where the iterator came from.

**Common patterns**:
- `iter()` → `Iter`
- `iter_mut()` → `IterMut`  
- `into_iter()` → `IntoIter`
- `keys()` → `Keys`
- `values()` → `Values`
- `lines()` → `Lines`
- `bytes()` → `Bytes`

---

## Feature Flags

### Feature Names Are Free of Placeholder Words

**Strength**: MUST

**Summary**: Don't use words like `use-` or `with-` in feature names. Name features directly after what they enable.

**Examples**:

```rust
// Good - Cargo.toml
[features]
default = ["std"]
std = []
serde = ["dep:serde"]

[dependencies]
serde = { version = "1.0", optional = true }

// Bad - Cargo.toml
[features]
default = ["use-std"]  // Don't add "use-"
use-std = []           // Just call it "std"
with-serde = ["dep:serde"]  // Just call it "serde"
```

```rust
// Good - enabling std library support
// In lib.rs
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub fn read_file(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

// Usage in dependent crate's Cargo.toml
[dependencies]
my-crate = { version = "1.0", features = ["std"] }  // Clean and simple
```

**Rationale**: Cargo automatically creates implicit features for optional dependencies using the dependency name. Explicit features should follow the same convention for consistency.

**Rule for `std` feature**:
- Always call it `std`, not `use-std` or `with-std`
- This aligns with implicit feature naming
- Users expect `features = ["std"]`

**Rule for negative features**:
- AVOID features like `no-std` or `disable-logging`
- Cargo requires features to be additive
- Use positive names: `std`, `logging`

---

## Consistent Word Order

### Names Use Consistent Word Order

**Strength**: SHOULD

**Summary**: Within a crate (and ideally ecosystem-wide), use consistent word ordering in type and function names.

**Examples**:

```rust
// Good - consistent verb-object-error order (from std)
pub struct ParseBoolError;
pub struct ParseCharError;
pub struct ParseFloatError;
pub struct ParseIntError;
pub struct ParseAddrError;  // Hypothetical, consistent with others

// Bad - inconsistent ordering
pub struct ParseBoolError;
pub struct CharParseError;   // Wrong - breaks pattern
pub struct ErrorParseFloat;  // Wrong - breaks pattern

// Good - consistent noun-direction order
pub struct IntoIter<T>;
pub struct IntoKeys<K, V>;
pub struct IntoValues<K, V>;

// Bad - inconsistent  
pub struct IntoIter<T>;
pub struct KeysInto<K, V>;   // Wrong - breaks pattern

// Good - error type patterns
pub enum Error {
    Io(io::Error),
    Parse(ParseError),
    Network(NetworkError),
    Database(DatabaseError),
}

// Good - consistent builder method order
impl RequestBuilder {
    pub fn with_header(self, key: &str, value: &str) -> Self { /* ... */ }
    pub fn with_timeout(self, duration: Duration) -> Self { /* ... */ }
    pub fn with_body(self, body: String) -> Self { /* ... */ }
    // All use "with_" prefix consistently
}
```

**Rationale**: Consistent word order makes APIs more predictable and easier to discover through autocomplete. When adding new items, developers can guess their names correctly.

**See also**: 
- Standard library error types for examples
- C-CASE for overall naming conventions
