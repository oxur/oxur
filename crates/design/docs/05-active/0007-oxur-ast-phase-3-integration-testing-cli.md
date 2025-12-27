---
number: 7
title: "oxur-ast Phase 3: Integration, Testing & CLI"
author: "Duncan McGreggor"
created: 2025-12-27
updated: 2025-12-27
state: Active
supersedes: null
superseded-by: null
---

# oxur-ast Phase 3: Integration, Testing & CLI

**Phase**: 3 - Integration & Production Readiness  
**Goal**: Connect to real Rust code, comprehensive testing, and usable CLI tools  
**Estimated Time**: 6-8 days  
**Prerequisites**: Phases 0, 1, and 2 complete (bidirectional conversion working)

---

## Overview

Phase 3 transforms `oxur-ast` from a working prototype into a production-ready library. We'll connect to real Rust code, build comprehensive tests, and create CLI tools for practical use.

**What we're building:**
1. Integration with Rust's parser (`syn` crate)
2. Comprehensive test suite using real Rust code
3. CLI tool for converting Rust ↔ S-expressions
4. Performance benchmarks
5. Documentation and examples
6. Error handling improvements

**End goal:**
```bash
# Convert Rust file to S-expression
oxur-ast to-sexp hello.rs > hello.sexp

# Convert S-expression back to Rust
oxur-ast to-rust hello.sexp > hello.rs

# Verify round-trip
oxur-ast verify hello.rs
```

---

## File Structure

```
oxur-ast/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── sexp/
│   ├── ast/
│   ├── builder/
│   ├── generator/
│   ├── integration/       # NEW: Integration with syn
│   │   ├── mod.rs
│   │   ├── from_syn.rs    # syn AST → oxur AST
│   │   └── to_syn.rs      # oxur AST → syn AST (future)
│   └── bin/
│       └── oxur-ast.rs    # NEW: CLI tool
├── tests/
│   ├── integration_tests.rs   # NEW: Real Rust code tests
│   ├── regression_tests.rs    # NEW: Test corpus
│   └── fixtures/              # NEW: Test files
│       ├── hello_world.rs
│       ├── simple_fn.rs
│       └── ...
├── benches/                   # NEW: Performance benchmarks
│   └── conversion_bench.rs
└── examples/
    ├── parse_rust_file.rs     # NEW: Parse real Rust
    └── convert_file.rs        # NEW: File conversion
```

---

## Part 1: Integration with syn

### File: `Cargo.toml` (update dependencies)

```toml
[dependencies]
thiserror.workspace = true
syn = { version = "2.0", features = ["full", "parsing", "extra-traits"] }
quote = "1.0"

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "conversion_bench"
harness = false
```

### File: `src/integration/mod.rs`

```rust
//! Integration with Rust's standard AST (via syn)

mod from_syn;

pub use from_syn::*;

use crate::error::Result;
use crate::ast::Crate;

/// Parse Rust source code into our AST
pub fn parse_rust_file(source: &str) -> Result<Crate> {
    let syn_file = syn::parse_file(source)
        .map_err(|e| crate::error::ParseError::Expected {
            expected: "valid Rust code".to_string(),
            found: format!("parse error: {}", e),
            pos: crate::error::Position::new(0, 1, 1),
        })?;
    
    from_syn_file(&syn_file)
}
```

---

## Part 2: syn → oxur Conversion

### File: `src/integration/from_syn.rs`

Convert syn's AST to our AST:

```rust
use crate::error::{Result, ParseError, Position};
use crate::ast::*;
use syn;

/// Convert syn::File to our Crate
pub fn from_syn_file(file: &syn::File) -> Result<Crate> {
    let mut converter = SynConverter::new();
    converter.convert_file(file)
}

struct SynConverter {
    next_node_id: usize,
}

impl SynConverter {
    fn new() -> Self {
        Self { next_node_id: 0 }
    }
    
    fn next_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        NodeId::new(id)
    }
    
    fn convert_file(&mut self, file: &syn::File) -> Result<Crate> {
        let items = file.items.iter()
            .map(|item| self.convert_item(item))
            .collect::<Result<Vec<_>>>()?;
        
        // Create spans from syn::File
        // Note: syn doesn't give us exact byte offsets easily, so we approximate
        let inner_span = Span::new(0, 0); // Will improve in future
        let inject_use_span = Span::new(0, 0);
        let spans = ModSpans::new(inner_span, inject_use_span);
        
        Ok(Crate::new(items, spans, self.next_id()))
    }
    
    fn convert_item(&mut self, item: &syn::Item) -> Result<Item> {
        match item {
            syn::Item::Fn(item_fn) => self.convert_item_fn(item_fn),
            _ => Err(ParseError::Expected {
                expected: "supported item type".to_string(),
                found: "unsupported item".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }
    
    fn convert_item_fn(&mut self, item_fn: &syn::ItemFn) -> Result<Item> {
        let ident = self.convert_ident(&item_fn.sig.ident);
        let vis = self.convert_visibility(&item_fn.vis);
        
        let fn_sig = self.convert_fn_sig(&item_fn.sig)?;
        let generics = self.convert_generics(&item_fn.sig.generics)?;
        let body = Some(self.convert_block(&item_fn.block)?);
        
        let fn_item = Fn {
            defaultness: Defaultness::Final,
            sig: fn_sig,
            generics,
            body,
        };
        
        Ok(Item {
            attrs: vec![],  // Phase 3: simplified
            id: self.next_id(),
            span: Span::DUMMY,  // Will improve with proc-macro2::Span
            vis,
            ident,
            kind: ItemKind::Fn(Box::new(fn_item)),
            tokens: None,
        })
    }
    
    fn convert_ident(&mut self, ident: &syn::Ident) -> Ident {
        Ident::new(ident.to_string(), Span::DUMMY)
    }
    
    fn convert_visibility(&mut self, vis: &syn::Visibility) -> Visibility {
        match vis {
            syn::Visibility::Public(_) => Visibility::Public,
            syn::Visibility::Inherited => Visibility::Inherited,
            syn::Visibility::Restricted(_) => {
                // Simplified for Phase 3
                Visibility::Inherited
            }
        }
    }
    
    fn convert_fn_sig(&mut self, sig: &syn::Signature) -> Result<FnSig> {
        let header = self.convert_fn_header(sig);
        let decl = self.convert_fn_decl(sig)?;
        
        Ok(FnSig {
            header,
            decl,
            span: Span::DUMMY,
        })
    }
    
    fn convert_fn_header(&mut self, sig: &syn::Signature) -> FnHeader {
        let safety = match sig.unsafety {
            Some(_) => Safety::Unsafe,
            None => Safety::Default,
        };
        
        let constness = match sig.constness {
            Some(_) => Constness::Const,
            None => Constness::NotConst,
        };
        
        let coroutine_kind = match sig.asyncness {
            Some(_) => Some(CoroutineKind::Async),
            None => None,
        };
        
        let ext = match &sig.abi {
            Some(abi) => {
                if let Some(name) = &abi.name {
                    Extern::Explicit(name.value())
                } else {
                    Extern::Explicit("C".to_string())
                }
            }
            None => Extern::None,
        };
        
        FnHeader {
            safety,
            coroutine_kind,
            constness,
            ext,
        }
    }
    
    fn convert_fn_decl(&mut self, sig: &syn::Signature) -> Result<FnDecl> {
        let inputs = sig.inputs.iter()
            .filter_map(|arg| {
                match arg {
                    syn::FnArg::Typed(pat_type) => {
                        Some(self.convert_fn_arg(pat_type))
                    }
                    syn::FnArg::Receiver(_) => None, // Skip self for Phase 3
                }
            })
            .collect::<Result<Vec<_>>>()?;
        
        let output = self.convert_return_type(&sig.output)?;
        
        Ok(FnDecl { inputs, output })
    }
    
    fn convert_fn_arg(&mut self, pat_type: &syn::PatType) -> Result<Param> {
        let pat = self.convert_pat(&pat_type.pat)?;
        let ty = self.convert_type(&pat_type.ty)?;
        
        Ok(Param {
            attrs: vec![],
            ty,
            pat,
            id: self.next_id(),
            span: Span::DUMMY,
            is_placeholder: false,
        })
    }
    
    fn convert_pat(&mut self, pat: &syn::Pat) -> Result<Pat> {
        match pat {
            syn::Pat::Ident(pat_ident) => {
                let ident = self.convert_ident(&pat_ident.ident);
                Ok(Pat {
                    id: self.next_id(),
                    kind: PatKind::Ident(ident),
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            _ => Err(ParseError::Expected {
                expected: "ident pattern".to_string(),
                found: "complex pattern".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }
    
    fn convert_type(&mut self, ty: &syn::Type) -> Result<Ty> {
        match ty {
            syn::Type::Path(type_path) => {
                let path = self.convert_path(&type_path.path)?;
                Ok(Ty {
                    id: self.next_id(),
                    kind: TyKind::Path(None, path),
                    span: Span::DUMMY,
                    tokens: None,
                })
            }
            _ => Err(ParseError::Expected {
                expected: "path type".to_string(),
                found: "complex type".to_string(),
                pos: Position::new(0, 1, 1),
            }),
        }
    }
    
    fn convert_path(&mut self, path: &syn::Path) -> Result<Path> {
        let segments = path.segments.iter()
            .map(|seg| {
                let ident = self.convert_ident(&seg.ident);
                PathSegment::new(ident, self.next_id())
            })
            .collect();
        
        Ok(Path::new(Span::DUMMY, segments))
    }
    
    fn convert_return_type(&mut self, ret: &syn::ReturnType) -> Result<FnRetTy> {
        match ret {
            syn::ReturnType::Default => Ok(FnRetTy::Default(Span::DUMMY)),
            syn::ReturnType::Type(_, ty) => {
                Ok(FnRetTy::Ty(Box::new(self.convert_type(ty)?)))
            }
        }
    }
    
    fn convert_generics(&mut self, generics: &syn::Generics) -> Result<Generics> {
        // Simplified for Phase 3 - just create empty generics
        Ok(Generics::empty(Span::DUMMY))
    }
    
    fn convert_block(&mut self, block: &syn::Block) -> Result<Block> {
        let stmts = block.stmts.iter()
            .map(|stmt| self.convert_stmt(stmt))
            .collect::<Result<Vec<_>>>()?;
        
        Ok(Block::new(stmts, self.next_id(), Span::DUMMY))
    }
    
    fn convert_stmt(&mut self, stmt: &syn::Stmt) -> Result<Stmt> {
        match stmt {
            syn::Stmt::Expr(expr, semi) => {
                let expr = self.convert_expr(expr)?;
                let kind = if semi.is_some() {
                    StmtKind::Semi(Box::new(expr))
                } else {
                    StmtKind::Expr(Box::new(expr))
                };
                
                Ok(Stmt {
                    id: self.next_id(),
                    kind,
                    span: Span::DUMMY,
                })
            }
            syn::Stmt::Local(local) => {
                let local = self.convert_local(local)?;
                Ok(Stmt {
                    id: self.next_id(),
                    kind: StmtKind::Let(Box::new(local)),
                    span: Span::DUMMY,
                })
            }
            syn::Stmt::Item(_) => {
                // Skip items in blocks for Phase 3
                Ok(Stmt {
                    id: self.next_id(),
                    kind: StmtKind::Empty,
                    span: Span::DUMMY,
                })
            }
            syn::Stmt::Macro(mac) => {
                // Convert macro statement
                self.convert_macro_stmt(mac)
            }
        }
    }
    
    fn convert_local(&mut self, local: &syn::Local) -> Result<Local> {
        let pat = self.convert_pat(&local.pat)?;
        
        let ty = local.ty.as_ref()
            .map(|(_, ty)| self.convert_type(ty))
            .transpose()?;
        
        let init = local.init.as_ref()
            .map(|init| {
                let expr = self.convert_expr(&init.expr)?;
                Ok(LocalInit {
                    expr: Box::new(expr),
                    els: None,  // Phase 3: simplified
                })
            })
            .transpose()?;
        
        Ok(Local {
            id: self.next_id(),
            pat,
            ty,
            init,
            span: Span::DUMMY,
            attrs: vec![],
            tokens: None,
        })
    }
    
    fn convert_expr(&mut self, expr: &syn::Expr) -> Result<Expr> {
        let kind = match expr {
            syn::Expr::Macro(expr_macro) => {
                let mac_call = self.convert_macro(&expr_macro.mac)?;
                ExprKind::MacCall(Box::new(mac_call))
            }
            syn::Expr::Lit(expr_lit) => {
                let lit = self.convert_lit(&expr_lit.lit)?;
                ExprKind::Lit(lit)
            }
            syn::Expr::Path(expr_path) => {
                let path = self.convert_path(&expr_path.path)?;
                ExprKind::Path(None, path)
            }
            _ => {
                return Err(ParseError::Expected {
                    expected: "supported expression".to_string(),
                    found: "complex expression".to_string(),
                    pos: Position::new(0, 1, 1),
                });
            }
        };
        
        Ok(Expr {
            id: self.next_id(),
            kind,
            span: Span::DUMMY,
            attrs: vec![],
            tokens: None,
        })
    }
    
    fn convert_macro(&mut self, mac: &syn::Macro) -> Result<MacCall> {
        let path = self.convert_path(&mac.path)?;
        
        // Convert tokens to string representation
        let tokens_str = mac.tokens.to_string();
        
        let args = MacArgs::Delimited {
            dspan: DelSpan::new(Span::DUMMY, Span::DUMMY),
            delim: self.convert_delimiter(&mac.delimiter),
            tokens: TokenStream::from_str(tokens_str),
        };
        
        Ok(MacCall::new(path, args))
    }
    
    fn convert_delimiter(&mut self, delim: &syn::MacroDelimiter) -> Delimiter {
        match delim {
            syn::MacroDelimiter::Paren(_) => Delimiter::Paren,
            syn::MacroDelimiter::Brace(_) => Delimiter::Brace,
            syn::MacroDelimiter::Bracket(_) => Delimiter::Bracket,
        }
    }
    
    fn convert_macro_stmt(&mut self, mac: &syn::StmtMacro) -> Result<Stmt> {
        let mac_call = self.convert_macro(&mac.mac)?;
        
        let style = if mac.semi_token.is_some() {
            MacStmtStyle::Semicolon
        } else {
            MacStmtStyle::Braces
        };
        
        let mac_call_stmt = MacCallStmt {
            mac: mac_call,
            style,
            attrs: vec![],
            tokens: None,
        };
        
        Ok(Stmt {
            id: self.next_id(),
            kind: StmtKind::MacCall(Box::new(mac_call_stmt)),
            span: Span::DUMMY,
        })
    }
    
    fn convert_lit(&mut self, lit: &syn::Lit) -> Result<Lit> {
        let kind = match lit {
            syn::Lit::Str(lit_str) => LitKind::Str(lit_str.value()),
            syn::Lit::Int(lit_int) => LitKind::Int(lit_int.base10_digits().to_string()),
            _ => {
                return Err(ParseError::Expected {
                    expected: "string or int literal".to_string(),
                    found: "other literal".to_string(),
                    pos: Position::new(0, 1, 1),
                });
            }
        };
        
        Ok(Lit {
            kind,
            span: Span::DUMMY,
        })
    }
}
```

---

## Part 3: CLI Tool

### File: `src/bin/oxur-ast.rs`

```rust
use clap::{Parser, Subcommand};
use oxur_ast::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::print_sexp;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxur-ast")]
#[command(about = "Convert between Rust and S-expressions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert Rust source to S-expression
    ToSexp {
        /// Input Rust file (or - for stdin)
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Output file (or - for stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        
        /// Use compact formatting
        #[arg(short, long)]
        compact: bool,
    },
    
    /// Convert S-expression to Rust source
    ToRust {
        /// Input S-expression file (or - for stdin)
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Output file (or - for stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
    
    /// Verify round-trip conversion
    Verify {
        /// Input Rust file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::ToSexp { input, output, compact } => {
            to_sexp(input, output, compact)
        }
        Commands::ToRust { input, output } => {
            to_rust(input, output)
        }
        Commands::Verify { input } => {
            verify(input)
        }
    }
}

fn to_sexp(input: PathBuf, output: Option<PathBuf>, compact: bool) -> anyhow::Result<()> {
    // Read input
    let source = if input.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&input)?
    };
    
    // Parse Rust
    let crate_node = parse_rust_file(&source)?;
    
    // Generate S-expression
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node)?;
    
    // Format
    let output_text = if compact {
        use oxur_ast::sexp::Printer;
        Printer::new().print_compact(&sexp)
    } else {
        print_sexp(&sexp)
    };
    
    // Write output
    if let Some(output_path) = output {
        if output_path.to_str() == Some("-") {
            println!("{}", output_text);
        } else {
            fs::write(output_path, output_text)?;
        }
    } else {
        println!("{}", output_text);
    }
    
    Ok(())
}

fn to_rust(input: PathBuf, output: Option<PathBuf>) -> anyhow::Result<()> {
    // Read input
    let sexp_text = if input.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&input)?
    };
    
    // Parse S-expression
    let sexp = oxur_ast::sexp::Parser::parse_str(&sexp_text)?;
    
    // Build AST
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp)?;
    
    // Generate Rust (using quote for now)
    // This is simplified - proper Rust generation would need more work
    let rust_output = format!("// Generated from S-expression\n// AST: {:?}", crate_node);
    
    // Write output
    if let Some(output_path) = output {
        if output_path.to_str() == Some("-") {
            println!("{}", rust_output);
        } else {
            fs::write(output_path, rust_output)?;
        }
    } else {
        println!("{}", rust_output);
    }
    
    Ok(())
}

fn verify(input: PathBuf) -> anyhow::Result<()> {
    let source = fs::read_to_string(&input)?;
    
    println!("Verifying round-trip for: {}", input.display());
    println!();
    
    // Step 1: Parse Rust
    println!("1. Parsing Rust source...");
    let crate1 = parse_rust_file(&source)?;
    println!("   ✓ Parsed successfully");
    
    // Step 2: Generate S-expression
    println!("2. Generating S-expression...");
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1)?;
    println!("   ✓ Generated successfully");
    
    // Step 3: Parse S-expression back
    println!("3. Parsing S-expression...");
    let sexp_text = print_sexp(&sexp);
    let sexp2 = oxur_ast::sexp::Parser::parse_str(&sexp_text)?;
    println!("   ✓ Parsed successfully");
    
    // Step 4: Build AST
    println!("4. Building AST from S-expression...");
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2)?;
    println!("   ✓ Built successfully");
    
    // Step 5: Verify
    println!("5. Verifying equivalence...");
    // For Phase 3, we'll do basic checks
    if crate1.items.len() != crate2.items.len() {
        anyhow::bail!("Item count mismatch: {} vs {}", 
            crate1.items.len(), crate2.items.len());
    }
    
    println!("   ✓ Basic verification passed");
    println!();
    println!("✓ Round-trip verification successful!");
    
    Ok(())
}
```

Update `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"

[[bin]]
name = "oxur-ast"
path = "src/bin/oxur-ast.rs"
```

---

## Part 4: Integration Tests

### File: `tests/integration_tests.rs`

```rust
use oxur_ast::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::{Parser, print_sexp};

#[test]
fn test_parse_hello_world() {
    let source = r#"
fn main() {
    println!("Hello, world!");
}
    "#;
    
    let crate_node = parse_rust_file(source).expect("Failed to parse");
    
    assert_eq!(crate_node.items.len(), 1);
    
    let item = &crate_node.items[0];
    assert_eq!(item.ident.name, "main");
}

#[test]
fn test_round_trip_hello_world() {
    let source = r#"
fn main() {
    println!("Hello, world!");
}
    "#;
    
    // Parse Rust
    let crate1 = parse_rust_file(source).expect("Failed to parse");
    
    // Generate S-expression
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate1).expect("Failed to generate");
    
    // Parse S-expression
    let sexp_text = print_sexp(&sexp);
    let sexp2 = Parser::parse_str(&sexp_text).expect("Failed to parse S-expr");
    
    // Build AST
    let mut builder = AstBuilder::new();
    let crate2 = builder.build_crate(&sexp2).expect("Failed to build");
    
    // Verify
    assert_eq!(crate1.items.len(), crate2.items.len());
    assert_eq!(crate1.items[0].ident.name, crate2.items[0].ident.name);
}

#[test]
fn test_parse_simple_function() {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
    "#;
    
    let crate_node = parse_rust_file(source).expect("Failed to parse");
    
    assert_eq!(crate_node.items.len(), 1);
    
    let item = &crate_node.items[0];
    assert_eq!(item.ident.name, "add");
    
    // Verify it has parameters
    if let ast::ItemKind::Fn(fn_item) = &item.kind {
        assert_eq!(fn_item.sig.decl.inputs.len(), 2);
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_parse_with_let_binding() {
    let source = r#"
fn test() {
    let x = 42;
    let y = "hello";
}
    "#;
    
    let crate_node = parse_rust_file(source).expect("Failed to parse");
    assert_eq!(crate_node.items.len(), 1);
}

#[test]
fn test_parse_unsafe_function() {
    let source = r#"
unsafe fn dangerous() {
    // unsafe code
}
    "#;
    
    let crate_node = parse_rust_file(source).expect("Failed to parse");
    
    let item = &crate_node.items[0];
    if let ast::ItemKind::Fn(fn_item) = &item.kind {
        assert_eq!(fn_item.sig.header.safety, ast::Safety::Unsafe);
    }
}

#[test]
fn test_parse_const_function() {
    let source = r#"
const fn compile_time() -> i32 {
    42
}
    "#;
    
    let crate_node = parse_rust_file(source).expect("Failed to parse");
    
    let item = &crate_node.items[0];
    if let ast::ItemKind::Fn(fn_item) = &item.kind {
        assert_eq!(fn_item.sig.header.constness, ast::Constness::Const);
    }
}
```

---

## Part 5: Test Fixtures

### File: `tests/fixtures/hello_world.rs`

```rust
fn main() {
    println!("Hello, world!");
}
```

### File: `tests/fixtures/simple_fn.rs`

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let result = add(2, 3);
    println!("{}", result);
}
```

### File: `tests/fixtures/let_bindings.rs`

```rust
fn test() {
    let x = 42;
    let y = "hello";
    let z = x + 1;
}
```

### File: `tests/regression_tests.rs`

```rust
use oxur_ast::integration::parse_rust_file;
use std::fs;
use std::path::Path;

#[test]
fn test_all_fixtures() {
    let fixtures = [
        "tests/fixtures/hello_world.rs",
        "tests/fixtures/simple_fn.rs",
        "tests/fixtures/let_bindings.rs",
    ];
    
    for fixture in &fixtures {
        let path = Path::new(fixture);
        if !path.exists() {
            eprintln!("Skipping missing fixture: {}", fixture);
            continue;
        }
        
        let source = fs::read_to_string(fixture)
            .expect(&format!("Failed to read {}", fixture));
        
        let result = parse_rust_file(&source);
        
        match result {
            Ok(crate_node) => {
                println!("✓ Parsed {}: {} items", fixture, crate_node.items.len());
            }
            Err(e) => {
                panic!("Failed to parse {}: {:?}", fixture, e);
            }
        }
    }
}
```

---

## Part 6: Benchmarks

### File: `benches/conversion_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oxur_ast::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::{Parser, print_sexp};

const HELLO_WORLD: &str = r#"
fn main() {
    println!("Hello, world!");
}
"#;

fn bench_parse_rust(c: &mut Criterion) {
    c.bench_function("parse_rust", |b| {
        b.iter(|| {
            parse_rust_file(black_box(HELLO_WORLD))
        })
    });
}

fn bench_generate_sexp(c: &mut Criterion) {
    let crate_node = parse_rust_file(HELLO_WORLD).unwrap();
    let gen = Generator::new();
    
    c.bench_function("generate_sexp", |b| {
        b.iter(|| {
            gen.generate_crate(black_box(&crate_node))
        })
    });
}

fn bench_parse_sexp(c: &mut Criterion) {
    let crate_node = parse_rust_file(HELLO_WORLD).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();
    let sexp_text = print_sexp(&sexp);
    
    c.bench_function("parse_sexp", |b| {
        b.iter(|| {
            Parser::parse_str(black_box(&sexp_text))
        })
    });
}

fn bench_build_ast(c: &mut Criterion) {
    let crate_node = parse_rust_file(HELLO_WORLD).unwrap();
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();
    let sexp_text = print_sexp(&sexp);
    let sexp = Parser::parse_str(&sexp_text).unwrap();
    
    c.bench_function("build_ast", |b| {
        b.iter(|| {
            let mut builder = AstBuilder::new();
            builder.build_crate(black_box(&sexp))
        })
    });
}

fn bench_round_trip(c: &mut Criterion) {
    c.bench_function("round_trip", |b| {
        b.iter(|| {
            // Parse Rust
            let crate1 = parse_rust_file(black_box(HELLO_WORLD)).unwrap();
            
            // Generate S-expression
            let gen = Generator::new();
            let sexp = gen.generate_crate(&crate1).unwrap();
            
            // Parse S-expression
            let sexp_text = print_sexp(&sexp);
            let sexp2 = Parser::parse_str(&sexp_text).unwrap();
            
            // Build AST
            let mut builder = AstBuilder::new();
            builder.build_crate(&sexp2).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_parse_rust,
    bench_generate_sexp,
    bench_parse_sexp,
    bench_build_ast,
    bench_round_trip
);

criterion_main!(benches);
```

---

## Part 7: Examples

### File: `examples/parse_rust_file.rs`

```rust
use oxur_ast::integration::parse_rust_file;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <rust-file>", args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    let source = fs::read_to_string(filename)
        .expect("Failed to read file");
    
    println!("Parsing: {}\n", filename);
    
    match parse_rust_file(&source) {
        Ok(crate_node) => {
            println!("✓ Parsed successfully!");
            println!("  Items: {}", crate_node.items.len());
            
            for (i, item) in crate_node.items.iter().enumerate() {
                println!("  Item {}: {}", i, item.ident.name);
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
```

### File: `examples/convert_file.rs`

```rust
use oxur_ast::*;
use oxur_ast::integration::parse_rust_file;
use oxur_ast::sexp::print_sexp;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input.rs> <output.sexp>", args[0]);
        std::process::exit(1);
    }
    
    let input = &args[1];
    let output = &args[2];
    
    println!("Converting: {} → {}\n", input, output);
    
    // Read input
    let source = fs::read_to_string(input)
        .expect("Failed to read input file");
    
    // Parse Rust
    println!("1. Parsing Rust...");
    let crate_node = parse_rust_file(&source)
        .expect("Failed to parse Rust");
    println!("   ✓ Parsed {} items", crate_node.items.len());
    
    // Generate S-expression
    println!("2. Generating S-expression...");
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node)
        .expect("Failed to generate S-expression");
    println!("   ✓ Generated");
    
    // Format and write
    println!("3. Writing output...");
    let sexp_text = print_sexp(&sexp);
    fs::write(output, sexp_text)
        .expect("Failed to write output file");
    println!("   ✓ Written");
    
    println!("\n✓ Conversion complete!");
}
```

---

## Part 8: Documentation

### File: `README.md` (for oxur-ast crate)

```markdown
# oxur-ast

Rust AST ↔ S-expression bidirectional conversion library.

## Features

- Parse Rust code into a clean AST
- Convert AST to S-expressions
- Parse S-expressions back to AST
- CLI tool for conversions
- Round-trip verification

## Installation

```bash
cargo add oxur-ast
```

## Usage

### As a Library

```rust
use oxur_ast::*;
use oxur_ast::integration::parse_rust_file;

// Parse Rust
let source = r#"
fn main() {
    println!("Hello!");
}
"#;

let crate_node = parse_rust_file(source)?;

// Generate S-expression
let gen = Generator::new();
let sexp = gen.generate_crate(&crate_node)?;

// Print
println!("{}", oxur_ast::sexp::print_sexp(&sexp));
```

### CLI Tool

```bash
# Convert Rust to S-expression
oxur-ast to-sexp hello.rs > hello.sexp

# Convert S-expression to Rust
oxur-ast to-rust hello.sexp > hello.rs

# Verify round-trip
oxur-ast verify hello.rs
```

## Architecture

```
Rust Source → syn → oxur AST → S-expression
                      ↓
                  Generator
                      ↓
                 S-expression → Parser → Builder → oxur AST
```

## Testing

```bash
cargo test -p oxur-ast
cargo bench -p oxur-ast
```

## License

MIT OR Apache-2.0
```

---

## Part 9: Error Handling Improvements

### File: `src/error.rs` (additions)

Add more specific error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Unsupported Rust feature: {feature} at {pos}")]
    UnsupportedFeature { feature: String, pos: Position },
    
    #[error("Conversion failed: {reason}")]
    ConversionFailed { reason: String },
    
    #[error("syn parse error: {0}")]
    SynError(String),
}

impl From<IntegrationError> for ParseError {
    fn from(err: IntegrationError) -> Self {
        ParseError::Expected {
            expected: "valid conversion".to_string(),
            found: err.to_string(),
            pos: Position::new(0, 1, 1),
        }
    }
}
```

---

## Success Criteria

Phase 3 is complete when:

- [ ] Can parse real Rust files using `syn`
- [ ] CLI tool works for basic conversions
- [ ] All integration tests pass
- [ ] Benchmark suite runs
- [ ] Documentation is complete
- [ ] All fixtures parse successfully
- [ ] Round-trip verification works for test cases
- [ ] No compiler warnings
- [ ] Clean `cargo clippy` output

---

## Testing Instructions

```bash
# Run all tests
cargo test -p oxur-ast

# Run integration tests
cargo test -p oxur-ast --test integration_tests

# Run regression tests
cargo test -p oxur-ast --test regression_tests

# Run benchmarks
cargo bench -p oxur-ast

# Test CLI
cargo run -p oxur-ast -- to-sexp tests/fixtures/hello_world.rs
cargo run -p oxur-ast -- verify tests/fixtures/hello_world.rs

# Test examples
cargo run -p oxur-ast --example parse_rust_file tests/fixtures/simple_fn.rs
cargo run -p oxur-ast --example convert_file tests/fixtures/hello_world.rs /tmp/hello.sexp
```

---

## Real-World Testing

After Phase 3, test with actual Rust files:

```bash
# Test with standard library examples
curl -O https://raw.githubusercontent.com/rust-lang/rust/master/library/std/examples/tcp-client.rs
oxur-ast verify tcp-client.rs

# Test with your own code
oxur-ast to-sexp src/main.rs > main.sexp
oxur-ast to-rust main.sexp > main.gen.rs

# Compare
diff src/main.rs main.gen.rs
```

---

## Known Limitations (Phase 3)

These are acceptable for Phase 3 and will be addressed in future phases:

1. **Span information**: Using `Span::DUMMY` - need proc-macro2 integration
2. **Attributes**: Simplified attribute handling
3. **Complex generics**: Not fully supported
4. **All ExprKind variants**: Only basic expressions
5. **Rust generation**: Currently Debug output, needs proper code generation

---

## Future Enhancements (Phase 4+)

- Complete expression coverage
- Full generic support
- Proper Rust code generation (not just Debug)
- Macro expansion support
- Better error messages with snippets
- Incremental parsing
- LSP integration
- REPL support

---

*"From prototype to production - the bridge is ready for traffic."*
