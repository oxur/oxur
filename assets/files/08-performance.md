# Performance Patterns

> Patterns for writing efficient Rust code without sacrificing clarity.

---

## PF-01: Avoid Unnecessary Allocations

**Strength**: SHOULD

**Summary**: Reuse buffers, prefer stack allocation, use references.

```rust
// ❌ BAD: Allocating in a loop
fn process_items(items: &[Item]) -> Vec<String> {
    let mut results = Vec::new();
    for item in items {
        let name = item.name.to_uppercase();  // Allocates each iteration
        results.push(name);
    }
    results
}

// ✅ BETTER: Pre-allocate with capacity
fn process_items(items: &[Item]) -> Vec<String> {
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        results.push(item.name.to_uppercase());
    }
    results
}

// ✅ BEST: Use iterators (often optimized)
fn process_items(items: &[Item]) -> Vec<String> {
    items.iter()
        .map(|item| item.name.to_uppercase())
        .collect()
}

// ❌ BAD: Creating String for comparison
fn contains_hello(s: &str) -> bool {
    s.to_lowercase() == "hello".to_string()  // Two allocations!
}

// ✅ GOOD: No allocation needed
fn contains_hello(s: &str) -> bool {
    s.eq_ignore_ascii_case("hello")
}
```

---

## PF-02: Use `&str` and `&[T]` Parameters

**Strength**: SHOULD

**Summary**: Accept slices to avoid forcing callers to allocate.

```rust
// ❌ BAD: Forces allocation
fn process(data: String) { }
fn sum(numbers: Vec<i32>) -> i32 { numbers.iter().sum() }

// Caller must convert:
let s = "hello";
process(s.to_string());  // Unnecessary allocation

// ✅ GOOD: Accept slices
fn process(data: &str) { }
fn sum(numbers: &[i32]) -> i32 { numbers.iter().sum() }

// Works with both:
process("hello");                    // &str
process(&String::from("hello"));     // &String coerces to &str
sum(&[1, 2, 3]);                     // array
sum(&vec![1, 2, 3]);                 // Vec coerces to &[T]
```

---

## PF-03: Prefer Iterators Over Index Loops

**Strength**: SHOULD

**Summary**: Iterator methods often optimize better than manual indexing.

```rust
// ❌ SLOWER: Index-based loop
fn sum_positive(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..numbers.len() {
        if numbers[i] > 0 {
            sum += numbers[i];  // Bounds check each access
        }
    }
    sum
}

// ✅ FASTER: Iterator (bounds checks eliminated)
fn sum_positive(numbers: &[i32]) -> i32 {
    numbers.iter()
        .filter(|&&n| n > 0)
        .sum()
}

// ✅ ALSO FAST: Iterator with for loop
fn sum_positive(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &n in numbers {  // No bounds checks needed
        if n > 0 {
            sum += n;
        }
    }
    sum
}

// ✅ When you need indices, use enumerate
for (i, item) in items.iter().enumerate() {
    println!("{}: {:?}", i, item);
}
```

---

## PF-04: Use `Cow` for Conditional Allocation

**Strength**: CONSIDER

**Summary**: `Cow` delays allocation until mutation is needed.

```rust
use std::borrow::Cow;

// ❌ ALWAYS ALLOCATES
fn normalize(s: &str) -> String {
    if s.contains(' ') {
        s.replace(' ', "_")
    } else {
        s.to_string()  // Allocates even when unchanged!
    }
}

// ✅ ALLOCATES ONLY WHEN NEEDED
fn normalize(s: &str) -> Cow<'_, str> {
    if s.contains(' ') {
        Cow::Owned(s.replace(' ', "_"))
    } else {
        Cow::Borrowed(s)  // No allocation
    }
}

// Usage:
let result = normalize("hello");      // Cow::Borrowed, no alloc
let result = normalize("hello world"); // Cow::Owned, allocates

// Cow implements Deref, so use it like &str:
println!("{}", result);
```

---

## PF-05: Small String Optimization Awareness

**Strength**: CONSIDER

**Summary**: Be aware of string optimization strategies.

```rust
// Standard String always heap-allocates

// For many small strings, consider:
// 1. smol_str crate - inline strings up to 23 bytes
use smol_str::SmolStr;
let s: SmolStr = "hello".into();  // Stack allocated!

// 2. compact_str crate - inline up to 24 bytes
use compact_str::CompactString;
let s: CompactString = "hello".into();

// 3. For fixed-size, use arrays
type ShortName = [u8; 32];  // No allocation

// 4. Interning for repeated strings
use string_interner::StringInterner;
let mut interner = StringInterner::default();
let sym1 = interner.get_or_intern("hello");
let sym2 = interner.get_or_intern("hello");
assert_eq!(sym1, sym2);  // Same symbol, deduplicated
```

---

## PF-06: Choose the Right Collection

**Strength**: SHOULD

**Summary**: Different collections have different performance characteristics.

```rust
// Vec<T>: Default choice, cache-friendly, O(1) push/pop at end
let mut v = Vec::new();
v.push(1);      // O(1) amortized
v.pop();        // O(1)
v.remove(0);    // O(n) - shifts all elements

// VecDeque<T>: O(1) push/pop at both ends
use std::collections::VecDeque;
let mut d = VecDeque::new();
d.push_front(1);  // O(1)
d.push_back(2);   // O(1)

// HashMap<K, V>: O(1) average lookup/insert
use std::collections::HashMap;
let mut m = HashMap::new();
m.insert("key", "value");  // O(1) average
m.get("key");              // O(1) average

// BTreeMap<K, V>: Sorted keys, O(log n) operations
use std::collections::BTreeMap;
let mut m = BTreeMap::new();
m.insert(3, "c");
m.insert(1, "a");
// Iteration is sorted: [(1, "a"), (3, "c")]

// HashSet<T> / BTreeSet<T>: Same trade-offs as maps

// For small N, Vec is often faster than HashMap due to cache
fn find_small<T: Eq>(haystack: &[(String, T)], needle: &str) -> Option<&T> {
    // For N < ~20, linear search beats HashMap
    haystack.iter()
        .find(|(k, _)| k == needle)
        .map(|(_, v)| v)
}
```

**Decision guide**:

| Need | Use |
|------|-----|
| General purpose | `Vec<T>` |
| Queue (FIFO) | `VecDeque<T>` |
| Stack (LIFO) | `Vec<T>` |
| Fast key lookup | `HashMap<K, V>` |
| Sorted iteration | `BTreeMap<K, V>` |
| Unique items | `HashSet<T>` / `BTreeSet<T>` |
| Small fixed N | Consider `Vec` with linear search |

---

## PF-07: Avoid `clone()` in Hot Paths

**Strength**: SHOULD

**Summary**: Clone has hidden costs; prefer references or `Rc`/`Arc`.

```rust
// ❌ BAD: Cloning in iteration
fn process(data: &Data) {
    for item in &data.items {
        let item = item.clone();  // Cloning each item!
        process_item(item);
    }
}

// ✅ GOOD: Pass by reference
fn process(data: &Data) {
    for item in &data.items {
        process_item(item);  // Borrow, no clone
    }
}

// ✅ GOOD: Use Rc/Arc for shared ownership
use std::rc::Rc;

fn process_shared(data: Rc<Data>) {
    let data_for_task = Rc::clone(&data);  // Cheap! Just increments counter
    do_something(data_for_task);
}

// ❌ BAD: Cloning to work around borrow checker
let cloned = data.clone();  // Are you sure you need this?

// ✅ GOOD: Restructure to avoid clone
// See mem::take, mem::replace patterns
```

---

## PF-08: Use `Entry` API for Map Updates

**Strength**: SHOULD

**Summary**: `entry()` avoids double lookup for insert-or-update.

```rust
use std::collections::HashMap;

// ❌ BAD: Double lookup
fn count_word(map: &mut HashMap<String, u32>, word: &str) {
    if map.contains_key(word) {
        *map.get_mut(word).unwrap() += 1;
    } else {
        map.insert(word.to_string(), 1);
    }
}

// ✅ GOOD: Single lookup with entry
fn count_word(map: &mut HashMap<String, u32>, word: &str) {
    *map.entry(word.to_string()).or_insert(0) += 1;
}

// ✅ GOOD: Lazy initialization
map.entry(key).or_insert_with(|| expensive_default());

// ✅ GOOD: Complex update logic
map.entry(key)
    .and_modify(|v| v.count += 1)
    .or_insert(Value { count: 1 });
```

---

## PF-09: Benchmark Before Optimizing

**Strength**: MUST

**Summary**: Measure actual performance; don't optimize blindly.

```rust
// Use criterion for benchmarks:
// Cargo.toml: criterion = "0.5"

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_sum(c: &mut Criterion) {
    let data: Vec<i32> = (0..10000).collect();
    
    c.bench_function("iterator sum", |b| {
        b.iter(|| {
            black_box(data.iter().sum::<i32>())
        })
    });
    
    c.bench_function("loop sum", |b| {
        b.iter(|| {
            let mut sum = 0;
            for &n in black_box(&data) {
                sum += n;
            }
            sum
        })
    });
}

criterion_group!(benches, benchmark_sum);
criterion_main!(benches);

// Run with: cargo bench
```

**Optimization workflow**:
1. Write correct code first
2. Profile to find actual bottlenecks
3. Benchmark the specific code path
4. Optimize and measure improvement
5. Ensure correctness is maintained

---

## PF-10: Zero-Copy Parsing

**Strength**: CONSIDER

**Summary**: Parse into references to avoid copying data.

```rust
// ❌ COPIES: Parsing into owned strings
fn parse_csv_line(line: &str) -> Vec<String> {
    line.split(',')
        .map(|s| s.to_string())  // Allocates each field!
        .collect()
}

// ✅ ZERO-COPY: Return references
fn parse_csv_line(line: &str) -> Vec<&str> {
    line.split(',').collect()
}

// ✅ ZERO-COPY: Struct with references
struct ParsedLine<'a> {
    name: &'a str,
    value: &'a str,
}

fn parse_line(line: &str) -> Option<ParsedLine<'_>> {
    let (name, value) = line.split_once('=')?;
    Some(ParsedLine { name: name.trim(), value: value.trim() })
}

// ✅ Libraries for zero-copy parsing:
// - nom: Parser combinators
// - winnow: Faster parser combinators  
// - serde with #[serde(borrow)]: Deserialize to borrowed data
```

---

## PF-11: Inline Small Functions

**Strength**: CONSIDER

**Summary**: `#[inline]` hints can help with small, frequently-called functions.

```rust
// ✅ Good candidates for #[inline]
impl Vector3 {
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    #[inline]
    pub fn length_squared(&self) -> f64 {
        self.dot(self)
    }
}

// #[inline(always)]: Force inline (use sparingly)
#[inline(always)]
fn tiny_helper(x: i32) -> i32 {
    x + 1
}

// #[cold]: Hint that function is rarely called
#[cold]
fn handle_error(e: Error) {
    // Error path, rarely executed
}

// Note: The compiler is usually good at inlining.
// Only add #[inline] after profiling shows it helps.
```

---

## PF-12: SIMD and Auto-Vectorization

**Strength**: CONSIDER

**Summary**: Write code that the compiler can auto-vectorize.

```rust
// ✅ AUTO-VECTORIZABLE: Simple loop over slices
fn add_arrays(a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..a.len().min(b.len()).min(out.len()) {
        out[i] = a[i] + b[i];
    }
}

// ✅ BETTER: Iterator version (often vectorizes)
fn add_arrays(a: &[f32], b: &[f32], out: &mut [f32]) {
    for ((o, a), b) in out.iter_mut().zip(a).zip(b) {
        *o = a + b;
    }
}

// For explicit SIMD, use portable-simd (nightly) or wide crate
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Or use libraries:
// - packed_simd2: Portable SIMD
// - wide: Safe SIMD wrappers
// - ndarray: Optimized array operations
```

---

## PF-13: Memory Layout Optimization

**Strength**: CONSIDER

**Summary**: Struct field order affects memory usage.

```rust
// ❌ WASTEFUL: Poor field ordering
struct Wasteful {
    a: u8,      // 1 byte + 7 padding
    b: u64,     // 8 bytes
    c: u8,      // 1 byte + 7 padding
    d: u64,     // 8 bytes
}  // Total: 32 bytes

// ✅ EFFICIENT: Largest to smallest
struct Efficient {
    b: u64,     // 8 bytes
    d: u64,     // 8 bytes
    a: u8,      // 1 byte
    c: u8,      // 1 byte + 6 padding
}  // Total: 24 bytes

// Let the compiler optimize:
#[repr(C)]  // Use only if you need specific layout (FFI)
struct ExactLayout { /* ... */ }

// Default (no repr): Compiler may reorder for efficiency

// Check with:
println!("Size: {}", std::mem::size_of::<MyStruct>());
println!("Align: {}", std::mem::align_of::<MyStruct>());
```

---

## Summary: Performance Checklist

**Allocations**:
- [ ] Pre-allocate with `Vec::with_capacity`
- [ ] Use `&str`/`&[T]` parameters, not `String`/`Vec<T>`
- [ ] Consider `Cow` for conditional allocation
- [ ] Avoid `clone()` in hot paths

**Collections**:
- [ ] Choose appropriate collection for access pattern
- [ ] Use `entry()` API for map updates
- [ ] Consider `SmallVec` for usually-small collections

**General**:
- [ ] Prefer iterators over index loops
- [ ] Profile before optimizing
- [ ] Benchmark with criterion
- [ ] Consider `#[inline]` for tiny hot functions

---

*See also: [11-anti-patterns.md](11-anti-patterns.md#ap-14) for performance anti-patterns.*
