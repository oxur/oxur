# Project Structure Guidelines

Guidelines for organizing crates, modules, features, and building Rust projects.


## PS-01: Conditional Compilation Patterns

**Strength**: SHOULD

**Summary**: Use cfg attributes for platform and feature-specific code.

```rust
// Platform-specific code
#[cfg(target_os = "linux")]
fn get_home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap())
}

#[cfg(target_os = "windows")]
fn get_home_dir() -> PathBuf {
    PathBuf::from(std::env::var("USERPROFILE").unwrap())
}

// Feature-gated implementations
#[cfg(feature = "async")]
impl MyType {
    pub async fn fetch(&self) -> Result<Data> {
        // async implementation
    }
}

#[cfg(not(feature = "async"))]
impl MyType {
    pub fn fetch(&self) -> Result<Data> {
        // sync implementation
    }
}

// Test-only code
#[cfg(test)]
fn test_helper() -> TestData {
    // Only compiled in test builds
}

// Debug-only code
#[cfg(debug_assertions)]
fn expensive_validation(&self) {
    // Only in debug builds
}

// Combine conditions
#[cfg(all(target_os = "linux", feature = "async"))]
fn linux_async_specific() { }

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_like() { }
```

---

## PS-02: Consistent Formatting

**Strength**: MUST

**Summary**: Use `rustfmt` for consistent formatting across the project.

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

---

## PS-03: Crate Organization

**Strength**: SHOULD

**Summary**: Structure your project for clarity and compilation speed.

---

## PS-04: Error Module Pattern

**Strength**: SHOULD

**Summary**: Centralize error types in a dedicated module.

```rust
// src/error.rs
use thiserror::Error;

/// All errors that can occur in this crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Configuration-specific errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },
}

/// A Result type alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;

// src/lib.rs
mod error;
pub use error::{Error, ConfigError, Result};
```

---

## PS-05: Feature Flags

**Strength**: SHOULD

**Summary**: Use features for optional functionality.

```rust
// src/lib.rs - Conditional compilation
pub mod core;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "xml")]
pub mod xml;

#[cfg(feature = "async")]
pub mod async_support;

// Conditional trait implementation
#[cfg(feature = "json")]
impl JsonSerializable for MyType {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// Feature-gated re-exports
#[cfg(feature = "json")]
pub use json::JsonParser;
```

---

## PS-06: Features Are Additive

**Strength**: MUST

**Summary**: All feature combinations must work; features must only add functionality, never remove it.

```rust
// WRONG - negative feature (subtractive)
#[cfg(not(feature = "no-std"))]
use std::collections::HashMap;

// CORRECT - positive feature (additive)
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

// WRONG - feature disables public items
#[cfg(not(feature = "minimal"))]
pub struct AdvancedConfig { }

// CORRECT - feature adds items
#[cfg(feature = "advanced")]
pub struct AdvancedConfig { }

// WRONG - features are mutually exclusive
#[cfg(all(feature = "tokio", not(feature = "async-std")))]
use tokio::task::spawn;

// CORRECT - features can coexist
#[cfg(feature = "tokio")]
pub mod tokio_compat { }

#[cfg(feature = "async-std")]
pub mod async_std_compat { }
```

**Rationale**: Cargo unifies features across the dependency graph. If crate A enables `feature-x` and crate B enables `feature-y`, both must work together. Subtractive or exclusive features cause build failures.

**See also**: M-FEATURES-ADDITIVE

---

## PS-07: Features vs. Crates

**Strength**: SHOULD

**Summary**: Use crates for independent functionality; use features to unlock extra capabilities.

**See also**: M-SMALLER-CRATES

---

## PS-08: If in Doubt, Split the Crate

**Strength**: SHOULD

**Summary**: Prefer multiple small crates over monolithic crates for compile time and modularity.

```rust
// In umbrella crate my_project/src/lib.rs
pub use my_project_client as client;
pub use my_project_server as server;
pub use my_project_protocols as protocols;

// Users can import from umbrella
use my_project::client::Client;
use my_project::server::Server;

// Or depend on specific crates
// [dependencies]
// my_project_client = "1.0"
```

**Rationale**: Small crates compile faster, especially during development. They prevent cyclic dependencies and enable users to depend only on what they need.

**See also**: M-SMALLER-CRATES

---

## PS-09: Internal vs External Crate Split

**Strength**: CONSIDER

**Summary**: Split large crates into public API and internal implementation.

```rust
// my_project/src/lib.rs
//! Public API - stable interface

// Re-export public items from core
pub use my_project_core::{Config, Parser, Result};

// Re-export macros
pub use my_project_macros::derive_parser;

// Core is not directly exposed to users
// Internal changes don't affect public API
```

---

## PS-10: Libraries Work Out of the Box

**Strength**: MUST

**Summary**: Libraries must compile on all supported platforms without additional dependencies beyond cargo and rustc.

```rust
// WRONG - requires external tool
// build.rs
fn main() {
    // Requires user to install protoc
    prost_build::compile_protos(&["proto/api.proto"], &["proto/"]).unwrap();
}

// CORRECT - generate code before publishing
// build.rs (used during development only)
fn main() {
    #[cfg(feature = "codegen")]
    {
        prost_build::compile_protos(&["proto/api.proto"], &["proto/"]).unwrap();
    }
}

// Include generated code in the package
// src/generated.rs (checked into git)
include!(concat!(env!("OUT_DIR"), "/api.rs"));
```

```rust
#[cfg(target_os = "windows")]
mod windows_impl;

#[cfg(target_os = "linux")]
mod linux_impl;

#[cfg(target_os = "windows")]
pub use windows_impl::*;

#[cfg(target_os = "linux")]
pub use linux_impl::*;
```

**See also**: M-OOBE

---

## PS-11: Module Files vs Module Directories

**Strength**: SHOULD

**Summary**: Use `mod.rs` for modules with submodules, single files for simple modules.

**Rationale**: File structure should reflect logical module organization.

---

---

## PS-12: Module Hierarchy

**Strength**: SHOULD

**Summary**: Use modules to organize related functionality.

```rust
// src/lib.rs - Declare module structure
pub mod config;
pub mod error;
pub mod utils;

mod internal;  // Private module

// Re-export important items at crate root
pub use config::Config;
pub use error::Error;

// src/config.rs - Simple module
pub struct Config {
    pub name: String,
    pub value: i32,
}

// src/utils/mod.rs - Module with submodules
pub mod parsing;
pub mod formatting;

// Re-export commonly used items
pub use parsing::Parser;
pub use formatting::Formatter;

// src/utils/parsing.rs
pub struct Parser { /* ... */ }

impl Parser {
    pub fn parse(&self, input: &str) -> Result<Ast, ParseError> {
        todo!()
    }
}
```

---

## PS-13: Multi-crate workspace:

**Strength**: SHOULD

**Summary**: 

---

## PS-14: Native -sys Crates Compile Without Dependencies

**Strength**: MUST

**Summary**: FFI bindings crates must compile without requiring external tools or libraries.

```rust
// In foo-sys/build.rs

fn main() {
    // CORRECT - fully govern build from Rust
    cc::Build::new()
        .file("vendor/foo/foo.c")
        .file("vendor/foo/bar.c")
        .compile("foo");
    
    // Don't run external build scripts:
    // - ❌ No Makefiles
    // - ❌ No CMake (unless cmake crate available)
    // - ❌ No Python/Perl scripts
}
```

**See also**: M-SYS-CRATES

---

## PS-15: Organize Code by Features

**Strength**: SHOULD

**Summary**: Use Cargo features to enable optional functionality without bloating default builds.

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

---

## PS-16: Organize Tests Appropriately

**Strength**: MUST

**Summary**: Use unit tests in modules, integration tests in tests/ directory.

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

---

## PS-17: Prelude Pattern

**Strength**: CONSIDER

**Summary**: Provide a prelude module for common imports.

```rust
// src/prelude.rs
//! Commonly used items for glob import.
//! 
//! ```
//! use my_crate::prelude::*;
//! ```

pub use crate::Config;
pub use crate::Error;
pub use crate::Result;
pub use crate::traits::{Parse, Format};

// In user code:
use my_crate::prelude::*;

// ⚠️ CAUTION: Preludes can cause name collisions
// Only include items that:
// 1. Are used very frequently
// 2. Have distinctive names
// 3. Are unlikely to conflict with user code

// Common prelude contents:
// - Main error type and Result alias
// - Core traits users need to implement or use
// - Essential type aliases
```

---

## PS-18: Project Checklist

**Strength**: SHOULD

**Summary**: 

---

## PS-19: Re-export at Crate Root

**Strength**: SHOULD

**Summary**: Re-export main types at the crate root for ergonomic imports.

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

---

## PS-20: Re-exports for Cleaner API

**Strength**: SHOULD

**Summary**: Re-export items to create a flat, user-friendly API.

```rust
// Internal organization (deep nesting)
// src/parsers/json/mod.rs
pub mod parser;
pub mod error;

// src/parsers/json/parser.rs
pub struct JsonParser { /* ... */ }

// Without re-exports, users need:
use my_crate::parsers::json::parser::JsonParser;
use my_crate::parsers::json::error::JsonError;

// ✅ BETTER: Re-export at crate root
// src/lib.rs
mod parsers;

// Flat public API
pub use parsers::json::parser::JsonParser;
pub use parsers::json::error::JsonError;

// Or group under a public module
pub mod json {
    pub use crate::parsers::json::parser::JsonParser;
    pub use crate::parsers::json::error::JsonError;
}

// Users can now:
use my_crate::JsonParser;
// or
use my_crate::json::{JsonParser, JsonError};
```

---

## PS-21: Separate Binary and Library Crates

**Strength**: SHOULD

**Summary**: For projects with both library and binary, put logic in lib, binary is thin wrapper.

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

**Rationale**: Separation enables testing, reuse, and maintains clean architecture.

---

---

## PS-22: Simple library:

**Strength**: SHOULD

**Summary**: 

---

## PS-23: Test Organization

**Strength**: SHOULD

**Summary**: Organize tests appropriately by type.

```rust
// src/lib.rs

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Unit tests: In the same file, test private functions
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
    
    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, 1), 0);
    }
}

// For larger test suites, use a submodule:
// src/parser.rs
pub fn parse(input: &str) -> Result<Ast> { /* ... */ }

#[cfg(test)]
mod tests;  // src/parser/tests.rs or src/parser/tests/mod.rs
```

```rust
// tests/integration_test.rs - Integration tests (tests/ directory)
// These test the public API as external users would

use my_crate::{Config, Parser};

#[test]
fn test_full_workflow() {
    let config = Config::default();
    let parser = Parser::new(&config);
    let result = parser.parse("input");
    assert!(result.is_ok());
}

// Test helper module
mod common;

#[test]
fn test_with_fixtures() {
    let fixture = common::load_fixture("test_case_1");
    // ...
}
```

```rust
// tests/common/mod.rs - Shared test utilities
use std::path::PathBuf;

pub fn load_fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

pub fn setup() -> TestContext {
    TestContext::new()
}
```

---

## PS-24: Test Utilities Behind Feature Gate

**Strength**: MUST

**Summary**: Mocking, testing utilities, and safety bypasses must be behind `test-util` feature.

```rust
// Gate mock constructors
#[cfg(feature = "test-util")]
impl Database {
    pub fn new_mocked() -> (Self, MockCtrl) {
        // ...
    }
}

// Gate test-only methods
impl HttpClient {
    #[cfg(feature = "test-util")]
    pub fn bypass_certificate_checks(&mut self) {
        self.verify_certs = false;
    }
}

// Gate mock modules
#[cfg(feature = "test-util")]
pub mod mock {
    pub struct MockCtrl { /* ... */ }
}
```

**Rationale**: Test utilities can bypass safety checks and shouldn't be available in production builds. Feature gates ensure they're only compiled when explicitly requested.

**See also**: M-TEST-UTIL

---

## PS-25: Use #[expect] for Lint Overrides

**Strength**: MUST

**Summary**: Lint overrides should use `#[expect]` not `#[allow]` to detect stale suppressions.

```rust
// WRONG - can become stale
#[allow(clippy::too_many_arguments)]
fn process(a: A, b: B, c: C, d: D) {
    // Later someone refactors to use a config struct
    // but #[allow] remains forever
}

// CORRECT - warns if not needed
#[expect(clippy::too_many_arguments, reason = "API compatibility")]
fn process(a: A, b: B, c: C, d: D) {
    // If refactored, compiler warns about unused #[expect]
}

// Always provide reason
#[expect(
    clippy::cast_possible_truncation,
    reason = "Value is guaranteed to be in u32 range"
)]
let value = large_value as u32;
```

**Rationale**: `#[expect]` warns if the lint is no longer triggered, preventing accumulation of outdated suppressions.

**See also**: M-LINT-OVERRIDE-EXPECT

---

## PS-26: Use Clippy for Linting

**Strength**: MUST

**Summary**: Run Clippy to catch common mistakes and enforce Rust idioms.

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

---

## PS-27: Use Hierarchical Module Structure

**Strength**: SHOULD

**Summary**: Organize code into logical module hierarchies that reflect the domain structure.

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

**Rationale**: Hierarchical modules make large codebases manageable and maintainable.

---

---

## PS-28: Use pub(crate) for Internal APIs

**Strength**: SHOULD

**Summary**: Use `pub(crate)` for items that need to be used across modules but shouldn't be public API.

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

---

## PS-29: Use Static Verification Tools

**Strength**: MUST

**Summary**: Projects must use compiler lints, clippy, rustfmt, and related tools.

**See also**: M-STATIC-VERIFICATION

---

## PS-30: Visibility Levels

**Strength**: MUST

**Summary**: Use the most restrictive visibility that works.

```rust
// Private (default): Only accessible in current module
struct Private;
fn private_fn() {}

// pub(crate): Visible within the crate only
pub(crate) struct CrateVisible;
pub(crate) fn crate_visible_fn() {}

// pub(super): Visible in parent module
pub(super) struct ParentVisible;

// pub(in path): Visible in specific module
pub(in crate::utils) struct UtilsVisible;

// pub: Visible everywhere (if crate is public)
pub struct Public;
pub fn public_fn() {}

// Struct with mixed field visibility
pub struct Config {
    pub name: String,           // Public field
    pub(crate) internal: i32,   // Crate-only field
    secret: String,             // Private field
}

impl Config {
    // Private method - internal logic
    fn validate(&self) -> bool {
        !self.secret.is_empty()
    }
    
    // Public method - part of API
    pub fn new(name: String) -> Self {
        Self {
            name,
            internal: 0,
            secret: String::new(),
        }
    }
}
```

---

## PS-31: Workspace Organization

**Strength**: CONSIDER

**Summary**: Use workspaces for multi-crate projects.

---

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| Split into small crates | SHOULD | Faster compile times |
| Features are additive | MUST | All combinations must work |
| Test utils feature-gated | MUST | Use `test-util` feature |
| Libraries work OOBE | MUST | No external dependencies |
| Sys crates self-contained | MUST | Build from `build.rs` |
| Use static verification | MUST | Clippy, rustfmt, audit |
| Use #[expect] not #[allow] | MUST | Detect stale suppressions |

## Project Checklist

```toml
# Cargo.toml template

[package]
name = "my-crate"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[features]
default = ["std"]
std = []
test-util = []

[lints.rust]
unsafe_code = "deny"
missing_debug_implementations = "warn"
unused_lifetimes = "warn"

[lints.clippy]
cargo = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
# ... other lints

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmarks"
harness = false
```

## Common Project Structures

### Simple library:
```
my-lib/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── module.rs
├── tests/
│   └── integration_test.rs
└── benches/
    └── bench.rs
```

### Multi-crate workspace:
```
my-project/
├── Cargo.toml  (workspace)
├── my-project/  (umbrella)
│   ├── Cargo.toml
│   └── src/lib.rs
├── my-project-core/
│   ├── Cargo.toml
│   └── src/lib.rs
├── my-project-macros/
│   ├── Cargo.toml
│   └── src/lib.rs
└── examples/
    └── example.rs
```

## Related Guidelines

- **API Design**: See `02-api-design.md` for public interfaces
- **Type Design**: See `05-type-design.md` for module organization

## External References

- [Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- Pragmatic Rust: M-SMALLER-CRATES, M-FEATURES-ADDITIVE, M-OOBE