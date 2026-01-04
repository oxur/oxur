---
number: 36
title: "oxur-ast Phase 8.5: Macros & Attributes Completeness"
author: "Claude Code"
component: AST
tags: [macros]
created: 2026-01-03
updated: 2026-01-04
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 8.5: Macros & Attributes Completeness

**Status**: Planned
**Estimated Effort**: 8-10 hours (1-2 days)
**Expected Coverage Gain**: +5-10% for real-world codebases
**Complexity**: Medium-High - Token stream handling is intricate
**Prerequisite**: Phase 8 Priority 1 & 2 complete
**Relationship**: Fills the gap between Phase 8 and Phases 9-13

## Overview

This phase implements **Phase 8 Priority 3: Macros & Attributes**, which is NOT covered by the Phases 9-13 roadmap. This work is essential for parsing real-world Rust code that uses derive macros, cfg attributes, and declarative macros.

### What This Phase Adds

1. **Enhanced macro call support** with token stream preservation
2. **Comprehensive attribute system** (derive, cfg, doc comments)
3. **Declarative macro definitions** (`macro_rules!`)

### Why This Matters

Current state can parse basic macro calls but lacks:

- ❌ Token stream preservation for complex macro arguments
- ❌ Derive macro attributes (`#[derive(Debug, Clone)]`)
- ❌ Cfg attributes (`#[cfg(test)]`, `#[cfg(target_os = "linux")]`)
- ❌ Inner vs outer attribute distinction
- ❌ `macro_rules!` definitions
- ❌ Doc comment handling

This prevents parsing ~40-60% of real Rust crates that use these features extensively.

## Current State Analysis

### What Exists

**File**: `crates/oxur-ast/src/ast/item.rs`

```rust
// MacCall already defined
pub struct MacCall {
    pub path: Path,
    pub args: MacArgs,
    pub prior_type_ascription: Option<(usize, bool)>,
}

pub enum MacArgs {
    Empty,
    Delimited(DelimSpan, MacDelimiter, TokenStream),
    Eq(Span, TokenStream),
}

// Items have attrs field
pub struct Item {
    pub attrs: Vec<Attribute>,
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ItemKind,
    pub tokens: Option<TokenStream>,
}
```

**File**: `crates/oxur-ast/src/ast/types.rs`

```rust
// Attribute structure exists
pub struct Attribute {
    pub kind: AttrKind,
    pub id: AttrId,
    pub style: AttrStyle,
    pub span: Span,
}

pub enum AttrStyle {
    Outer,  // #[...]
    Inner,  // #![...]
}

pub enum AttrKind {
    Normal(NormalAttr),
    DocComment(CommentKind, String),
}
```

### What's Missing

1. **Token stream handling** in MacCall conversion
2. **Attribute conversion** from syn to oxur AST
3. **MacroDef** structure for `macro_rules!`
4. **Generators** for attributes and macro definitions
5. **Integration tests** for real-world macro patterns

## Detailed Tasks

### Task 1: Enhanced Macro Call Support (3-4 hours)

**Priority**: HIGH - Blocks parsing of ~80% of Rust code with macros

#### 1.1: Verify MacCall Structure

**File**: `crates/oxur-ast/src/ast/item.rs`

The MacCall structure should already exist. Verify it has:

```rust
pub struct MacCall {
    pub path: Path,
    pub args: MacArgs,
    pub prior_type_ascription: Option<(usize, bool)>,
}

pub enum MacArgs {
    Empty,
    Delimited(DelimSpan, MacDelimiter, TokenStream),
    Eq(Span, TokenStream),
}

pub enum MacDelimiter {
    Paren,      // ( )
    Brace,      // { }
    Bracket,    // [ ]
}

pub struct DelimSpan {
    pub open: Span,
    pub close: Span,
}
```

If any pieces are missing, add them.

#### 1.2: Implement Token Stream Conversion

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

Add helper function to convert syn's MacCall:

```rust
fn convert_macro_call(&mut self, mac: &syn::Macro) -> Result<MacCall> {
    let path = self.convert_path(&mac.path)?;

    let args = self.convert_mac_args(&mac.tokens)?;

    Ok(MacCall {
        path,
        args,
        prior_type_ascription: None,
    })
}

fn convert_mac_args(&mut self, tokens: &proc_macro2::TokenStream) -> Result<MacArgs> {
    // Check if tokens are empty
    if tokens.is_empty() {
        return Ok(MacArgs::Empty);
    }

    // For now, preserve tokens as a string representation
    // More sophisticated parsing can be added later
    let token_stream = TokenStream::new(tokens.to_string());

    // Detect delimiter from token stream structure
    // Default to parentheses if ambiguous
    let delim = MacDelimiter::Paren;

    let delim_span = DelimSpan {
        open: Span::DUMMY,
        close: Span::DUMMY,
    };

    Ok(MacArgs::Delimited(delim_span, delim, token_stream))
}
```

**Note**: TokenStream may need to be updated:

**File**: `crates/oxur-ast/src/ast/types.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStream {
    pub source: String,  // Raw token representation
}

impl TokenStream {
    pub fn new(source: String) -> Self {
        TokenStream { source }
    }

    pub fn empty() -> Self {
        TokenStream { source: String::new() }
    }
}
```

#### 1.3: Handle Macro Calls in Expression Position

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

The `convert_expr()` function should already handle MacCall. Verify:

```rust
syn::Expr::Macro(expr_macro) => {
    let mac = self.convert_macro_call(&expr_macro.mac)?;
    ExprKind::MacCall(mac)
}
```

If missing, add it.

#### 1.4: Update Generators for MacCall

**File**: `crates/oxur-ast/src/gen_rs/expr.rs`

```rust
fn generate_expr(&mut self, expr: &Expr) -> String {
    match &expr.kind {
        // ... existing cases ...

        ExprKind::MacCall(mac) => {
            self.generate_macro_call(mac)
        }
    }
}

fn generate_macro_call(&mut self, mac: &MacCall) -> String {
    let path = self.generate_path(&mac.path);

    let args_str = match &mac.args {
        MacArgs::Empty => "()".to_string(),
        MacArgs::Delimited(_, delim, tokens) => {
            let (open, close) = match delim {
                MacDelimiter::Paren => ("(", ")"),
                MacDelimiter::Brace => ("{", "}"),
                MacDelimiter::Bracket => ("[", "]"),
            };
            format!("{}{}{}", open, tokens.source, close)
        }
        MacArgs::Eq(_, tokens) => {
            format!(" = {}", tokens.source)
        }
    };

    format!("{}!{}", path, args_str)
}
```

**File**: `crates/oxur-ast/src/gen_sexp/expr.rs`

```rust
fn generate_expr(&self, expr: &Expr) -> SExp {
    match &expr.kind {
        // ... existing cases ...

        ExprKind::MacCall(mac) => {
            list(&[
                sym("MacCall"),
                self.generate_path(&mac.path),
                self.generate_mac_args(&mac.args),
            ])
        }
    }
}

fn generate_mac_args(&self, args: &MacArgs) -> SExp {
    match args {
        MacArgs::Empty => sym("Empty"),
        MacArgs::Delimited(_, delim, tokens) => {
            list(&[
                sym("Delimited"),
                sym(match delim {
                    MacDelimiter::Paren => "Paren",
                    MacDelimiter::Brace => "Brace",
                    MacDelimiter::Bracket => "Bracket",
                }),
                string(&tokens.source),
            ])
        }
        MacArgs::Eq(_, tokens) => {
            list(&[
                sym("Eq"),
                string(&tokens.source),
            ])
        }
    }
}
```

#### 1.5: Test Cases for Macro Calls

**File**: `tests/phase8_5_macro_call_tests.rs` (NEW)

```rust
use oxur_ast::{Parser, RustCodegen, Generator};

#[test]
fn test_println_macro() {
    let code = r#"
        fn main() {
            println!("Hello, {}!", name);
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    // Verify macro call was parsed
    let sexp = Generator::new().generate(&ast);
    assert!(sexp.to_string().contains("MacCall"));
    assert!(sexp.to_string().contains("println"));

    // Round-trip
    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("println!"));
}

#[test]
fn test_vec_macro() {
    let code = r#"
        fn test() {
            let v = vec![1, 2, 3];
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("vec!"));
}

#[test]
fn test_assert_eq_macro() {
    let code = r#"
        fn test() {
            assert_eq!(a, b);
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("assert_eq!"));
}

#[test]
fn test_format_macro() {
    let code = r#"
        fn test() {
            let s = format!("x = {}", x);
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("format!"));
}

#[test]
fn test_macro_with_braces() {
    let code = r#"
        fn test() {
            thread::spawn(|| {
                println!("thread");
            });
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("println!"));
}

#[test]
fn test_nested_macros() {
    let code = r#"
        fn test() {
            println!("{:?}", vec![1, 2, 3]);
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("println!"));
    assert!(rust.contains("vec!"));
}
```

---

### Task 2: Comprehensive Attribute Support (3-4 hours)

**Priority**: HIGH - Derive macros used in ~90% of Rust structs/enums

#### 2.1: Verify Attribute Structure

**File**: `crates/oxur-ast/src/ast/types.rs`

Verify these structures exist:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub kind: AttrKind,
    pub id: AttrId,
    pub style: AttrStyle,
    pub span: Span,
}

pub type AttrId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrStyle {
    Outer,  // #[...]
    Inner,  // #![...]
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrKind {
    Normal(NormalAttr),
    DocComment(CommentKind, String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalAttr {
    pub item: AttrItem,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttrItem {
    pub path: Path,
    pub args: MacArgs,  // Reuse MacArgs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Line,
    Block,
}
```

Add any missing structures.

#### 2.2: Implement Attribute Conversion

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

Add attribute conversion functions:

```rust
fn convert_attributes(&mut self, attrs: &[syn::Attribute]) -> Result<Vec<Attribute>> {
    attrs.iter()
        .map(|attr| self.convert_attribute(attr))
        .collect()
}

fn convert_attribute(&mut self, attr: &syn::Attribute) -> Result<Attribute> {
    let style = match attr.style {
        syn::AttrStyle::Outer => AttrStyle::Outer,
        syn::AttrStyle::Inner(_) => AttrStyle::Inner,
    };

    let kind = if let Some(doc) = self.extract_doc_comment(attr) {
        AttrKind::DocComment(CommentKind::Line, doc)
    } else {
        let path = self.convert_path(&attr.path())?;
        let args = self.convert_attr_args(&attr.meta)?;

        let item = AttrItem { path, args };
        let normal = NormalAttr { item };
        AttrKind::Normal(normal)
    };

    Ok(Attribute {
        kind,
        id: 0,  // Will be assigned during AST building
        style,
        span: Span::DUMMY,
    })
}

fn extract_doc_comment(&self, attr: &syn::Attribute) -> Option<String> {
    // Check if this is a doc comment attribute
    if attr.path().is_ident("doc") {
        if let syn::Meta::NameValue(meta) = &attr.meta {
            if let syn::Expr::Lit(expr_lit) = &meta.value {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}

fn convert_attr_args(&mut self, meta: &syn::Meta) -> Result<MacArgs> {
    match meta {
        syn::Meta::Path(_) => Ok(MacArgs::Empty),

        syn::Meta::List(meta_list) => {
            // Convert tokens to our TokenStream
            let tokens = TokenStream::new(meta_list.tokens.to_string());
            let delim_span = DelimSpan {
                open: Span::DUMMY,
                close: Span::DUMMY,
            };
            Ok(MacArgs::Delimited(delim_span, MacDelimiter::Paren, tokens))
        }

        syn::Meta::NameValue(meta_nv) => {
            // For #[key = "value"] style
            let value = match &meta_nv.value {
                syn::Expr::Lit(expr_lit) => {
                    format!("{:?}", expr_lit.lit)
                }
                other => format!("{:?}", other),
            };
            let tokens = TokenStream::new(value);
            Ok(MacArgs::Eq(Span::DUMMY, tokens))
        }
    }
}
```

#### 2.3: Update Item Conversion to Include Attributes

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

Update all `convert_item_*` functions to handle attributes:

```rust
fn convert_item_struct(&mut self, item_struct: &syn::ItemStruct) -> Result<Item> {
    // Convert attributes FIRST
    let attrs = self.convert_attributes(&item_struct.attrs)?;

    let vis = self.convert_visibility(&item_struct.vis);
    let ident = self.convert_ident(&item_struct.ident);

    // ... rest of struct conversion ...

    Ok(Item {
        attrs,  // Include converted attributes
        id: NodeId::DUMMY,
        span: Span::DUMMY,
        vis,
        ident,
        kind: ItemKind::Struct(variant_data),
        tokens: None,
    })
}

// Do the same for:
// - convert_item_enum
// - convert_item_fn
// - convert_item_trait
// - convert_item_impl
// - convert_item_mod
// - etc.
```

#### 2.4: Generate Attributes in Rust Output

**File**: `crates/oxur-ast/src/gen_rs/item.rs`

```rust
fn generate_item(&mut self, item: &Item) -> String {
    let mut result = String::new();

    // Generate attributes first
    for attr in &item.attrs {
        result.push_str(&self.generate_attribute(attr));
        result.push('\n');
    }

    // Generate visibility
    if !matches!(item.vis, Visibility::Inherited) {
        result.push_str(&self.generate_visibility(&item.vis));
        result.push(' ');
    }

    // Generate item kind
    result.push_str(&self.generate_item_kind(&item.kind, &item.ident));

    result
}

fn generate_attribute(&self, attr: &Attribute) -> String {
    let prefix = match attr.style {
        AttrStyle::Outer => "#",
        AttrStyle::Inner => "#!",
    };

    let content = match &attr.kind {
        AttrKind::Normal(normal) => {
            let path = self.generate_path(&normal.item.path);
            let args = match &normal.item.args {
                MacArgs::Empty => String::new(),
                MacArgs::Delimited(_, delim, tokens) => {
                    let (open, close) = match delim {
                        MacDelimiter::Paren => ("(", ")"),
                        MacDelimiter::Brace => ("{", "}"),
                        MacDelimiter::Bracket => ("[", "]"),
                    };
                    format!("{}{}{}", open, tokens.source, close)
                }
                MacArgs::Eq(_, tokens) => {
                    format!(" = {}", tokens.source)
                }
            };
            format!("{}{}", path, args)
        }

        AttrKind::DocComment(kind, text) => {
            match kind {
                CommentKind::Line => format!("doc = \"{}\"", text),
                CommentKind::Block => format!("doc = \"{}\"", text),
            }
        }
    };

    format!("{}[{}]", prefix, content)
}
```

**File**: `crates/oxur-ast/src/gen_sexp/item.rs`

```rust
fn generate_item(&self, item: &Item) -> SExp {
    list(&[
        sym("Item"),
        keyword("attrs"),
        self.generate_attributes(&item.attrs),
        keyword("id"),
        self.generate_node_id(item.id),
        keyword("span"),
        self.generate_span(&item.span),
        keyword("vis"),
        self.generate_visibility(&item.vis),
        keyword("ident"),
        self.generate_ident(&item.ident),
        keyword("kind"),
        self.generate_item_kind(&item.kind),
    ])
}

fn generate_attributes(&self, attrs: &[Attribute]) -> SExp {
    list(
        &attrs.iter()
            .map(|attr| self.generate_attribute(attr))
            .collect::<Vec<_>>()
    )
}

fn generate_attribute(&self, attr: &Attribute) -> SExp {
    list(&[
        sym("Attribute"),
        keyword("kind"),
        self.generate_attr_kind(&attr.kind),
        keyword("id"),
        integer(attr.id as i64),
        keyword("style"),
        sym(match attr.style {
            AttrStyle::Outer => "Outer",
            AttrStyle::Inner => "Inner",
        }),
        keyword("span"),
        self.generate_span(&attr.span),
    ])
}

fn generate_attr_kind(&self, kind: &AttrKind) -> SExp {
    match kind {
        AttrKind::Normal(normal) => {
            list(&[
                sym("Normal"),
                list(&[
                    sym("AttrItem"),
                    keyword("path"),
                    self.generate_path(&normal.item.path),
                    keyword("args"),
                    self.generate_mac_args(&normal.item.args),
                ]),
            ])
        }

        AttrKind::DocComment(kind, text) => {
            list(&[
                sym("DocComment"),
                sym(match kind {
                    CommentKind::Line => "Line",
                    CommentKind::Block => "Block",
                }),
                string(text),
            ])
        }
    }
}
```

#### 2.5: Test Cases for Attributes

**File**: `tests/phase8_5_attribute_tests.rs` (NEW)

```rust
use oxur_ast::{Parser, RustCodegen, Generator};

#[test]
fn test_derive_macro() {
    let code = r#"
        #[derive(Debug, Clone, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let sexp = Generator::new().generate(&ast);
    assert!(sexp.to_string().contains("derive"));
    assert!(sexp.to_string().contains("Debug"));

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[derive(Debug, Clone, PartialEq)]"));
}

#[test]
fn test_cfg_test_attribute() {
    let code = r#"
        #[cfg(test)]
        mod tests {
            fn test_foo() {}
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[cfg(test)]"));
}

#[test]
fn test_cfg_target_os() {
    let code = r#"
        #[cfg(target_os = "linux")]
        fn platform_specific() {}
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[cfg(target_os"));
}

#[test]
fn test_multiple_attributes() {
    let code = r#"
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Data {
            value: i32,
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[derive(Debug)]"));
    assert!(rust.contains("#[allow(dead_code)]"));
}

#[test]
fn test_inner_attribute() {
    let code = r#"
        mod example {
            #![allow(unused)]

            fn helper() {}
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#![allow(unused)]"));
}

#[test]
fn test_doc_comment() {
    let code = r#"
        /// This is a doc comment
        /// on multiple lines
        fn documented() {}
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let sexp = Generator::new().generate(&ast);
    assert!(sexp.to_string().contains("DocComment"));
}

#[test]
fn test_attribute_on_enum() {
    let code = r#"
        #[derive(Debug, Clone)]
        enum Status {
            Active,
            Inactive,
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[derive(Debug, Clone)]"));
}

#[test]
fn test_attribute_on_impl() {
    let code = r#"
        #[cfg(feature = "std")]
        impl Point {
            fn new() -> Self {
                Point { x: 0, y: 0 }
            }
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[cfg(feature"));
}
```

---

### Task 3: Declarative Macro Definitions (2-3 hours)

**Priority**: MEDIUM - Less common but needed for completeness

#### 3.1: Add MacroDef to ItemKind

**File**: `crates/oxur-ast/src/ast/item.rs`

Check if `ItemKind` has MacroDef variant. If not, add:

```rust
pub enum ItemKind {
    // ... existing variants ...

    /// Declarative macro definition: `macro_rules! name { ... }`
    MacroDef(Box<MacroDef>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub macro_rules: bool,  // true for macro_rules!, false for macro
    pub body: MacArgs,      // Token stream of macro body
}
```

#### 3.2: Implement Macro Definition Conversion

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

Add to the `convert_item()` match:

```rust
syn::Item::Macro(item_macro) => {
    self.convert_item_macro(item_macro)
}
```

Add conversion function:

```rust
fn convert_item_macro(&mut self, item_macro: &syn::ItemMacro) -> Result<Item> {
    let attrs = self.convert_attributes(&item_macro.attrs)?;

    let ident = if let Some(ref i) = item_macro.ident {
        self.convert_ident(i)
    } else {
        // Anonymous macro - generate a placeholder name
        Ident::new("_macro", Span::DUMMY)
    };

    let body = self.convert_mac_args(&item_macro.mac.tokens)?;

    let macro_def = MacroDef {
        macro_rules: item_macro.mac.path.is_ident("macro_rules"),
        body,
    };

    Ok(Item {
        attrs,
        id: NodeId::DUMMY,
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident,
        kind: ItemKind::MacroDef(Box::new(macro_def)),
        tokens: None,
    })
}
```

#### 3.3: Generate Macro Definitions

**File**: `crates/oxur-ast/src/gen_rs/item.rs`

```rust
fn generate_item_kind(&mut self, kind: &ItemKind, ident: &Ident) -> String {
    match kind {
        // ... existing cases ...

        ItemKind::MacroDef(macro_def) => {
            let keyword = if macro_def.macro_rules {
                "macro_rules!"
            } else {
                "macro"
            };

            let body = match &macro_def.body {
                MacArgs::Delimited(_, delim, tokens) => {
                    let (open, close) = match delim {
                        MacDelimiter::Paren => ("(", ")"),
                        MacDelimiter::Brace => ("{", "}"),
                        MacDelimiter::Bracket => ("[", "]"),
                    };
                    format!("{}{}{}", open, tokens.source, close)
                }
                _ => "{}".to_string(),
            };

            format!("{} {} {}", keyword, ident.name, body)
        }
    }
}
```

**File**: `crates/oxur-ast/src/gen_sexp/item.rs`

```rust
fn generate_item_kind(&self, kind: &ItemKind) -> SExp {
    match kind {
        // ... existing cases ...

        ItemKind::MacroDef(macro_def) => {
            list(&[
                sym("MacroDef"),
                keyword("macro_rules"),
                boolean(macro_def.macro_rules),
                keyword("body"),
                self.generate_mac_args(&macro_def.body),
            ])
        }
    }
}
```

#### 3.4: Test Cases for Macro Definitions

**File**: `tests/phase8_5_macro_def_tests.rs` (NEW)

```rust
use oxur_ast::{Parser, RustCodegen, Generator};

#[test]
fn test_simple_macro_rules() {
    let code = r#"
        macro_rules! say_hello {
            () => {
                println!("Hello!");
            }
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let sexp = Generator::new().generate(&ast);
    assert!(sexp.to_string().contains("MacroDef"));

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("macro_rules!"));
    assert!(rust.contains("say_hello"));
}

#[test]
fn test_vec_macro_definition() {
    let code = r#"
        macro_rules! vec {
            ( $( $x:expr ),* ) => {
                {
                    let mut temp_vec = Vec::new();
                    $(
                        temp_vec.push($x);
                    )*
                    temp_vec
                }
            };
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("macro_rules! vec"));
}

#[test]
fn test_macro_with_multiple_patterns() {
    let code = r#"
        macro_rules! max {
            ($x:expr) => ($x);
            ($x:expr, $($y:expr),+) => {
                std::cmp::max($x, max!($($y),+))
            }
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("macro_rules! max"));
}

#[test]
fn test_macro_round_trip() {
    let code = r#"
        macro_rules! create_function {
            ($func_name:ident) => {
                fn $func_name() {
                    println!("function called");
                }
            }
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    // Round-trip test
    let rust = RustCodegen::new().generate(&ast);
    let ast2 = parser.parse(&rust).unwrap();

    let sexp1 = Generator::new().generate(&ast);
    let sexp2 = Generator::new().generate(&ast2);

    assert_eq!(sexp1.to_string(), sexp2.to_string());
}
```

---

## Testing Strategy

### Unit Tests

Create 3 new test files:

1. **tests/phase8_5_macro_call_tests.rs** (6+ tests)
   - Basic macro calls (println!, vec!, assert_eq!, format!)
   - Different delimiters
   - Nested macros
   - Macro in expression position

2. **tests/phase8_5_attribute_tests.rs** (8+ tests)
   - Derive macros
   - Cfg attributes (test, target_os, feature)
   - Multiple attributes
   - Inner vs outer attributes
   - Doc comments
   - Attributes on different item types

3. **tests/phase8_5_macro_def_tests.rs** (4+ tests)
   - Simple macro_rules! definitions
   - Macros with multiple patterns
   - Macros with repetition operators
   - Round-trip tests

**Total New Tests**: 18-20 tests minimum

### Integration Tests

**File**: `tests/phase8_5_integration_tests.rs` (NEW)

```rust
use oxur_ast::{Parser, RustCodegen, Generator};

#[test]
fn test_real_world_struct_with_derives() {
    let code = r#"
        /// A point in 2D space
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub struct Point {
            pub x: i32,
            pub y: i32,
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[derive(Debug, Clone, Copy"));
    assert!(rust.contains("#[cfg_attr(feature"));

    // Round-trip
    let ast2 = parser.parse(&rust).unwrap();
    let rust2 = RustCodegen::new().generate(&ast2);
    assert_eq!(rust, rust2);
}

#[test]
fn test_test_module_pattern() {
    let code = r#"
        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_addition() {
                assert_eq!(2 + 2, 4);
            }

            #[test]
            #[should_panic]
            fn test_panic() {
                panic!("This should panic");
            }
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[cfg(test)]"));
    assert!(rust.contains("#[test]"));
    assert!(rust.contains("#[should_panic]"));
    assert!(rust.contains("assert_eq!"));
    assert!(rust.contains("panic!"));
}

#[test]
fn test_conditional_compilation() {
    let code = r#"
        #[cfg(unix)]
        fn platform_func() {
            println!("Unix platform");
        }

        #[cfg(windows)]
        fn platform_func() {
            println!("Windows platform");
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("#[cfg(unix)]"));
    assert!(rust.contains("#[cfg(windows)]"));
}

#[test]
fn test_macro_definition_and_usage() {
    let code = r#"
        macro_rules! double {
            ($x:expr) => { $x * 2 }
        }

        fn main() {
            let x = double!(5);
            println!("Result: {}", x);
        }
    "#;

    let parser = Parser::new();
    let ast = parser.parse(code).unwrap();

    let sexp = Generator::new().generate(&ast);
    assert!(sexp.to_string().contains("MacroDef"));
    assert!(sexp.to_string().contains("MacCall"));

    let rust = RustCodegen::new().generate(&ast);
    assert!(rust.contains("macro_rules! double"));
    assert!(rust.contains("double!(5)"));
}
```

### Coverage Goals

- **Overall**: Maintain 95%+ test coverage
- **New code**: 90%+ coverage minimum
- **Integration**: Real-world Rust patterns tested
- **Total tests**: 25-30 new tests minimum

---

## Success Criteria

### Phase 8.5 Complete When

1. ✅ **Macro Calls**:
   - All common macros parse correctly (println!, vec!, assert_eq!, format!)
   - Token streams preserved in AST
   - All three delimiters supported: `()`, `[]`, `{}`
   - Nested macros work
   - Round-trip successful

2. ✅ **Attributes**:
   - Derive macros parse and generate correctly
   - Cfg attributes work (test, target_os, feature)
   - Inner and outer attributes distinguished
   - Doc comments captured
   - Multiple attributes on single item work
   - Attributes on all item types (struct, enum, fn, impl, mod)

3. ✅ **Macro Definitions**:
   - `macro_rules!` definitions parse
   - Macro body preserved as token stream
   - Basic round-trip works
   - (Note: Macro expansion is out of scope)

4. ✅ **Testing**:
   - 25+ new tests pass
   - Real-world patterns tested
   - 95%+ coverage maintained

5. ✅ **Real-World Validation**:
   - Can parse typical Rust struct with derives
   - Can parse `#[cfg(test)]` test modules
   - Can parse conditional compilation attributes
   - Can parse macro definitions from common crates

---

## Common Pitfalls

### 1. Token Stream Handling

**Issue**: Token streams are complex, proper parsing is hard

**Solution**: For Phase 8.5, preserve tokens as strings. Don't try to parse token tree structure deeply. Let rustc handle the actual macro expansion.

### 2. Attribute Syntax Variations

**Issue**: Attributes have many forms:

- `#[simple]`
- `#[path::to::attr]`
- `#[attr(args)]`
- `#[attr = "value"]`
- `#[attr(key = "value")]`

**Solution**: Use `syn::Meta` which handles all these cases. Convert to MacArgs which is flexible enough.

### 3. Doc Comments vs Attributes

**Issue**: Doc comments (`///`, `//!`) are actually attributes in the AST

**Solution**: Detect and convert them to `AttrKind::DocComment` for better semantic representation.

### 4. Macro Expansion

**Issue**: Users might expect macros to be expanded

**Solution**: **DO NOT** attempt macro expansion. This is the compiler's job. Only preserve the macro call structure.

### 5. Missing proc_macro2 Dependency

**Issue**: `proc_macro2::TokenStream` might not be available

**Solution**: Ensure `Cargo.toml` has:

```toml
[dependencies]
syn = { version = "2.0", features = ["full", "parsing"] }
proc-macro2 = "1.0"  # For TokenStream handling
```

---

## File Checklist

### Files to Modify

- [ ] `crates/oxur-ast/src/ast/item.rs` - Verify MacroDef
- [ ] `crates/oxur-ast/src/ast/types.rs` - Verify Attribute structures
- [ ] `crates/oxur-ast/src/integration/from_syn.rs` - Add conversions
- [ ] `crates/oxur-ast/src/gen_rs/item.rs` - Generate attributes
- [ ] `crates/oxur-ast/src/gen_rs/expr.rs` - Generate macro calls
- [ ] `crates/oxur-ast/src/gen_sexp/item.rs` - Generate S-exp attributes
- [ ] `crates/oxur-ast/src/gen_sexp/expr.rs` - Generate S-exp macros
- [ ] `crates/oxur-ast/Cargo.toml` - Verify proc-macro2 dependency

### Files to Create

- [ ] `tests/phase8_5_macro_call_tests.rs` (6+ tests)
- [ ] `tests/phase8_5_attribute_tests.rs` (8+ tests)
- [ ] `tests/phase8_5_macro_def_tests.rs` (4+ tests)
- [ ] `tests/phase8_5_integration_tests.rs` (4+ tests)

---

## Implementation Order

Follow this order for smooth implementation:

1. **Day 1 Morning** (2-3 hours): Task 1 - Macro Calls
   - Verify MacCall structure
   - Implement conversion
   - Update generators
   - Write tests

2. **Day 1 Afternoon** (2-3 hours): Task 2 Part 1 - Basic Attributes
   - Verify Attribute structure
   - Implement conversion
   - Test with derive macros

3. **Day 2 Morning** (2 hours): Task 2 Part 2 - Complete Attributes
   - Update all item conversions
   - Add generators
   - Test cfg and doc comments

4. **Day 2 Afternoon** (2-3 hours): Task 3 - Macro Definitions
   - Add MacroDef
   - Implement conversion
   - Generate output
   - Write tests

5. **Final** (1 hour): Integration Testing & Validation
   - Run all tests
   - Test real-world code
   - Fix any issues
   - Verify coverage

---

## Dependencies

**Required**:

- Phase 8 Priority 1 complete (Traits, Impl, Modules, Use)
- Phase 8 Priority 2 complete (Expressions - covered by current session)

**Cargo.toml**:

```toml
[dependencies]
syn = { version = "2.0", features = ["full", "parsing"] }
proc-macro2 = "1.0"
quote = "1.0"  # Optional, for better token handling
```

---

## Notes

- **Macro expansion is OUT OF SCOPE** - We only preserve structure
- **Token stream parsing is simplified** - Store as strings for now
- **Focus on common patterns** - 90% of usage is derive, cfg, println!, vec!
- **Round-trip is critical** - Must be able to parse → generate → parse again

---

## Post-Phase 8.5

After this phase:

- **Phase 8 is COMPLETE** (all 3 priorities done)
- Can proceed to **Phases 9-13** for expression/pattern completeness
- Will handle ~95%+ of real-world Rust code structures

The combination of Phase 8.5 + Phases 9-13 will achieve near-complete Rust syntax coverage.
