use criterion::{criterion_group, criterion_main, Criterion};

use tokiobench::work as w;

fn workload(name: &str, work: w::Work, c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("group:{name}"));

    group.bench_function(format!("{name}"), |b| {
        b.iter(work);
    });
}

fn bench_float_fst(c: &mut Criterion) {
    workload("float_fst", w::float_fst, c)
}

fn bench_float_snd(c: &mut Criterion) {
    workload("float_snd", w::float_snd, c)
}

fn bench_float_thd(c: &mut Criterion) {
    workload("float_thd", w::float_thd, c)
}

fn bench_float_fth(c: &mut Criterion) {
    workload("float_fth", w::float_fth, c)
}

fn bench_float_fft(c: &mut Criterion) {
    workload("float_fft", w::float_fft, c)
}

criterion_group!(
    benches,
    bench_float_fst,
    bench_float_snd,
    bench_float_thd,
    bench_float_fth,
    bench_float_fft,
);

criterion_main!(benches);
