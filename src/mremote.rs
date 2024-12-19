use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{mpsc, Arc};

use tokiobench::params;
use tokiobench::params::metrics as m;
use tokiobench::path::metrics as mpath;
use tokiobench::rt;
use tokiobench::watcher;

type Handles = Vec<tokio::task::JoinHandle<()>>;

fn run_iter(
    nspawn: usize,
    nworkers: usize,
    handles: &mut Handles,
) -> Vec<tokio_metrics::RuntimeMetrics> {
    let rt = rt::new(nworkers);

    let (m_tx, m_rx) = mpsc::sync_channel(m::CHAN_SIZE);
    let rem: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(1));

    let metrics_handler = {
        let rem = Arc::clone(&rem);
        let handle = rt.handle();
        let rt_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

        watcher::run(m_tx, rem, rt_monitor, m::SAMPLE_SLICE)
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

    return m_rx.into_iter().collect::<Vec<_>>();
}

fn run_metrics(name: &str, nspawn: usize, nworkers: usize) {
    let name = format!("{}_nwork({})", name, nworkers);
    let prefix = mpath::mk_prefix(&name);
    let mut handles: Handles = Vec::with_capacity(nspawn);

    for niter in 0..m::N_ITER {
        let metrics = run_iter(nspawn, nworkers, &mut handles);
        let metrics_u8 = serde_json::to_vec_pretty(&metrics).unwrap();

        let name = format!("iter_{niter}.json");
        mpath::store(&prefix, &name, &metrics_u8);
    }
}

fn main() -> () {
    for nwork in params::NS_WORKERS {
        run_metrics("remote", 10000000, nwork);
    }
}
