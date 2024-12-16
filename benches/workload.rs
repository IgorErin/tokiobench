use cfg_if::cfg_if;
use itertools::iproduct;

use std::sync::mpsc;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use tokiobench::params as p;
use tokiobench::rt;

fn bench(name: &str, nspawn: &[usize], nspawner: &[usize], c: &mut Criterion) {
    let (tx, rx) = mpsc::sync_channel(1);
    let mut group = c.benchmark_group(format!("workload/{name}"));

    for (&nspawn, &nspawner) in iproduct!(nspawn, nspawner) {
        let rt = rt::new(p::N_WORKERS);

        group.throughput(Throughput::Elements((nspawner * nspawn) as u64));
        group.bench_function(format!("nspawn({nspawn})/nspawner({nspawner})"), |b| {
            let leaf_handles = (0..nspawner)
                .map(|_| Vec::with_capacity(nspawn))
                .collect::<Vec<_>>();
            let root_handles = Vec::with_capacity(nspawner);

            b.iter_reuse(
                (leaf_handles, root_handles),
                |(mut leaf_handles, mut root_handles)| {
                    cfg_if!(if #[cfg(feature = "check")] {
                        assert!(root_handles.is_empty());
                        assert!(root_handles.capacity() == nspawner);

                        assert!(leaf_handles.iter().all(|i| i.is_empty()));
                        assert!(leaf_handles.iter().all(|i| i.capacity() == nspawn));
                    });

                    let tx = tx.clone();
                    let _gurad = rt.enter();

                    for mut leaf_handle in leaf_handles.drain(..) {
                        root_handles.push(tokio::spawn(async move {
                            for _ in 0..leaf_handle.capacity() {
                                leaf_handle.push(tokio::spawn(async {
                                    std::hint::black_box(());
                                }));
                            }

                            leaf_handle
                        }));
                    }

                    tokio::spawn(async move {
                        for leaf_handle in root_handles.drain(..) {
                            let mut leaf_handle = leaf_handle.await.unwrap();

                            for handle in leaf_handle.drain(..) {
                                handle.await.unwrap();
                            }

                            leaf_handles.push(leaf_handle);
                        }

                        tx.send((leaf_handles, root_handles)).unwrap();
                    });

                    rx.recv().unwrap()
                },
            );
        });
    }
    group.finish();
}

fn bench_thousand(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..10 + 1).map(|i| i * 1000).collect();
    let nspawner: Vec<usize> = (1..20 + 1).collect();

    bench("thousand", nspawn.as_ref(), nspawner.as_ref(), c)
}

fn bench_hundred(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..10 + 1).map(|i| i * 100).collect();
    let nspawner: Vec<usize> = (1..20 + 1).collect();

    bench("hundred", nspawn.as_ref(), nspawner.as_ref(), c)
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::from_secs(60))
        .warm_up_time(Duration::from_secs(60));

    targets = bench_hundred, bench_thousand
);

criterion_main!(benches);
