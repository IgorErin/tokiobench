use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::mpsc;
use std::sync::Arc;
use std::{sync::mpsc::SyncSender, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use futures::prelude::*;
use itertools::iproduct;

use tokiobench::rt;

const NUM_THREADS: usize = 12;

async fn task() {
    std::hint::black_box(());
}

async fn spawn_tasks(n: usize) {
    // assume compiler reduce allocation TODO()
    future::join_all((0..n).into_iter().map(|_| tokio::spawn(task()))).await;
}

async fn spawn_spawners_tx(nspawner: usize, nspawn: usize, tx: SyncSender<()>) {
    // assume compiler reduce allocation TODO()
    future::join_all(
        (0..nspawner)
            .into_iter()
            .map(|_| tokio::spawn(spawn_tasks(nspawn))),
    )
    .await;

    tx.send(()).unwrap();
}

fn ch(name: &str, nspawn: &[usize], nspawner: &[usize], c: &mut Criterion) {
    let (tx, rx) = mpsc::sync_channel(1);
    let mut group = c.benchmark_group(format!("tatlin/{name}"));

    for (&nspawn, &nspawner) in iproduct!(nspawn, nspawner) {
        let rt = rt::new(NUM_THREADS);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(format!("nspawn({nspawn})/nspawner({nspawner})"), |b| {
            b.iter(|| {
                let tx = tx.clone();
                rt.block_on(async {
                    tokio::spawn(spawn_spawners_tx(nspawner, nspawn, tx));

                    rx.recv().unwrap();
                });
            });
        });
    }
    group.finish();
}

async fn spawn_spawners_spin(nspawner: usize, nspawn: usize, bit: Arc<AtomicBool>) {
    // assume compiler reduce allocation TODO()
    future::join_all(
        (0..nspawner)
            .into_iter()
            .map(|_| tokio::spawn(spawn_tasks(nspawn))),
    )
    .await;

    bit.store(true, Release);
}

fn spin(name: &str, nspawn: &[usize], nspawner: &[usize], c: &mut Criterion) {
    let end = Arc::new(AtomicBool::new(false));
    let mut group = c.benchmark_group(format!("tatlin/{name}"));

    for (&nspawn, &nspawner) in iproduct!(nspawn, nspawner) {
        let rt = rt::new(NUM_THREADS);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(format!("nspawn({nspawn})/nspawner({nspawner})"), |b| {
            b.iter(|| {
                end.store(false, Release);
                let task_end = Arc::clone(&end);

                let _guard = rt.enter();
                tokio::spawn(spawn_spawners_spin(nspawner, nspawn, task_end));

                while !end.load(Acquire) {
                    std::hint::spin_loop();
                }
            });
        });
    }
    group.finish();
}

fn bench_ch(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..=10).map(|i| i * 1000).collect();
    let nspawner: Vec<usize> = (1..=20).collect();

    ch("ch", nspawn.as_ref(), nspawner.as_ref(), c)
}

fn bench_spin(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..=10).map(|i| i * 1000).collect();
    let nspawner: Vec<usize> = (1..=20).collect();

    spin("spin", nspawn.as_ref(), nspawner.as_ref(), c)
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));

    targets = bench_ch, bench_spin
);

criterion_main!(benches);
