# Trait Design

> Patterns for defining traits, implementing them, and using trait objects.

---

## TR-01: Trait Definition Basics

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

## TR-02: Supertraits for Trait Composition

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

## TR-03: Generic Traits vs Associated Types

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

## TR-04: Object Safety

**Strength**: MUST (understand)

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

**Object safety rules** (simplified):
1. No generic methods (no `<T>` in method signatures)
2. No methods returning `Self`
3. No methods with `Self` in argument position (except `self`)
4. No associated constants
5. Associated types must not have bounds

**Workaround**: Provide object-safe subset:

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

## TR-05: Blanket Implementations

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

## TR-06: Trait Objects vs Generics

**Strength**: SHOULD (choose appropriately)

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

**Decision guide**:

| Use | When |
|-----|------|
| Generics | Homogeneous collections, performance critical, compile-time known types |
| Trait objects | Heterogeneous collections, plugin systems, reducing binary size, runtime type selection |

---

## TR-07: The `Deref` Trait (Use Sparingly)

**Strength**: CONSIDER (for smart pointers only)

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

**Legitimate `Deref` uses**:
- `Box<T>` → `T`
- `String` → `str`
- `Vec<T>` → `[T]`
- `Arc<T>` → `T`
- Custom smart pointers

---

## TR-08: Extension Traits

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

**Naming convention**: `{Type}Ext` or `{Capability}Ext`

---

## TR-09: Marker Traits

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

## TR-10: Negative Trait Bounds (Unstable Pattern)

**Strength**: CONSIDER (workaround)

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

## TR-11: Coherence and Orphan Rules

**Strength**: MUST (understand)

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

## TR-12: `impl Trait` in Argument Position

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

## Summary: Trait Design Checklist

**Design**:
- [ ] Traits describe behavior, not data structure
- [ ] Use supertraits for required capabilities
- [ ] Associated types for "one per impl", generics for "many per impl"
- [ ] Consider object safety if `dyn Trait` might be needed

**Implementation**:
- [ ] Blanket impls for broad applicability
- [ ] Extension traits for foreign types
- [ ] Newtype wrapper when orphan rules block you

**Choice**:
- [ ] Generics for performance, monomorphization
- [ ] Trait objects for heterogeneous collections, plugins
- [ ] `impl Trait` for cleaner signatures

---

*See also: [05-type-design.md](05-type-design.md) for type patterns that work with traits.*
