use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Release;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use cfg_if::cfg_if;

use futures::prelude::*;
use tokio::task::JoinHandle;

async fn task() {
    std::hint::black_box(());
}

async fn spawn_tasks(n: usize) {
    // assume compiler reduce allocation TODO()
    future::join_all((0..n).into_iter().map(|_| tokio::spawn(task()))).await;
}

pub fn tx(nspawner: usize, nspawn: usize, tx: SyncSender<()>) {
    tokio::spawn(async move {
        // assume compiler reduce allocation TODO()
        future::join_all(
            (0..nspawner)
                .into_iter()
                .map(|_| tokio::spawn(spawn_tasks(nspawn))),
        )
        .await;

        tx.send(()).unwrap();
    });
}

pub fn spin(nspawner: usize, nspawn: usize, end: Arc<AtomicBool>) {
    tokio::spawn(async move {
        // assume compiler reduce allocation TODO()
        future::join_all(
            (0..nspawner)
                .into_iter()
                .map(|_| tokio::spawn(spawn_tasks(nspawn))),
        )
        .await;

        end.store(true, Release);
    });
}

async fn spawn_tasks_vec(n: usize, mut handles: Vec<JoinHandle<()>>) -> Vec<JoinHandle<()>> {
    // assume compiler reduce allocation TODO()
    for _ in 0..handles.capacity() {
        handles.push(tokio::spawn(task()))
    }

    handles
}

// TODO (duplication, types etc)
pub fn for_await_ch(
    nspawner: usize,
    nspawn: usize,
    tx: SyncSender<(
        Vec<JoinHandle<Vec<JoinHandle<()>>>>,
        Vec<Vec<JoinHandle<()>>>,
    )>,
    mut root_handles: Vec<JoinHandle<Vec<JoinHandle<()>>>>,
    mut leaf_handles: Vec<Vec<JoinHandle<()>>>,
) {
    cfg_if!(if #[cfg(feature = "check")] {
        assert!(root_handles.is_empty());
        assert!(root_handles.capacity() == nspawner);

        assert!(leaf_handles.iter().all(|i| i.is_empty() && i.capacity() == nspawn));
        assert!(leaf_handles.len() == nspawner);
    });

    tokio::spawn(async move {
        // assume compiler reduce allocation TODO()
        for leaf_handle in leaf_handles.drain(..) {
            root_handles.push(tokio::spawn(spawn_tasks_vec(nspawn, leaf_handle)));
        }

        for leaf_handle in root_handles.drain(..) {
            let mut leaf_handle = leaf_handle.await.unwrap();

            for leaf in leaf_handle.drain(..) {
                leaf.await.unwrap();
            }

            leaf_handles.push(leaf_handle)
        }

        tx.send((root_handles, leaf_handles)).unwrap()
    });
}
