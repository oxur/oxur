# Documentation

> Patterns for doc comments, examples, and rustdoc.

---

## DC-01: Every Public Item Needs Documentation

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

## DC-02: Document Sections

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

**Standard sections**:

| Section | When to use |
|---------|-------------|
| (Summary) | Always - first paragraph |
| # Arguments | When parameters need explanation |
| # Returns | When return value needs explanation |
| # Errors | When function returns Result |
| # Panics | When function can panic |
| # Examples | Complex functions, public API |
| # Safety | Unsafe functions (required) |
| # See Also | Related functions/types |

---

## DC-03: Examples in Documentation

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

## DC-04: Unsafe Documentation Requirements

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

## DC-05: Module and Crate Documentation

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

## DC-06: Link to Related Items

**Strength**: SHOULD

**Summary**: Use intra-doc links to connect related documentation.

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

---

## DC-07: Document Trait Implementors

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

## DC-08: Error Type Documentation

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

## DC-09: Feature-Gated Documentation

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

```toml
# Cargo.toml - Enable feature badges on docs.rs
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

---

## DC-10: Doc Aliases

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

## DC-11: Documentation Tests as Integration Tests

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

## DC-12: README and Documentation Sync

**Strength**: CONSIDER

**Summary**: Keep README and crate docs in sync.

```rust
// src/lib.rs
#![doc = include_str!("../README.md")]

// This includes the entire README as crate documentation
// Pros: Single source of truth
// Cons: README must be valid rustdoc (links, examples, etc.)
```

Alternative: Use a template

```markdown
<!-- README.md -->
# My Crate

[Documentation](https://docs.rs/my_crate)

<!-- Keep in sync with src/lib.rs doc comment -->
```

---

## Summary: Documentation Checklist

**Required**:
- [ ] All public items have doc comments
- [ ] Unsafe functions have `# Safety` sections
- [ ] Functions returning Result have `# Errors` sections
- [ ] Panicking functions have `# Panics` sections

**Recommended**:
- [ ] Examples for public API items
- [ ] Module-level documentation (`//!`)
- [ ] Crate-level documentation with quick start
- [ ] Intra-doc links to related items

**Nice to have**:
- [ ] `# Arguments` and `# Returns` sections
- [ ] `# See Also` sections
- [ ] Doc aliases for discoverability
- [ ] Feature-gated documentation badges

**Testing**:
- [ ] All doc examples compile: `cargo test --doc`
- [ ] Documentation renders correctly: `cargo doc --open`

---

*See also: [02-api-design.md](02-api-design.md#api-08) for API documentation.*
