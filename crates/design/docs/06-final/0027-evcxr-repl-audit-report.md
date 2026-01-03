---
number: 27
title: "evcxr_repl Audit Report"
author: "Claude Code"
component: REPL
tags: [research, compiler]
created: 2026-01-03
updated: 2026-01-03
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# evcxr_repl Audit Report

**Date:** 2026-01-02
**Audited by:** Claude Sonnet 4.5
**Repository:** <https://github.com/evcxr/evcxr>
**Focus:** Patterns and techniques applicable to Oxur REPL implementation

---

## 1. Executive Summary

### High-Level Overview

evcxr_repl is a sophisticated Rust REPL that compiles user code into dynamic libraries (cdylib) and executes them in a separate subprocess. The architecture emphasizes **compilation over interpretation**, maintaining state between evaluations through a runtime variable store, and providing robust error recovery through multi-pass compilation with automatic fixes.

**Core Architecture:**

- **Main Process**: Parses input, manages state, invokes cargo/rustc, handles I/O
- **Subprocess (Runtime)**: Loads and executes compiled dynamic libraries via libloading
- **Variable Persistence**: Type-erased storage using `Box<dyn Any>` with runtime type checking
- **Incremental State**: Clone-try-commit pattern - state changes are tentative until compilation succeeds

### Key Takeaways for Oxur

1. **Two-Process Model is Essential** - Isolating code execution in a subprocess prevents crashes from propagating to the REPL and enables restart-on-panic without losing session state.

2. **Type-Erased Variable Storage Works** - Using `Box<dyn Any>` with runtime type checking is a proven approach for persisting typed variables across evaluations without requiring serialization.

3. **Multi-Pass Compilation with Auto-Fix** - Rather than failing immediately on compilation errors, evcxr retries up to 5 times with automatic adjustments (variable capture, async mode, question mark support) leading to better UX.

4. **Dynamic Library Loading is Fast** - After initial compilation (~50-200ms), loading pre-compiled .so/.dylib files is nearly instantaneous, making evcxr's approach viable for interactive use.

5. **State Management Complexity** - The clone-try-commit pattern adds significant complexity but is necessary to maintain consistency when compilation can fail at any point.

### Biggest Surprises

- **No Custom Allocator**: Variables are stored as heap-allocated `Box<dyn Any>` passed via raw pointer between compilations, not through any shared memory scheme.

- **Rustc Wrapper Trick**: evcxr wraps rustc itself to force all dependencies to compile as dylibs, which is critical for fast loading but adds architectural complexity.

- **Aggressive Auto-Fixing**: The REPL automatically enables async mode, question mark support, and tokio dependencies based on compilation errors - this "just works" magic is impressive.

- **No Output Capture via TLS**: Despite expectations, evcxr doesn't use thread-local storage for stdout/stderr capture; instead it relies on the subprocess isolation and simple markers in output streams.

---

## 2. Pattern Catalog

### Pattern 1: Two-Process Execution with Subprocess Isolation

**Description:**

evcxr uses a parent-child process model where the main REPL process handles compilation and state management, while a separate subprocess loads and executes the compiled dynamic libraries. The subprocess runs in a loop, receiving commands via stdin to load libraries and execute functions, then sending results back via stdout.

This isolation prevents user code panics, segfaults, or infinite loops from crashing the REPL session. The parent process can detect subprocess termination and restart it while preserving all variable state and definitions.

**Code Example:**

```rust
// From child_process.rs
pub(crate) struct ChildProcess {
    process_handle: Arc<Mutex<std::process::Child>>,
    stdout: std::io::Lines<BufReader<std::process::ChildStdout>>,
    stdin: Option<std::process::ChildStdin>,
    command: Arc<Mutex<process::Command>>,
}

impl ChildProcess {
    pub(crate) fn send(&mut self, command: &str) -> Result<(), Error> {
        writeln!(self.stdin.as_mut().unwrap(), "{command}")?;
        self.stdin.as_mut().unwrap().flush()?;
        Ok(())
    }

    pub(crate) fn recv_line(&mut self) -> Result<String, Error> {
        Ok(self.stdout.next().ok_or_else(|| self.get_termination_error())??)
    }
}

// From runtime.rs - subprocess side
fn run_loop(&mut self) -> ! {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if let Err(error) = self.handle_line(&line) {
            eprintln!("Error: {error:?}");
            std::process::exit(99);
        }
    }
    std::process::exit(0);
}

fn load_and_run(&mut self, so_path: &str, fn_name: &str) -> Result<(), Error> {
    let shared_object = unsafe { libloading::Library::new(so_path) }?;
    unsafe {
        let user_fn = shared_object
            .get::<extern "C" fn(*mut c_void) -> *mut c_void>(fn_name.as_bytes())?;
        self.variable_store_ptr = user_fn(self.variable_store_ptr);
    }
    println!("{EVCXR_EXECUTION_COMPLETE}");
    self.shared_objects.push(shared_object);
    Ok(())
}
```

**Relevance to Oxur:** **High** - Critical for Oxur's session isolation and network protocol design. Since Oxur must support multiple concurrent sessions, this subprocess pattern is essential.

**Complexity:** **Moderate** - Subprocess management, IPC protocol, and error handling require careful implementation but are well-understood patterns.

**Priority:** **P0** - Must have for v1.0. Core to safe execution and crash recovery.

**Integration Notes:**

For Oxur, we'd extend this pattern to support the network protocol layer. Each REPL session would have its own subprocess (or pool of subprocesses for different session IDs). The protocol messages (eval, load-file, interrupt) would translate to subprocess commands.

Key adaptations:

- Use session ID to route commands to correct subprocess
- Extend the simple text protocol to handle binary postcard serialization
- Add timeout handling for long-running evaluations
- Implement graceful shutdown per session

**Risks/Considerations:**

- **Resource overhead**: One process per session could be heavy for many concurrent users
- **Platform differences**: Process creation and IPC differ on Windows vs Unix
- **Serialization boundary**: Passing complex types between parent/child requires careful ABI design
- **Debugging complexity**: Errors can occur in either process, requiring good error propagation

---

### Pattern 2: Type-Erased Variable Storage with Runtime Checking

**Description:**

evcxr persists variables across evaluations using a `VariableStore` that holds `Box<dyn Any + 'static>`. Before each evaluation, the generated code checks that variables haven't changed type using `Any::downcast_ref`, then extracts them with `Any::downcast`. This allows storing arbitrary user types without serialization or trait requirements.

The parent process tracks variable names and their Rust type strings (e.g., `"i32"`, `"Vec<String>"`), which are determined via rust-analyzer's type inference. The generated code includes type annotations matching these tracked types.

**Code Example:**

```rust
// From evcxr_internal_runtime.rs (embedded in generated code)
pub struct VariableStore {
    variables: std::collections::HashMap<String, Box<dyn std::any::Any + 'static>>,
}

impl VariableStore {
    pub fn put_variable<T: 'static>(&mut self, name: &str, value: T) {
        self.variables.insert(name.to_owned(), Box::new(value));
    }

    pub fn check_variable<T: 'static>(&mut self, name: &str) -> bool {
        if let Some(v) = self.variables.get(name)
            && v.downcast_ref::<T>().is_none()
        {
            eprintln!("The type of the variable {name} was redefined, so was lost.");
            println!("{VARIABLE_CHANGED_TYPE}{name}");
            return false;
        }
        true
    }

    pub fn take_variable<T: 'static>(&mut self, name: &str) -> T {
        match self.variables.remove(name) {
            Some(v) => *v.downcast().expect("Variable changed type"),
            None => panic!("Variable '{name}' has gone missing"),
        }
    }
}

// Generated code pattern (from eval_context.rs)
fn run_user_code_N(
    mut evcxr_variable_store: *mut evcxr_internal_runtime::VariableStore
) -> *mut evcxr_internal_runtime::VariableStore {
    let evcxr_variable_store = unsafe {&mut *evcxr_variable_store};

    // Check types match
    if !evcxr_variable_store.check_variable::<i32>("x") {
        return evcxr_variable_store;
    }

    // Load variables
    let mut x = evcxr_variable_store.take_variable::<i32>("x");

    // User code runs here
    x = x + 1;

    // Store variables
    evcxr_variable_store.put_variable::<i32>("x", x);

    evcxr_variable_store
}
```

**Relevance to Oxur:** **High** - Directly applicable to Tier 2 execution. Oxur's compilation mode needs the same variable persistence mechanism.

**Complexity:** **Simple** - The `Any` trait provides all necessary functionality. Main complexity is tracking types and generating correct code.

**Priority:** **P0** - Essential for v1.0. Variables must persist between REPL evaluations.

**Integration Notes:**

Oxur can use this exact pattern for Tier 2 (compiled) execution. For Tier 1 (calculator mode), we'd skip the variable store entirely since we're only evaluating pure expressions.

Enhancements for Oxur:

- Add support for serializable types to enable session persistence across server restarts
- Consider using `evcxr_runtime` crate directly rather than embedding the code
- Track variable definitions in S-expression form for better error messages

**Risks/Considerations:**

- **Type changes lose variables**: If user redefines `let x: String`, the previous `i32` value is dropped
- **No compile-time safety**: Wrong type strings cause runtime panics
- **Requires 'static**: Variables can't contain non-static references (evcxr provides good error messages for this)
- **Memory leaks possible**: If subprocess crashes before returning variable store, memory leaks in parent

---

### Pattern 3: Multi-Pass Compilation with Automatic Error Fixing

**Description:**

Rather than failing immediately when rustc reports errors, evcxr analyzes the error codes and attempts automatic fixes, then retries compilation (up to 5 times). This creates a "smart REPL" that enables async mode, question mark operators, or adjusts variable capture automatically based on what the compiler reports.

The pattern uses error code matching (E0728 for async, E0277 for question mark, E0382 for moved values, etc.) to trigger specific fixes. Each fix is tracked to avoid infinite loops, and the retry counter prevents runaway attempts.

**Code Example:**

```rust
// From eval_context.rs
fn run_statements(
    &mut self,
    mut user_code: CodeBlock,
    code_info: &UserCodeInfo,
    state: &mut ContextState,
    phases: &mut PhaseDetailsBuilder,
    callbacks: &mut EvalCallbacks,
) -> Result<EvalOutputs, Error> {
    let mut remaining_retries = 5;
    loop {
        // Try to compile and run the code
        let result = self.try_run_statements(
            user_code.clone(),
            state,
            state.compilation_mode(),
            phases,
            callbacks,
        );
        match result {
            Ok(execution_artifacts) => {
                return Ok(execution_artifacts.output);
            }
            Err(Error::CompilationErrors(errors)) => {
                if remaining_retries > 0 {
                    let mut fixed = HashSet::new();
                    for error in &errors {
                        self.attempt_to_fix_error(error, &mut user_code, state, &mut fixed)?;
                    }
                    if !fixed.is_empty() {
                        remaining_retries -= 1;
                        let fixed_sorted: Vec<_> = fixed.into_iter().collect();
                        phases.phase_complete(&fixed_sorted.join("|"));
                        continue;
                    }
                }
                // No more fixes, return error
                return Err(Error::CompilationErrors(errors));
            }
            Err(error) => return Err(error),
        }
    }
}

fn attempt_to_fix_error(
    &mut self,
    error: &CompilationError,
    user_code: &mut CodeBlock,
    state: &mut ContextState,
    fixed_errors: &mut HashSet<&'static str>,
) -> Result<(), Error> {
    for code_origin in &error.code_origins {
        match code_origin {
            CodeKind::OriginalUserCode(_) | CodeKind::OtherUserCode => {
                // E0728: `await` is only allowed inside `async` functions
                if error.code() == Some("E0728") && !state.async_mode {
                    state.async_mode = true;
                    if !state.external_deps.contains_key("tokio") {
                        state.add_dep("tokio",
                            "{version=\"1.34.0\", features=[\"rt\", \"rt-multi-thread\"]}")?;
                        self.write_cargo_toml(state)?;
                    }
                    fixed_errors.insert("Enabled async mode");
                }
                // E0277: `?` couldn't convert the error to the result type
                else if error.code() == Some("E0277") && !state.allow_question_mark {
                    state.allow_question_mark = true;
                    fixed_errors.insert("Allow question mark");
                }
            }
            CodeKind::PackVariable { variable_name } => {
                // E0382: use of moved value
                if error.code() == Some("E0382") {
                    state.variable_states.remove(variable_name);
                    fixed_errors.insert("Captured value");
                }
            }
            CodeKind::WithFallback(fallback) => {
                user_code.apply_fallback(fallback);
                fixed_errors.insert("Fallback");
            }
            _ => {}
        }
    }
    Ok(())
}
```

**Relevance to Oxur:** **Medium** - Useful for improving UX, but not critical for core functionality. Oxur's two-tier model makes some of these fixes less relevant.

**Complexity:** **Complex** - Requires deep understanding of rustc error codes, careful state tracking, and comprehensive testing to avoid false fixes.

**Priority:** **P2** - Nice to have for v1.0, important for v2.0. Start simple with explicit error reporting, add auto-fixes incrementally.

**Integration Notes:**

For Oxur, we'd implement a simplified version:

1. Start with no auto-fixes - just report errors clearly
2. Add the most common fixes:
   - Variable moved → remove from store
   - Type changed → warn and remove
   - Missing async → suggest `(async ...)` wrapper form

Avoid the complexity of async mode auto-enabling since Oxur's Lisp syntax makes async explicit. The question mark auto-enabling could be useful if we support `?` operator.

**Risks/Considerations:**

- **Magical behavior**: Users may not understand why their code suddenly works or why variables disappeared
- **Error code stability**: Rustc error codes can change between versions
- **False positives**: Auto-fixes might mask real errors user should address
- **Performance**: Multiple compilation attempts add latency (evcxr accepts this trade-off)

---

### Pattern 4: Protocol-Based Parent-Child Communication

**Description:**

evcxr uses a simple text-based protocol between parent and child processes. Commands are sent via stdin as text lines (e.g., `LOAD_AND_RUN /path/to/lib.so function_name`), and responses come via stdout with special markers to delimit different content types.

The protocol supports:

- Loading and executing dynamic libraries
- MIME-typed output (via `EVCXR_BEGIN_CONTENT mime_type` ... `EVCXR_END_CONTENT`)
- Error signaling (via special prefixes like `EVCXR_PANIC_NOTIFICATION`)
- Input requests (via `EVCXR_GET_INPUT` for stdin simulation)

**Code Example:**

```rust
// From runtime.rs - Child side
fn run_loop(&mut self) -> ! {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if let Err(error) = self.handle_line(&line) {
            eprintln!("While processing instruction `{line:?}`, got error: {error:?}");
            std::process::exit(99);
        }
    }
    std::process::exit(0);
}

fn handle_line(&mut self, line: &io::Result<String>) -> Result<(), Error> {
    let line = line.as_ref()?;
    static LOAD_AND_RUN: Lazy<Regex> =
        Lazy::new(|| Regex::new("LOAD_AND_RUN ([^ ]+) ([^ ]+)").unwrap());
    if let Some(captures) = LOAD_AND_RUN.captures(line) {
        self.load_and_run(&captures[1], &captures[2])
    } else {
        bail!("Unrecognised line: {}", line);
    }
}

// From eval_context.rs - Parent side
fn run_and_capture_output(
    &mut self,
    state: &mut ContextState,
    so_file: &SoFile,
    callbacks: &mut EvalCallbacks,
) -> Result<EvalOutputs, Error> {
    let mut output = EvalOutputs::new();
    let fn_name = state.current_user_fn_name();

    // Send command
    self.child_process.send(&format!(
        "LOAD_AND_RUN {} {}",
        so_file.path.to_string_lossy(),
        fn_name,
    ))?;

    // Process responses
    loop {
        let line = self.child_process.recv_line()?;
        if line == runtime::EVCXR_EXECUTION_COMPLETE {
            break;
        }
        if line == PANIC_NOTIFICATION {
            got_panic = true;
        } else if let Some(captures) = MIME_OUTPUT.captures(&line) {
            let mime_type = captures[1].to_owned();
            let mut content = String::new();
            loop {
                let line = self.child_process.recv_line()?;
                if line == "EVCXR_END_CONTENT" {
                    break;
                }
                content.push_str(&line);
                content.push('\n');
            }
            output.content_by_mime_type.insert(mime_type, content);
        } else {
            // Regular stdout line
            let _ = self.stdout_sender.send(line);
        }
    }
    Ok(output)
}
```

**Relevance to Oxur:** **Medium** - Oxur will use a more structured binary protocol (Postcard/MessagePack), but the parent-child communication pattern is directly applicable.

**Complexity:** **Simple** - Text-based protocols are easy to debug. Binary protocols add some complexity but provide better performance.

**Priority:** **P1** - Should have for v1.0. The protocol design is core to the architecture.

**Integration Notes:**

Oxur will use a richer protocol with:

- **Framing**: Length-prefixed messages instead of line-based
- **Binary serialization**: Postcard (v1.0) or MessagePack (future)
- **Request-response correlation**: Message IDs to match responses
- **Multiple operations**: Not just LOAD_AND_RUN, but also: EVAL, LOAD_FILE, INTERRUPT, DESCRIBE, etc.

We can still learn from evcxr's protocol design:

- Use special markers for output types (text/plain, text/html, etc.)
- Have a clear "execution complete" signal
- Support streaming partial results for long computations
- Handle input requests from user code

**Risks/Considerations:**

- **Text protocol limitations**: Line-based parsing breaks with multiline content (evcxr uses delimiters to handle this)
- **No version negotiation**: Protocol changes require coordinated parent/child updates
- **Error handling**: Need to handle partial reads, process crashes mid-message
- **Performance**: Text parsing overhead vs binary protocols

---

### Pattern 5: Clone-Try-Commit State Management

**Description:**

evcxr manages REPL state using a clone-try-commit pattern. Before evaluating code, it clones the current committed state (`ContextState`), attempts compilation and execution with the clone, and only commits the changes if successful. This ensures that failed compilations or panicked executions don't corrupt the REPL state.

The `ContextState` struct contains:

- Defined items (functions, structs, types) by name
- External dependencies (crates)
- Variable states (types and mutability)
- Configuration (opt level, error format, etc.)

All modifications are made to the cloned state, and the original is only replaced on success.

**Code Example:**

```rust
// From eval_context.rs
pub struct EvalContext {
    child_process: ChildProcess,
    module: Module,
    committed_state: ContextState,  // The "source of truth"
    analyzer: RustAnalyzer,
    // ...
}

#[derive(Clone, Debug)]
pub struct ContextState {
    items_by_name: HashMap<String, CodeBlock>,
    unnamed_items: Vec<CodeBlock>,
    external_deps: HashMap<String, ExternalCrate>,
    extern_crate_stmts: HashMap<String, String>,
    variable_states: HashMap<String, VariableState>,
    stored_variable_states: HashMap<String, VariableState>,
    attributes: HashMap<String, CodeBlock>,
    async_mode: bool,
    allow_question_mark: bool,
    build_num: i32,
    config: Config,
}

impl EvalContext {
    pub fn state(&self) -> ContextState {
        self.committed_state.clone()  // Clone for tentative modifications
    }

    pub fn eval_with_state(
        &mut self,
        code: &str,
        state: ContextState,  // Pass modified state
    ) -> Result<EvalOutputs, Error> {
        let (user_code, code_info) = CodeBlock::from_original_user_code(code);
        self.eval_with_callbacks(user_code, state, &code_info, &mut EvalCallbacks::default())
    }

    pub(crate) fn eval_with_callbacks(
        &mut self,
        user_code: CodeBlock,
        mut state: ContextState,  // Mutable clone
        code_info: &UserCodeInfo,
        callbacks: &mut EvalCallbacks,
    ) -> Result<EvalOutputs, Error> {
        let mut outputs =
            match self.run_statements(code_out, code_info, &mut state, phases, callbacks) {
                Err(Error::CompilationErrors(errors)) => {
                    // Error: Don't commit state
                    return Err(Error::CompilationErrors(errors));
                }
                error @ Err(_) => return error,
                Ok(x) => x,
            };

        // Success: Commit the state
        self.commit_state(state);

        Ok(outputs)
    }

    fn commit_state(&mut self, mut state: ContextState) {
        // Clean up definition spans
        for variable_state in state.variable_states.values_mut() {
            variable_state.definition_span = None;
        }
        state.stored_variable_states.clone_from(&state.variable_states);
        state.commit_old_user_code();
        self.committed_state = state;  // Replace committed state
    }
}
```

**Relevance to Oxur:** **High** - Essential for maintaining REPL consistency. Oxur's session-based model needs the same transactional state management.

**Complexity:** **Moderate** - The pattern itself is simple (clone, modify, commit), but managing all the state pieces correctly requires careful design.

**Priority:** **P0** - Must have for v1.0. Core to correctness.

**Integration Notes:**

For Oxur, we'd extend this to support:

- **Per-session state**: Each session ID maps to its own `ContextState`
- **State snapshots**: Allow cloning sessions (protocol's `clone` operation)
- **State serialization**: Optional persistence to disk for long-running sessions

The basic pattern remains:

```rust
pub struct SessionState {
    definitions: HashMap<Symbol, Definition>,  // Top-level defs
    variables: VariableStore,  // Runtime variables
    compiler_config: CompilerConfig,  // Optimization, features, etc.
}

impl Session {
    pub fn eval(&mut self, code: SExp) -> Result<Output> {
        let mut state = self.state.clone();  // Clone
        let result = self.try_eval(code, &mut state)?;  // Try
        self.state = state;  // Commit
        Ok(result)
    }
}
```

**Risks/Considerations:**

- **Clone performance**: `ContextState` contains many `HashMap`s that are cloned on each evaluation. Evcxr accepts this cost for correctness.
- **Memory overhead**: Cloning doubles memory usage during evaluation
- **Inconsistent state**: If commit logic has bugs, the REPL can get into invalid states
- **Arc optimization**: Could use `Arc<T>` for large immutable portions to reduce clone overhead

---

### Pattern 6: Dynamic Library Compilation Strategy

**Description:**

evcxr compiles all user code as a `cdylib` (C-compatible dynamic library) with a predictable extern "C" function name that takes and returns a raw pointer to the variable store. This allows loading and executing the code via `libloading` without requiring the subprocess to be recompiled.

Every compilation produces a uniquely named .so/.dylib file (e.g., `libcode_1.so`, `libcode_2.so`) to avoid conflicts. Previous libraries remain loaded to prevent TLS destructor issues, but are never unloaded.

Additionally, evcxr wraps rustc to force all dependencies to compile as dylibs (not rlibs), which dramatically speeds up loading time since the linker doesn't need to statically link large dependency trees.

**Code Example:**

```rust
// From module.rs
fn get_cargo_toml_contents(&self, state: &ContextState) -> String {
    let crate_imports = state.format_cargo_deps();
    format!(
        r#"
[package]
name = "{}"
version = "1.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]  # Compile as dynamic library
path = "src/lib.rs"

[profile.dev]
opt-level = {}
rpath = true  # Enable runtime library search path
# ... other settings

[dependencies]
{}
"#,
        CRATE_NAME,
        state.opt_level(),
        crate_imports
    )
}

pub(crate) fn compile(
    &mut self,
    code_block: &CodeBlock,
    config: &Config,
) -> Result<SoFile, Error> {
    self.write_code(code_block, config)?;
    let cargo_output = run_cargo(config.cargo_command("build"), code_block)?;

    self.build_num += 1;
    let copied_so_file = config.deps_dir().join(
        shared_object_name_from_crate_name(&format!("code_{}", self.build_num))
    );

    // Rename output to unique name to allow loading multiple versions
    rename_or_copy_so_file(&self.so_path(config), &copied_so_file)?;
    Ok(SoFile { path: copied_so_file })
}

// From eval_context.rs - Generated code wrapper
fn wrap_user_code(
    &self,
    mut user_code: CodeBlock,
    compilation_mode: CompilationMode,
) -> CodeBlock {
    code
        .generated("#[unsafe(no_mangle)]")
        .generated(format!(
            "pub extern \"C\" fn {}(",  // Extern C function
            self.current_user_fn_name()  // e.g., run_user_code_42
        ))
        .generated("mut evcxr_variable_store: *mut evcxr_internal_runtime::VariableStore)")
        .generated("  -> *mut evcxr_internal_runtime::VariableStore {")
        // Load variables, run user code, store variables
        .generated("evcxr_variable_store")
        .generated("}")
}

// Rustc wrapper to force dylibs (from module.rs)
fn rustc_command() -> Result<Command> {
    let mut command = std::process::Command::new(rustc);

    if should_force_dylibs() {
        // Make dependencies use dylibs instead of rlibs
        if arg == "--extern" {
            let ext = args.next().ok_or_else(|| anyhow!("Insufficient args"))?;
            let so_arg = map_extern_arg(&ext);  // Convert rlib path to dylib path
            command.arg(so_arg);
        }
        command.arg("--extern").arg(core_extern);  // Force dylib for std
        command.arg("-C").arg("prefer-dynamic");
    }
    Ok(command)
}
```

**Relevance to Oxur:** **High** - This is the core compilation strategy Oxur Tier 2 should use.

**Complexity:** **Complex** - Requires rustc wrapper, careful path management, platform-specific dylib handling, and ABI design.

**Priority:** **P0** - Must have for v1.0. This IS the Tier 2 compilation approach.

**Integration Notes:**

Oxur can largely adopt this pattern:

1. **Compile to cdylib**: Same approach, but signature is:

   ```rust
   extern "C" fn oxur_eval_N(
       var_store: *mut VariableStore,
       args: *const u8,     // Serialized input
       args_len: usize,
   ) -> *mut EvalResult {   // Heap-allocated result
   ```

2. **Force dylibs**: Use the same rustc wrapper pattern

3. **Unique names**: Use session ID + build number: `libsession_abc123_eval_5.so`

4. **Loading**: Use `libloading` same as evcxr

Differences for Oxur:

- Generate from S-expressions (not raw text)
- Support both sync and async evaluation
- Include source map information for error reporting back to Oxur source

**Risks/Considerations:**

- **Compilation speed**: First compile is slow (~50-200ms). Cache aggressively.
- **Disk space**: Each eval creates a new .so file. Need cleanup strategy.
- **Platform differences**: dylib behavior varies (Windows DLL locking, macOS timestamp issues)
- **ABI stability**: Raw pointer passing is unsafe and fragile
- **No unloading**: Libraries accumulate in memory. Accept this or implement careful unloading with TLS workarounds.

---

### Pattern 7: Rust Analyzer Integration for Type Inference

**Description:**

evcxr uses rust-analyzer's LSP-based type inference to determine variable types automatically. Instead of requiring users to annotate types, evcxr analyzes the generated wrapper code to extract variable type names, which it then uses in subsequent code generation.

This avoids the need for a separate type inference engine and leverages rust-analyzer's comprehensive understanding of Rust's type system (including complex types like closures, impl Trait, etc.).

**Code Example:**

```rust
// From eval_context.rs
fn fix_variable_types(
    &mut self,
    state: &mut ContextState,
    code: CodeBlock,
) -> Result<(), Error> {
    // Set analyzer source to wrapper function containing all variables
    self.analyzer.set_source(code.code_string())?;

    // Ask rust-analyzer for types of top-level variables
    for (variable_name, VariableInfo { type_name, is_mutable })
        in self.analyzer.top_level_variables("evcxr_analysis_wrapper")
    {
        if variable_name == "evcxr_variable_store" {
            continue;  // Skip internal variable
        }

        let type_name = match type_name {
            TypeName::Named(x) => x,
            TypeName::Closure => bail!(
                "The variable `{}` is a closure, which cannot be persisted.",
                variable_name
            ),
            TypeName::Unknown => bail!(
                "Couldn't automatically determine type of variable `{}`.",
                variable_name
            ),
        };

        // Store the inferred type
        state
            .variable_states
            .entry(variable_name)
            .or_insert_with(|| VariableState {
                type_name: String::new(),
                is_mut: is_mutable,
                move_state: VariableMoveState::New,
                definition_span: None,
            })
            .type_name = type_name;
    }
    Ok(())
}

// Analysis wrapper that exposes variables (generated code)
fn analysis_code(&self, user_code: CodeBlock) -> CodeBlock {
    let mut code = CodeBlock::new()
        .generated("#[allow(unused_variables)]")
        .generated("async fn evcxr_analysis_wrapper(");

    // Previous variables as parameters (with known types)
    for (var_name, state) in &self.stored_variable_states {
        code = code.generated(format!(
            "{}{}: {},",
            if state.is_mut { "mut " } else { "" },
            var_name,
            state.type_name
        ));
    }

    code = code
        .generated(") -> Result<(), EvcxrUserCodeError> {")
        .add_all(user_code)  // User code defines new variables
        .generated("Ok(())}")  // Rust analyzer can now infer types
        .generated("}");

    code
}
```

**Relevance to Oxur:** **Medium** - Useful for Tier 2, but Oxur may have different type annotation conventions.

**Complexity:** **Moderate** - Requires embedding rust-analyzer or using LSP protocol. Adds dependency weight.

**Priority:** **P2** - Nice to have for v1.0. Start with explicit type annotations in Oxur code.

**Integration Notes:**

Oxur could use rust-analyzer for type inference in Tier 2, but there are alternatives:

**Option 1: Use rust-analyzer** (like evcxr)

- Pro: Comprehensive type inference, handles all Rust complexity
- Con: Heavy dependency, may be overkill for Oxur's needs

**Option 2: Require explicit types** (simpler)

- User writes: `(let x:i32 42)` in Oxur
- Pro: No inference needed, clearer code, faster
- Con: Less ergonomic for interactive use

**Option 3: Limited inference**

- Infer types only from literals and simple expressions
- Pro: Lighter weight, good enough for common cases
- Con: More complex than option 2, less comprehensive than option 1

Recommendation: Start with **Option 2** (explicit types) for v1.0, consider adding **Option 1** for v2.0 based on user feedback.

**Risks/Considerations:**

- **Dependency size**: rust-analyzer is large (~10MB of code)
- **API stability**: LSP internals may change between versions
- **Performance**: Type inference adds latency before compilation
- **Error messages**: Type inference failures produce confusing errors

---

### Pattern 8: Multi-Line Input Validation with Lexical Scanning

**Description:**

evcxr uses a custom lexical scanner to determine whether user input is complete, incomplete, or invalid. This drives the multi-line REPL behavior: if input is incomplete (e.g., unclosed brace), the REPL continues accepting input; if complete, it submits for compilation.

The scanner handles:

- Bracket matching (`()`, `[]`, `{}`)
- String literals (including raw strings `r#"..."#`)
- Comments (line and nested block comments)
- Character literals and lifetimes
- Attributes (`#[...]`)

It avoids false positives (thinking complete code is incomplete) by carefully handling contexts where braces don't indicate nesting (e.g., inside strings or comments).

**Code Example:**

```rust
// From scan.rs
pub enum FragmentValidity {
    Valid,       // Not obviously incomplete or invalid
    Incomplete,  // Unclosed brackets, strings, etc.
    Invalid,     // Mismatched brackets or other errors
}

pub fn validate_source_fragment(source: &str) -> FragmentValidity {
    let mut stack: Vec<Bracket> = vec![];
    let mut input = source.char_indices().peekable();

    while let Some((i, c)) = input.next() {
        match c {
            '/' => match input.peek() {
                Some((_, '/')) => {
                    eat_comment_line(&mut input);
                }
                Some((_, '*')) => {
                    input.next();
                    if !eat_comment_block(&mut input) {
                        return FragmentValidity::Incomplete;  // Unclosed /*
                    }
                }
                _ => {}
            },
            '(' => stack.push(Bracket::Round),
            '[' => stack.push(Bracket::Square),
            '{' => stack.push(Bracket::Curly),
            ')' | ']' | '}' => {
                match (stack.pop(), c) {
                    (Some(Bracket::Round), ')') |
                    (Some(Bracket::Curly), '}') |
                    (Some(Bracket::Square), ']') => {
                        // Matched bracket
                    }
                    _ => {
                        // Mismatched or extra closing bracket
                        return FragmentValidity::Invalid;
                    }
                }
            }
            '"' => {
                // Handle string literals (including raw strings)
                if let Some(hash_count) = check_raw_str(source, i) {
                    if !eat_string(&mut input, hash_count) {
                        return FragmentValidity::Incomplete;  // Unclosed string
                    }
                } else {
                    return FragmentValidity::Invalid;
                }
            }
            '\'' => {
                // Character literal or lifetime
                match eat_char(&mut input) {
                    Some(EatCharRes::SawInvalid) => return FragmentValidity::Invalid,
                    Some(_) => {}
                    None => return FragmentValidity::Incomplete,
                }
            }
            _ => {}
        }
    }

    if stack.is_empty() {
        FragmentValidity::Valid
    } else {
        FragmentValidity::Incomplete  // Unclosed brackets
    }
}
```

**Relevance to Oxur:** **Low** - S-expression syntax makes bracket matching trivial. Oxur's reader handles this automatically.

**Complexity:** **Moderate** - Rust's syntax is complex. S-expressions are much simpler.

**Priority:** **P3** - Not needed for v1.0. Oxur's reader provides this for free.

**Integration Notes:**

Oxur doesn't need custom validation logic because S-expression syntax has trivial nesting rules. Any S-expression reader will naturally:

- Return `Incomplete` when parentheses are unbalanced
- Return `Complete` when all parens are matched
- Handle strings and comments correctly

For the network protocol, the client is responsible for determining when to send input. The server just attempts to parse each message.

For a hypothetical Oxur CLI REPL (not the main use case), we'd use a standard readline library that understands balanced parentheses.

**Risks/Considerations:**

- Not applicable to Oxur's primary use case (network protocol)
- If building a local CLI, use existing Lisp-aware readline (e.g., `rustyline` with custom validator)

---

### Pattern 9: Cargo.toml Dynamic Generation with Dependency Validation

**Description:**

evcxr dynamically generates `Cargo.toml` before each compilation, incorporating user-added dependencies via the `:dep` command. Before adding a dependency, evcxr validates it by writing a temporary `Cargo.toml` and running `cargo metadata` to ensure the crate exists and has a valid lib target.

This prevents common errors like depending on binary-only crates or typos in crate names. The validation also provides better error messages by filtering cargo's verbose output to show only relevant information.

**Code Example:**

```rust
// From cargo_metadata.rs
pub(crate) fn validate_dep(dep: &str, dep_config: &str, config: &Config) -> Result<()> {
    // Write temporary Cargo.toml with just this dependency
    std::fs::write(
        config.crate_dir().join("Cargo.toml"),
        format!(
            r#"
[package]
name = "evcxr_dummy_validate_dep"
version = "0.0.1"
edition = "2024"

[dependencies]
{dep} = {dep_config}
"#
        ),
    )?;

    // Run cargo metadata to validate
    let output = config
        .cargo_command("metadata")
        .arg("--format-version=1")
        .output()?;

    if output.status.success() {
        // Check for "missing lib target" warning
        static NO_LIB_PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new("ignoring invalid dependency `(.*)` which is missing a lib target").unwrap()
        });
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(captures) = NO_LIB_PATTERN.captures(&stderr) {
            bail!("Dependency `{}` is missing a lib target", &captures[1]);
        }
        Ok(())
    } else {
        // Parse and simplify error message
        let mut message = Vec::new();
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            if let Some(captures) = PRIMARY_ERROR_PATTERN.captures(line) {
                message.push(captures[1].to_string());
            } else if !IGNORED_LINES_PATTERN.is_match(line) {
                message.push(line.to_owned());
            }
        }
        bail!(message.join("\n"));
    }
}

// From eval_context.rs - State management
pub fn add_dep(&mut self, dep: &str, dep_config: &str) -> Result<(), Error> {
    // Avoid re-validating if already added
    if let Some(existing) = self.external_deps.get(dep)
        && existing.config == dep_config
    {
        return Ok(());
    }

    let external = ExternalCrate::new(dep.to_owned(), dep_config.to_owned())?;

    // Validate before adding
    crate::cargo_metadata::validate_dep(&external.name, &external.config, &self.config)?;

    self.external_deps.insert(dep.to_owned(), external);
    Ok(())
}

// From module.rs - Cargo.toml generation
fn get_cargo_toml_contents(&self, state: &ContextState) -> String {
    let crate_imports = state.format_cargo_deps();  // Format all dependencies
    format!(
        r#"
[package]
name = "{}"
version = "1.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
{}
"#,
        CRATE_NAME,
        crate_imports  // Inserted here
    )
}
```

**Relevance to Oxur:** **Medium** - Dependency management is important but Oxur may use different mechanisms.

**Complexity:** **Simple** - The validation is straightforward. Main complexity is error message parsing.

**Priority:** **P1** - Should have for v1.0, but implementation can be simpler initially.

**Integration Notes:**

For Oxur, we need dependency management but with some differences:

**Approach for Oxur:**

1. **Protocol command**: `(require "crate-name" :version "1.0")` in Oxur syntax
2. **Validation**: Same as evcxr - run `cargo metadata` before accepting
3. **Session-scoped**: Dependencies are per-session, included in session state
4. **Preloaded deps**: Common crates (serde, tokio, etc.) can be pre-compiled in server

**Enhancements:**

- Cache validation results across sessions
- Support features and git dependencies
- Allow "standard library" of blessed crates that skip validation
- Track dependency tree for debugging

**Risks/Considerations:**

- **Network latency**: Fetching crates from crates.io can be slow (evcxr has `:offline` mode)
- **Security**: Arbitrary dependency loading is a supply chain risk (consider allow-list for production)
- **Disk space**: Each session with different deps creates separate build directories
- **Version conflicts**: Multiple sessions may need different versions of same crate

---

### Pattern 10: Code Block Structure with Origin Tracking

**Description:**

evcxr uses a `CodeBlock` structure that tracks the origin of every segment of generated code. Each segment is tagged with a `CodeKind` that indicates whether it's original user code, generated wrapper code, variable packing code, etc. This enables:

- Accurate error reporting (map compiler errors back to user's original input)
- Selective error handling (e.g., ignore errors in generated code)
- Code transformations (apply fallbacks, remove segments)

The `CodeBlock` is the fundamental data structure that flows through the compilation pipeline, accumulating generated code while preserving the provenance of each piece.

**Code Example:**

```rust
// From code_block.rs
#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub segments: Vec<Segment>,
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub code: String,
    pub kind: CodeKind,
    pub sequence: Option<usize>,  // For tracking position in original input
}

#[derive(Clone, Debug)]
pub enum CodeKind {
    OriginalUserCode(OriginalUserCodeInfo),  // Typed by user
    OtherUserCode,                           // User code from previous evaluations
    PackVariable { variable_name: String },  // Generated: variable storage
    LoadVariable,                            // Generated: variable loading
    WithFallback(String),                    // Code with a backup if compilation fails
    Command(CommandCall),                    // REPL command (like :dep)
    // ... other variants
}

impl CodeBlock {
    pub fn original_user_code(code: String) -> CodeBlock {
        CodeBlock {
            segments: vec![Segment {
                code,
                kind: CodeKind::OriginalUserCode(OriginalUserCodeInfo { node_index: 0 }),
                sequence: Some(0),
            }],
        }
    }

    pub fn generated(mut self, code: impl Into<String>) -> Self {
        self.segments.push(Segment {
            code: code.into(),
            kind: CodeKind::Generated,
            sequence: None,
        });
        self
    }

    pub fn pack_variable(&mut self, variable_name: String, code: String) {
        self.segments.push(Segment {
            code,
            kind: CodeKind::PackVariable { variable_name },
            sequence: None,
        });
    }

    pub fn code_string(&self) -> String {
        self.segments.iter().map(|s| s.code.as_str()).collect()
    }
}

// Error mapping (from eval_context.rs)
fn apply_custom_errors(
    &self,
    errors: Vec<CompilationError>,
    user_code: &CodeBlock,
    code_info: &UserCodeInfo,
) -> Vec<CompilationError> {
    errors
        .into_iter()
        .filter_map(|error| self.customize_error(error, user_code))
        .map(|mut error| {
            error.fill_lines(code_info);  // Map back to original line numbers
            error
        })
        .collect()
}
```

**Relevance to Oxur:** **High** - Critical for error reporting back to Oxur source positions.

**Complexity:** **Moderate** - The structure is simple, but using it correctly throughout the pipeline requires discipline.

**Priority:** **P0** - Must have for v1.0. Users need accurate error locations.

**Integration Notes:**

Oxur should adopt a similar structure but adapted for S-expressions:

```rust
pub struct OxurCodeBlock {
    segments: Vec<CodeSegment>,
    source_map: SourceMap,  // Maps generated Rust → Oxur S-expr positions
}

pub enum CodeSegment {
    UserCode {
        rust_code: String,
        oxur_sexp: SExp,          // Original S-expression
        position: SourcePosition,  // Position in Oxur source file
    },
    Generated {
        rust_code: String,
        purpose: GeneratedPurpose,  // Why this was generated
    },
    VariablePack { name: String, code: String },
    VariableLoad { name: String, code: String },
}

pub enum GeneratedPurpose {
    WrapperFunction,
    VariableStore,
    ErrorHandling,
    AsyncWrapper,
    // ... others
}
```

When rustc reports an error at line N of the generated Rust code, we:

1. Find the segment containing line N
2. If it's `UserCode`, map back to the Oxur source position
3. Report error with Oxur file/line/column, highlighting the original S-expression

**Risks/Considerations:**

- **Source map accuracy**: Must be kept in perfect sync with generated code
- **Error spans**: Rustc errors may span multiple segments with different origins
- **Performance**: Tracking every segment adds memory overhead
- **Debugging**: Generated code should still be viewable for debugging

---

## 3. Architecture Comparison

| Aspect | evcxr_repl | Oxur REPL | Assessment |
|--------|-----------|-----------|------------|
| **Transport** | Local only (stdin/stdout) | Multi-transport (TCP, Unix, pipe, in-process) | Oxur more flexible for client-server use |
| **Session Model** | Implicit (single global state) | Explicit (session IDs, concurrent sessions) | Oxur better isolation and concurrency |
| **Syntax** | Rust source code | S-expressions (Lisp) | Oxur simpler to parse, easier input validation |
| **Compilation Strategy** | Always compile to cdylib | Two-tier: interpret literals, compile everything else | Oxur faster for simple cases (~1ms vs ~50ms) |
| **State Persistence** | In-memory only, lost on quit | In-memory + optional serialization | Oxur can persist sessions across restarts |
| **Variable Storage** | `Box<dyn Any>` with raw pointers | Same mechanism for Tier 2 | Equivalent approach |
| **Type Inference** | rust-analyzer for automatic types | Explicit type annotations (initially) | evcxr more ergonomic, Oxur more explicit |
| **Error Recovery** | Multi-pass with auto-fixes (async, ?, moved vars) | Single-pass with clear errors (initially) | evcxr more magical, Oxur more predictable |
| **Dependency Management** | `:dep` command, validates with cargo metadata | `(require ...)` form, same validation | Equivalent functionality, different syntax |
| **Output Capture** | MIME types via special markers in stdout | Separate stdout/stderr in protocol response | Oxur cleaner separation |
| **Multi-line Input** | Lexical scanner for Rust syntax | S-expression reader (builtin) | Oxur simpler implementation |
| **Process Model** | One subprocess per process | One subprocess per session | Oxur higher resource usage but better isolation |
| **Async Support** | Auto-detects and wraps in tokio runtime | Explicit `(async ...)` form | Oxur more explicit, no magic |
| **rustc Integration** | Custom wrapper to force dylibs | Same approach needed | Equivalent complexity |
| **Compilation Speed** | 50-200ms first time, ~0ms cached | Same for Tier 2, ~1ms for Tier 1 | Oxur faster average due to calculator mode |
| **Source Mapping** | CodeBlock with origin tracking | Same with S-expr positions | Equivalent capability, Oxur slightly simpler |
| **Command System** | `:command args` (colon prefix) | `(command ...)` (S-expr form) | Both work, Oxur more uniform syntax |
| **Completions** | rust-analyzer LSP | May use rust-analyzer for Tier 2 | Equivalent for compilation mode |
| **Network Protocol** | N/A (local only) | Postcard/MessagePack with framing | Oxur unique feature |
| **Interrupt Handling** | Kill subprocess, restart | Protocol message + subprocess signal | Oxur more graceful |

**Overall Assessment:** Oxur's architecture is significantly more sophisticated for networked, multi-session use cases, while evcxr is optimized for single-user local REPL interaction. The core compilation and state management techniques are directly transferable, but Oxur adds essential features for server deployment.

---

## 4. Recommendations

### Must Adopt (P0)

**1. Two-Process Execution Model** (`runtime.rs`, `child_process.rs`)

- **Why**: Isolates user code crashes from REPL session. Critical for server stability.
- **Justification**: Non-negotiable for production. User code can panic, segfault, or loop infinitely.
- **Implementation**: Create `subprocess.rs` module with similar architecture, extended for session management.

**2. Type-Erased Variable Storage** (`evcxr_internal_runtime.rs`)

- **Why**: Proven approach for persisting typed values without serialization.
- **Justification**: Simpler than alternatives (serde for all types, reflection, codegen).
- **Implementation**: Use `evcxr_runtime` crate directly or fork for Oxur-specific needs.

**3. Clone-Try-Commit State Management** (`eval_context.rs` lines 462-540)

- **Why**: Ensures REPL consistency when compilation/execution fails.
- **Justification**: Transactional semantics prevent corrupted state.
- **Implementation**: `SessionState::eval()` clones state, attempts eval, commits on success.

**4. Dynamic Library Compilation** (`module.rs` lines 179-217)

- **Why**: Core to Tier 2 execution strategy. Fast loading after compilation.
- **Justification**: This IS the compilation approach for Oxur Tier 2.
- **Implementation**: Generate Cargo.toml, compile to cdylib, load with libloading.

**5. Code Origin Tracking** (`code_block.rs`)

- **Why**: Accurate error reporting is essential for UX.
- **Justification**: Users must know where errors are in their Oxur source.
- **Implementation**: Extend CodeBlock for S-expr source mapping.

### Should Consider (P1)

**6. Dependency Validation** (`cargo_metadata.rs` lines 38-91)

- **Why**: Prevents common errors and provides better messages.
- **Justification**: Improves UX significantly, low implementation cost.
- **Implementation**: Wrap `cargo metadata` calls before accepting `(require ...)` forms.

**7. rustc Wrapper for Forced Dylibs** (`module.rs` lines 289-432)

- **Why**: Dramatically improves load times by avoiding static linking.
- **Justification**: Makes Tier 2 compilation practical for interactive use.
- **Implementation**: Create `oxur-rustc-wrapper` binary, set RUSTC_WRAPPER env var.

**8. Structured Protocol Communication** (evcxr pattern extended)

- **Why**: Foundation for network protocol, better than ad-hoc parsing.
- **Justification**: Enables robust client-server communication.
- **Implementation**: Design postcard-based binary protocol with message framing.

**9. Subprocess Restart on Crash** (`child_process.rs` lines 118-134)

- **Why**: Recovers from user code crashes without losing session.
- **Justification**: Critical for resilience in server environment.
- **Implementation**: Detect subprocess exit, respawn, reinitialize with preserved state.

### Can Skip (P2-P3)

**10. Multi-Pass Compilation with Auto-Fix** (`eval_context.rs` lines 691-787)

- **Why**: Complex, magical behavior that may confuse users.
- **Trade-off**: Better errors vs. simplicity and transparency.
- **Recommendation**: Skip for v1.0. Revisit for v2.0 based on user feedback. Start with clear error messages.

**11. rust-analyzer Integration** (type inference)

- **Why**: Heavy dependency, may be overkill for Oxur's needs.
- **Trade-off**: Ergonomics vs. simplicity and explicit types.
- **Recommendation**: Start with explicit type annotations `(let x:i32 42)`. Add inference in v2.0 if users request it.

**12. Multi-Line Input Scanner** (`scan.rs`)

- **Why**: S-expression syntax makes this trivial - readers handle it natively.
- **Trade-off**: Not applicable to Oxur's main use case (network protocol).
- **Recommendation**: Skip entirely. Use standard S-expr reader for CLI if needed.

**13. Command System** (`:command` syntax)

- **Why**: Oxur can use regular S-expr forms like `(session describe)` instead.
- **Trade-off**: Uniform syntax vs. special syntax for meta-commands.
- **Recommendation**: Use S-expr forms for commands, not special syntax.

### Novel Solutions Needed

**14. Session Management**

- **Gap**: evcxr has implicit single-session model; Oxur needs explicit multi-session.
- **Solution**:

  ```rust
  pub struct SessionManager {
      sessions: HashMap<SessionId, Session>,
      subprocess_pool: SubprocessPool,
  }
  ```

- **Complexity**: New architectural layer not present in evcxr.

**15. Network Protocol Implementation**

- **Gap**: evcxr is local-only; Oxur needs TCP/Unix socket servers.
- **Solution**: Use tokio for async I/O, implement length-delimited framing, postcard serialization.
- **Complexity**: Moderate - well-understood patterns, but requires careful design.

**16. Tier 1 Calculator Mode**

- **Gap**: evcxr always compiles; Oxur needs fast-path for literals.
- **Solution**: Pattern match on S-expressions to detect calculator forms, evaluate directly:

  ```rust
  match sexp {
      List([Symbol("+"), Int(a), Int(b)]) => Ok(Int(a + b)),  // ~100ns
      _ => compile_and_execute(sexp)  // ~50-200ms
  }
  ```

- **Complexity**: Simple - straightforward pattern matching on limited forms.

**17. S-Expression to Rust Code Generation**

- **Gap**: evcxr works with Rust text; Oxur needs S-expr → Rust codegen.
- **Solution**: Implement `SExpCompiler` that walks S-expressions and generates CodeBlock:

  ```rust
  impl SExpCompiler {
      fn compile_to_rust(&self, sexp: &SExp) -> Result<CodeBlock> {
          match sexp {
              List([Symbol("let"), Symbol(var), ty, expr]) => { /* generate let statement */ }
              List([Symbol("fn"), Symbol(name), args, body]) => { /* generate function */ }
              // ... handle all Oxur forms
          }
      }
  }
  ```

- **Complexity**: High - core compiler component, requires comprehensive implementation.

**18. Source Map Integration**

- **Gap**: evcxr maps Rust → Rust; Oxur needs Rust → Oxur (S-expr).
- **Solution**: Extend CodeBlock to track S-expr positions:

  ```rust
  pub struct SourcePosition {
      file: PathBuf,
      sexp_start: usize,  // Byte offset in Oxur source
      sexp_end: usize,
  }
  ```

  Generate source map during compilation, use when reporting errors.
- **Complexity**: Moderate - requires careful position tracking during codegen.

**19. Session State Serialization**

- **Gap**: evcxr state is in-memory only; Oxur may need persistence.
- **Solution**: Implement `serde` for `SessionState`, save to disk on shutdown:

  ```rust
  impl SessionState {
      pub fn save(&self, path: &Path) -> Result<()> {
          let serialized = postcard::to_allocvec(self)?;
          std::fs::write(path, serialized)?;
          Ok(())
      }
      pub fn load(path: &Path) -> Result<Self> {
          let bytes = std::fs::read(path)?;
          Ok(postcard::from_bytes(&bytes)?)
      }
  }
  ```

- **Complexity**: Moderate - requires making all state types serializable.

**20. Oxur Macro Expansion**

- **Gap**: evcxr doesn't handle Lisp macros; Oxur needs macro expansion before compilation.
- **Solution**: Implement macro expander before lowering to Rust:

  ```
  Oxur Source → Parse → Surface Forms → **Macro Expand** → Core Forms → Lower to Rust AST
  ```

- **Complexity**: High - core language feature, requires hygiene, recursion limits, etc.

---

## 5. Risk Assessment

### Technical Risks

**1. Dynamic Library Loading Reliability**

- **Risk**: Platform-specific bugs in dlopen/LoadLibrary, especially on Windows
- **Impact**: High - core functionality breaks
- **Mitigation**: Extensive testing on all platforms, fallback to static linking if needed
- **evcxr Experience**: They handle platform differences carefully (see `rename_or_copy_so_file`)

**2. Memory Leaks from Unloaded Libraries**

- **Risk**: evcxr never unloads libraries to avoid TLS destructor issues; memory grows unbounded
- **Impact**: Medium - long-running servers accumulate memory
- **Mitigation**: Implement subprocess recycling (kill and restart after N evaluations)
- **evcxr Experience**: They accept the leak as necessary trade-off

**3. Type-Erased Storage Panics**

- **Risk**: `Any::downcast` panics if types mismatch, could crash subprocess
- **Impact**: Low - subprocess crash is recoverable, but disrupts user
- **Mitigation**: Thorough type checking before downcast, comprehensive testing
- **evcxr Experience**: Rare in practice due to rust-analyzer type inference

**4. rustc Version Compatibility**

- **Risk**: Rustc changes error codes, flags, or dylib format
- **Impact**: Medium - breaks auto-fixes or compilation
- **Mitigation**: Pin rustc version for server, test new versions before upgrading
- **evcxr Experience**: They track stable and nightly, adapt to changes incrementally

**5. Subprocess Communication Deadlocks**

- **Risk**: Parent/child block waiting for each other on stdin/stdout
- **Impact**: High - hangs entire session
- **Mitigation**: Careful protocol design, timeouts, asynchronous I/O
- **evcxr Experience**: They use blocking I/O but manage it carefully

### Maintenance Risks

**6. Complexity of Multi-Pass Compilation**

- **Risk**: Auto-fix logic is complex, hard to debug, may introduce subtle bugs
- **Impact**: Medium - confusing behavior, hard to troubleshoot
- **Mitigation**: Skip for v1.0, start simple with clear errors
- **evcxr Experience**: They handle complexity but it's a significant maintenance burden

**7. State Management Bugs**

- **Risk**: Clone-try-commit pattern has many edge cases, state corruption possible
- **Impact**: High - invalid REPL state requires session restart
- **Mitigation**: Extensive unit tests, property tests for state invariants
- **evcxr Experience**: Mature implementation, but still has occasional state bugs

**8. Platform-Specific Code Paths**

- **Risk**: Windows, macOS, Linux all have subtle differences (file locking, timestamps, etc.)
- **Impact**: Medium - works on one platform, broken on another
- **Mitigation**: CI testing on all platforms, conditional compilation carefully managed
- **evcxr Experience**: They handle this well with platform-specific code blocks

### Compatibility Risks

**9. Oxur Syntax vs Rust Semantics Mismatch**

- **Risk**: Not all Rust features map cleanly to S-expression syntax
- **Impact**: High - core language design question
- **Mitigation**: Careful language design, document unsupported features
- **evcxr Experience**: N/A - they use Rust syntax directly

**10. Network Protocol Evolution**

- **Risk**: Protocol changes break existing clients
- **Impact**: Medium - client/server version mismatches
- **Mitigation**: Version negotiation in protocol, maintain backward compatibility
- **evcxr Experience**: N/A - no network protocol

**11. Session Isolation Violations**

- **Risk**: Global state leaks between sessions (environment variables, global statics, etc.)
- **Impact**: High - security and correctness issue
- **Mitigation**: Separate processes per session, audit global state carefully
- **evcxr Experience**: N/A - single session model

**12. Dependency Bloat**

- **Risk**: rust-analyzer, tokio, and other deps make Oxur binary large (>50MB)
- **Impact**: Low - disk space is cheap, but slows downloads
- **Mitigation**: Make heavy deps optional, provide minimal and full builds
- **evcxr Experience**: They include rust-analyzer, binary is ~20MB stripped

---

## 6. Questions for Further Investigation

### Architecture Questions

**1. Session Subprocess Model**

- **Question**: Should Oxur use one subprocess per session, or a shared pool?
- **Trade-offs**:
  - One per session: Full isolation, higher resource usage
  - Shared pool: Lower resources, potential cross-contamination
- **Investigation**: Benchmark memory usage and startup time for each model
- **evcxr**: One subprocess total (single session)

**2. Variable Store Serialization**

- **Question**: Should variable stores be serializable for session persistence?
- **Trade-offs**:
  - Yes: Sessions survive server restarts
  - No: Simpler implementation, no serialization overhead
- **Investigation**: Survey user needs for long-running sessions
- **evcxr**: No serialization (in-memory only)

**3. Compilation Caching Strategy**

- **Question**: How aggressively should we cache compiled libraries?
- **Trade-offs**:
  - Aggressive (content-addressed): Faster for repeated code, complex implementation
  - Simple (session-scoped): Slower but easier to manage
- **Investigation**: Measure cache hit rates in realistic REPL usage
- **evcxr**: Minimal caching (incremental compilation only)

### Implementation Questions

**4. Type Inference for Tier 2**

- **Question**: Should Oxur require explicit types or infer from Rust?
- **Trade-offs**:
  - Explicit: Simpler, faster, more control
  - Inferred: Ergonomic, but adds rust-analyzer dependency
- **Investigation**: User study on type annotation preferences
- **evcxr**: Full inference via rust-analyzer

**5. Error Recovery Automation**

- **Question**: Which auto-fixes (if any) should Oxur implement?
- **Trade-offs**:
  - None: Transparent, predictable
  - Some (moved variables): Convenient, potential confusion
  - Many (async, ?): Magic, harder to understand
- **Investigation**: Analyze most common user errors in prototype
- **evcxr**: Many auto-fixes

**6. Subprocess Communication Protocol**

- **Question**: Text-based or binary protocol for parent-child?
- **Trade-offs**:
  - Text: Easy to debug, less efficient
  - Binary: Faster, compact, harder to troubleshoot
- **Investigation**: Benchmark protocol overhead for typical evaluations
- **evcxr**: Text-based with regex parsing

### Operational Questions

**7. Dynamic Library Cleanup**

- **Question**: When and how should we clean up old .so files?
- **Options**:
  - Never (like evcxr) - simple but grows unbounded
  - On session close - simple but prevents reload
  - LRU eviction - complex but manages resources
- **Investigation**: Measure typical .so file sizes and REPL session lengths
- **evcxr**: Never cleans up libraries (memory leak accepted)

**8. Subprocess Restart Strategy**

- **Question**: When should we restart subprocesses?
- **Options**:
  - On crash only
  - After N evaluations (prevent resource leaks)
  - On user request
  - All of the above
- **Investigation**: Profile memory growth in long-running subprocesses
- **evcxr**: On crash only (via explicit restart API)

**9. Dependency Pre-compilation**

- **Question**: Should common crates be pre-compiled on server startup?
- **Trade-offs**:
  - Yes: First use is fast, server startup slower
  - No: Faster startup, first use is slow
- **Investigation**: Identify most commonly used crates in Rust REPL usage
- **evcxr**: No pre-compilation (on-demand only)

### Security Questions

**10. Dependency Allow-List**

- **Question**: Should production Oxur servers restrict allowed dependencies?
- **Trade-offs**:
  - Allow-list: Secure, limited
  - Open: Flexible, potential supply chain attacks
- **Investigation**: Define production deployment security model
- **evcxr**: No restrictions (assumes trusted users)

**11. Resource Limits**

- **Question**: What limits should apply to user code (CPU, memory, disk, network)?
- **Options**:
  - No limits (trust model)
  - Soft limits with warnings
  - Hard limits with termination
- **Investigation**: Design threat model for multi-tenant deployments
- **evcxr**: No limits (single-user assumption)

**12. Sandbox Requirements**

- **Question**: Should Oxur run user code in a sandbox (containers, VMs, seccomp)?
- **Trade-offs**:
  - Sandbox: Secure, complex, resource overhead
  - No sandbox: Simple, requires trusted environment
- **Investigation**: Determine deployment scenarios (internal tool vs public service)
- **evcxr**: No sandbox (trusted user model)

---

## 7. Code Hotspots

### Critical Files Worth Deep Study

**State Management:**

```
evcxr/src/eval_context.rs:50-689
  - ContextState structure and cloning
  - commit_state() pattern
  - Variable type tracking and inference integration
```

**Subprocess Execution:**

```
evcxr/src/runtime.rs:34-135
  - Runtime loop and command handling
  - Dynamic library loading with libloading
  - Crash handler installation (Unix signals)

evcxr/src/child_process.rs:16-195
  - Subprocess lifecycle management
  - stdin/stdout communication protocol
  - Restart-while-preserving-handle pattern
```

**Compilation Strategy:**

```
evcxr/src/module.rs:128-287
  - Cargo.toml generation
  - cdylib compilation configuration
  - Unique .so file naming and copying

evcxr/src/module.rs:289-432
  - rustc wrapper for forcing dylibs
  - External dependency path manipulation
  - --extern flag handling
```

**Variable Persistence:**

```
evcxr/src/evcxr_internal_runtime.rs:15-80
  - VariableStore with Box<dyn Any>
  - Type checking and downcasting
  - lazy_arc pattern for expensive initialization

evcxr/src/eval_context.rs:1552-1763
  - Code generation for variable loading/storing
  - Type annotation injection
  - catch_unwind wrapping for panic recovery
```

**Error Recovery:**

```
evcxr/src/eval_context.rs:691-787
  - Multi-pass compilation loop
  - attempt_to_fix_error() pattern
  - Error code matching (E0382, E0728, etc.)

evcxr/src/eval_context.rs:964-1042
  - Error code to fix mapping
  - State mutation on fixes
  - Fixed error tracking
```

**Code Origin Tracking:**

```
evcxr/src/code_block.rs:1-500 (entire file)
  - CodeBlock and Segment structures
  - CodeKind variants
  - Builder pattern for code assembly

evcxr/src/eval_context.rs:1414-1483
  - Error customization based on code origins
  - Source span mapping
  - User-facing error generation
```

**Dependency Management:**

```
evcxr/src/cargo_metadata.rs:38-91
  - Dependency validation with cargo metadata
  - Error message filtering and simplification
  - Library name extraction

evcxr/src/eval_context.rs:1385-1403
  - add_dep() with validation
  - External crate storage in ContextState
```

**Multi-Line Input:**

```
evcxr_repl/src/scan.rs:91-300
  - Source fragment validation
  - Bracket/string/comment handling
  - Incomplete vs invalid detection

evcxr_repl/src/repl.rs:81-112
  - Validator integration with rustyline
  - Double-newline escape hatch
```

**Type Inference Integration:**

```
evcxr/src/eval_context.rs:819-868
  - rust-analyzer source setup
  - top_level_variables extraction
  - Type name cleaning and validation

evcxr/src/rust_analyzer.rs:1-500 (entire file)
  - LSP server integration
  - Completion and hover queries
  - Type information extraction
```

**Output Capture:**

```
evcxr/src/eval_context.rs:870-962
  - MIME-typed output parsing
  - Panic notification handling
  - Input request callbacks
  - Streaming output to channels
```

### Key Patterns Demonstrated

**Pattern: Generate wrapper function with predictable signature**

```
evcxr/src/eval_context.rs:1615-1709
Shows: How to wrap user code in extern "C" function with variable store parameter
```

**Pattern: Parse JSON output from cargo commands**

```
evcxr/src/module.rs:436-497
Shows: Cargo build error extraction, JSON message parsing
```

**Pattern: Platform-specific handling**

```
evcxr/src/module.rs:90-118
Shows: Windows vs Unix file operations (copy vs rename)

evcxr/src/module.rs:225-241
Shows: macOS filesystem timestamp workarounds
```

**Pattern: Retry loop with error-based fixes**

```
evcxr/src/eval_context.rs:720-786
Shows: Loop with counter, fix tracking, early exit on success
```

**Pattern: Raw pointer passing across FFI**

```
evcxr/src/runtime.rs:77-88
Shows: Loading function from dylib, passing/receiving raw pointer
```

---

## 8. Implementation Roadmap for Oxur

### Phase 1: Core Execution (Weeks 1-4)

**Goal**: Get basic Tier 2 compilation working

1. **Implement VariableStore** (Week 1)
   - Port `evcxr_internal_runtime.rs` to `oxur-runtime` crate
   - Add tests for put/check/take with various types
   - Document 'static and Any limitations

2. **Build Subprocess Infrastructure** (Week 1-2)
   - Create `subprocess.rs` with ChildProcess equivalent
   - Implement text protocol: LOAD_AND_RUN, COMPLETE
   - Add crash detection and restart logic
   - Test on all platforms (Windows, macOS, Linux)

3. **Implement CodeBlock for S-expressions** (Week 2)
   - Create `OxurCodeBlock` with source map tracking
   - Add segment types for user/generated code
   - Implement builders for common patterns

4. **Basic Compiler (S-expr → Rust)** (Week 3-4)
   - Handle let bindings: `(let x:i32 42)`
   - Handle function calls: `(+ x 1)`
   - Generate wrapper extern "C" function
   - Emit Cargo.toml with cdylib config
   - Test round-trip: eval → compile → load → execute

**Success Criteria**:

- Can evaluate `(let x:i32 42)` followed by `(+ x 1)` → returns `43`
- Variables persist between evaluations
- Subprocess recovers from panic

### Phase 2: State Management (Weeks 5-6)

**Goal**: Transactional state with session management

1. **Implement ContextState Clone-Try-Commit** (Week 5)
   - Create `SessionState` with Clone
   - Implement tentative state pattern in eval()
   - Add variable state tracking (types, mutability)
   - Test state rollback on compilation errors

2. **Add Session Manager** (Week 6)
   - Map SessionId → Session
   - Implement session create/clone/close
   - Add subprocess pool or per-session process
   - Test concurrent sessions

**Success Criteria**:

- Failed compilation doesn't corrupt state
- Multiple sessions run independently
- Session cloning works correctly

### Phase 3: Dependency & Compilation (Weeks 7-9)

**Goal**: Dynamic dependency management and optimized compilation

1. **Implement Dependency Management** (Week 7)
   - Add `(require "serde" :version "1.0")` support
   - Port cargo metadata validation
   - Generate Cargo.toml with dependencies
   - Test common crates (serde, tokio, etc.)

2. **Create rustc Wrapper** (Week 8)
   - Build `oxur-rustc-wrapper` binary
   - Force dylibs for all dependencies
   - Set RUSTC_WRAPPER environment variable
   - Measure compilation speed improvement

3. **Optimize Compilation Pipeline** (Week 9)
   - Add compilation caching (session-scoped initially)
   - Implement fast linker integration (mold/lld)
   - Profile and optimize code generation
   - Benchmark: target <100ms for cached, <200ms for new

**Success Criteria**:

- Can add serde and use it in code
- Compilation uses dylibs (verify with `ldd`/`otool`)
- Cached compilation < 100ms

### Phase 4: Network Protocol (Weeks 10-12)

**Goal**: Production-ready network REPL server

1. **Implement Protocol Layer** (Week 10)
    - Define postcard message types
    - Implement length-delimited framing
    - Add request ID correlation
    - Test serialization round-trips

2. **Build TCP/Unix Socket Server** (Week 11)
    - Use tokio for async I/O
    - Implement transport abstraction
    - Route messages to SessionManager
    - Handle client disconnect gracefully

3. **Add Advanced Operations** (Week 12)
    - Implement interrupt (kill subprocess)
    - Add describe (show session state)
    - Implement history (track eval sequence)
    - Add timeout handling

**Success Criteria**:

- Can connect via TCP and evaluate S-expressions
- Interrupt terminates long-running code
- Protocol handles binary data correctly

### Phase 5: Error Reporting & Polish (Weeks 13-14)

**Goal**: Production-quality error messages and debugging

1. **Implement Source Mapping** (Week 13)
    - Generate accurate source maps during compilation
    - Map rustc errors back to S-expr positions
    - Format errors with S-expr highlighting
    - Test complex multi-line expressions

2. **Add Tier 1 Calculator Mode** (Week 14)
    - Pattern match for literal arithmetic
    - Evaluate directly without compilation
    - Benchmark: target <1ms for calculator
    - Seamlessly fall back to Tier 2

**Success Criteria**:

- Errors point to correct S-expression
- `(+ 1 2)` evaluates in <1ms
- Complex expressions fall back to compilation

---

## 9. Testing Strategy

### Adopt from evcxr

**Integration Tests Pattern** (`evcxr/tests/integration_tests.rs`):

```rust
#[test]
fn test_variable_persistence() {
    let (mut ctx, _outputs) = EvalContext::new().unwrap();

    // First eval defines variable
    assert!(ctx.eval("let x = 42;").is_ok());

    // Second eval uses it
    let result = ctx.eval("x + 1").unwrap();
    assert_eq!(result.get("text/plain"), Some("43"));
}
```

**Adopt for Oxur**:

- Test S-expr → Rust → execute → result for common patterns
- Test state persistence across evals
- Test error cases (type mismatch, moved variables, etc.)

### Property-Based Tests (New)

Use `proptest` to generate random S-expressions and verify invariants:

```rust
proptest! {
    #[test]
    fn test_calculator_matches_compilation(expr in arb_arithmetic_sexp()) {
        let tier1_result = calculator_eval(&expr)?;
        let tier2_result = compile_and_eval(&expr)?;
        assert_eq!(tier1_result, tier2_result);
    }
}
```

### Subprocess Crash Tests

```rust
#[test]
fn test_panic_recovery() {
    let (mut ctx, _) = SessionManager::new().unwrap();

    // Cause panic
    assert!(ctx.eval("(panic \"boom\")").is_err());

    // Should still work
    assert!(ctx.eval("(+ 1 2)").is_ok());
}
```

### Platform-Specific Tests

```rust
#[cfg(target_os = "windows")]
#[test]
fn test_dll_loading() { /* ... */ }

#[cfg(target_os = "macos")]
#[test]
fn test_dylib_timestamps() { /* ... */ }
```

---

## 10. Conclusion

evcxr_repl provides a **solid foundation** for Oxur's Tier 2 compilation strategy. The core patterns - two-process execution, type-erased variable storage, dynamic library compilation, and clone-try-commit state management - are directly applicable and battle-tested.

### Key Insights

1. **Don't reinvent**: The subprocess model, variable store, and compilation strategy are proven. Adopt them.

2. **Extend for sessions**: Oxur's multi-session architecture is a natural extension of evcxr's single-session model.

3. **Simplify auto-magic**: Skip the complex error auto-fixing initially. Start with clear errors, add fixes incrementally based on data.

4. **Protocol is key**: The network protocol is Oxur's unique value-add. Invest in robust design here.

5. **Two tiers complement**: evcxr's compilation-only approach validates Tier 2; Oxur's Tier 1 calculator mode will provide the "feels instant" experience for simple math.

### Confidence Level

After this audit, I'm confident that:

- ✅ Oxur's Tier 2 architecture is viable (evcxr proves it)
- ✅ Variable persistence via `Box<dyn Any>` will work
- ✅ Subprocess isolation is the right approach
- ✅ Dynamic library loading is fast enough for interactive use
- ⚠️ Session management complexity is underestimated (new territory)
- ⚠️ S-expr → Rust codegen quality will make or break UX

### Next Steps

1. **Prototype**: Build minimal Tier 2 (Phase 1) to validate architecture
2. **Benchmark**: Measure compilation speed with dylib forcing
3. **Design protocol**: Finalize message format before implementing server
4. **Plan codegen**: Design S-expression lowering strategy carefully

This audit provides the roadmap and confidence to implement Oxur REPL Tier 2 successfully. 🚀

---

**End of Audit Report**
