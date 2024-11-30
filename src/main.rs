use std::io::Write;
use std::{fs, thread};

use std::time::Duration;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{mpsc, mpsc::SyncSender, Arc};

use tokio_metrics::RuntimeMonitor;

use std::fs::File;
use std::path::{Path, PathBuf};

use tokiobench::params;
use tokiobench::params::metrics as m;
use tokiobench::rt;
use tokiobench::spawner;

fn metrics_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();

    path.push("target");
    path.push("metrics");

    path
}

fn mk_prefix_dir(folder: &str) -> PathBuf {
    let mut path = metrics_path();
    path.push(folder);

    if !Path::exists(&path) {
        fs::create_dir_all(&path).unwrap();
    }

    path
}

fn store(prefix: &Path, name: &str, data: &[u8]) {
    let result_path = {
        let mut prefix = PathBuf::from(prefix);
        prefix.push(name);
        prefix
    };

    let mut f = File::create(result_path).unwrap();
    f.write_all(data).unwrap();
}

type MetricSyncSender = SyncSender<tokio_metrics::RuntimeMetrics>;

fn run_watcher(
    metric_tx: MetricSyncSender,
    rem: Arc<AtomicUsize>,
    rt_monitor: RuntimeMonitor,
) -> std::thread::JoinHandle<()> {
    let thread_handle = thread::spawn(move || {
        let mut metrics_count = 0;

        for interval in rt_monitor.intervals() {
            metrics_count += 1;
            if metrics_count >= m::CHAN_SIZE {
                panic!("metrics overflow");
            }
            metric_tx.send(interval).unwrap();

            if rem.load(Relaxed) == 0 {
                break;
            }

            thread::sleep(Duration::from_millis(m::SAMPLE_SLICE));
        }
    });

    thread_handle
}

fn run_iter(
    count_down: usize,
    nworkers: usize,
    bench_fn: spawner::BenchFn,
) -> Vec<tokio_metrics::RuntimeMetrics> {
    let rt = rt::new(nworkers);

    let (tx, rx) = mpsc::sync_channel(1);
    let (m_tx, m_rx) = mpsc::sync_channel(m::CHAN_SIZE);
    let rem: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(count_down));

    let metrics_handler = {
        let rem = Arc::clone(&rem);
        let handle = rt.handle();
        let rt_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

        run_watcher(m_tx, rem, rt_monitor)
    };

    rt.block_on(async move {
        bench_fn(count_down, tx, rem);

        rx.recv().unwrap();
    });

    metrics_handler.join().unwrap();

    return m_rx.into_iter().collect::<Vec<_>>();
}

fn run_metrics(name: &str, count_down: usize, nworkers: usize, bench_fn: spawner::BenchFn) {
    let name = format!("{}_nwork({})", name, nworkers);

    let prefix = mk_prefix_dir(&name);

    for niter in 0..m::N_ITER {
        let metrics = run_iter(count_down, nworkers, bench_fn);
        let metrics_u8 = serde_json::to_vec_pretty(&metrics).unwrap();

        let name = format!("iter_{niter}.json");
        store(&prefix, &name, &metrics_u8);
    }
}

fn main() -> () {
    for nwork in params::NS_WORKERS {
        run_metrics(
            "spawner_current_recstall",
            params::N_SPAWN_LOCAL,
            nwork,
            spawner::spawn_current_rec,
        );
        run_metrics(
            "spawner_local_recstall",
            params::N_SPAWN_LOCAL,
            nwork,
            spawner::spawn_local_rec,
        );
    }
}
