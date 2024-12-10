use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{mpsc, Arc};

use itertools::iproduct;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use tokiobench::params as p;
use tokiobench::rt;
use tokiobench::work as w;

fn nspawn() -> Vec<usize> {
    const BOUND: usize = 10;
    const MULTIPLYER: usize = 1000;

    (1..BOUND + 1).map(|i| i * MULTIPLYER).collect()
}

fn nspawner() -> Vec<usize> {
    const BOUND: usize = 20;

    (1..BOUND + 1).collect()
}

fn workload(name: &str, work: w::Work, c: &mut Criterion) {
    let (tx, rx) = mpsc::sync_channel(1);
    let rem: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let mut group = c.benchmark_group(format!("aworkload/{name}"));

    for (nspawn, nspawner) in iproduct!(nspawn(), nspawner()) {
        let rt = rt::new(p::N_WORKERS);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_with_input(
            format!("nspawn({nspawn})/nspawner({nspawner})"),
            &nspawn,
            |b, &nspawn| {
                b.iter(|| {
                    let tx = tx.clone();
                    let rem = rem.clone();
                    rem.store(nspawn * nspawner, Relaxed);

                    rt.block_on(async {
                        for _ in 0..nspawner {
                            let rem = rem.clone();
                            let tx = tx.clone();

                            tokio::spawn(async move {
                                for _ in 0..nspawn {
                                    let rem = rem.clone();
                                    let tx = tx.clone();

                                    tokio::spawn(async move {
                                        work();

                                        if 1 == rem.fetch_sub(1, Relaxed) {
                                            tx.send(()).unwrap();
                                        }
                                    });
                                }
                            });
                        }

                        rx.recv().unwrap();
                    })
                });
            },
        );
    }

    group.finish();
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
