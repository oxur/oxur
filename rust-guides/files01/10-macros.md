# Macros

> Patterns for declarative (`macro_rules!`) and procedural macros.

---

## MC-01: When to Use Macros

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

**Valid reasons for macros**:
- Variadic arguments: `println!("x={}, y={}", x, y)`
- Code generation: `#[derive(Debug)]`
- DSLs: `html! { <div class="foo">...</div> }`
- Compile-time computation: `include_str!`
- Reducing boilerplate that can't be abstracted otherwise

---

## MC-02: Declarative Macro Basics

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

**Fragment specifiers**:

| Specifier | Matches |
|-----------|---------|
| `expr` | Expressions |
| `ty` | Types |
| `ident` | Identifiers |
| `path` | Paths (`std::vec::Vec`) |
| `tt` | Single token tree |
| `literal` | Literals |
| `stmt` | Statements |
| `block` | Blocks (`{ ... }`) |
| `item` | Items (fn, struct, etc.) |
| `pat` | Patterns |

---

## MC-03: Hygiene and Scope

**Strength**: MUST (understand)

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

## MC-04: The `$crate` Meta-Variable

**Strength**: MUST (for library macros)

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

## MC-05: Repetition Patterns

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

## MC-06: TT Muncher Pattern

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

## MC-07: Procedural Macros Overview

**Strength**: SHOULD (understand)

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

## MC-08: Derive Macro Best Practices

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

## MC-11: Exporting Macros

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

## MC-12: Common Macro Patterns

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

## Summary: Macro Decision Guide

**Use `macro_rules!` when**:
- Variadic arguments needed
- Simple pattern-based code generation
- Performance-critical (zero runtime cost)
- Compile-time string manipulation

**Use proc macros when**:
- Complex parsing needed
- Access to type information
- Custom derive implementations
- Generating code from external data (files, etc.)

**Avoid macros when**:
- A function would work
- Generics can express the abstraction
- Trait-based polymorphism is sufficient

**Checklist**:
- [ ] Use `$crate` for paths in exported macros
- [ ] Use fully qualified paths (`::std::...`)
- [ ] Handle trailing commas: `$(,)?`
- [ ] Document macro syntax in doc comments
- [ ] Test macro expansion with `cargo expand`

---

*See also: [01-core-idioms.md](01-core-idioms.md) for when NOT to use macros.*
