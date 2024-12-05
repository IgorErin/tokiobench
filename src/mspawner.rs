use std::sync::atomic::AtomicUsize;
use std::sync::{mpsc, Arc};

use tokiobench::{params, work};
use tokiobench::params::metrics as m;
use tokiobench::path::metrics as mpath;
use tokiobench::rt;
use tokiobench::spawner;
use tokiobench::watcher;
use tokiobench::work::Work;

fn run_iter(
    count_down: usize,
    nworkers: usize,
    bench_fn: spawner::BenchFn,
    work: Work, spawn_work: Option<Work>
) -> Vec<tokio_metrics::RuntimeMetrics> {
    let rt = rt::new(nworkers);

    let (tx, rx) = mpsc::sync_channel(1);
    let (m_tx, m_rx) = mpsc::sync_channel(m::CHAN_SIZE);
    let rem: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(count_down));

    let metrics_handler = {
        let rem = Arc::clone(&rem);
        let handle = rt.handle();
        let rt_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

        watcher::run(m_tx, rem, rt_monitor)
    };

    rt.block_on(async move {
        bench_fn(count_down, tx, rem, work, spawn_work);

        rx.recv().unwrap();
    });

    metrics_handler.join().unwrap();

    return m_rx.into_iter().collect::<Vec<_>>();
}

fn run_metrics(name: &str, count_down: usize, nworkers: usize, bench_fn: spawner::BenchFn, work: Work, spawn_work: Option<Work>) {
    let name = format!("{}_nwork({})", name, nworkers);
    let prefix = mpath::mk_prefix(&name);

    for niter in 0..m::N_ITER {
        let metrics = run_iter(count_down, nworkers, bench_fn, work, spawn_work);
        let metrics_u8 = serde_json::to_vec_pretty(&metrics).unwrap();

        let name = format!("iter_{niter}.json");
        mpath::store(&prefix, &name, &metrics_u8);
    }
}

fn main() -> () {
    for nwork in params::NS_WORKERS {
        run_metrics(
            "spawner_current",
            params::N_SPAWN_LOCAL,
            nwork,
            spawner::spawn_current,
            work::nothing,
            None,
        );
        run_metrics(
            "spawner_local",
            params::N_SPAWN_LOCAL,
            nwork,
            spawner::spawn_local,
            work::nothing,
            None,
        );
        run_metrics(
            "spawner_current_mid_int",
            params::N_SPAWN_LOCAL,
            nwork,
            spawner::spawn_current,
            work::int_mid,
            None,
        );
        run_metrics(
            "spawner_local_mid_float",
            params::N_SPAWN_LOCAL,
            nwork,
            spawner::spawn_local,
            work::float_mid,
            None
        );
    }
}
