# Documentation

Guidelines for writing effective Rust documentation using rustdoc.

## Crate-Level Documentation

### Crate Docs Are Thorough With Examples

**Strength**: MUST

**Summary**: Every crate should have comprehensive crate-level documentation explaining purpose, usage, and providing examples.

**Examples**:

```rust
//! # My HTTP Client
//!
//! A fast, ergonomic HTTP client for Rust.
//!
//! ## Features
//!
//! - Async/await support with tokio
//! - Automatic retries and timeouts
//! - Cookie management
//! - Connection pooling
//!
//! ## Quick Start
//!
//! ```rust
//! use my_http::Client;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::new();
//! let response = client.get("https://api.example.com/data").await?;
//! println!("Status: {}", response.status());
//! # Ok(())
//! # }
//! ```
//!
//! ## Configuration
//!
//! Customize the client with a builder:
//!
//! ```rust
//! use my_http::{Client, Duration};
//!
//! let client = Client::builder()
//!     .timeout(Duration::from_secs(30))
//!     .max_retries(3)
//!     .build();
//! ```
//!
//! ## Error Handling
//!
//! All fallible operations return `Result<T, Error>`:
//!
//! ```rust
//! use my_http::{Client, Error};
//!
//! # async fn example() -> Result<(), Error> {
//! let client = Client::new();
//! 
//! match client.get("https://api.example.com/data").await {
//!     Ok(response) => println!("Success: {}", response.text().await?),
//!     Err(Error::Timeout) => println!("Request timed out"),
//!     Err(Error::Network(e)) => println!("Network error: {}", e),
//!     Err(e) => println!("Other error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```

// Crate code follows...
```

**Required sections**:
1. **Title** - One-line description
2. **Overview** - What the crate does
3. **Quick Start** - Minimal working example
4. **Features** - Key capabilities
5. **Examples** - Common use cases
6. **Links** - Related crates, documentation

**Rationale**: Crate-level docs are the first thing users see. They should enable quick evaluation and getting started.

**See also**: C-CRATE-DOC

---

## Item Documentation

### All Public Items Have Examples

**Strength**: MUST

**Summary**: Every public function, method, type, trait, and macro should have a rustdoc example.

**Examples**:

```rust
/// Parses a configuration file.
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Examples
///
/// ```
/// use myapp::parse_config;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = parse_config("config.toml")?;
/// assert_eq!(config.timeout, 30);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The file doesn't exist or can't be read
/// - The file contains invalid TOML syntax
/// - Required fields are missing
pub fn parse_config(path: &str) -> Result<Config, Error> {
    // ...
}

/// A connection to a database.
///
/// # Examples
///
/// ```
/// use mydb::Connection;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let conn = Connection::open("database.db")?;
/// conn.execute("CREATE TABLE users (id INTEGER, name TEXT)")?;
/// # Ok(())
/// # }
/// ```
pub struct Connection {
    // ...
}

/// Iterator over the elements of a collection.
///
/// # Examples
///
/// ```
/// use mycollection::MyVec;
///
/// let vec = MyVec::from([1, 2, 3, 4, 5]);
/// 
/// for item in vec.iter() {
///     println!("{}", item);
/// }
///
/// // Or collect into a Vec
/// let items: Vec<_> = vec.iter().collect();
/// assert_eq!(items, vec![&1, &2, &3, &4, &5]);
/// ```
pub struct Iter<'a, T> {
    // ...
}
```

**Example code best practices**:

```rust
/// Example with error handling using ?
///
/// ```
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let result = some_operation()?;
/// assert_eq!(result, expected_value);
/// # Ok(())
/// # }
/// ```

/// Example with setup code hidden from docs
///
/// ```
/// use mylib::Widget;
///
/// # let window = create_test_window();
/// # let canvas = window.canvas();
/// let widget = Widget::new();
/// widget.draw(&mut canvas);
/// # drop(window);
/// ```
```

**When examples can be omitted**:
- Private items (though examples still helpful)
- Obvious getters (`fn len(&self) -> usize`)
- Items linked to another item with examples

**Rationale**: Examples show how to use the API and serve as documentation tests.

**See also**: C-EXAMPLE

---

### Examples Use ?, Not try! or unwrap

**Strength**: MUST

**Summary**: Example code should use `?` operator, not the deprecated `try!` macro or `unwrap()`.

**Examples**:

```rust
// GOOD - using ? operator
/// Reads configuration from a file.
///
/// # Examples
///
/// ```
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let config = myapp::load_config("app.toml")?;
/// assert_eq!(config.port, 8080);
/// # Ok(())
/// # }
/// ```
pub fn load_config(path: &str) -> Result<Config, Error> {
    // ...
}

// BAD - using try! macro (deprecated since Rust 1.13)
/// ```
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let config = try!(myapp::load_config("app.toml"));  // DON'T DO THIS
/// # Ok(())
/// # }
/// ```

// BAD - using unwrap
/// ```
/// let config = myapp::load_config("app.toml").unwrap();  // DON'T DO THIS
/// ```

// GOOD - multiple ? operations
/// ```
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let data = std::fs::read_to_string("input.txt")?;
/// let parsed = parse_data(&data)?;
/// let result = process(parsed)?;
/// assert_eq!(result.status, "ok");
/// # Ok(())
/// # }
/// ```
```

**Why use ?**:
- Idiomatic modern Rust
- Shows proper error handling
- Examples compile and run in tests

**When unwrap() is acceptable**:
```rust
// In test code (not example code)
#[test]
fn test_something() {
    let result = some_operation().unwrap();
    assert_eq!(result, expected);
}

// When documenting panic behavior
/// # Panics
///
/// Panics if the value is out of range.
///
/// ```should_panic
/// mylib::get_item(1000);  // Panics!
/// ```
```

**Rationale**: Examples should demonstrate best practices and compile without warnings.

**See also**: C-QUESTION-MARK

---

### Document Errors, Panics, and Safety

**Strength**: MUST

**Summary**: Use "Errors", "Panics", and "Safety" sections to document failure conditions.

**Examples**:

```rust
/// Reads exactly `n` bytes from the reader.
///
/// # Arguments
///
/// * `n` - Number of bytes to read
///
/// # Errors
///
/// This function returns an error if:
/// - The reader reaches EOF before reading `n` bytes
/// - An I/O error occurs during reading
///
/// # Examples
///
/// ```
/// use std::io::Read;
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let mut reader = "hello world".as_bytes();
/// let mut buffer = [0u8; 5];
/// read_exact(&mut reader, &mut buffer, 5)?;
/// assert_eq!(&buffer, b"hello");
/// # Ok(())
/// # }
/// ```
pub fn read_exact<R: Read>(reader: &mut R, buf: &mut [u8], n: usize) -> io::Result<()> {
    // ...
}

/// Inserts an element at the given position.
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
/// assert_eq!(vec, vec![1, 4, 2, 3]);
/// ```
///
/// Panicking example:
///
/// ```should_panic
/// let mut vec = vec![1, 2, 3];
/// vec.insert(10, 4);  // Panics!
/// ```
pub fn insert(&mut self, index: usize, element: T) {
    assert!(index <= self.len());
    // ...
}

/// Reads data from raw memory.
///
/// # Safety
///
/// Callers must ensure that:
/// - `ptr` points to valid, initialized memory
/// - `ptr` is properly aligned for type `T`
/// - The memory at `ptr` will not be accessed after this call
/// - If `T` is not `Copy`, the caller must not use the original value
///
/// # Examples
///
/// ```
/// # use std::ptr;
/// let x = 42;
/// let ptr = &x as *const i32;
/// 
/// unsafe {
///     let value = ptr::read(ptr);
///     assert_eq!(value, 42);
/// }
/// ```
pub unsafe fn read<T>(ptr: *const T) -> T {
    // ...
}
```

**Section guidelines**:

**Errors section**:
- Enumerate all error conditions
- Explain what causes each error
- Link to error type documentation

**Panics section**:
- Document all panic conditions
- Provide `should_panic` examples
- Suggest non-panicking alternatives if available

**Safety section**:
- List all invariants caller must maintain
- Explain consequences of violating invariants
- Provide correct usage examples

**Rationale**: Documenting failure modes helps users write correct code and handle edge cases.

**See also**: C-FAILURE

---

## Documentation Quality

### Prose Contains Hyperlinks

**Strength**: SHOULD

**Summary**: Link to related types, functions, and external resources throughout documentation.

**Examples**:

```rust
/// A connection to a remote server.
///
/// Created using [`Client::connect`] or [`Client::builder`].
///
/// # Examples
///
/// ```
/// use myclient::Client;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new();
/// let conn = client.connect("example.com:80").await?;
/// # Ok(())
/// # }
/// ```
///
/// See also:
/// - [`Client`] - for creating connections
/// - [`Response`] - for reading response data
pub struct Connection {
    // ...
}

/// Sends a GET request to the specified URL.
///
/// Returns a [`Response`] on success.
///
/// # Errors
///
/// Returns [`Error::InvalidUrl`] if the URL is malformed.
/// Returns [`Error::Timeout`] if the request times out.
///
/// See [`Client::post`] for sending POST requests.
///
/// # Examples
///
/// ```
/// # use myclient::Client;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new();
/// let response = client.get("https://api.example.com/users").await?;
/// # Ok(())
/// # }
/// ```
pub async fn get(&self, url: &str) -> Result<Response, Error> {
    // ...
}
```

**Link syntax**:

```rust
/// Links to items in the same module
/// [`Config`] or [`Config::new`]
///
/// Links to items in other modules
/// [`crate::http::Client`]
/// [`super::Error`]
///
/// Links with custom text
/// [configuration object][`Config`]
/// [the `connect` method][`Client::connect`]
///
/// Links to external documentation
/// See the [Rust Book](https://doc.rust-lang.org/book/)
/// 
/// Links to traits and methods
/// Implements [`Iterator::next`]
/// See [`std::io::Read`]
```

**Intra-doc links** (preferred):

```rust
/// Returns a [`Config`] with default settings.
///
/// Use [`Config::builder`] for customization.
```

**Explicit links** (when needed):

```rust
/// Returns a [`Config`](crate::config::Config) instance.
```

**Rationale**: Links make documentation navigable and help users discover related functionality.

**See also**: C-LINK

---

### Rustdoc Hides Implementation Details

**Strength**: SHOULD

**Summary**: Use `#[doc(hidden)]` and `pub(crate)` to hide implementation details from public documentation.

**Examples**:

```rust
pub struct MyType {
    pub public_field: i32,
    
    // Private fields are automatically hidden
    private_field: String,
}

/// Public error type shown in docs
pub enum PublicError {
    Network(NetworkError),
    Parse(ParseError),
}

/// Internal error type
struct InternalError {
    // ...
}

// Hide conversion implementation from docs
#[doc(hidden)]
impl From<InternalError> for PublicError {
    fn from(err: InternalError) -> PublicError {
        PublicError::Network(NetworkError::from(err))
    }
}

// Public module shown in docs
pub mod client {
    pub struct Client {
        // Use pub(crate) for internal use without showing in docs
        pub(crate) connection_pool: ConnectionPool,
    }
}

// Hide entire module from docs
#[doc(hidden)]
pub mod internal {
    // Internal utilities that must be public for cross-crate use
    // but shouldn't appear in user-facing docs
}

// Conditional compilation for test utilities
#[cfg(test)]
pub mod test_utils {
    // Only compiled during tests, not shown in docs
}
```

**When to use #[doc(hidden)]**:
- Internal conversion implementations
- Helper types that users shouldn't use directly
- Re-exports for internal organization
- Compatibility shims

**When to use pub(crate)**:
- Items used across modules within the crate
- Internal APIs not meant for users
- Intermediate types in a pipeline

**Rationale**: Users should only see the API they need, not implementation details.

**See also**: C-HIDDEN

---

## Cargo.toml Metadata

### Include All Common Metadata

**Strength**: MUST

**Summary**: Cargo.toml should include complete metadata for crates.io publication.

**Examples**:

```toml
[package]
name = "my-awesome-crate"
version = "1.0.0"
edition = "2021"

# Required metadata
authors = ["Jane Developer <jane@example.com>"]
description = "A fast, ergonomic library for doing amazing things"
license = "MIT OR Apache-2.0"
repository = "https://github.com/username/my-awesome-crate"
readme = "README.md"

# Recommended metadata
keywords = ["async", "http", "client"]  # Max 5 keywords
categories = ["network-programming", "web-programming::http-client"]
homepage = "https://myawesomecrate.dev"  # Only if different from repository
documentation = "https://docs.rs/my-awesome-crate"  # Usually auto-set

# Optional but helpful
rust-version = "1.70"  # Minimum supported Rust version (MSRV)
exclude = [
    "tests/fixtures/*",
    ".github/*",
]

[dependencies]
# Well-documented dependencies

[dev-dependencies]
# Test dependencies
```

**License field**:
```toml
# Recommended: dual license like Rust itself
license = "MIT OR Apache-2.0"

# Or single license
license = "MIT"
license = "Apache-2.0"

# Or custom (requires license file)
license-file = "LICENSE"
```

**Keywords** (5 max):
- Specific, searchable terms
- No redundant words ("rust", "crate")
- Lowercase, hyphenated

```toml
# Good
keywords = ["http", "async", "client", "rest", "api"]

# Bad
keywords = ["rust", "rust-crate", "library"]  # Too generic
```

**Categories** (5 max):
- Must be from crates.io category list
- See https://crates.io/categories

```toml
categories = [
    "network-programming",
    "web-programming::http-client",
    "asynchronous",
]
```

**README**:
- Should have installation, example, license
- Automatically displayed on crates.io

**Rationale**: Good metadata helps users discover and evaluate your crate.

**See also**: C-METADATA

---

## Release Notes

### Document All Significant Changes

**Strength**: MUST

**Summary**: Maintain a CHANGELOG.md documenting all user-visible changes.

**Examples**:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `Client::builder()` method for configuration

### Changed
- Improved error messages for connection failures

## [2.0.0] - 2024-01-15

### Added
- Async/await support
- Connection pooling
- Automatic retry with exponential backoff

### Changed
- **BREAKING**: `connect()` now async, requires tokio runtime
- **BREAKING**: Error type now implements `std::error::Error`
- Upgraded to tokio 1.0

### Deprecated
- `Client::connect_sync()` - use async version instead

### Removed
- **BREAKING**: Removed deprecated `Client::old_connect()` method

### Fixed
- Fixed memory leak in connection pool
- Fixed panic when server returns empty response

### Security
- Updated dependencies to fix CVE-2024-12345

## [1.2.1] - 2023-12-01

### Fixed
- Fixed compilation error on Windows
- Documentation typo fixes

## [1.2.0] - 2023-11-15

### Added
- Support for custom headers
- New `Response::text()` method

## [1.1.0] - 2023-10-01

### Added
- Cookie support
- Timeout configuration

[Unreleased]: https://github.com/user/repo/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/user/repo/compare/v1.2.1...v2.0.0
[1.2.1]: https://github.com/user/repo/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/user/repo/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/user/repo/releases/tag/v1.1.0
```

**Section categories**:
- **Added** - New features
- **Changed** - Changes to existing functionality
- **Deprecated** - Features to be removed
- **Removed** - Removed features (breaking)
- **Fixed** - Bug fixes
- **Security** - Security fixes

**Highlight breaking changes**:
```markdown
### Changed
- **BREAKING**: Renamed `get_user()` to `user()`
- **BREAKING**: `Error` type no longer implements `Clone`
```

**Git tags**:
```bash
# Tag releases
git tag -a v1.2.0 -m "Release version 1.2.0"
git push --tags
```

**Rationale**: Clear changelog helps users understand what changed and whether to upgrade.

**See also**: C-RELNOTES
