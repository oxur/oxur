# Error Handling

Comprehensive guidelines for designing error types, handling failures, and using Result/Option effectively.

## Error Type Design

### Error Types Implement std::error::Error

**Strength**: MUST

**Summary**: All error types must implement the `Error` trait, `Display`, `Debug`, `Send`, and `Sync`.

**Examples**:

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

**Required trait implementations**:

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

**Error message guidelines**:
- Lowercase without trailing punctuation
- Be specific and actionable
- Include context when available

Good examples:
- ✅ `"unexpected end of file"`
- ✅ `"invalid IP address syntax"`
- ✅ `"environment variable 'PATH' was not valid unicode"`
- ✅ `"second time provided was later than self"`

Bad examples:
- ❌ `"Error"` (too vague)
- ❌ `"An error occurred."`  (not specific)
- ❌ `"ERROR: Bad input!!"` (not lowercase, has punctuation)

**Rationale**: Following these conventions ensures errors work with error handling libraries like `anyhow`, `thiserror`, and `eyre`, and can be used in concurrent contexts.

**See also**: C-GOOD-ERR

---

### Error Types Should Be Enum-Based

**Strength**: SHOULD

**Summary**: Use enums to represent different error cases, allowing callers to handle specific errors differently.

**Examples**:

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

**Variant design**:
- Include relevant context in each variant
- Use struct-style variants for multiple fields
- Wrap underlying errors when propagating

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

### Error Conversions Use From Trait

**Strength**: MUST

**Summary**: Implement `From<OtherError>` to enable `?` operator and error conversion chains.

**Examples**:

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

### Errors Expose Source Chain

**Strength**: SHOULD

**Summary**: Implement `Error::source()` to expose the underlying cause of the error.

**Examples**:

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

## Result and Option Patterns

### Use ? Operator for Propagation

**Strength**: MUST

**Summary**: Use `?` to propagate errors, avoid explicit match or `unwrap()` in library code.

**Examples**:

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

**When to use unwrap()**:
- In tests where panicking is acceptable
- In examples where error handling would obscure the example
- When you have a guarantee the operation cannot fail (document why!)

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

### Prefer Result Over Option for Errors

**Strength**: SHOULD

**Summary**: Use `Result` with meaningful errors rather than `Option` when an operation can fail for multiple reasons.

**Examples**:

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

**When to use Option**:
- Single failure mode (not found, empty, None)
- Failure is expected and not exceptional
- No additional context needed

**When to use Result**:
- Multiple failure modes
- Need to convey why it failed
- Failure is exceptional
- Building error chains

**Rationale**: `Result` forces callers to think about error cases and provides information about what went wrong.

---

### Use ok_or/ok_or_else to Convert Option to Result

**Strength**: SHOULD

**Summary**: When converting `Option` to `Result`, use `ok_or` or `ok_or_else` rather than matching.

**Examples**:

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

**ok_or vs ok_or_else**:
- Use `ok_or` when the error is cheap to construct (no allocation, simple value)
- Use `ok_or_else` when the error is expensive or you need current state

```rust
// ok_or - error is simple
option.ok_or(404)

// ok_or_else - error involves allocation or computation  
option.ok_or_else(|| format!("Not found: {}", id))
option.ok_or_else(|| Error::new(Utc::now()))
```

**Rationale**: These methods are more concise and idiomatic than explicit matching.

---

### Document Error Conditions

**Strength**: MUST

**Summary**: Document when functions can return errors using an "Errors" section in rustdoc.

**Examples**:

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

### Document Panic Conditions

**Strength**: MUST

**Summary**: Document when functions may panic using a "Panics" section in rustdoc.

**Examples**:

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

**What not to document**:
- Don't document every conceivable panic, focus on the contract
- Don't document panics in dependencies unless relevant
- Don't document panics that come from obvious misuse

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

### Provide Fallible and Infallible Variants

**Strength**: CONSIDER

**Summary**: For operations that might panic, consider providing both panicking and `Result`-returning variants.

**Examples**:

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

**Naming conventions**:
- Base name for panicking version: `remove`, `unwrap`, `index`
- `try_*` prefix for Result version: `try_remove`, `try_unwrap`
- `get` vs `get_unchecked` for indexing

**Rationale**: Provides ergonomics for cases where panicking is acceptable while still supporting error handling when needed.
