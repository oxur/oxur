# Performance Guidelines

Patterns for optimizing throughput, identifying hot paths, and managing CPU budgets in Rust applications.

## Table of Contents

- [Throughput Optimization](#throughput-optimization)
- [Hot Path Identification](#hot-path-identification)
- [Yield Points](#yield-points)
- [Application Setup](#application-setup)

---

## Throughput Optimization

### Optimize for Throughput, Avoid Empty Cycles

**Strength**: SHOULD

**Summary**: Design for items-per-CPU-cycle, not single-item latency; avoid hot-spinning and frequent task switching.

**Example**:
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

**Best practices**:
- Partition work ahead of time
- Let threads/tasks process chunks independently
- Sleep or yield when no work is available
- Design APIs for batched operations
- Perform work via batched APIs where available
- Exploit CPU caches through temporal/spatial locality

**Avoid**:
- Hot-spinning to receive individual items faster
- Single-item processing when batching is possible
- Work-stealing for individual items (overhead often exceeds benefit)
- Frequent contention on shared state

**See also**: M-THROUGHPUT

---

## Hot Path Identification

### Identify, Profile, Optimize the Hot Path Early

**Strength**: MUST (for performance-critical code)

**Summary**: For performance-critical crates, identify hot paths early, create benchmarks, and profile regularly.

**Example**:
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

**Process**:
1. Identify if your crate is performance-critical
2. Create benchmarks for suspected hot paths
3. Profile with CPU and allocation profiling tools
4. Document performance-sensitive areas
5. Re-profile after changes

**Common hot path issues**:
- Frequent allocations (use bump allocators or reuse buffers)
- String cloning and growth (use `&str`, pre-allocate capacity)
- Collection cloning (use references or `Arc`)
- Re-hashing identical data (cache hash values)
- Rust's default hasher where collision resistance isn't needed

**Typical wins**:
- 15-50% improvement from reducing String allocations
- String-heavy hot paths can see 50%+ improvement when optimized

**See also**: M-HOTPATH

---

### Enable Debug Symbols for Profiling

**Strength**: MUST (for benchmarks)

**Summary**: Add debug symbols to benchmark profile for meaningful profiler output.

**Example**:
```toml
# In Cargo.toml
[profile.bench]
debug = 1  # Enable line-level debug info

# Or more comprehensive:
[profile.bench]
debug = true  # Full debug info
```

**Rationale**: Without debug symbols, profilers show assembly addresses instead of function names and line numbers. Debug info adds to compile time but is essential for actionable profiling.

**See also**: M-HOTPATH

---

## Yield Points

### Long-Running Tasks Should Have Yield Points

**Strength**: MUST

**Summary**: CPU-bound async tasks must yield periodically to prevent starving concurrent tasks.

**Example**:
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

**Guidelines**:
- I/O-heavy tasks: Natural yield points at `.await`
- CPU-bound tasks: Explicit `yield_now().await` every 10-100μs
- Variable-duration work: Check `has_budget_remaining()`

**Rule of thumb**: 10-100μs of continuous CPU work between yields balances switching overhead (<1%) against responsiveness.

**See also**: M-YIELD-POINTS

---

## Application Setup

### Use Mimalloc for Applications

**Strength**: SHOULD

**Summary**: Applications should use mimalloc as the global allocator for improved performance.

**Example**:
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

**Notes**:
- Only for applications (binaries), not libraries
- Zero configuration required
- May not help if allocations aren't a bottleneck
- Other allocators to consider: jemalloc, snmalloc

**See also**: M-MIMALLOC-APP

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
