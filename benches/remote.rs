use cfg_if::cfg_if;
use futures::future;
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
            b.iter(|| {
                cfg_if!(if #[cfg(feature = "check")] {
                    assert!(handles.is_empty());
                    assert!(handles.capacity() == nspawn);
                });

                let tx = tx.clone();
                let _guard = rt.enter();

                let handles = (0..nspawn)
                    .into_iter()
                    .map(|_| tokio::spawn(async { std::hint::black_box(()) }));

                tokio::spawn(async move {
                    future::join_all(handles).await;
                    tx.send(()).unwrap();
                });

                rx.recv().unwrap()
            });
        });
    }
    group.finish();
}

fn bench_fst(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..=10).map(|i| i * 1000_000).collect();
    let nworker: Vec<usize> = (2..=20).collect();

    bench("thousand", nspawn.as_ref(), nworker.as_ref(), c)
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(10));

    targets = bench_fst
);

criterion_main!(benches);
