use std::thread;

use std::time::Duration;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

use tokio_metrics::RuntimeMonitor;

use crate::metrics;

pub type MetricSyncSender = SyncSender<tokio_metrics::RuntimeMetrics>;

pub fn run(
    rt_monitor: RuntimeMonitor,
    stop_rx: Receiver<()>,
    slice: Duration,
    mut result: Vec<metrics::RuntimeMetrics>,
) -> std::thread::JoinHandle<Vec<metrics::RuntimeMetrics>> {
    let thread_handle = thread::spawn(move || {
        let mut metrics_count = 0;

        for interval in rt_monitor.intervals() {
            metrics_count += 1;
            if metrics_count >= result.capacity() {
                panic!("metrics overflow");
            }
            // rewrite to vector
            result.push(metrics::from_tokio_metrics(interval));

            if stop_rx.try_recv().is_ok() {
                break;
            }

            thread::sleep(slice);
        }

        result
    });

    thread_handle
}
