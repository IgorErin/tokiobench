use itertools::iproduct;

use cfg_if::cfg_if;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use tokiobench::params;
use tokiobench::rt;

fn current(name: &str, c: &mut Criterion) {
    let mut group = c.benchmark_group(name);

    for (nspawn, nworkers) in iproduct!(params::NS_SPAWN_LOCAL, params::NS_WORKERS) {
        let rt = rt::new(nworkers);
        let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::with_capacity(nspawn);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(
            format!("nspawn({nspawn})/nwork({nworkers})"),
            |b| {
                b.iter(|| {
                    cfg_if!(if #[cfg(feature = "check")] {
                        assert!(handles.is_empty());
                        assert_eq!(handles.capacity(), nspawn);
                    });

                    rt.block_on(async {
                        for _ in 0..handles.capacity() {
                            handles.push(tokio::spawn(async move {
                                // TODO(work)
                            }));
                        }

                        for handle in handles.drain(..) {
                            handle.await.unwrap();
                        }
                    });
                });
            },
        );
    }
}

fn local(name: &str, c: &mut Criterion) {
    let mut group = c.benchmark_group(name);

    for (nspawn, nworkers) in iproduct!(params::NS_SPAWN_LOCAL, params::NS_WORKERS) {
        let rt = rt::new(nworkers);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_with_input(
            format!("nspawn({nspawn})/nwork({nworkers})"),
            &nspawn,
            |b, &_| {
                b.iter_reuse(Vec::with_capacity(nspawn), |mut handles| {
                    cfg_if!(if #[cfg(feature = "check")] {
                        assert!(handles.is_empty());
                        assert_eq!(handles.capacity(), nspawn);
                    });

                    rt.block_on(async {
                        let mut handles = tokio::spawn(async move {
                            for _ in 0..handles.capacity() {
                                handles.push(tokio::spawn(async {
                                    // TODO(work)
                                }));
                            }
                            handles
                        })
                        .await
                        .unwrap();

                        for handle in handles.drain(..) {
                            handle.await.unwrap();
                        }
                        handles
                    })
                });
            },
        );
    }
}

fn bench_current(c: &mut Criterion) {
    current("spawn_current", c)
}

fn bench_local(c: &mut Criterion) {
    local("spawn_local", c)
}

criterion_group!(benches, bench_current, bench_local,);

criterion_main!(benches);
