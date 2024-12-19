use itertools::{iproduct, Itertools};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{mpsc, Arc};
use tokiobench::params::metrics as m;
use tokiobench::path::metrics as mpath;
use tokiobench::rt;
use tokiobench::serializer::MetricsSerializable;
use tokiobench::watcher;

type Handles = Vec<tokio::task::JoinHandle<()>>;

fn run_iter(
    nspawn: usize,
    nworkers: usize,
    handles: &mut Handles,
    sample_slice: u64,
) -> Vec<tokio_metrics::RuntimeMetrics> {
    let rt = rt::new(nworkers);

    let (m_tx, m_rx) = mpsc::sync_channel(m::CHAN_SIZE);
    let rem: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(1));

    let metrics_handler = {
        let rem = Arc::clone(&rem);
        let handle = rt.handle();
        let rt_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

        watcher::run(m_tx, rem, rt_monitor, sample_slice)
    };

    for _ in 0..nspawn {
        handles.push(rt.spawn(async move {
            std::hint::black_box(()); // TODO(work)
        }));
    }

    rt.block_on(async {
        for handle in handles.drain(..) {
            handle.await.unwrap();
        }
    });

    rem.fetch_sub(1, Relaxed);
    metrics_handler.join().unwrap();

    assert!(handles.is_empty());

    m_rx.into_iter().collect_vec()
}

fn run_metrics(name: &str, nspawn: &[usize], nworkers: &[usize], sample_slice: u64) {
    for (&nspawn, &nworkers) in iproduct!(nspawn, nworkers) {
        let prefix = mpath::mk_prefix(&name);
        let name = format!("nspawn({})_nwork({})", nspawn, nworkers);
        let mut handles: Handles = Vec::with_capacity(nspawn);
        let mut metrics = Vec::new();

        for niter in 0..m::N_ITER {
            let m = run_iter(nspawn, nworkers, &mut handles, sample_slice);
            m.iter().for_each(|m| { metrics.push(MetricsSerializable::new(niter, &m)) });
        }
        mpath::store(&prefix, &name, &metrics);
    }
}

fn main() -> () {
    // collect metrics for thousands tasks
    let nspawn: Vec<usize> = (1..=12).map(|i| i * 1000).collect();
    let nwork: Vec<usize> = (1..=20).collect();
    run_metrics("remote_thousands", &nspawn, &nwork,1);

    // collect metrics for hundreds thousands tasks
    let nspawn: Vec<usize> = (1..=6).map(|i| i * 100_000).collect();
    run_metrics("remote_hthousands", &nspawn, &nwork,5);

    // collect metrics for millions tasks
    let nspawn: Vec<usize> = (1..=3).map(|i| i * 1_000_000).collect();
    run_metrics("remote_millions", &nspawn, &nwork,20);
}
