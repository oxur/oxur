# Project Structure

Guidelines for organizing Rust projects, modules, visibility, and code layout.

## Module Organization

### Use Hierarchical Module Structure

**Strength**: SHOULD

**Summary**: Organize code into logical module hierarchies that reflect the domain structure.

**Examples**:

```
my-crate/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Crate root and public API
│   ├── client/         # Client module
│   │   ├── mod.rs      # Module root
│   │   ├── builder.rs  # ClientBuilder
│   │   └── pool.rs     # Connection pool
│   ├── server/         # Server module
│   │   ├── mod.rs
│   │   ├── handler.rs
│   │   └── router.rs
│   ├── protocol/       # Protocol implementation
│   │   ├── mod.rs
│   │   ├── request.rs
│   │   └── response.rs
│   └── error.rs        # Error types
└── tests/
    └── integration_tests.rs
```

**In lib.rs**:

```rust
//! # My Crate
//!
//! Comprehensive crate documentation here...

// Re-export main types at crate root
pub use client::Client;
pub use error::Error;
pub use server::Server;

// Modules - private by default
mod client;
mod server;
mod protocol;
pub mod error;  // Public module

// Private utilities
mod utils;

// Feature-gated modules
#[cfg(feature = "async")]
pub mod async_client;
```

**In client/mod.rs**:

```rust
//! Client implementation

// Private submodules
mod builder;
mod pool;

// Re-export public types
pub use builder::ClientBuilder;
pub use pool::ConnectionPool;

// Public type
pub struct Client {
    pool: ConnectionPool,  // Private field
}

impl Client {
    pub fn new() -> Self {
        Self {
            pool: ConnectionPool::new(),
        }
    }
    
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
}
```

**Benefits of this structure**:
1. Clear separation of concerns
2. Easy to navigate
3. Supports gradual exposure of internals
4. Enables feature flags per module

**Rationale**: Hierarchical modules make large codebases manageable and maintainable.

---

### Module Files vs Module Directories

**Strength**: SHOULD

**Summary**: Use `mod.rs` for modules with submodules, single files for simple modules.

**Examples**:

```
// Small module - single file
src/
├── lib.rs
└── utils.rs  # Contains mod utils { ... }

// In lib.rs:
mod utils;
pub use utils::format_duration;

// In utils.rs:
pub fn format_duration(secs: u64) -> String {
    // ...
}
```

```
// Large module - directory with mod.rs
src/
├── lib.rs
└── client/
    ├── mod.rs      # Module root
    ├── builder.rs  # Submodule
    └── config.rs   # Submodule

// In lib.rs:
mod client;
pub use client::Client;

// In client/mod.rs:
mod builder;
mod config;

pub use builder::ClientBuilder;
pub use config::ClientConfig;

pub struct Client {
    // ...
}
```

**Modern alternative** (Rust 2018+):

```
// Instead of client/mod.rs, use client.rs alongside client/ directory
src/
├── lib.rs
├── client.rs       # Instead of client/mod.rs
└── client/
    ├── builder.rs
    └── config.rs

// In lib.rs:
mod client;

// In client.rs:
mod builder;
mod config;
// ... rest of module
```

**When to use each**:

**Single file** (`utils.rs`):
- Small, focused module
- Few related functions
- No submodules needed

**Directory with mod.rs**:
- Complex module
- Multiple submodules
- Needs organization

**Rationale**: File structure should reflect logical module organization.

---

## Visibility and Encapsulation

### Use pub(crate) for Internal APIs

**Strength**: SHOULD

**Summary**: Use `pub(crate)` for items that need to be used across modules but shouldn't be public API.

**Examples**:

```rust
// In src/protocol/mod.rs
pub(crate) struct InternalProtocol {
    // Visible within crate, but not to external users
}

impl InternalProtocol {
    pub(crate) fn parse(data: &[u8]) -> Result<Self, Error> {
        // ...
    }
}

// In src/client/mod.rs
use crate::protocol::InternalProtocol;  // Can use it

pub struct Client {
    protocol: InternalProtocol,  // Private field using pub(crate) type
}

// External users cannot access InternalProtocol
// use my_crate::protocol::InternalProtocol;  // Error!
```

**Visibility modifiers**:

```rust
// Private - only in this module
struct Private;

// Visible within crate
pub(crate) struct CrateLevel;

// Visible to parent module
pub(super) struct ParentLevel;

// Visible to specific path
pub(in crate::client) struct PathLevel;

// Fully public
pub struct Public;
```

**Common pattern** - internal utilities:

```rust
// src/utils.rs
pub(crate) fn validate_email(email: &str) -> bool {
    email.contains('@')
}

pub(crate) fn sanitize_input(input: &str) -> String {
    input.trim().to_lowercase()
}

// Used throughout crate but not exposed
```

**Rationale**: `pub(crate)` allows internal organization without exposing implementation details.

---

### Re-export at Crate Root

**Strength**: SHOULD

**Summary**: Re-export main types at the crate root for ergonomic imports.

**Examples**:

```rust
// In src/lib.rs
pub use client::Client;
pub use error::{Error, Result};
pub use server::Server;
pub use config::Config;

// Now users can import easily:
// use mycrate::{Client, Server, Config};

// Instead of:
// use mycrate::client::Client;
// use mycrate::server::Server;
// use mycrate::config::Config;
```

**Conditional re-exports**:

```rust
// In src/lib.rs

// Always available
pub use error::Error;

// Feature-gated
#[cfg(feature = "async")]
pub use async_client::AsyncClient;

#[cfg(feature = "blocking")]
pub use blocking_client::BlockingClient;
```

**Module organization with re-exports**:

```rust
// Complex internal structure:
// src/client/async/builder.rs
// src/client/async/pool.rs
// src/client/blocking/simple.rs

// But simple public API:
pub use client::async_client::{AsyncClient, AsyncClientBuilder};
pub use client::blocking_client::BlockingClient;

// Users see:
// use mycrate::{AsyncClient, BlockingClient};
```

**Rationale**: Re-exports provide a clean, simple API while maintaining internal organization.

---

## Crate Organization

### Separate Binary and Library Crates

**Strength**: SHOULD

**Summary**: For projects with both library and binary, put logic in lib, binary is thin wrapper.

**Examples**:

```
my-tool/
├── Cargo.toml
├── src/
│   ├── lib.rs      # Library code
│   ├── main.rs     # Binary - thin wrapper
│   ├── cli.rs      # CLI argument parsing
│   └── commands/   # Command implementations
│       ├── mod.rs
│       ├── build.rs
│       └── run.rs
└── tests/
    └── integration.rs
```

**In lib.rs**:

```rust
//! Core library functionality

pub mod commands;
pub mod config;
pub mod error;

pub use error::{Error, Result};
pub use config::Config;

// Main library logic
pub fn execute_build(config: &Config) -> Result<()> {
    // Core logic here - testable, reusable
    Ok(())
}
```

**In main.rs**:

```rust
//! Binary entry point - thin wrapper around library

use my_tool::{Config, execute_build};

fn main() {
    let config = Config::from_args();
    
    if let Err(e) = execute_build(&config) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

**Benefits**:
1. Library logic is testable
2. Can be used by other projects
3. Binary is simple, focused on CLI concerns
4. Easy to add multiple binaries

**Multiple binaries**:

```
my-tool/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── bin/
│   │   ├── my-tool.rs       # Main binary
│   │   ├── my-tool-admin.rs # Admin binary
│   │   └── my-tool-server.rs # Server binary
│   └── ...
```

**In Cargo.toml**:

```toml
[package]
name = "my-tool"
version = "1.0.0"

# Library
[lib]
name = "my_tool"
path = "src/lib.rs"

# Multiple binaries
[[bin]]
name = "my-tool"
path = "src/bin/my-tool.rs"

[[bin]]
name = "my-tool-admin"
path = "src/bin/my-tool-admin.rs"
```

**Rationale**: Separation enables testing, reuse, and maintains clean architecture.

---

### Organize Tests Appropriately

**Strength**: MUST

**Summary**: Use unit tests in modules, integration tests in tests/ directory.

**Examples**:

**Unit tests** - in same file as code:

```rust
// src/parser.rs
pub fn parse_number(s: &str) -> Result<i32, ParseError> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_number() {
        assert_eq!(parse_number("42").unwrap(), 42);
    }

    #[test]
    fn test_parse_invalid_number() {
        assert!(parse_number("not a number").is_err());
    }

    #[test]
    fn test_parse_negative_number() {
        assert_eq!(parse_number("-42").unwrap(), -42);
    }
}
```

**Integration tests** - in tests/ directory:

```
my-crate/
├── src/
│   └── lib.rs
└── tests/
    ├── integration_test.rs
    ├── end_to_end.rs
    └── common/           # Shared test utilities
        └── mod.rs
```

```rust
// tests/integration_test.rs
use my_crate::{Client, Config};

#[test]
fn test_client_connection() {
    let config = Config::default();
    let client = Client::new(config);
    
    // Test public API as external user would
    assert!(client.connect().is_ok());
}
```

**Shared test utilities**:

```rust
// tests/common/mod.rs
pub fn setup_test_database() -> TestDb {
    // Shared setup code
}

pub fn cleanup_test_files(dir: &Path) {
    // Shared cleanup
}

// tests/integration_test.rs
mod common;  // Import shared utilities

#[test]
fn test_with_database() {
    let db = common::setup_test_database();
    // Test using db
}
```

**Benchmark tests**:

```
my-crate/
├── benches/
│   └── benchmarks.rs
└── Cargo.toml
```

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmarks"
harness = false
```

```rust
// benches/benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use my_crate::expensive_function;

fn benchmark_expensive_function(c: &mut Criterion) {
    c.bench_function("expensive_function", |b| {
        b.iter(|| expensive_function(black_box(100)))
    });
}

criterion_group!(benches, benchmark_expensive_function);
criterion_main!(benches);
```

**Rationale**: Proper test organization improves maintainability and separates concerns.

---

## Feature Flags

### Organize Code by Features

**Strength**: SHOULD

**Summary**: Use Cargo features to enable optional functionality without bloating default builds.

**Examples**:

```toml
# Cargo.toml
[features]
default = ["std"]

# Feature flags
std = []
async = ["tokio", "futures"]
serde = ["dep:serde", "dep:serde_json"]
full = ["async", "serde"]  # Meta-feature

[dependencies]
# Always included
bytes = "1.0"

# Optional dependencies
tokio = { version = "1.0", features = ["full"], optional = true }
futures = { version = "0.3", optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
```

**In code**:

```rust
// src/lib.rs

// Always available
pub mod core;
pub mod error;

// Feature-gated modules
#[cfg(feature = "async")]
pub mod async_client;

#[cfg(feature = "serde")]
pub mod serialization;

// Feature-gated implementations
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

pub struct Data {
    pub value: String,
}

#[cfg(feature = "serde")]
impl Serialize for Data {
    // Implementation
}

// Conditional compilation based on features
#[cfg(feature = "async")]
pub async fn async_operation() -> Result<(), Error> {
    // Async implementation
}

#[cfg(not(feature = "async"))]
pub fn blocking_operation() -> Result<(), Error> {
    // Blocking implementation
}

// Multiple feature requirements
#[cfg(all(feature = "async", feature = "serde"))]
pub async fn async_serialize() -> Result<String, Error> {
    // Requires both features
}
```

**Feature organization patterns**:

```rust
// Pattern 1: Separate module files
// src/async_client.rs - only compiled with "async" feature
#![cfg(feature = "async")]

pub struct AsyncClient {
    // Implementation
}

// Pattern 2: Conditional sections in same file
pub struct Client {
    #[cfg(feature = "cache")]
    cache: Cache,
}

impl Client {
    #[cfg(feature = "cache")]
    pub fn with_cache(cache: Cache) -> Self {
        // Implementation
    }
}
```

**Testing with features**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_functionality() {
        // Always runs
    }

    #[test]
    #[cfg(feature = "async")]
    fn test_async_functionality() {
        // Only runs with async feature
    }
}
```

**Rationale**: Feature flags allow users to pay only for what they use while maintaining a rich ecosystem.

---

## Code Layout

### Consistent Formatting

**Strength**: MUST

**Summary**: Use `rustfmt` for consistent formatting across the project.

**Examples**:

```toml
# rustfmt.toml
max_width = 100
tab_spaces = 4
edition = "2021"
```

**Run formatting**:

```bash
# Format entire project
cargo fmt

# Check formatting (CI)
cargo fmt -- --check
```

**In code**:

```rust
// Good - formatted with rustfmt
pub struct Config {
    pub timeout: Duration,
    pub max_retries: u32,
    pub user_agent: String,
}

impl Config {
    pub fn new(timeout: Duration, max_retries: u32, user_agent: String) -> Self {
        Config {
            timeout,
            max_retries,
            user_agent,
        }
    }
}

// rustfmt handles long lines
pub fn process_data(
    input: &[u8],
    output: &mut Vec<u8>,
    options: ProcessOptions,
) -> Result<usize, Error> {
    // Implementation
}
```

**Rationale**: Consistent formatting reduces bikeshedding and improves readability.

---

### Use Clippy for Linting

**Strength**: MUST

**Summary**: Run Clippy to catch common mistakes and enforce Rust idioms.

**Examples**:

```bash
# Run clippy
cargo clippy

# Run clippy with all features
cargo clippy --all-features

# Fail on warnings (CI)
cargo clippy -- -D warnings
```

**In CI**:

```yaml
# .github/workflows/ci.yml
- name: Run Clippy
  run: cargo clippy --all-targets --all-features -- -D warnings
```

**Configure clippy**:

```toml
# Cargo.toml
[lints.clippy]
# Deny by default
all = "deny"

# But allow some pedantic lints
pedantic = "warn"

# Specific overrides
too_many_arguments = "allow"
module_name_repetitions = "allow"
```

**Or in code**:

```rust
// File-level
#![warn(clippy::all)]
#![deny(clippy::correctness)]

// Function-level
#[allow(clippy::too_many_arguments)]
fn complex_function(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) {
    // ...
}
```

**Rationale**: Clippy catches bugs and suggests idiomatic patterns automatically.
