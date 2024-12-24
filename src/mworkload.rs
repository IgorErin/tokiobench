use cfg_if::cfg_if;
use itertools::{iproduct, Itertools};
use std::sync::mpsc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokiobench::metrics::RuntimeMetrics;
use tokiobench::params as p;
use tokiobench::params::metrics as m;
use tokiobench::path::metrics as mpath;
use tokiobench::path::metrics::store_vec;
use tokiobench::rt;
use tokiobench::watcher;

type Handles = Vec<JoinHandle<()>>;

struct Reusable {
    pub root_handles: Vec<JoinHandle<Handles>>,
    pub leaf_handles: Vec<Handles>,
    pub metrics: Vec<RuntimeMetrics>,
}

fn run_iter(
    _nspawn: usize,
    _nspawner: usize,
    sample_slice: Duration,
    reuse: Reusable,
) -> Reusable {
    let rt = rt::new(p::N_WORKERS);
    let (tx, rx) = mpsc::sync_channel(1);
    let (stop_tx, stop_rx) = mpsc::sync_channel(1);
    let Reusable {
        mut root_handles, mut leaf_handles, mut metrics
    } = reuse;

    cfg_if!(if #[cfg(feature = "check")] {
                        assert!(root_handles.is_empty());
                        assert!(root_handles.capacity() == _nspawner);
                        assert!(leaf_handles.iter().all(|i| i.is_empty()));
                        assert!(leaf_handles.iter().all(|i| i.capacity() == _nspawn));
                    });

    let metrics_handler = {
        let handle = rt.handle();
        let rt_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

        watcher::run(rt_monitor, stop_rx, sample_slice, metrics)
    };


    let _guard = rt.enter();

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

    (leaf_handles, root_handles) = rx.recv().unwrap();

    stop_tx.send(()).unwrap();
    metrics = metrics_handler.join().unwrap();

    assert!(root_handles.is_empty());

    Reusable { root_handles, leaf_handles, metrics }
}

fn run_metrics(name: &str, nspawn: &[usize], nspawner: &[usize], sample_slice: Duration) {
    for (&nspawn, &nspawner) in iproduct!(nspawn, nspawner) {
        let mut reuse = Reusable {
            root_handles: Vec::with_capacity(nspawner),
            leaf_handles: (0..nspawner)
                .map(|_| Vec::with_capacity(nspawn))
                .collect::<Vec<_>>(),
            metrics: Vec::with_capacity(m::CHAN_SIZE),
        };

        for niter in 0..m::N_ITER {
            reuse = run_iter(nspawn, nspawner, sample_slice, reuse);

            let prefix = mpath::mk_prefix(&format!(
                "sampling({name})_nspawn({nspawn})_nspawner({nspawner})"
            ));
            let name = &format!("iter({niter}).csv");
            store_vec(&prefix, &name, &reuse.metrics);
        };
    }
}

fn main() -> () {
    // collect metrics for thousands tasks
    let nspawn: Vec<usize> = (1..=12).map(|i| i * 1000).collect();
    let nspawner: Vec<usize> = (1..=20).collect();
    run_metrics("workload_thousands", &nspawn, &nspawner, Duration::from_micros(10));

    // collect metrics for hundreds thousands tasks
    let nspawn: Vec<usize> = (1..=6).map(|i| i * 100_000).collect();
    run_metrics("workload_hthousands", &nspawn, &nspawner, Duration::from_micros(700));

    // collect metrics for millions tasks
    let nspawn: Vec<usize> = (1..=3).map(|i| i * 1_000_000).collect();
    run_metrics("workload_millions", &nspawn, &nspawner, Duration::from_millis(2));
}