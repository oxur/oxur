# Documentation Guidelines

Best practices for writing excellent Rust documentation with rustdoc.


## DC-01: Added

**Strength**: SHOULD

**Summary**: 

---

## DC-02: All Public Items Have Examples

**Strength**: MUST

**Summary**: Every public function, method, type, trait, and macro should have a rustdoc example.

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

**Rationale**: Examples show how to use the API and serve as documentation tests.

**See also**: C-EXAMPLE

---

## DC-03: Avoid Over-Formatting

**Strength**: SHOULD

**Summary**: Minimize use of bold, headers, and bullets in documentation; prefer natural prose.

```rust
// WRONG - over-formatted
/// **Process** user data
///
/// ## Features
/// - Validates input
/// - **Normalizes** data
/// - Stores in database
///
/// ## Returns
/// `Result<User, Error>`
pub fn process_user(data: UserData) -> Result<User, Error> { }

// CORRECT - natural prose
/// Processes user data by validating input, normalizing fields, and storing
/// the result in the database.
///
/// Returns the created user record or an error if validation fails.
pub fn process_user(data: UserData) -> Result<User, Error> { }

// Lists are OK when actually listing items
/// Supported formats:
/// - TOML
/// - JSON
/// - YAML
pub fn parse_config(path: &Path) -> Config { }
```

**Rationale**: Heavy formatting makes documentation feel like a marketing page rather than technical reference. Natural prose is more readable.

**See also**: M-FIRST-DOC-SENTENCE

---

## DC-04: Changed

**Strength**: SHOULD

**Summary**: 

**Rationale**: Clear changelog helps users understand what changed and whether to upgrade.

**See also**: C-RELNOTES

---

## DC-05: Crate Docs Are Thorough With Examples

**Strength**: MUST

**Summary**: Every crate should have comprehensive crate-level documentation explaining purpose, usage, and providing examples.

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

**Rationale**: Crate-level docs are the first thing users see. They should enable quick evaluation and getting started.

**See also**: C-CRATE-DOC

---

## DC-06: Deprecated

**Strength**: SHOULD

**Summary**: 

---

## DC-07: Doc Aliases

**Strength**: CONSIDER

**Summary**: Add search aliases for discoverability.

```rust
/// A first-in, first-out queue.
///
/// Also known as a FIFO queue or simply a queue.
#[doc(alias = "FIFO")]
#[doc(alias = "queue")]
pub struct Queue<T> { /* ... */ }

/// Removes and returns the first element.
///
/// This is also known as "dequeue" in some contexts.
#[doc(alias = "dequeue")]
#[doc(alias = "pop_front")]
pub fn pop(&mut self) -> Option<T> { todo!() }

// Users can now find Queue by searching:
// - Queue
// - FIFO
// - queue
```

---

## DC-08: Document All Significant Changes

**Strength**: MUST

**Summary**: Maintain a CHANGELOG.md documenting all user-visible changes.

---

## DC-09: Document Errors, Panics, and Safety

**Strength**: MUST

**Summary**: Use "Errors", "Panics", and "Safety" sections to document failure conditions.

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

**Rationale**: Documenting failure modes helps users write correct code and handle edge cases.

**See also**: C-FAILURE

---

## DC-10: Document Sections

**Strength**: SHOULD

**Summary**: Use standard sections for comprehensive documentation.

```rust
/// Parses a date string into a `DateTime` object.
///
/// Accepts dates in ISO 8601 format (YYYY-MM-DD) with optional
/// time component (HH:MM:SS).
///
/// # Arguments
///
/// * `input` - A string slice containing the date to parse
/// * `timezone` - Optional timezone for interpretation
///
/// # Returns
///
/// A `DateTime` in the specified or UTC timezone.
///
/// # Errors
///
/// Returns `ParseError::InvalidFormat` if the string doesn't match
/// the expected format.
///
/// Returns `ParseError::OutOfRange` if the date components are invalid
/// (e.g., month 13).
///
/// # Panics
///
/// Panics if the system clock is unavailable (extremely rare).
///
/// # Examples
///
/// ```
/// use my_crate::parse_date;
///
/// let date = parse_date("2024-01-15", None)?;
/// assert_eq!(date.year(), 2024);
/// assert_eq!(date.month(), 1);
/// assert_eq!(date.day(), 15);
/// # Ok::<(), my_crate::ParseError>(())
/// ```
///
/// With timezone:
///
/// ```
/// use my_crate::{parse_date, Timezone};
///
/// let date = parse_date("2024-01-15", Some(Timezone::EST))?;
/// # Ok::<(), my_crate::ParseError>(())
/// ```
///
/// # See Also
///
/// * [`parse_datetime`] - For full datetime parsing
/// * [`DateTime::parse`] - The underlying parsing method
pub fn parse_date(input: &str, timezone: Option<Timezone>) -> Result<DateTime, ParseError> {
    todo!()
}
```

---

## DC-11: Document Trait Implementors

**Strength**: SHOULD

**Summary**: Document trait requirements and provide implementation guidance.

```rust
/// A trait for types that can be converted to bytes.
///
/// # Implementing
///
/// Implementations should ensure that:
///
/// 1. The returned bytes represent the value completely
/// 2. The encoding is deterministic (same input → same output)
/// 3. The encoding is reasonably efficient
///
/// # Examples
///
/// Implementing for a custom type:
///
/// ```
/// use my_crate::ToBytes;
///
/// struct Point { x: i32, y: i32 }
///
/// impl ToBytes for Point {
///     fn to_bytes(&self) -> Vec<u8> {
///         let mut bytes = Vec::with_capacity(8);
///         bytes.extend_from_slice(&self.x.to_le_bytes());
///         bytes.extend_from_slice(&self.y.to_le_bytes());
///         bytes
///     }
/// }
/// ```
///
/// # Provided Implementations
///
/// This trait is implemented for:
///
/// * All primitive integer types
/// * `String` and `&str` (as UTF-8)
/// * `Vec<u8>` and `&[u8]` (identity)
pub trait ToBytes {
    /// Converts this value to a byte vector.
    fn to_bytes(&self) -> Vec<u8>;
}
```

---

## DC-12: Documentation Checklist

**Strength**: SHOULD

**Summary**: 

```rust
/// Summary sentence in one line, approximately 15 words maximum.
///
/// Extended description providing more context about what this item does,
/// when to use it, and how it fits into the larger system.
///
/// # Examples
///
/// ```
/// use my_crate::Item;
///
/// let item = Item::new();
/// assert!(item.is_valid());
/// ```
///
/// # Errors
///
/// (For Result-returning functions)
/// Returns an error if...
///
/// # Panics
///
/// (When applicable)
/// Panics if...
///
/// # Safety
///
/// (For unsafe functions - required)
/// The caller must ensure...
pub fn item() -> Result<T, E> {
    // ...
}
```

---

## DC-13: Documentation Has Canonical Sections

**Strength**: MUST

**Summary**: Use standard sections in a consistent order for comprehensive documentation.

```rust
/// Parses a configuration file from the given path.
///
/// This function reads the file and parses it as TOML format. The configuration
/// is validated against the expected schema.
///
/// # Examples
///
/// ```
/// use my_crate::parse_config;
///
/// let config = parse_config("app.toml")?;
/// assert_eq!(config.timeout, Duration::from_secs(30));
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read (I/O error)
/// - The file contains invalid TOML syntax
/// - Required configuration fields are missing
///
/// # Panics
///
/// Panics if the path contains invalid Unicode characters.
///
/// # Safety
///
/// (Only for unsafe functions)
/// The caller must ensure that the file path points to a valid location.
pub fn parse_config(path: &Path) -> Result<Config, ConfigError> {
    // ...
}
```

**See also**: M-CANONICAL-DOCS, C-FAILURE

---

## DC-14: Documentation Tests as Integration Tests

**Strength**: SHOULD

**Summary**: Use doc tests to ensure examples stay correct.

```rust
/// Adds two numbers.
///
/// # Examples
///
/// ```
/// # // Hidden setup lines start with #
/// use my_crate::add;
///
/// assert_eq!(add(2, 2), 4);
/// assert_eq!(add(-1, 1), 0);
/// ```
///
/// Overflow handling:
///
/// ```
/// use my_crate::add;
///
/// // Large numbers work correctly
/// assert_eq!(add(i32::MAX - 1, 1), i32::MAX);
/// ```
///
/// ```should_panic
/// use my_crate::add;
///
/// // This panics on overflow in debug builds
/// add(i32::MAX, 1);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b  // Note: This doesn't actually panic, example is illustrative
}

// Run doc tests with: cargo test --doc
```

---

## DC-15: Error Type Documentation

**Strength**: SHOULD

**Summary**: Document error types with causes and handling guidance.

```rust
/// Errors that can occur during parsing.
///
/// # Handling
///
/// Most parsing errors are recoverable. Check the error variant
/// to determine appropriate handling:
///
/// ```
/// use my_crate::{parse, ParseError};
///
/// match parse(input) {
///     Ok(data) => process(data),
///     Err(ParseError::InvalidSyntax { line, .. }) => {
///         eprintln!("Syntax error on line {}", line);
///     }
///     Err(ParseError::Io(e)) => {
///         eprintln!("I/O error: {}", e);
///     }
///     Err(e) => {
///         eprintln!("Unexpected error: {}", e);
///     }
/// }
/// # fn process(_: ()) {}
/// # let input = "";
/// ```
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Invalid syntax in the input.
    ///
    /// Contains the line number and column where the error was detected.
    #[error("invalid syntax at {line}:{column}: {message}")]
    InvalidSyntax {
        line: usize,
        column: usize,
        message: String,
    },
    
    /// The input ended unexpectedly.
    ///
    /// This usually means the input is incomplete (e.g., unclosed bracket).
    #[error("unexpected end of input, expected {expected}")]
    UnexpectedEof { expected: String },
    
    /// An I/O error occurred while reading input.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## DC-16: Every Public Item Needs Documentation

**Strength**: MUST

**Summary**: Document all public items with at least a one-line summary.

```rust
// ❌ BAD: No documentation
pub fn process(input: &str) -> Result<Output, Error> {
    todo!()
}

// ✅ GOOD: Clear, concise documentation
/// Processes the input string and returns parsed output.
///
/// # Errors
///
/// Returns an error if the input is malformed or empty.
pub fn process(input: &str) -> Result<Output, Error> {
    todo!()
}

// ✅ GOOD: Module-level documentation
//! Parser module for handling input data.
//!
//! This module provides utilities for parsing various input formats
//! including JSON, XML, and custom DSLs.

pub mod json;
pub mod xml;
pub mod dsl;

// ✅ GOOD: Type documentation
/// A configuration for the parser.
///
/// Contains settings that control parsing behavior including
/// strictness, encoding, and error handling.
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether to fail on first error or collect all errors.
    pub strict: bool,
    
    /// Maximum recursion depth for nested structures.
    pub max_depth: usize,
}
```

---

## DC-17: Examples in Documentation

**Strength**: SHOULD

**Summary**: Provide runnable examples for public API.

```rust
/// Creates a new buffer with the specified capacity.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use my_crate::Buffer;
///
/// let buffer = Buffer::with_capacity(1024);
/// assert!(buffer.capacity() >= 1024);
/// ```
///
/// Growing the buffer:
///
/// ```
/// use my_crate::Buffer;
///
/// let mut buffer = Buffer::with_capacity(10);
/// buffer.extend(b"hello world - this is longer than 10 bytes");
/// assert!(buffer.capacity() > 10);
/// ```
pub fn with_capacity(cap: usize) -> Self {
    todo!()
}

// ✅ Examples that should compile but not run
/// ```no_run
/// use my_crate::connect;
///
/// // This would actually connect to a server
/// let conn = connect("localhost:8080")?;
/// # Ok::<(), std::io::Error>(())
/// ```

// ✅ Examples that should fail to compile
/// ```compile_fail
/// use my_crate::ImmutableData;
///
/// let data = ImmutableData::new();
/// data.modify();  // This should not compile!
/// ```

// ✅ Examples that demonstrate expected panic
/// ```should_panic
/// use my_crate::divide;
///
/// divide(1, 0);  // Panics on division by zero
/// ```

// ✅ Hiding boilerplate in examples
/// ```
/// # use my_crate::{Config, Error};
/// # fn main() -> Result<(), Error> {
/// let config = Config::from_file("config.toml")?;
/// println!("Loaded: {:?}", config);
/// # Ok(())
/// # }
/// ```
```

---

## DC-18: Examples Should Compile

**Strength**: SHOULD

**Summary**: Doc examples should be valid, runnable code unless explicitly marked as pseudocode.

```rust
/// Processes user data.
///
/// # Examples
///
/// ```
/// use my_crate::process_user;
///
/// let user = User::new("alice");
/// let result = process_user(user)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// For testing purposes, you can use a mock:
///
/// ```
/// # use my_crate::*;
/// let (processor, mock) = Processor::new_mocked();
/// mock.set_user_data(test_data);
/// assert!(processor.validate());
/// ```
pub fn process_user(user: User) -> Result<(), Error> {
    // ...
}

// Use `no_run` for examples that shouldn't execute
/// Connects to a remote server.
///
/// # Examples
///
/// ```no_run
/// let client = connect("api.example.com:443")?;
/// ```
pub fn connect(addr: &str) -> Result<Client, Error> {
    // ...
}

// Use `ignore` for pseudocode
/// Complex algorithm outline.
///
/// # Algorithm
///
/// ```ignore
/// for each item:
///     if condition(item):
///         process(item)
/// ```
pub fn algorithm() { }
```

**Rationale**: Runnable examples serve as tests and documentation. They verify code examples stay current as the API evolves.

**See also**: C-EXAMPLE

---

## DC-19: Examples Use ?, Not try! or unwrap

**Strength**: MUST

**Summary**: Example code should use `?` operator, not the deprecated `try!` macro or `unwrap()`.

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

## DC-20: Feature-Gated Documentation

**Strength**: SHOULD

**Summary**: Document feature requirements for conditional items.

```rust
/// Async parser implementation.
///
/// This is only available with the `async` feature enabled.
///
/// # Feature
///
/// This requires the `async` feature:
///
/// ```toml
/// [dependencies]
/// my_crate = { version = "1.0", features = ["async"] }
/// ```
///
/// # Examples
///
/// ```ignore
/// use my_crate::AsyncParser;
///
/// #[tokio::main]
/// async fn main() {
///     let parser = AsyncParser::new();
///     let result = parser.parse("input").await?;
/// }
/// ```
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub struct AsyncParser { /* ... */ }
```

---

## DC-21: First Sentence is One Line, ~15 Words

**Strength**: MUST

**Summary**: The first sentence becomes the summary—keep it to one line and approximately 15 words.

```rust
/// Opens a file at the specified path for reading.
///
/// This function will attempt to open the file in read-only mode. If the file
/// does not exist, it will return an error.
///
/// # Examples
///
/// ```
/// let file = open_file("config.toml")?;
/// ```
pub fn open_file(path: &str) -> Result<File, Error> {
    // ...
}

// WRONG - first sentence too long
/// Opens a file at the specified path for reading and returns a File handle that can be used to read the contents of the file.
pub fn open_file(path: &str) -> Result<File, Error> {
    // Summary wraps awkwardly in listings
}
```

**Rationale**: Rustdoc extracts the first sentence for module listings and search results. Keeping it to one line (~15 words) makes documentation skimmable.

**See also**: M-FIRST-DOC-SENTENCE

---

## DC-22: Fixed

**Strength**: SHOULD

**Summary**: 

---

## DC-23: Include All Common Metadata

**Strength**: MUST

**Summary**: Cargo.toml should include complete metadata for crates.io publication.

**Rationale**: Good metadata helps users discover and evaluate your crate.

**See also**: C-METADATA

---

## DC-24: Link to Related Items

**Strength**: SHOULD

**Summary**: Use intra-doc links to connect related documentation. Link to related types, functions, and modules using rustdoc link syntax.

```rust
/// A parser for JSON data.
///
/// This is the primary parser for JSON format. For XML parsing,
/// see [`XmlParser`]. For a unified interface, see the [`Parse`] trait.
///
/// # Related
///
/// * [`Parser::new`] - Create a new parser
/// * [`Parse::parse`] - The parsing method
/// * [`crate::error::ParseError`] - Error type
pub struct JsonParser { /* ... */ }

impl JsonParser {
    /// Creates a new parser with default settings.
    ///
    /// For custom configuration, use [`JsonParser::with_config`].
    pub fn new() -> Self { todo!() }
    
    /// Creates a new parser with the given configuration.
    ///
    /// See [`Config`] for available options.
    pub fn with_config(config: Config) -> Self { todo!() }
}

// Link syntax:
// [`Type`] - link to type
// [`Type::method`] - link to method
// [`module::Type`] - link to type in module
// [`crate::Type`] - link to type in crate root
// [link text](`Type`) - custom link text
```

```rust
/// Configuration for HTTP clients.
///
/// Created using [`ClientBuilder`] and passed to [`Client::new()`].
///
/// See also: [`ServerConfig`], [`parse_config()`]
///
/// [`ClientBuilder`]: crate::ClientBuilder
/// [`Client::new()`]: crate::Client::new
/// [`ServerConfig`]: crate::ServerConfig
/// [`parse_config()`]: crate::parse_config
pub struct ClientConfig {
    // ...
}

// Intra-doc link syntax (preferred)
/// See [`Config`] for configuration options.
///
/// Related: [`process`], [`validate`]
pub fn parse_config() -> Config {
    // ...
}
```

**See also**: C-LINK

---

## DC-25: Mark pub use with #[doc(inline)]

**Strength**: SHOULD

**Summary**: Re-exported items should use `#[doc(inline)]` to appear naturally in documentation.

```rust
// WRONG - creates opaque "Re-exports" section
pub use other_crate::ImportantType;

// CORRECT - inlines documentation
#[doc(inline)]
pub use other_crate::ImportantType;

// For glob re-exports (when necessary)
#[doc(inline)]
pub use internal::*;

// Don't inline external crates (make it clear they're external)
pub use std::io::Error;  // No inline - clearly external
```

**Rationale**: Inline documentation makes re-exported items feel like first-class citizens of your module rather than external imports.

**See also**: M-DOC-INLINE

---

## DC-26: Module and Crate Documentation

**Strength**: SHOULD

**Summary**: Document modules and crates with `//!` comments.

```rust
// src/lib.rs
//! # My Crate
//!
//! `my_crate` provides utilities for parsing and formatting data.
//!
//! ## Quick Start
//!
//! ```
//! use my_crate::{parse, format};
//!
//! let data = parse("input")?;
//! let output = format(&data);
//! # Ok::<(), my_crate::Error>(())
//! ```
//!
//! ## Features
//!
//! - **Fast parsing**: Zero-copy parsing where possible
//! - **Flexible formatting**: Multiple output formats supported
//! - **Async support**: Optional async API with `async` feature
//!
//! ## Feature Flags
//!
//! - `async`: Enable async API (requires tokio)
//! - `serde`: Enable serialization support
//!
//! ## Modules
//!
//! - [`parser`]: Input parsing utilities
//! - [`formatter`]: Output formatting utilities
//! - [`error`]: Error types

pub mod parser;
pub mod formatter;
pub mod error;

// src/parser.rs
//! Parsing utilities for various input formats.
//!
//! This module provides parsers for JSON, XML, and custom formats.
//! All parsers implement the [`Parse`] trait.
//!
//! # Examples
//!
//! ```
//! use my_crate::parser::{JsonParser, Parse};
//!
//! let parser = JsonParser::new();
//! let result = parser.parse(r#"{"key": "value"}"#)?;
//! # Ok::<(), my_crate::Error>(())
//! ```

pub trait Parse { /* ... */ }
pub struct JsonParser { /* ... */ }
```

---

## DC-27: Module Template

**Strength**: SHOULD

**Summary**: 

```rust
//! One-line summary of what this module provides.
//!
//! Extended description explaining the module's purpose, design decisions,
//! and how to use it effectively.
//!
//! # Examples
//!
//! ```
//! use my_crate::my_module::{Thing, do_thing};
//!
//! let thing = Thing::new();
//! do_thing(&thing)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Design
//!
//! (Optional) Explain key design decisions or architectural patterns used
//! in this module.

pub struct Thing { /* ... */ }
pub fn do_thing(t: &Thing) -> Result<(), Error> { /* ... */ }
```

---

## DC-28: Modules Have Comprehensive Documentation

**Strength**: MUST

**Summary**: All public modules must have `//!` documentation covering purpose, usage, and examples.

```rust
//! Configuration parsing and validation.
//!
//! This module provides types and functions for working with application
//! configuration files. It supports TOML format and includes validation
//! for common configuration errors.
//!
//! # Examples
//!
//! ```
//! use my_crate::config::{Config, load_config};
//!
//! let config = load_config("app.toml")?;
//! println!("Server: {}", config.server_url);
//! ```
//!
//! # Configuration Format
//!
//! The configuration file should be in TOML format:
//!
//! ```toml
//! server_url = "https://api.example.com"
//! timeout_seconds = 30
//! max_retries = 3
//! ```
//!
//! # Error Handling
//!
//! All functions return detailed errors that indicate the specific problem,
//! including file location and line numbers for parsing errors.

pub struct Config { /* ... */ }
pub fn load_config(path: &Path) -> Result<Config, ConfigError> { /* ... */ }
```

**See also**: M-MODULE-DOCS

---

## DC-29: Parameters Are Explained in Prose

**Strength**: SHOULD

**Summary**: Don't create parameter tables; explain parameter usage in the description.

```rust
// WRONG - parameter table
/// Copies a file.
///
/// # Parameters
/// - src: The source file path
/// - dst: The destination file path
/// - overwrite: Whether to overwrite existing files
fn copy_file(src: &Path, dst: &Path, overwrite: bool) { }

// CORRECT - prose explanation
/// Copies a file from `src` to `dst`.
///
/// If `overwrite` is true, any existing file at `dst` will be replaced.
/// Otherwise, the function returns an error if `dst` already exists.
fn copy_file(src: &Path, dst: &Path, overwrite: bool) { }
```

**Rationale**: Rust documentation style favors natural language over structured tables. Parameters are typically clear from their names and types.

**See also**: M-CANONICAL-DOCS

---

## DC-30: Prose Contains Hyperlinks

**Strength**: SHOULD

**Summary**: Link to related types, functions, and external resources throughout documentation.

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

```rust
/// Returns a [`Config`] with default settings.
///
/// Use [`Config::builder`] for customization.
```

```rust
/// Returns a [`Config`](crate::config::Config) instance.
```

**Rationale**: Links make documentation navigable and help users discover related functionality.

**See also**: C-LINK

---

## DC-31: README and Documentation Sync

**Strength**: CONSIDER

**Summary**: Keep README and crate docs in sync.

```rust
// src/lib.rs
#![doc = include_str!("../README.md")]

// This includes the entire README as crate documentation
// Pros: Single source of truth
// Cons: README must be valid rustdoc (links, examples, etc.)
```

---

## DC-32: Removed

**Strength**: SHOULD

**Summary**: 

---

## DC-33: Rustdoc Hides Implementation Details

**Strength**: SHOULD

**Summary**: Use `#[doc(hidden)]` and `pub(crate)` to hide implementation details from public documentation.

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

**Rationale**: Users should only see the API they need, not implementation details.

**See also**: C-HIDDEN

---

## DC-34: Security

**Strength**: SHOULD

**Summary**: 

---

## DC-35: Unsafe Documentation Requirements

**Strength**: MUST

**Summary**: Unsafe functions require a `# Safety` section.

```rust
/// Reads a value from the raw pointer.
///
/// # Safety
///
/// The caller must ensure that:
///
/// * `ptr` is non-null
/// * `ptr` is properly aligned for `T`
/// * `ptr` points to a valid, initialized instance of `T`
/// * The memory referenced by `ptr` is not mutated during this call
/// * The memory referenced by `ptr` is valid for reads of `size_of::<T>()` bytes
///
/// # Examples
///
/// ```
/// use my_crate::read_ptr;
///
/// let value = 42i32;
/// let ptr = &value as *const i32;
///
/// // SAFETY: ptr is valid, aligned, and points to initialized data
/// let read_value = unsafe { read_ptr(ptr) };
/// assert_eq!(read_value, 42);
/// ```
pub unsafe fn read_ptr<T>(ptr: *const T) -> T {
    ptr.read()
}
```

---

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| First sentence ~15 words | MUST | Enables skimming |
| Canonical sections | MUST | Standard structure |
| Module docs comprehensive | MUST | Purpose, usage, examples |
| #[doc(inline)] for re-exports | SHOULD | Natural integration |
| Parameters in prose | SHOULD | Not table format |
| Examples compile | SHOULD | Living documentation |
| Minimize formatting | SHOULD | Natural prose |
| Link related items | SHOULD | Navigation |

## Documentation Checklist

```rust
/// Summary sentence in one line, approximately 15 words maximum.
///
/// Extended description providing more context about what this item does,
/// when to use it, and how it fits into the larger system.
///
/// # Examples
///
/// ```
/// use my_crate::Item;
///
/// let item = Item::new();
/// assert!(item.is_valid());
/// ```
///
/// # Errors
///
/// (For Result-returning functions)
/// Returns an error if...
///
/// # Panics
///
/// (When applicable)
/// Panics if...
///
/// # Safety
///
/// (For unsafe functions - required)
/// The caller must ensure...
pub fn item() -> Result<T, E> {
    // ...
}
```

## Module Template

```rust
//! One-line summary of what this module provides.
//!
//! Extended description explaining the module's purpose, design decisions,
//! and how to use it effectively.
//!
//! # Examples
//!
//! ```
//! use my_crate::my_module::{Thing, do_thing};
//!
//! let thing = Thing::new();
//! do_thing(&thing)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Design
//!
//! (Optional) Explain key design decisions or architectural patterns used
//! in this module.

pub struct Thing { /* ... */ }
pub fn do_thing(t: &Thing) -> Result<(), Error> { /* ... */ }
```

## Related Guidelines

- **Core Idioms**: See `01-core-idioms.md` for Debug trait
- **API Design**: See `02-api-design.md` for clear interfaces
- **Error Handling**: See `03-error-handling.md` for error documentation

## External References

- [Rustdoc Book](https://doc.rust-lang.org/rustdoc/)
- [API Guidelines - Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)
- Pragmatic Rust: M-FIRST-DOC-SENTENCE, M-MODULE-DOCS, M-CANONICAL-DOCS