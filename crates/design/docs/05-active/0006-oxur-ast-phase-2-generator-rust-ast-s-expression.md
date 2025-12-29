---
number: 6
title: "oxur-ast Phase 2: Generator (Rust AST → S-expression)"
author: Duncan McGreggor & Claude
created: 2025-12-27
updated: 2025-12-27
state: Active
supersedes: null
superseded-by: null
---

# oxur-ast Phase 2: Generator (Rust AST → S-expression)

**Phase**: 2 - Generator
**Goal**: Generate S-expressions from Rust AST
**Estimated Time**: 4-6 days
**Prerequisites**: Phase 0 (S-expr) and Phase 1 (AST & Builder) complete

---

## Overview

Phase 2 builds the reverse direction: converting Rust AST back into S-expressions. Combined with Phase 1, this completes the bidirectional conversion layer.

**What we're building:**

1. AST visitor/walker infrastructure
2. Generator for each AST node type
3. Pretty-printer integration
4. Round-trip verification tests

**The complete flow:**

```
S-expression → (Parser) → SExp AST → (Builder) → Rust AST
Rust AST → (Generator) → SExp AST → (Printer) → S-expression text
```

**Round-trip guarantee:**

```rust
fn main() { println!("Hello, world!"); }
  ↓ [rustc parser]
Rust AST
  ↓ [Generator - Phase 2]
S-expression
  ↓ [Builder - Phase 1]
Rust AST (should be equivalent)
```

---

## File Structure

Extend `oxur-ast` with:

```
oxur-ast/
├── src/
│   ├── generator/
│   │   ├── mod.rs       # Generator module exports
│   │   ├── gen.rs       # Main generator logic
│   │   ├── item.rs      # Item generation
│   │   ├── expr.rs      # Expression generation
│   │   ├── stmt.rs      # Statement generation
│   │   └── helpers.rs   # Shared utilities
├── tests/
│   ├── generator_tests.rs    # Generator tests
│   └── round_trip_tests.rs   # Full round-trip tests
└── examples/
    └── generate_hello.rs      # Generate Hello World S-expr
```

---

## Part 1: Generator Infrastructure

### File: `src/generator/helpers.rs`

Utilities for building S-expressions:

```rust
use crate::sexp::{SExp, Symbol, Keyword, StringLit, Number, List};
use crate::error::Position;

/// Create a symbol S-expression
pub fn sym(name: impl Into<String>) -> SExp {
    SExp::Symbol(Symbol::new(name, Position::new(0, 1, 1)))
}

/// Create a keyword S-expression
pub fn kw(name: impl Into<String>) -> SExp {
    SExp::Keyword(Keyword::new(name, Position::new(0, 1, 1)))
}

/// Create a string S-expression
pub fn string(value: impl Into<String>) -> SExp {
    SExp::String(StringLit::new(value, Position::new(0, 1, 1)))
}

/// Create a number S-expression
pub fn num(value: impl ToString) -> SExp {
    SExp::Number(Number::new(value.to_string(), Position::new(0, 1, 1)))
}

/// Create a list S-expression
pub fn list(elements: Vec<SExp>) -> SExp {
    SExp::List(List::new(elements, Position::new(0, 1, 1)))
}

/// Create an empty list
pub fn empty_list() -> SExp {
    list(vec![])
}

/// Create a keyword-value pair
pub fn kwarg(key: &str, value: SExp) -> Vec<SExp> {
    vec![kw(key), value]
}

/// Create a typed node: (Type :field1 val1 :field2 val2 ...)
pub fn typed_node(type_name: &str, fields: Vec<SExp>) -> SExp {
    let mut elements = vec![sym(type_name)];
    elements.extend(fields);
    list(elements)
}

/// Flatten multiple kwarg pairs into a single vec
pub fn kwargs(pairs: Vec<Vec<SExp>>) -> Vec<SExp> {
    pairs.into_iter().flatten().collect()
}
```

---

## Part 2: Main Generator

### File: `src/generator/gen.rs`

Core generator logic:

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::generator::helpers::*;

pub struct Generator;

impl Generator {
    pub fn new() -> Self {
        Self
    }

    /// Generate S-expression from Crate
    pub fn generate_crate(&self, crate_node: &Crate) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&crate_node.attrs)?),
            kwarg("items", self.generate_items(&crate_node.items)?),
            kwarg("spans", self.generate_mod_spans(&crate_node.spans)?),
            kwarg("id", self.generate_node_id(crate_node.id)),
            kwarg("is-placeholder", sym(if crate_node.is_placeholder { "true" } else { "false" })),
        ]);

        Ok(typed_node("Crate", fields))
    }

    fn generate_attr_vec(&self, attrs: &AttrVec) -> Result<SExp> {
        // Phase 2: Just empty list for now
        // Will expand in future phases
        Ok(empty_list())
    }

    fn generate_items(&self, items: &[Item]) -> Result<SExp> {
        let item_sexps: Result<Vec<SExp>> = items.iter()
            .map(|item| self.generate_item(item))
            .collect();
        Ok(list(item_sexps?))
    }

    fn generate_mod_spans(&self, spans: &ModSpans) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("inner-span", self.generate_span(spans.inner_span)),
            kwarg("inject-use-span", self.generate_span(spans.inject_use_span)),
        ]);

        Ok(typed_node("ModSpans", fields))
    }

    pub fn generate_span(&self, span: Span) -> SExp {
        let fields = kwargs(vec![
            kwarg("lo", num(span.lo)),
            kwarg("hi", num(span.hi)),
        ]);

        // Only include ctxt if non-zero
        let fields = if span.ctxt != 0 {
            let mut f = fields;
            f.extend(kwarg("ctxt", num(span.ctxt)));
            f
        } else {
            fields
        };

        typed_node("Span", fields)
    }

    fn generate_node_id(&self, id: NodeId) -> SExp {
        num(id.0)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Part 3: Item Generation

### File: `src/generator/item.rs`

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::generator::helpers::*;
use crate::generator::gen::Generator;

impl Generator {
    pub fn generate_item(&self, item: &Item) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&item.attrs)?),
            kwarg("id", self.generate_node_id(item.id)),
            kwarg("span", self.generate_span(item.span)),
            kwarg("vis", self.generate_visibility(&item.vis)),
            kwarg("ident", self.generate_ident(&item.ident)),
            kwarg("kind", self.generate_item_kind(&item.kind)?),
        ]);

        // Only include tokens if present
        let fields = if item.tokens.is_some() {
            let mut f = fields;
            f.extend(kwarg("tokens", sym("nil")));
            f
        } else {
            fields
        };

        Ok(typed_node("Item", fields))
    }

    fn generate_visibility(&self, vis: &Visibility) -> SExp {
        match vis {
            Visibility::Public => list(vec![sym("Public")]),
            Visibility::Inherited => list(vec![sym("Inherited")]),
            Visibility::Restricted { path, shorthand, span } => {
                let fields = kwargs(vec![
                    kwarg("path", self.generate_path(path)),
                    kwarg("shorthand", self.generate_vis_restriction_kind(*shorthand)),
                    kwarg("span", self.generate_span(*span)),
                ]);
                typed_node("Restricted", fields)
            }
        }
    }

    fn generate_vis_restriction_kind(&self, kind: VisRestrictionKind) -> SExp {
        match kind {
            VisRestrictionKind::Crate => sym("Crate"),
            VisRestrictionKind::Super => sym("Super"),
            VisRestrictionKind::In => sym("In"),
        }
    }

    pub fn generate_ident(&self, ident: &Ident) -> SExp {
        let fields = kwargs(vec![
            kwarg("name", string(&ident.name)),
            kwarg("span", self.generate_span(ident.span)),
        ]);

        typed_node("Ident", fields)
    }

    fn generate_item_kind(&self, kind: &ItemKind) -> Result<SExp> {
        match kind {
            ItemKind::Fn(fn_item) => {
                let fields = kwargs(vec![
                    kwarg("defaultness", self.generate_defaultness(fn_item.defaultness)),
                    kwarg("sig", self.generate_fn_sig(&fn_item.sig)?),
                    kwarg("generics", self.generate_generics(&fn_item.generics)?),
                ]);

                // Only include body if present
                let fields = if let Some(body) = &fn_item.body {
                    let mut f = fields;
                    f.extend(kwarg("body", self.generate_block(body)?));
                    f
                } else {
                    fields
                };

                Ok(typed_node("Fn", fields))
            }
        }
    }

    fn generate_defaultness(&self, defaultness: Defaultness) -> SExp {
        match defaultness {
            Defaultness::Default => sym("Default"),
            Defaultness::Final => sym("Final"),
        }
    }

    fn generate_fn_sig(&self, sig: &FnSig) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("header", self.generate_fn_header(&sig.header)),
            kwarg("decl", self.generate_fn_decl(&sig.decl)?),
            kwarg("span", self.generate_span(sig.span)),
        ]);

        Ok(typed_node("FnSig", fields))
    }

    fn generate_fn_header(&self, header: &FnHeader) -> SExp {
        let mut fields = kwargs(vec![
            kwarg("safety", self.generate_safety(header.safety)),
            kwarg("constness", self.generate_constness(header.constness)),
            kwarg("ext", self.generate_extern(&header.ext)),
        ]);

        // Only include coroutine_kind if present
        if let Some(kind) = header.coroutine_kind {
            fields.extend(kwarg("coroutine-kind", self.generate_coroutine_kind(kind)));
        } else {
            fields.extend(kwarg("coroutine-kind", sym("nil")));
        }

        typed_node("FnHeader", fields)
    }

    fn generate_safety(&self, safety: Safety) -> SExp {
        match safety {
            Safety::Unsafe => sym("Unsafe"),
            Safety::Safe => sym("Safe"),
            Safety::Default => sym("Default"),
        }
    }

    fn generate_constness(&self, constness: Constness) -> SExp {
        match constness {
            Constness::Const => sym("Const"),
            Constness::NotConst => sym("NotConst"),
        }
    }

    fn generate_extern(&self, ext: &Extern) -> SExp {
        match ext {
            Extern::None => sym("None"),
            Extern::Explicit(abi) => {
                list(vec![sym("Explicit"), string(abi)])
            }
        }
    }

    fn generate_coroutine_kind(&self, kind: CoroutineKind) -> SExp {
        match kind {
            CoroutineKind::Async => sym("Async"),
            CoroutineKind::Gen => sym("Gen"),
        }
    }

    fn generate_fn_decl(&self, decl: &FnDecl) -> Result<SExp> {
        let inputs = self.generate_params(&decl.inputs)?;
        let output = self.generate_fn_ret_ty(&decl.output)?;

        let fields = kwargs(vec![
            kwarg("inputs", inputs),
            kwarg("output", output),
        ]);

        Ok(typed_node("FnDecl", fields))
    }

    fn generate_params(&self, params: &[Param]) -> Result<SExp> {
        let param_sexps: Result<Vec<SExp>> = params.iter()
            .map(|p| self.generate_param(p))
            .collect();
        Ok(list(param_sexps?))
    }

    fn generate_param(&self, param: &Param) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&param.attrs)?),
            kwarg("ty", self.generate_ty(&param.ty)?),
            kwarg("pat", self.generate_pat(&param.pat)?),
            kwarg("id", self.generate_node_id(param.id)),
            kwarg("span", self.generate_span(param.span)),
            kwarg("is-placeholder", sym(if param.is_placeholder { "true" } else { "false" })),
        ]);

        Ok(typed_node("Param", fields))
    }

    fn generate_fn_ret_ty(&self, ret_ty: &FnRetTy) -> Result<SExp> {
        match ret_ty {
            FnRetTy::Default(span) => {
                Ok(list(vec![sym("Default"), self.generate_span(*span)]))
            }
            FnRetTy::Ty(ty) => {
                Ok(list(vec![sym("Ty"), self.generate_ty(ty)?]))
            }
        }
    }

    fn generate_generics(&self, generics: &Generics) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("params", self.generate_generic_params(&generics.params)?),
            kwarg("where-clause", self.generate_where_clause(&generics.where_clause)?),
            kwarg("span", self.generate_span(generics.span)),
        ]);

        Ok(typed_node("Generics", fields))
    }

    fn generate_generic_params(&self, _params: &[GenericParam]) -> Result<SExp> {
        // Phase 2: Empty for now
        Ok(empty_list())
    }

    fn generate_where_clause(&self, clause: &WhereClause) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("has-where-token", sym(if clause.has_where_token { "true" } else { "false" })),
            kwarg("predicates", self.generate_where_predicates(&clause.predicates)?),
            kwarg("span", self.generate_span(clause.span)),
        ]);

        Ok(typed_node("WhereClause", fields))
    }

    fn generate_where_predicates(&self, _predicates: &[WherePredicate]) -> Result<SExp> {
        // Phase 2: Empty for now
        Ok(empty_list())
    }

    fn generate_ty(&self, ty: &Ty) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("id", self.generate_node_id(ty.id)),
            kwarg("kind", self.generate_ty_kind(&ty.kind)?),
            kwarg("span", self.generate_span(ty.span)),
        ]);

        // Only include tokens if present
        let fields = if ty.tokens.is_some() {
            let mut f = fields;
            f.extend(kwarg("tokens", sym("nil")));
            f
        } else {
            fields
        };

        Ok(typed_node("Ty", fields))
    }

    fn generate_ty_kind(&self, kind: &TyKind) -> Result<SExp> {
        match kind {
            TyKind::Path(qself, path) => {
                let qself_sexp = if let Some(_qself) = qself {
                    // Will implement in future phases
                    sym("nil")
                } else {
                    sym("nil")
                };

                Ok(list(vec![
                    sym("Path"),
                    qself_sexp,
                    self.generate_path(path),
                ]))
            }
        }
    }

    fn generate_pat(&self, pat: &Pat) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("id", self.generate_node_id(pat.id)),
            kwarg("kind", self.generate_pat_kind(&pat.kind)?),
            kwarg("span", self.generate_span(pat.span)),
        ]);

        // Only include tokens if present
        let fields = if pat.tokens.is_some() {
            let mut f = fields;
            f.extend(kwarg("tokens", sym("nil")));
            f
        } else {
            fields
        };

        Ok(typed_node("Pat", fields))
    }

    fn generate_pat_kind(&self, kind: &PatKind) -> Result<SExp> {
        match kind {
            PatKind::Ident(ident) => {
                Ok(list(vec![sym("Ident"), self.generate_ident(ident)]))
            }
        }
    }
}
```

---

## Part 4: Expression Generation

### File: `src/generator/expr.rs`

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::generator::helpers::*;
use crate::generator::gen::Generator;

impl Generator {
    pub fn generate_block(&self, block: &Block) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("stmts", self.generate_stmts(&block.stmts)?),
            kwarg("id", self.generate_node_id(block.id)),
            kwarg("rules", self.generate_block_check_mode(block.rules)),
            kwarg("span", self.generate_span(block.span)),
        ]);

        // Only include optional fields if present
        let mut fields = fields;
        if block.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }
        fields.extend(kwarg("could-be-bare-literal", sym(if block.could_be_bare_literal { "true" } else { "false" })));

        Ok(typed_node("Block", fields))
    }

    fn generate_block_check_mode(&self, mode: BlockCheckMode) -> SExp {
        match mode {
            BlockCheckMode::Default => sym("Default"),
            BlockCheckMode::Unsafe => sym("Unsafe"),
        }
    }

    fn generate_stmts(&self, stmts: &[Stmt]) -> Result<SExp> {
        let stmt_sexps: Result<Vec<SExp>> = stmts.iter()
            .map(|s| self.generate_stmt(s))
            .collect();
        Ok(list(stmt_sexps?))
    }

    pub fn generate_expr(&self, expr: &Expr) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("id", self.generate_node_id(expr.id)),
            kwarg("kind", self.generate_expr_kind(&expr.kind)?),
            kwarg("span", self.generate_span(expr.span)),
            kwarg("attrs", self.generate_attr_vec(&expr.attrs)?),
        ]);

        // Only include tokens if present
        let fields = if expr.tokens.is_some() {
            let mut f = fields;
            f.extend(kwarg("tokens", sym("nil")));
            f
        } else {
            fields
        };

        Ok(typed_node("Expr", fields))
    }

    fn generate_expr_kind(&self, kind: &ExprKind) -> Result<SExp> {
        match kind {
            ExprKind::MacCall(mac_call) => {
                Ok(list(vec![
                    sym("MacCall"),
                    self.generate_mac_call(mac_call)?,
                ]))
            }
            ExprKind::Lit(lit) => {
                Ok(list(vec![
                    sym("Lit"),
                    self.generate_lit(lit)?,
                ]))
            }
            ExprKind::Path(qself, path) => {
                let qself_sexp = if let Some(_qself) = qself {
                    sym("nil")  // Will implement in future
                } else {
                    sym("nil")
                };

                Ok(list(vec![
                    sym("Path"),
                    qself_sexp,
                    self.generate_path(path),
                ]))
            }
        }
    }

    pub fn generate_mac_call(&self, mac_call: &MacCall) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("path", self.generate_path(&mac_call.path)),
            kwarg("args", self.generate_mac_args(&mac_call.args)?),
        ]);

        // Only include prior_type_ascription if present
        if let Some((pos, flag)) = mac_call.prior_type_ascription {
            fields.extend(kwarg("prior-type-ascription",
                list(vec![num(pos), sym(if flag { "true" } else { "false" })])));
        } else {
            fields.extend(kwarg("prior-type-ascription", sym("nil")));
        }

        Ok(typed_node("MacCall", fields))
    }

    pub fn generate_path(&self, path: &Path) -> SExp {
        let mut fields = kwargs(vec![
            kwarg("span", self.generate_span(path.span)),
            kwarg("segments", self.generate_path_segments(&path.segments)),
        ]);

        // Only include tokens if present
        if path.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }

        typed_node("Path", fields)
    }

    fn generate_path_segments(&self, segments: &[PathSegment]) -> SExp {
        let seg_sexps: Vec<SExp> = segments.iter()
            .map(|s| self.generate_path_segment(s))
            .collect();
        list(seg_sexps)
    }

    fn generate_path_segment(&self, segment: &PathSegment) -> SExp {
        let mut fields = kwargs(vec![
            kwarg("ident", self.generate_ident(&segment.ident)),
            kwarg("id", self.generate_node_id(segment.id)),
        ]);

        // Only include args if present
        if segment.args.is_some() {
            fields.extend(kwarg("args", sym("nil")));  // Will implement in future
        } else {
            fields.extend(kwarg("args", sym("nil")));
        }

        typed_node("PathSegment", fields)
    }

    fn generate_mac_args(&self, args: &MacArgs) -> Result<SExp> {
        match args {
            MacArgs::Empty => Ok(list(vec![sym("Empty")])),
            MacArgs::Delimited { dspan, delim, tokens } => {
                let fields = kwargs(vec![
                    kwarg("dspan", self.generate_del_span(*dspan)),
                    kwarg("delim", self.generate_delimiter(*delim)),
                    kwarg("tokens", self.generate_token_stream(tokens)?),
                ]);
                Ok(typed_node("Delimited", fields))
            }
            MacArgs::Eq { eq_span, tokens } => {
                let fields = kwargs(vec![
                    kwarg("eq-span", self.generate_span(*eq_span)),
                    kwarg("tokens", self.generate_token_stream(tokens)?),
                ]);
                Ok(typed_node("Eq", fields))
            }
        }
    }

    fn generate_del_span(&self, dspan: DelSpan) -> SExp {
        let fields = kwargs(vec![
            kwarg("open", self.generate_span(dspan.open)),
            kwarg("close", self.generate_span(dspan.close)),
        ]);
        typed_node("DelSpan", fields)
    }

    fn generate_delimiter(&self, delim: Delimiter) -> SExp {
        match delim {
            Delimiter::Paren => sym("Paren"),
            Delimiter::Brace => sym("Brace"),
            Delimiter::Bracket => sym("Bracket"),
            Delimiter::Invisible => sym("Invisible"),
        }
    }

    fn generate_token_stream(&self, tokens: &TokenStream) -> Result<SExp> {
        match tokens {
            TokenStream::Empty => Ok(list(vec![sym("Empty")])),
            TokenStream::Source(s) => {
                let fields = kwargs(vec![
                    kwarg("source", string(s)),
                ]);
                Ok(typed_node("TokenStream", fields))
            }
        }
    }

    fn generate_lit(&self, lit: &Lit) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("kind", self.generate_lit_kind(&lit.kind)?),
            kwarg("span", self.generate_span(lit.span)),
        ]);
        Ok(typed_node("Lit", fields))
    }

    fn generate_lit_kind(&self, kind: &LitKind) -> Result<SExp> {
        match kind {
            LitKind::Str(s) => {
                Ok(list(vec![sym("Str"), string(s)]))
            }
            LitKind::Int(i) => {
                Ok(list(vec![sym("Int"), string(i)]))
            }
        }
    }
}
```

---

## Part 5: Statement Generation

### File: `src/generator/stmt.rs`

```rust
use crate::error::Result;
use crate::sexp::SExp;
use crate::ast::*;
use crate::generator::helpers::*;
use crate::generator::gen::Generator;

impl Generator {
    pub fn generate_stmt(&self, stmt: &Stmt) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("id", self.generate_node_id(stmt.id)),
            kwarg("kind", self.generate_stmt_kind(&stmt.kind)?),
            kwarg("span", self.generate_span(stmt.span)),
        ]);

        Ok(typed_node("Stmt", fields))
    }

    fn generate_stmt_kind(&self, kind: &StmtKind) -> Result<SExp> {
        match kind {
            StmtKind::Expr(expr) => {
                Ok(list(vec![
                    sym("Expr"),
                    self.generate_expr(expr)?,
                ]))
            }
            StmtKind::Semi(expr) => {
                Ok(list(vec![
                    sym("Semi"),
                    self.generate_expr(expr)?,
                ]))
            }
            StmtKind::Let(local) => {
                Ok(list(vec![
                    sym("Let"),
                    self.generate_local(local)?,
                ]))
            }
            StmtKind::Item(item) => {
                Ok(list(vec![
                    sym("Item"),
                    self.generate_item(item)?,
                ]))
            }
            StmtKind::MacCall(mac_call_stmt) => {
                Ok(list(vec![
                    sym("MacCall"),
                    self.generate_mac_call_stmt(mac_call_stmt)?,
                ]))
            }
            StmtKind::Empty => {
                Ok(sym("Empty"))
            }
        }
    }

    fn generate_local(&self, local: &Local) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("id", self.generate_node_id(local.id)),
            kwarg("pat", self.generate_pat(&local.pat)?),
            kwarg("span", self.generate_span(local.span)),
            kwarg("attrs", self.generate_attr_vec(&local.attrs)?),
        ]);

        // Optional ty
        if let Some(ty) = &local.ty {
            fields.extend(kwarg("ty", self.generate_ty(ty)?));
        }

        // Optional init
        if let Some(init) = &local.init {
            fields.extend(kwarg("init", self.generate_local_init(init)?));
        }

        // Optional tokens
        if local.tokens.is_some() {
            fields.extend(kwarg("tokens", sym("nil")));
        }

        Ok(typed_node("Local", fields))
    }

    fn generate_local_init(&self, init: &LocalInit) -> Result<SExp> {
        let mut fields = kwargs(vec![
            kwarg("expr", self.generate_expr(&init.expr)?),
        ]);

        // Optional else block
        if let Some(els) = &init.els {
            fields.extend(kwarg("els", self.generate_block(els)?));
        }

        Ok(typed_node("LocalInit", fields))
    }

    fn generate_mac_call_stmt(&self, stmt: &MacCallStmt) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("mac", self.generate_mac_call(&stmt.mac)?),
            kwarg("style", self.generate_mac_stmt_style(stmt.style)),
            kwarg("attrs", self.generate_attr_vec(&stmt.attrs)?),
        ]);

        // Optional tokens
        let fields = if stmt.tokens.is_some() {
            let mut f = fields;
            f.extend(kwarg("tokens", sym("nil")));
            f
        } else {
            fields
        };

        Ok(typed_node("MacCallStmt", fields))
    }

    fn generate_mac_stmt_style(&self, style: MacStmtStyle) -> SExp {
        match style {
            MacStmtStyle::Semicolon => sym("Semicolon"),
            MacStmtStyle::Braces => sym("Braces"),
            MacStmtStyle::NoBraces => sym("NoBraces"),
        }
    }
}

use crate::ast::*;
```

---

## Part 6: Module Exports

### File: `src/generator/mod.rs`

```rust
mod gen;
mod helpers;
mod item;
mod expr;
mod stmt;

pub use gen::Generator;
pub use helpers::*;
```

Update `src/lib.rs`:

```rust
pub mod error;
pub mod sexp;
pub mod ast;
pub mod builder;
pub mod generator;

pub use error::{ParseError, LexError, Position, Result};
pub use sexp::{SExp, Parser, Printer, print_sexp};
pub use ast::Crate;
pub use builder::AstBuilder;
pub use generator::Generator;
```

---

## Part 7: Generator Tests

### File: `tests/generator_tests.rs`

```rust
use oxur_ast::*;
use oxur_ast::ast::*;
use oxur_ast::sexp::print_sexp;

#[test]
fn test_generate_span() {
    let span = Span::new(0, 10);
    let gen = Generator::new();
    let sexp = gen.generate_span(span);

    let output = print_sexp(&sexp);
    assert!(output.contains("Span"));
    assert!(output.contains(":lo"));
    assert!(output.contains("0"));
    assert!(output.contains(":hi"));
    assert!(output.contains("10"));
}

#[test]
fn test_generate_ident() {
    let ident = Ident::new("main", Span::new(3, 7));
    let gen = Generator::new();
    let sexp = gen.generate_ident(&ident).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Ident"));
    assert!(output.contains(":name"));
    assert!(output.contains("main"));
}

#[test]
fn test_generate_path() {
    let ident = Ident::new("println", Span::new(17, 24));
    let segment = PathSegment::from_ident(ident);
    let path = Path::new(Span::new(17, 24), vec![segment]);

    let gen = Generator::new();
    let sexp = gen.generate_path(&path);

    let output = print_sexp(&sexp);
    assert!(output.contains("Path"));
    assert!(output.contains("PathSegment"));
    assert!(output.contains("println"));
}

#[test]
fn test_generate_empty_crate() {
    let crate_node = Crate::new(
        vec![],
        ModSpans::new(Span::new(0, 10), Span::new(0, 0)),
        NodeId::new(0),
    );

    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("Crate"));
    assert!(output.contains(":items"));
    assert!(output.contains(":spans"));
}

#[test]
fn test_generate_visibility() {
    let gen = Generator::new();

    let vis = Visibility::Public;
    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Public"));

    let vis = Visibility::Inherited;
    let sexp = gen.generate_visibility(&vis);
    let output = print_sexp(&sexp);
    assert!(output.contains("Inherited"));
}

#[test]
fn test_generate_mac_call() {
    let ident = Ident::new("println", Span::new(17, 24));
    let segment = PathSegment::from_ident(ident);
    let path = Path::new(Span::new(17, 24), vec![segment]);

    let args = MacArgs::Delimited {
        dspan: DelSpan::new(Span::new(24, 25), Span::new(42, 43)),
        delim: Delimiter::Paren,
        tokens: TokenStream::from_str("\"Hello, world!\""),
    };

    let mac_call = MacCall::new(path, args);

    let gen = Generator::new();
    let sexp = gen.generate_mac_call(&mac_call).unwrap();

    let output = print_sexp(&sexp);
    assert!(output.contains("MacCall"));
    assert!(output.contains("println"));
    assert!(output.contains("Delimited"));
    assert!(output.contains("Hello, world!"));
}
```

---

## Part 8: Round-Trip Tests

### File: `tests/round_trip_tests.rs`

The ultimate test - complete round-trip:

```rust
use oxur_ast::*;
use oxur_ast::sexp::{Parser, print_sexp};

#[test]
fn test_round_trip_span() {
    let original = "(Span :lo 0 :hi 10)";

    // Parse to SExp
    let sexp1 = Parser::parse_str(original).unwrap();

    // Build to AST
    let mut builder = AstBuilder::new();
    let span = builder.build_span(&sexp1).unwrap();

    // Generate back to SExp
    let gen = Generator::new();
    let sexp2 = gen.generate_span(span);

    // Parse again
    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    // Should be equivalent
    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_ident() {
    let original = r#"(Ident :name "main" :span (Span :lo 3 :hi 7))"#;

    let sexp1 = Parser::parse_str(original).unwrap();

    let mut builder = AstBuilder::new();
    let ident = builder.build_ident(&sexp1).unwrap();

    let gen = Generator::new();
    let sexp2 = gen.generate_ident(&ident).unwrap();

    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_path() {
    let original = r#"
(Path
  :span (Span :lo 17 :hi 24)
  :segments (
    (PathSegment
      :ident (Ident :name "println" :span (Span :lo 17 :hi 24))
      :id 0
      :args nil)))
    "#;

    let sexp1 = Parser::parse_str(original).unwrap();

    let mut builder = AstBuilder::new();
    let path = builder.build_path(&sexp1).unwrap();

    let gen = Generator::new();
    let sexp2 = gen.generate_path(&path);

    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_simple_crate() {
    let original = r#"
(Crate
  :attrs ()
  :items ()
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 10)
           :inject-use-span (Span :lo 0 :hi 0))
  :id 0
  :is-placeholder false)
    "#;

    let sexp1 = Parser::parse_str(original).unwrap();

    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp1).unwrap();

    let gen = Generator::new();
    let sexp2 = gen.generate_crate(&crate_node).unwrap();

    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    assert_eq!(sexp1, sexp3);
}

#[test]
fn test_round_trip_hello_world() {
    // Simplified Hello World for testing
    let original = r#"
(Crate
  :attrs ()
  :items (
    (Item
      :attrs ()
      :id 0
      :span (Span :lo 0 :hi 50)
      :vis (Inherited)
      :ident (Ident :name "main" :span (Span :lo 3 :hi 7))
      :kind (Fn
              :defaultness Final
              :sig (FnSig
                     :header (FnHeader
                               :safety Default
                               :coroutine-kind nil
                               :constness NotConst
                               :ext None)
                     :decl (FnDecl
                             :inputs ()
                             :output (Default (Span :lo 10 :hi 10)))
                     :span (Span :lo 0 :hi 10))
              :generics (Generics
                          :params ()
                          :where-clause (WhereClause
                                          :has-where-token false
                                          :predicates ()
                                          :span (Span :lo 10 :hi 10))
                          :span (Span :lo 7 :hi 10))
              :body (Block
                      :stmts (
                        (Stmt
                          :id 1
                          :kind (Semi
                                  (Expr
                                    :id 2
                                    :kind (MacCall
                                            (MacCall
                                              :path (Path
                                                      :span (Span :lo 17 :hi 24)
                                                      :segments (
                                                        (PathSegment
                                                          :ident (Ident
                                                                   :name "println"
                                                                   :span (Span :lo 17 :hi 24))
                                                          :id 0
                                                          :args nil)))
                                              :args (Delimited
                                                      :dspan (DelSpan
                                                               :open (Span :lo 24 :hi 25)
                                                               :close (Span :lo 42 :hi 43))
                                                      :delim Paren
                                                      :tokens (TokenStream
                                                                :source "\"Hello, world!\""))
                                              :prior-type-ascription nil))
                                    :span (Span :lo 17 :hi 44)
                                    :attrs ()))
                          :span (Span :lo 17 :hi 44)))
                      :id 3
                      :rules Default
                      :span (Span :lo 13 :hi 48)
                      :could-be-bare-literal false))))
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 50)
           :inject-use-span (Span :lo 0 :hi 0))
  :id 0
  :is-placeholder false)
    "#;

    // Parse original
    let sexp1 = Parser::parse_str(original).unwrap();

    // Build to AST
    let mut builder = AstBuilder::new();
    let crate_node = builder.build_crate(&sexp1).unwrap();

    // Verify basic structure
    assert_eq!(crate_node.items.len(), 1);
    assert_eq!(crate_node.items[0].ident.name, "main");

    // Generate back to SExp
    let gen = Generator::new();
    let sexp2 = gen.generate_crate(&crate_node).unwrap();

    // Parse again
    let printed = print_sexp(&sexp2);
    let sexp3 = Parser::parse_str(&printed).unwrap();

    // Should be equivalent
    assert_eq!(sexp1, sexp3);

    println!("\n✓ Round-trip successful!");
    println!("Original → AST → Generated → AST");
    println!("All structures preserved!");
}
```

---

## Part 9: Example

### File: `examples/generate_hello.rs`

```rust
use oxur_ast::*;
use oxur_ast::ast::*;
use oxur_ast::sexp::print_sexp;

fn main() {
    println!("Building Hello World AST manually...\n");

    // Build the AST by hand
    let println_ident = Ident::new("println", Span::new(17, 24));
    let println_segment = PathSegment::from_ident(println_ident);
    let println_path = Path::new(Span::new(17, 24), vec![println_segment]);

    let mac_args = MacArgs::Delimited {
        dspan: DelSpan::new(Span::new(24, 25), Span::new(42, 43)),
        delim: Delimiter::Paren,
        tokens: TokenStream::from_str("\"Hello, world!\""),
    };

    let mac_call = MacCall::new(println_path, mac_args);

    let expr = Expr {
        id: NodeId::new(2),
        kind: ExprKind::MacCall(Box::new(mac_call)),
        span: Span::new(17, 44),
        attrs: vec![],
        tokens: None,
    };

    let stmt = Stmt {
        id: NodeId::new(1),
        kind: StmtKind::Semi(Box::new(expr)),
        span: Span::new(17, 44),
    };

    let block = Block::new(
        vec![stmt],
        NodeId::new(3),
        Span::new(13, 48),
    );

    let fn_sig = FnSig {
        header: FnHeader::default(),
        decl: FnDecl {
            inputs: vec![],
            output: FnRetTy::Default(Span::new(10, 10)),
        },
        span: Span::new(0, 10),
    };

    let fn_item = Fn {
        defaultness: Defaultness::Final,
        sig: fn_sig,
        generics: Generics::empty(Span::new(7, 10)),
        body: Some(block),
    };

    let item = Item {
        attrs: vec![],
        id: NodeId::new(0),
        span: Span::new(0, 50),
        vis: Visibility::Inherited,
        ident: Ident::new("main", Span::new(3, 7)),
        kind: ItemKind::Fn(Box::new(fn_item)),
        tokens: None,
    };

    let crate_node = Crate::new(
        vec![item],
        ModSpans::new(Span::new(0, 50), Span::new(0, 0)),
        NodeId::new(0),
    );

    println!("✓ AST built successfully!\n");

    // Generate S-expression
    println!("Generating S-expression...\n");
    let gen = Generator::new();
    let sexp = gen.generate_crate(&crate_node).expect("Failed to generate S-expression");

    println!("✓ S-expression generated!\n");

    // Print it
    let output = print_sexp(&sexp);
    println!("Generated S-expression:\n");
    println!("{}", output);
}
```

---

## Part 10: Integration with Printer

The printer from Phase 0 should already work well, but we can add formatting hints:

### File: `src/sexp/printer.rs` (additions)

Add a method for compact printing:

```rust
impl Printer {
    /// Print in compact mode (minimal whitespace)
    pub fn print_compact(&mut self, sexp: &SExp) -> String {
        let mut output = String::new();
        self.print_sexp_compact(sexp, &mut output);
        output
    }

    fn print_sexp_compact(&mut self, sexp: &SExp, output: &mut String) {
        match sexp {
            SExp::List(l) => {
                write!(output, "(").unwrap();
                for (i, elem) in l.elements.iter().enumerate() {
                    if i > 0 {
                        write!(output, " ").unwrap();
                    }
                    self.print_sexp_compact(elem, output);
                }
                write!(output, ")").unwrap();
            }
            _ => self.print_sexp(sexp, output),
        }
    }
}

/// Convenience function for compact printing
pub fn print_sexp_compact(sexp: &SExp) -> String {
    Printer::new().print_compact(sexp)
}
```

---

## Success Criteria

Phase 2 is complete when:

- [ ] All generator methods implemented
- [ ] Generator produces valid S-expressions
- [ ] All generator tests pass
- [ ] Round-trip tests pass (S-expr → AST → S-expr → AST)
- [ ] Hello World example generates correctly
- [ ] Generated S-expressions can be parsed back
- [ ] No compiler warnings
- [ ] Clean `cargo clippy` output
- [ ] Code formatted with `cargo fmt`

---

## Testing Instructions

```bash
# Run all tests
cargo test -p oxur-ast

# Run generator tests specifically
cargo test -p oxur-ast --test generator_tests

# Run round-trip tests (the critical ones!)
cargo test -p oxur-ast --test round_trip_tests

# Run examples
cargo run -p oxur-ast --example generate_hello

# The big test - round-trip Hello World
cargo test -p oxur-ast round_trip_hello_world -- --nocapture

# Check
cargo fmt --check -p oxur-ast
cargo clippy -p oxur-ast -- -D warnings
```

---

## Verification Steps

After Phase 2 is complete, verify:

1. **Basic round-trip**:

   ```bash
   cargo test round_trip_span
   cargo test round_trip_ident
   cargo test round_trip_path
   ```

2. **Complex round-trip**:

   ```bash
   cargo test round_trip_hello_world -- --nocapture
   ```

   This should show: "Round-trip successful!"

3. **Manual verification**:

   ```bash
   cargo run -p oxur-ast --example generate_hello
   ```

   Check that the output S-expression looks correct

4. **Parse the generated output**:
   - Copy the generated S-expression
   - Feed it back through the builder
   - Verify you get the same AST

---

## Next Phase Preview

**Phase 3: Integration & Testing**

Once Phase 2 is complete, we'll have complete bidirectional conversion! Phase 3 will focus on:

- Integration with real Rust code (using `syn` or `rustc_ast`)
- Comprehensive test suite using rust-lang/rust test cases
- Performance optimization
- Documentation
- CLI tools for conversion
- REPL integration preparation

This completes the `oxur-ast` foundation!

---

## Debugging Tips

If round-trip tests fail:

1. **Check S-expression equality**:
   - Print both S-expressions
   - Use a diff tool
   - Look for missing fields or wrong types

2. **Verify AST structure**:
   - Add debug prints in builder
   - Check that all fields are populated
   - Ensure NodeIds are consistent

3. **Test incrementally**:
   - Start with simple types (Span, Ident)
   - Move to complex types (Path, MacCall)
   - Finally test complete structures (Crate)

4. **Use --nocapture**:

   ```bash
   cargo test test_name -- --nocapture
   ```

   This shows all println! output

---

*"The circle is complete: AST → S-expr → AST. Bidirectional harmony achieved."*
