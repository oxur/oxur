# Concurrency and Async

> Patterns for async/await, threading, and concurrent data access.

---

## CA-01: Understand `Send` and `Sync`

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

**Why this matters**:
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

## CA-02: Choose the Right Synchronization Primitive

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

**Selection guide**:

| Need | Use |
|------|-----|
| Simple counter/flag | `Atomic*` |
| Exclusive mutable access | `Mutex<T>` |
| Many readers, rare writers | `RwLock<T>` |
| Shared ownership | `Arc<T>` |
| Shared + mutable | `Arc<Mutex<T>>` or `Arc<RwLock<T>>` |
| One-time initialization | `OnceLock` (std) or `once_cell` |

---

## CA-03: Avoid Blocking in Async Code

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

## CA-04: Understand Future Pinning

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

**When you need pinning**:
- Storing futures in collections
- Implementing `Future` manually with self-references
- Using `select!` or similar combinators
- Trait objects: `Pin<Box<dyn Future>>`

---

## CA-05: Structured Concurrency with Scoped Threads

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

## CA-06: Async Task Spawning Patterns

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

## CA-07: Cancellation Safety

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

**Cancellation-safe patterns**:
- Avoid holding critical resources across `.await`
- Use transactions for multi-step operations
- Document cancellation behavior for public async functions

---

## CA-08: Channels for Task Communication

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

## CA-10: Async Trait Methods

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

## CA-11: Executor-Agnostic Code

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

**Reality check**: Full executor agnosticism is often impractical. Pick a runtime (usually Tokio) and stick with it. Abstract at higher levels if needed.

---

## Summary: Concurrency Decision Tree

```
Need concurrent execution?
├─ CPU-bound work
│   ├─ In async context → spawn_blocking
│   └─ Pure sync → rayon / std::thread::scope
│
├─ I/O-bound work
│   ├─ Need to share data
│   │   ├─ Read-heavy → Arc<RwLock<T>>
│   │   └─ Write-heavy → Arc<Mutex<T>>
│   └─ Need to communicate
│       ├─ Request/response → oneshot
│       ├─ Stream of work → mpsc
│       ├─ Broadcast updates → broadcast / watch
│       └─ Between sync/async → mpsc with blocking_recv
│
└─ Simple counter/flag → Atomic*
```

---

*See also: [11-anti-patterns.md](11-anti-patterns.md#ap-18) for async anti-patterns.*
