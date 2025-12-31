# Performance Guidelines

Patterns for optimizing throughput, identifying hot paths, and managing CPU budgets in Rust applications.


## PF-01: Avoid `clone()` in Hot Paths

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

## PF-02: Avoid Unnecessary Allocations

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

## PF-03: Benchmark Before Optimizing

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

---

## PF-04: Choose the Right Collection

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

---

## PF-05: Cloning Collections

**Strength**: SHOULD

**Summary**: 

```rust
// Bad - expensive clone
fn process(data: Vec<Item>) -> Vec<Item> {
    let mut result = data.clone();
    // ...
    result
}

// Good - take ownership or use references
fn process(mut data: Vec<Item>) -> Vec<Item> {
    // ...
    data
}
```

---

## PF-06: Default Hasher

**Strength**: SHOULD

**Summary**: 

```rust
use std::collections::HashMap;
use rustc_hash::FxHashMap;  // Faster non-cryptographic hash

// Bad - default hasher has collision resistance overhead
let map: HashMap<u64, String> = HashMap::new();

// Good - faster hasher when DOS resistance not needed
let map: FxHashMap<u64, String> = FxHashMap::default();
```

---

## PF-07: Enable Debug Symbols for Profiling

**Strength**: MUST

**Summary**: Add debug symbols to benchmark profile for meaningful profiler output.

**Rationale**: Without debug symbols, profilers show assembly addresses instead of function names and line numbers. Debug info adds to compile time but is essential for actionable profiling.

**See also**: M-HOTPATH

---

## PF-08: Identify, Profile, Optimize the Hot Path Early

**Strength**: MUST

**Summary**: For performance-critical crates, identify hot paths early, create benchmarks, and profile regularly.

```rust
// In benches/benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    
    for size in [100, 1000, 10000] {
        let input = generate_input(size);
        
        group.bench_with_input(
            BenchmarkId::new("optimized", size),
            &input,
            |b, input| b.iter(|| parse_optimized(input))
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);

// In Cargo.toml - enable debug symbols for profiling
// [profile.bench]
// debug = 1

// Profile with tools
// - cargo bench
// - Intel VTune (Windows)
// - Superluminal (Windows)  
// - perf (Linux)
// - Instruments (macOS)
```

**Rationale**: Premature optimization is wasteful, but identifying hot paths early prevents architectural decisions that are hard to change later. Regular profiling reveals actual bottlenecks rather than assumed ones.

**See also**: M-HOTPATH

---

## PF-09: Inline Small Functions

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

## PF-10: Long-Running Tasks Should Have Yield Points

**Strength**: MUST

**Summary**: CPU-bound async tasks must yield periodically to prevent starving concurrent tasks.

```rust
use tokio::task;

// Bad - blocks the runtime
async fn process_large_file(file: File) {
    let data = read_all(file).await;
    
    // Long CPU-bound work with no yield
    for item in data.iter() {
        expensive_computation(item);
    }
    // Other tasks starve!
}

// Good - yield periodically
async fn process_large_file_yielding(file: File) {
    let data = read_all(file).await;
    
    for item in data.iter() {
        expensive_computation(item);
        
        // Yield after each item
        tokio::task::yield_now().await;
    }
}

// Better - batch yielding
async fn process_large_file_batched(file: File) {
    let data = read_all(file).await;
    
    for chunk in data.chunks(100) {
        for item in chunk {
            expensive_computation(item);
        }
        
        // Yield after processing batch
        tokio::task::yield_now().await;
    }
}

// Best - use budget API when available
async fn process_with_budget(file: File) {
    let data = read_all(file).await;
    
    for item in data.iter() {
        expensive_computation(item);
        
        // Yield only when budget exhausted
        if !tokio::runtime::Handle::current().has_budget_remaining() {
            tokio::task::yield_now().await;
        }
    }
}
```

**Rationale**: Thread-per-core runtimes can't preempt tasks. Long CPU-bound work without yields starves other tasks, causing high tail latencies.

**See also**: M-YIELD-POINTS

---

## PF-11: Memory Layout Optimization

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

## PF-12: Optimize for Throughput, Avoid Empty Cycles

**Strength**: SHOULD

**Summary**: Design for items-per-CPU-cycle, not single-item latency; avoid hot-spinning and frequent task switching.

```rust
use tokio::time::{sleep, Duration};

// Bad - processing items one at a time with empty cycles
async fn process_items_slow(rx: Receiver<Item>) {
    loop {
        // Hot-spinning waiting for single item
        if let Ok(item) = rx.try_recv() {
            process_one(item).await;
        }
        // Empty cycle - wasted CPU
        sleep(Duration::from_micros(1)).await;
    }
}

// Good - batch processing
async fn process_items_batched(mut rx: Receiver<Item>) {
    let mut batch = Vec::with_capacity(100);
    
    loop {
        // Collect a batch of items
        while batch.len() < 100 {
            match rx.try_recv() {
                Ok(item) => batch.push(item),
                Err(_) if !batch.is_empty() => break,
                Err(_) => {
                    // Only sleep when no work available
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
            }
        }
        
        if !batch.is_empty() {
            // Process entire batch together
            process_batch(&batch).await;
            batch.clear();
        }
    }
}

// Good - partition work upfront
async fn process_partitioned(items: Vec<Item>, num_workers: usize) {
    let chunk_size = items.len() / num_workers;
    let chunks: Vec<_> = items.chunks(chunk_size).collect();
    
    // Each worker processes its chunk independently
    let handles: Vec<_> = chunks.into_iter()
        .map(|chunk| {
            tokio::spawn(async move {
                for item in chunk {
                    process_one(item).await;
                }
            })
        })
        .collect();
    
    // Wait for all workers
    for handle in handles {
        handle.await.unwrap();
    }
}
```

**Rationale**: Hot-spinning wastes CPU cycles that could do useful work. Batching and partitioning maximize CPU utilization while minimizing overhead from task switching and synchronization.

**See also**: M-THROUGHPUT

---

## PF-13: Performance Checklist

**Strength**: SHOULD

**Summary**: 

```rust
// 1. Benchmark hot paths
#[bench]
fn bench_hot_path() { /* ... */ }

// 2. Enable profiling
// [profile.bench]
// debug = 1

// 3. Identify bottlenecks via profiling
// - CPU time (VTune, Superluminal, perf)
// - Allocations (flamegraph, heaptrack)
// - Lock contention (if applicable)

// 4. Common optimizations:
// - Reduce allocations
//   * Reuse buffers
//   * Use &str instead of String
//   * Pre-allocate capacity
// - Reduce cloning
//   * Use references
//   * Use Arc for shared data
// - Cache expensive computations
// - Use faster hash algorithm if collision resistance not needed

// 5. Yield in long tasks
async fn long_task() {
    for batch in data.chunks(100) {
        process(batch);
        tokio::task::yield_now().await;
    }
}

// 6. Use mimalloc
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

---

## PF-14: Prefer Iterators Over Index Loops

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

## PF-15: SIMD and Auto-Vectorization

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

## PF-16: Small String Optimization Awareness

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

## PF-17: String Allocations

**Strength**: SHOULD

**Summary**: 

```rust
// Bad - allocates every iteration
for i in 0..1000 {
    let msg = format!("Processing item {}", i);
    process(&msg);
}

// Good - reuse buffer
let mut msg = String::with_capacity(50);
for i in 0..1000 {
    use std::fmt::Write;
    msg.clear();
    write!(&mut msg, "Processing item {}", i).unwrap();
    process(&msg);
}
```

---

## PF-18: Use `&str` and `&[T]` Parameters

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

## PF-19: Use `Cow` for Conditional Allocation

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

## PF-20: Use `Entry` API for Map Updates

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

## PF-21: Use Mimalloc for Applications

**Strength**: SHOULD

**Summary**: Applications should use mimalloc as the global allocator for improved performance.

```rust
// In Cargo.toml
[dependencies]
mimalloc = "0.1"

// In main.rs or lib.rs
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    // Application code
}
```

**Rationale**: Mimalloc typically provides 10-25% performance improvement on allocation-heavy workloads with minimal integration effort.

**See also**: M-MIMALLOC-APP

---

## PF-22: Zero-Copy Parsing

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

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| Optimize for throughput | SHOULD | Items/cycle, not latency |
| Batch processing | SHOULD | Process chunks, not single items |
| Partition work upfront | SHOULD | Independent chunks per worker |
| Profile early | MUST* | *For performance-critical code |
| Enable debug symbols | MUST | For profiling benchmarks |
| Yield in CPU-bound tasks | MUST | Every 10-100μs of CPU work |
| Use budget APIs | SHOULD | For variable-duration work |
| Use mimalloc | SHOULD | For applications |

## Performance Checklist

For performance-critical code:

```rust
// 1. Benchmark hot paths
#[bench]
fn bench_hot_path() { /* ... */ }

// 2. Enable profiling
// [profile.bench]
// debug = 1

// 3. Identify bottlenecks via profiling
// - CPU time (VTune, Superluminal, perf)
// - Allocations (flamegraph, heaptrack)
// - Lock contention (if applicable)

// 4. Common optimizations:
// - Reduce allocations
//   * Reuse buffers
//   * Use &str instead of String
//   * Pre-allocate capacity
// - Reduce cloning
//   * Use references
//   * Use Arc for shared data
// - Cache expensive computations
// - Use faster hash algorithm if collision resistance not needed

// 5. Yield in long tasks
async fn long_task() {
    for batch in data.chunks(100) {
        process(batch);
        tokio::task::yield_now().await;
    }
}

// 6. Use mimalloc
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

## Common Performance Pitfalls

### String Allocations

```rust
// Bad - allocates every iteration
for i in 0..1000 {
    let msg = format!("Processing item {}", i);
    process(&msg);
}

// Good - reuse buffer
let mut msg = String::with_capacity(50);
for i in 0..1000 {
    use std::fmt::Write;
    msg.clear();
    write!(&mut msg, "Processing item {}", i).unwrap();
    process(&msg);
}
```

### Cloning Collections

```rust
// Bad - expensive clone
fn process(data: Vec<Item>) -> Vec<Item> {
    let mut result = data.clone();
    // ...
    result
}

// Good - take ownership or use references
fn process(mut data: Vec<Item>) -> Vec<Item> {
    // ...
    data
}
```

### Default Hasher

```rust
use std::collections::HashMap;
use rustc_hash::FxHashMap;  // Faster non-cryptographic hash

// Bad - default hasher has collision resistance overhead
let map: HashMap<u64, String> = HashMap::new();

// Good - faster hasher when DOS resistance not needed
let map: FxHashMap<u64, String> = FxHashMap::default();
```

## Related Guidelines

- **Concurrency**: See `07-concurrency-async.md` for async patterns
- **Type Design**: See `05-type-design.md` for zero-cost abstractions

## External References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion.rs](https://github.com/bheisler/criterion.rs)
- Pragmatic Rust: M-THROUGHPUT, M-HOTPATH, M-YIELD-POINTS, M-MIMALLOC-APP