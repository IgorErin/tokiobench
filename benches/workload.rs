use cfg_if::cfg_if;
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

const YIELD_COUNT: usize = 10;

fn workload(name: &str, work: w::Work, c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("workload/{name}"));
    for (nspawn, nspawner) in iproduct!(nspawn(), nspawner()) {
        let rt = rt::new(p::N_WORKERS);

        group.throughput(Throughput::Elements((nspawner * nspawn) as u64));
        group.bench_function(format!("nspawn({nspawn})/nspawner({nspawner})"), |b| {
            // lift this TODO()
            let leaf_handles = (0..nspawner)
                .map(|_| Vec::with_capacity(nspawn))
                .collect::<Vec<_>>();
            b.iter_reuse(
                (Vec::with_capacity(nspawner), leaf_handles),
                |(mut root_handles, mut leaf_handles)| {
                    cfg_if!(if #[cfg(feature = "check")] {
                        assert!(root_handles.is_empty());
                        assert!(root_handles.capacity() == nspawner);

                        assert!(leaf_handles.iter().all(|i| i.is_empty()));
                        assert!(leaf_handles.iter().all(|i| i.capacity() == nspawn));
                    });

                    rt.block_on(async {
                        for mut leaf_handle in leaf_handles.drain(..) {
                            root_handles.push(tokio::spawn(async move {
                                for _ in 0..leaf_handle.capacity() {
                                    leaf_handle.push(tokio::spawn(async move {
                                        for _ in 0..YIELD_COUNT {
                                            cfg_if!(if #[cfg(feature = "yield")] {
                                                tokio::task::yield_now().await;
                                            });
                                            std::hint::black_box(work());
                                        }
                                    }));
                                }

                                leaf_handle
                            }));
                        }

                        for leaf_handle in root_handles.drain(..) {
                            let mut leaf_handle = leaf_handle.await.unwrap();

                            for handle in leaf_handle.drain(..) {
                                handle.await.unwrap();
                            }

                            leaf_handles.push(leaf_handle);
                        }
                    });

                    (root_handles, leaf_handles)
                },
            );
        });
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
