use cfg_if::cfg_if;
use itertools::iproduct;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use tokiobench::params;
use tokiobench::rt;
use tokiobench::work;
use tokiobench::{split, split::SplitType};

type CallBack = fn() -> ();

fn workload(st: SplitType, nsplits: &[usize], name: &str, work: CallBack, c: &mut Criterion) {
    let mut group = c.benchmark_group(name);

    for (nworkers, &nsplit) in iproduct!(params::NS_WORKERS, nsplits) {
        let nspawn = params::N_SPAWN_GLOBAL;
        let workload = split::split(st, nspawn, nsplit);
        let rt = rt::new(nworkers);

        group.throughput(Throughput::Elements(nspawn as u64));
        group.bench_function(
            format!("nspawn({nspawn})/nwork({nworkers})/nsplit({nsplit}, {st})"),
            |b| {
                // lift this TODO()
                let workload_and_buffers = workload
                    .iter()
                    .map(|&n| Vec::with_capacity(n))
                    .collect::<Vec<_>>();
                b.iter_reuse(
                    (Vec::with_capacity(workload.len()), workload_and_buffers),
                    |(mut root_handles, mut leaf_handles)| {
                        cfg_if!(if #[cfg(feature = "check")] {
                            assert!(root_handles.is_empty());
                            assert!(root_handles.capacity() == workload.len());

                            assert!(leaf_handles.iter().all(|i| i.is_empty()));
                            assert_eq!(leaf_handles.iter().map(|i| i.capacity()).sum::<usize>(), nspawn);
                        });

                        rt.block_on(async {
                            for mut leaf_handle in leaf_handles.drain(..) {
                                root_handles.push(tokio::spawn(async move {
                                    for _ in 0..leaf_handle.capacity() {
                                        leaf_handle.push(tokio::spawn(async move {
                                            work();
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
            },
        );
    }
}

// Uniform local split

fn spawn_workload_uniform_local(c: &mut Criterion) {
    workload(
        SplitType::Uniform,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::nothing,
        c,
    );
}

fn spawn_workload_uniform_local_float(c: &mut Criterion) {
    workload(
        SplitType::Uniform,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::float_max,
        c,
    );
}

fn spawn_workload_uniform_local_int(c: &mut Criterion) {
    workload(
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
        SplitType::Uniform,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::nothing,
        c,
    );
}

fn spawn_workload_uniform_global_float(c: &mut Criterion) {
    workload(
        SplitType::Uniform,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::float_max,
        c,
    );
}

fn spawn_workload_uniform_global_int(c: &mut Criterion) {
    workload(
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
        SplitType::Geometric,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::nothing,
        c,
    );
}

fn spawn_workload_geometric_local_float(c: &mut Criterion) {
    workload(
        SplitType::Geometric,
        &params::NS_SPLIT_LOCAL,
        "workload_local",
        work::float_max,
        c,
    );
}

fn spawn_workload_geometric_local_int(c: &mut Criterion) {
    workload(
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
        SplitType::Geometric,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::nothing,
        c,
    );
}

fn spawn_workload_geometric_global_float(c: &mut Criterion) {
    workload(
        SplitType::Geometric,
        &params::NS_SPLIT_GLOBAL,
        "workload_global",
        work::float_max,
        c,
    );
}

fn spawn_workload_geometric_global_int(c: &mut Criterion) {
    workload(
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
