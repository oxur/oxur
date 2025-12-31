# Concurrency and Async Patterns

Guidelines for async programming, Send/Sync traits, futures, and concurrency patterns in Rust.


## CA-01: Assert Futures Are Send

**Strength**: SHOULD

**Summary**: Test that your async functions produce `Send` futures, especially library code.

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

---

## CA-02: Async Functions Return Send Futures

**Strength**: MUST

**Summary**: Library async functions must return `Send` futures to work with work-stealing runtimes.

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

## CA-03: Async Task Spawning Patterns

**Strength**: SHOULD

**Summary**: Choose the right spawning method for your use case.

```rust
use tokio::task;

// ✅ spawn: Independent task, runs to completion
async fn background_work() {
    task::spawn(async {
        loop {
            do_periodic_work().await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
    // Task continues running after this function returns
}

// ✅ spawn with handle: Wait for result
async fn compute_parallel() -> (i32, i32) {
    let handle1 = task::spawn(async { expensive_compute_1().await });
    let handle2 = task::spawn(async { expensive_compute_2().await });
    
    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();
    (result1, result2)
}

// ✅ spawn_blocking: For sync/CPU-bound code
async fn hash_file(path: PathBuf) -> Hash {
    task::spawn_blocking(move || {
        let data = std::fs::read(&path).unwrap();
        compute_hash(&data)
    }).await.unwrap()
}

// ✅ spawn_local: For !Send futures (single-threaded runtime)
// Use when the future can't be sent between threads

// ✅ JoinSet: Manage multiple tasks
use tokio::task::JoinSet;

async fn process_urls(urls: Vec<String>) -> Vec<Response> {
    let mut set = JoinSet::new();
    
    for url in urls {
        set.spawn(async move {
            fetch(&url).await
        });
    }
    
    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.unwrap());
    }
    results
}
```

---

## CA-04: Async Trait Methods

**Strength**: SHOULD

**Summary**: Use `async-trait` crate or native async traits (Rust 1.75+).

```rust
// Rust 1.75+: Native async in traits (with limitations)
trait Service {
    async fn call(&self, request: Request) -> Response;
}

impl Service for MyService {
    async fn call(&self, request: Request) -> Response {
        // ...
    }
}

// For trait objects, use async-trait crate:
use async_trait::async_trait;

#[async_trait]
trait DynService: Send + Sync {
    async fn call(&self, request: Request) -> Response;
}

#[async_trait]
impl DynService for MyService {
    async fn call(&self, request: Request) -> Response {
        // ...
    }
}

// Now works as trait object:
async fn use_service(service: &dyn DynService) {
    service.call(request).await;
}
```

---

## CA-05: Avoid Blocking in Async Code

**Strength**: MUST

**Summary**: Never use blocking I/O or long computations in async functions.

```rust
// ❌ BAD: Blocking I/O in async context
async fn read_config() -> Config {
    // This blocks the entire async runtime thread!
    let contents = std::fs::read_to_string("config.json").unwrap();
    serde_json::from_str(&contents).unwrap()
}

// ✅ GOOD: Use async I/O
async fn read_config() -> Result<Config, Error> {
    let contents = tokio::fs::read_to_string("config.json").await?;
    Ok(serde_json::from_str(&contents)?)
}

// ❌ BAD: CPU-intensive work in async
async fn hash_password(password: &str) -> Hash {
    // Blocks the runtime!
    argon2::hash(password)
}

// ✅ GOOD: Use spawn_blocking for CPU work
async fn hash_password(password: String) -> Hash {
    tokio::task::spawn_blocking(move || {
        argon2::hash(&password)
    }).await.unwrap()
}

// ❌ BAD: Blocking mutex in async
async fn update_cache() {
    let guard = CACHE.lock().unwrap();  // std::sync::Mutex blocks!
    // ...
}

// ✅ GOOD: Use async-aware mutex
use tokio::sync::Mutex;

async fn update_cache() {
    let guard = CACHE.lock().await;  // Yields instead of blocking
    // ...
}
```

---

## CA-06: Cancellation Safety

**Strength**: SHOULD

**Summary**: Understand what happens when a future is dropped.

```rust
// ⚠️ CANCELLATION UNSAFE: Partial state on drop
async fn transfer_unsafe(from: &mut Account, to: &mut Account, amount: u64) {
    from.withdraw(amount).await;  // If cancelled here...
    // ...money is withdrawn but not deposited!
    to.deposit(amount).await;
}

// ✅ CANCELLATION SAFE: Transactional
async fn transfer_safe(from: &mut Account, to: &mut Account, amount: u64) -> Result<(), Error> {
    // Use a transaction or ensure atomicity
    let tx = Transaction::begin().await?;
    tx.withdraw(from, amount).await?;
    tx.deposit(to, amount).await?;
    tx.commit().await?;  // All or nothing
    Ok(())
}

// ✅ CANCELLATION SAFE: select! considerations
use tokio::select;

async fn with_timeout() {
    select! {
        // Both branches should be cancellation-safe
        result = cancellation_safe_operation() => {
            handle(result);
        }
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            println!("timeout");
        }
    }
}
```

---

## CA-07: Channels for Task Communication

**Strength**: SHOULD

**Summary**: Use channels to communicate between tasks instead of shared state.

```rust
use tokio::sync::{mpsc, oneshot, broadcast, watch};

// ✅ mpsc: Multiple producers, single consumer
async fn worker_pool() {
    let (tx, mut rx) = mpsc::channel::<Job>(100);
    
    // Spawn workers
    for _ in 0..4 {
        let mut rx = rx.clone(); // ERROR: mpsc::Receiver isn't Clone
    }
    // Actually, for worker pools:
    let (tx, rx) = mpsc::channel::<Job>(100);
    let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
    
    // Or use a proper work-stealing queue
}

// ✅ oneshot: Single value, single use
async fn request_response() {
    let (tx, rx) = oneshot::channel();
    
    tokio::spawn(async move {
        let result = compute().await;
        let _ = tx.send(result);
    });
    
    let response = rx.await.unwrap();
}

// ✅ broadcast: Multiple consumers, each gets all messages
async fn pub_sub() {
    let (tx, _rx) = broadcast::channel::<Event>(100);
    
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();
    
    tx.send(Event::new()).unwrap();
    // Both rx1 and rx2 receive the event
}

// ✅ watch: Single value that can be updated, multiple readers
async fn config_reload() {
    let (tx, rx) = watch::channel(Config::default());
    
    // Readers
    tokio::spawn(async move {
        let mut rx = rx.clone();
        loop {
            rx.changed().await.unwrap();
            let config = rx.borrow().clone();
            apply_config(config);
        }
    });
    
    // Writer
    tx.send(new_config).unwrap();
}
```

---

## CA-08: Choose the Right Synchronization Primitive

**Strength**: SHOULD

**Summary**: Use the simplest primitive that meets your needs.

```rust
use std::sync::{Mutex, RwLock, Arc, atomic::{AtomicU64, Ordering}};

// ✅ ATOMIC: For simple counters/flags
struct Counter {
    value: AtomicU64,
}

impl Counter {
    fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }
    
    fn get(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }
}

// ✅ MUTEX: For exclusive access to complex data
struct SharedState {
    data: Mutex<Vec<String>>,
}

impl SharedState {
    fn push(&self, item: String) {
        self.data.lock().unwrap().push(item);
    }
}

// ✅ RWLOCK: Many readers OR one writer
struct Cache {
    data: RwLock<HashMap<String, Value>>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<Value> {
        self.data.read().unwrap().get(key).cloned()
    }
    
    fn set(&self, key: String, value: Value) {
        self.data.write().unwrap().insert(key, value);
    }
}

// ✅ ARC: Shared ownership across threads
let shared = Arc::new(SharedState::new());
let shared_clone = Arc::clone(&shared);
std::thread::spawn(move || {
    shared_clone.push("from thread".into());
});
```

---

## CA-09: Error Handling in Async Code

**Strength**: SHOULD

**Summary**: Handle errors appropriately in spawned tasks.

```rust
// ❌ BAD: Panics in spawned tasks are silent
tokio::spawn(async {
    might_panic().await;  // Panic is swallowed!
});

// ✅ GOOD: Handle JoinError
let handle = tokio::spawn(async {
    might_fail().await
});

match handle.await {
    Ok(Ok(result)) => println!("Success: {:?}", result),
    Ok(Err(e)) => println!("Task returned error: {:?}", e),
    Err(e) => println!("Task panicked or was cancelled: {:?}", e),
}

// ✅ GOOD: Use JoinSet for multiple tasks
let mut set = JoinSet::new();
set.spawn(task1());
set.spawn(task2());

while let Some(result) = set.join_next().await {
    match result {
        Ok(Ok(value)) => handle_success(value),
        Ok(Err(e)) => handle_error(e),
        Err(e) => handle_panic(e),
    }
}

// ✅ GOOD: Propagate errors with ?
async fn orchestrate() -> Result<(), Error> {
    let handle = tokio::spawn(async {
        fallible_work().await
    });
    
    handle.await??;  // First ? for JoinError, second for inner Result
    Ok(())
}
```

---

## CA-10: Executor-Agnostic Code

**Strength**: CONSIDER

**Summary**: Write async code that works with any runtime when possible.

```rust
// ❌ TIED TO TOKIO: Uses tokio-specific types
async fn fetch_tokio() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    let data = tokio::fs::read("file.txt").await;
}

// ✅ EXECUTOR-AGNOSTIC: Uses traits/abstractions
use futures::io::AsyncReadExt;

async fn fetch_generic<R: AsyncReadExt + Unpin>(mut reader: R) -> Vec<u8> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await.unwrap();
    buf
}

// ✅ PRACTICAL: Accept runtime as parameter
pub struct Client<R: Runtime> {
    runtime: R,
}

pub trait Runtime {
    fn spawn<F: Future + Send + 'static>(&self, future: F);
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()>;
}
```

---

## CA-11: Futures Should Complete Eventually

**Strength**: MUST

**Summary**: Futures must not block forever without making progress or being cancelled.

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

---

## CA-12: I/O Provides Natural Yield Points

**Strength**: SHOULD

**Summary**: Async functions that regularly perform I/O don't need explicit yields—`.await` provides yield points.

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

---

## CA-13: Long-Running Tasks Should Yield

**Strength**: MUST

**Summary**: CPU-bound work in async functions must include `yield_now().await` points to avoid starving other tasks.

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

```rust
// Check if we should yield (Tokio)
if !tokio::task::yield_now().has_budget_remaining() {
    tokio::task::yield_now().await;
}
```

**Rationale**: In cooperative multitasking, tasks must voluntarily yield. Long-running CPU work prevents the runtime from scheduling other tasks, causing latency spikes and poor utilization.

**See also**: M-YIELD-POINTS

---

## CA-14: Never Block in Async

**Strength**: MUST

**Summary**: Never call blocking operations in async functions without using `spawn_blocking`.

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

**Rationale**: Blocking operations stall the entire async executor thread, preventing other tasks from running. This causes severe latency issues and poor throughput.

---

---

## CA-15: Service Types Should Be Clone

**Strength**: SHOULD

**Summary**: Service-level types should implement cheap `Clone` using `Arc` internally to enable sharing.

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

## CA-16: Structured Concurrency with Scoped Threads

**Strength**: SHOULD

**Summary**: Prefer scoped threads when possible for safer lifetimes.

```rust
// ❌ PROBLEMATIC: Regular threads need 'static data
fn process_items(items: &[Item]) {
    let handles: Vec<_> = items.iter().map(|item| {
        // ERROR: `item` doesn't live long enough
        std::thread::spawn(|| process(item))
    }).collect();
}

// ✅ GOOD: Scoped threads can borrow local data
fn process_items(items: &[Item]) {
    std::thread::scope(|s| {
        for item in items {
            s.spawn(|| process(item));  // Can borrow `item`!
        }
        // All threads joined automatically at end of scope
    });
}

// ✅ GOOD: Parallel iterator with rayon
use rayon::prelude::*;

fn process_items(items: &[Item]) {
    items.par_iter().for_each(|item| {
        process(item);
    });
}
```

---

## CA-17: The Cost of Send

**Strength**: CONSIDER

**Summary**: Atomics have measurable cost, but it's usually worth it for ecosystem compatibility.

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

**See also**: M-TYPES-SEND, "The Cost of Send" section

---

## CA-18: Types Should Be Send

**Strength**: SHOULD

**Summary**: Public types should be `Send` for compatibility with Tokio and most async runtimes.

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

**See also**: M-TYPES-SEND

---

## CA-19: Understand `Send` and `Sync`

**Strength**: MUST

**Summary**: `Send` means transferable between threads; `Sync` means shareable between threads.

```rust
use std::sync::{Arc, Mutex, Rc, RefCell};
use std::cell::Cell;

// Send: Safe to MOVE to another thread
// T: Send means you can do: thread::spawn(move || use(value))

// Sync: Safe to SHARE references between threads  
// T: Sync means &T: Send (references can be sent)

// ✅ SEND + SYNC: Most types
struct SafeData {
    value: i32,
    name: String,
}
// Automatically Send + Sync (all fields are Send + Sync)

// ✅ SEND but not SYNC: Interior mutability without sync
struct SendOnly {
    data: Cell<i32>,  // Cell is Send but not Sync
}

// ❌ NOT SEND: Contains non-Send types
struct NotSendData {
    rc: Rc<i32>,  // Rc is not Send (not thread-safe reference counting)
}

// ✅ MAKING IT SEND: Use Arc instead of Rc
struct ThreadSafeData {
    arc: Arc<i32>,  // Arc IS Send + Sync
}

// Common types:
// Send + Sync: i32, String, Vec<T>, Arc<T>, Mutex<T>
// Send only: Cell<T>, RefCell<T>, mpsc::Receiver
// Neither: Rc<T>, *const T, *mut T
```

```rust
// This won't compile if Data isn't Send:
fn spawn_worker(data: Data) {
    std::thread::spawn(move || {
        process(data);  // Error if Data: !Send
    });
}

// This won't compile if Data isn't Sync:
fn share_data(data: &Data) {
    std::thread::scope(|s| {
        s.spawn(|| read(data));   // Error if Data: !Sync
        s.spawn(|| read(data));
    });
}
```

---

## CA-20: Understand Future Pinning

**Strength**: SHOULD

**Summary**: Futures that borrow across `.await` must be pinned.

```rust
use std::pin::Pin;
use std::future::Future;

// Most of the time, you don't need to think about pinning:
async fn simple() {
    let data = fetch_data().await;
    process(data).await;
}

// ✅ When storing futures, use Box::pin or pin!
async fn with_timeout<F, T>(future: F, duration: Duration) -> Option<T>
where
    F: Future<Output = T>,
{
    tokio::select! {
        result = future => Some(result),
        _ = tokio::time::sleep(duration) => None,
    }
}

// ✅ When you need Pin explicitly:
fn returns_future() -> Pin<Box<dyn Future<Output = i32> + Send>> {
    Box::pin(async {
        42
    })
}

// ✅ Using pin! macro (nightly or pin-utils crate)
use std::pin::pin;

async fn example() {
    let future = async { 42 };
    let pinned = pin!(future);
    // pinned is Pin<&mut impl Future>
}
```

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