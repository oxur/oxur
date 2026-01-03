# REPL Phase 1: Foundation Components - Implementation Plan

**Version:** 1.0
**Created:** 2026-01-03
**Status:** Ready to Execute
**Dependencies:** None (can start immediately)
**Estimated Duration:** 1 week (5-6 days)

---

## Table of Contents

1. [Overview](#overview)
2. [File Structure](#file-structure)
3. [Component 1: VariableStore](#component-1-variablestore)
4. [Component 2: SessionDir](#component-2-sessiondir)
5. [Component 3: Cargo Integration](#component-3-cargo-integration)
6. [Component 4: Subprocess Runtime](#component-4-subprocess-runtime)
7. [Integration Testing](#integration-testing)
8. [Success Criteria](#success-criteria)
9. [Timeline & Execution Order](#timeline--execution-order)

---

## Overview

### Goals

Implement the 4 foundational REPL components that have **zero external dependencies**:

1. **VariableStore** - Type-erased variable storage using `Box<dyn Any>`
2. **SessionDir** - Per-session temporary directory and Cargo project management
3. **Cargo Integration** - Wrapper for invoking cargo and parsing output
4. **Subprocess Runtime** - Separate binary for executing compiled code

These components form the foundation for the REPL compilation pipeline and can be implemented while Core Forms and Lowering are being designed by the oxur-lang and oxur-comp teams.

### Design Document References

- **Primary:** ODD-0030 (REPL Implementation Specification)
  - Section 3.1 - CachedCompiler architecture
  - Section 4 - Cargo invocation
  - Section 5 - SessionDir management
  - ADR-001 - Value representation
- **Secondary:** ODD-0031 (REPL Component Proto-Plans)

### Why These Components First?

1. **No blockers** - Can implement without Core Forms or Lowering
2. **Clear specifications** - Well-defined in ODD-0030
3. **Testable in isolation** - Each component has clear contracts
4. **Foundation for later work** - Required by CachedCompiler
5. **Parallel work** - Allows progress while language team designs Core Forms

---

## File Structure

### New Directories and Files

```
crates/oxur-repl/
├── src/
│   ├── lib.rs                          # Update exports
│   ├── runtime/
│   │   ├── mod.rs                      # [NEW] Runtime module exports
│   │   ├── variable_store.rs           # [NEW] Component 1
│   │   └── variable_store_tests.rs     # [NEW] Component 1 tests
│   ├── session/
│   │   ├── mod.rs                      # Update exports
│   │   ├── dir.rs                      # [NEW] Component 2
│   │   └── dir_tests.rs                # [NEW] Component 2 tests
│   ├── compiler/
│   │   ├── mod.rs                      # [NEW] Compiler module exports
│   │   ├── cargo.rs                    # [NEW] Component 3
│   │   └── cargo_tests.rs              # [NEW] Component 3 tests
│   └── bin/
│       └── oxur-subprocess.rs          # [NEW] Component 4 binary
│
├── Cargo.toml                          # Update dependencies
└── test-data/
    └── integration/
        ├── basic-project/              # [NEW] Test fixture
        └── compilation-errors/         # [NEW] Test fixture

crates/oxur-subprocess/                 # [NEW] Separate crate
├── Cargo.toml                          # [NEW]
├── src/
│   ├── main.rs                         # [NEW] Component 4 main
│   ├── loader.rs                       # [NEW] Dynamic library loading
│   ├── protocol.rs                     # [NEW] Communication protocol
│   └── lib.rs                          # [NEW] For testing
└── README.md                           # [NEW]
```

### Module Organization

```rust
// crates/oxur-repl/src/lib.rs additions
pub mod runtime;    // [NEW] Runtime components
pub mod compiler;   // [NEW] Compiler infrastructure

// Existing modules
pub mod eval;
pub mod protocol;
pub mod server;
pub mod session;
```

---

## Component 1: VariableStore

**Complexity:** Low
**Lines of Code:** ~50 (implementation) + ~150 (tests)
**Duration:** 1 day
**Dependencies:** None

### Purpose

Type-erased storage for user-defined variables across REPL evaluations. Based on evcxr's pattern using `Box<dyn Any + 'static>`.

### Implementation Tasks

#### Task 1.1: Create Module Structure (15 min)

**File:** `crates/oxur-repl/src/runtime/mod.rs`

```rust
//! Runtime components for REPL execution
//!
//! Provides variable storage and execution environment.

mod variable_store;

pub use variable_store::{VariableStore, VariableStoreError};

#[cfg(test)]
mod variable_store_tests;
```

**File:** `crates/oxur-repl/src/lib.rs`

```rust
// Add to exports
pub mod runtime;
```

#### Task 1.2: Define Error Types (30 min)

**File:** `crates/oxur-repl/src/runtime/variable_store.rs`

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum VariableStoreError {
    #[error("Variable '{name}' not found")]
    VariableNotFound { name: String },

    #[error("Type mismatch for variable '{name}': expected {expected}, found {found}")]
    TypeMismatch {
        name: String,
        expected: &'static str,
        found: &'static str,
    },

    #[error("Variable '{name}' already exists")]
    VariableExists { name: String },
}

pub type Result<T> = std::result::Result<T, VariableStoreError>;
```

**Key decisions:**
- Use thiserror for ergonomic error definitions
- Include variable name in all errors for debugging
- Type names for mismatch errors (use `std::any::type_name`)
- Clone + PartialEq for testing

#### Task 1.3: Implement Core Storage (1 hour)

**File:** `crates/oxur-repl/src/runtime/variable_store.rs`

```rust
/// Type-erased variable storage using `Box<dyn Any>`.
///
/// Allows storing arbitrary user-defined types without serialization.
/// Based on evcxr's variable storage pattern.
///
/// # Example
///
/// ```rust
/// use oxur_repl::runtime::VariableStore;
///
/// let mut store = VariableStore::new();
///
/// // Store a value
/// store.put_variable("x", 42i32).unwrap();
///
/// // Check type
/// assert!(store.check_variable::<i32>("x"));
///
/// // Retrieve value (consumes it)
/// let value: i32 = store.take_variable("x").unwrap();
/// assert_eq!(value, 42);
/// ```
#[derive(Debug, Default)]
pub struct VariableStore {
    variables: HashMap<String, Box<dyn Any + 'static>>,
}

impl VariableStore {
    /// Creates a new empty variable store.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Stores a variable with the given name.
    ///
    /// # Errors
    ///
    /// Returns `VariableExists` if a variable with this name already exists.
    pub fn put_variable<T: 'static>(&mut self, name: impl Into<String>, value: T) -> Result<()> {
        let name = name.into();

        if self.variables.contains_key(&name) {
            return Err(VariableStoreError::VariableExists { name });
        }

        self.variables.insert(name, Box::new(value));
        Ok(())
    }

    /// Checks if a variable exists with the correct type.
    ///
    /// Returns `true` if the variable exists and has type `T`.
    pub fn check_variable<T: 'static>(&self, name: &str) -> bool {
        self.variables
            .get(name)
            .map(|boxed| boxed.as_ref().type_id() == TypeId::of::<T>())
            .unwrap_or(false)
    }

    /// Takes ownership of a variable, removing it from the store.
    ///
    /// # Errors
    ///
    /// - `VariableNotFound` if the variable doesn't exist
    /// - `TypeMismatch` if the variable has a different type than `T`
    pub fn take_variable<T: 'static>(&mut self, name: &str) -> Result<T> {
        let boxed = self.variables.remove(name).ok_or_else(|| {
            VariableStoreError::VariableNotFound {
                name: name.to_string(),
            }
        })?;

        boxed.downcast::<T>().map(|b| *b).map_err(|boxed| {
            let expected = std::any::type_name::<T>();
            let found = std::any::type_name_of_val(boxed.as_ref());

            // Put it back since downcast failed
            self.variables.insert(name.to_string(), boxed);

            VariableStoreError::TypeMismatch {
                name: name.to_string(),
                expected,
                found,
            }
        })
    }

    /// Returns the number of variables stored.
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Returns `true` if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Removes all variables from the store.
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Checks if a variable exists (regardless of type).
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
}
```

**Key implementation notes:**
- `put_variable` errors if variable exists (no implicit overwrite)
- `take_variable` removes and returns the value (consuming ownership)
- `check_variable` uses `TypeId` for type checking
- On downcast failure, restore the variable (don't lose data)
- Use `std::any::type_name` for helpful error messages

#### Task 1.4: Write Comprehensive Tests (2 hours)

**File:** `crates/oxur-repl/src/runtime/variable_store_tests.rs`

```rust
use super::*;

// ========================================
// Basic Operations Tests
// ========================================

#[test]
fn test_new_store_is_empty() {
    let store = VariableStore::new();
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
}

#[test]
fn test_put_and_take_primitive() {
    let mut store = VariableStore::new();

    store.put_variable("x", 42i32).unwrap();
    assert_eq!(store.len(), 1);
    assert!(!store.is_empty());

    let value: i32 = store.take_variable("x").unwrap();
    assert_eq!(value, 42);
    assert_eq!(store.len(), 0);
}

#[test]
fn test_put_multiple_types() {
    let mut store = VariableStore::new();

    store.put_variable("int", 42i32).unwrap();
    store.put_variable("float", 3.14f64).unwrap();
    store.put_variable("string", "hello".to_string()).unwrap();
    store.put_variable("bool", true).unwrap();

    assert_eq!(store.len(), 4);
}

#[test]
fn test_check_variable_correct_type() {
    let mut store = VariableStore::new();
    store.put_variable("x", 42i32).unwrap();

    assert!(store.check_variable::<i32>("x"));
    assert!(!store.check_variable::<f64>("x"));
    assert!(!store.check_variable::<String>("x"));
}

#[test]
fn test_check_variable_nonexistent() {
    let store = VariableStore::new();
    assert!(!store.check_variable::<i32>("nonexistent"));
}

#[test]
fn test_contains() {
    let mut store = VariableStore::new();
    store.put_variable("x", 42i32).unwrap();

    assert!(store.contains("x"));
    assert!(!store.contains("y"));
}

#[test]
fn test_clear() {
    let mut store = VariableStore::new();
    store.put_variable("x", 42i32).unwrap();
    store.put_variable("y", 3.14f64).unwrap();

    store.clear();
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
}

// ========================================
// User-Defined Types Tests
// ========================================

#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn test_put_and_take_user_type() {
    let mut store = VariableStore::new();

    let point = Point { x: 10, y: 20 };
    store.put_variable("p", point).unwrap();

    let retrieved = store.take_variable::<Point>("p").unwrap();
    assert_eq!(retrieved, Point { x: 10, y: 20 });
}

#[test]
fn test_check_variable_user_type() {
    let mut store = VariableStore::new();
    store.put_variable("p", Point { x: 1, y: 2 }).unwrap();

    assert!(store.check_variable::<Point>("p"));
    assert!(!store.check_variable::<i32>("p"));
}

// ========================================
// Error Cases Tests
// ========================================

#[test]
fn test_take_nonexistent_variable() {
    let mut store = VariableStore::new();

    let result = store.take_variable::<i32>("nonexistent");
    assert!(matches!(
        result,
        Err(VariableStoreError::VariableNotFound { name }) if name == "nonexistent"
    ));
}

#[test]
fn test_take_wrong_type() {
    let mut store = VariableStore::new();
    store.put_variable("x", 42i32).unwrap();

    let result = store.take_variable::<f64>("x");
    assert!(matches!(
        result,
        Err(VariableStoreError::TypeMismatch { name, .. }) if name == "x"
    ));

    // Variable should still exist after failed downcast
    assert!(store.contains("x"));
    assert_eq!(store.len(), 1);
}

#[test]
fn test_put_duplicate_variable() {
    let mut store = VariableStore::new();
    store.put_variable("x", 42i32).unwrap();

    let result = store.put_variable("x", 100i32);
    assert!(matches!(
        result,
        Err(VariableStoreError::VariableExists { name }) if name == "x"
    ));

    // Original value should be unchanged
    let value = store.take_variable::<i32>("x").unwrap();
    assert_eq!(value, 42);
}

// ========================================
// Complex Scenarios Tests
// ========================================

#[test]
fn test_store_vec_of_user_types() {
    let mut store = VariableStore::new();

    let points = vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
    store.put_variable("points", points).unwrap();

    let retrieved = store.take_variable::<Vec<Point>>("points").unwrap();
    assert_eq!(retrieved.len(), 2);
    assert_eq!(retrieved[0], Point { x: 1, y: 2 });
}

#[test]
fn test_store_closure() {
    let mut store = VariableStore::new();

    let f = Box::new(|x: i32| x * 2) as Box<dyn Fn(i32) -> i32>;
    store.put_variable("f", f).unwrap();

    let retrieved = store.take_variable::<Box<dyn Fn(i32) -> i32>>("f").unwrap();
    assert_eq!(retrieved(21), 42);
}

#[test]
fn test_lifecycle_sequence() {
    let mut store = VariableStore::new();

    // Define x
    store.put_variable("x", 10i32).unwrap();
    assert_eq!(store.len(), 1);

    // Define y
    store.put_variable("y", 20i32).unwrap();
    assert_eq!(store.len(), 2);

    // Use x (take and redefine)
    let x_val = store.take_variable::<i32>("x").unwrap();
    store.put_variable("x", x_val + 5).unwrap();

    // Check final state
    assert_eq!(store.take_variable::<i32>("x").unwrap(), 15);
    assert_eq!(store.take_variable::<i32>("y").unwrap(), 20);
    assert!(store.is_empty());
}
```

**Test coverage checklist:**
- [x] Basic operations (new, put, take, check)
- [x] Primitive types (i32, f64, String, bool)
- [x] User-defined types (struct)
- [x] Complex types (Vec, Box<dyn Fn>)
- [x] Error cases (not found, type mismatch, duplicate)
- [x] Edge cases (empty store, clear)
- [x] Lifecycle scenarios (define, use, redefine)

#### Task 1.5: Documentation (30 min)

- Module-level docs explaining the pattern
- Doc comments on all public items
- Usage examples in doc comments
- Link to ODD-0030 section 3.1

### Success Criteria

- [ ] All tests pass (100% coverage)
- [ ] No clippy warnings
- [ ] Documentation complete
- [ ] Can store and retrieve primitives
- [ ] Can store and retrieve user types
- [ ] Type mismatch errors are helpful
- [ ] Variable preservation on failed downcast

---

## Component 2: SessionDir

**Complexity:** Medium
**Lines of Code:** ~200 (implementation) + ~300 (tests)
**Duration:** 1 day
**Dependencies:** None

### Purpose

Manages per-session temporary directories and Cargo project setup. Each REPL session gets its own isolated directory with a Cargo.toml.

### Implementation Tasks

#### Task 2.1: Add Dependencies (15 min)

**File:** `crates/oxur-repl/Cargo.toml`

```toml
[dependencies]
tempfile = "3.8"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

[dev-dependencies]
assert_fs = "1.0"  # For filesystem testing
predicates = "3.0"
```

#### Task 2.2: Define SessionDir Structure (1 hour)

**File:** `crates/oxur-repl/src/session/dir.rs`

```rust
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionDirError {
    #[error("Failed to create session directory: {0}")]
    CreateFailed(#[from] std::io::Error),

    #[error("Failed to write Cargo.toml: {0}")]
    CargoTomlWriteFailed(std::io::Error),

    #[error("Invalid session directory: {0}")]
    InvalidDirectory(String),
}

pub type Result<T> = std::result::Result<T, SessionDirError>;

/// Manages a temporary directory for a REPL session.
///
/// Each session gets:
/// - A temporary directory (cleaned up on drop)
/// - A Cargo.toml for compiling user code
/// - A src/ directory for generated code
/// - A target/ directory for compilation artifacts
///
/// # Example
///
/// ```rust
/// use oxur_repl::session::SessionDir;
///
/// let session = SessionDir::new("my-session")?;
/// let src_path = session.src_dir();
/// std::fs::write(src_path.join("lib.rs"), "pub fn foo() -> i32 { 42 }")?;
/// ```
pub struct SessionDir {
    /// Temporary directory (cleaned up on drop)
    temp_dir: TempDir,

    /// Session identifier
    session_id: String,
}

impl SessionDir {
    /// Creates a new session directory.
    pub fn new(session_id: impl Into<String>) -> Result<Self> {
        let session_id = session_id.into();
        let temp_dir = TempDir::new()?;

        let session = Self { temp_dir, session_id };
        session.initialize()?;

        Ok(session)
    }

    /// Initializes the directory structure and Cargo.toml.
    fn initialize(&self) -> Result<()> {
        // Create src/ directory
        std::fs::create_dir(self.src_dir())
            .map_err(SessionDirError::CreateFailed)?;

        // Write Cargo.toml
        self.write_cargo_toml()?;

        Ok(())
    }

    /// Returns the root directory path.
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Returns the src/ directory path.
    pub fn src_dir(&self) -> PathBuf {
        self.root().join("src")
    }

    /// Returns the target/ directory path.
    pub fn target_dir(&self) -> PathBuf {
        self.root().join("target")
    }

    /// Returns the path to the generated library file.
    pub fn lib_rs_path(&self) -> PathBuf {
        self.src_dir().join("lib.rs")
    }

    /// Returns the path to Cargo.toml.
    pub fn cargo_toml_path(&self) -> PathBuf {
        self.root().join("Cargo.toml")
    }

    /// Writes the Cargo.toml file.
    fn write_cargo_toml(&self) -> Result<()> {
        let cargo_toml = self.generate_cargo_toml();
        std::fs::write(self.cargo_toml_path(), cargo_toml)
            .map_err(SessionDirError::CargoTomlWriteFailed)?;
        Ok(())
    }

    /// Generates the Cargo.toml content.
    fn generate_cargo_toml(&self) -> String {
        format!(
            r#"[package]
name = "oxur-repl-{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# User dependencies will be added dynamically

[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
"#,
            self.session_id
        )
    }

    /// Returns the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Keeps the directory alive (prevents cleanup on drop).
    ///
    /// Useful for debugging failed compilations.
    pub fn persist(self) -> PathBuf {
        self.temp_dir.into_path()
    }
}
```

#### Task 2.3: Platform-Specific Handling (1 hour)

**File:** `crates/oxur-repl/src/session/dir.rs` (additions)

```rust
impl SessionDir {
    /// Returns the expected compiled library filename.
    ///
    /// Platform-specific:
    /// - Linux: `liboxur-repl-{session_id}.so`
    /// - macOS: `liboxur-repl-{session_id}.dylib`
    /// - Windows: `oxur-repl-{session_id}.dll`
    pub fn lib_filename(&self) -> String {
        if cfg!(target_os = "linux") {
            format!("liboxur_repl_{}.so", self.session_id.replace('-', "_"))
        } else if cfg!(target_os = "macos") {
            format!("liboxur_repl_{}.dylib", self.session_id.replace('-', "_"))
        } else if cfg!(target_os = "windows") {
            format!("oxur_repl_{}.dll", self.session_id.replace('-', "_"))
        } else {
            panic!("Unsupported platform")
        }
    }

    /// Returns the full path to the compiled library.
    pub fn lib_path(&self) -> PathBuf {
        self.target_dir()
            .join("debug")
            .join(self.lib_filename())
    }

    /// Adds a dependency to Cargo.toml.
    pub fn add_dependency(&self, name: &str, version: &str) -> Result<()> {
        let cargo_toml_path = self.cargo_toml_path();
        let mut content = std::fs::read_to_string(&cargo_toml_path)
            .map_err(SessionDirError::CargoTomlWriteFailed)?;

        // Simple append to [dependencies] section
        let dependency_line = format!("{} = \"{}\"\n", name, version);

        if let Some(pos) = content.find("[dependencies]") {
            let insert_pos = content[pos..]
                .find('\n')
                .map(|n| pos + n + 1)
                .unwrap_or(content.len());
            content.insert_str(insert_pos, &dependency_line);
        }

        std::fs::write(&cargo_toml_path, content)
            .map_err(SessionDirError::CargoTomlWriteFailed)?;

        Ok(())
    }

    /// Writes the lib.rs file with the given content.
    pub fn write_lib_rs(&self, content: &str) -> Result<()> {
        std::fs::write(self.lib_rs_path(), content)
            .map_err(SessionDirError::CargoTomlWriteFailed)?;
        Ok(())
    }

    /// Reads the lib.rs file content.
    pub fn read_lib_rs(&self) -> Result<String> {
        std::fs::read_to_string(self.lib_rs_path())
            .map_err(SessionDirError::CreateFailed)
    }
}
```

#### Task 2.4: Write Tests (2 hours)

**File:** `crates/oxur-repl/src/session/dir_tests.rs`

```rust
use super::*;
use std::fs;

#[test]
fn test_new_session_creates_structure() {
    let session = SessionDir::new("test-session").unwrap();

    assert!(session.root().exists());
    assert!(session.src_dir().exists());
    assert!(session.cargo_toml_path().exists());
}

#[test]
fn test_cargo_toml_content() {
    let session = SessionDir::new("test-123").unwrap();
    let cargo_toml = fs::read_to_string(session.cargo_toml_path()).unwrap();

    assert!(cargo_toml.contains("name = \"oxur-repl-test-123\""));
    assert!(cargo_toml.contains("edition = \"2021\""));
    assert!(cargo_toml.contains("crate-type = [\"cdylib\"]"));
}

#[test]
fn test_lib_filename_platform_specific() {
    let session = SessionDir::new("my-session").unwrap();
    let filename = session.lib_filename();

    if cfg!(target_os = "linux") {
        assert!(filename.ends_with(".so"));
        assert!(filename.starts_with("lib"));
    } else if cfg!(target_os = "macos") {
        assert!(filename.ends_with(".dylib"));
        assert!(filename.starts_with("lib"));
    } else if cfg!(target_os = "windows") {
        assert!(filename.ends_with(".dll"));
    }
}

#[test]
fn test_write_and_read_lib_rs() {
    let session = SessionDir::new("test").unwrap();

    let code = "pub fn foo() -> i32 { 42 }";
    session.write_lib_rs(code).unwrap();

    let read_code = session.read_lib_rs().unwrap();
    assert_eq!(read_code, code);
}

#[test]
fn test_add_dependency() {
    let session = SessionDir::new("test").unwrap();

    session.add_dependency("serde", "1.0").unwrap();

    let cargo_toml = fs::read_to_string(session.cargo_toml_path()).unwrap();
    assert!(cargo_toml.contains("serde = \"1.0\""));
}

#[test]
fn test_cleanup_on_drop() {
    let root_path = {
        let session = SessionDir::new("temp").unwrap();
        session.root().to_path_buf()
    };

    // After drop, directory should be cleaned up
    assert!(!root_path.exists());
}

#[test]
fn test_persist_keeps_directory() {
    let session = SessionDir::new("persistent").unwrap();
    let root_path = session.root().to_path_buf();

    session.persist();

    assert!(root_path.exists());

    // Clean up manually
    fs::remove_dir_all(&root_path).unwrap();
}
```

### Success Criteria

- [ ] Creates temp directory structure
- [ ] Generates valid Cargo.toml
- [ ] Platform-specific library naming
- [ ] Dependency management works
- [ ] Cleanup on drop
- [ ] Persist option for debugging

---

## Component 3: Cargo Integration

**Complexity:** Medium
**Lines of Code:** ~300 (implementation) + ~400 (tests)
**Duration:** 1-2 days
**Dependencies:** SessionDir (Component 2)

### Purpose

Wrapper around cargo build command with JSON message parsing and error extraction.

### Implementation Tasks

#### Task 3.1: Add Dependencies (15 min)

**File:** `crates/oxur-repl/Cargo.toml`

```toml
[dependencies]
serde_json = "1.0"
tokio = { version = "1.35", features = ["process", "io-util"] }
```

#### Task 3.2: Define Cargo Types (1 hour)

**File:** `crates/oxur-repl/src/compiler/cargo.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum CargoError {
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Cargo execution failed: {0}")]
    ExecutionFailed(#[from] std::io::Error),

    #[error("JSON parsing failed: {0}")]
    JsonParseFailed(#[from] serde_json::Error),

    #[error("Library artifact not found")]
    ArtifactNotFound,
}

pub type Result<T> = std::result::Result<T, CargoError>;

/// Cargo JSON message format.
///
/// Based on cargo's --message-format=json output.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "reason")]
pub enum CargoMessage {
    #[serde(rename = "compiler-artifact")]
    CompilerArtifact {
        target: Target,
        filenames: Vec<PathBuf>,
        fresh: bool,
    },

    #[serde(rename = "compiler-message")]
    CompilerMessage { message: DiagnosticMessage },

    #[serde(rename = "build-finished")]
    BuildFinished { success: bool },

    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Target {
    pub name: String,
    pub kind: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagnosticMessage {
    pub message: String,
    pub level: String,
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Span {
    pub file_name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub is_primary: bool,
}

/// Result of a cargo build operation.
#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    pub artifacts: Vec<PathBuf>,
    pub errors: Vec<DiagnosticMessage>,
    pub warnings: Vec<DiagnosticMessage>,
}
```

#### Task 3.3: Implement Cargo Wrapper (2 hours)

**File:** `crates/oxur-repl/src/compiler/cargo.rs` (continued)

```rust
/// Wrapper for invoking cargo and parsing output.
pub struct CargoBuilder {
    /// Path to cargo binary (defaults to "cargo")
    cargo_path: PathBuf,

    /// Extra environment variables
    env_vars: Vec<(String, String)>,
}

impl Default for CargoBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CargoBuilder {
    /// Creates a new cargo builder with default settings.
    pub fn new() -> Self {
        Self {
            cargo_path: PathBuf::from("cargo"),
            env_vars: Vec::new(),
        }
    }

    /// Sets the path to the cargo binary.
    pub fn with_cargo_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.cargo_path = path.into();
        self
    }

    /// Adds an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Detects and configures a fast linker (lld or mold).
    pub fn with_fast_linker(mut self) -> Self {
        // Try mold first (fastest), then lld
        if which::which("mold").is_ok() {
            self.env_vars.push(("RUSTFLAGS".into(), "-C link-arg=-fuse-ld=mold".into()));
        } else if which::which("lld").is_ok() {
            self.env_vars.push(("RUSTFLAGS".into(), "-C link-arg=-fuse-ld=lld".into()));
        }
        self
    }

    /// Builds a library in the given directory.
    pub async fn build_lib(&self, project_dir: &Path) -> Result<BuildResult> {
        let mut cmd = Command::new(&self.cargo_path);
        cmd.arg("build")
            .arg("--message-format=json")
            .arg("--lib")
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn()?;

        // Read stdout (JSON messages)
        let stdout = child.stdout.take().expect("stdout was piped");
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut artifacts = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut success = false;

        while let Some(line) = lines.next_line().await? {
            if let Ok(msg) = serde_json::from_str::<CargoMessage>(&line) {
                match msg {
                    CargoMessage::CompilerArtifact { filenames, .. } => {
                        artifacts.extend(filenames);
                    }
                    CargoMessage::CompilerMessage { message } => {
                        match message.level.as_str() {
                            "error" => errors.push(message),
                            "warning" => warnings.push(message),
                            _ => {}
                        }
                    }
                    CargoMessage::BuildFinished { success: s } => {
                        success = s;
                    }
                    CargoMessage::Other => {}
                }
            }
        }

        // Wait for process to complete
        let status = child.wait().await?;

        if !status.success() {
            let error_msg = errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join("\n");
            return Err(CargoError::CompilationFailed(error_msg));
        }

        Ok(BuildResult {
            success,
            artifacts,
            errors,
            warnings,
        })
    }
}
```

#### Task 3.4: Add Helper for Linker Detection (30 min)

**File:** `crates/oxur-repl/Cargo.toml`

```toml
[dependencies]
which = "6.0"  # For finding executables
```

**File:** `crates/oxur-repl/src/compiler/cargo.rs` (additions)

```rust
impl CargoBuilder {
    /// Detects the best available linker.
    pub fn detect_linker() -> Option<&'static str> {
        if which::which("mold").is_ok() {
            Some("mold")
        } else if which::which("lld").is_ok() {
            Some("lld")
        } else {
            None
        }
    }
}
```

#### Task 3.5: Write Tests (3 hours)

**File:** `crates/oxur-repl/src/compiler/cargo_tests.rs`

```rust
use super::*;
use crate::session::SessionDir;

#[tokio::test]
async fn test_build_simple_library() {
    let session = SessionDir::new("test-build").unwrap();

    // Write simple library code
    session.write_lib_rs("pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

    let builder = CargoBuilder::new();
    let result = builder.build_lib(session.root()).await.unwrap();

    assert!(result.success);
    assert!(!result.artifacts.is_empty());
    assert!(result.errors.is_empty());
}

#[tokio::test]
async fn test_build_with_compilation_error() {
    let session = SessionDir::new("test-error").unwrap();

    // Invalid Rust code
    session.write_lib_rs("pub fn bad() -> i32 { \"not an int\" }").unwrap();

    let builder = CargoBuilder::new();
    let result = builder.build_lib(session.root()).await;

    assert!(result.is_err());
    if let Err(CargoError::CompilationFailed(msg)) = result {
        assert!(msg.contains("mismatched types") || msg.contains("expected"));
    }
}

#[tokio::test]
async fn test_build_with_warnings() {
    let session = SessionDir::new("test-warning").unwrap();

    // Code with unused variable
    session.write_lib_rs(r#"
        pub fn with_warning() -> i32 {
            let unused = 42;
            100
        }
    "#).unwrap();

    let builder = CargoBuilder::new();
    let result = builder.build_lib(session.root()).await.unwrap();

    assert!(result.success);
    // May or may not have warnings depending on rustc version
}

#[test]
fn test_detect_linker() {
    // This test just verifies the function doesn't panic
    let linker = CargoBuilder::detect_linker();
    println!("Detected linker: {:?}", linker);
}

#[tokio::test]
async fn test_with_fast_linker() {
    let session = SessionDir::new("test-linker").unwrap();
    session.write_lib_rs("pub fn foo() {}").unwrap();

    let builder = CargoBuilder::new().with_fast_linker();
    let result = builder.build_lib(session.root()).await.unwrap();

    assert!(result.success);
}
```

### Success Criteria

- [ ] Invokes cargo successfully
- [ ] Parses JSON output
- [ ] Extracts compilation errors
- [ ] Handles warnings
- [ ] Fast linker detection works
- [ ] Integration with SessionDir

---

## Component 4: Subprocess Runtime

**Complexity:** High
**Lines of Code:** ~400-500 (implementation) + ~400 (tests)
**Duration:** 2 days
**Dependencies:** VariableStore (Component 1)

### Purpose

Separate binary that loads and executes compiled dynamic libraries in isolation.

### Implementation Tasks

#### Task 4.1: Create New Crate (30 min)

**File:** `crates/oxur-subprocess/Cargo.toml`

```toml
[package]
name = "oxur-subprocess"
version = "0.1.0"
edition = "2021"

[dependencies]
libloading = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tokio = { version = "1.35", features = ["full"] }

[dev-dependencies]
tempfile = "3.8"
```

**File:** `Cargo.toml` (workspace root)

```toml
[workspace]
members = [
    "crates/design",
    "crates/oxur-ast",
    "crates/oxur-cli",
    "crates/oxur-comp",
    "crates/oxur-lang",
    "crates/oxur-repl",
    "crates/oxur-subprocess",  # [NEW]
]
```

#### Task 4.2: Define Communication Protocol (1 hour)

**File:** `crates/oxur-subprocess/src/protocol.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Commands sent from REPL to subprocess.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubprocessCommand {
    /// Load a dynamic library.
    LoadLibrary { path: PathBuf },

    /// Execute a function from the loaded library.
    Execute { function_name: String },

    /// Shutdown the subprocess.
    Shutdown,
}

/// Responses sent from subprocess to REPL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubprocessResponse {
    /// Library loaded successfully.
    LibraryLoaded,

    /// Execution completed successfully.
    ExecutionSuccess { output: String },

    /// An error occurred.
    Error { message: String },

    /// Subprocess is shutting down.
    ShuttingDown,
}
```

#### Task 4.3: Implement Library Loader (2 hours)

**File:** `crates/oxur-subprocess/src/loader.rs`

```rust
use libloading::{Library, Symbol};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Failed to load library: {0}")]
    LoadFailed(#[from] libloading::Error),

    #[error("Symbol '{symbol}' not found")]
    SymbolNotFound { symbol: String },

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

pub type Result<T> = std::result::Result<T, LoaderError>;

/// Loads and executes dynamic libraries.
pub struct LibraryLoader {
    /// Currently loaded library
    library: Option<Library>,
}

impl LibraryLoader {
    pub fn new() -> Self {
        Self { library: None }
    }

    /// Loads a dynamic library from the given path.
    pub fn load(&mut self, path: &Path) -> Result<()> {
        // Safety: We're loading libraries compiled by our own process
        let library = unsafe { Library::new(path)? };
        self.library = Some(library);
        Ok(())
    }

    /// Executes a function from the loaded library.
    ///
    /// The function must have signature: `extern "C" fn() -> *mut String`
    pub fn execute(&self, function_name: &str) -> Result<String> {
        let library = self.library.as_ref().ok_or_else(|| {
            LoaderError::ExecutionFailed("No library loaded".to_string())
        })?;

        // Safety: We compiled this library ourselves with known signature
        let func: Symbol<extern "C" fn() -> *mut String> = unsafe {
            library
                .get(function_name.as_bytes())
                .map_err(|_| LoaderError::SymbolNotFound {
                    symbol: function_name.to_string(),
                })?
        };

        // Execute the function
        let result_ptr = func();

        // Safety: The function returns a Box<String> leaked as raw pointer
        let result = unsafe { Box::from_raw(result_ptr) };

        Ok(*result)
    }

    /// Unloads the current library.
    pub fn unload(&mut self) {
        self.library = None;
    }
}

impl Default for LibraryLoader {
    fn default() -> Self {
        Self::new()
    }
}
```

#### Task 4.4: Implement Main Subprocess Loop (2 hours)

**File:** `crates/oxur-subprocess/src/main.rs`

```rust
mod loader;
mod protocol;

use loader::LibraryLoader;
use protocol::{SubprocessCommand, SubprocessResponse};
use std::io::{self, BufRead, Write};

fn main() {
    if let Err(e) = run() {
        eprintln!("Subprocess error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut loader = LibraryLoader::new();

    for line in stdin.lock().lines() {
        let line = line?;

        let command: SubprocessCommand = match serde_json::from_str(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                send_response(&mut stdout, SubprocessResponse::Error {
                    message: format!("Invalid command: {}", e),
                })?;
                continue;
            }
        };

        let response = handle_command(&mut loader, command);
        send_response(&mut stdout, response)?;

        if matches!(response, SubprocessResponse::ShuttingDown) {
            break;
        }
    }

    Ok(())
}

fn handle_command(loader: &mut LibraryLoader, command: SubprocessCommand) -> SubprocessResponse {
    match command {
        SubprocessCommand::LoadLibrary { path } => match loader.load(&path) {
            Ok(_) => SubprocessResponse::LibraryLoaded,
            Err(e) => SubprocessResponse::Error {
                message: format!("Load failed: {}", e),
            },
        },

        SubprocessCommand::Execute { function_name } => match loader.execute(&function_name) {
            Ok(output) => SubprocessResponse::ExecutionSuccess { output },
            Err(e) => SubprocessResponse::Error {
                message: format!("Execution failed: {}", e),
            },
        },

        SubprocessCommand::Shutdown => {
            loader.unload();
            SubprocessResponse::ShuttingDown
        }
    }
}

fn send_response(stdout: &mut io::Stdout, response: SubprocessResponse) -> io::Result<()> {
    let json = serde_json::to_string(&response).expect("Serialize response");
    writeln!(stdout, "{}", json)?;
    stdout.flush()?;
    Ok(())
}
```

#### Task 4.5: Add Subprocess Client (2 hours)

**File:** `crates/oxur-repl/src/compiler/subprocess.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

// Re-export protocol types
pub use crate::subprocess_protocol::{SubprocessCommand, SubprocessResponse};

#[derive(Debug, Error)]
pub enum SubprocessError {
    #[error("Failed to spawn subprocess: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("Subprocess communication error: {0}")]
    CommunicationFailed(String),

    #[error("Subprocess returned error: {0}")]
    SubprocessError(String),
}

pub type Result<T> = std::result::Result<T, SubprocessError>;

/// Client for communicating with the subprocess runtime.
pub struct SubprocessClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl SubprocessClient {
    /// Spawns a new subprocess.
    pub async fn spawn() -> Result<Self> {
        let mut child = Command::new("oxur-subprocess")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.take().expect("stdin was piped");
        let stdout = child.stdout.take().expect("stdout was piped");
        let stdout = BufReader::new(stdout);

        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    /// Sends a command and waits for response.
    pub async fn send_command(&mut self, command: SubprocessCommand) -> Result<SubprocessResponse> {
        // Send command
        let json = serde_json::to_string(&command).expect("Serialize command");
        self.stdin
            .write_all(json.as_bytes())
            .await
            .map_err(|e| SubprocessError::CommunicationFailed(e.to_string()))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| SubprocessError::CommunicationFailed(e.to_string()))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| SubprocessError::CommunicationFailed(e.to_string()))?;

        // Read response
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .await
            .map_err(|e| SubprocessError::CommunicationFailed(e.to_string()))?;

        let response: SubprocessResponse = serde_json::from_str(&line)
            .map_err(|e| SubprocessError::CommunicationFailed(e.to_string()))?;

        Ok(response)
    }

    /// Loads a library in the subprocess.
    pub async fn load_library(&mut self, path: PathBuf) -> Result<()> {
        let response = self
            .send_command(SubprocessCommand::LoadLibrary { path })
            .await?;

        match response {
            SubprocessResponse::LibraryLoaded => Ok(()),
            SubprocessResponse::Error { message } => {
                Err(SubprocessError::SubprocessError(message))
            }
            _ => Err(SubprocessError::CommunicationFailed(
                "Unexpected response".to_string(),
            )),
        }
    }

    /// Executes a function in the subprocess.
    pub async fn execute(&mut self, function_name: String) -> Result<String> {
        let response = self
            .send_command(SubprocessCommand::Execute { function_name })
            .await?;

        match response {
            SubprocessResponse::ExecutionSuccess { output } => Ok(output),
            SubprocessResponse::Error { message } => {
                Err(SubprocessError::SubprocessError(message))
            }
            _ => Err(SubprocessError::CommunicationFailed(
                "Unexpected response".to_string(),
            )),
        }
    }

    /// Shuts down the subprocess.
    pub async fn shutdown(mut self) -> Result<()> {
        let _ = self.send_command(SubprocessCommand::Shutdown).await;
        let _ = self.child.wait().await;
        Ok(())
    }
}
```

#### Task 4.6: Write Tests (3 hours)

Create integration tests that compile a simple library and execute it via subprocess.

**File:** `crates/oxur-subprocess/tests/integration_tests.rs`

```rust
use oxur_subprocess::{LibraryLoader, SubprocessCommand, SubprocessResponse};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_compile_and_load_simple_library() {
    // This test requires a full compilation setup
    // See test implementation in actual code
}
```

### Success Criteria

- [ ] Subprocess spawns successfully
- [ ] Loads dynamic libraries
- [ ] Executes functions
- [ ] Communication protocol works
- [ ] Error handling robust
- [ ] Integration tests pass

---

## Integration Testing

### Cross-Component Tests

**File:** `crates/oxur-repl/tests/phase1_integration.rs`

```rust
use oxur_repl::compiler::CargoBuilder;
use oxur_repl::session::SessionDir;

#[tokio::test]
async fn test_full_compilation_pipeline() {
    // 1. Create session directory
    let session = SessionDir::new("integration-test").unwrap();

    // 2. Write Rust code
    session
        .write_lib_rs(
            r#"
        #[no_mangle]
        pub extern "C" fn eval_result() -> *mut String {
            Box::into_raw(Box::new("42".to_string()))
        }
    "#,
        )
        .unwrap();

    // 3. Compile with cargo
    let builder = CargoBuilder::new().with_fast_linker();
    let result = builder.build_lib(session.root()).await.unwrap();

    assert!(result.success);
    assert!(!result.artifacts.is_empty());

    // 4. Verify library exists
    assert!(session.lib_path().exists());
}
```

---

## Success Criteria

### Per-Component Criteria

**VariableStore:**
- [ ] All unit tests pass (100% coverage)
- [ ] Can store primitives and user types
- [ ] Type checking works correctly
- [ ] Error messages are helpful
- [ ] Documentation complete

**SessionDir:**
- [ ] Creates valid directory structure
- [ ] Generates correct Cargo.toml
- [ ] Platform-specific naming works
- [ ] Cleanup works (drop and persist)
- [ ] Dependency addition works

**Cargo Integration:**
- [ ] Successfully invokes cargo
- [ ] Parses JSON output correctly
- [ ] Extracts errors and warnings
- [ ] Fast linker detection works
- [ ] Integration with SessionDir works

**Subprocess Runtime:**
- [ ] Subprocess spawns successfully
- [ ] Loads libraries correctly
- [ ] Executes functions
- [ ] Communication protocol works
- [ ] Error handling is robust

### Overall Phase 1 Criteria

- [ ] All components implemented
- [ ] All tests passing (95%+ coverage)
- [ ] No clippy warnings
- [ ] Documentation complete
- [ ] Integration tests pass
- [ ] Can compile and execute simple Rust code via full pipeline

---

## Timeline & Execution Order

### Day 1: VariableStore + SessionDir Setup

**Morning (3-4 hours):**
- Task 1.1-1.3: VariableStore implementation
- Task 1.4: VariableStore tests (partial)

**Afternoon (3-4 hours):**
- Complete VariableStore tests
- Task 1.5: VariableStore documentation
- Task 2.1-2.2: SessionDir structure

**Deliverable:** VariableStore complete, SessionDir started

### Day 2: Complete SessionDir

**Morning (3-4 hours):**
- Task 2.3: Platform-specific handling
- Task 2.4: SessionDir tests (partial)

**Afternoon (3-4 hours):**
- Complete SessionDir tests
- SessionDir documentation
- Begin Cargo integration planning

**Deliverable:** SessionDir complete

### Day 3: Cargo Integration

**Morning (3-4 hours):**
- Task 3.1-3.2: Cargo types and structures
- Task 3.3: Cargo wrapper implementation (partial)

**Afternoon (3-4 hours):**
- Complete Cargo wrapper
- Task 3.4: Linker detection
- Begin Cargo tests

**Deliverable:** Cargo integration implemented

### Day 4: Complete Cargo + Start Subprocess

**Morning (3-4 hours):**
- Task 3.5: Complete Cargo tests
- Cargo documentation

**Afternoon (3-4 hours):**
- Task 4.1: Create oxur-subprocess crate
- Task 4.2: Protocol definition
- Task 4.3: Library loader (partial)

**Deliverable:** Cargo complete, subprocess started

### Day 5: Subprocess Runtime

**Morning (3-4 hours):**
- Complete Task 4.3: Library loader
- Task 4.4: Main subprocess loop

**Afternoon (3-4 hours):**
- Task 4.5: Subprocess client
- Begin subprocess tests

**Deliverable:** Subprocess runtime implemented

### Day 6: Testing & Integration

**Morning (3-4 hours):**
- Task 4.6: Complete subprocess tests
- Integration testing
- Bug fixes

**Afternoon (3-4 hours):**
- Documentation review
- Coverage verification
- Integration tests
- Final polish

**Deliverable:** Phase 1 complete!

### Flexible Buffer (Optional Day 7)

- Performance testing
- Additional edge case tests
- Documentation improvements
- Code review and refactoring

---

## Dependencies to Install

```bash
# Rust toolchain (should already be installed)
rustup component add clippy rustfmt

# Coverage tool
cargo install cargo-llvm-cov

# Optional: Fast linkers for better compilation speed
# Linux:
sudo apt install lld mold  # Debian/Ubuntu

# macOS:
brew install llvm  # Includes lld
```

---

## Commands for Execution

```bash
# Start Phase 1 implementation
cd /Users/oubiwann/lab/oxur/oxur

# Run tests continuously
cargo watch -x "test --package oxur-repl"

# Check coverage
cargo llvm-cov --package oxur-repl --html
open target/llvm-cov/html/index.html

# Lint
cargo clippy --package oxur-repl -- -D warnings

# Format
cargo fmt --package oxur-repl

# Build all
make build

# Run full check
make check-all
```

---

## Notes and Considerations

### Design Decisions

1. **VariableStore uses Box<dyn Any>** - This is the evcxr pattern. Alternative would be serialization, but that requires Serialize bounds.

2. **SessionDir uses TempDir** - Automatic cleanup is critical for REPL sessions. `persist()` allows debugging.

3. **Cargo JSON parsing** - Using serde for robust parsing. Alternative would be regex, but JSON is more reliable.

4. **Subprocess isolation** - Separate process protects main REPL from user code crashes.

5. **Communication via stdin/stdout** - Simple, robust, cross-platform. Alternative would be Unix sockets.

### Potential Issues

1. **Platform differences** - Library extensions (.so/.dylib/.dll) need careful handling
2. **Linker availability** - Fast linkers may not be installed on all systems
3. **Subprocess lifecycle** - Need robust cleanup even on errors
4. **Type erasure limitations** - `Box<dyn Any>` can't cross FFI boundaries easily

### Future Enhancements (Post-Phase 1)

- Incremental compilation support
- Dependency caching
- Better error messages with source positions
- Performance monitoring
- Resource limits for subprocess

---

## Checklist for Completion

### Before Starting
- [ ] Read ODD-0030 (REPL Implementation Spec)
- [ ] Read ODD-0031 (Proto-Plans)
- [ ] Ensure Rust toolchain up to date
- [ ] Install cargo-llvm-cov
- [ ] Review existing oxur-repl code

### During Implementation
- [ ] Follow TDD (write tests first)
- [ ] Run clippy frequently
- [ ] Check coverage after each component
- [ ] Update documentation as you go
- [ ] Commit after each component

### Before Declaring Complete
- [ ] All tests pass
- [ ] Coverage ≥ 95%
- [ ] No clippy warnings
- [ ] All public items documented
- [ ] Integration tests pass
- [ ] README updated
- [ ] Design docs referenced in code comments

---

**End of Implementation Plan**

Ready to execute! Start with Component 1 (VariableStore) and proceed sequentially through Components 2-4.
