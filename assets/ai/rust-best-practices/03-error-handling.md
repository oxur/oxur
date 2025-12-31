# Error Handling Guidelines

Comprehensive patterns for error handling in Rust, including Result, custom error types, and backtrace management.


## EH-01: `From` Implementations for Error Conversion

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

## EH-02: `Option` vs `Result` Decision

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

---

## EH-03: Applications May Use Anyhow or Derivatives

**Strength**: CONSIDER

**Summary**: Application crates (not libraries) may use anyhow, eyre, or similar for simplified error handling.

```rust
// For applications (binaries), this is acceptable:
use eyre::Result;

fn start_application() -> Result<()> {
    let config = load_config()?;  // Any error type works
    let db = connect_database(&config.db_url)?;  // Different error type
    start_server(config, db)?;  // Yet another error type
    Ok(())
}

// For libraries, use proper error types:
pub struct LibraryError { /* ... */ }

pub fn library_function() -> Result<Data, LibraryError> {
    // Never use anyhow/eyre in public library APIs
}
```

**Rationale**: Applications are the final layer—they don't need to expose their errors to callers. Error aggregation crates like anyhow provide convenient ergonomics for application logic.

**See also**: M-APP-ERROR

---

## EH-04: Avoid `unwrap()` and `expect()` in Libraries

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

---

## EH-05: Capture Backtraces When Creating Errors

**Strength**: MUST

**Summary**: Always capture a backtrace when constructing error instances, including in From implementations.

```rust
use std::backtrace::Backtrace;

pub struct DatabaseError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl DatabaseError {
    pub(crate) fn connection_failed(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Connection(err),
            backtrace: Backtrace::capture(),  // Capture here!
        }
    }
}

// Capture in From implementations too
impl From<std::io::Error> for DatabaseError {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
            backtrace: Backtrace::capture(),  // And here!
        }
    }
}

// Helper macro to reduce boilerplate
macro_rules! bail {
    ($kind:expr) => {
        return Err(DatabaseError {
            kind: $kind,
            backtrace: Backtrace::capture(),
        })
    };
}

fn process_query() -> Result<(), DatabaseError> {
    if condition_failed {
        bail!(ErrorKind::InvalidQuery);
    }
    Ok(())
}
```

**Rationale**: Backtraces are invaluable for debugging, especially in async code where errors travel through many stack frames. Capture is cheap when `RUST_BACKTRACE` is not set (just a few CPU instructions).

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

## EH-06: Consider a Private bail!() Macro

**Strength**: CONSIDER

**Summary**: For crates with many error sites, create a private `bail!()` macro to reduce boilerplate.

```rust
// Define macro in error module
macro_rules! bail {
    ($kind:expr) => {
        return Err($crate::error::MyError {
            kind: $kind,
            backtrace: std::backtrace::Backtrace::capture(),
        })
    };
}

pub(crate) use bail;

// Usage throughout crate
fn process_request(req: &Request) -> Result<Response, MyError> {
    if !req.is_valid() {
        bail!(ErrorKind::InvalidRequest);
    }
    
    let user = authenticate(req)?;
    if !user.has_permission() {
        bail!(ErrorKind::PermissionDenied {
            user: user.name.clone()
        });
    }
    
    Ok(Response::ok())
}
```

**Rationale**: Reduces error construction boilerplate while ensuring backtraces are always captured. Keeps error handling concise and consistent.

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

## EH-07: Define Custom Error Types for Libraries

**Strength**: SHOULD

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

---

## EH-08: Display Must Follow Rust Conventions

**Strength**: MUST

**Summary**: Error Display implementations should provide a summary sentence, then backtrace, then cause chain.

```rust
use std::backtrace::Backtrace;
use std::fmt;

pub struct ConfigError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1. Summary sentence
        match &self.kind {
            ErrorKind::NotFound(path) => {
                write!(f, "Configuration file not found: {}", path.display())?;
            }
            ErrorKind::ParseError { line, msg } => {
                write!(f, "Parse error at line {}: {}", line, msg)?;
            }
        }
        
        // 2. Backtrace (if available)
        if self.backtrace.status() == std::backtrace::BacktraceStatus::Captured {
            write!(f, "\n\n{}", self.backtrace)?;
        }
        
        Ok(())
    }
}

// Debug should include both Display and full backtrace
impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self, self.backtrace)
    }
}

impl std::error::Error for ConfigError {
    // Provide source for error chain
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::NotFound(e) => Some(e),
            _ => None,
        }
    }
}
```

**Rationale**: Consistent error formatting helps debugging. Display shows user-facing message; Debug includes full backtrace; source() enables error chain traversal.

**See also**: M-ERRORS-CANONICAL-STRUCTS, M-PUBLIC-DISPLAY

---

## EH-09: Document Error Conditions

**Strength**: MUST

**Summary**: Document when functions can return errors using an "Errors" section in rustdoc.

```rust
/// Loads configuration from a TOML file.
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Errors
///
/// This function will return an error if:
/// - The file does not exist or cannot be read
/// - The file contains invalid TOML syntax
/// - Required fields are missing from the configuration
///
/// # Examples
///
/// ```
/// use myapp::Config;
///
/// let config = Config::load("config.toml")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn load(path: &str) -> Result<Config, ConfigError> {
    // ...
}

/// Reads exactly `buf.len()` bytes from the reader.
///
/// # Errors
///
/// If this function encounters an EOF before filling the buffer,
/// it returns an error of kind `ErrorKind::UnexpectedEof`.
///
/// If any other I/O error is encountered, it is returned directly.
pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
    // ...
}

/// Parses a network address from a string.
///
/// # Errors
///
/// Returns `ParseError::InvalidFormat` if the input is not in the
/// format "host:port".
///
/// Returns `ParseError::InvalidPort` if the port number is not a
/// valid u16 or is in the reserved range (< 1024).
pub fn parse_address(s: &str) -> Result<SocketAddr, ParseError> {
    // ...
}
```

**Rationale**: Documenting error conditions helps users write correct error handling code and understand what can go wrong.

**See also**: C-FAILURE

---

## EH-10: Document Panic Conditions

**Strength**: MUST

**Summary**: Document when functions may panic using a "Panics" section in rustdoc.

```rust
/// Inserts an element at position `index` within the vector.
///
/// # Panics
///
/// Panics if `index > len`.
///
/// # Examples
///
/// ```
/// let mut vec = vec![1, 2, 3];
/// vec.insert(1, 4);
/// assert_eq!(vec, &[1, 4, 2, 3]);
/// ```
pub fn insert(&mut self, index: usize, element: T) {
    assert!(index <= self.len(), "index out of bounds");
    // ...
}

/// Returns the value at the given index, or panics.
///
/// # Panics
///
/// Panics if the index is out of bounds.
///
/// For a non-panicking alternative, see [`get`](#method.get).
pub fn index(&self, index: usize) -> &T {
    &self.data[index]
}

/// Divides two numbers.
///
/// # Panics
///
/// Panics if `divisor` is zero.
pub fn divide(dividend: f64, divisor: f64) -> f64 {
    assert!(divisor != 0.0, "division by zero");
    dividend / divisor
}
```

```rust
// Don't need to document this panic - it's from caller's code
pub fn with_formatter<F>(f: F) 
where 
    F: Fn(&str) -> String 
{
    let result = f("input");
    println!("{}", result);
}
```

**Rationale**: Panics are exceptional control flow that users need to be aware of to write robust code.

**See also**: C-FAILURE

---

## EH-11: Don't Expose ErrorKind Directly

**Strength**: MUST

**Summary**: Keep error enums private; expose helper methods to check error types.

```rust
// Bad - exposing enum directly
pub enum ErrorKind {
    Io(std::io::Error),
    Protocol,
}

pub struct HttpError {
    pub kind: ErrorKind,  // Public!
}

// Users write brittle code
match error.kind {
    ErrorKind::Io(_) => { /* ... */ }
    ErrorKind::Protocol => { /* ... */ }
    // Breaks when you add new variants!
}

// Good - hide enum, expose methods
pub struct HttpError {
    kind: ErrorKind,  // Private!
    backtrace: Backtrace,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Io(std::io::Error),
    Protocol,
}

impl HttpError {
    pub fn is_io(&self) -> bool {
        matches!(self.kind, ErrorKind::Io(_))
    }
    
    pub fn is_protocol(&self) -> bool {
        matches!(self.kind, ErrorKind::Protocol)
    }
    
    // Can add more variants without breaking callers!
}

// Usage - stable API
if error.is_io() {
    // Handle I/O errors
}
```

**Rationale**: Exposing error enums forces exhaustive matching in caller code, making adding new error variants a breaking change. Helper methods provide stable API.

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

## EH-12: Error Context and Chaining

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

---

## EH-13: Error Conversions Use From Trait

**Strength**: MUST

**Summary**: Implement `From<OtherError>` to enable `?` operator and error conversion chains.

```rust
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
    Custom(String),
}

// Implement From for automatic conversion
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::Io(err)
    }
}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> AppError {
        AppError::Parse(err)
    }
}

// Now ? operator works seamlessly
fn read_number_from_file(path: &str) -> Result<i32, AppError> {
    let contents = std::fs::read_to_string(path)?;  // io::Error → AppError
    let number = contents.trim().parse()?;          // ParseIntError → AppError
    Ok(number)
}

// Using thiserror crate (recommended)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),  // Automatically implements From
    
    #[error("parse error: {0}")]
    Parse(#[from] ParseIntError),
    
    #[error("{0}")]
    Custom(String),
}

// Same function works without manual From implementations
fn read_number_from_file(path: &str) -> Result<i32, AppError> {
    let contents = std::fs::read_to_string(path)?;
    let number = contents.trim().parse()?;
    Ok(number)
}
```

**Rationale**: `From` implementations enable the `?` operator to automatically convert between error types, making error propagation ergonomic.

**See also**: C-CONV-TRAITS

---

## EH-14: Error Handling in `main()`

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

## EH-15: Error Types Implement std::error::Error

**Strength**: MUST

**Summary**: All error types must implement the `Error` trait, `Display`, `Debug`, `Send`, and `Sync`.

```rust
use std::error::Error;
use std::fmt;

// Good - complete error type
#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError { line: usize, message: String },
    MissingField(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError { line, message } => {
                write!(f, "parse error at line {}: {}", line, message)
            }
            ConfigError::MissingField(field) => {
                write!(f, "missing required field: {}", field)
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

// Automatically Send + Sync if all fields are

// Good - unit struct error
#[derive(Debug, Clone, Copy)]
pub struct EmptyInputError;

impl fmt::Display for EmptyInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "input was empty")
    }
}

impl Error for EmptyInputError {}

// Bad - using () as error type
fn parse_config(s: &str) -> Result<Config, ()> {  // DON'T DO THIS
    // Problems:
    // - Can't use with ? in functions returning other errors
    // - No error message
    // - Can't use with error handling libraries
    // - Unhelpful to users
}

// Good - specific error type
fn parse_config(s: &str) -> Result<Config, ConfigError> {
    // Now users can:
    // - Pattern match on error variants
    // - Get useful error messages
    // - Use ? operator
    // - Chain with other Result types
}
```

```rust
// Minimum requirements for any error type
pub trait MyError: 
    Error +           // std::error::Error
    Debug +           // Debugging output
    Display +         // User-facing messages  
    Send +            // Can send across threads
    Sync              // Can share across threads
{}
```

**Rationale**: Following these conventions ensures errors work with error handling libraries like `anyhow`, `thiserror`, and `eyre`, and can be used in concurrent contexts.

**See also**: C-GOOD-ERR

---

## EH-16: Error Types Should Be Enum-Based

**Strength**: SHOULD

**Summary**: Use enums to represent different error cases, allowing callers to handle specific errors differently.

```rust
// Good - enum with variants for different cases
#[derive(Debug)]
pub enum DatabaseError {
    ConnectionFailed { 
        host: String, 
        port: u16, 
        source: std::io::Error 
    },
    QueryFailed { 
        query: String, 
        reason: String 
    },
    Timeout { 
        operation: String, 
        duration: std::time::Duration 
    },
    RecordNotFound { 
        table: String, 
        id: i64 
    },
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::ConnectionFailed { host, port, source } => {
                write!(f, "failed to connect to {}:{}: {}", host, port, source)
            }
            DatabaseError::QueryFailed { query, reason } => {
                write!(f, "query failed: {}: {}", query, reason)
            }
            DatabaseError::Timeout { operation, duration } => {
                write!(f, "operation '{}' timed out after {:?}", operation, duration)
            }
            DatabaseError::RecordNotFound { table, id } => {
                write!(f, "record {} not found in table '{}'", id, table)
            }
        }
    }
}

// Usage - caller can handle specific cases
match database.fetch_user(id) {
    Ok(user) => println!("Found user: {}", user.name),
    Err(DatabaseError::RecordNotFound { .. }) => {
        println!("User not found, creating new one");
        database.create_user(id)?;
    }
    Err(DatabaseError::Timeout { .. }) => {
        println!("Timeout, retrying...");
        database.fetch_user(id)?;
    }
    Err(e) => return Err(e),
}

// Bad - opaque error with only string
#[derive(Debug)]
pub struct DatabaseError {
    message: String,
}

// Caller can't distinguish between different error cases
// Can only examine the string message
```

```rust
#[derive(Debug)]
pub enum ParseError {
    // Struct-style for multiple fields
    InvalidFormat { 
        line: usize, 
        column: usize, 
        expected: String 
    },
    
    // Tuple-style for single wrapped error
    Io(std::io::Error),
    
    // Unit-style when no additional data needed
    UnexpectedEof,
}
```

**Rationale**: Enum-based errors enable pattern matching, provide structured data, and allow callers to handle different cases appropriately.

---

---

## EH-17: Errors Are Canonical Structs

**Strength**: MUST

**Summary**: Errors should be situation-specific structs containing a Backtrace, upstream cause, and helper methods—not bare enums.

```rust
use std::backtrace::Backtrace;

// Bad - bare enum, no backtrace or context
pub enum ConfigError {
    NotFound,
    ParseError,
    InvalidFormat,
}

// Good - canonical struct with full context
pub struct ConfigError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    NotFound(std::io::Error),
    ParseError(toml::de::Error),
    InvalidFormat { field: String, reason: String },
}

impl ConfigError {
    pub(crate) fn not_found(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::NotFound(err),
            backtrace: Backtrace::capture(),
        }
    }
    
    pub(crate) fn parse_error(err: toml::de::Error) -> Self {
        Self {
            kind: ErrorKind::ParseError(err),
            backtrace: Backtrace::capture(),
        }
    }
    
    pub(crate) fn invalid_format(field: String, reason: String) -> Self {
        Self {
            kind: ErrorKind::InvalidFormat { field, reason },
            backtrace: Backtrace::capture(),
        }
    }
    
    // Public helper methods for error inspection
    pub fn is_not_found(&self) -> bool {
        matches!(self.kind, ErrorKind::NotFound(_))
    }
    
    pub fn is_parse_error(&self) -> bool {
        matches!(self.kind, ErrorKind::ParseError(_))
    }
}

impl std::fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self, self.backtrace)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::NotFound(e) => 
                write!(f, "Configuration file not found: {}", e),
            ErrorKind::ParseError(e) => 
                write!(f, "Failed to parse configuration: {}", e),
            ErrorKind::InvalidFormat { field, reason } =>
                write!(f, "Invalid configuration field '{}': {}", field, reason),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::NotFound(e) => Some(e),
            ErrorKind::ParseError(e) => Some(e),
            ErrorKind::InvalidFormat { .. } => None,
        }
    }
}

// Implement From for upstream errors
impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        Self::not_found(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::parse_error(err)
    }
}
```

**Rationale**: Struct-based errors provide comprehensive debugging information. The internal `ErrorKind` enum groups failure modes while keeping future-proofing (callers use `is_xxx()` methods, not exhaustive matches).

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

## EH-18: Errors Expose Source Chain

**Strength**: SHOULD

**Summary**: Implement `Error::source()` to expose the underlying cause of the error.

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ConfigLoad { source: std::io::Error },
    ConfigParse { source: toml::de::Error },
    DatabaseConnect { url: String, source: sqlx::Error },
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::ConfigLoad { source } => Some(source),
            AppError::ConfigParse { source } => Some(source),
            AppError::DatabaseConnect { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::ConfigLoad { .. } => {
                write!(f, "failed to load configuration file")
            }
            AppError::ConfigParse { .. } => {
                write!(f, "failed to parse configuration")
            }
            AppError::DatabaseConnect { url, .. } => {
                write!(f, "failed to connect to database at {}", url)
            }
        }
    }
}

// Usage - can walk the error chain
fn print_error_chain(mut err: &dyn Error) {
    eprintln!("Error: {}", err);
    while let Some(source) = err.source() {
        eprintln!("  Caused by: {}", source);
        err = source;
    }
}

// With thiserror, this is automatic
#[derive(Error, Debug)]
pub enum AppError {
    #[error("failed to load configuration file")]
    ConfigLoad {
        #[source]  // Automatically implements source()
        source: std::io::Error,
    },
    
    #[error("failed to parse configuration")]
    ConfigParse {
        #[source]
        source: toml::de::Error,
    },
}
```

**Rationale**: Exposing the source chain helps with debugging by preserving the full context of what went wrong.

---

---

## EH-19: Fallible Constructors

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

## EH-20: Implement `std::error::Error` for Custom Errors

**Strength**: MUST

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

---

## EH-21: Prefer Result Over Option for Errors

**Strength**: SHOULD

**Summary**: Use `Result` with meaningful errors rather than `Option` when an operation can fail for multiple reasons.

```rust
// Good - Result with informative error
fn parse_port(s: &str) -> Result<u16, ParsePortError> {
    let num: u16 = s.parse()
        .map_err(|_| ParsePortError::InvalidFormat)?;
    
    if num < 1024 {
        return Err(ParsePortError::ReservedPort(num));
    }
    
    Ok(num)
}

#[derive(Debug)]
pub enum ParsePortError {
    InvalidFormat,
    ReservedPort(u16),
}

// Bad - Option loses information
fn parse_port_bad(s: &str) -> Option<u16> {
    let num: u16 = s.parse().ok()?;
    if num < 1024 {
        return None;  // Why did it fail? Who knows!
    }
    Some(num)
}

// Good - Option when there's truly only one failure mode
fn first_word(s: &str) -> Option<&str> {
    s.split_whitespace().next()
    // Only fails if string is empty - Option is appropriate
}

// Good - Result when multiple things can go wrong
fn connect_to_database(url: &str) -> Result<Connection, DbError> {
    // Can fail for many reasons:
    // - Invalid URL format
    // - Network unreachable
    // - Authentication failed
    // - Database doesn't exist
    // Using Result lets us distinguish these
}
```

**Rationale**: `Result` forces callers to think about error cases and provides information about what went wrong.

---

---

## EH-22: Provide Contextual Helper Methods

**Strength**: SHOULD

**Summary**: Error types should provide methods to access contextual information.

```rust
pub struct ConfigError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

enum ErrorKind {
    NotFound { path: PathBuf },
    ParseError { path: PathBuf, line: usize },
}

impl ConfigError {
    // Type checking methods
    pub fn is_not_found(&self) -> bool {
        matches!(self.kind, ErrorKind::NotFound { .. })
    }
    
    // Context accessors
    pub fn config_path(&self) -> Option<&Path> {
        match &self.kind {
            ErrorKind::NotFound { path } => Some(path),
            ErrorKind::ParseError { path, .. } => Some(path),
        }
    }
    
    pub fn line_number(&self) -> Option<usize> {
        match &self.kind {
            ErrorKind::ParseError { line, .. } => Some(*line),
            _ => None,
        }
    }
}

// Usage
match load_config() {
    Err(e) if e.is_not_found() => {
        println!("Config not found at {:?}", e.config_path());
    }
    Err(e) => {
        if let Some(line) = e.line_number() {
            println!("Parse error at line {}", line);
        }
    }
    Ok(config) => { /* ... */ }
}
```

**Rationale**: Helper methods provide stable access to error context without exposing internal structure. Enables better error handling and logging.

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

## EH-23: Provide Fallible and Infallible Variants

**Strength**: CONSIDER

**Summary**: For operations that might panic, consider providing both panicking and `Result`-returning variants.

```rust
// Panicking variant
impl<T> Vec<T> {
    /// Removes and returns the element at position `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len());
        // ...
    }
}

// Could provide Result variant (though std doesn't)
impl<T> Vec<T> {
    /// Attempts to remove and return the element at position `index`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is out of bounds.
    pub fn try_remove(&mut self, index: usize) -> Result<T, IndexError> {
        if index < self.len() {
            Ok(self.remove(index))
        } else {
            Err(IndexError { index, len: self.len() })
        }
    }
}

// Real example from std - slice indexing
impl<T> [T] {
    // Panicking version
    pub fn get_unchecked(&self, index: usize) -> &T {
        unsafe { &*self.as_ptr().add(index) }
    }
    
    // Result version
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(&self[index])
        } else {
            None
        }
    }
}
```

**Rationale**: Provides ergonomics for cases where panicking is acceptable while still supporting error handling when needed.

---

## EH-24: Sensitive Data in Errors

**Strength**: MUST

**Summary**: Error types containing sensitive data must implement custom Display/Debug that redacts secrets.

```rust
pub struct AuthError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

enum ErrorKind {
    InvalidToken { token: String },
    InvalidPassword { username: String, password: String },
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::InvalidToken { .. } => {
                write!(f, "Invalid authentication token")
                // Don't include the actual token!
            }
            ErrorKind::InvalidPassword { username, .. } => {
                write!(f, "Invalid password for user '{}'", username)
                // Don't include the password!
            }
        }
    }
}

impl std::fmt::Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Debug also redacts sensitive data
        write!(f, "{}\n{}", self, self.backtrace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sensitive_data_not_leaked() {
        let password = "super_secret_123";
        let error = AuthError::invalid_password(
            "alice".to_string(),
            password.to_string(),
        );
        
        let display = format!("{}", error);
        let debug = format!("{:?}", error);
        
        assert!(!display.contains(password));
        assert!(!debug.contains(password));
    }
}
```

**Rationale**: Errors often end up in logs. Leaking passwords, tokens, or API keys in error messages creates security vulnerabilities.

**See also**: M-PUBLIC-DEBUG, M-PUBLIC-DISPLAY

---

## EH-25: Separate Error Types for Different Contexts

**Strength**: SHOULD

**Summary**: Create distinct error types for different API contexts rather than one global error enum.

```rust
// Bad - one error to rule them all
pub enum GlobalError {
    // Download errors
    NetworkTimeout,
    InvalidUrl,
    
    // VM errors  
    VmStartFailed,
    OutOfMemory,
    
    // Config errors
    ConfigNotFound,
    ParseError,
}

pub fn download_iso() -> Result<(), GlobalError> { /* ... */ }
pub fn start_vm() -> Result<(), GlobalError> { /* ... */ }

// Good - separate error types per context
pub struct DownloadError { /* ... */ }
pub struct VmError { /* ... */ }
pub struct ConfigError { /* ... */ }

pub fn download_iso() -> Result<(), DownloadError> { /* ... */ }
pub fn start_vm() -> Result<(), VmError> { /* ... */ }
pub fn load_config() -> Result<Config, ConfigError> { /* ... */ }

// Related contexts can share error types
pub struct ParseError { /* ... */ }

pub fn parse_json(s: &str) -> Result<Value, ParseError> { /* ... */ }
pub fn parse_toml(s: &str) -> Result<Value, ParseError> { /* ... */ }
```

**Rationale**: Context-specific errors are more precise and maintainable. Global error enums grow unwieldy and mix unrelated failure modes. Error types should be general enough to be reused but specific enough to be meaningful.

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

## EH-26: The `?` Operator for Propagation

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

---

## EH-27: The Full Error Template

**Strength**: SHOULD

**Summary**: 

```rust
use std::backtrace::Backtrace;
use std::fmt;

pub struct MyError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Variant1(UpstreamError),
    Variant2 { field: String },
}

impl MyError {
    pub(crate) fn variant1(err: UpstreamError) -> Self {
        Self {
            kind: ErrorKind::Variant1(err),
            backtrace: Backtrace::capture(),
        }
    }
    
    pub fn is_variant1(&self) -> bool {
        matches!(self.kind, ErrorKind::Variant1(_))
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Variant1(e) => write!(f, "Description: {}", e),
            ErrorKind::Variant2 { field } => write!(f, "Description: {}", field),
        }
    }
}

impl fmt::Debug for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self, self.backtrace)
    }
}

impl std::error::Error for MyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Variant1(e) => Some(e),
            _ => None,
        }
    }
}

impl From<UpstreamError> for MyError {
    fn from(err: UpstreamError) -> Self {
        Self::variant1(err)
    }
}
```

---

## EH-28: Use ? Operator for Propagation

**Strength**: MUST

**Summary**: Use `?` to propagate errors, avoid explicit match or `unwrap()` in library code.

```rust
// Good - using ? operator
fn load_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    let config = toml::from_str(&contents)?;
    Ok(config)
}

// Bad - explicit match
fn load_config_bad(path: &str) -> Result<Config, ConfigError> {
    let contents = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(ConfigError::from(e)),
    };
    let config = match toml::from_str(&contents) {
        Ok(c) => c,
        Err(e) => return Err(ConfigError::from(e)),
    };
    Ok(config)
}

// Bad - unwrap in library code
fn load_config_worse(path: &str) -> Config {
    let contents = std::fs::read_to_string(path).unwrap();  // DON'T DO THIS
    toml::from_str(&contents).unwrap()
}

// Good - ? with Option
fn find_user(users: &[User], id: UserId) -> Option<&User> {
    let user = users.iter().find(|u| u.id == id)?;
    Some(user)
}

// Good - converting Option to Result
fn get_user(users: &[User], id: UserId) -> Result<&User, Error> {
    users
        .iter()
        .find(|u| u.id == id)
        .ok_or(Error::UserNotFound(id))
}
```

```rust
// OK - in tests
#[test]
fn test_parse() {
    let result = parse_input("valid").unwrap();
    assert_eq!(result, expected);
}

// OK - with documented justification
fn process_ascii(s: &str) -> &str {
    // SAFETY: Input is guaranteed to be ASCII by type system
    // (assuming we have a newtype that enforces this)
    std::str::from_utf8(&s.as_bytes()[0..10]).unwrap()
}

// Good - use expect() with message in non-library code
fn main() {
    let config = load_config("config.toml")
        .expect("Failed to load config.toml");
}
```

**Rationale**: The `?` operator makes error handling concise and clear, while preserving the error chain.

---

---

## EH-29: Use `#[must_use]` on Error-Returning Functions

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

---

## EH-30: Use `anyhow` for Application Error Handling

**Strength**: SHOULD

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

---

## EH-31: Use `Result` for Recoverable Errors, `panic!` for Bugs

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

---

## EH-32: Use ok_or/ok_or_else to Convert Option to Result

**Strength**: SHOULD

**Summary**: When converting `Option` to `Result`, use `ok_or` or `ok_or_else` rather than matching.

```rust
// Good - using ok_or
fn get_config_value(key: &str) -> Result<String, ConfigError> {
    std::env::var(key)
        .ok()  // Result → Option
        .ok_or(ConfigError::MissingKey(key.to_string()))
}

// Good - using ok_or_else for lazy evaluation
fn get_user(id: UserId, db: &Database) -> Result<User, AppError> {
    db.users
        .get(&id)
        .cloned()
        .ok_or_else(|| AppError::UserNotFound { 
            id, 
            searched_at: Utc::now() 
        })
}

// Bad - explicit match
fn get_config_value_bad(key: &str) -> Result<String, ConfigError> {
    match std::env::var(key).ok() {
        Some(v) => Ok(v),
        None => Err(ConfigError::MissingKey(key.to_string())),
    }
}

// Good - chaining conversions
fn parse_env_port() -> Result<u16, AppError> {
    std::env::var("PORT")
        .ok()
        .ok_or(AppError::MissingEnvVar("PORT"))?
        .parse()
        .map_err(|_| AppError::InvalidPort)
}
```

```rust
// ok_or - error is simple
option.ok_or(404)

// ok_or_else - error involves allocation or computation  
option.ok_or_else(|| format!("Not found: {}", id))
option.ok_or_else(|| Error::new(Utc::now()))
```

**Rationale**: These methods are more concise and idiomatic than explicit matching.

---

---

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| Errors are canonical structs | MUST | Backtrace + internal enum + helpers |
| Don't expose ErrorKind | MUST | Provide is_xxx() methods instead |
| Separate error types per context | SHOULD | Avoid global error enums |
| Capture backtraces | MUST | In constructors and From impls |
| Contextual helper methods | SHOULD | Stable access to error details |
| Anyhow for applications only | CONSIDER | Never in libraries |
| Display follows conventions | MUST | Summary + backtrace + cause |
| Redact sensitive data | MUST | Test that secrets don't leak |
| Consider bail!() macro | CONSIDER | For crates with many error sites |

## Common Patterns

### The Full Error Template

```rust
use std::backtrace::Backtrace;
use std::fmt;

pub struct MyError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Variant1(UpstreamError),
    Variant2 { field: String },
}

impl MyError {
    pub(crate) fn variant1(err: UpstreamError) -> Self {
        Self {
            kind: ErrorKind::Variant1(err),
            backtrace: Backtrace::capture(),
        }
    }
    
    pub fn is_variant1(&self) -> bool {
        matches!(self.kind, ErrorKind::Variant1(_))
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Variant1(e) => write!(f, "Description: {}", e),
            ErrorKind::Variant2 { field } => write!(f, "Description: {}", field),
        }
    }
}

impl fmt::Debug for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self, self.backtrace)
    }
}

impl std::error::Error for MyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Variant1(e) => Some(e),
            _ => None,
        }
    }
}

impl From<UpstreamError> for MyError {
    fn from(err: UpstreamError) -> Self {
        Self::variant1(err)
    }
}
```

## Related Guidelines

- **Core Idioms**: See `01-core-idioms.md` for panic vs Result
- **API Design**: See `02-api-design.md` for Result in APIs
- **Documentation**: See `13-documentation.md` for documenting errors

## External References

- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html)
- Pragmatic Rust: M-ERRORS-CANONICAL-STRUCTS, M-APP-ERROR