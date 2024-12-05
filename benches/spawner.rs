use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{mpsc, Arc};

use itertools::iproduct;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use tokiobench::rt;
use tokiobench::spawner as sp;
use tokiobench::work::Work;
use tokiobench::{params, work};

fn bench(bench_fn: sp::BenchFn, work: Work, spawn_work: Option<Work>, name: &str, c: &mut Criterion) {
    let (tx, rx) = mpsc::sync_channel(1000);
    let rem: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let mut group = c.benchmark_group(name);

    for (nspawn, nworkers) in iproduct!(params::NS_SPAWN_LOCAL, params::NS_WORKERS) {
        let rt = rt::new(nworkers);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_with_input(
            format!("nspawn({nspawn})/nwork({nworkers})"),
            &nspawn,
            |b, &nspawn| {
                b.iter(|| {
                    let tx = tx.clone();
                    let rem = rem.clone();

                    rem.store(nspawn, Relaxed);
                    rt.block_on(async {
                        bench_fn(nspawn, tx, rem, work, spawn_work);

                        rx.recv().unwrap();
                    });
                });
            },
        );
    }
}

fn spawn_current(c: &mut Criterion) {
    bench(sp::spawn_current, work::nothing, None, "spawn_current", c)
}

fn spawn_local(c: &mut Criterion) {
    bench(sp::spawn_local, work::nothing, None, "spawn_local", c);
}

fn spawn_local_float_max(c: &mut Criterion) {
    bench(sp::spawn_local, work::float_max, None, "spawn_local_float_max", c);
}

fn spawn_local_int_max(c: &mut Criterion) {
    bench(sp::spawn_local, work::int_max, None, "spawn_local_int_max", c);
}

fn spawn_local_float_max_s_float_min(c: &mut Criterion) {
    bench(sp::spawn_local, work::float_max, Some(work::float_min), "spawn_local_float_max_s_float_min", c)
}
fn spawn_local_float_max_s_float_mid(c: &mut Criterion) {
    bench(sp::spawn_local, work::float_max, Some(work::float_mid), "spawn_local_float_max_s_float_mid", c)
}

fn spawn_current_float_max(c: &mut Criterion) {
    bench(sp::spawn_current, work::float_max, None, "spawn_current_float_max", c)
}

fn spawn_current_int_max(c: &mut Criterion) {
    bench(sp::spawn_current, work::int_max, None, "spawn_current_int_max", c)
}

fn spawn_current_int_max_s_int_mid(c: &mut Criterion) {
    bench(sp::spawn_current, work::int_max, Some(work::int_mid), "spawn_current_int_max_s_int_mid", c)
}

fn spawn_current_int_max_s_int_min(c: &mut Criterion) {
    bench(sp::spawn_current, work::int_max, Some(work::int_min), "spawn_current_int_max_s_int_min", c)
}

criterion_group!(
    benches,
    spawn_current,
    spawn_local,
    spawn_local_float_max,
    spawn_local_int_max,
    spawn_local_float_max_s_float_mid,
    spawn_local_float_max_s_float_min,
    spawn_current_float_max,
    spawn_current_int_max,
    spawn_current_int_max_s_int_mid,
    spawn_current_int_max_s_int_min,
);

criterion_main!(benches);
