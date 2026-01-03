---
number: 33
title: "oxur-ast Phase 11: Literal Completeness and Pattern Refinement"
author: "Duncan McGreggor"
created: 2026-01-03
updated: 2026-01-03
state: Under Review
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-ast Phase 11: Literal Completeness and Pattern Refinement

**Status**: Planned
**Estimated Effort**: 1-2 days
**Expected Coverage Gain**: +3-5% (from ~90% to ~93-95%)
**Complexity**: Low-Medium - Straightforward additions
**Prerequisite**: Phases 9 and 10 must be completed

## Overview

This phase completes literal type support and refines pattern matching capabilities. While not as impactful as closures or match expressions, these features are essential for handling all basic Rust data types and improving pattern matching completeness.

**Impact**: Enables parsing of virtually all basic Rust programs including those using boolean logic, floating-point arithmetic, character literals, and byte strings.

## Goals

1. Implement all remaining literal types (Bool, Float, Char, Byte, ByteStr, CStr)
2. Add missing pattern types (Box, Macro, Const)
3. Improve literal pattern matching
4. Handle shorthand struct patterns and fields
5. Achieve 93-95% coverage of common Rust syntax

## Current State

### Literals Currently Supported (2/9)

From `crates/oxur-ast/src/integration/from_syn.rs`:

- ✅ `Str` - String literals
- ✅ `Int` - Integer literals

### Missing Literal Types (7/9)

- ❌ `Bool` - Boolean literals (`true`, `false`) - **VERY COMMON**
- ❌ `Float` - Floating-point literals (`3.14`, `1.0e10`) - **COMMON**
- ❌ `Char` - Character literals (`'a'`, `'\n'`) - **COMMON**
- ❌ `Byte` - Byte literals (`b'A'`, `b'\n'`)
- ❌ `ByteStr` - Byte string literals (`b"hello"`)
- ❌ `CStr` - C string literals (`c"hello"`)
- ❌ `Verbatim` - Other literal forms (rare)

### Pattern Types from Phase 9

Most were added in Phase 9, but we need to refine:

- Box patterns
- Macro patterns
- Const patterns
- Shorthand detection for struct patterns/fields

## Detailed Tasks

### Task 1: Complete Literal Support

**File**: `crates/oxur-ast/src/integration/from_syn.rs`
**Function**: `convert_expr_lit()` or similar

#### 1.1: Understand Current Literal Handling

Check how literals are currently converted. There should be a function like:

```rust
fn convert_expr_lit(&mut self, lit: &syn::Lit) -> Result<Expr> {
    let kind = match lit {
        syn::Lit::Str(lit_str) => {
            LitKind::Str(lit_str.value())
        }
        syn::Lit::Int(lit_int) => {
            let value = lit_int.base10_parse::<i128>()?;
            LitKind::Int(value)
        }
        // Add missing cases here
    }

    Ok(Expr {
        id: self.next_id(),
        kind: ExprKind::Lit(Lit { kind, span: Span::DUMMY }),
        span: Span::DUMMY,
        attrs: vec![],
        tokens: None,
    })
}
```

#### 1.2: Add Boolean Literals (CRITICAL)

```rust
syn::Lit::Bool(lit_bool) => {
    LitKind::Bool(lit_bool.value)
}
```

**Test Cases**:

```rust
let flag = true;
let disabled = false;

if condition {
    true
} else {
    false
}

// Common in match
match value {
    true => "yes",
    false => "no",
}
```

#### 1.3: Add Float Literals (IMPORTANT)

```rust
syn::Lit::Float(lit_float) => {
    // Parse the float value
    let value = lit_float.base10_parse::<f64>()?;
    LitKind::Float(value)
}
```

**Note**: Check if `LitKind::Float` exists in AST. If not, may need to add it or use a string representation.

**Test Cases**:

```rust
let pi = 3.14159;
let euler = 2.71828;
let scientific = 6.022e23;
let small = 1.23e-10;

// Common in calculations
let area = radius * radius * 3.14159;
```

#### 1.4: Add Character Literals

```rust
syn::Lit::Char(lit_char) => {
    LitKind::Char(lit_char.value())
}
```

**Test Cases**:

```rust
let letter = 'a';
let newline = '\n';
let unicode = '\u{1F600}'; // 😀

match ch {
    'a'..='z' => "lowercase",
    'A'..='Z' => "uppercase",
    _ => "other",
}
```

#### 1.5: Add Byte Literals

```rust
syn::Lit::Byte(lit_byte) => {
    LitKind::Byte(lit_byte.value())
}
```

**Test Cases**:

```rust
let ascii_a = b'A';
let newline = b'\n';

// Common in byte stream processing
match byte {
    b'0'..=b'9' => "digit",
    b'a'..=b'z' => "lowercase",
    _ => "other",
}
```

#### 1.6: Add Byte String Literals

```rust
syn::Lit::ByteStr(lit_bytestr) => {
    LitKind::ByteStr(lit_bytestr.value())
}
```

**Test Cases**:

```rust
let bytes = b"hello world";
let protocol_magic = b"\x00\x01\x02\x03";

// Common in network/file I/O
if buffer.starts_with(b"HTTP") {
    // ...
}
```

#### 1.7: Add C String Literals (Rust 1.77+)

```rust
syn::Lit::CStr(lit_cstr) => {
    LitKind::CStr(lit_cstr.value())
}
```

**Test Cases**:

```rust
let c_string = c"hello\0";

// Common in FFI
extern "C" {
    fn print(s: *const c_char);
}

print(c"Hello from Rust".as_ptr());
```

#### 1.8: Handle Verbatim Literals

```rust
syn::Lit::Verbatim(tokens) => {
    // For now, store as string representation
    LitKind::Verbatim(tokens.to_string())
}
```

#### 1.9: Update LitKind Enum if Needed

**File**: `crates/oxur-ast/src/ast/expr.rs`

Check if all these variants exist in `LitKind`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum LitKind {
    Str(String),
    Int(i128),
    Bool(bool),         // Add if missing
    Float(f64),         // Add if missing (or use String)
    Char(char),         // Add if missing
    Byte(u8),           // Add if missing
    ByteStr(Vec<u8>),   // Add if missing
    CStr(Vec<u8>),      // Add if missing
    Verbatim(String),   // Add if missing
}
```

### Task 2: Refine Pattern Support

#### 2.1: Add Box Pattern (Low Priority)

**Note**: Box patterns are unstable and rarely used. Can be skipped initially.

```rust
syn::Pat::Box(pat_box) => {
    let inner = Box::new(self.convert_pat(&pat_box.pat)?);

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Box(inner),
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case** (requires nightly):

```rust
#![feature(box_patterns)]

match value {
    box x => x,
}
```

#### 2.2: Add Macro Pattern

```rust
syn::Pat::Macro(pat_macro) => {
    let mac = self.convert_macro(&pat_macro.mac)?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::MacCall(mac),
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case**:

```rust
macro_rules! pat {
    ($e:expr) => { $e };
}

match value {
    pat!(Some(x)) => x,
    _ => 0,
}
```

#### 2.3: Add Const Pattern (Const blocks in patterns)

```rust
syn::Pat::Const(pat_const) => {
    // For now, treat as a literal expression
    let expr = self.convert_expr(&pat_const.expr)?;

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Lit(Box::new(expr)),
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Case** (Rust 1.79+):

```rust
const PATTERN: i32 = 42;

match value {
    const { PATTERN } => "found it",
    _ => "not found",
}
```

### Task 3: Implement Shorthand Detection

Shorthand syntax is when field names match binding names:

- `Point { x }` is shorthand for `Point { x: x }`
- `Point { x, y }` is shorthand for `Point { x: x, y: y }`

#### 3.1: Detect Shorthand in Struct Expressions

**File**: `crates/oxur-ast/src/integration/from_syn.rs`

Update the struct literal conversion from Phase 9:

```rust
syn::Expr::Struct(expr_struct) => {
    let path = self.convert_path(&expr_struct.path)?;

    let fields = expr_struct.fields
        .iter()
        .map(|field| {
            let ident = match &field.member {
                syn::Member::Named(name) => self.convert_ident(name),
                syn::Member::Unnamed(idx) => {
                    Ident::new(&idx.index.to_string(), Span::DUMMY)
                }
            };

            let expr = self.convert_expr(&field.expr)?;

            // Detect shorthand: if field expr is just a Path with same name
            let is_shorthand = if let ExprKind::Path(None, path) = &expr.kind {
                if path.segments.len() == 1 {
                    path.segments[0].ident.name == ident.name
                } else {
                    false
                }
            } else {
                false
            };

            Ok(ExprField {
                attrs: vec![],
                ident,
                expr,
                is_shorthand,
                span: Span::DUMMY,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    ExprKind::Struct { path, fields }
}
```

**Test Cases**:

```rust
// Shorthand
let x = 10;
let y = 20;
let point = Point { x, y };  // is_shorthand = true for both

// Mixed
let point = Point { x, y: 30 };  // x is shorthand, y is not

// Explicit
let point = Point { x: x, y: y };  // Neither is shorthand
```

#### 3.2: Detect Shorthand in Struct Patterns

Similar logic for patterns:

```rust
syn::Pat::Struct(pat_struct) => {
    let path = self.convert_path(&pat_struct.path)?;

    let fields = pat_struct.fields
        .iter()
        .map(|field| {
            let ident = match &field.member {
                syn::Member::Named(name) => self.convert_ident(name),
                syn::Member::Unnamed(idx) => {
                    Ident::new(&idx.index.to_string(), Span::DUMMY)
                }
            };

            let pat = self.convert_pat(&field.pat)?;

            // Detect shorthand: if pat is just an ident with same name
            let is_shorthand = if let PatKind::Ident { ident: pat_ident, .. } = &pat.kind {
                pat_ident.name == ident.name
            } else {
                false
            };

            Ok(PatField {
                attrs: vec![],
                ident,
                pat,
                is_shorthand,
                span: Span::DUMMY,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Handle rest pattern (..)
    let rest = pat_struct.rest.is_some();

    Ok(Pat {
        id: self.next_id(),
        kind: PatKind::Struct { path, fields, rest },
        span: Span::DUMMY,
        tokens: None,
    })
}
```

**Test Cases**:

```rust
// Shorthand
match point {
    Point { x, y } => (x, y),  // Both shorthand
}

// Mixed
match point {
    Point { x, y: new_y } => (x, new_y),  // x shorthand, y not
}

// With rest
match point {
    Point { x, .. } => x,  // rest = true
}
```

### Task 4: Update Generators

Update code generators to handle new literal types.

#### 4.1: Update Rust Generator for Literals

**File**: `crates/oxur-ast/src/gen_rs/expr.rs`

```rust
fn generate_lit(&mut self, lit: &Lit) -> String {
    match &lit.kind {
        LitKind::Str(s) => format!("\"{}\"", escape_string(s)),
        LitKind::Int(i) => i.to_string(),
        LitKind::Bool(b) => b.to_string(),
        LitKind::Float(f) => {
            // Ensure we always have a decimal point
            let s = f.to_string();
            if s.contains('.') || s.contains('e') {
                s
            } else {
                format!("{}.0", s)
            }
        }
        LitKind::Char(c) => format!("'{}'", escape_char(*c)),
        LitKind::Byte(b) => format!("b'{}'", escape_byte(*b)),
        LitKind::ByteStr(bytes) => {
            format!("b\"{}\"", escape_bytes(bytes))
        }
        LitKind::CStr(bytes) => {
            format!("c\"{}\"", escape_bytes(bytes))
        }
        LitKind::Verbatim(s) => s.clone(),
    }
}

fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn escape_char(c: char) -> String {
    match c {
        '\'' => "\\'".to_string(),
        '\\' => "\\\\".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        c => c.to_string(),
    }
}

fn escape_byte(b: u8) -> String {
    match b {
        b'\'' => "\\'".to_string(),
        b'\\' => "\\\\".to_string(),
        b'\n' => "\\n".to_string(),
        b'\r' => "\\r".to_string(),
        b'\t' => "\\t".to_string(),
        b if b.is_ascii_graphic() || b == b' ' => (b as char).to_string(),
        b => format!("\\x{:02x}", b),
    }
}

fn escape_bytes(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|&b| match b {
            b'"' => "\\\"".to_string(),
            b'\\' => "\\\\".to_string(),
            b'\n' => "\\n".to_string(),
            b'\r' => "\\r".to_string(),
            b'\t' => "\\t".to_string(),
            b if b.is_ascii_graphic() || b == b' ' => (b as char).to_string(),
            b => format!("\\x{:02x}", b),
        })
        .collect()
}
```

#### 4.2: Update S-Expression Generator

**File**: `crates/oxur-ast/src/gen_sexp/expr.rs`

Add cases for new literal types in the S-expression generator:

```rust
fn generate_lit(&self, lit: &Lit) -> Result<SExp> {
    Ok(match &lit.kind {
        LitKind::Str(s) => str_lit(s),
        LitKind::Int(i) => int_lit(*i),
        LitKind::Bool(b) => bool_lit(*b),
        LitKind::Float(f) => float_lit(*f),
        LitKind::Char(c) => char_lit(*c),
        LitKind::Byte(b) => byte_lit(*b),
        LitKind::ByteStr(bytes) => bytestr_lit(bytes),
        LitKind::CStr(bytes) => cstr_lit(bytes),
        LitKind::Verbatim(s) => verbatim_lit(s),
    })
}
```

### Task 5: Handle Shorthand in Generators

#### 5.1: Update Struct Expression Generator

```rust
fn generate_struct_expr(&mut self, path: &Path, fields: &[ExprField]) -> String {
    let path_str = self.generate_path(path);

    let fields_str = fields
        .iter()
        .map(|field| {
            if field.is_shorthand {
                // Shorthand: just the field name
                field.ident.name.clone()
            } else {
                // Full: name: expr
                format!("{}: {}", field.ident.name, self.generate_expr(&field.expr))
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("{} {{ {} }}", path_str, fields_str)
}
```

#### 5.2: Update Struct Pattern Generator

```rust
fn generate_struct_pat(&mut self, path: &Path, fields: &[PatField], rest: bool) -> String {
    let path_str = self.generate_path(path);

    let mut field_strs: Vec<String> = fields
        .iter()
        .map(|field| {
            if field.is_shorthand {
                field.ident.name.clone()
            } else {
                format!("{}: {}", field.ident.name, self.generate_pat(&field.pat))
            }
        })
        .collect();

    if rest {
        field_strs.push("..".to_string());
    }

    format!("{} {{ {} }}", path_str, field_strs.join(", "))
}
```

## Testing Strategy

### Test File: Literal Tests

**File**: `crates/oxur-ast/tests/phase11_literals_tests.rs`

```rust
use oxur_ast::*;

#[test]
fn test_bool_literal() {
    let code = r#"
        fn main() {
            let t = true;
            let f = false;
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
    // Verify bool literals
}

#[test]
fn test_float_literal() {
    let code = r#"
        fn main() {
            let pi = 3.14159;
            let e = 2.71828;
            let sci = 6.022e23;
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_char_literal() {
    let code = r#"
        fn main() {
            let letter = 'a';
            let newline = '\n';
            let unicode = '\u{1F600}';
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_byte_literal() {
    let code = r#"
        fn main() {
            let byte = b'A';
            let bytes = b"hello";
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_all_literals_in_match() {
    let code = r#"
        fn classify(value: &str) -> &str {
            match value {
                "true" => "bool",
                "3.14" => "float",
                "a" => "char",
                _ => "other",
            }
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

### Test File: Shorthand Tests

**File**: `crates/oxur-ast/tests/phase11_shorthand_tests.rs`

```rust
#[test]
fn test_struct_shorthand_expression() {
    let code = r#"
        struct Point { x: i32, y: i32 }

        fn main() {
            let x = 10;
            let y = 20;
            let p = Point { x, y };
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
    // Verify shorthand detection
}

#[test]
fn test_struct_mixed_shorthand() {
    let code = r#"
        struct Point { x: i32, y: i32 }

        fn main() {
            let x = 10;
            let p = Point { x, y: 20 };
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_pattern_shorthand() {
    let code = r#"
        struct Point { x: i32, y: i32 }

        fn main() {
            let Point { x, y } = point;
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}

#[test]
fn test_pattern_with_rest() {
    let code = r#"
        struct Point { x: i32, y: i32, z: i32 }

        fn main() {
            let Point { x, .. } = point;
        }
    "#;

    let ast = parse_rust_source(code).unwrap();
}
```

### Round-Trip Tests

```rust
#[test]
fn test_literal_round_trip() {
    let cases = vec![
        "true",
        "false",
        "3.14",
        "'a'",
        "b'A'",
        r#"b"hello""#,
    ];

    for case in cases {
        let ast = parse_expression(case).unwrap();
        let generated = generate_rust(&ast);
        let ast2 = parse_expression(&generated).unwrap();
        assert_eq!(ast, ast2, "Failed for: {}", case);
    }
}

#[test]
fn test_shorthand_round_trip() {
    let code = r#"Point { x, y: 20 }"#;
    let ast = parse_expression(code).unwrap();
    let generated = generate_rust(&ast);

    // Should preserve shorthand
    assert!(generated.contains("x,") || generated.contains("x }"));
    assert!(generated.contains("y: 20"));
}
```

## Success Criteria

- ✅ All 7 missing literal types parse correctly
- ✅ Boolean and float literals work in common contexts
- ✅ Shorthand struct syntax detected and preserved
- ✅ Struct patterns with rest (`..`) work
- ✅ Round-trip tests preserve shorthand syntax
- ✅ Can parse 93-95% of typical Rust code
- ✅ All Phase 11 tests pass (minimum 25 new tests)

## Files to Modify

1. **Primary**:
   - `crates/oxur-ast/src/integration/from_syn.rs` - Literal and shorthand logic

2. **AST Definitions** (may need updates):
   - `crates/oxur-ast/src/ast/expr.rs` - Add missing LitKind variants

3. **Generators**:
   - `crates/oxur-ast/src/gen_rs/expr.rs` - Literal generation + shorthand
   - `crates/oxur-ast/src/gen_sexp/expr.rs` - Literal generation

4. **Testing**:
   - `crates/oxur-ast/tests/phase11_literals_tests.rs` - New
   - `crates/oxur-ast/tests/phase11_shorthand_tests.rs` - New
   - `crates/oxur-ast/tests/phase11_round_trip_tests.rs` - New

## Common Pitfalls

1. **Float Formatting**: Ensure floats always have decimal point (3.0 not 3)
2. **Escape Sequences**: Handle all escape sequences correctly in strings/chars
3. **Unicode**: Ensure proper handling of unicode in char/string literals
4. **Byte vs Char**: Don't confuse `b'A'` with `'A'`
5. **Shorthand Edge Cases**: Handle single-field structs, empty structs correctly

## Dependencies

- ✅ Phase 9 (patterns) must be complete
- ✅ Phase 10 (match) makes literals more useful
- ⚠️ Need to verify LitKind enum is complete

## Next Phase

After Phase 11, proceed to **Phase 12: Advanced Type System** which adds:

- Impl Trait types (`impl Iterator`)
- Trait Object types (`dyn Display`)
- Bare function types (`fn(i32) -> bool`)
- Complete type system coverage

---

**Estimated Timeline**:

- Day 1: Literal types + testing (6-8 hours)
- Day 2: Shorthand detection + generator updates (4-6 hours)

**Total**: 1-2 days for experienced developer

## Verification Checklist

Before marking Phase 11 complete:

- [ ] Boolean literals: true/false in all contexts
- [ ] Float literals: decimal, scientific notation
- [ ] Char literals: ASCII, escape sequences, unicode
- [ ] Byte literals: b'X' syntax
- [ ] Byte strings: b"..." syntax
- [ ] C strings: c"..." syntax (if Rust 1.77+)
- [ ] Shorthand struct expressions detected
- [ ] Shorthand struct patterns detected
- [ ] Rest patterns in structs work
- [ ] All round-trip tests pass
- [ ] Generators preserve shorthand
