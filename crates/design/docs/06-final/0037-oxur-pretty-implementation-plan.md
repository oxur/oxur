---
number: 37
title: "oxur-pretty Implementation Plan"
author: "Duncan McGreggor"
component: All
tags: [tooling]
created: 2026-01-03
updated: 2026-01-04
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-pretty Implementation Plan

## Overview

Create a new Rust crate `oxur-pretty` for pretty-printing S-expression formatted data structures, specifically targeting Oxur AST output but designed to be general-purpose for S-expression formatting.

**Primary Goal**: Transform unreadable, deeply-nested S-expressions like this:

```clojure
(Crate
  :attrs
  ()
  :items
  ((Item
      :attrs
      ()
      :id
      0
      :span
      (Span
        :lo
        0
        :hi
        0)
      :vis
      (Inherited)
      :ident
      (Ident
        :name
        "use"
        :span
        (Span
          :lo
          0
          :hi
          0))
      :kind ...
```

Into human-readable formatted output like this:

```clojure
(Crate
  :attrs ()
  :items ((Item
            :attrs ()
            :id 0
            :span (Span :lo 0 :hi 0)
            :vis (Inherited)
            :ident (Ident :name "use" :span (Span :lo 0 :hi 0))
            :kind ...)))
```

## Design Goals

1. **Zero dependencies on oxur-ast or oxur-lang** - Avoid cyclic dependencies
2. **General-purpose S-expression formatting** - Not Oxur-specific
3. **Configurable formatting rules** - Allow customization of output style
4. **High performance** - Suitable for large AST files
5. **Library-first design** - Can be consumed by oxur-ast, oxur-cli, or other tools

## Project Structure

```
crates/oxur-pretty/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── ast/                # AST-oriented formatting (Phase 1)
│   │   ├── mod.rs
│   │   ├── formatter.rs    # Core AST formatting logic
│   │   ├── config.rs       # Configuration types
│   │   └── rules.rs        # Formatting rules engine
│   ├── lisp/               # Lisp-style formatting (placeholder for future)
│   │   └── mod.rs
│   └── common/             # Common utilities (placeholder for future)
│       └── mod.rs
└── tests/
    ├── ast_formatting_tests.rs
    └── fixtures/
        ├── input/
        └── expected/
```

## Phase 1: AST Formatting (Initial Implementation)

### Core Components

#### 1. S-Expression Data Model

Create a lightweight AST representation for S-expressions that doesn't depend on oxur-ast:

```rust
// src/ast/mod.rs or src/common/sexp.rs

/// A generic S-expression node
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    /// An atom (symbol, keyword, string, number, etc.)
    Atom(String),
    /// A list of S-expressions
    List(Vec<SExpr>),
}

/// Token types for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    OpenParen,
    CloseParen,
    Keyword,      // :name, :attrs, etc.
    Symbol,       // Crate, Item, Span, etc.
    String,       // "use", etc.
    Number,       // 0, 1, etc.
    Nil,          // ()
}
```

#### 2. Configuration System

```rust
// src/ast/config.rs

/// Configuration for formatting S-expressions
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Maximum line width before breaking
    pub max_width: usize,

    /// Number of spaces per indentation level
    pub indent_size: usize,

    /// Whether to align keywords vertically
    pub align_keywords: bool,

    /// Whether to keep simple values on one line
    pub compact_simple_values: bool,

    /// Maximum number of items to keep on one line
    pub max_inline_items: usize,

    /// Whether to add extra spacing around nested structures
    pub spacious_nesting: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            max_width: 80,
            indent_size: 2,
            align_keywords: true,
            compact_simple_values: true,
            max_inline_items: 3,
            spacious_nesting: false,
        }
    }
}
```

#### 3. Formatting Rules Engine

```rust
// src/ast/rules.rs

/// Determines how to format a given S-expression
pub struct FormattingRules {
    config: FormatConfig,
}

impl FormattingRules {
    /// Decide if this expression should be formatted inline
    pub fn should_inline(&self, expr: &SExpr) -> bool {
        match expr {
            SExpr::Atom(_) => true,
            SExpr::List(items) => {
                // Keep small lists inline
                if items.len() <= self.config.max_inline_items {
                    items.iter().all(|item| matches!(item, SExpr::Atom(_)))
                } else {
                    false
                }
            }
        }
    }

    /// Determine if this is a keyword-value pair structure
    pub fn is_keyword_value_pair(&self, items: &[SExpr]) -> bool {
        items.len() >= 2 &&
        matches!(&items[0], SExpr::Atom(s) if s.starts_with(':'))
    }

    /// Check if this represents a simple struct-like form
    /// e.g., (Span :lo 0 :hi 0)
    pub fn is_simple_struct(&self, items: &[SExpr]) -> bool {
        if items.is_empty() {
            return false;
        }

        // First item is a symbol (constructor)
        let has_constructor = matches!(&items[0], SExpr::Atom(s) if !s.starts_with(':'));

        // Rest are keyword-value pairs with atomic values
        let has_simple_pairs = items[1..].chunks(2).all(|chunk| {
            chunk.len() == 2 &&
            matches!(&chunk[0], SExpr::Atom(s) if s.starts_with(':')) &&
            matches!(&chunk[1], SExpr::Atom(_))
        });

        has_constructor && has_simple_pairs
    }
}
```

#### 4. Core Formatter

```rust
// src/ast/formatter.rs

pub struct Formatter {
    config: FormatConfig,
    rules: FormattingRules,
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self {
        let rules = FormattingRules::new(&config);
        Self { config, rules }
    }

    /// Format an S-expression to a string
    pub fn format(&self, expr: &SExpr) -> String {
        let mut output = String::new();
        self.format_expr(expr, 0, &mut output);
        output
    }

    /// Format an S-expression at a given indentation level
    fn format_expr(&self, expr: &SExpr, indent: usize, output: &mut String) {
        match expr {
            SExpr::Atom(s) => {
                output.push_str(s);
            }
            SExpr::List(items) if items.is_empty() => {
                output.push_str("()");
            }
            SExpr::List(items) => {
                self.format_list(items, indent, output);
            }
        }
    }

    fn format_list(&self, items: &[SExpr], indent: usize, output: &mut String) {
        output.push('(');

        if self.rules.should_inline_list(items) {
            // Format everything on one line
            self.format_inline(items, output);
        } else if self.rules.is_simple_struct(items) {
            // Format as compact struct: (Type :key1 val1 :key2 val2)
            self.format_compact_struct(items, output);
        } else {
            // Format with proper indentation and line breaks
            self.format_multiline(items, indent, output);
        }

        output.push(')');
    }

    fn format_inline(&self, items: &[SExpr], output: &mut String) {
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            self.format_expr(item, 0, output);
        }
    }

    fn format_compact_struct(&self, items: &[SExpr], output: &mut String) {
        // (StructName :key1 val1 :key2 val2)
        self.format_expr(&items[0], 0, output);

        for chunk in items[1..].chunks(2) {
            output.push(' ');
            self.format_expr(&chunk[0], 0, output);
            if chunk.len() > 1 {
                output.push(' ');
                self.format_expr(&chunk[1], 0, output);
            }
        }
    }

    fn format_multiline(&self, items: &[SExpr], indent: usize, output: &mut String) {
        let new_indent = indent + self.config.indent_size;

        // First item stays on same line as opening paren
        if let Some(first) = items.first() {
            self.format_expr(first, new_indent, output);
        }

        // Rest get their own lines
        for item in items.iter().skip(1) {
            output.push('\n');
            output.push_str(&" ".repeat(new_indent));
            self.format_expr(item, new_indent, output);
        }
    }
}
```

### API Design

```rust
// src/lib.rs

pub mod ast;
pub mod lisp;   // Placeholder
pub mod common; // Placeholder

pub use ast::{FormatConfig, Formatter};

/// Quick format function for one-liners
pub fn format_sexp(input: &str) -> Result<String, FormattingError> {
    let config = FormatConfig::default();
    format_sexp_with_config(input, config)
}

/// Format with custom configuration
pub fn format_sexp_with_config(
    input: &str,
    config: FormatConfig
) -> Result<String, FormattingError> {
    let expr = parse_sexp(input)?;
    let formatter = Formatter::new(config);
    Ok(formatter.format(&expr))
}

/// Parse S-expression from string
fn parse_sexp(input: &str) -> Result<SExpr, FormattingError> {
    // Simple recursive descent parser
    // This should NOT depend on oxur-ast's parser
    todo!("Implement lightweight S-expression parser")
}
```

### Dependencies

```toml
[package]
name = "oxur-pretty"
version = "0.1.0"
edition = "2021"

[dependencies]
# Minimal dependencies - only what's absolutely needed

[dev-dependencies]
# For testing
```

## Phase 2: Clojure-Style Formatting Strategies

Drawing inspiration from clj-commons/pretty and standard EDN formatting:

### Key Formatting Strategies

1. **Hang Indentation**
   - When a list breaks across lines, subsequent items align with the first item
   - Example: `(defn foo [x y] ...)` → items after `defn` align

2. **Flow Indentation**
   - Standard indentation for lists that don't benefit from hang style
   - Each item gets its own line with consistent indent

3. **Compact Pairs**
   - Keyword-value pairs stay together when values are simple
   - Example: `:id 0` stays on one line, not broken across two

4. **Structure Recognition**
   - Simple structs like `(Span :lo 0 :hi 0)` stay compact
   - Complex nested structures break intelligently

### Formatting Examples

**Before (Current):**

```clojure
(Span
  :lo
  0
  :hi
  0)
```

**After (Pretty):**

```clojure
(Span :lo 0 :hi 0)
```

**Before (Current):**

```clojure
(Ident
  :name
  "use"
  :span
  (Span
    :lo
    0
    :hi
    0))
```

**After (Pretty):**

```clojure
(Ident
  :name "use"
  :span (Span :lo 0 :hi 0))
```

**Before (Current):**

```clojure
(Item
  :attrs
  ()
  :id
  0
  :span
  (Span
    :lo
    0
    :hi
    0)
  :vis
  (Inherited)
  :ident
  (Ident
    :name
    "use"
    :span
    (Span
      :lo
      0
      :hi
      0))
  :kind
  ...)
```

**After (Pretty):**

```clojure
(Item
  :attrs ()
  :id 0
  :span (Span :lo 0 :hi 0)
  :vis (Inherited)
  :ident (Ident
           :name "use"
           :span (Span :lo 0 :hi 0))
  :kind ...)
```

## Implementation Strategy

### Step 1: Minimal Parser (Week 1)

Build a lightweight S-expression parser that doesn't depend on oxur-ast:

```rust
// src/common/parser.rs

pub struct Parser {
    input: String,
    pos: usize,
}

impl Parser {
    pub fn parse(&mut self) -> Result<SExpr, ParseError> {
        self.skip_whitespace();

        if self.peek() == Some('(') {
            self.parse_list()
        } else {
            self.parse_atom()
        }
    }

    fn parse_list(&mut self) -> Result<SExpr, ParseError> {
        self.expect('(')?;
        let mut items = Vec::new();

        loop {
            self.skip_whitespace();

            if self.peek() == Some(')') {
                self.advance();
                break;
            }

            items.push(self.parse()?);
        }

        Ok(SExpr::List(items))
    }

    fn parse_atom(&mut self) -> Result<SExpr, ParseError> {
        let start = self.pos;

        // Handle strings
        if self.peek() == Some('"') {
            return self.parse_string();
        }

        // Handle keywords, symbols, numbers, nil, true, false
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            self.advance();
        }

        let text = self.input[start..self.pos].to_string();
        Ok(SExpr::Atom(text))
    }

    fn parse_string(&mut self) -> Result<SExpr, ParseError> {
        self.expect('"')?;
        let mut s = String::from("\"");

        loop {
            match self.peek() {
                Some('"') => {
                    s.push('"');
                    self.advance();
                    break;
                }
                Some('\\') => {
                    s.push('\\');
                    self.advance();
                    if let Some(ch) = self.peek() {
                        s.push(ch);
                        self.advance();
                    }
                }
                Some(ch) => {
                    s.push(ch);
                    self.advance();
                }
                None => return Err(ParseError::UnterminatedString),
            }
        }

        Ok(SExpr::Atom(s))
    }
}
```

### Step 2: Formatting Rules Engine (Week 1-2)

Implement intelligent rules for deciding how to format:

```rust
// src/ast/rules.rs

impl FormattingRules {
    /// Check if all items in a list are simple atoms
    pub fn all_atoms(&self, items: &[SExpr]) -> bool {
        items.iter().all(|item| matches!(item, SExpr::Atom(_)))
    }

    /// Estimate the width if formatted on one line
    pub fn estimate_width(&self, items: &[SExpr]) -> usize {
        let mut width = 2; // For parens

        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                width += 1; // Space
            }
            width += match item {
                SExpr::Atom(s) => s.len(),
                SExpr::List(inner) => self.estimate_width(inner),
            };
        }

        width
    }

    /// Determine if this is a simple keyword-value structure
    /// Examples: (Span :lo 0 :hi 0), (Inherited), etc.
    pub fn is_compact_struct(&self, items: &[SExpr]) -> bool {
        if items.is_empty() {
            return true; // Empty list ()
        }

        // First element is a type name (not a keyword)
        if let Some(SExpr::Atom(first)) = items.first() {
            if first.starts_with(':') {
                return false;
            }
        } else {
            return false;
        }

        // Check if fits width constraint
        let width = self.estimate_width(items);
        if width > self.config.max_width {
            return false;
        }

        // Check if all values are simple
        items[1..].chunks(2).all(|chunk| {
            chunk.len() == 2 &&
            matches!(&chunk[0], SExpr::Atom(s) if s.starts_with(':')) &&
            self.is_simple_value(&chunk[1])
        })
    }

    fn is_simple_value(&self, expr: &SExpr) -> bool {
        match expr {
            SExpr::Atom(_) => true,
            SExpr::List(items) => {
                // Recursively check nested simple structs
                self.is_compact_struct(items)
            }
        }
    }

    /// Check if this is a keyword-value pair pattern
    pub fn has_keyword_structure(&self, items: &[SExpr]) -> bool {
        if items.len() < 3 {
            return false;
        }

        // First is type name, rest are key-value pairs
        items[1..].chunks(2).all(|chunk| {
            chunk.len() == 2 &&
            matches!(&chunk[0], SExpr::Atom(s) if s.starts_with(':'))
        })
    }
}
```

### Step 3: Smart Formatter (Week 2)

```rust
// src/ast/formatter.rs

impl Formatter {
    fn format_list(&self, items: &[SExpr], indent: usize, output: &mut String) {
        output.push('(');

        if items.is_empty() {
            output.push(')');
            return;
        }

        // Decision tree for formatting
        if self.rules.is_compact_struct(items) {
            // Format: (Type :key1 val1 :key2 val2)
            self.format_inline(items, output);
        } else if self.rules.has_keyword_structure(items) {
            // Format with aligned keywords
            self.format_keyword_aligned(items, indent, output);
        } else if items.len() == 1 {
            // Single element: (Element)
            self.format_expr(&items[0], indent, output);
        } else {
            // Complex multiline
            self.format_multiline_smart(items, indent, output);
        }

        output.push(')');
    }

    fn format_keyword_aligned(&self, items: &[SExpr], indent: usize, output: &mut String) {
        let new_indent = indent + self.config.indent_size;

        // First item (type name) on same line as opening paren
        self.format_expr(&items[0], new_indent, output);

        // Process keyword-value pairs
        for chunk in items[1..].chunks(2) {
            output.push('\n');
            output.push_str(&" ".repeat(new_indent));

            // Keyword
            self.format_expr(&chunk[0], new_indent, output);

            if chunk.len() > 1 {
                output.push(' ');

                // Value - may be inline or need its own formatting
                if self.rules.is_simple_value(&chunk[1]) {
                    self.format_expr(&chunk[1], new_indent, output);
                } else {
                    // Complex value gets proper indentation
                    self.format_expr(&chunk[1], new_indent + self.config.indent_size, output);
                }
            }
        }
    }

    fn format_multiline_smart(&self, items: &[SExpr], indent: usize, output: &mut String) {
        let new_indent = indent + self.config.indent_size;

        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                output.push('\n');
                output.push_str(&" ".repeat(new_indent));
            }

            self.format_expr(item, new_indent, output);
        }
    }
}
```

### Step 4: Testing Infrastructure (Week 2-3)

```rust
// tests/ast_formatting_tests.rs

#[test]
fn test_simple_span() {
    let input = "(Span :lo 0 :hi 0)";
    let output = format_sexp(input).unwrap();
    assert_eq!(output, "(Span :lo 0 :hi 0)");
}

#[test]
fn test_expanded_span() {
    let input = r#"(Span
  :lo
  0
  :hi
  0)"#;
    let output = format_sexp(input).unwrap();
    assert_eq!(output, "(Span :lo 0 :hi 0)");
}

#[test]
fn test_nested_ident() {
    let input = r#"(Ident
  :name
  "use"
  :span
  (Span
    :lo
    0
    :hi
    0))"#;

    let expected = r#"(Ident
  :name "use"
  :span (Span :lo 0 :hi 0))"#;

    let output = format_sexp(input).unwrap();
    assert_eq!(output, expected);
}

#[test]
fn test_complex_item() {
    let input = include_str!("fixtures/input/complex_item.sexp");
    let expected = include_str!("fixtures/expected/complex_item.sexp");

    let output = format_sexp(input).unwrap();
    assert_eq!(output, expected);
}

#[test]
fn test_full_crate_formatting() {
    let input = include_str!("../../test-data/main_oxur.sexp");
    let output = format_sexp(input).unwrap();

    // Verify key structural improvements
    assert!(output.contains("(Span :lo 0 :hi 0)"));
    assert!(output.contains(":attrs ()"));
    assert!(!output.contains(":lo\n  0")); // No broken pairs
}
```

### Step 5: oxpretty CLI tool

You will need to go into plan mode in order to create a development plan that satisfies the following:

- Update the oxur-pretty crate to be a binary crate in addition to being a library create
- Follow the CLI patterns established by ./crates/oxur-ast/src/main.rs and ./crates/design/src/main.rs: with special attention to:
  - how they use supporting libraries (including oxur-cli)
  - how their Cargo.toml files are configured
  - how they are included in the top-level Makefile that builds them
- Additionally, you will want to conform VERY closely to Rust formatting tools as far as flags, defaults, and behaviours, with special attention to how intput and outputs are treated, how files are modified in-place, etc.

## Integration Points

### With oxur-ast CLI

```rust
// In oxur-ast CLI tool

use oxur_pretty::{format_sexp, FormatConfig};

fn to_ast_command(input: &str, output: &str, pretty: bool) -> Result<()> {
    // ... existing code to convert to S-expression ...

    let sexp_str = if pretty {
        let config = FormatConfig::default();
        oxur_pretty::format_sexp_with_config(&raw_sexp, config)?
    } else {
        raw_sexp
    };

    fs::write(output, sexp_str)?;
    Ok(())
}
```

### CLI Flag

Add a `--pretty` or `--format` flag:

```
oxur-ast to-ast input.rs output.sexp --pretty
```

## Configuration Options

Allow users to customize formatting:

```rust
// Example usage
let config = FormatConfig {
    max_width: 100,           // Longer lines allowed
    indent_size: 4,           // Wider indentation
    align_keywords: true,     // Align :keywords
    compact_simple_values: true,  // (Span :lo 0 :hi 0) style
    max_inline_items: 5,      // More items inline
    spacious_nesting: false,  // Dense vs spacious
};

let formatted = format_sexp_with_config(input, config)?;
```

## Performance Considerations

1. **Streaming Output**: For very large files, consider streaming rather than building entire string in memory
2. **Lazy Evaluation**: Don't parse entire tree if only formatting a subsection
3. **Caching**: Cache width calculations for repeated structures
4. **Zero-Copy**: Use string slices where possible

## Future Enhancements (Post Phase 1)

1. **Color Output**: ANSI colors for terminal output
2. **Syntax Highlighting**: Integration with tree-sitter
3. **Diff Mode**: Show before/after formatting
4. **Language Server**: Real-time formatting in editors
5. **Lisp Module**: Support for actual Oxur Lisp syntax
6. **Web Assembly**: Browser-based formatter

## Dependencies

```toml
[package]
name = "oxur-pretty"
version = "0.1.0"
edition = "2021"
authors = ["Oxur Contributors"]
description = "Pretty-printer for S-expression formatted data"
repository = "https://github.com/oxur/oxur"
license = "Apache-2.0"

[dependencies]
# Minimal - maybe just thiserror for error handling
thiserror = "2.0"

[dev-dependencies]
# For property-based testing
proptest = "1.5"

[[example]]
name = "format_file"
path = "examples/format_file.rs"
```

## Deliverables

### Phase 1 (Weeks 1-3)

- [ ] Complete S-expression parser
- [ ] Basic formatting engine
- [ ] Rule system for structure detection
- [ ] A new oxpretty binary that can operate on Oxur .sexp files
- [ ] Comprehensive test suite
- [ ] Documentation and examples
- [ ] Integration with oxur-ast CLI

### Success Criteria

- Formats the provided `main_oxur.sexp` into readable output
- All tests pass
- No dependencies on oxur-ast or oxur-lang
- Can be integrated into oxur-ast CLI tool
- Performance: formats 10,000 line files in < 100ms

## Example CLI Tool (Standalone)

```rust
// examples/format_file.rs

use oxur_pretty::{format_sexp, FormatConfig};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: format_file <input.sexp> [output.sexp]");
        std::process::exit(1);
    }

    let input = fs::read_to_string(&args[1])?;
    let formatted = format_sexp(&input)?;

    if args.len() > 2 {
        fs::write(&args[2], formatted)?;
    } else {
        println!("{}", formatted);
    }

    Ok(())
}
```

## Questions for Claude Code

1. Should we support configuration files (e.g., `.oxur-pretty.toml`)?
2. Should the formatter be idempotent (formatting twice gives same result)?
3. Should we preserve comments if any exist in the S-expressions?
4. What's the maximum file size we should support efficiently?
5. Should we provide a `--check` mode that exits with error if formatting needed?

## References

- Clojure/EDN formatting: <https://github.com/clj-commons/pretty>
- S-expression standards: Wikipedia S-expression article
- Rust formatting tools: rustfmt for inspiration on configuration
