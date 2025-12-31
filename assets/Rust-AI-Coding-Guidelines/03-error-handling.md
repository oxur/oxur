# Error Handling in Rust

> Patterns for Result, Option, custom errors, and propagation strategies.

---

## EH-01: Use `Result` for Recoverable Errors, `panic!` for Bugs

**Strength**: MUST

**Summary**: `Result` for expected failures, `panic!` only for programming errors.

```rust
// ✅ CORRECT: File not found is expected, use Result
fn read_config(path: &Path) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    toml::from_str(&contents).map_err(ConfigError::Parse)
}

// ✅ CORRECT: Invalid index is a bug, panic is appropriate
fn get_unchecked(slice: &[i32], index: usize) -> i32 {
    assert!(index < slice.len(), "index out of bounds: bug in caller");
    slice[index]
}

// ❌ WRONG: Panicking on expected error
fn read_config(path: &Path) -> Config {
    let contents = std::fs::read_to_string(path)
        .expect("config file must exist");  // User might not have it!
    toml::from_str(&contents).unwrap()
}

// ❌ WRONG: Result for invariant violation
fn process(data: &[i32]) -> Result<i32, Error> {
    if data.is_empty() {
        // This is a bug in the caller, not a runtime error
        return Err(Error::EmptyData);  // Should be panic or assert
    }
    Ok(data[0])
}
```

**Guidelines**:
- User input → `Result` (users make mistakes)
- File/network I/O → `Result` (external systems fail)
- Internal invariants → `panic!`/`assert!` (these are bugs)
- Out of memory → `panic!` (usually unrecoverable)

---

## EH-02: The `?` Operator for Propagation

**Strength**: MUST

**Summary**: Use `?` to propagate errors up the call stack.

```rust
// ❌ VERBOSE: Manual propagation
fn fetch_user(id: i32) -> Result<User, Error> {
    let response = match http_client.get(url) {
        Ok(r) => r,
        Err(e) => return Err(e.into()),
    };
    let body = match response.text() {
        Ok(b) => b,
        Err(e) => return Err(e.into()),
    };
    match serde_json::from_str(&body) {
        Ok(user) => Ok(user),
        Err(e) => Err(e.into()),
    }
}

// ✅ CONCISE: ? operator
fn fetch_user(id: i32) -> Result<User, Error> {
    let response = http_client.get(url)?;
    let body = response.text()?;
    let user = serde_json::from_str(&body)?;
    Ok(user)
}

// ✅ EVEN MORE CONCISE: Chained
fn fetch_user(id: i32) -> Result<User, Error> {
    Ok(serde_json::from_str(&http_client.get(url)?.text()?)?)
}
```

**How `?` works**:
- On `Ok(v)` → unwraps to `v`
- On `Err(e)` → returns `Err(e.into())` (note the automatic conversion)

---

## EH-03: Define Custom Error Types for Libraries

**Strength**: SHOULD (for libraries)

**Summary**: Libraries should define their own error types, not use `Box<dyn Error>`.

```rust
// ❌ OPAQUE: Users can't match on error kinds
pub fn parse(input: &str) -> Result<Ast, Box<dyn std::error::Error>> {
    // ...
}

// ✅ GOOD: Using thiserror for custom errors
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token '{found}' at position {position}, expected {expected}")]
    UnexpectedToken {
        found: String,
        expected: String,
        position: usize,
    },
    
    #[error("unexpected end of input")]
    UnexpectedEof,
    
    #[error("invalid number: {0}")]
    InvalidNumber(#[from] std::num::ParseIntError),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn parse(input: &str) -> Result<Ast, ParseError> {
    // ...
}

// Users can now match on specific errors:
match parse(input) {
    Ok(ast) => process(ast),
    Err(ParseError::UnexpectedEof) => eprintln!("Input incomplete"),
    Err(ParseError::InvalidNumber(e)) => eprintln!("Bad number: {e}"),
    Err(e) => eprintln!("Parse failed: {e}"),
}
```

**Using `thiserror`**:
- `#[error("...")]` — Implements `Display`
- `#[from]` — Implements `From` for automatic conversion with `?`
- `#[source]` — Marks the underlying cause (for error chains)

---

## EH-04: Use `anyhow` for Application Error Handling

**Strength**: SHOULD (for applications)

**Summary**: Applications can use `anyhow::Result` for convenient error handling.

```rust
use anyhow::{Context, Result, bail, ensure};

fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;
    
    run_server(config)
        .context("Server crashed")?;
    
    Ok(())
}

fn load_config() -> Result<Config> {
    let path = std::env::var("CONFIG_PATH")
        .context("CONFIG_PATH not set")?;
    
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {path}"))?;
    
    // bail! for early return with error
    if contents.is_empty() {
        bail!("Config file is empty");
    }
    
    // ensure! for assertions that return errors
    ensure!(contents.len() < 1_000_000, "Config file too large");
    
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}
```

**Key `anyhow` features**:
- `Result<T>` = `Result<T, anyhow::Error>`
- `.context("msg")` — Add context to errors
- `.with_context(|| format!(...))` — Lazy context (when formatting is expensive)
- `bail!("msg")` — Return error immediately
- `ensure!(condition, "msg")` — Assert with error return

**When to use which**:
- **Library**: `thiserror` with custom error type
- **Application**: `anyhow` for convenience
- **Boundary**: Convert library errors with `.context()` at call sites

---

## EH-05: Error Context and Chaining

**Strength**: SHOULD

**Summary**: Add context to errors as they propagate up.

```rust
// ❌ BAD: Raw error with no context
fn process_file(path: &Path) -> Result<Data, std::io::Error> {
    let contents = std::fs::read_to_string(path)?;  // "No such file"
    // User has no idea WHICH file
    todo!()
}

// ✅ GOOD: Error with context
fn process_file(path: &Path) -> Result<Data, anyhow::Error> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    // Error: "Failed to read /etc/config.toml: No such file or directory"
    todo!()
}

// ✅ GOOD: With thiserror, wrap errors
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("failed to read {path}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    // ...
}

fn process_file(path: &Path) -> Result<Data, ProcessError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|source| ProcessError::ReadFile { 
            path: path.to_owned(), 
            source 
        })?;
    todo!()
}
```

**Context guideline**: Each layer should add information about what it was *trying* to do, not just what failed.

---

## EH-06: `Option` vs `Result` Decision

**Strength**: SHOULD

**Summary**: Use `Option` for "not found", `Result` for "something went wrong".

```rust
// ✅ Option: Absence is normal, not an error
fn find_user(id: UserId) -> Option<User> {
    users.get(&id).cloned()
}

// ✅ Result: Failure needs explanation
fn load_user(id: UserId) -> Result<User, UserError> {
    let row = db.query("SELECT * FROM users WHERE id = ?", &[&id])
        .map_err(UserError::Database)?;
    
    match row {
        Some(r) => Ok(parse_user(r)?),
        None => Err(UserError::NotFound(id)),
    }
}

// Converting between them:
fn get_or_error(opt: Option<T>) -> Result<T, MyError> {
    opt.ok_or(MyError::NotFound)
}

fn get_or_none(result: Result<T, E>) -> Option<T> {
    result.ok()
}
```

**Decision tree**:
1. Is absence a normal case? → `Option`
2. Does the caller need to know *why* it failed? → `Result`
3. Are there multiple failure modes? → `Result` with enum

---

## EH-07: Avoid `unwrap()` and `expect()` in Libraries

**Strength**: SHOULD

**Summary**: Libraries should propagate errors, not panic.

```rust
// ❌ BAD: Library panics on error
pub fn parse_config(s: &str) -> Config {
    serde_json::from_str(s).unwrap()  // Panics on invalid JSON!
}

// ✅ GOOD: Library returns Result
pub fn parse_config(s: &str) -> Result<Config, ConfigError> {
    serde_json::from_str(s).map_err(ConfigError::Json)
}

// ✅ ACCEPTABLE: expect() with proof it can't fail
pub fn compile_regex() -> Regex {
    // This regex is a literal, so we know it's valid
    Regex::new(r"^\d{4}-\d{2}-\d{2}$")
        .expect("hardcoded regex is valid")
}

// ✅ ACCEPTABLE: expect() with clear precondition
impl MyVec<T> {
    /// Returns the first element.
    /// 
    /// # Panics
    /// Panics if the vector is empty.
    pub fn first(&self) -> &T {
        self.data.first().expect("MyVec is never empty")
    }
}
```

**When `expect()` is OK**:
- Hardcoded values that are statically known to be valid
- After checking a precondition (document the panic)
- In tests

---

## EH-08: Implement `std::error::Error` for Custom Errors

**Strength**: MUST (for error types)

**Summary**: Custom error types must implement `Error` to work with `?` and error handling ecosystem.

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct MyError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

// Much easier with thiserror:
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct MyError {
    message: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
}
```

**Requirements for `Error`**:
- `Debug` (required by Error trait)
- `Display` (required by Error trait)
- `source()` method (optional but recommended for chaining)

---

## EH-09: Use `#[must_use]` on Error-Returning Functions

**Strength**: SHOULD

**Summary**: Mark functions whose return value should not be ignored.

```rust
// ✅ GOOD: Compiler warns if Result is ignored
#[must_use]
pub fn save(&self) -> Result<(), SaveError> {
    // ...
}

// Usage:
config.save();  // WARNING: unused Result that must be used

// ✅ GOOD: On the Result type itself (already done in std)
#[must_use = "this `Result` may be an `Err` variant, which should be handled"]
pub enum Result<T, E> { ... }

// ✅ GOOD: Custom message
#[must_use = "this returns a new string and does not modify the original"]
pub fn to_uppercase(&self) -> String { ... }
```

**Rationale**: Prevents silent failures where errors are accidentally ignored.

---

## EH-10: Error Handling in `main()`

**Strength**: SHOULD

**Summary**: Return `Result` from `main()` for proper error reporting.

```rust
// ❌ POOR: Manual error handling
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

// ✅ GOOD: Return Result from main
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    run_server(config)?;
    Ok(())
}

// ✅ BETTER: With anyhow for nice error display
fn main() -> anyhow::Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;
    run_server(config)?;
    Ok(())
}

// ✅ BEST: Custom error handling with exit codes
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:#}");  // {:#} for full error chain
            ExitCode::FAILURE
        }
    }
}
```

---

## EH-11: `From` Implementations for Error Conversion

**Strength**: SHOULD

**Summary**: Implement `From` to enable automatic error conversion with `?`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),  // #[from] generates From impl
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

// Now ? automatically converts:
fn fetch_data() -> Result<Data, AppError> {
    let response = reqwest::blocking::get(url)?;  // Http variant
    let text = std::fs::read_to_string(path)?;    // Io variant
    let data = serde_json::from_str(&text)?;      // Json variant
    Ok(data)
}

// Manual From implementation:
impl From<CustomError> for AppError {
    fn from(e: CustomError) -> Self {
        AppError::Custom { 
            message: e.message,
            code: e.code,
        }
    }
}
```

---

## EH-12: Fallible Constructors

**Strength**: SHOULD

**Summary**: Use `new() -> Result<Self, Error>` or `try_new()` for fallible construction.

```rust
// ✅ OPTION A: new() returns Result (when construction usually can fail)
pub struct Config {
    // ...
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let parsed = toml::from_str(&contents)?;
        Ok(Self { /* ... */ })
    }
}

// ✅ OPTION B: try_new() for fallible, new() for infallible
pub struct PositiveInt(i32);

impl PositiveInt {
    /// Creates a new PositiveInt.
    /// 
    /// # Panics
    /// Panics if `value <= 0`.
    pub fn new(value: i32) -> Self {
        Self::try_new(value).expect("value must be positive")
    }
    
    /// Creates a new PositiveInt, returning `None` if value is not positive.
    pub fn try_new(value: i32) -> Option<Self> {
        if value > 0 {
            Some(Self(value))
        } else {
            None
        }
    }
}

// ✅ OPTION C: FromStr for parsing
impl std::str::FromStr for Config {
    type Err = ConfigError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(ConfigError::Parse)
    }
}
```

---

## Summary: Error Handling Decision Tree

```
Is it a programming error (bug)?
├─ Yes → panic! / assert! / unreachable!
└─ No → Result<T, E>
         │
         Is this a library?
         ├─ Yes → Custom error type (thiserror)
         │        └─ Implement Error, Display, Debug
         │        └─ Provide From impls for common sources
         │        └─ Don't use anyhow in public API
         └─ No (application) → anyhow::Result
                  └─ Add .context() for clarity
                  └─ Use bail!/ensure! for early returns
```

**Crate recommendations**:
- **Libraries**: `thiserror` for custom error types
- **Applications**: `anyhow` for convenient error handling
- **Both**: Standard `Result` and `?` for propagation

---

*See also: [11-anti-patterns.md](11-anti-patterns.md) for error handling anti-patterns.*
