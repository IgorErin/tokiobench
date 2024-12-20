use itertools::iproduct;

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tokiobench::rt;

use futures::prelude::*;

const NUM_THREADS: usize = 12;

async fn task() {
    std::hint::black_box(());
}

async fn spawn_tasks(n: usize) {
    // assume compiler reduce allocation TODO()
    future::join_all((0..n).into_iter().map(|_| tokio::spawn(task()))).await;
}

async fn spawn_spawners(nspawner: usize, nspawn: usize) {
    // assume compiler reduce allocation TODO()
    future::join_all(
        (0..nspawner)
            .into_iter()
            .map(|_| tokio::spawn(spawn_tasks(nspawn))),
    )
    .await;
}

fn bench(name: &str, nspawn: &[usize], nspawner: &[usize], c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("tatlin/{name}"));

    for (&nspawn, &nspawner) in iproduct!(nspawn, nspawner) {
        let rt = rt::new(NUM_THREADS);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(format!("nspawn({nspawn})/nspawner({nspawner})"), |b| {
            b.iter(|| {
                rt.block_on(async {
                    tokio::spawn(spawn_spawners(nspawner, nspawn))
                        .await
                        .unwrap();
                });
            });
        });
    }
    group.finish();
}

fn bench_tatlin(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..=10).map(|i| i * 1000).collect();
    let nspawner: Vec<usize> = (1..=20).collect();

    bench("thousand", nspawn.as_ref(), nspawner.as_ref(), c)
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));

    targets = bench_tatlin
);

criterion_main!(benches);
