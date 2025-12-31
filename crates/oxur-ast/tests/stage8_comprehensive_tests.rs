//! Comprehensive tests for Stage 8: Remaining Items

use oxur_ast::ast::*;
use oxur_ast::codegen::RustCodegen;
use oxur_ast::generator::Generator;

fn simple_path(name: &str) -> Path {
    Path {
        span: Span::DUMMY,
        segments: vec![PathSegment {
            ident: Ident::new(name, Span::DUMMY),
            id: NodeId(0),
            args: None,
        }],
        tokens: None,
    }
}

fn make_crate(items: Vec<Item>) -> Crate {
    Crate {
        attrs: Vec::new(),
        items,
        spans: ModSpans { inner_span: Span::DUMMY, inject_use_span: Span::DUMMY },
        id: NodeId(0),
        is_placeholder: false,
    }
}

#[test]
fn test_codegen_use_simple() {
    let use_tree = UseTree { prefix: simple_path("std::io"), kind: UseTreeKind::Simple(None) };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("", Span::DUMMY),
        kind: ItemKind::Use(use_tree),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("use std::io;"));
}

#[test]
fn test_codegen_use_with_rename() {
    let use_tree = UseTree {
        prefix: simple_path("std::io"),
        kind: UseTreeKind::Simple(Some(Ident::new("stdio", Span::DUMMY))),
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("", Span::DUMMY),
        kind: ItemKind::Use(use_tree),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("use std::io as stdio;"));
}

#[test]
fn test_codegen_use_glob() {
    let use_tree = UseTree { prefix: simple_path("std::io"), kind: UseTreeKind::Glob };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("", Span::DUMMY),
        kind: ItemKind::Use(use_tree),
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("use std::io::*;"));
}

#[test]
fn test_codegen_static_immutable() {
    let ty = Ty {
        id: NodeId(0),
        kind: TyKind::Path(None, simple_path("i32")),
        span: Span::DUMMY,
        tokens: None,
    };
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(42), span: Span::DUMMY }),
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("ANSWER", Span::DUMMY),
        kind: ItemKind::Static { mutability: Mutability::Not, ty, expr: Some(expr) },
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("static ANSWER: i32 = 42;"));
}

#[test]
fn test_codegen_static_mutable() {
    let ty = Ty {
        id: NodeId(0),
        kind: TyKind::Path(None, simple_path("i32")),
        span: Span::DUMMY,
        tokens: None,
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Public,
        ident: Ident::new("COUNTER", Span::DUMMY),
        kind: ItemKind::Static { mutability: Mutability::Mut, ty, expr: None },
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("pub static mut COUNTER: i32;"));
}

#[test]
fn test_codegen_const() {
    let ty = Ty {
        id: NodeId(0),
        kind: TyKind::Path(None, simple_path("usize")),
        span: Span::DUMMY,
        tokens: None,
    };
    let expr = Expr {
        id: NodeId(0),
        kind: ExprKind::Lit(Lit { kind: LitKind::Int(100), span: Span::DUMMY }),
        span: Span::DUMMY,
        attrs: Vec::new(),
        tokens: None,
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Public,
        ident: Ident::new("MAX_SIZE", Span::DUMMY),
        kind: ItemKind::Const { ty, expr: Some(expr) },
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("pub const MAX_SIZE: usize = 100;"));
}

#[test]
fn test_codegen_type_alias() {
    let ty = Ty {
        id: NodeId(0),
        kind: TyKind::Path(None, simple_path("String")),
        span: Span::DUMMY,
        tokens: None,
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Public,
        ident: Ident::new("MyString", Span::DUMMY),
        kind: ItemKind::TyAlias { generics: Generics::empty(), ty: Some(ty) },
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("pub type MyString = String;"));
}

#[test]
fn test_codegen_mod_external() {
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Public,
        ident: Ident::new("utils", Span::DUMMY),
        kind: ItemKind::Mod { items: None },
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("pub mod utils;"));
}

#[test]
fn test_codegen_mod_inline_empty() {
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("inner", Span::DUMMY),
        kind: ItemKind::Mod { items: Some(Vec::new()) },
        tokens: None,
    };
    let mut codegen = RustCodegen::new();
    let output = codegen.generate_crate(&make_crate(vec![item])).unwrap();
    assert!(output.contains("mod inner {"));
    assert!(output.contains("}"));
}

#[test]
fn test_generator_use_simple() {
    let use_tree = UseTree { prefix: simple_path("std::io"), kind: UseTreeKind::Simple(None) };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("", Span::DUMMY),
        kind: ItemKind::Use(use_tree),
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_item(&item);
    assert!(result.is_ok());
}

#[test]
fn test_generator_static() {
    let ty = Ty {
        id: NodeId(0),
        kind: TyKind::Path(None, simple_path("i32")),
        span: Span::DUMMY,
        tokens: None,
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("X", Span::DUMMY),
        kind: ItemKind::Static { mutability: Mutability::Not, ty, expr: None },
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_item(&item);
    assert!(result.is_ok());
}

#[test]
fn test_generator_const() {
    let ty = Ty {
        id: NodeId(0),
        kind: TyKind::Path(None, simple_path("usize")),
        span: Span::DUMMY,
        tokens: None,
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("Y", Span::DUMMY),
        kind: ItemKind::Const { ty, expr: None },
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_item(&item);
    assert!(result.is_ok());
}

#[test]
fn test_generator_type_alias() {
    let ty = Ty {
        id: NodeId(0),
        kind: TyKind::Path(None, simple_path("String")),
        span: Span::DUMMY,
        tokens: None,
    };
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("MyType", Span::DUMMY),
        kind: ItemKind::TyAlias { generics: Generics::empty(), ty: Some(ty) },
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_item(&item);
    assert!(result.is_ok());
}

#[test]
fn test_generator_mod() {
    let item = Item {
        attrs: Vec::new(),
        id: NodeId(0),
        span: Span::DUMMY,
        vis: Visibility::Inherited,
        ident: Ident::new("my_mod", Span::DUMMY),
        kind: ItemKind::Mod { items: None },
        tokens: None,
    };
    let generator = Generator::new();
    let result = generator.generate_item(&item);
    assert!(result.is_ok());
}
