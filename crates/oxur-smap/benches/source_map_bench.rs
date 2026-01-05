//! Benchmarks for oxur-smap
//!
//! Run with: cargo bench --package oxur-smap

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use oxur_smap::{new_node_id, SourceMap, SourcePos};

/// Benchmark NodeId generation
fn bench_node_id_generation(c: &mut Criterion) {
    c.bench_function("node_id_generation", |b| {
        b.iter(|| {
            black_box(new_node_id());
        });
    });
}

/// Benchmark recording a single surface node
fn bench_record_surface_node(c: &mut Criterion) {
    c.bench_function("record_surface_node", |b| {
        b.iter_batched(
            || {
                let map = SourceMap::new();
                let node = new_node_id();
                let pos = SourcePos::repl(1, 10, 5);
                (map, node, pos)
            },
            |(mut map, node, pos)| {
                map.record_surface_node(black_box(node), black_box(pos));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark recording an expansion
fn bench_record_expansion(c: &mut Criterion) {
    c.bench_function("record_expansion", |b| {
        b.iter_batched(
            || {
                let mut map = SourceMap::new();
                let surface = new_node_id();
                let core = new_node_id();
                let pos = SourcePos::repl(1, 10, 5);
                map.record_surface_node(surface, pos);
                (map, surface, core)
            },
            |(mut map, surface, core)| {
                map.record_expansion(black_box(surface), black_box(core));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark recording a lowering
fn bench_record_lowering(c: &mut Criterion) {
    c.bench_function("record_lowering", |b| {
        b.iter_batched(
            || {
                let mut map = SourceMap::new();
                let surface = new_node_id();
                let core = new_node_id();
                let rust = new_node_id();
                let pos = SourcePos::repl(1, 10, 5);
                map.record_surface_node(surface, pos);
                map.record_expansion(surface, core);
                (map, core, rust)
            },
            |(mut map, core, rust)| {
                map.record_lowering(black_box(core), black_box(rust));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark lookup operation (full chain)
fn bench_lookup_full_chain(c: &mut Criterion) {
    c.bench_function("lookup_full_chain", |b| {
        b.iter_batched(
            || {
                let mut map = SourceMap::new();
                let surface = new_node_id();
                let core = new_node_id();
                let rust = new_node_id();
                let pos = SourcePos::repl(1, 10, 5);
                map.record_surface_node(surface, pos);
                map.record_expansion(surface, core);
                map.record_lowering(core, rust);
                (map, rust)
            },
            |(map, rust)| {
                black_box(map.lookup(&rust));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark stats calculation with varying chain counts
fn bench_lookup_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_stats");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut map = SourceMap::new();
                    // Create 'size' complete chains
                    for _ in 0..size {
                        let surface = new_node_id();
                        let core = new_node_id();
                        let rust = new_node_id();
                        let pos = SourcePos::repl(1, 10, 5);
                        map.record_surface_node(surface, pos);
                        map.record_expansion(surface, core);
                        map.record_lowering(core, rust);
                    }
                    map
                },
                |map| {
                    black_box(map.lookup_stats());
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark building a large source map (simulates real compilation)
fn bench_build_large_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_source_map");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut map = SourceMap::new();
                // Simulate compilation pipeline
                for i in 0..size {
                    let surface = new_node_id();
                    let core = new_node_id();
                    let rust = new_node_id();
                    let pos = SourcePos::repl(i + 1, (i + 1) * 10, 5);
                    map.record_surface_node(surface, pos);
                    map.record_expansion(surface, core);
                    map.record_lowering(core, rust);
                }
                black_box(map);
            });
        });
    }
    group.finish();
}

/// Benchmark freeze operation
fn bench_freeze(c: &mut Criterion) {
    c.bench_function("freeze", |b| {
        b.iter_batched(
            || {
                let mut map = SourceMap::new();
                // Add some data
                for _ in 0..100 {
                    let surface = new_node_id();
                    let pos = SourcePos::repl(1, 10, 5);
                    map.record_surface_node(surface, pos);
                }
                map
            },
            |mut map| {
                map.freeze();
                black_box(map);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark content hash calculation (used for caching)
fn bench_content_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_hash");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut map = SourceMap::new();
                    for i in 0..size {
                        let surface = new_node_id();
                        let core = new_node_id();
                        let rust = new_node_id();
                        let pos = SourcePos::repl(i + 1, (i + 1) * 10, 5);
                        map.record_surface_node(surface, pos);
                        map.record_expansion(surface, core);
                        map.record_lowering(core, rust);
                    }
                    map
                },
                |map| {
                    black_box(map.content_hash());
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_node_id_generation,
    bench_record_surface_node,
    bench_record_expansion,
    bench_record_lowering,
    bench_lookup_full_chain,
    bench_lookup_stats,
    bench_build_large_map,
    bench_freeze,
    bench_content_hash
);
criterion_main!(benches);
