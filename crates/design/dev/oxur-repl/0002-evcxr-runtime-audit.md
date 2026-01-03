# Claude Code Prompt: evcxr_runtime Audit

## Your Mission

You are auditing the `evcxr_runtime` crate to understand its value representation, execution model, and runtime services. This crate is likely to be a direct dependency of Oxur REPL, so your analysis should focus on API integration points and what functionality we need from it.

## Context: What is Oxur?

Oxur is a Lisp dialect that compiles to Rust with 100% interoperability. It has a unique architecture:

### Compilation Pipeline

1. **Parse** - Oxur syntax → Surface Forms (S-expressions with sugar/macros)
2. **Expand** - Surface Forms → Core Forms (canonical S-expressions, the IR)
3. **Lower** - Core Forms → Rust AST
4. **Generate** - Rust AST → Rust source code
5. **Compile** - Rust source → Binary (via rustc)

### REPL Evaluation Strategy

Oxur uses a **two-tier execution model**:

**Tier 1: Calculator Mode (Interpret)**

- Only interpret literal arithmetic: `(+ 1 2)`, `(* 3 4)`
- No variables, no side effects, no control flow
- Target: <1ms response time
- ~100 lines of code
- **Does NOT use evcxr_runtime** (pure Rust arithmetic)

**Tier 2: Cached Compilation (Compile Everything Else)**

- Variables, functions, IO, control flow - all compile through Rust
- First time: 50-200ms (full rustc compilation)
- Cached: ~0ms (reuse compiled dynamic library)
- **DOES use evcxr_runtime** for value representation and execution

### How Oxur Will Use evcxr_runtime

```rust
// Conceptual integration
pub struct CachedCompiler {
    cache: HashMap<CodeHash, CompiledCode>,
    runtime: evcxr_runtime::Runtime,  // ← We'll use this
}

impl CachedCompiler {
    pub async fn eval(&mut self, form: CoreForm) -> Result<Response> {
        // 1. Lower Core Forms → Rust AST
        let rust_ast = lower(form)?;

        // 2. Generate Rust source
        let rust_code = generate(&rust_ast)?;

        // 3. Compile to dynamic library
        let compiled = self.compile_dylib(&rust_code).await?;

        // 4. Execute with evcxr_runtime ← Key integration point
        let result = unsafe {
            compiled.call_with_runtime(&self.runtime)?
        };

        Ok(Response {
            value: Some(result.value),  // ← From evcxr_runtime
            out: Some(result.stdout),   // ← From evcxr_runtime
            err: Some(result.stderr),   // ← From evcxr_runtime
            status: vec![Status::Done],
        })
    }
}
```

### What We Need from evcxr_runtime

1. **Value Representation**: How to represent Rust values for REPL display
2. **Display/Debug**: Formatting values for user output
3. **Error Handling**: Catching and reporting runtime errors
4. **Output Capture**: Intercepting stdout/stderr from compiled code
5. **Type Erasure**: Handling different return types uniformly
6. **Memory Safety**: Safe handling of dynamically loaded code

## Your Specific Focus Areas

When auditing `evcxr_runtime`, pay special attention to:

### 1. Value Representation Internals

**Questions:**

- What types can evcxr_runtime represent?
- How are values serialized for display?
- How does it handle complex types (structs, enums, collections)?
- What's the API for creating/extracting values?
- How are generic types handled?

**Look for:**

- Core value types and their implementations
- Trait implementations (Display, Debug, etc.)
- Conversion methods between Rust types and runtime values
- Limitations on what can be represented

### 2. Display and Debug Formatting

**Questions:**

- How does evcxr_runtime pretty-print values?
- Can we customize display formats?
- How are different types formatted (numbers, strings, collections, custom types)?
- Is there REPL-specific formatting (e.g., truncation for large values)?

**Look for:**

- Display trait implementations
- Formatting utilities
- Configuration options for output format
- Handling of recursive/cyclic structures

### 3. Execution Model

**Questions:**

- How does evcxr_runtime actually execute compiled code?
- What's the interface between compiled code and runtime?
- How are function calls marshalled?
- How is the stack/heap managed?
- What safety guarantees exist?

**Look for:**

- Entry points for compiled code
- FFI boundaries and safety
- Memory management patterns
- Panic handling and recovery

### 4. Output Capture Implementation

**Questions:**

- How is stdout/stderr captured?
- Is capture per-thread or global?
- How does it handle async output?
- Can output be streamed or only buffered?
- What happens with output from spawned threads?

**Look for:**

- Output redirection mechanisms
- Buffer management
- Thread safety considerations
- Integration with standard I/O

### 5. Error Handling and Recovery

**Questions:**

- How are runtime errors caught and reported?
- How are panics handled?
- Can evaluation continue after an error?
- How is error context preserved?
- What information is available in error messages?

**Look for:**

- Error types and their hierarchy
- Panic catch mechanisms
- Backtrace/stack trace handling
- Error propagation patterns

### 6. Integration API Surface

**Questions:**

- What's the minimal API we need to use?
- What features are optional vs. required?
- How do we initialize the runtime?
- How do we pass values in/out?
- What cleanup is required?

**Look for:**

- Public API documentation
- Required initialization/teardown
- Ownership and lifetime considerations
- Feature flags and optional dependencies

## Deliverables

Please produce a markdown report at ./workbench/evcxr-runtime-audit-report.md with the following sections:

### 1. Executive Summary

- High-level overview of evcxr_runtime architecture
- 3-5 key capabilities it provides
- Our recommended integration approach

### 2. API Analysis

For each major API surface, provide:

**API Name**: Module or struct name

**Purpose**: What it's for (1 paragraph)

**Key Methods/Functions**: List with signatures

**Usage Example**:

```rust
// Concrete example showing how to use this API
let runtime = Runtime::new();
let value = runtime.eval(/* ... */)?;
println!("{}", value.display());
```

**Relevance to Oxur**:

- High: We'll definitely use this
- Medium: Might use depending on needs
- Low: Probably won't need

**Complexity**:

- Simple: Straightforward to use
- Moderate: Some learning curve
- Complex: Significant integration effort

**Priority**:

- P0: Must integrate for v1.0
- P1: Should integrate for v1.0
- P2: Nice to have for v1.0
- P3: Consider for v2.0+

**Integration Notes**: How we'd use this in Oxur REPL (1-2 paragraphs)

**Dependencies**: What this API requires (other crates, features, etc.)

---

### Example API Entry

**API Name**: `evcxr_runtime::Runtime`

**Purpose**:
The main runtime context that manages execution of compiled REPL code. Handles value representation, output capture, and error recovery. Provides a stable ABI for compiled code to call into.

**Key Methods/Functions**:

```rust
impl Runtime {
    pub fn new() -> Self;
    pub fn eval<T: EvalResult>(&mut self, code: CompiledCode) -> Result<T>;
    pub fn capture_output(&mut self) -> OutputGuard;
    pub fn last_output(&self) -> (String, String);
}
```

**Usage Example**:

```rust
use evcxr_runtime::{Runtime, EvalResult};

// Initialize runtime
let mut runtime = Runtime::new();

// Install output capture
let _guard = runtime.capture_output();

// Execute compiled code
let result: i32 = runtime.eval(compiled_code)?;

// Get captured output
let (stdout, stderr) = runtime.last_output();

println!("Result: {}", result);
println!("Output: {}", stdout);
```

**Relevance to Oxur**: High - This is our primary integration point

**Complexity**: Moderate - Need to understand safety contract and lifetime management

**Priority**: P0 - Essential for Tier 2 evaluation

**Integration Notes**:
We'll maintain one Runtime instance per REPL session in our `CachedCompiler`. Each evaluation will:

1. Install output capture
2. Load and call compiled code
3. Extract result and output
4. Return to user via protocol Response message

Need to ensure Runtime is thread-safe if we support concurrent evaluations (likely need one Runtime per session, not shared globally).

**Dependencies**:

- `libloading` for dynamic library loading
- Possibly nightly Rust features for output capture

---

### 3. Value Type System Analysis

Create a comprehensive map of evcxr_runtime's type system:

**Supported Primitive Types**:

- [ ] Integers (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128)
- [ ] Floats (f32, f64)
- [ ] Bool
- [ ] Char
- [ ] String, &str
- [ ] Unit ()

**Supported Compound Types**:

- [ ] Tuples
- [ ] Arrays
- [ ] Slices
- [ ] Vec<T>
- [ ] HashMap<K, V>
- [ ] Option<T>
- [ ] Result<T, E>
- [ ] Custom structs
- [ ] Custom enums

**Type Limitations**:

- What types CAN'T be represented?
- What are the size limits (e.g., max tuple elements)?
- Any restrictions on generic parameters?

**Display Formatting**:
For each type, note how it's formatted for REPL output

### 4. Memory and Safety Model

Document the safety guarantees and requirements:

**Safe Operations**:

- What can be done safely through the API?
- What guarantees does the runtime provide?

**Unsafe Operations**:

- Where is `unsafe` required?
- What invariants must we maintain?
- What can go wrong if misused?

**Memory Management**:

- Who owns values (runtime vs. compiled code)?
- How is cleanup handled?
- Are there memory leaks we need to worry about?

**Thread Safety**:

- Is Runtime Send + Sync?
- Can multiple threads evaluate concurrently?
- Any shared mutable state?

### 5. Integration Checklist

Create a concrete checklist for using evcxr_runtime in Oxur:

**Required Steps**:

- [ ] Add dependency: `evcxr_runtime = "X.Y.Z"`
- [ ] Initialize Runtime in CachedCompiler
- [ ] Modify code generation to call into runtime
- [ ] Handle value extraction and display
- [ ] Integrate output capture
- [ ] Implement error handling
- [ ] Test with compiled Oxur code

**Optional Enhancements**:

- [ ] Custom value display formatting
- [ ] Streaming output support
- [ ] Enhanced error messages
- [ ] Performance optimizations

### 6. Recommendations

#### What to Use Directly

List evcxr_runtime features we should use as-is

#### What to Wrap/Adapt

List features we should wrap in Oxur-specific abstractions

#### What to Replace

List features we should implement ourselves instead

#### What's Missing

List capabilities evcxr_runtime doesn't provide that we need

### 7. Code Examples

Provide 3-5 concrete, compilable examples showing:

1. **Basic value evaluation**

   ```rust
   // Complete, runnable example
   ```

2. **Output capture**

   ```rust
   // Complete, runnable example
   ```

3. **Error handling**

   ```rust
   // Complete, runnable example
   ```

4. **Complex type display**

   ```rust
   // Complete, runnable example
   ```

5. **Custom integration** (if applicable)

   ```rust
   // How we'd wrap evcxr_runtime for Oxur
   ```

### 8. Risk Assessment

**Technical Risks**:

- What could break during integration?
- What are the sharp edges in the API?
- Any version compatibility concerns?

**Performance Risks**:

- Any performance bottlenecks?
- Memory usage concerns?
- Overhead of runtime vs. direct execution?

**Maintenance Risks**:

- How stable is the API?
- Frequency of breaking changes?
- Quality of documentation?

### 9. Dependency Analysis

**Direct Dependencies**:
List what evcxr_runtime depends on and why it matters

**Version Constraints**:
Note any version requirements or conflicts

**Feature Flags**:
Document optional features and whether we need them

**Platform Support**:
Note any platform-specific code or limitations

### 10. Alternative Approaches

If evcxr_runtime isn't suitable, what are our options?

**Option 1: Use evcxr_runtime as-is**

- Pros: ...
- Cons: ...

**Option 2: Fork and customize evcxr_runtime**

- Pros: ...
- Cons: ...

**Option 3: Build our own runtime**

- Pros: ...
- Cons: ...

**Recommendation**: Which approach and why?

## Analysis Guidelines

### Do

- ✅ Test code examples to ensure they work
- ✅ Look at actual usage in evcxr_repl for real-world patterns
- ✅ Consider how this integrates with our two-tier evaluation model
- ✅ Think about session isolation (multiple Runtimes?)
- ✅ Note version numbers and stability concerns
- ✅ Identify minimal API surface we actually need

### Don't

- ❌ Just copy documentation without testing
- ❌ Assume APIs work without verifying
- ❌ Ignore safety requirements and constraints
- ❌ Overlook platform-specific considerations
- ❌ Focus on features we won't use

## Success Criteria

Your audit is successful if:

1. ✅ We know exactly which evcxr_runtime APIs to use
2. ✅ We have working code examples we can reference
3. ✅ We understand the safety contracts and requirements
4. ✅ We know what evcxr_runtime provides vs. what we build ourselves
5. ✅ We can write the integration code for Oxur REPL with confidence

## Repository Location

The evcxr repository has been cloned to your workspace. Focus your analysis on:

```
workbench/evcxr/
└── evcxr_runtime/       # Main focus of this audit
    ├── src/
    │   ├── lib.rs       # Public API
    │   ├── runtime.rs   # Runtime implementation
    │   └── ...
    ├── Cargo.toml       # Dependencies and features
    └── examples/        # Usage examples (if any)
```

Also look at how evcxr_repl uses evcxr_runtime for real-world patterns.

## Final Notes

evcxr_runtime is likely to be a **direct dependency** of Oxur REPL for Tier 2 evaluation. Your analysis should focus on practical integration: what we need, how to use it, and what could go wrong.

**Think like an integration engineer, not just a code reader.**

Good luck! 🦀
