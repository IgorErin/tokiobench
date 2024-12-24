use itertools::{iproduct, Itertools};
use std::sync::mpsc;
use std::time::Duration;
use tokiobench::metrics::RuntimeMetrics;
use tokiobench::params::metrics as m;
use tokiobench::path::metrics as mpath;
use tokiobench::path::metrics::store_vec;
use tokiobench::rt;
use tokiobench::watcher;

type Handles = Vec<tokio::task::JoinHandle<()>>;

fn run_iter(
    nspawn: usize,
    nworkers: usize,
    handles: &mut Handles,
    sample_slice: Duration,
    mut metrics: Vec<RuntimeMetrics>
) -> Vec<RuntimeMetrics> {
    let rt = rt::new(nworkers);
    let (stop_tx, stop_rx) = mpsc::sync_channel(1);

    let metrics_handler = {
        let handle = rt.handle();
        let rt_monitor = tokio_metrics::RuntimeMonitor::new(&handle);
        watcher::run(rt_monitor, stop_rx, sample_slice, metrics)
    };

    for _ in 0..nspawn {
        handles.push(rt.spawn(async move {
            std::hint::black_box(());
        }));
    }

    rt.block_on(async {
        for handle in handles.drain(..) {
            handle.await.unwrap();
        }
    });

    stop_tx.send(()).unwrap();
    metrics = metrics_handler.join().unwrap();
    assert!(handles.is_empty());
    metrics
}

fn run_metrics(name: &str, nspawn: &[usize], nworkers: &[usize], sample_slice: Duration) {
    for (&nspawn, &nworkers) in iproduct!(nspawn, nworkers) {
        let mut handles: Handles = Vec::with_capacity(nspawn);
        let mut metrics = Vec::with_capacity(m::CHAN_SIZE);

        for niter in 0..m::N_ITER {
            metrics = run_iter(nspawn, nworkers, &mut handles, sample_slice, metrics);
            let prefix = mpath::mk_prefix(&format!(
                "sampling({name})_nspawn({nspawn})_nworkers({nworkers})"
            ));
            let name = &format!("iter({niter}).csv");
            store_vec(&prefix, &name, &metrics);
        }
    }
}

fn main() -> () {
    // collect metrics for thousands tasks
    let nspawn: Vec<usize> = (1..=12).map(|i| i * 1000).collect();
    let nwork: Vec<usize> = (1..=20).collect();
    run_metrics("remote_thousands", &nspawn, &nwork, Duration::from_micros(10));

    // collect metrics for hundreds thousands tasks
    let nspawn: Vec<usize> = (1..=6).map(|i| i * 100_000).collect();
    run_metrics("remote_hthousands", &nspawn, &nwork, Duration::from_micros(100));

    // collect metrics for millions tasks
    let nspawn: Vec<usize> = (1..=3).map(|i| i * 1_000_000).collect();
    run_metrics("remote_millions", &nspawn, &nwork, Duration::from_millis(1));
}
