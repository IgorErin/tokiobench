use std::sync::mpsc;

use itertools::iproduct;
use tokiobench::params::metrics as m;
use tokiobench::path::metrics as mpath;
use tokiobench::rt;
use tokiobench::tatlin;
use tokiobench::watcher;

const NUM_THREADS: usize = 12;

fn run_samping(name: &str, nspawn: usize, nspawner: usize) {
    let name = format!("nspawn({})_nspawner({})", name, nspawner);
    let prefix = mpath::mk_prefix(&name);

    let mut leaf_handles = (0..nspawner)
        .map(|_| Vec::with_capacity(nspawn))
        .collect::<Vec<_>>();
    let mut root_handles = Vec::with_capacity(nspawner);

    for niter in 0..m::N_ITER {
        let metrics = {
            let rt = rt::new(NUM_THREADS);

            let (m_tx, m_rx) = mpsc::sync_channel(m::CHAN_SIZE);
            let (m_stop_tx, m_stop_rx) = mpsc::sync_channel(1);
            let (rt_tx, rt_rx) = mpsc::sync_channel(1);

            let metrics_handler = {
                let rt_monitor = tokio_metrics::RuntimeMonitor::new(rt.handle());

                watcher::run(rt_monitor, m_tx, m_stop_rx)
            };

            {
                let _guard = rt.enter();
                tatlin::for_await_ch(nspawner, nspawn, rt_tx, root_handles, leaf_handles);
            }

            (root_handles, leaf_handles) = rt_rx.recv().unwrap();

            m_stop_tx.send(()).unwrap();
            metrics_handler.join().unwrap();

            m_rx.into_iter().collect::<Vec<_>>()
        };

        let metrics_u8 = serde_json::to_vec_pretty(&metrics).unwrap();

        let name = format!("iter_{niter}.json");
        mpath::store(&prefix, &name, &metrics_u8);
    }
}

fn main() -> () {
    for (nspawn, nspawner) in iproduct!(5000..=5000, 1..10) {
        run_samping("tatlin", nspawn, nspawner);
    }
}
