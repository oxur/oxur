# Documentation Guidelines

Best practices for writing excellent Rust documentation with rustdoc.

## Table of Contents

- [Documentation Structure](#documentation-structure)
- [Module Documentation](#module-documentation)
- [Function Documentation](#function-documentation)
- [Formatting](#formatting)

---

## Documentation Structure

### First Sentence is One Line, ~15 Words

**Strength**: MUST

**Summary**: The first sentence becomes the summary—keep it to one line and approximately 15 words.

**Example**:
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

### Documentation Has Canonical Sections

**Strength**: MUST

**Summary**: Use standard sections in a consistent order for comprehensive documentation.

**Example**:
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

**Canonical sections in order**:
1. **Summary** - First sentence (required)
2. **Extended description** - Additional paragraphs (recommended)
3. **Examples** - Code examples showing usage (strongly encouraged)
4. **Errors** - Conditions that return Err (for Result-returning functions)
5. **Panics** - Conditions that may panic (when applicable)
6. **Safety** - Safety requirements (for unsafe functions, required)
7. **Abort** - Conditions that may abort (when applicable)

**See also**: M-CANONICAL-DOCS, C-FAILURE

---

## Module Documentation

### Modules Have Comprehensive Documentation

**Strength**: MUST

**Summary**: All public modules must have `//!` documentation covering purpose, usage, and examples.

**Example**:
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

**What to include**:
- What the module contains
- When it should be used
- Examples showing common usage
- Subsystem specifications (like `std::fmt`)
- Observable side effects
- Implementation details when relevant

**Good examples from std**:
- `std::fmt` - Describes formatting mini-language
- `std::pin` - Explains pinning concept thoroughly
- `std::option` - Shows usage patterns

**See also**: M-MODULE-DOCS

---

### Mark pub use with #[doc(inline)]

**Strength**: SHOULD

**Summary**: Re-exported items should use `#[doc(inline)]` to appear naturally in documentation.

**Example**:
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

**Exception**: External crate types should not be inlined to make their external nature clear.

**See also**: M-DOC-INLINE

---

## Function Documentation

### Parameters Are Explained in Prose

**Strength**: SHOULD

**Summary**: Don't create parameter tables; explain parameter usage in the description.

**Example**:
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

### Examples Should Compile

**Strength**: SHOULD

**Summary**: Doc examples should be valid, runnable code unless explicitly marked as pseudocode.

**Example**:
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

**Hidden lines**: Use `#` prefix for setup code that shouldn't appear in rendered docs.

**See also**: C-EXAMPLE

---

## Formatting

### Avoid Over-Formatting

**Strength**: SHOULD

**Summary**: Minimize use of bold, headers, and bullets in documentation; prefer natural prose.

**Example**:
```rust
// WRONG - over-formatted
/// **Process** user data
///
/// ## Features
/// - Validates input
/// - **Normalizes** data
/// - Stores in database
///
/// ### Returns
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

**When to use formatting**:
- Lists for actual enumeration
- Code blocks for examples
- Headers for distinct sections
- Links to related items

**See also**: M-FIRST-DOC-SENTENCE

---

### Link to Related Items

**Strength**: SHOULD

**Summary**: Link to related types, functions, and modules using rustdoc link syntax.

**Example**:
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

**Link styles**:
- `[`Type`]` - Link to type
- `[`function()`]` - Link to function
- `[`module`]` - Link to module
- `[custom text][link]` - Custom link text

**See also**: C-LINK

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
