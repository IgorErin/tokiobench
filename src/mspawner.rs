use itertools::{iproduct, Itertools};
use std::sync::mpsc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokiobench::metrics::RuntimeMetrics;
use tokiobench::params::metrics as m;
use tokiobench::path::metrics as mpath;
use tokiobench::path::metrics::store_vec;
use tokiobench::rt;
use tokiobench::watcher;

type Handles = Vec<JoinHandle<()>>;
fn run_iter(
    nspawn: usize,
    nworkers: usize,
    mut handles: Handles,
    sample_slice: Duration,
    mut metrics: Vec<RuntimeMetrics>,
) -> (Handles, Vec<RuntimeMetrics>) {
    let rt = rt::new(nworkers);
    let (stop_tx, stop_rx) = mpsc::sync_channel(1);
    let (tx, rx) = mpsc::sync_channel(1);

    let metrics_handler = {
        let handle = rt.handle();
        let rt_monitor = tokio_metrics::RuntimeMonitor::new(&handle);
        watcher::run(rt_monitor, stop_rx, sample_slice, metrics)
    };

    let _guard = rt.enter();

    tokio::spawn(async move {
        for _ in 0..nspawn {
            handles.push(tokio::spawn(async { std::hint::black_box(()) }));
        }

        for handle in handles.drain(..) {
            handle.await.unwrap();
        }

        tx.send(handles).unwrap();
    });

    handles = rx.recv().unwrap();
    assert!(handles.is_empty());

    stop_tx.send(()).unwrap();
    metrics = metrics_handler.join().unwrap();

    (handles, metrics)
}

fn run_metrics(name: &str, nspawn: &[usize], nworkers: &[usize], sample_slice: Duration) {
    for (&nspawn, &nworkers) in iproduct!(nspawn, nworkers) {
        let mut handles = Vec::with_capacity(nspawn);
        let mut metrics = Vec::with_capacity(m::CHAN_SIZE);

        for niter in 0..m::N_ITER {
            (handles, metrics) = run_iter(nspawn, nworkers, handles, sample_slice, metrics);
            let prefix = mpath::mk_prefix(&format!(
                "sampling({name})_nspawn({nspawn})_nworkers({nworkers})"
            ));
            let name = &format!("iter({niter}).csv");
            store_vec(&prefix, &name, &metrics);
            metrics.clear()
        }
    }
}

fn main() -> () {
    // collect metrics for thousands tasks
    let nspawn: Vec<usize> = (1..=12).map(|i| i * 1000).collect();
    let nwork: Vec<usize> = (1..=20).collect();
    run_metrics("spawner_thousands", &nspawn, &nwork, Duration::from_micros(200));

    // collect metrics for hundreds thousands tasks
    let nspawn: Vec<usize> = (1..=6).map(|i| i * 100_000).collect();
    run_metrics("spawner_hthousands", &nspawn, &nwork, Duration::from_micros(600));

    // collect metrics for millions tasks
    let nspawn: Vec<usize> = (1..=3).map(|i| i * 1_000_000).collect();
    run_metrics("spawner_millions", &nspawn, &nwork, Duration::from_millis(2));
}
