use serde::Serialize;
use tokio_metrics;

#[derive(Serialize)]
pub struct RuntimeMetrics {
    pub total_steal_count: u64,
    pub max_steal_count: u64,
    pub min_steal_count: u64,
    pub total_steal_operations: u64,
    pub max_steal_operations: u64,
    pub min_steal_operations: u64,
    pub num_remote_schedules: u64,
    pub total_local_schedule_count: u64,
    pub max_local_schedule_count: u64,
    pub min_local_schedule_count: u64,
    pub total_overflow_count: u64,
    pub max_overflow_count: u64,
    pub min_overflow_count: u64,
    pub total_polls_count: u64,
    pub max_polls_count: u64,
    pub min_polls_count: u64,
    pub total_local_queue_depth: usize,
    pub max_local_queue_depth: usize,
    pub min_local_queue_depth: usize,
    pub elapsed: u128,
}

pub fn from_tokio_metrics(m: tokio_metrics::RuntimeMetrics) -> RuntimeMetrics {
    RuntimeMetrics {
        total_steal_count: m.total_steal_count,
        max_steal_count: m.max_steal_count,
        min_steal_count: m.min_steal_count,
        total_steal_operations: m.total_steal_operations,
        max_steal_operations: m.max_steal_operations,
        min_steal_operations: m.min_steal_operations,
        num_remote_schedules: m.num_remote_schedules,
        total_local_schedule_count: m.total_local_schedule_count,
        max_local_schedule_count: m.max_local_schedule_count,
        min_local_schedule_count: m.min_local_schedule_count,
        total_overflow_count: m.total_overflow_count,
        max_overflow_count: m.max_overflow_count,
        min_overflow_count: m.min_overflow_count,
        total_polls_count: m.total_polls_count,
        max_polls_count: m.max_polls_count,
        min_polls_count: m.min_polls_count,
        total_local_queue_depth: m.total_local_queue_depth,
        max_local_queue_depth: m.max_local_queue_depth,
        min_local_queue_depth: m.min_local_queue_depth,
        elapsed: m.elapsed.as_nanos(),
    }
}

#[derive(Serialize)]
pub struct TotalMetrics {
    pub spawned_tasks_count: u64,
    pub remote_schedule_count: u64,
    pub worker_steal_count: u64,
    pub worker_steal_operations: u64,
    pub worker_poll_count: u64,
    pub worker_total_busy_duration: u128,
    pub worker_local_schedule_count: u64,
    pub worker_overflow_count: u64,
}

pub fn total(rt: tokio::runtime::Runtime) -> TotalMetrics {
    let m = rt.metrics();

    let workers = m.num_workers();

    TotalMetrics {
        spawned_tasks_count: m.spawned_tasks_count(),
        remote_schedule_count: m.remote_schedule_count(),
        worker_steal_count: (0..workers).map(|i| m.worker_steal_count(i)).sum(),
        worker_steal_operations: (0..workers).map(|i| m.worker_steal_operations(i)).sum(),
        worker_poll_count: (0..workers).map(|i| m.worker_poll_count(i)).sum(),
        worker_total_busy_duration: (0..workers)
            .map(|i| m.worker_total_busy_duration(i).as_nanos())
            .sum(),
        worker_local_schedule_count: (0..workers)
            .map(|i| m.worker_local_schedule_count(i))
            .sum(),
        worker_overflow_count: (0..workers).map(|i| m.worker_overflow_count(i)).sum(),
    }
}
