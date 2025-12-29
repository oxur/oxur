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
