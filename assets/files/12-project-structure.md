# Project Structure

> Patterns for organizing crates, modules, and visibility.

---

## PS-01: Crate Organization

**Strength**: SHOULD

**Summary**: Structure your project for clarity and compilation speed.

```
my_project/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Library root (if library crate)
│   ├── main.rs         # Binary root (if binary crate)
│   ├── bin/            # Additional binaries
│   │   ├── tool1.rs
│   │   └── tool2.rs
│   ├── config.rs       # Module file
│   ├── error.rs        # Module file
│   └── utils/          # Module directory
│       ├── mod.rs      # Module root
│       ├── parsing.rs
│       └── formatting.rs
├── tests/              # Integration tests
│   └── integration_test.rs
├── benches/            # Benchmarks
│   └── benchmark.rs
└── examples/           # Example programs
    └── basic_usage.rs
```

**Cargo.toml for mixed crate**:
```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_project"
path = "src/lib.rs"

[[bin]]
name = "my_project"
path = "src/main.rs"

[[bin]]
name = "my_tool"
path = "src/bin/tool1.rs"
```

---

## PS-02: Module Hierarchy

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

## PS-03: Visibility Levels

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

**Visibility guideline**:
1. Start with private (default)
2. Use `pub(crate)` for shared internal items
3. Use `pub` only for items that are part of the public API

---

## PS-04: Re-exports for Cleaner API

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

## PS-05: Prelude Pattern

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

## PS-06: Feature Flags

**Strength**: SHOULD

**Summary**: Use features for optional functionality.

```toml
# Cargo.toml
[features]
# Default features (enabled unless user opts out)
default = ["json"]

# Optional features
json = ["serde_json"]
xml = ["quick-xml"]
async = ["tokio"]

# Feature that enables other features
full = ["json", "xml", "async"]

[dependencies]
serde = "1.0"
serde_json = { version = "1.0", optional = true }
quick-xml = { version = "0.30", optional = true }
tokio = { version = "1.0", optional = true, features = ["full"] }
```

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

## PS-07: Workspace Organization

**Strength**: CONSIDER

**Summary**: Use workspaces for multi-crate projects.

```
my_workspace/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── my_core/           # Core library
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── my_cli/            # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── my_server/         # Server binary
│   │   ├── Cargo.toml
│   │   └── src/
│   └── my_utils/          # Shared utilities
│       ├── Cargo.toml
│       └── src/
└── tests/                  # Workspace-level integration tests
```

```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/my_core",
    "crates/my_cli",
    "crates/my_server",
    "crates/my_utils",
]

# Shared settings
[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/..."

# Shared dependencies
[workspace.dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
my_core = { path = "crates/my_core" }
my_utils = { path = "crates/my_utils" }
```

```toml
# crates/my_cli/Cargo.toml
[package]
name = "my_cli"
version.workspace = true
edition.workspace = true

[dependencies]
my_core.workspace = true
my_utils.workspace = true
serde.workspace = true
```

---

## PS-08: Error Module Pattern

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

## PS-09: Test Organization

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

## PS-10: Conditional Compilation Patterns

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

## PS-11: Internal vs External Crate Split

**Strength**: CONSIDER

**Summary**: Split large crates into public API and internal implementation.

```
my_project/
├── Cargo.toml           # Workspace
├── my_project/          # Public crate (what users depend on)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs       # Re-exports from internal
├── my_project_core/     # Internal implementation
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── my_project_macros/   # Proc macros (if needed)
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

```toml
# my_project/Cargo.toml
[package]
name = "my_project"

[dependencies]
my_project_core = { path = "../my_project_core" }
my_project_macros = { path = "../my_project_macros" }
```

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

## Summary: Project Structure Checklist

**Crate Organization**:
- [ ] Clear separation of lib.rs and main.rs (if both)
- [ ] Binaries in src/bin/ for additional executables
- [ ] Examples in examples/
- [ ] Integration tests in tests/

**Module Structure**:
- [ ] Logical grouping of related functionality
- [ ] Re-exports for flat public API
- [ ] Private modules for internal code
- [ ] Prelude module for common imports (optional)

**Visibility**:
- [ ] Start private, widen as needed
- [ ] Use pub(crate) for internal sharing
- [ ] Document public items

**Features**:
- [ ] Optional dependencies behind feature flags
- [ ] Sensible default features
- [ ] Feature documentation in Cargo.toml

**Testing**:
- [ ] Unit tests in #[cfg(test)] modules
- [ ] Integration tests in tests/
- [ ] Shared test utilities in tests/common/

---

*See also: [02-api-design.md](02-api-design.md) for public API organization.*
