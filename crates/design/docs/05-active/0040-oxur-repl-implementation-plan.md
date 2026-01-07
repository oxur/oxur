---
number: 40
title: "Oxur REPL Implementation Plan"
author: "level of"
component: All
tags: [change-me]
created: 2026-01-06
updated: 2026-01-06
state: Active
supersedes: null
superseded-by: null
version: 1.1
---

# Oxur REPL Implementation Plan

**Document Version:** 1.1
**Date:** 2026-01-07
**Purpose:** Detailed implementation guide for completing oxur-repl based on ODD-0038 spec and code analysis
**Audience:** Claude Code (implementation agent)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Implementation Phases](#2-implementation-phases)
3. [Phase 0: Immediate Quality Tasks](#3-phase-0-immediate-quality-tasks)
4. [Phase 1: VariableStore + Subprocess Runtime](#4-phase-1-variablestore--subprocess-runtime)
5. [Phase 2: RustAstWrapper](#5-phase-2-rustastawrapper)
6. [Phase 3: EvalContext + Integration](#6-phase-3-evalcontext--integration)
7. [Phase 4: SourceMap Integration](#7-phase-4-sourcemap-integration)
8. [Phase 5: Testing & Polish](#8-phase-5-testing--polish)
9. [Phase 6: CLI Integration](#9-phase-6-cli-integration)
10. [Summary: Implementation Order](#10-summary-implementation-order)
11. [Success Criteria](#11-success-criteria)
12. [Notes for Claude Code](#12-notes-for-claude-code)
13. [Version History](#13-version-history)

---

## 1. Executive Summary

Based on comprehensive review of:

1. **ODD-0038** - [Oxur REPL Architecture Specification](./01-draft/0038-oxur-repl-architecture.md) (5,634 lines)
2. **Claude Code's Analysis Report** - [Implementation progress and quality assessment](../dev/oxur-repl/0019-oxur-repl-analysis-and-status-report.md) (1,616 lines)

### Current State

- **Implementation Progress:** 75-80% complete
- **Code Quality:** Excellent (242 tests, 100% pass rate, minimal tech debt)
- **Architecture:** Sound foundation aligned with spec

### What's Working

- ✅ Protocol layer (messages, codec, serialization)
- ✅ Server infrastructure (ReplServer, MessageHandler, SessionManager)
- ✅ Session management (SessionDir with tmpfs optimization)
- ✅ Artifact caching (SHA256 content-addressed, LRU eviction)
- ✅ CachedCompiler (Rust compilation with caching)
- ✅ Transport abstraction (TCP with trait-based design)
- ✅ SubprocessExecutor (lifecycle management, Phase 1)

### Critical Gaps (Must Implement)

1. **RustAstWrapper** - REPL scaffolding generation
2. **Subprocess IPC Protocol** - Complete stdin/stdout protocol
3. **Subprocess Runtime** - Binary that executes code
4. **VariableStore** - Type-erased variable persistence
5. **EvalContext** - Full evaluation orchestration
6. **SourceMap Integration** - Error position translation

---

## 2. Implementation Phases

### Overview

| Phase | Focus | Duration | Priority |
|-------|-------|----------|----------|
| **Phase 0** | Immediate Quality Tasks | 1-2 days | HIGH |
| **Phase 1** | VariableStore + Subprocess Runtime | 1-2 weeks | CRITICAL |
| **Phase 2** | RustAstWrapper | 2-3 weeks | CRITICAL |
| **Phase 3** | EvalContext + Integration | 1-2 weeks | CRITICAL |
| **Phase 4** | SourceMap Integration | 1-2 weeks | HIGH |
| **Phase 5** | Testing & Polish | 1-2 weeks | HIGH |
| **Phase 6** | CLI Integration | 1-2 weeks | HIGH |

**Total Estimated Time:** 8-13 weeks

---

## 3. Phase 0: Immediate Quality Tasks

**Duration:** 1-2 days
**Priority:** HIGH (do first before any implementation)

### Task 0.1: Run Linting and Fix Issues

```bash
cd crates/oxur-repl
cargo clippy --all-targets --all-features
cargo fmt --check
```

**Action:** Fix any warnings or formatting issues before proceeding.

### Task 0.2: Measure Test Coverage

```bash
cargo llvm-cov --html
open target/llvm-cov/html/index.html
```

**Action:** Document current coverage baseline. Target: >95% coverage.

### Task 0.3: Convert TODOs to GitHub Issues

The analysis found 8 TODO markers. Convert each to a tracked issue:

| Location | TODO Content | Suggested Issue Title |
|----------|--------------|----------------------|
| `src/compiler/cached.rs:121` | Track dependencies when needed | "Track compilation dependencies in CachedCompiler" |
| `src/compiler/cached.rs:122` | Add source map configuration | "Add SourceMap to cache key generation" |
| `src/compiler/error_translator.rs` | Integrate with oxur-smap | "Integrate SourceMap with ErrorTranslator" |
| `src/executor/subprocess.rs` | Protocol implementation Phase 2 | "Complete subprocess IPC protocol" |
| `src/server/session.rs` | Add session timeout cleanup | "Implement session timeout and cleanup" |
| `src/eval/context.rs` | Implement full eval pipeline | "Complete EvalContext evaluation pipeline" |

**Action:** Create issues, reference them in code comments, remove TODO markers.

### Task 0.4: Review Existing Tests

Verify all 242 tests still pass:

```bash
cargo test --all-features
```

**Action:** Ensure clean baseline before adding new code.

---

## 4. Phase 1: VariableStore + Subprocess Runtime

**Duration:** 1-2 weeks
**Priority:** CRITICAL
**Blocks:** All code execution functionality

### Context from ODD-0038

The subprocess maintains a `VariableStore` that holds type-erased values:

```rust
// From ODD-0038 §1.1
pub struct VariableStore {
    vars: HashMap<String, Box<dyn Any + 'static>>,
}
```

**Key Constraint:** All values must be owned (`'static`) - no inter-variable references.

### Task 1.1: Implement VariableStore

**File:** `src/subprocess/variable_store.rs`

```rust
use std::any::Any;
use std::collections::HashMap;

/// Type-erased variable storage for REPL session state
///
/// # Constraints
///
/// All stored values must be 'static (owned data, no references).
/// This aligns with Lisp semantics (immutable data structures).
pub struct VariableStore {
    vars: HashMap<String, Box<dyn Any + 'static>>,
}

impl VariableStore {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Store a value with the given name
    ///
    /// Replaces any existing value with the same name.
    pub fn set(&mut self, name: String, value: Box<dyn Any + 'static>) {
        self.vars.insert(name, value);
    }

    /// Get a reference to a stored value
    ///
    /// Returns None if name doesn't exist or type doesn't match.
    pub fn get<T: 'static>(&self, name: &str) -> Option<&T> {
        self.vars.get(name)?.downcast_ref::<T>()
    }

    /// Get a mutable reference to a stored value
    pub fn get_mut<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.vars.get_mut(name)?.downcast_mut::<T>()
    }

    /// Take ownership of a stored value, removing it from the store
    pub fn take<T: 'static>(&mut self, name: &str) -> Option<T> {
        let boxed = self.vars.remove(name)?;
        boxed.downcast::<T>().ok().map(|b| *b)
    }

    /// Check if a variable exists with the expected type
    pub fn check_type<T: 'static>(&self, name: &str) -> bool {
        self.vars.get(name)
            .map(|v| v.downcast_ref::<T>().is_some())
            .unwrap_or(true) // Missing is OK (will be created)
    }

    /// List all variable names
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.vars.keys()
    }

    /// Clear all variables (used on subprocess restart)
    pub fn clear(&mut self) {
        self.vars.clear();
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        Self::new()
    }
}
```

**Tests to add:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_i32() {
        let mut store = VariableStore::new();
        store.set("x".to_string(), Box::new(42i32));
        assert_eq!(store.get::<i32>("x"), Some(&42));
    }

    #[test]
    fn test_type_mismatch_returns_none() {
        let mut store = VariableStore::new();
        store.set("x".to_string(), Box::new(42i32));
        assert_eq!(store.get::<String>("x"), None);
    }

    #[test]
    fn test_take_removes_value() {
        let mut store = VariableStore::new();
        store.set("x".to_string(), Box::new("hello".to_string()));
        let value = store.take::<String>("x");
        assert_eq!(value, Some("hello".to_string()));
        assert!(store.get::<String>("x").is_none());
    }

    #[test]
    fn test_overwrite_existing() {
        let mut store = VariableStore::new();
        store.set("x".to_string(), Box::new(1i32));
        store.set("x".to_string(), Box::new(2i32));
        assert_eq!(store.get::<i32>("x"), Some(&2));
    }
}
```

### Task 1.2: Implement Subprocess Runtime

**File:** `src/bin/subprocess.rs`

This is the binary that runs in isolation and executes user code.

```rust
//! Oxur REPL Subprocess Runtime
//!
//! This binary runs in an isolated process and:
//! - Listens for commands on stdin
//! - Loads dynamic libraries via libloading
//! - Executes user code functions
//! - Maintains variable state in VariableStore
//! - Returns results via stdout
//!
//! # Protocol
//!
//! Commands (stdin):
//!   LOAD_AND_RUN <lib_path> <fn_name>
//!
//! Responses (stdout):
//!   OXUR_EXECUTION_COMPLETE
//!   OXUR_RUNTIME_ERROR: <message>
//!   OXUR_PANIC_LOCATION: <file>:<line>:<col>

use std::io::{self, BufRead, Write};
use std::panic;
use std::path::Path;

use libloading::{Library, Symbol};
use oxur_repl::subprocess::VariableStore;

/// Type signature for generated evaluation functions
///
/// The generated code wraps user code in this signature:
/// ```rust,ignore
/// #[no_mangle]
/// pub extern "C" fn run_user_code_N(
///     vars: &mut VariableStore
/// ) -> Box<dyn std::any::Any + 'static>
/// ```
type EvalFn = extern "C" fn(&mut VariableStore) -> Box<dyn std::any::Any + 'static>;

struct Runtime {
    variable_store: VariableStore,
    loaded_libraries: Vec<Library>, // Keep loaded to prevent unloading
}

impl Runtime {
    fn new() -> Self {
        Self {
            variable_store: VariableStore::new(),
            loaded_libraries: Vec::new(),
        }
    }

    fn load_and_run(&mut self, lib_path: &str, fn_name: &str) -> Result<(), String> {
        // Load the dynamic library
        let lib = unsafe {
            Library::new(lib_path)
                .map_err(|e| format!("Failed to load library: {}", e))?
        };

        // Look up the function
        let func: Symbol<EvalFn> = unsafe {
            lib.get(fn_name.as_bytes())
                .map_err(|e| format!("Failed to find function '{}': {}", fn_name, e))?
        };

        // Execute with panic catching
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            func(&mut self.variable_store)
        }));

        // Keep library loaded (functions may reference it later)
        self.loaded_libraries.push(lib);

        match result {
            Ok(_value) => {
                // TODO: Could serialize value back if needed
                Ok(())
            }
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                Err(msg)
            }
        }
    }
}

fn main() {
    let mut runtime = Runtime::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                break;
            }
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("LOAD_AND_RUN ") {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() != 3 {
                println!("OXUR_RUNTIME_ERROR: Invalid LOAD_AND_RUN command format");
                stdout.flush().unwrap();
                continue;
            }

            let lib_path = parts[1];
            let fn_name = parts[2];

            match runtime.load_and_run(lib_path, fn_name) {
                Ok(()) => {
                    println!("OXUR_EXECUTION_COMPLETE");
                }
                Err(msg) => {
                    println!("OXUR_RUNTIME_ERROR: {}", msg);
                }
            }
        } else {
            println!("OXUR_RUNTIME_ERROR: Unknown command: {}", line);
        }

        stdout.flush().unwrap();
    }
}
```

### Task 1.3: Complete SubprocessExecutor IPC

**File:** `src/executor/subprocess.rs`

Add the protocol implementation to the existing stub:

```rust
impl SubprocessExecutor {
    /// Execute a function from a compiled library
    ///
    /// Sends LOAD_AND_RUN command to subprocess and waits for response.
    pub fn execute(
        &mut self,
        lib_path: &Path,
        fn_name: &str,
    ) -> Result<ExecutionResult, ExecutorError> {
        // Ensure subprocess is running
        if !self.is_running() {
            self.restart()?;
        }

        // Send command
        let command = format!(
            "LOAD_AND_RUN {} {}\n",
            lib_path.display(),
            fn_name
        );

        self.stdin
            .as_mut()
            .ok_or(ExecutorError::SubprocessNotRunning)?
            .write_all(command.as_bytes())
            .map_err(|e| ExecutorError::IpcError(e.to_string()))?;

        self.stdin
            .as_mut()
            .unwrap()
            .flush()
            .map_err(|e| ExecutorError::IpcError(e.to_string()))?;

        // Read response
        let mut response = String::new();
        self.stdout
            .as_mut()
            .ok_or(ExecutorError::SubprocessNotRunning)?
            .read_line(&mut response)
            .map_err(|e| ExecutorError::IpcError(e.to_string()))?;

        let response = response.trim();

        // Parse response
        if response == "OXUR_EXECUTION_COMPLETE" {
            Ok(ExecutionResult::Success {
                output: String::new(), // TODO: Capture stdout separately
            })
        } else if response.starts_with("OXUR_RUNTIME_ERROR: ") {
            let msg = response
                .strip_prefix("OXUR_RUNTIME_ERROR: ")
                .unwrap_or("Unknown error");
            Ok(ExecutionResult::RuntimeError {
                message: msg.to_string(),
            })
        } else if response.starts_with("OXUR_PANIC_LOCATION: ") {
            let location = response
                .strip_prefix("OXUR_PANIC_LOCATION: ")
                .unwrap_or("unknown");
            Ok(ExecutionResult::Panic {
                location: location.to_string(),
                message: "Panic in user code".to_string(),
            })
        } else {
            Err(ExecutorError::ProtocolError(format!(
                "Unexpected response: {}",
                response
            )))
        }
    }

    /// Restart the subprocess (e.g., after crash or Ctrl-C)
    pub fn restart(&mut self) -> Result<(), ExecutorError> {
        // Kill existing if running
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Spawn new subprocess
        self.spawn()?;

        // Note: VariableStore state is lost on restart
        // This is acceptable - user can re-evaluate definitions

        Ok(())
    }

    fn is_running(&self) -> bool {
        self.child
            .as_ref()
            .map(|c| c.try_wait().ok().flatten().is_none())
            .unwrap_or(false)
    }
}

/// Result of executing user code in subprocess
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionResult {
    Success { output: String },
    RuntimeError { message: String },
    Panic { location: String, message: String },
}
```

### Task 1.4: Add Integration Tests

**File:** `tests/subprocess_execution.rs`

```rust
use oxur_repl::executor::{SubprocessExecutor, ExecutionResult};
use std::path::PathBuf;
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_subprocess_lifecycle() {
    let executor = SubprocessExecutor::new().expect("Failed to create executor");
    assert!(executor.is_running());
}

#[tokio::test]
async fn test_subprocess_restart() {
    let mut executor = SubprocessExecutor::new().expect("Failed to create executor");
    executor.restart().expect("Failed to restart");
    assert!(executor.is_running());
}

// Note: Full execution tests require compiled test libraries
// These will be added after RustAstWrapper is implemented
```

### Phase 1 Completion Criteria

- [ ] `VariableStore` implemented with full test coverage
- [ ] Subprocess binary compiles and runs
- [ ] `SubprocessExecutor::execute()` sends/receives protocol messages
- [ ] Subprocess can load a dynamic library (test with simple .so)
- [ ] Restart functionality works
- [ ] All existing tests still pass

---

## 5. Phase 2: RustAstWrapper

**Duration:** 2-3 weeks
**Priority:** CRITICAL
**Blocks:** All evaluation functionality

### Context from ODD-0038

RustAstWrapper takes already-lowered Rust AST from `oxur-comp::lower()` and wraps it with REPL scaffolding:

1. Variable loading from VariableStore
2. `extern "C"` wrapper function for dynamic loading
3. Variable storing back to VariableStore
4. Source map comments (NodeId annotations)

**Input:** Pure Rust AST from `oxur-comp::lower()`
**Output:** Complete library AST ready for compilation

### Task 2.1: Define the Interface

**File:** `src/wrapper.rs`

```rust
//! Rust AST Wrapper for REPL Scaffolding
//!
//! Takes pure Rust AST from oxur-comp and wraps it with:
//! - VariableStore integration (load/store variables)
//! - extern "C" wrapper function for dynamic loading
//! - Source map comments for error translation
//!
//! # Example Generated Code
//!
//! ```rust,ignore
//! // User's lowered code (from oxur-comp)
//! fn user_code_5() -> i32 {
//!     /* oxur_node=300 */ x + y
//! }
//!
//! // REPL scaffolding (added by RustAstWrapper)
//! #[no_mangle]
//! pub extern "C" fn run_user_code_5(
//!     vars: &mut VariableStore
//! ) -> Box<dyn Any + 'static> {
//!     // Load variables from store
//!     let x: i32 = vars.take("x").unwrap();
//!     let y: i32 = vars.take("y").unwrap();
//!
//!     // Execute user code
//!     let result = user_code_5();
//!
//!     // Store variables back (they may have been modified)
//!     vars.set("x".to_string(), Box::new(x));
//!     vars.set("y".to_string(), Box::new(y));
//!
//!     // Store and return result
//!     vars.set("_".to_string(), Box::new(result.clone()));
//!     Box::new(result)
//! }
//! ```

use oxur_smap::SourceMap;
use syn::File as SynFile;

use crate::session::SessionState;

/// Configuration for code wrapping
pub struct WrapperConfig {
    /// Counter for unique function names (run_user_code_N)
    pub eval_counter: u32,
    /// Variables to load from store (name -> type)
    pub variables_to_load: Vec<(String, String)>,
    /// Variables to store back
    pub variables_to_store: Vec<(String, String)>,
    /// Name of result variable (usually "_")
    pub result_var: String,
}

/// Wraps Rust AST with REPL scaffolding
pub struct RustAstWrapper {
    config: WrapperConfig,
}

impl RustAstWrapper {
    pub fn new(config: WrapperConfig) -> Self {
        Self { config }
    }

    /// Wrap a Rust AST with REPL scaffolding
    ///
    /// # Arguments
    ///
    /// * `rust_ast` - Pure Rust AST from oxur-comp::lower()
    /// * `source_map` - Source map for NodeId comment generation
    /// * `state` - Session state for variable tracking
    ///
    /// # Returns
    ///
    /// Complete Rust source code ready for compilation
    pub fn wrap(
        &self,
        rust_ast: &SynFile,
        source_map: &SourceMap,
        state: &SessionState,
    ) -> Result<String, WrapperError> {
        // Implementation in Task 2.2
        todo!("Implement AST wrapping")
    }

    /// Generate the wrapper function signature
    fn generate_wrapper_fn(&self) -> String {
        // Implementation in Task 2.2
        todo!()
    }

    /// Generate variable loading code
    fn generate_var_loads(&self) -> String {
        // Implementation in Task 2.2
        todo!()
    }

    /// Generate variable storing code
    fn generate_var_stores(&self) -> String {
        // Implementation in Task 2.2
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WrapperError {
    #[error("Failed to generate wrapper: {0}")]
    GenerationFailed(String),

    #[error("Type inference required for variable: {0}")]
    TypeInferenceNeeded(String),
}
```

### Task 2.2: Implement AST Wrapping

This is the core implementation using `syn` and `quote`:

```rust
use quote::quote;
use syn::{parse_quote, ItemFn};
use proc_macro2::TokenStream;

impl RustAstWrapper {
    pub fn wrap(
        &self,
        rust_ast: &SynFile,
        source_map: &SourceMap,
        state: &SessionState,
    ) -> Result<String, WrapperError> {
        let eval_num = self.config.eval_counter;
        let user_fn_name = format!("user_code_{}", eval_num);
        let wrapper_fn_name = format!("run_user_code_{}", eval_num);

        // Extract the user's code body from the AST
        let user_code = self.extract_user_code(rust_ast)?;

        // Generate variable loading statements
        let var_loads = self.generate_var_loads();

        // Generate variable storing statements
        let var_stores = self.generate_var_stores();

        // Build the complete library source
        let output = format!(
            r#"
// Generated by oxur-repl RustAstWrapper

use std::any::Any;
use std::collections::HashMap;

// VariableStore definition (must match subprocess runtime)
mod oxur_variable_store {{
    use std::any::Any;
    use std::collections::HashMap;

    pub struct VariableStore {{
        pub vars: HashMap<String, Box<dyn Any + 'static>>,
    }}

    impl VariableStore {{
        pub fn get<T: 'static>(&self, name: &str) -> Option<&T> {{
            self.vars.get(name)?.downcast_ref::<T>()
        }}

        pub fn take<T: 'static>(&mut self, name: &str) -> Option<T> {{
            let boxed = self.vars.remove(name)?;
            boxed.downcast::<T>().ok().map(|b| *b)
        }}

        pub fn set(&mut self, name: String, value: Box<dyn Any + 'static>) {{
            self.vars.insert(name, value);
        }}
    }}
}}

use oxur_variable_store::VariableStore;

// User's lowered code
{user_code}

// REPL wrapper function
#[no_mangle]
pub extern "C" fn {wrapper_fn_name}(
    vars: &mut VariableStore
) -> Box<dyn Any + 'static> {{
    // Load variables from store
{var_loads}

    // Execute user code
    let __oxur_result = {user_fn_name}();

    // Store variables back
{var_stores}

    // Store and return result
    vars.set("_".to_string(), Box::new(__oxur_result.clone()));
    Box::new(__oxur_result)
}}
"#,
            user_code = user_code,
            wrapper_fn_name = wrapper_fn_name,
            user_fn_name = user_fn_name,
            var_loads = var_loads,
            var_stores = var_stores,
        );

        Ok(output)
    }

    fn generate_var_loads(&self) -> String {
        self.config
            .variables_to_load
            .iter()
            .map(|(name, type_str)| {
                format!(
                    "    let {name}: {type_str} = vars.take(\"{name}\").expect(\"Variable '{name}' not found\");",
                    name = name,
                    type_str = type_str
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_var_stores(&self) -> String {
        self.config
            .variables_to_store
            .iter()
            .map(|(name, _type_str)| {
                format!(
                    "    vars.set(\"{name}\".to_string(), Box::new({name}));",
                    name = name
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn extract_user_code(&self, ast: &SynFile) -> Result<String, WrapperError> {
        // Use prettyplease or quote to convert AST back to string
        let tokens = quote! { #ast };
        Ok(tokens.to_string())
    }
}
```

### Task 2.3: Handle Source Map Comments

Add NodeId comments to generated code for error translation:

```rust
impl RustAstWrapper {
    /// Add source map comments to generated code
    ///
    /// Inserts /* oxur_node=N */ comments that can be parsed
    /// during error translation to map back to original source.
    fn add_source_comments(&self, code: &str, source_map: &SourceMap) -> String {
        // For now, pass through unchanged
        // Full implementation requires AST-level transformation
        // This is a v1.1 enhancement
        code.to_string()
    }
}
```

### Task 2.4: Add Wrapper Tests

**File:** `tests/wrapper_tests.rs`

```rust
use oxur_repl::wrapper::{RustAstWrapper, WrapperConfig};
use oxur_smap::SourceMap;

#[test]
fn test_simple_expression_wrapping() {
    let config = WrapperConfig {
        eval_counter: 1,
        variables_to_load: vec![],
        variables_to_store: vec![],
        result_var: "_".to_string(),
    };

    let wrapper = RustAstWrapper::new(config);

    // Create a simple AST for: fn user_code_1() -> i32 { 1 + 2 }
    let ast: syn::File = syn::parse_quote! {
        fn user_code_1() -> i32 {
            1 + 2
        }
    };

    let source_map = SourceMap::new();
    let state = Default::default();

    let result = wrapper.wrap(&ast, &source_map, &state).unwrap();

    assert!(result.contains("run_user_code_1"));
    assert!(result.contains("extern \"C\""));
    assert!(result.contains("#[no_mangle]"));
}

#[test]
fn test_variable_loading() {
    let config = WrapperConfig {
        eval_counter: 2,
        variables_to_load: vec![
            ("x".to_string(), "i32".to_string()),
            ("y".to_string(), "i32".to_string()),
        ],
        variables_to_store: vec![
            ("x".to_string(), "i32".to_string()),
            ("y".to_string(), "i32".to_string()),
        ],
        result_var: "_".to_string(),
    };

    let wrapper = RustAstWrapper::new(config);

    let ast: syn::File = syn::parse_quote! {
        fn user_code_2() -> i32 {
            x + y
        }
    };

    let source_map = SourceMap::new();
    let state = Default::default();

    let result = wrapper.wrap(&ast, &source_map, &state).unwrap();

    assert!(result.contains("vars.take(\"x\")"));
    assert!(result.contains("vars.take(\"y\")"));
    assert!(result.contains("vars.set(\"x\""));
    assert!(result.contains("vars.set(\"y\""));
}
```

### Phase 2 Completion Criteria

- [ ] `RustAstWrapper` generates valid Rust code
- [ ] Generated code compiles with `cargo build`
- [ ] Variable loading/storing works correctly
- [ ] Wrapper function has correct signature for `libloading`
- [ ] Integration with `CachedCompiler` works
- [ ] All tests pass

---

## 6. Phase 3: EvalContext + Integration

**Duration:** 1-2 weeks
**Priority:** CRITICAL
**Blocks:** User-facing REPL functionality

### Context from ODD-0038

`EvalContext` orchestrates the full evaluation pipeline:

1. Parse user input (via `oxur-lang`)
2. Decide execution tier (Calculator/Cached/JIT)
3. Check cache for existing artifact
4. If cache miss: compile via `CachedCompiler`
5. Execute via `SubprocessExecutor`
6. Return result to user

### Task 3.1: Complete EvalContext

**File:** `src/eval/context.rs`

```rust
use oxur_lang::{parse_lisp, expand, CoreForm};
use oxur_smap::SourceMap;

use crate::cache::ArtifactCache;
use crate::compiler::CachedCompiler;
use crate::executor::{SubprocessExecutor, ExecutionResult};
use crate::session::{SessionDir, SessionState};
use crate::wrapper::{RustAstWrapper, WrapperConfig};

pub struct EvalContext {
    session_id: String,
    mode: ReplMode,
    compiler: CachedCompiler,
    executor: SubprocessExecutor,
    cache: Arc<ArtifactCache>,
    state: SessionState,
    history: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplMode {
    Lisp,   // Lisp-1 surface syntax
    Sexpr,  // Core form S-expressions
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub input: String,
    pub result: Result<String, String>,
    pub tier: ExecutionTier,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionTier {
    Calculator, // Direct Rust evaluation, <1ms
    Cached,     // Cache hit, 1-5ms
    Jit,        // Full compilation, 50-300ms
}

impl EvalContext {
    pub fn new(
        session_id: String,
        session_dir: Arc<SessionDir>,
        cache: Arc<ArtifactCache>,
    ) -> Result<Self, EvalError> {
        let compiler = CachedCompiler::new(cache.clone(), session_dir.clone());
        let executor = SubprocessExecutor::new()?;

        Ok(Self {
            session_id,
            mode: ReplMode::Lisp,
            compiler,
            executor,
            cache,
            state: SessionState::new(),
            history: Vec::new(),
        })
    }

    /// Evaluate user code
    ///
    /// This is the main entry point for REPL evaluation.
    pub fn eval(&mut self, code: &str) -> Result<EvalResult, EvalError> {
        let start = std::time::Instant::now();

        // Create source map for this evaluation
        let mut source_map = SourceMap::new();

        // Stage 1: Parse
        let surface = parse_lisp(code, &mut source_map)
            .map_err(|e| EvalError::Parse(e.to_string()))?;

        // Stage 2: Expand
        let core = expand(surface, &mut source_map)
            .map_err(|e| EvalError::Expand(e.to_string()))?;

        // Stage 3: Decide execution tier
        let tier = self.decide_tier(&core, code);

        let result = match tier {
            ExecutionTier::Calculator => {
                self.eval_calculator(&core)?
            }
            ExecutionTier::Cached | ExecutionTier::Jit => {
                self.eval_compiled(&core, source_map, tier == ExecutionTier::Cached)?
            }
        };

        // Record history
        let duration = start.elapsed().as_millis() as u64;
        self.history.push(HistoryEntry {
            input: code.to_string(),
            result: Ok(result.display.clone()),
            tier,
            duration_ms: duration,
        });

        Ok(result)
    }

    fn decide_tier(&self, core: &CoreForm, code: &str) -> ExecutionTier {
        // Tier 1: Simple arithmetic
        if self.is_simple_arithmetic(core) {
            return ExecutionTier::Calculator;
        }

        // Tier 2: Check cache
        let cache_key = self.compute_cache_key(code);
        if self.cache.get(&cache_key).is_some() {
            return ExecutionTier::Cached;
        }

        // Tier 3: Must compile
        ExecutionTier::Jit
    }

    fn is_simple_arithmetic(&self, core: &CoreForm) -> bool {
        // Simple heuristic: only literals and basic ops
        match core {
            CoreForm::Literal(_) => true,
            CoreForm::FunctionCall { function, args, .. } => {
                matches!(function.as_str(), "+" | "-" | "*" | "/" | "%")
                    && args.len() <= 3
                    && args.iter().all(|a| self.is_simple_arithmetic(a))
            }
            _ => false,
        }
    }

    fn eval_calculator(&self, core: &CoreForm) -> Result<EvalResult, EvalError> {
        // Direct Rust evaluation for simple arithmetic
        let value = self.evaluate_arithmetic(core)?;
        Ok(EvalResult {
            value: Some(value.clone()),
            display: format!("{}", value),
            tier: ExecutionTier::Calculator,
        })
    }

    fn evaluate_arithmetic(&self, core: &CoreForm) -> Result<Value, EvalError> {
        // Implement simple arithmetic evaluation
        // This is a fast path that avoids compilation
        todo!("Implement calculator mode")
    }

    fn eval_compiled(
        &mut self,
        core: &CoreForm,
        source_map: SourceMap,
        is_cached: bool,
    ) -> Result<EvalResult, EvalError> {
        // Increment eval counter
        self.state.eval_counter += 1;
        let eval_num = self.state.eval_counter;

        // Lower to Rust AST
        let rust_ast = oxur_comp::lower(core, &mut source_map.clone())
            .map_err(|e| EvalError::Lower(e.to_string()))?;

        // Create wrapper config
        let config = WrapperConfig {
            eval_counter: eval_num,
            variables_to_load: self.state.get_variables_with_types(),
            variables_to_store: self.state.get_variables_with_types(),
            result_var: "_".to_string(),
        };

        // Wrap with REPL scaffolding
        let wrapper = RustAstWrapper::new(config);
        let source = wrapper.wrap(&rust_ast, &source_map, &self.state)?;

        // Compile
        let cache_key = format!("eval_{}", eval_num);
        let artifact_path = self.compiler.compile(&cache_key, &source, 0)?;

        // Execute
        let fn_name = format!("run_user_code_{}", eval_num);
        let exec_result = self.executor.execute(&artifact_path, &fn_name)?;

        match exec_result {
            ExecutionResult::Success { output } => {
                Ok(EvalResult {
                    value: None, // TODO: Deserialize result
                    display: output,
                    tier: if is_cached {
                        ExecutionTier::Cached
                    } else {
                        ExecutionTier::Jit
                    },
                })
            }
            ExecutionResult::RuntimeError { message } => {
                Err(EvalError::Runtime(message))
            }
            ExecutionResult::Panic { message, .. } => {
                // Restart subprocess after panic
                self.executor.restart()?;
                Err(EvalError::Panic(message))
            }
        }
    }

    fn compute_cache_key(&self, code: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn set_mode(&mut self, mode: ReplMode) {
        self.mode = mode;
    }

    pub fn history(&self) -> &[HistoryEntry] {
        &self.history
    }
}

#[derive(Debug, Clone)]
pub struct EvalResult {
    pub value: Option<Value>,
    pub display: String,
    pub tier: ExecutionTier,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    // Add more as needed
}

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Expansion error: {0}")]
    Expand(String),

    #[error("Lowering error: {0}")]
    Lower(String),

    #[error("Compilation error: {0}")]
    Compile(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Panic in user code: {0}")]
    Panic(String),

    #[error("Executor error: {0}")]
    Executor(#[from] crate::executor::ExecutorError),

    #[error("Wrapper error: {0}")]
    Wrapper(#[from] crate::wrapper::WrapperError),
}
```

### Task 3.2: Integrate with MessageHandler

Update `src/server/handler.rs` to use `EvalContext`:

```rust
impl MessageHandler {
    async fn handle_eval(&self, request: &Request) -> Response {
        let session_id = &request.session_id;

        // Get or create session
        let context = self.session_manager.get_or_create(session_id)?;

        // Lock and evaluate
        let mut context = context.lock().await;

        match context.eval(&request.code) {
            Ok(result) => Response {
                id: request.id.clone(),
                session: session_id.clone(),
                value: result.display,
                status: vec![Status::Done],
                error: None,
            },
            Err(e) => Response {
                id: request.id.clone(),
                session: session_id.clone(),
                value: String::new(),
                status: vec![Status::Error],
                error: Some(e.to_string()),
            },
        }
    }
}
```

### Phase 3 Completion Criteria

- [ ] `EvalContext::eval()` works end-to-end
- [ ] Simple expressions like `(+ 1 2)` evaluate correctly
- [ ] Variable definitions work: `(def x 42)`
- [ ] Variable references work: `(+ x 10)`
- [ ] History tracking works
- [ ] Tier selection works correctly
- [ ] Integration with MessageHandler complete
- [ ] All tests pass

---

## 7. Phase 4: SourceMap Integration

**Duration:** 1-2 weeks
**Priority:** HIGH
**Purpose:** Rustc-quality error messages

### Context from ODD-0038

The source map tracks transformations across:

1. **Surface Forms** (original Oxur source) → positions recorded
2. **Core Forms** (after macro expansion) → surface→core mapping
3. **Rust AST** (after lowering) → core→rust mapping

This enables error translation: `rustc error → Rust position → Core position → Surface position → original Oxur source`

### Task 4.1: Thread SourceMap Through Pipeline

Update `oxur-lang` and `oxur-comp` to accept and populate `SourceMap`:

```rust
// In oxur-lang (already has this API per spec)
pub fn parse_lisp(
    source: &str,
    source_map: &mut SourceMap,
) -> Result<SurfaceForms, ParseError>;

pub fn expand(
    surface: SurfaceForms,
    source_map: &mut SourceMap,
) -> Result<CoreForms, ExpandError>;

// In oxur-comp (already has this API per spec)
pub fn lower(
    core: &CoreForm,
    source_map: &mut SourceMap,
) -> Result<syn::File, LowerError>;
```

### Task 4.2: Update ErrorTranslator

**File:** `src/compiler/error_translator.rs`

```rust
use oxur_smap::{SourceMap, SourcePos, NodeId};

pub struct ErrorTranslator {
    source_map: SourceMap,
}

impl ErrorTranslator {
    pub fn new(source_map: SourceMap) -> Self {
        Self { source_map }
    }

    /// Translate a rustc error to original Oxur position
    pub fn translate(&self, rustc_error: &RustcDiagnostic) -> TranslatedError {
        // 1. Parse rustc span
        let rustc_pos = &rustc_error.spans[0];

        // 2. Read source line to find NodeId comment
        let node_id = self.extract_node_id_at_position(
            &rustc_pos.file_name,
            rustc_pos.line_start,
            rustc_pos.column_start,
        );

        // 3. Look up original position via SourceMap
        let original_pos = node_id.and_then(|id| self.source_map.lookup(id));

        TranslatedError {
            message: rustc_error.message.clone(),
            code: rustc_error.code.clone(),
            level: rustc_error.level.clone(),
            position: original_pos.unwrap_or(SourcePos {
                file: "<generated>".to_string(),
                line: 0,
                column: 0,
                length: 0,
            }),
        }
    }

    fn extract_node_id_at_position(
        &self,
        file: &str,
        line: u32,
        column: u32,
    ) -> Option<NodeId> {
        // Read the source line
        let source = std::fs::read_to_string(file).ok()?;
        let line_content = source.lines().nth((line - 1) as usize)?;

        // Find /* oxur_node=N */ comment near column
        let pattern = regex::Regex::new(r"/\* oxur_node=(\d+) \*/").ok()?;

        let mut best_match: Option<(usize, NodeId)> = None;
        for cap in pattern.captures_iter(line_content) {
            let match_start = cap.get(0)?.start();
            let node_id: u32 = cap.get(1)?.as_str().parse().ok()?;

            let distance = (match_start as i32 - column as i32).abs() as usize;

            if best_match.is_none() || distance < best_match.unwrap().0 {
                best_match = Some((distance, NodeId::from(node_id)));
            }
        }

        best_match.map(|(_, id)| id)
    }
}

#[derive(Debug, Clone)]
pub struct TranslatedError {
    pub message: String,
    pub code: Option<String>,
    pub level: String,
    pub position: SourcePos,
}
```

### Task 4.3: Pretty Error Display with ariadne

```rust
use ariadne::{Report, ReportKind, Label, Source};

pub fn display_error(error: &TranslatedError, source: &str) {
    let report = Report::build(
        ReportKind::Error,
        &error.position.file,
        error.position.column as usize,
    )
    .with_code(error.code.as_deref().unwrap_or("E????"))
    .with_message(&error.message)
    .with_label(
        Label::new((
            &error.position.file,
            error.position.column as usize
                ..(error.position.column + error.position.length) as usize,
        ))
        .with_message(&error.message),
    );

    report
        .finish()
        .print(Source::from(source))
        .expect("Failed to print error");
}
```

### Phase 4 Completion Criteria

- [ ] SourceMap correctly populated during parsing
- [ ] SourceMap correctly populated during expansion
- [ ] SourceMap correctly populated during lowering
- [ ] Error translation finds correct original position
- [ ] Error messages display with correct file/line/column
- [ ] ariadne produces beautiful error output
- [ ] All tests pass

---

## 8. Phase 5: Testing & Polish

**Duration:** 1-2 weeks
**Priority:** HIGH

### Task 5.1: Create Test Data Directory

Create test data for the Oxur language like we did for the Oxur AST. Here's a partial listing of what we did for the AST:

```
$ tree -ar crates/oxur-ast/test-data/
crates/oxur-ast/test-data/
├── README.md
├── fixtures
│   ├── stmt
│   │   ├── wrong-node-type.sexp
...
│   │   └── complex-expression.sexp
│   ├── path
│   │   ├── wrong-node-type.sexp
...
│   │   └── empty-segments.sexp
│   ├── item
│   │   ├── wrong-node-type.sexp
...
│   │   └── const-function.sexp
│   ├── fn
│   │   ├── with-nil-body.sexp
...
│   │   └── complete-function-item.sexp
│   ├── expr
│   │   ├── wrong-node-type.sexp
...
│   │   └── block-expr.sexp
│   ├── crate
│   │   ├── wrong-node-type.sexp
...
│   │   └── complex-nested.sexp
│   └── block
│       ├── wrong-node-type.sexp
...
│       └── empty-block.sexp
├── examples
│   ├── simple
│   │   ├── symbol.sexp
...
│   │   └── empty-crate.sexp
│   ├── intermediate
│   │   ├── visibility-variants.sexp
...
│   │   └── macro-call.sexp
│   └── complex
│       ├── multi-stmt-function.sexp
...
│       └── all-node-types.sexp
└── error-cases
    ├── unterminated-string.sexp
...
    └── invalid-escape.sexp

14 directories, 119 files
```

Even though the work we are doing in this plan is for the REPL, the test data we need is for the language, so you will need to create test data files in the following manner:

```
crates/oxur-lang/test-data/
├── README.md
├── fixtures (by form ... but you might not need this for the current work)
│   ├── deffn
...
│   ├── let
...
├── examples/ (examples by level of complexity)
│   ├── simple/
│   │   ├── arithmetic.oxur            # (* 2 (+ 1 2 3 4 5 6))
│   │   ├── repl-variable.oxur         # (def answer:i32 42)
│   │   ├── repl-variable-mutable.oxur # (def mut counter:i64 0)
│   │   └── function.oxur              # (deffn add (a:i32 b:i32) (:> i32) (+ a b))
│   ├── intermediate/
│   │   ├── recursion.oxur
│   │   └── closures.oxur
│   └── complex/
│       └── macros.oxur
├── edge-cases/
│   ├── empty.oxur
│   └── unicode.oxur
└── error-cases/
    ├── syntax_error.oxur
    ├── type_error.oxur
    └── runtime_error.oxur

```

You will also need to implement or adapt utility functions what what were done to read the test data in oxur-ast tests. If the testing infra code is reusable, the you should move the code that reads the test data out of the oxur-ast crate and into a new crate: oxur-testing. Then:

- generalise the functions for use by any crate
- update the use of these functions in oxur-ast
- use these functions in oxur-repl tests

### Task 5.2: End-to-End Tests

**File:** `tests/e2e_tests.rs`

```rust
#[tokio::test]
async fn test_simple_arithmetic() {
    let mut repl = start_test_repl().await;
    file_data = ... ; # read arithmetic.oxur file
    let result = repl.eval(file_data).await;
    assert_eq!(result.display, "42");
    assert_eq!(result.tier, ExecutionTier::Calculator);
}

#[tokio::test]
async fn test_repl_variable_definition() {
    let mut repl = start_test_repl().await;
    file_data = ... ; # read repl-variable.oxur file
    repl.eval(file_data).await;
    let result = repl.eval("answer").await;
    assert_eq!(result.display, "42");
}

#[tokio::test]
async fn test_repl_variable_definition() {
    let mut repl = start_test_repl().await;
    file_data = ... ; # read repl-variable-mutable.oxur file
    repl.eval(file_data).await;
    let result1 = repl.eval("counter").await;
    assert_eq!(result1.display, "0");
    let result2 = repl.eval("(set! counter (+ counter 1))").await;
    assert_eq!(result2.display, "1");
    let result3 = repl.eval("(set! counter (+ counter 1))").await;
    assert_eq!(result3.display, "2");
}

#[tokio::test]
async fn test_function_definition_and_call() {
    let mut repl = start_test_repl().await;
    file_data = ... ; # read function.oxur file
    repl.eval(file_data).await;
    let result = repl.eval("(add 29 13)").await;
    assert_eq!(result.display, "42");
}

#[tokio::test]
async fn test_cache_hit() {
    let mut repl = start_test_repl().await;
    file_data = ... ; # read function.oxur file
    repl.eval(file_data).await;

    // First call: JIT
    let result1 = repl.eval("(add 25 17)").await;
    assert_eq!(result1.tier, ExecutionTier::Jit);

    // Second call with same code: Cached
    let result2 = repl.eval("(add 25 17)").await;
    // Note: actual tier depends on cache implementation
}

#[tokio::test]
async fn test_error_with_position() {
    let mut repl = start_test_repl().await;
    let result = repl.eval("(+ x y)").await; // x, y undefined

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("cannot find"));
    // TODO: Check position points to original source
}
```

### Task 5.3: Performance Benchmarks

**File:** `benches/repl_benchmarks.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_calculator_tier(c: &mut Criterion) {
    let mut repl = setup_repl();

    // Warm up cache
    file_data = ... ; # read arithmetic.oxur file
    let result = repl.eval(file_data).await;

    c.bench_function("calculator: {}", |b| { // Fix this so that it provides the string "calculator: " + file_data
        b.iter(|| repl.eval(file_data))
    });
}

fn bench_cached_tier(c: &mut Criterion) {
    let mut repl = setup_repl();

    // Warm up cache
    file_data = ... ; # read function.oxur file
    repl.eval(file_data).await;

    c.bench_function("cached: {}", |b| { // Same fix as mentioned above
        b.iter(|| repl.eval(file_data))
    });
}

fn bench_jit_tier(c: &mut Criterion) {
    let mut repl = setup_repl();
    let mut counter = 0;

    c.bench_function("jit: new function", |b| {
        b.iter(|| {
            counter += 1;
            repl.eval(&format!(new_function_data_from_file, counter))
        })
    });
}

criterion_group!(benches, bench_calculator_tier, bench_cached_tier, bench_jit_tier);
criterion_main!(benches);
```

### Task 5.4: Documentation

Update all doc comments and create user documentation:

1. **API Documentation** - Ensure all public items have doc comments
2. **Architecture Documentation** - Update ODD-0038 with implementation decisions
3. **User Guide** - Basic usage examples
4. **README.md** - Quick start guide

### Phase 5 Completion Criteria

- [ ] Test data directory created
- [ ] End-to-end tests pass
- [ ] Performance benchmarks meet targets:
  - Calculator: <1ms
  - Cached: 1-5ms
  - JIT: 50-300ms
- [ ] Documentation complete
- [ ] All 242+ tests pass
- [ ] Coverage >95%
- [ ] Zero clippy warnings

---

## 9. Phase 6: CLI Integration

**Duration:** 1-2 weeks
**Priority:** HIGH
**Depends on:** Phase 3 (working EvalContext)

### Context from ODD-0038

See **Section 1.5: CLI Integration** for the complete architectural specification of the CLI interface.

The CLI provides user-facing access to the REPL through the unified `oxur` binary. When `oxur` is invoked without a subcommand, it defaults to `oxur repl`.

### Task 6.1: Update oxur-cli Repl Command

**File:** `oxur-cli/src/main.rs`

Replace the stub `Commands::Repl` with full flag support:

```rust
use clap::{Parser, Subcommand, Args};

#[derive(Subcommand)]
enum Commands {
    // ... other commands ...

    /// Start the interactive REPL
    Repl(ReplArgs),
}

#[derive(Args)]
struct ReplArgs {
    /// Start the default built-in REPL server and connect to it
    /// with the built-in client. This is the default behavior.
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// Connect to a running REPL server with the built-in client.
    /// Default: 127.0.0.1:5099
    #[arg(short = 'c', long = "connect", value_name = "HOST:PORT")]
    connect: Option<Option<String>>,

    /// Disable ANSI colors in interactive or connect modes.
    #[arg(long = "no-color")]
    no_color: bool,

    /// Start a REPL server only. PATH for Unix socket, HOST:PORT for TCP.
    #[arg(short = 's', long = "serve", value_name = "PATH|HOST:PORT")]
    serve: Option<String>,

    /// Acknowledge port to another nREPL server on ACK-PORT.
    #[arg(long = "ack", value_name = "ACK-PORT")]
    ack: Option<u16>,

    /// The transport module to use. Default: oxur_repl::transport::tcp
    #[arg(short = 't', long = "transport", value_name = "TRANSPORT")]
    transport: Option<String>,
}
```

**Implementation:**

```rust
Commands::Repl(args) => {
    let color_enabled = !args.no_color;

    if let Some(addr) = args.serve {
        // Server mode: start ReplServer and listen
        run_server_mode(&addr, args.ack, color_enabled).await?;
    } else if let Some(addr) = args.connect {
        // Connect mode: connect to existing server
        let addr = addr.unwrap_or_else(|| "127.0.0.1:5099".to_string());
        run_connect_mode(&addr, color_enabled).await?;
    } else {
        // Interactive mode (default): in-memory server + client
        run_interactive_mode(color_enabled).await?;
    }
}
```

### Task 6.2: Implement InProcessTransport

**File:** `oxur-repl/src/transport/inprocess.rs`

Zero-overhead transport using Tokio channels for in-memory client-server communication:

```rust
use tokio::sync::mpsc;
use crate::protocol::{Request, Response};

/// In-process transport for fastest possible REPL startup.
///
/// Uses unbounded channels - no serialization, no network overhead.
pub struct InProcessTransport {
    client_tx: mpsc::UnboundedSender<Request>,
    client_rx: mpsc::UnboundedReceiver<Response>,
    server_tx: mpsc::UnboundedSender<Response>,
    server_rx: mpsc::UnboundedReceiver<Request>,
}

impl InProcessTransport {
    /// Create a new in-process transport pair (client side, server side).
    pub fn new() -> (InProcessClient, InProcessServer) {
        let (client_to_server_tx, client_to_server_rx) = mpsc::unbounded_channel();
        let (server_to_client_tx, server_to_client_rx) = mpsc::unbounded_channel();

        let client = InProcessClient {
            tx: client_to_server_tx,
            rx: server_to_client_rx,
        };

        let server = InProcessServer {
            tx: server_to_client_tx,
            rx: client_to_server_rx,
        };

        (client, server)
    }
}

pub struct InProcessClient {
    tx: mpsc::UnboundedSender<Request>,
    rx: mpsc::UnboundedReceiver<Response>,
}

pub struct InProcessServer {
    tx: mpsc::UnboundedSender<Response>,
    rx: mpsc::UnboundedReceiver<Request>,
}
```

### Task 6.3: Implement Terminal Interface

**File:** `oxur-cli/src/repl/terminal.rs`

Interactive terminal with line editing and history:

```rust
use rustyline::{Editor, Config, EditMode};
use rustyline::error::ReadlineError;

pub struct ReplTerminal {
    editor: Editor<()>,
    history_path: PathBuf,
    color_enabled: bool,
}

impl ReplTerminal {
    pub fn new(color_enabled: bool) -> Result<Self> {
        let config = Config::builder()
            .edit_mode(EditMode::Emacs)
            .auto_add_history(true)
            .build();

        let mut editor = Editor::<()>::with_config(config)?;

        let history_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("oxur")
            .join("repl_history");

        if history_path.exists() {
            let _ = editor.load_history(&history_path);
        }

        Ok(Self { editor, history_path, color_enabled })
    }

    pub fn read_line(&mut self, prompt: &str) -> Result<Option<String>, ReadlineError> {
        match self.editor.readline(prompt) {
            Ok(line) => Ok(Some(line)),
            Err(ReadlineError::Interrupted) => Ok(None),  // Ctrl-C
            Err(ReadlineError::Eof) => Err(ReadlineError::Eof),  // Ctrl-D
            Err(e) => Err(e),
        }
    }

    pub fn save_history(&mut self) -> Result<()> {
        std::fs::create_dir_all(self.history_path.parent().unwrap())?;
        self.editor.save_history(&self.history_path)?;
        Ok(())
    }
}
```

### Task 6.4: Implement Interactive Mode

**File:** `oxur-cli/src/repl/interactive.rs`

The main REPL loop for interactive mode:

```rust
pub async fn run_interactive_mode(color_enabled: bool) -> Result<()> {
    // 1. Create in-process transport
    let (client_transport, server_transport) = InProcessTransport::new();

    // 2. Start server in background task
    let server = ReplServer::new(server_transport)?;
    let server_handle = tokio::spawn(async move {
        server.serve().await
    });

    // 3. Create client and establish session
    let mut client = ReplClient::new(client_transport);
    let session_id = client.clone_session().await?;

    // 4. Create terminal interface
    let mut terminal = ReplTerminal::new(color_enabled)?;

    // 5. Print welcome banner
    println!("Oxur REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type (help) for assistance, Ctrl-D to exit.\n");

    // 6. Main REPL loop
    loop {
        let prompt = if color_enabled { "\x1b[32moxur>\x1b[0m " } else { "oxur> " };

        match terminal.read_line(prompt) {
            Ok(Some(line)) => {
                if line.trim().is_empty() {
                    continue;
                }

                match client.eval(&line).await {
                    Ok(result) => {
                        if !result.stdout.is_empty() {
                            print!("{}", result.stdout);
                        }
                        if let Some(value) = result.value {
                            println!("{}", value);
                        }
                        if !result.stderr.is_empty() {
                            eprintln!("{}", result.stderr);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Ok(None) => {
                // Ctrl-C - interrupt current evaluation
                let _ = client.interrupt().await;
                println!();
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D - exit
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    // 7. Cleanup
    terminal.save_history()?;
    client.close().await?;
    server_handle.abort();

    Ok(())
}
```

### Task 6.5: Implement Server and Connect Modes

**File:** `oxur-cli/src/repl/server.rs`

Server mode implementation:

```rust
pub async fn run_server_mode(
    addr: &str,
    ack_port: Option<u16>,
    _color_enabled: bool,
) -> Result<()> {
    // Parse address to determine transport type
    let is_unix_socket = addr.starts_with('/') || addr.starts_with("./");

    let server = if is_unix_socket {
        // Unix domain socket
        let transport = UnixTransport::bind(PathBuf::from(addr)).await?;
        eprintln!("REPL server listening on {}", addr);
        ReplServer::new(transport)?
    } else {
        // TCP
        let socket_addr: SocketAddr = addr.parse()?;
        let transport = TcpTransport::bind(socket_addr).await?;
        let actual_addr = transport.local_addr()?;
        eprintln!("REPL server listening on {}", actual_addr);

        // ACK protocol if requested
        if let Some(ack_port) = ack_port {
            ack_port_to_server(ack_port, &actual_addr).await?;
        }

        ReplServer::new(transport)?
    };

    server.serve().await?;
    Ok(())
}
```

**File:** `oxur-cli/src/repl/connect.rs`

Connect mode implementation:

```rust
pub async fn run_connect_mode(addr: &str, color_enabled: bool) -> Result<()> {
    // Connect to existing server
    let transport = TcpTransport::connect(addr).await?;
    let mut client = ReplClient::new(transport);
    let _session_id = client.clone_session().await?;

    // Create terminal and run same loop as interactive mode
    let mut terminal = ReplTerminal::new(color_enabled)?;

    println!("Connected to REPL server at {}", addr);

    // ... same REPL loop as interactive mode ...

    Ok(())
}
```

### Task 6.6: Add Dependencies to oxur-cli

**File:** `oxur-cli/Cargo.toml`

```toml
[dependencies]
# ... existing deps ...

# For REPL terminal
rustyline = "14.0"
dirs = "5.0"

# For async runtime
tokio = { version = "1.0", features = ["full"] }
```

### Task 6.7: Default Command Behavior

**File:** `oxur-cli/src/main.rs`

Make `oxur` default to `oxur repl`:

```rust
#[derive(Parser)]
#[command(name = "oxur")]
#[command(about = "Oxur - A Lisp that compiles to Rust", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,  // Make optional
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => handle_command(cmd),
        None => {
            // Default to interactive REPL
            handle_command(Commands::Repl(ReplArgs::default()))
        }
    }
}
```

### Phase 6 Completion Criteria

- [ ] `oxur` (no args) starts interactive REPL
- [ ] `oxur repl` starts interactive REPL
- [ ] `oxur repl -i` starts interactive REPL (explicit)
- [ ] `oxur repl -s 127.0.0.1:5099` starts server only
- [ ] `oxur repl -s /tmp/oxur.sock` starts server on Unix socket
- [ ] `oxur repl -c` connects to 127.0.0.1:5099
- [ ] `oxur repl -c localhost:5099` connects to specified address
- [ ] `oxur repl --no-color` disables colors
- [ ] `oxur repl -s 0.0.0.0:0 --ack 5099` ACK protocol works
- [ ] Ctrl-C interrupts evaluation
- [ ] Ctrl-D exits REPL
- [ ] Command history persists across sessions
- [ ] Line editing works (arrow keys, home/end, etc.)

---

## 10. Summary: Implementation Order

```
Phase 0: Quality Tasks (1-2 days)
    ├── Run linting
    ├── Measure coverage
    ├── Convert TODOs to issues
    └── Verify baseline

Phase 1: VariableStore + Subprocess (1-2 weeks)
    ├── Task 1.1: VariableStore
    ├── Task 1.2: Subprocess Runtime
    ├── Task 1.3: SubprocessExecutor IPC
    └── Task 1.4: Integration tests

Phase 2: RustAstWrapper (2-3 weeks)
    ├── Task 2.1: Interface definition
    ├── Task 2.2: AST wrapping implementation
    ├── Task 2.3: Source map comments
    └── Task 2.4: Wrapper tests

Phase 3: EvalContext (1-2 weeks)
    ├── Task 3.1: Complete EvalContext
    └── Task 3.2: MessageHandler integration

Phase 4: SourceMap Integration (1-2 weeks)
    ├── Task 4.1: Thread SourceMap
    ├── Task 4.2: ErrorTranslator
    └── Task 4.3: ariadne display

Phase 5: Testing & Polish (1-2 weeks)
    ├── Task 5.1: Test data
    ├── Task 5.2: E2E tests
    ├── Task 5.3: Benchmarks
    └── Task 5.4: Documentation

Phase 6: CLI Integration (1-2 weeks)
    ├── Task 6.1: Update oxur-cli Repl command
    ├── Task 6.2: Implement InProcessTransport
    ├── Task 6.3: Implement Terminal Interface
    ├── Task 6.4: Implement Interactive Mode
    ├── Task 6.5: Implement Server/Connect Modes
    ├── Task 6.6: Add dependencies
    └── Task 6.7: Default command behavior
```

---

## 11. Success Criteria

### Minimum Viable REPL (End of Phase 3)

- [ ] Can evaluate `(+ 1 2)` → `3`
- [ ] Can define variables: `(def x:i32 42)`
- [ ] Can reference variables: `(+ x 10)` → `52`
- [ ] Can define functions: `(deffn square (x:i32) (:> i32) (* x x))`
- [ ] Can call functions: `(square 5)` → `25`
- [ ] Errors show helpful messages
- [ ] Ctrl-C interrupts long-running code

### Production Quality (End of Phase 5)

- [ ] Error messages point to exact Oxur source location
- [ ] Test coverage >95%
- [ ] REPL response time <100ms for cached code
- [ ] <1ms for calculator tier
- [ ] 1-5ms for cached tier
- [ ] Documentation complete
- [ ] Zero clippy warnings

### User-Ready Release (End of Phase 6)

- [ ] `oxur` command starts interactive REPL by default
- [ ] Server mode (`-s`) works with TCP and Unix sockets
- [ ] Connect mode (`-c`) works with running servers
- [ ] ACK protocol enables editor integration
- [ ] Command history persists across sessions
- [ ] Line editing provides familiar UX
- [ ] Ctrl-C/Ctrl-D behavior matches user expectations
- [ ] `--no-color` works for accessibility/scripting

---

## 12. Notes for Claude Code

1. **Start with Phase 0** - Don't skip the quality tasks. They establish a clean baseline.

2. **Follow the order** - Each phase builds on the previous. Don't jump ahead.

3. **Run tests frequently** - After each task, run `cargo test` to ensure nothing broke.

4. **Ask questions** - If something in the spec is unclear, ask before implementing.

5. **Reference the spec** - ODD-0038 is the authoritative source. This plan summarizes key points but the spec has full details.

6. **Track progress** - Check off items as you complete them. Report blockers immediately.

7. **Commit often** - Make small, focused commits with clear messages.

---

## 13. Version History

### Version 1.1 (2026-01-07)

Added CLI integration phase and improved document organization.

**Changes:**

1. **New Phase 6: CLI Integration**
   - Added complete implementation plan for `oxur repl` CLI command
   - 7 tasks covering: command flags, InProcessTransport, terminal interface, interactive/server/connect modes
   - 13 completion criteria for CLI functionality
   - References ODD-0038 Section 1.5 for architecture

2. **Document Organization**
   - Added Table of Contents with section links
   - Numbered all major sections (1-13)
   - Added "User-Ready Release (End of Phase 6)" success criteria

3. **Updated Estimates**
   - Total time updated from 7-11 weeks to 8-13 weeks (includes Phase 6)
   - Updated overview table to include Phase 6

**Impact:** Plan now covers full user-facing REPL experience, not just backend implementation

---

### Version 1.0 (2026-01-06)

Initial implementation plan based on ODD-0038 v1.2 and code analysis.

**Contents:**

- Phases 0-5 covering core REPL implementation
- Task breakdown for VariableStore, RustAstWrapper, EvalContext, SourceMap
- Success criteria for Minimum Viable REPL and Production Quality
- Notes for Claude Code implementation agent

---

**End of Implementation Plan**
