use cfg_if::cfg_if;
use itertools::iproduct;

use std::sync::mpsc;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tokiobench::rt;

fn bench(name: &str, nspawn: &[usize], nworker: &[usize], c: &mut Criterion) {
    let (tx, rx) = mpsc::sync_channel(1);
    let mut group = c.benchmark_group(format!("remote/{name}"));

    for (&nspawn, &nworker) in iproduct!(nspawn, nworker) {
        let rt = rt::new(nworker);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(format!("nspawn({nspawn})/nworker({nworker})"), |b| {
            let handles = Vec::with_capacity(nspawn);
            b.iter_reuse(handles, |mut handles| {
                cfg_if!(if #[cfg(feature = "check")] {
                    assert!(handles.is_empty());
                    assert!(handles.capacity() == nspawn);
                });

                let tx = tx.clone();
                let _guard = rt.enter();

                for _ in 0..nspawn {
                    handles.push(tokio::spawn(async { std::hint::black_box(()) }));
                }

                tokio::spawn(async move {
                    for handle in handles.drain(..) {
                        handle.await.unwrap();
                    }

                    tx.send(handles).unwrap();
                });

                rx.recv().unwrap()
            });
        });
    }
    group.finish();
}

fn bench_thousand(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..=10).map(|i| i * 1000).collect();
    let nworker: Vec<usize> = (2..=20).collect();

    bench("thousand", nspawn.as_ref(), nworker.as_ref(), c)
}

fn bench_hundred(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..=10).map(|i| i * 100).collect();
    let nworker: Vec<usize> = (2..=20).collect();

    bench("hundred", nspawn.as_ref(), nworker.as_ref(), c)
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::from_secs(100))
        .warm_up_time(Duration::from_secs(10));

    targets = bench_hundred, bench_thousand
);

criterion_main!(benches);
