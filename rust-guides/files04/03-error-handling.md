# Error Handling Guidelines

Comprehensive patterns for error handling in Rust, including Result, custom error types, and backtrace management.

## Table of Contents

- [Error Type Design](#error-type-design)
- [Error Context](#error-context)
- [Application-Level Errors](#application-level-errors)
- [Display Implementation](#display-implementation)
- [Helper Macros](#helper-macros)

---

## Error Type Design

### Errors Are Canonical Structs

**Strength**: MUST

**Summary**: Errors should be situation-specific structs containing a Backtrace, upstream cause, and helper methods—not bare enums.

**Example**:
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

**Required elements**:
- `Backtrace` field (captured when error is created)
- Internal `ErrorKind` enum (not public)
- Public `is_xxx()` helper methods
- Proper `Display` and `Error` implementations
- `From` impls for upstream errors

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

### Don't Expose ErrorKind Directly

**Strength**: MUST

**Summary**: Keep error enums private; expose helper methods to check error types.

**Example**:
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

### Separate Error Types for Different Contexts

**Strength**: SHOULD

**Summary**: Create distinct error types for different API contexts rather than one global error enum.

**Example**:
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

## Error Context

### Capture Backtraces When Creating Errors

**Strength**: MUST

**Summary**: Always capture a backtrace when constructing error instances, including in From implementations.

**Example**:
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

**When you get backtraces**:
- Backtraces are a development tool, not runtime diagnostic
- By default `Backtrace::capture()` returns empty (very cheap)
- Set `RUST_BACKTRACE=1` to enable capture
- Overhead when enabled: ~4μs per capture (based on typical hardware)

**See also**: M-ERRORS-CANONICAL-STRUCTS

---

### Provide Contextual Helper Methods

**Strength**: SHOULD

**Summary**: Error types should provide methods to access contextual information.

**Example**:
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

## Application-Level Errors

### Applications May Use Anyhow or Derivatives

**Strength**: CONSIDER

**Summary**: Application crates (not libraries) may use anyhow, eyre, or similar for simplified error handling.

**Example**:
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

**Rules**:
- Only use in application crates and internal application modules
- Never use in library crates (crates used by other crates)
- Don't mix multiple application error crates (pick one)
- Re-export common Result: `pub type Result<T> = eyre::Result<T>;`

**See also**: M-APP-ERROR

---

## Display Implementation

### Display Must Follow Rust Conventions

**Strength**: MUST

**Summary**: Error Display implementations should provide a summary sentence, then backtrace, then cause chain.

**Example**:
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

**Display format**:
1. Summary sentence (what happened)
2. Backtrace (if captured and available)
3. Cause information (via source() chain)

**See also**: M-ERRORS-CANONICAL-STRUCTS, M-PUBLIC-DISPLAY

---

### Sensitive Data in Errors

**Strength**: MUST

**Summary**: Error types containing sensitive data must implement custom Display/Debug that redacts secrets.

**Example**:
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

## Helper Macros

### Consider a Private bail!() Macro

**Strength**: CONSIDER

**Summary**: For crates with many error sites, create a private `bail!()` macro to reduce boilerplate.

**Example**:
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

**Alternative**: Use helper methods on the error type itself.

**See also**: M-ERRORS-CANONICAL-STRUCTS

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
