use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use crate::params;
use crate::work::Work;

pub type BenchFn = fn(usize, SyncSender<()>, Arc<AtomicUsize>, work: Work, spawner_work: Option<Work>) -> ();

#[inline(always)]
pub fn spawn_current(nspawn: usize, tx: SyncSender<()>, rem: Arc<AtomicUsize>, work: Work, spawner_work: Option<Work>) {
    for _ in 0..nspawn {
        let tx = tx.clone();
        let rem = rem.clone();
        
        if let Some(f) = spawner_work {
            for _ in 0..params::SPAWNER_WORK_BOUND {
                std::hint::black_box(f);
            }
        }
        
        tokio::spawn(async move {
            for _ in 0..params::YIELD_BOUND {
                std::hint::black_box(work());
                tokio::task::yield_now().await;
            }

            if 1 == rem.fetch_sub(1, Relaxed) {
                tx.send(()).unwrap();
            }
        });
    }
}

#[inline(always)]
pub fn spawn_local(nspawn: usize, tx: SyncSender<()>, rem: Arc<AtomicUsize>, work: Work, spawner_work: Option<Work>) {
    tokio::spawn(async move {
        for _ in 0..nspawn {
            let rem = rem.clone();
            let tx = tx.clone();

            if let Some(f) = spawner_work {
                for _ in 0..params::SPAWNER_WORK_BOUND {
                    std::hint::black_box(f);
                    tokio::task::yield_now().await;
                }
            }

            tokio::spawn(async move {
                for _ in 0..params::YIELD_BOUND {
                    std::hint::black_box(work());
                    tokio::task::yield_now().await;
                }

                if 1 == rem.fetch_sub(1, Relaxed) {
                    tx.send(()).unwrap();
                }
            });
        }
    });
}
