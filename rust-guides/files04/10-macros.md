# Macro Guidelines

Guidelines for writing and using macros in Rust. Note: The Pragmatic Rust Guidelines have limited macro-specific content; this primarily covers macro usage patterns and general principles.

## Table of Contents

- [When to Use Macros](#when-to-use-macros)
- [Macro Hygiene](#macro-hygiene)
- [Error Messages](#error-messages)
- [Testing Macros](#testing-macros)

---

## When to Use Macros

### Prefer Functions Over Macros

**Strength**: SHOULD

**Summary**: Use regular functions, generic functions, or traits instead of macros when possible. Reserve macros for cases where functions cannot work.

**Example**:
```rust
// Bad - macro when function works fine
macro_rules! add {
    ($a:expr, $b:expr) => {
        $a + $b
    };
}

// Good - use a function
fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

// Good - macro for syntax that functions can't provide
macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

// Usage: syntax sugar functions can't provide
let map = hashmap! {
    "a" => 1,
    "b" => 2,
};
```

**Valid macro use cases**:
1. **Syntax extension**: Create DSLs or syntax sugar
2. **Variadic functions**: Accept variable number of arguments
3. **Compile-time code generation**: Generate code based on patterns
4. **Custom derives**: Implement traits automatically
5. **Const evaluation**: Evaluate at compile time

**Rationale**: Functions are easier to read, debug, type-check, and provide better error messages. Macros should be a last resort when the type system or syntax cannot express what you need.

---

## Macro Hygiene

### Macros Should Be Hygienic

**Strength**: MUST

**Summary**: Macros must not capture identifiers from the calling scope unless explicitly intended.

**Example**:
```rust
// Bad - captures 'result' from outer scope
macro_rules! check_bad {
    ($expr:expr) => {
        let result = $expr; // ❌ Could conflict with user's 'result'
        if !result {
            panic!("Check failed");
        }
    };
}

// Good - use unique identifier
macro_rules! check_good {
    ($expr:expr) => {
        let __macro_result = $expr; // Unique name
        if !__macro_result {
            panic!("Check failed");
        }
    };
}

// Better - use hygiene from macro 2.0
// Declarative macros are hygienic by default for most cases
macro_rules! check {
    ($expr:expr) => {{
        let result = $expr; // Hygienic - won't conflict
        if !result {
            panic!("Check failed");
        }
    }};
}
```

**Rationale**: Non-hygienic macros can accidentally capture or shadow variables from the call site, causing subtle bugs. Use unique identifiers or rely on macro hygiene.

---

## Error Messages

### Macros Should Provide Clear Error Messages

**Strength**: SHOULD

**Summary**: Use `compile_error!` to provide helpful error messages for invalid macro input.

**Example**:
```rust
macro_rules! create_config {
    // Valid pattern
    (name: $name:expr, value: $value:expr) => {
        Config {
            name: $name.to_string(),
            value: $value,
        }
    };
    
    // Helpful error for common mistakes
    ($($tt:tt)*) => {
        compile_error!(
            "Invalid syntax. Expected: create_config!(name: \"...\", value: ...)"
        )
    };
}

// Procedural macro error handling
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyDerive)]
pub fn my_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Validate input
    if !matches!(input.data, syn::Data::Struct(_)) {
        return quote! {
            compile_error!("MyDerive can only be used on structs");
        }.into();
    }
    
    // Generate code...
    quote! { /* ... */ }.into()
}
```

**Rationale**: Macro errors can be cryptic. Clear error messages save users time debugging and improve the macro's usability.

---

## Testing Macros

### Test Macro Expansion

**Strength**: SHOULD

**Summary**: Test that macros expand to the expected code and handle edge cases correctly.

**Example**:
```rust
macro_rules! repeat {
    ($val:expr; $count:expr) => {{
        let mut vec = Vec::new();
        for _ in 0..$count {
            vec.push($val);
        }
        vec
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repeat_macro() {
        let result = repeat!(5; 3);
        assert_eq!(result, vec![5, 5, 5]);
        
        let result = repeat!(0; 0);
        assert_eq!(result, vec![]);
    }
    
    #[test]
    fn test_repeat_with_expression() {
        let mut counter = 0;
        let result = repeat!({ counter += 1; counter }; 3);
        // Each invocation evaluates the expression
        assert_eq!(result, vec![1, 2, 3]);
    }
}

// For procedural macros, use trybuild for compile tests
#[test]
fn test_derive_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_*.rs");
    t.pass("tests/ui/pass_*.rs");
}
```

**Rationale**: Macros can have subtle expansion issues. Tests ensure they work correctly across different inputs and catch regressions.

---

## Documentation

### Document Macro Behavior

**Strength**: MUST

**Summary**: Macros must have comprehensive documentation explaining syntax, behavior, and examples.

**Example**:
```rust
/// Creates a HashMap with the specified key-value pairs.
///
/// # Syntax
///
/// ```ignore
/// hashmap! {
///     key1 => value1,
///     key2 => value2,
/// }
/// ```
///
/// # Examples
///
/// ```
/// # use my_crate::hashmap;
/// let map = hashmap! {
///     "a" => 1,
///     "b" => 2,
/// };
/// assert_eq!(map.get("a"), Some(&1));
/// ```
///
/// # Notes
///
/// - Keys and values can be any expression
/// - Trailing comma is optional
/// - The map's type is inferred from usage
#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = ::std::collections::HashMap::new();
        $(
            map.insert($key, $value);
        )*
        map
    }};
}
```

**Required documentation**:
1. **Purpose**: What the macro does
2. **Syntax**: Valid input patterns
3. **Examples**: Common use cases
4. **Edge cases**: Limitations or special behaviors

**Rationale**: Macros have non-standard syntax that needs clear explanation. Good documentation makes macros approachable for users.

---

## Declarative vs Procedural

### Choose the Right Macro Type

**Strength**: SHOULD

**Summary**: Use declarative macros (`macro_rules!`) for simple patterns; use procedural macros for complex transformations.

**When to use declarative macros**:
- Pattern matching on syntax
- Simple code generation
- Syntax sugar for common patterns
- Most variadic functions

**When to use procedural macros**:
- Derive macros (auto-implementing traits)
- Attribute macros (modifying items)
- Complex parsing and validation
- Need to inspect type information
- Generate code based on struct fields

**Example**:
```rust
// Declarative - simple pattern
macro_rules! vec_of_strings {
    ($($x:expr),*) => {
        vec![$($x.to_string()),*]
    };
}

// Procedural - complex derive
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Complex parsing and code generation
    let name = input.ident;
    let fields = match input.data {
        syn::Data::Struct(s) => s.fields,
        _ => panic!("Builder only works on structs"),
    };
    
    // Generate builder struct and methods...
    quote! {
        // Generated code
    }.into()
}
```

**Rationale**: Declarative macros are simpler for pattern-based generation. Procedural macros provide more power but require separate crates and more setup.

---

## Macro Invocation

### Follow Macro Naming Conventions

**Strength**: SHOULD

**Summary**: Use lowercase with underscores for macro names (like functions). Macros invoked like derives should be UpperCamelCase.

**Example**:
```rust
// Good - function-like macro
macro_rules! create_parser {
    // ...
}

create_parser!(MyParser);

// Good - derive-like macro  
#[derive(Debug, Clone, MyDerive)]
struct Foo;

// Good - attribute-like macro
#[my_attribute]
fn handler() {}
```

**Rationale**: Consistent naming helps users understand how to invoke macros. Function-like macros follow function naming, derive/attribute macros follow type naming.

---

## Summary Table

| Pattern | Strength | Key Insight |
|---------|----------|-------------|
| Prefer functions over macros | SHOULD | Use macros only when functions can't work |
| Macros must be hygienic | MUST | Don't capture calling scope identifiers |
| Provide clear error messages | SHOULD | Use `compile_error!` for invalid input |
| Test macro expansion | SHOULD | Test edge cases and expected output |
| Document macro syntax | MUST | Include syntax, examples, and notes |
| Choose right macro type | SHOULD | Declarative for simple, procedural for complex |
| Follow naming conventions | SHOULD | Function-like = snake_case, Derive = PascalCase |

---

## Related Guidelines

- **API Design**: See `02-api-design.md` for builder patterns (often macro-generated)
- **Documentation**: See `13-documentation.md` for documentation best practices
- **Anti-patterns**: See `11-anti-patterns.md` for macro misuse

---

## External References

- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
- [Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html)
- [Declarative Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)

---

## Note on Coverage

The Pragmatic Rust Guidelines have limited macro-specific content. For comprehensive macro guidance, refer to The Little Book of Rust Macros and the official Rust documentation on macros. This guide focuses on general principles that apply when using or creating macros.
