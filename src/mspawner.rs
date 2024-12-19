use std::sync::atomic::AtomicUsize;
use std::sync::{mpsc, Arc};
use std::sync::atomic::Ordering::Relaxed;
use itertools::{iproduct, Itertools};
use tokio::task::JoinHandle;
use tokiobench::params::metrics as m;
use tokiobench::path::metrics as mpath;
use tokiobench::rt;
use tokiobench::serializer::MetricsSerializable;
use tokiobench::watcher;

type Handles = Vec<JoinHandle<()>>;
fn run_iter(
    nspawn: usize,
    nworkers: usize,
    mut handles: Handles,
    sample_slice: u64,
) -> (Handles, Vec<tokio_metrics::RuntimeMetrics>) {
    let rt = rt::new(nworkers);

    let (m_tx, m_rx) = mpsc::sync_channel(m::CHAN_SIZE);
    let rem: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(1));
    let (tx, rx) = mpsc::sync_channel(1);

    let metrics_handler = {
        let rem = Arc::clone(&rem);
        let handle = rt.handle();
        let rt_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

        watcher::run(m_tx, rem, rt_monitor, sample_slice)
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

    rem.fetch_sub(1, Relaxed);
    metrics_handler.join().unwrap();

    (handles, m_rx.into_iter().collect_vec())
}

fn run_metrics(name: &str, nspawn: &[usize], nworkers: &[usize], sample_slice: u64) {
    for (&nspawn, &nworkers) in iproduct!(nspawn, nworkers) {
        let prefix = mpath::mk_prefix(&name);
        let name = format!("nspawn({})_nwork({})", nspawn, nworkers);
        let mut handles = Vec::with_capacity(nspawn);
        let mut metrics = Vec::new();

        for niter in 0..m::N_ITER {
            let output = run_iter(nspawn, nworkers, handles, sample_slice);
            handles = output.0;
            let m = output.1;
            m.iter().for_each(|m| { metrics.push(MetricsSerializable::new(niter, &m)) });
        }
        mpath::store(&prefix, &name, &metrics);
    }
}

fn main() -> () {
    // collect metrics for thousands tasks
    let nspawn: Vec<usize> = (1..=12).map(|i| i * 1000).collect();
    let nwork: Vec<usize> = (1..=20).collect();
    run_metrics("spawner_thousands", &nspawn, &nwork,1);

    // collect metrics for hundreds thousands tasks
    let nspawn: Vec<usize> = (1..=6).map(|i| i * 100_000).collect();
    run_metrics("spawner_hthousands", &nspawn, &nwork,5);

    // collect metrics for millions tasks
    let nspawn: Vec<usize> = (1..=3).map(|i| i * 1_000_000).collect();
    run_metrics("spawner_millions", &nspawn, &nwork,20);
}
