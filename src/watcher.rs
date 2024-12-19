use std::thread;

use std::time::Duration;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{mpsc::SyncSender, Arc};

use tokio_metrics::RuntimeMonitor;

use crate::params::metrics as m;

pub type MetricSyncSender = SyncSender<tokio_metrics::RuntimeMetrics>;

pub fn run(
    metric_tx: MetricSyncSender,
    rem: Arc<AtomicUsize>,
    rt_monitor: RuntimeMonitor,
    sample_slice: u64
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

            thread::sleep(Duration::from_millis(sample_slice));
        }
    });

    thread_handle
}
