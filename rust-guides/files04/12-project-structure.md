# Project Structure Guidelines

Guidelines for organizing crates, modules, features, and building Rust projects.

## Table of Contents

- [Crate Organization](#crate-organization)
- [Feature Flags](#feature-flags)
- [Building](#building)
- [Static Verification](#static-verification)

---

## Crate Organization

### If in Doubt, Split the Crate

**Strength**: SHOULD

**Summary**: Prefer multiple small crates over monolithic crates for compile time and modularity.

**Example**:
```
# WRONG - everything in one crate
my_project/
└── src/
    ├── lib.rs
    ├── client/
    │   └── mod.rs  (10k lines)
    ├── server/
    │   └── mod.rs  (15k lines)
    └── protocols/
        └── mod.rs  (5k lines)

# CORRECT - split into crates
my_project/
├── my_project_client/
│   └── src/lib.rs
├── my_project_server/
│   └── src/lib.rs  
├── my_project_protocols/
│   └── src/lib.rs
└── my_project/  (umbrella crate)
    └── src/lib.rs  (re-exports)
```

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

**When to split**:
- Module can be used independently
- Would reduce compile times
- Breaks cyclic dependencies
- Different stability or versioning needs

**Re-joining with umbrella crates**:
- Use for user convenience
- Always re-export proc macro crates
- Consider feature flags to enable/disable components

**See also**: M-SMALLER-CRATES

---

### Features vs. Crates

**Strength**: SHOULD

**Summary**: Use crates for independent functionality; use features to unlock extra capabilities.

**Example**:
```toml
# WRONG - using features for what should be crates
[dependencies]
my_lib = { version = "1.0", features = ["client", "server"] }

# Components are independent - should be separate crates!

# CORRECT - separate crates
[dependencies]
my_lib_client = "1.0"
my_lib_server = "1.0"

# Features for optional capabilities
[dependencies]
my_lib_client = { version = "1.0", features = ["tls", "compression"] }
```

**Rule of thumb**:
- **Crates**: For items that can reasonably be used on their own
- **Features**: To unlock extra functionality that can't live alone

**Example feature usage**:
```toml
[features]
default = ["std"]
std = []
tls = ["dep:rustls"]
serde = ["dep:serde"]
```

**See also**: M-SMALLER-CRATES

---

## Feature Flags

### Features Are Additive

**Strength**: MUST

**Summary**: All feature combinations must work; features must only add functionality, never remove it.

**Example**:
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

**Rules**:
- ❌ No `no-std` feature → Use `std` feature instead
- ❌ Features can't disable public items
- ❌ Features can't be mutually exclusive
- ✅ Features only add functionality
- ✅ Any feature combination must compile

**See also**: M-FEATURES-ADDITIVE

---

### Test Utilities Behind Feature Gate

**Strength**: MUST

**Summary**: Mocking, testing utilities, and safety bypasses must be behind `test-util` feature.

**Example**:
```toml
[features]
test-util = []
```

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

**What to gate**:
- Mock constructors
- Ability to inspect sensitive data
- Safety check overrides
- Fake data generation

**See also**: M-TEST-UTIL

---

## Building

### Libraries Work Out of the Box

**Strength**: MUST

**Summary**: Libraries must compile on all supported platforms without additional dependencies beyond cargo and rustc.

**Example**:
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

**Platform support**:
- Must build on all Tier 1 platforms
- Can use conditional compilation for platform-specific code
- External dependencies must be behind feature flags

**Example conditional compilation**:
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

### Native -sys Crates Compile Without Dependencies

**Strength**: MUST

**Summary**: FFI bindings crates must compile without requiring external tools or libraries.

**Example**:
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

**Requirements**:
- Embed upstream source code
- Build from `build.rs` using `cc` crate
- Make external tools optional
- Pre-generate `bindgen` output if possible
- Support both static and dynamic linking

**See also**: M-SYS-CRATES

---

## Static Verification

### Use Static Verification Tools

**Strength**: MUST

**Summary**: Projects must use compiler lints, clippy, rustfmt, and related tools.

**Example**:
```toml
# In Cargo.toml
[lints.rust]
unsafe_code = "deny"
missing_debug_implementations = "warn"
missing_docs = "warn"
trivial_casts = "warn"
trivial_numeric_casts = "warn"
unused_import_braces = "warn"
unused_lifetimes = "warn"
unused_qualifications = "warn"

[lints.clippy]
cargo = { level = "warn", priority = -1 }
complexity = { level = "warn", priority = -1 }
correctness = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
perf = { level = "warn", priority = -1 }
style = { level = "warn", priority = -1 }
suspicious = { level = "warn", priority = -1 }

# Clippy restriction lints
allow_attributes_without_reason = "warn"
as_pointer_underscore = "warn"
assertions_on_result_states = "warn"
clone_on_ref_ptr = "warn"
undocumented_unsafe_blocks = "warn"

# Opt-outs
literal_string_with_formatting_args = "allow"
```

**Recommended tools**:
- `rustfmt` - Code formatting
- `clippy` - Lint collection
- `cargo-audit` - Security vulnerabilities
- `cargo-hack` - Feature combination testing
- `cargo-udeps` - Unused dependencies
- `miri` - Unsafe code validation

**See also**: M-STATIC-VERIFICATION

---

### Use #[expect] for Lint Overrides

**Strength**: MUST

**Summary**: Lint overrides should use `#[expect]` not `#[allow]` to detect stale suppressions.

**Example**:
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

**Exception**: `#[allow]` is acceptable in generated code and macros.

**See also**: M-LINT-OVERRIDE-EXPECT

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
