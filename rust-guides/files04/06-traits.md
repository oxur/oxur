# Trait Design and Implementation

Guidelines for designing traits, implementing them effectively, and using trait objects.

## Table of Contents

- [Essential Functionality in Inherent Impls](#essential-functionality-in-inherent-impls)
- [Trait Design Principles](#trait-design-principles)
- [Common Trait Implementations](#common-trait-implementations)
- [Trait Objects vs Generics](#trait-objects-vs-generics)

---

## Essential Functionality in Inherent Impls

### Essential Functionality Should be Inherent

**Strength**: MUST

**Summary**: Types should implement core functionality inherently; trait implementations should forward to inherent methods.

**Example**:
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

## Trait Design Principles

### Narrow Traits Over Wide Traits

**Strength**: SHOULD

**Summary**: When designing trait hierarchies, prefer multiple narrow traits over one wide trait.

**Example**:
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

## Common Trait Implementations

### Types Eagerly Implement Common Traits

**Strength**: SHOULD

**Summary**: Public types should derive or implement common traits where applicable: `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Default`, `Debug`, `Display`.

**Example**:
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

**Guidelines**:
- `Debug`: Always for public types (see M-PUBLIC-DEBUG)
- `Display`: For types meant to be displayed to users
- `Clone`: When copying is reasonable
- `Copy`: Only for cheap-to-copy types (prefer explicit `.clone()`)
- `Eq`/`PartialEq`: For types that can be compared
- `Hash`: When type should work in HashMap/HashSet
- `Default`: When a sensible default value exists

**See also**: C-COMMON-TRAITS, M-PUBLIC-DEBUG

---

## Trait Objects vs Generics

### Prefer Concrete Types > Generics > dyn Trait

**Strength**: SHOULD

**Summary**: Use concrete types when possible, generics when flexibility is needed, and trait objects (`dyn Trait`) only when necessary to avoid excessive nesting.

**Example**:
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

**Rationale**: 
- Concrete types are simple and have zero runtime overhead
- Generics enable flexibility and are zero-cost abstractions (monomorphized)
- Trait objects (`dyn Trait`) have small runtime cost but prevent type nesting explosion

Use trait objects when:
- Generic type parameters would nest excessively (3+ levels)
- Runtime polymorphism is genuinely needed
- Compile times are becoming problematic from monomorphization

**See also**: M-DI-HIERARCHY, M-SIMPLE-ABSTRACTIONS

---

### Object Safety Considerations

**Strength**: MUST (when using trait objects)

**Summary**: Traits used as trait objects must be object-safe: no generic methods, no `Self: Sized` bounds, methods return simple types.

**Example**:
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

## Trait Implementation Patterns

### Forwarding Implementations

**Strength**: SHOULD

**Summary**: When implementing traits on wrapper types, forward to the inner implementation.

**Example**:
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

## Trait Bounds

### Keep Trait Bounds Minimal

**Strength**: SHOULD

**Summary**: Only require trait bounds that are actually needed for the implementation.

**Example**:
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
