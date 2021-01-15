use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use space_filler::{hilbert, morton, CurveIdx};

pub fn morton_benchmark(c: &mut Criterion) {
    c.bench_function("morton min", |b| {
        b.iter(|| morton::decode_2d(black_box(CurveIdx::MIN)))
    });
    c.bench_function("morton max", |b| {
        b.iter(|| morton::decode_2d(black_box(CurveIdx::MAX)))
    });

    let mut group = c.benchmark_group("morton iter");
    group.throughput(Throughput::Elements(
        CurveIdx::MAX as u64 - CurveIdx::MIN as u64 + 1,
    ));
    group.bench_function("naive", |b| {
        b.iter(|| {
            for i in CurveIdx::MIN..=CurveIdx::MAX {
                black_box(morton::decode_2d(i));
            }
        })
    });
    group.bench_function("optimized", |b| {
        b.iter(|| {
            for coords in morton::iter_2d() {
                black_box(coords);
            }
        })
    });
}

pub fn hilbert_benchmark(c: &mut Criterion) {
    c.bench_function("hilbert min", |b| {
        b.iter(|| hilbert::decode_2d(black_box(CurveIdx::MIN)))
    });
    c.bench_function("hilbert max", |b| {
        b.iter(|| hilbert::decode_2d(black_box(CurveIdx::MAX)))
    });

    let mut group = c.benchmark_group("hilbert iter");
    group.throughput(Throughput::Elements(
        CurveIdx::MAX as u64 - CurveIdx::MIN as u64 + 1,
    ));
    group.bench_function("naive", |b| {
        b.iter(|| {
            for i in CurveIdx::MIN..=CurveIdx::MAX {
                black_box(hilbert::decode_2d(i));
            }
        })
    });
    // TODO: Add optimized Hilbert curve iterator
}

criterion_group!(benches, morton_benchmark, hilbert_benchmark);
criterion_main!(benches);
