use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::SyncSender;
use std::sync::{mpsc, Arc};

use itertools::iproduct;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use tokiobench::params;
use tokiobench::rt;
use tokiobench::work;
use tokiobench::{split, split::SplitType};

type BenchFn = fn(&[usize], tx: SyncSender<()>, rem: Arc<AtomicUsize>, work: CallBack);
type CallBack = fn() -> ();

#[inline]
fn bench(nspawns: &[usize], tx: SyncSender<()>, rem: Arc<AtomicUsize>, work: CallBack) {
    for &nspawn in nspawns {
        let rem = rem.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            for _ in 0..nspawn {
                let rem = rem.clone();
                let tx = tx.clone();

                tokio::spawn(async move {
                    for _ in 0..params::YIELD_BOUND {
                        std::hint::black_box(work());
                        tokio::task::yield_now().await;
                    }

                    if 1 == rem.fetch_sub(1, Relaxed) {
                        tx.send(()).unwrap();
                    }
                });
            }
        });
    }
}

fn workload(
    bench_fn: BenchFn,
    st: SplitType,
    nsplits: &[usize],
    name: &str,
    work: CallBack,
    c: &mut Criterion,
) {
    let (tx, rx) = mpsc::sync_channel(1000);
    let rem: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let mut group = c.benchmark_group(name);

    for (nworkers, nsplit) in iproduct!(params::NS_WORKERS, nsplits) {
        let nspawn = params::N_SPAWN_GLOBAL;
        let workload = split::split(st, nspawn, nsplit.clone());
        let rt = rt::new(nworkers);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_with_input(
            format!("nspawn({nspawn})/nwork({nworkers})/nsplit({nsplit}, {st})"),
            &nspawn,
            |b, &nspawn| {
                b.iter(|| {
                    let tx = tx.clone();
                    let rem = rem.clone();

                    rem.store(nspawn, Relaxed);
                    rt.block_on(async {
                        bench_fn(workload.as_ref(), tx, rem, work);

                        rx.recv().unwrap();
                    });
                });
            },
        );
    }
}

// Uniform local split

fn spawn_workload_uniform_local(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Uniform,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::nothing,
        c,
    );
}

fn spawn_workload_uniform_local_float(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Uniform,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::float_max,
        c,
    );
}

fn spawn_workload_uniform_local_int(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Uniform,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::int_max,
        c,
    );
}

// Uniform global split

fn spawn_workload_uniform_global(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Uniform,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::nothing,
        c,
    );
}

fn spawn_workload_uniform_global_float(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Uniform,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::float_max,
        c,
    );
}

fn spawn_workload_uniform_global_int(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Uniform,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::int_max,
        c,
    );
}

// Geometric local

fn spawn_workload_geometric_local(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Geometric,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::nothing,
        c,
    );
}

fn spawn_workload_geometric_local_float(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Geometric,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::float_max,
        c,
    );
}

fn spawn_workload_geometric_local_int(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Geometric,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::int_max,
        c,
    );
}

// Geometric global

fn spawn_workload_geometric_global(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Geometric,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::nothing,
        c,
    );
}

fn spawn_workload_geometric_global_float(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Geometric,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::float_max,
        c,
    );
}

fn spawn_workload_geometric_global_int(c: &mut Criterion) {
    workload(
        bench,
        SplitType::Geometric,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::int_max,
        c,
    );
}

criterion_group!(
    spawn_benches,
    // work: nothing
    spawn_workload_uniform_local,
    spawn_workload_uniform_global,
    spawn_workload_geometric_local,
    spawn_workload_geometric_global,
    // work: float max
    spawn_workload_uniform_local_float,
    spawn_workload_uniform_global_float,
    spawn_workload_geometric_local_float,
    spawn_workload_geometric_global_float,
    // work: int max
    spawn_workload_uniform_local_int,
    spawn_workload_uniform_global_int,
    spawn_workload_geometric_local_int,
    spawn_workload_geometric_global_int,
);

criterion_main!(spawn_benches);
