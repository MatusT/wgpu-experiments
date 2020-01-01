use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::Rng;
use rayon::prelude::*;

pub fn sort_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort");
    for size in [1000, 10_000, 100_000, 1_000_000].iter() {
        let mut rng = rand::thread_rng();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut vec: Vec<f32> = (0..size).map(|_| rng.gen()).collect();
            b.iter(|| {
                vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
            });
        });
    }
    group.finish();
}

pub fn par_sort_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("par sort");
    for size in [1000, 10_000, 100_000, 1_000_000].iter() {
        let mut rng = rand::thread_rng();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut vec: Vec<f32> = (0..size).map(|_| rng.gen()).collect();
            b.iter(|| {
                vec.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            });
        });
    }
    group.finish();
}


criterion_group!(benches, sort_benchmark, par_sort_benchmark);
criterion_main!(benches);
