# Ownership, Borrowing, and Lifetimes

> Strategies for working with Rust's ownership system effectively.

---

## OB-01: Prefer Borrowing Over Ownership in Parameters

**Strength**: SHOULD

**Summary**: Functions should borrow (`&T`) unless they need ownership.

```rust
// ❌ UNNECESSARY OWNERSHIP: Forces caller to give up or clone
fn process(data: Vec<i32>) -> i32 {
    data.iter().sum()
}

// Caller must clone if they need the data later:
let data = vec![1, 2, 3];
let sum = process(data.clone());  // Unnecessary allocation
println!("{:?}", data);

// ✅ BORROWING: Caller retains ownership
fn process(data: &[i32]) -> i32 {
    data.iter().sum()
}

// Caller keeps their data:
let data = vec![1, 2, 3];
let sum = process(&data);
println!("{:?}", data);  // Still valid!

// ✅ OWNERSHIP NEEDED: When you must store or modify
fn consume_into_result(data: Vec<i32>) -> ProcessedData {
    ProcessedData { 
        original: data,  // Need to store it
        // ...
    }
}
```

**When to take ownership**:
- Storing in a struct field
- Transferring to another thread
- Transforming into a different type
- The function name suggests consumption (`into_*`, `consume_*`)

---

## OB-02: Return Owned Types, Not References

**Strength**: SHOULD (with exceptions)

**Summary**: Functions typically return owned values; references need lifetime annotation.

```rust
// ✅ SIMPLE: Return owned type
fn create_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

// ❌ WON'T COMPILE: Can't return reference to local
fn bad_greeting(name: &str) -> &str {
    let greeting = format!("Hello, {name}!");
    &greeting  // ERROR: greeting dropped at end of function
}

// ✅ RETURNING REFERENCE: When returning part of input
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

// ✅ RETURNING REFERENCE: From struct field
impl User {
    fn name(&self) -> &str {
        &self.name
    }
}

// ✅ COW: When sometimes owned, sometimes borrowed
use std::borrow::Cow;

fn maybe_modify(s: &str) -> Cow<'_, str> {
    if s.contains("bad") {
        Cow::Owned(s.replace("bad", "good"))
    } else {
        Cow::Borrowed(s)
    }
}
```

---

## OB-03: Use Lifetime Elision Where Possible

**Strength**: SHOULD

**Summary**: Let the compiler infer lifetimes; annotate only when required.

```rust
// ❌ VERBOSE: Explicit lifetimes where not needed
fn first<'a>(s: &'a str) -> &'a str {
    &s[..1]
}

// ✅ ELIDED: Compiler infers the lifetime
fn first(s: &str) -> &str {
    &s[..1]
}

// The elision rules:
// 1. Each elided input lifetime becomes a distinct parameter
// 2. If there's exactly one input lifetime, it's assigned to all outputs
// 3. If there's &self or &mut self, its lifetime is assigned to outputs

// ✅ EXPLICIT NEEDED: Multiple input lifetimes, ambiguous output
fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() { s1 } else { s2 }
}

// ✅ EXPLICIT NEEDED: Output lifetime differs from inputs
struct Parser<'input> {
    input: &'input str,
}

impl<'input> Parser<'input> {
    fn new(input: &'input str) -> Self {
        Self { input }
    }
}
```

---

## OB-04: Struct Decomposition for Independent Borrowing

**Strength**: CONSIDER

**Summary**: Split structs to allow borrowing fields independently.

```rust
// ❌ PROBLEM: Can't borrow two fields mutably through &mut self
struct Game {
    player: Player,
    enemies: Vec<Enemy>,
}

impl Game {
    fn update(&mut self) {
        for enemy in &mut self.enemies {
            // ERROR: Can't borrow player while enemies borrowed
            enemy.attack(&mut self.player);
        }
    }
}

// ✅ SOLUTION A: Pass fields separately
impl Game {
    fn update(&mut self) {
        let (player, enemies) = (&mut self.player, &mut self.enemies);
        for enemy in enemies {
            enemy.attack(player);  // Works!
        }
    }
}

// ✅ SOLUTION B: Decompose into sub-structs
struct GameState {
    player: Player,
}

struct GameEntities {
    enemies: Vec<Enemy>,
}

struct Game {
    state: GameState,
    entities: GameEntities,
}

impl Game {
    fn update(&mut self) {
        for enemy in &mut self.entities.enemies {
            enemy.attack(&mut self.state.player);  // Different sub-structs!
        }
    }
}
```

**Rationale**: The borrow checker sees struct fields independently only within the same function. Decomposition helps when methods need concurrent access to different parts.

---

## OB-05: Use `'static` Correctly

**Strength**: MUST

**Summary**: `'static` means "can live forever", not "must live forever".

```rust
// MISCONCEPTION: 'static means it lives for entire program
// TRUTH: 'static means it CAN live for the entire program (no borrowed references)

// ✅ String literals are 'static
let s: &'static str = "hello";

// ✅ Owned types satisfy 'static bounds
fn spawn_thread<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::spawn(f);
}

let owned = String::from("hello");
spawn_thread(move || {
    println!("{}", owned);  // Works: String is 'static (no borrowed refs)
});

// ❌ WRONG: Borrowed references don't satisfy 'static
let local = String::from("hello");
let borrowed: &str = &local;
spawn_thread(move || {
    // println!("{}", borrowed);  // ERROR: borrowed has non-static lifetime
});

// ✅ CORRECT understanding for trait objects:
fn takes_static(x: Box<dyn std::fmt::Debug + 'static>) { }

takes_static(Box::new(String::from("owned")));  // OK: String is 'static
// takes_static(Box::new(&local_string));  // ERROR: reference not 'static
```

**Common uses of `'static` bounds**:
- Thread spawning (data must outlive thread)
- Storing in global/lazy statics
- Trait objects without borrowed data

---

## OB-06: Interior Mutability Patterns

**Strength**: CONSIDER

**Summary**: Use `Cell`, `RefCell`, `Mutex`, etc. when shared references need mutation.

```rust
use std::cell::{Cell, RefCell};
use std::sync::{Mutex, RwLock, Arc};

// Cell<T>: For Copy types, no runtime cost
struct Counter {
    value: Cell<i32>,
}

impl Counter {
    fn increment(&self) {  // Note: &self, not &mut self
        self.value.set(self.value.get() + 1);
    }
}

// RefCell<T>: Runtime borrow checking, panics on violation
struct Document {
    content: RefCell<String>,
}

impl Document {
    fn append(&self, text: &str) {
        self.content.borrow_mut().push_str(text);
    }
    
    fn read(&self) -> String {
        self.content.borrow().clone()
    }
}

// Mutex<T>: Thread-safe, blocking
struct SharedState {
    data: Mutex<Vec<i32>>,
}

impl SharedState {
    fn add(&self, value: i32) {
        self.data.lock().unwrap().push(value);
    }
}

// RwLock<T>: Multiple readers OR single writer
struct Cache {
    data: RwLock<HashMap<String, String>>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }
    
    fn set(&self, key: String, value: String) {
        self.data.write().unwrap().insert(key, value);
    }
}
```

**Choosing interior mutability**:

| Type | Thread-safe | Cost | Use when |
|------|-------------|------|----------|
| `Cell<T>` | No | Zero | T: Copy, single-threaded |
| `RefCell<T>` | No | Runtime borrow check | Single-threaded, non-Copy |
| `Mutex<T>` | Yes | Lock contention | Multi-threaded, exclusive access |
| `RwLock<T>` | Yes | Lock contention | Multi-threaded, many readers |
| `Atomic*` | Yes | CPU atomic ops | Primitives, lock-free |

---

## OB-07: Clone Strategically

**Strength**: SHOULD

**Summary**: Clone intentionally, not to satisfy the borrow checker.

```rust
// ❌ BAD: Cloning to avoid borrow issues (anti-pattern)
fn process(data: &Data) {
    let cloned = data.clone();  // Why?
    // ... use cloned where data would have worked
}

// ✅ GOOD: Clone when you need independent ownership
fn process(data: &Data) -> ProcessHandle {
    let owned = data.clone();  // Need to store in handle
    ProcessHandle { data: owned }
}

// ✅ GOOD: Clone for thread transfer
fn spawn_processor(data: &Data) {
    let owned = data.clone();  // Threads need 'static
    std::thread::spawn(move || {
        process(owned);
    });
}

// ✅ GOOD: Clone with Rc/Arc is cheap
use std::rc::Rc;

let shared = Rc::new(expensive_data());
let handle1 = Rc::clone(&shared);  // Just increments counter
let handle2 = Rc::clone(&shared);  // Still just increments counter
```

**Clone cost awareness**:
- `String`, `Vec<T>`, `HashMap<K,V>`: Allocates + copies all data
- `Rc<T>`, `Arc<T>`: Just increments counter (cheap!)
- `Copy` types: Implicit, usually very cheap

---

## OB-08: Use `Cow` for Optional Ownership

**Strength**: CONSIDER

**Summary**: `Cow` (Clone-on-Write) delays allocation until mutation is needed.

```rust
use std::borrow::Cow;

// ✅ Function that might need to modify input
fn normalize_path(path: &str) -> Cow<'_, str> {
    if path.contains("//") {
        // Need to allocate
        Cow::Owned(path.replace("//", "/"))
    } else {
        // No allocation needed
        Cow::Borrowed(path)
    }
}

let path1 = normalize_path("/home/user");     // Borrowed, no alloc
let path2 = normalize_path("/home//user");    // Owned, allocated

// ✅ Struct that might own or borrow
struct Config<'a> {
    name: Cow<'a, str>,
}

impl<'a> Config<'a> {
    fn borrowed(name: &'a str) -> Self {
        Self { name: Cow::Borrowed(name) }
    }
    
    fn owned(name: String) -> Self {
        Self { name: Cow::Owned(name) }
    }
    
    fn name(&self) -> &str {
        &self.name  // Works for both variants
    }
}
```

**When to use `Cow`**:
- Functions that usually return their input unchanged
- Avoiding allocation in hot paths
- APIs that accept both borrowed and owned

---

## OB-09: Lifetime Bounds on Structs

**Strength**: MUST (when storing references)

**Summary**: Structs storing references must declare their lifetime parameters.

```rust
// ✅ CORRECT: Lifetime parameter for borrowed data
struct Parser<'input> {
    input: &'input str,
    position: usize,
}

impl<'input> Parser<'input> {
    fn new(input: &'input str) -> Self {
        Self { input, position: 0 }
    }
    
    fn remaining(&self) -> &'input str {
        &self.input[self.position..]
    }
}

// Usage:
let text = String::from("hello world");
let parser = Parser::new(&text);
// parser cannot outlive text

// ✅ ALTERNATIVE: Store owned data (no lifetime needed)
struct OwnedParser {
    input: String,
    position: usize,
}

// ✅ MULTIPLE LIFETIMES: When needed
struct Processor<'a, 'b> {
    input: &'a str,
    output: &'b mut String,
}

// 'a and 'b can be different, allowing more flexible usage
```

---

## OB-10: Smart Pointer Selection

**Strength**: SHOULD

**Summary**: Choose the right smart pointer for your ownership needs.

```rust
// Box<T>: Single owner, heap allocation
let boxed: Box<[i32; 1000]> = Box::new([0; 1000]);  // Large array on heap

// Rc<T>: Multiple owners, single-threaded
use std::rc::Rc;
let shared = Rc::new(data);
let clone1 = Rc::clone(&shared);
let clone2 = Rc::clone(&shared);
// All three point to same data

// Arc<T>: Multiple owners, thread-safe
use std::sync::Arc;
let shared = Arc::new(data);
let for_thread = Arc::clone(&shared);
std::thread::spawn(move || {
    println!("{:?}", for_thread);
});

// Weak<T>: Non-owning reference (prevents cycles)
use std::rc::Weak;
struct Node {
    parent: Option<Weak<Node>>,  // Doesn't prevent parent from being dropped
    children: Vec<Rc<Node>>,     // Owns children
}
```

**Selection guide**:

| Need | Use |
|------|-----|
| Single owner, heap data | `Box<T>` |
| Shared ownership, single-threaded | `Rc<T>` |
| Shared ownership, multi-threaded | `Arc<T>` |
| Shared + mutable, single-threaded | `Rc<RefCell<T>>` |
| Shared + mutable, multi-threaded | `Arc<Mutex<T>>` or `Arc<RwLock<T>>` |
| Break reference cycles | `Weak<T>` |

---

## OB-11: Move Closures

**Strength**: SHOULD (when needed)

**Summary**: Use `move` to transfer ownership into closures.

```rust
// ❌ WON'T COMPILE: Closure borrows, but outlives the data
fn spawn_printer(message: String) {
    std::thread::spawn(|| {
        println!("{}", message);  // ERROR: message borrowed, not moved
    });
}

// ✅ CORRECT: move captures ownership
fn spawn_printer(message: String) {
    std::thread::spawn(move || {
        println!("{}", message);  // OK: message moved into closure
    });
}

// Selective capture with move:
fn example() {
    let owned = String::from("owned");
    let to_clone = String::from("cloned");
    let to_clone = to_clone.clone();  // Clone before closure
    
    std::thread::spawn(move || {
        // Both owned and to_clone are moved in
        println!("{} {}", owned, to_clone);
    });
}

// Rebinding for partial moves:
fn selective_move() {
    let data = ComplexStruct { a: String::new(), b: 42 };
    let a = data.a;  // Move out just `a`
    
    std::thread::spawn(move || {
        println!("{}", a);  // Only `a` captured
    });
    
    // data.b is still accessible (b is Copy)
    println!("{}", data.b);
}
```

---

## OB-12: The `mem::take` and `mem::replace` Pattern

**Strength**: SHOULD

**Summary**: Move values out of mutable references without unsafe.

```rust
use std::mem;

// ✅ mem::take: Replace with Default, return old value
fn drain_queue<T: Default>(queue: &mut Vec<T>) -> Vec<T> {
    mem::take(queue)  // queue is now empty Vec, returns old contents
}

// ✅ mem::replace: Replace with specific value
fn update_state(state: &mut State) -> State {
    mem::replace(state, State::Updated)  // Returns old state
}

// ✅ Use case: Moving out of enum variants
enum Status {
    Pending(Data),
    Complete,
}

fn complete(status: &mut Status) -> Option<Data> {
    match status {
        Status::Pending(data) => {
            let data = mem::take(data);  // Take the data
            *status = Status::Complete;
            Some(data)
        }
        Status::Complete => None,
    }
}

// ✅ Option::take is a convenience method for this
fn extract_value<T>(opt: &mut Option<T>) -> Option<T> {
    opt.take()  // Same as mem::take(opt)
}
```

---

## Summary: Ownership Decision Guide

```
Do you need to store the data?
├─ Yes → Take ownership (T or Box<T>)
│        └─ Multiple owners?
│           ├─ Single-thread → Rc<T>
│           └─ Multi-thread → Arc<T>
└─ No → Borrow it (&T or &mut T)
         └─ Need to mutate?
            ├─ Yes → &mut T
            └─ No → &T

Is the reference long-lived?
├─ Across threads → Must be 'static or Arc
├─ In a struct → Add lifetime parameter or own the data
└─ Same function → Compiler handles it

Need shared mutation?
├─ Single-thread → RefCell<T> or Cell<T>
└─ Multi-thread → Mutex<T> or RwLock<T>
```

---

*See also: [11-anti-patterns.md](11-anti-patterns.md#ap-01) for Clone anti-pattern.*
