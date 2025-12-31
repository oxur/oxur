# Macro Guidelines

Guidelines for writing and using macros in Rust. Note: The Pragmatic Rust Guidelines have limited macro-specific content; this primarily covers macro usage patterns and general principles.


## MC-01: Choose the Right Macro Type

**Strength**: SHOULD

**Summary**: Use declarative macros (`macro_rules!`) for simple patterns; use procedural macros for complex transformations.

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

---

## MC-02: Common Macro Patterns

**Strength**: CONSIDER

**Summary**: Useful patterns that appear frequently.

```rust
// ✅ Callback pattern: Let user provide the macro to call
macro_rules! with_items {
    ($callback:ident) => {
        $callback!(apple, banana, cherry)
    };
}

macro_rules! make_enum {
    ($($item:ident),*) => {
        enum Fruit { $($item),* }
    };
}

with_items!(make_enum);  // Creates enum Fruit { apple, banana, cherry }

// ✅ Internal rules pattern: Use @name for internal rules
macro_rules! complex {
    // Public entry point
    ($($input:tt)*) => {
        complex!(@parse [] $($input)*)
    };
    // Internal parsing rule
    (@parse [$($acc:tt)*] $first:tt $($rest:tt)*) => {
        complex!(@parse [$($acc)* $first] $($rest)*)
    };
    // Internal base case
    (@parse [$($acc:tt)*]) => {
        [$($acc)*]
    };
}

// ✅ Push-down accumulation: Build up result in macro argument
macro_rules! reverse {
    // Entry
    ([$($input:tt)*]) => {
        reverse!(@ [] $($input)*)
    };
    // Accumulate
    (@ [$($acc:tt)*] $first:tt $($rest:tt)*) => {
        reverse!(@ [$first $($acc)*] $($rest)*)
    };
    // Done
    (@ [$($acc:tt)*]) => {
        [$($acc)*]
    };
}

// reverse!([a b c]) => [c b a]
```

---

## MC-03: Declarative Macro Basics

**Strength**: SHOULD

**Summary**: Use `macro_rules!` for pattern-based code generation.

```rust
// ✅ Basic pattern matching
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

say_hello!();          // Hello!
say_hello!("world");   // Hello, world!

// ✅ Repetition patterns
macro_rules! create_struct {
    ($name:ident { $($field:ident : $type:ty),* $(,)? }) => {
        #[derive(Debug)]
        struct $name {
            $($field: $type),*
        }
    };
}

create_struct!(Point { x: f64, y: f64 });

// ✅ Multiple match arms (most specific first)
macro_rules! calculate {
    (add $a:expr, $b:expr) => { $a + $b };
    (mul $a:expr, $b:expr) => { $a * $b };
    ($a:expr) => { $a };
}

let x = calculate!(add 1, 2);  // 3
let y = calculate!(mul 3, 4);  // 12
let z = calculate!(5);         // 5
```

---

## MC-04: Derive Macro Best Practices

**Strength**: SHOULD

**Summary**: Follow conventions for derive macros.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = syn::Ident::new(
        &format!("{}Builder", name),
        name.span()
    );
    
    // Handle different data types
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Builder only supports structs with named fields"),
        },
        _ => panic!("Builder only supports structs"),
    };
    
    // Generate field definitions for builder
    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: Option<#ty> }
    });
    
    // Generate setters
    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        }
    });
    
    // Generate build method
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.ok_or(concat!(stringify!(#name), " is required"))?
        }
    });
    
    quote! {
        pub struct #builder_name {
            #(#builder_fields),*
        }
        
        impl #builder_name {
            #(#setters)*
            
            pub fn build(self) -> Result<#name, &'static str> {
                Ok(#name {
                    #(#build_fields),*
                })
            }
        }
        
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#fields.ident: None),*
                }
            }
        }
    }.into()
}
```

---

## MC-05: Document Macro Behavior

**Strength**: MUST

**Summary**: Macros must have comprehensive documentation explaining syntax, behavior, and examples.

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

**Rationale**: Macros have non-standard syntax that needs clear explanation. Good documentation makes macros approachable for users.

---

---

## MC-06: Exporting Macros

**Strength**: SHOULD

**Summary**: Export macros correctly for use in other crates.

```rust
// In lib.rs of your crate:

// Method 1: #[macro_export] (puts in crate root)
#[macro_export]
macro_rules! my_public_macro {
    () => { /* ... */ };
}
// Users: use your_crate::my_public_macro;

// Method 2: Re-export with pub use
mod macros {
    macro_rules! internal_macro {
        () => { /* ... */ };
    }
    pub(crate) use internal_macro;
}

#[macro_export]
macro_rules! public_wrapper {
    () => {
        $crate::macros::internal_macro!()
    };
}

// For proc macros: they're automatically exported
// Just make sure Cargo.toml has proc-macro = true
```

---

## MC-07: Follow Macro Naming Conventions

**Strength**: SHOULD

**Summary**: Use lowercase with underscores for macro names (like functions). Macros invoked like derives should be UpperCamelCase.

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

---

## MC-08: Hygiene and Scope

**Strength**: MUST

**Summary**: Macros are partially hygienic — understand what leaks.

```rust
// ✅ HYGIENIC: Local variables don't leak
macro_rules! five {
    () => {{
        let x = 5;  // This x is local to the macro
        x
    }};
}

let x = 10;
let y = five!();  // y = 5, x still = 10

// ⚠️ IDENTIFIERS PASSED IN are not hygienic
macro_rules! set_to_five {
    ($var:ident) => {
        $var = 5;  // Modifies the passed identifier
    };
}

let mut x = 10;
set_to_five!(x);  // x is now 5

// ✅ Use fully qualified paths for reliability
macro_rules! new_vec {
    () => {
        ::std::vec::Vec::new()  // Always refers to std::vec::Vec
    };
}

// ❌ BAD: Might refer to user's Vec
macro_rules! bad_new_vec {
    () => {
        Vec::new()  // Could be shadowed!
    };
}
```

---

## MC-09: Macro Debugging

**Strength**: SHOULD

**Summary**: Use tools to debug macro expansion.

```rust
// 1. cargo-expand: See expanded code
// $ cargo install cargo-expand
// $ cargo expand

// 2. trace_macros! (nightly)
#![feature(trace_macros)]
trace_macros!(true);
my_macro!(args);
trace_macros!(false);

// 3. log_syntax! (nightly) 
#![feature(log_syntax)]
macro_rules! debug_macro {
    ($($tt:tt)*) => {
        log_syntax!($($tt)*);  // Prints during compilation
        // actual expansion...
    };
}

// 4. Compile error for inspection
macro_rules! inspect {
    ($($tt:tt)*) => {
        compile_error!(stringify!($($tt)*));
    };
}

// 5. For proc macros: use eprintln! in the macro
#[proc_macro]
pub fn debug_proc_macro(input: TokenStream) -> TokenStream {
    eprintln!("Input: {}", input);
    input
}
```

---

## MC-10: Macro Hygiene and Captures

**Strength**: MUST

**Summary**: Avoid unintended name collisions.

```rust
// ❌ BAD: May capture user's variable
macro_rules! bad_increment {
    ($e:expr) => {
        {
            let temp = $e;  // What if user has `temp`?
            temp + 1
        }
    };
}

// ✅ GOOD: Use unlikely names or fresh identifiers
macro_rules! good_increment {
    ($e:expr) => {
        {
            let __increment_temp = $e;
            __increment_temp + 1
        }
    };
}

// ✅ BEST (proc macros): Use Span::call_site() or Span::mixed_site()
// to control hygiene explicitly

// ❌ BAD: Macro tries to use items not in scope
macro_rules! use_regex {
    ($pattern:expr) => {
        Regex::new($pattern)  // User needs `use regex::Regex`
    };
}

// ✅ GOOD: Use fully qualified path
macro_rules! use_regex {
    ($pattern:expr) => {
        ::regex::Regex::new($pattern)
    };
}
```

---

## MC-11: Macros Should Be Hygienic

**Strength**: MUST

**Summary**: Macros must not capture identifiers from the calling scope unless explicitly intended.

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

---

## MC-12: Macros Should Provide Clear Error Messages

**Strength**: SHOULD

**Summary**: Use `compile_error!` to provide helpful error messages for invalid macro input.

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

---

## MC-13: Note on Coverage

**Strength**: SHOULD

**Summary**: 

---

## MC-14: Prefer Functions Over Macros

**Strength**: SHOULD

**Summary**: Use regular functions, generic functions, or traits instead of macros when possible. Reserve macros for cases where functions cannot work.

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

**Rationale**: Functions are easier to read, debug, type-check, and provide better error messages. Macros should be a last resort when the type system or syntax cannot express what you need.

---

---

## MC-15: Procedural Macros Overview

**Strength**: SHOULD

**Summary**: Use proc macros for derive, attributes, and function-like macros.

```rust
// Procedural macros live in a separate crate with proc-macro = true
// Cargo.toml:
// [lib]
// proc-macro = true

// Three types of proc macros:

// 1. Derive macros: #[derive(MyTrait)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyTrait)]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    quote! {
        impl MyTrait for #name {
            fn my_method(&self) {
                println!("Called on {}", stringify!(#name));
            }
        }
    }.into()
}

// 2. Attribute macros: #[my_attribute]
#[proc_macro_attribute]
pub fn my_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr contains the attribute arguments
    // item contains the annotated item
    item  // Return modified item
}

// 3. Function-like macros: my_macro!(...)
#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse and transform input
    input
}
```

---

## MC-16: Repetition Patterns

**Strength**: SHOULD

**Summary**: Use `$(...)` for repeating patterns.

```rust
// ✅ Zero or more: $(...)* 
macro_rules! make_vec {
    ($($elem:expr),* $(,)?) => {
        {
            let mut v = Vec::new();
            $(v.push($elem);)*
            v
        }
    };
}

let v = make_vec![1, 2, 3];  // Vec containing 1, 2, 3

// ✅ One or more: $(...)+
macro_rules! min {
    ($first:expr $(, $rest:expr)+) => {
        {
            let mut min = $first;
            $(
                if $rest < min {
                    min = $rest;
                }
            )+
            min
        }
    };
    // Single element case
    ($single:expr) => { $single };
}

let m = min!(5, 2, 8, 1);  // 1

// ✅ Optional: $(...)? 
macro_rules! make_fn {
    ($name:ident $(, $ret:ty)?) => {
        fn $name() $(-> $ret)? {
            Default::default()
        }
    };
}

make_fn!(no_return);        // fn no_return() { ... }
make_fn!(with_return, i32); // fn with_return() -> i32 { ... }

// ✅ Nested repetition
macro_rules! make_structs {
    ($(
        struct $name:ident {
            $($field:ident : $ty:ty),* $(,)?
        }
    )*) => {
        $(
            struct $name { $($field: $ty),* }
        )*
    };
}
```

---

## MC-17: Test Macro Expansion

**Strength**: SHOULD

**Summary**: Test that macros expand to the expected code and handle edge cases correctly.

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

---

## MC-18: The `$crate` Meta-Variable

**Strength**: MUST

**Summary**: Use `$crate` to refer to your crate in exported macros.

```rust
// In your library crate:

pub fn helper() {
    println!("Helper called");
}

#[macro_export]
macro_rules! my_macro {
    () => {
        // ❌ BAD: Won't work when called from other crates
        // crate::helper();
        
        // ✅ GOOD: Always refers to your crate
        $crate::helper();
    };
}

// In user's code:
use your_crate::my_macro;
my_macro!();  // Works because $crate points to your_crate
```

---

## MC-19: TT Muncher Pattern

**Strength**: CONSIDER

**Summary**: Process tokens recursively for complex parsing.

```rust
// TT Muncher: Process tokens one at a time
macro_rules! count_tokens {
    () => { 0 };
    ($first:tt $($rest:tt)*) => {
        1 + count_tokens!($($rest)*)
    };
}

let c = count_tokens!(a b c d);  // 4

// ✅ Accumulator pattern for complex parsing
macro_rules! parse_key_values {
    // Base case: done
    (@accum ($($acc:tt)*) ) => {
        vec![$($acc)*]
    };
    // Recursive case: consume one key-value pair
    (@accum ($($acc:tt)*) $key:ident = $value:expr, $($rest:tt)*) => {
        parse_key_values!(@accum ($($acc)* (stringify!($key), $value),) $($rest)*)
    };
    // Entry point
    ($($tokens:tt)*) => {
        parse_key_values!(@accum () $($tokens)*)
    };
}

let pairs = parse_key_values!(
    name = "Alice",
    age = 30,
);
// vec![("name", "Alice"), ("age", 30)]
```

---

## MC-20: When to Use Macros

**Strength**: CONSIDER

**Summary**: Use macros only when functions and generics aren't sufficient.

```rust
// ❌ UNNECESSARY: Function would work
macro_rules! add {
    ($a:expr, $b:expr) => { $a + $b };
}

// ✅ BETTER: Just use a function
fn add(a: i32, b: i32) -> i32 { a + b }
```

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