# Concurrency and Async Patterns

Guidelines for async programming, Send/Sync traits, futures, and concurrency patterns in Rust.

## Table of Contents

- [Send and Sync](#send-and-sync)
- [Async Patterns](#async-patterns)
- [Yield Points](#yield-points)
- [Future Design](#future-design)
- [Services and Clone](#services-and-clone)

---

## Send and Sync

### Types Should Be Send

**Strength**: SHOULD

**Summary**: Public types should be `Send` for compatibility with Tokio and most async runtimes.

**Example**:
```rust
// Good - Send future
async fn process_data(data: Vec<u8>) -> Result<(), Error> {
    // Vec is Send, future is Send
    database.store(data).await?;
    Ok(())
}

// Bad - holding Rc across await makes future !Send
async fn process_data_bad(data: Vec<u8>) -> Result<(), Error> {
    let rc = Rc::new(data); // Rc is !Send
    database.store(rc.clone()).await?; // ❌ future is !Send
    Ok(())
}

// Good - use Arc instead of Rc for shared ownership
async fn process_data_good(data: Vec<u8>) -> Result<(), Error> {
    let arc = Arc::new(data); // Arc is Send
    database.store(arc.clone()).await?; // ✓ future is Send
    Ok(())
}
```

**Testing Send**:
```rust
struct MyFuture {
    // ...
}

impl Future for MyFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // ...
    }
}

// Assert that the future is Send
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<MyFuture>();
};

// For async functions, test at call site
#[test]
fn test_async_fn_is_send() {
    fn assert_send<T: Send>(_: T) {}
    assert_send(my_async_function());
}
```

**Rationale**: Most async runtimes (especially Tokio) require futures to be `Send` to move them between threads. Non-Send types like `Rc<T>`, `RefCell<T>`, and raw pointers held across `.await` points make futures `!Send`, breaking compatibility.

**Exceptions**: Types used instantaneously (not held across `.await`) may be `!Send` if there's a good reason.

**See also**: M-TYPES-SEND

---

### The Cost of Send

**Strength**: CONSIDER

**Summary**: Atomics have measurable cost, but it's usually worth it for ecosystem compatibility.

**Example**:
```rust
// Ideal for thread-per-core (no contention)
struct FastCounter {
    count: Rc<RefCell<usize>>, // No atomics!
}

// Reality: need Send for Tokio compatibility
struct Counter {
    count: Arc<AtomicUsize>, // Small overhead, but widely compatible
}

// The performance difference matters only in tight loops
// accessing the same atomic frequently (< 64 words apart)
```

**Rationale**: While thread-per-core designs can theoretically avoid atomic overhead, the lack of `Send` types means reinventing the ecosystem. Uncontended atomics have negligible overhead in most real-world code.

**Guideline**: Use `Send` types (Arc, Mutex) by default. Only optimize to `!Send` (Rc, RefCell) if profiling shows atomic operations are a bottleneck.

**See also**: M-TYPES-SEND, "The Cost of Send" section

---

## Async Patterns

### Async Functions Return Send Futures

**Strength**: MUST (for libraries)

**Summary**: Library async functions must return `Send` futures to work with work-stealing runtimes.

**Example**:
```rust
// Good - returns Send future
pub async fn fetch_data(url: &str) -> Result<Data, Error> {
    let response = http_client.get(url).await?;
    Ok(response.json().await?)
}

// Bad - returns !Send future if http_client is !Send
pub async fn fetch_data_bad(url: &str) -> Result<Data, Error> {
    let client = Rc::new(HttpClient::new()); // Rc is !Send
    let response = client.get(url).await?; // ❌ Future is !Send
    Ok(response.json().await?)
}

// Explicit Send bound when needed
pub fn fetch_data_explicit(url: String) -> impl Future<Output = Result<Data, Error>> + Send {
    async move {
        let response = HTTP_CLIENT.get(&url).await?;
        Ok(response.json().await?)
    }
}
```

**Rationale**: Work-stealing runtimes like Tokio need to move futures between threads. Non-Send futures cannot be used in `.spawn()` or similar APIs, severely limiting their utility.

**See also**: M-TYPES-SEND

---

## Yield Points

### Long-Running Tasks Should Yield

**Strength**: MUST

**Summary**: CPU-bound work in async functions must include `yield_now().await` points to avoid starving other tasks.

**Example**:
```rust
// Bad - blocks the runtime
async fn process_large_file(file: File) {
    let data = file.read_all().await;
    
    for item in data {
        expensive_computation(item); // ❌ No yield point!
    }
}

// Good - yields periodically
async fn process_large_file(file: File) {
    let data = file.read_all().await;
    
    for (i, item) in data.iter().enumerate() {
        expensive_computation(item);
        
        // Yield every 100 items
        if i % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
}

// Better - batch processing with yields
async fn process_large_file_batched(file: File) {
    let data = file.read_all().await;
    
    for chunk in data.chunks(100) {
        for item in chunk {
            expensive_computation(item);
        }
        tokio::task::yield_now().await;
    }
}
```

**How often to yield**:
- Target: 10-100μs of CPU work between yields
- Too frequent: Overhead from task switching
- Too infrequent: Starves other tasks

**Runtime APIs**:
```rust
// Check if we should yield (Tokio)
if !tokio::task::yield_now().has_budget_remaining() {
    tokio::task::yield_now().await;
}
```

**Rationale**: In cooperative multitasking, tasks must voluntarily yield. Long-running CPU work prevents the runtime from scheduling other tasks, causing latency spikes and poor utilization.

**See also**: M-YIELD-POINTS

---

### I/O Provides Natural Yield Points

**Strength**: Note

**Summary**: Async functions that regularly perform I/O don't need explicit yields—`.await` provides yield points.

**Example**:
```rust
// No explicit yields needed - .await provides them
async fn process_requests(stream: TcpStream) {
    loop {
        let request = read_request(&stream).await; // Yield point
        let response = process(request);
        write_response(&stream, response).await; // Yield point
    }
}

// Only need explicit yields for CPU-bound sections
async fn process_mixed_workload() {
    let data = fetch_from_network().await; // Yield point
    
    // CPU-bound processing needs yields
    for (i, item) in data.iter().enumerate() {
        compute_heavy(item);
        if i % 100 == 0 {
            yield_now().await; // Explicit yield
        }
    }
    
    save_to_network(result).await; // Yield point
}
```

**Rationale**: Every `.await` is a potential yield point where the runtime can schedule other tasks. I/O-heavy code naturally yields frequently.

---

## Future Design

### Futures Should Complete Eventually

**Strength**: MUST

**Summary**: Futures must not block forever without making progress or being cancelled.

**Example**:
```rust
// Bad - infinite loop with no yields
async fn bad_future() {
    loop {
        // ❌ Never awaits, never yields, never returns
        compute_something();
    }
}

// Good - yields in loop
async fn good_future() {
    loop {
        compute_something();
        tokio::task::yield_now().await;
        
        if should_stop() {
            break;
        }
    }
}

// Good - finite work
async fn finite_future() {
    for i in 0..1000 {
        compute_something(i);
        if i % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
}
```

**Rationale**: Futures that never complete and never yield are runtime deadlocks. The runtime cannot make progress on other work.

---

## Services and Clone

### Service Types Should Be Clone

**Strength**: SHOULD

**Summary**: Service-level types should implement cheap `Clone` using `Arc` internally to enable sharing.

**Example**:
```rust
// Good - cheap clone via Arc
struct HttpClient {
    inner: Arc<ClientInner>,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ClientInner::new()),
        }
    }
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// Usage: easy to share across async tasks
async fn handler(client: HttpClient) {
    // Clone is cheap - just increments Arc refcount
    let client_clone = client.clone();
    
    tokio::spawn(async move {
        client_clone.get("https://example.com").await
    });
}
```

**Pattern**:
```rust
// Internal state
struct ServiceInner {
    // Actual state and logic
}

// Public handle
#[derive(Clone)]
pub struct Service {
    inner: Arc<ServiceInner>,
}

impl Service {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ServiceInner::new()),
        }
    }
    
    // Methods forward to inner
    pub fn operation(&self) {
        self.inner.operation()
    }
}
```

**Rationale**: Async code frequently needs to move services into futures/tasks. Cloneable services enable ergonomic sharing without explicit Arc wrapping at every use site.

**See also**: M-SERVICES-CLONE

---

## Testing Async Code

### Assert Futures Are Send

**Strength**: SHOULD

**Summary**: Test that your async functions produce `Send` futures, especially library code.

**Example**:
```rust
pub async fn fetch_user(id: UserId) -> Result<User, Error> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn fetch_user_is_send() {
        fn assert_send<T: Send>(_: T) {}
        assert_send(fetch_user(UserId(1)));
    }
    
    // Or use a const assertion
    const _: () = {
        const fn assert_send<T: Send>() {}
        // This will fail to compile if future is !Send
        assert_send::<impl Future<Output = Result<User, Error>>>();
    };
}
```

**Rationale**: It's easy to accidentally break `Send` by holding the wrong type across an `.await`. Tests catch this before it reaches users.

---

## Blocking Operations

### Never Block in Async

**Strength**: MUST

**Summary**: Never call blocking operations in async functions without using `spawn_blocking`.

**Example**:
```rust
// Bad - blocks the async runtime
async fn process_file(path: PathBuf) -> Result<String, Error> {
    let contents = std::fs::read_to_string(path)?; // ❌ Blocking I/O!
    Ok(contents)
}

// Good - use async I/O
async fn process_file(path: PathBuf) -> Result<String, Error> {
    let contents = tokio::fs::read_to_string(path).await?;
    Ok(contents)
}

// When blocking is unavoidable, spawn_blocking
async fn compute_intensive(data: Vec<u8>) -> Result<Hash, Error> {
    tokio::task::spawn_blocking(move || {
        // Runs on blocking thread pool
        expensive_hash_function(&data)
    }).await?
}
```

**Common blocking operations**:
- `std::fs::*` (use `tokio::fs::*` or `async_fs::*`)
- `std::net::*` (use `tokio::net::*`)
- Long-running CPU work (use `spawn_blocking`)
- Mutex locks that may be held (use `tokio::sync::Mutex`)

**Rationale**: Blocking operations stall the entire async executor thread, preventing other tasks from running. This causes severe latency issues and poor throughput.

---

## Summary Table

| Pattern | Strength | Key Insight |
|---------|----------|-------------|
| Types should be Send | SHOULD | Required for Tokio and work-stealing runtimes |
| Async functions return Send futures | MUST | Critical for library compatibility |
| Long CPU work must yield | MUST | Use `yield_now().await` every 10-100μs of work |
| I/O provides natural yields | Note | `.await` on I/O is a yield point |
| Services implement Clone | SHOULD | Use `Arc<Inner>` pattern for cheap clones |
| Test futures are Send | SHOULD | Prevent accidental !Send in libraries |
| Never block in async | MUST | Use async I/O or `spawn_blocking` |

---

## Related Guidelines

- **Type Design**: See `05-type-design.md` for Send/Sync type design
- **Performance**: See `08-performance.md` for throughput optimization
- **Anti-patterns**: See `11-anti-patterns.md` for async mistakes

---

## External References

- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- Pragmatic Rust Guidelines: M-TYPES-SEND, M-YIELD-POINTS, M-SERVICES-CLONE
