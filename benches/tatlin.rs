use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use itertools::iproduct;
use tokiobench::tatlin;

use tokiobench::rt;

const NUM_THREADS: usize = 12;

fn ch(name: &str, nspawn: &[usize], nspawner: &[usize], c: &mut Criterion) {
    let (tx, rx) = mpsc::sync_channel(1);
    let mut group = c.benchmark_group(format!("tatlin/{name}"));

    for (&nspawn, &nspawner) in iproduct!(nspawn, nspawner) {
        let rt = rt::new(NUM_THREADS);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(format!("nspawn({nspawn})/nspawner({nspawner})"), |b| {
            b.iter(|| {
                let tx = tx.clone();

                let _gurad = rt.enter();
                tatlin::tx(nspawner, nspawn, tx);

                rx.recv().unwrap();
            });
        });
    }
    group.finish();
}

fn spin(name: &str, nspawn: &[usize], nspawner: &[usize], c: &mut Criterion) {
    let end = Arc::new(AtomicBool::new(false));
    let mut group = c.benchmark_group(format!("tatlin/{name}"));

    for (&nspawn, &nspawner) in iproduct!(nspawn, nspawner) {
        let rt = rt::new(NUM_THREADS);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(format!("nspawn({nspawn})/nspawner({nspawner})"), |b| {
            b.iter(|| {
                end.store(false, Relaxed);

                let _guard = rt.enter();
                tatlin::spin(nspawner, nspawn, Arc::clone(&end));

                // TODO clever await
                while !end.load(Acquire) {
                    std::hint::spin_loop();
                }
            });
        });
    }
    group.finish();
}

fn for_ch(name: &str, nspawn: &[usize], nspawner: &[usize], c: &mut Criterion) {
    let (tx, rx) = mpsc::sync_channel(1);
    let mut group = c.benchmark_group(format!("tatlin/{name}"));

    for (&nspawn, &nspawner) in iproduct!(nspawn, nspawner) {
        let rt = rt::new(NUM_THREADS);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(format!("nspawn({nspawn})/nspawner({nspawner})"), |b| {
            let leaf_handles = (0..nspawner)
                .map(|_| Vec::with_capacity(nspawn))
                .collect::<Vec<_>>();
            let root_handles = Vec::with_capacity(nspawner);

            b.iter_reuse(
                (root_handles, leaf_handles),
                |(root_handles, leaf_handles)| {
                    let tx = tx.clone();

                    let _gurad = rt.enter();
                    tatlin::for_await_ch(nspawner, nspawn, tx, root_handles, leaf_handles);

                    rx.recv().unwrap()
                },
            );
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

fn bench_for_ch(c: &mut Criterion) {
    let nspawn: Vec<usize> = (1..=10).map(|i| i * 1000).collect();
    let nspawner: Vec<usize> = (1..=20).collect();

    for_ch("for_ch", nspawn.as_ref(), nspawner.as_ref(), c)
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));

    targets = bench_for_ch
);

criterion_main!(benches);
