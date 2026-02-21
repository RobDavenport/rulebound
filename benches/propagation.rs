use criterion::{criterion_group, criterion_main, Criterion};

fn propagation_benchmark(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {
            // TODO: benchmark propagation on realistic constraint problems
            42
        })
    });
}

criterion_group!(benches, propagation_benchmark);
criterion_main!(benches);
