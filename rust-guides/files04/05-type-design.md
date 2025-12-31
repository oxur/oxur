# Type Design Guidelines

Patterns for designing types, including newtypes, enums, and strong typing strategies.

## Table of Contents

- [Debug and Display Traits](#debug-and-display-traits)
- [Type Families](#type-families)
- [Newtype Pattern](#newtype-pattern)
- [Common Traits](#common-traits)

---

## Debug and Display Traits

### Public Types Implement Debug

**Strength**: MUST

**Summary**: All public types must implement `Debug`; types with sensitive data must use custom implementations.

**Example**:
```rust
use std::fmt::{self, Debug, Formatter};

// Good - simple derived Debug
#[derive(Debug)]
pub struct Endpoint {
    url: String,
    timeout: Duration,
}

// Good - custom Debug for sensitive data
pub struct UserCredentials {
    username: String,
    password: String,
    api_key: String,
}

impl Debug for UserCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserCredentials")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("api_key", &"<redacted>")
            .finish()
    }
}

// Test that sensitive data doesn't leak
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_credentials_debug_redacts_secrets() {
        let creds = UserCredentials {
            username: "alice".to_string(),
            password: "super_secret_123".to_string(),
            api_key: "sk-1234567890abcdef".to_string(),
        };
        
        let debug_output = format!("{:?}", creds);
        
        // Verify sensitive data is not present
        assert!(!debug_output.contains("super_secret"));
        assert!(!debug_output.contains("sk-1234"));
        assert!(debug_output.contains("redacted"));
        assert!(debug_output.contains("alice"));  // Username is OK
    }
}
```

**Rationale**: `Debug` is essential for development and logging. Custom implementations prevent accidental leakage of secrets in debug output, logs, or error messages.

**When to use custom Debug**:
- Passwords, tokens, API keys
- Personal information (PII)
- Cryptographic keys
- Any data that shouldn't appear in logs

**See also**: M-PUBLIC-DEBUG

---

### Readable Types Implement Display

**Strength**: MUST (for user-facing types)

**Summary**: Types intended to be read by users must implement `Display` following Rust conventions.

**Example**:
```rust
use std::fmt::{self, Display, Formatter};

// Good - Display for user-facing type
pub struct UserId(u64);

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

// Good - Display for error type
pub struct ValidationError {
    field: String,
    message: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Validation failed for '{}': {}", self.field, self.message)
    }
}

// Display handles newlines and escape sequences
pub struct MultilineMessage {
    lines: Vec<String>,
}

impl Display for MultilineMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

// Usage
let user_id = UserId(42);
println!("User: {}", user_id);  // "User: user:42"

let error = ValidationError {
    field: "email".to_string(),
    message: "Invalid email format".to_string(),
};
eprintln!("{}", error);  // User-friendly error message
```

**Rationale**: `Display` is for user-facing output. It should produce clean, readable text. Error types must implement it (required by `std::error::Error`).

**When to implement Display**:
- Error types (mandatory)
- IDs and identifiers shown to users
- Types wrapping string-like data
- Types that appear in user-facing output

**See also**: M-PUBLIC-DISPLAY

---

## Type Families

### Use Strong Types

**Strength**: SHOULD

**Summary**: Use the most specific standard library type for your domain; create newtypes when std types aren't sufficient.

**Example**:
```rust
use std::path::{Path, PathBuf};
use std::time::Duration;

// Bad - primitive obsession
pub struct Config {
    timeout_seconds: u64,
    max_retries: u64,
    config_file: String,  // Should be PathBuf
    user_id: u64,
    api_key: String,
}

pub fn create_user(id: u64, timeout: u64) -> User {
    // Easy to swap parameters!
}

// Good - strong types
pub struct Config {
    timeout: Duration,
    max_retries: RetryCount,
    config_file: PathBuf,
    user_id: UserId,
    api_key: ApiKey,
}

pub struct RetryCount(u32);
pub struct UserId(u64);
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(key: String) -> Result<Self, ValidationError> {
        if key.len() < 20 {
            return Err(ValidationError::InvalidApiKey);
        }
        Ok(Self(key))
    }
}

pub fn create_user(id: UserId, timeout: Duration) -> User {
    // Can't swap parameters - compile error!
}

// Usage
let config = Config {
    timeout: Duration::from_secs(30),
    max_retries: RetryCount(3),
    config_file: PathBuf::from("app.toml"),
    user_id: UserId(42),
    api_key: ApiKey::new("sk-very-long-key".to_string())?,
};
```

**Rationale**: Strong types prevent bugs through type safety. Parameters can't be swapped, validation can be enforced at construction, and intent is clearer.

**See also**: M-STRONG-TYPES, C-NEWTYPE

---

## Newtype Pattern

### When to Use Newtypes

**Strength**: SHOULD

**Summary**: Create newtypes to add semantics, enforce invariants, or prevent parameter confusion.

**Example**:
```rust
// 1. Add semantics
pub struct Meters(f64);
pub struct Seconds(f64);

pub fn calculate_speed(distance: Meters, time: Seconds) -> f64 {
    distance.0 / time.0
}

// Can't accidentally pass (time, distance)

// 2. Enforce invariants
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: String) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Some(Self(s))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// 3. Prevent parameter confusion in builders
pub struct Account {
    bank: Bank,
    customer: Customer,
}

pub struct Bank(String);
pub struct Customer(String);

impl Account {
    // Clear which is which
    pub fn new(bank: Bank, customer: Customer) -> Self {
        Self { bank, customer }
    }
}

// 4. Hide implementation details
pub struct RequestId(Uuid);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    // Don't expose Uuid directly
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// 5. Add trait implementations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(u64);

impl UserId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn get(&self) -> u64 {
        self.0
    }
}

// Can use UserId as HashMap key
let mut users: HashMap<UserId, User> = HashMap::new();
```

**Rationale**: Newtypes are zero-cost abstractions that add type safety. They prevent common bugs without runtime overhead.

**Common newtype patterns**:
1. **Units**: Meters, Seconds, Bytes
2. **Constraints**: NonEmpty, Positive, Validated
3. **Semantic clarity**: UserId vs GroupId vs RequestId
4. **Hide dependencies**: Wrap external types
5. **Add traits**: Make non-Clone types Clone-able

**See also**: C-NEWTYPE

---

### Newtype Best Practices

**Strength**: SHOULD

**Summary**: Newtypes should provide accessor methods and implement common traits.

**Example**:
```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Temperature(i32);

impl Temperature {
    /// Creates a new temperature in Celsius
    pub fn celsius(value: i32) -> Self {
        Self(value)
    }
    
    /// Creates a temperature from Fahrenheit
    pub fn fahrenheit(value: i32) -> Self {
        Self((value - 32) * 5 / 9)
    }
    
    /// Gets the temperature in Celsius
    pub fn as_celsius(&self) -> i32 {
        self.0
    }
    
    /// Gets the temperature in Fahrenheit
    pub fn as_fahrenheit(&self) -> i32 {
        self.0 * 9 / 5 + 32
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°C", self.0)
    }
}

// Good trait implementations
impl Default for Temperature {
    fn default() -> Self {
        Self::celsius(20)  // Room temperature
    }
}

// Allow conversion from inner type when unambiguous
impl From<i32> for Temperature {
    fn from(celsius: i32) -> Self {
        Self::celsius(celsius)
    }
}

// Usage
let temp = Temperature::celsius(25);
println!("{}", temp);  // "25°C"

let temp2 = Temperature::fahrenheit(77);
assert_eq!(temp, temp2);

let temp3: Temperature = 25.into();
```

**Rationale**: Good accessor methods and trait implementations make newtypes ergonomic to use while maintaining encapsulation.

**Standard traits to consider**:
- `Debug` (always)
- `Clone` / `Copy` (if inner type supports it)
- `PartialEq` / `Eq` (for comparisons)
- `PartialOrd` / `Ord` (for ordering)
- `Hash` (for use in HashMap/HashSet)
- `Display` (for user-facing types)
- `Default` (if there's a sensible default)
- `From` / `Into` (for conversions)

**See also**: C-COMMON-TRAITS

---

## Common Traits

### Eagerly Implement Common Traits

**Strength**: SHOULD

**Summary**: Public types should implement standard traits when semantically appropriate.

**Example**:
```rust
// Implement all applicable traits
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(u64);

#[derive(Debug, Clone, PartialEq)]
pub struct UserData {
    name: String,
    email: String,
}

// Implement Default when there's a sensible default
#[derive(Debug, Clone, Default)]
pub struct Config {
    timeout: Option<Duration>,
    retry_count: Option<u32>,
}

// Manual implementation when needed
impl PartialOrd for Temperature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Temperature {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
```

**Rationale**: Implementing standard traits makes types composable and usable in generic contexts. Users expect common types to work with standard library collections and algorithms.

**Common trait checklist**:
- `Debug` - Always implement (required for development)
- `Clone` - If copying makes sense
- `Copy` - Only for trivial types (prefer Clone)
- `PartialEq` / `Eq` - For comparison
- `PartialOrd` / `Ord` - For ordering
- `Hash` - For use as map keys or in sets
- `Default` - If there's an obvious default
- `Display` - For user-facing types

**See also**: C-COMMON-TRAITS

---

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| Public types have Debug | MUST | Use custom impl for sensitive data |
| User-facing types have Display | MUST | Implement for errors and identifiers |
| Use strong types | SHOULD | PathBuf not String, Duration not u64 |
| Newtype for semantics | SHOULD | Prevent parameter confusion |
| Newtype for invariants | SHOULD | Validate at construction |
| Provide accessors | SHOULD | Methods to get inner value |
| Implement common traits | SHOULD | Make types composable |

## Type Design Checklist

When creating a new type, consider:

```rust
// 1. Should this be a newtype?
pub struct UserId(u64);  // vs u64

// 2. What traits make sense?
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

// 3. Does it need validation?
impl UserId {
    pub fn new(id: u64) -> Result<Self, ValidationError> {
        // Validate...
    }
}

// 4. Should it have a Display?
impl Display for UserId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

// 5. What accessors are needed?
impl UserId {
    pub fn get(&self) -> u64 { self.0 }
}

// 6. Are there useful constructors?
impl UserId {
    pub fn from_string(s: &str) -> Result<Self, ParseError> {
        // Parse...
    }
}

// 7. Should it be Copy or Clone?
// Clone if it owns data, Copy if trivial

// 8. Does it need Default?
impl Default for Config {
    fn default() -> Self {
        // Sensible defaults
    }
}
```

## Related Guidelines

- **Core Idioms**: See `01-core-idioms.md` for Debug trait
- **API Design**: See `02-api-design.md` for type simplicity
- **Error Handling**: See `03-error-handling.md` for error Display

## External References

- [Rust API Guidelines - Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html)
- [Newtype Pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
- Pragmatic Rust: M-PUBLIC-DEBUG, M-PUBLIC-DISPLAY, M-STRONG-TYPES
