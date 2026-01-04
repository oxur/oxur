//! Performance benchmarks for the formatter.

use criterion::{criterion_group, criterion_main, Criterion};
use oxur_pretty::format_sexp;

fn bench_simple_formatting(c: &mut Criterion) {
    c.bench_function("format simple sexp", |b| {
        b.iter(|| format_sexp("(a b c)"))
    });
}

fn bench_compact_struct(c: &mut Criterion) {
    c.bench_function("format compact struct", |b| {
        b.iter(|| format_sexp("(Span :lo 0 :hi 10)"))
    });
}

fn bench_nested_formatting(c: &mut Criterion) {
    c.bench_function("format nested struct", |b| {
        b.iter(|| format_sexp(r#"(Ident :name "use" :span (Span :lo 0 :hi 3))"#))
    });
}

criterion_group!(benches, bench_simple_formatting, bench_compact_struct, bench_nested_formatting);
criterion_main!(benches);
