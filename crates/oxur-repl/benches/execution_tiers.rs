//! Benchmarks for REPL Three-Tier Execution
//!
//! Measures performance of the three execution strategies described in ODD-0038 §2.3:
//!
//! 1. **Calculator Tier** - Direct evaluation of simple arithmetic (target: <1ms)
//! 2. **CachedLoaded Tier** - Pre-compiled cached library execution (target: 1-5ms)
//! 3. **JustInTime Tier** - Full compile-execute cycle (target: 50-300ms)
//!
//! Run with: `cargo bench --package oxur-repl`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use oxur_repl::eval::{EvalContext, LispEvaluator};
use oxur_repl::protocol::{ReplMode, SessionId};

/// Benchmark Calculator Tier: Direct arithmetic evaluation
///
/// Target: <1ms
///
/// Tests simple arithmetic expressions that can be evaluated directly
/// without compilation.
fn bench_calculator_tier(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculator_tier");

    // Simple arithmetic expressions
    let expressions = vec![
        ("simple_add", "(+ 1 2)"),
        ("simple_mult", "(* 3 4)"),
        ("nested_add", "(+ 1 (+ 2 3))"),
        ("complex_expr", "(+ (* 2 3) (- 10 4))"),
        ("deep_nesting", "(+ 1 (+ 2 (+ 3 (+ 4 5))))"),
    ];

    for (name, expr) in expressions {
        group.bench_with_input(BenchmarkId::from_parameter(name), &expr, |b, &expr| {
            let mut calculator = LispEvaluator::new();
            b.iter(|| {
                let result = calculator.try_eval_calculator(black_box(expr));
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark CachedLoaded Tier: Pre-compiled library execution
///
/// Target: 1-5ms
///
/// Tests execution of code that's already been compiled and cached.
/// This measures library loading + execution overhead.
///
/// NOTE: This benchmark is currently a placeholder. Full Tier 2/3 execution
/// requires the complete compilation pipeline which is still in development.
fn bench_cached_tier(c: &mut Criterion) {
    let mut group = c.benchmark_group("cached_tier");

    // TODO: Implement once EvalContext has async evaluate() method
    // For now, we benchmark context creation as a placeholder
    group.bench_function("context_creation", |b| {
        b.iter(|| {
            let ctx = EvalContext::new(SessionId::new("bench-session"), ReplMode::Lisp);
            black_box(ctx)
        });
    });

    group.finish();
}

/// Benchmark JustInTime Tier: Full compilation cycle
///
/// Target: 50-300ms
///
/// Tests the full compilation pipeline: parsing → wrapping → compiling → executing
/// This is the most realistic "first time" evaluation scenario.
///
/// NOTE: This benchmark is currently a placeholder. Full Tier 3 execution
/// requires the complete compilation pipeline which is still in development.
fn bench_jit_tier(c: &mut Criterion) {
    let mut group = c.benchmark_group("jit_tier");
    group.sample_size(10); // Reduce sample size for slow benchmarks

    // TODO: Implement once EvalContext has async evaluate() and clear_cache() methods
    // For now, we benchmark context creation as a placeholder
    group.bench_function("context_creation", |b| {
        b.iter(|| {
            let ctx = EvalContext::new(SessionId::new("bench-jit-session"), ReplMode::Lisp);
            black_box(ctx)
        });
    });

    group.finish();
}

/// Benchmark comparison across all tiers
///
/// Runs the same simple expression through all three tiers to show
/// the performance difference.
///
/// NOTE: Currently only benchmarks Tier 1 (Calculator). Tier 2/3 benchmarks
/// are placeholders pending completion of the async evaluation pipeline.
fn bench_tier_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("tier_comparison");

    // Calculator tier (Tier 1) - IMPLEMENTED
    group.bench_function("calculator", |b| {
        let mut calculator = LispEvaluator::new();
        b.iter(|| {
            let result = calculator.try_eval_calculator(black_box("(+ 2 2)"));
            black_box(result)
        });
    });

    // Cached tier (Tier 2) - TODO: Implement when async evaluate() is ready
    group.bench_function("context_for_cached", |b| {
        b.iter(|| {
            let ctx = EvalContext::new(SessionId::new("comparison-cached"), ReplMode::Lisp);
            black_box(ctx)
        });
    });

    // JIT tier (Tier 3) - TODO: Implement when async evaluate() is ready
    group.bench_function("context_for_jit", |b| {
        b.iter(|| {
            let ctx = EvalContext::new(SessionId::new("comparison-jit"), ReplMode::Lisp);
            black_box(ctx)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_calculator_tier,
    bench_cached_tier,
    bench_jit_tier,
    bench_tier_comparison
);
criterion_main!(benches);
