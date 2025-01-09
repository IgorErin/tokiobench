import pathlib as lpath
import re
import sys
import json

import matplotlib.pyplot as plt
import matplotlib
import pandas as pd
import seaborn as sns

import params as p


def fetch_sampling() -> list[lpath.Path]:
    return list((p.METRICS_PATH / "sampling").glob("*"))


def fetch_n(name: str, path: lpath.Path, *, is_dir: bool) -> int | None:
    assert path.is_dir() is is_dir

    pattern = re.compile(fr"{name}_(\d+)")
    match = pattern.match(path.name)
    if not match:
        return None

    return int(match.group(1))


def fetch_sampling_iters() -> list[dict[str, int | str | lpath.Path]]:
    res = []
    for worker_path in fetch_sampling():
        nworker = fetch_n("nworker", worker_path, is_dir=True)
        if not nworker: continue
        for nspawner_path in worker_path.glob("*"):
            nspawner = fetch_n("nspawner", nspawner_path, is_dir=True)
            if not nspawner: continue
            for nspawn_path in nspawner_path.glob("*"):
                nspawn = fetch_n("nspawn", nspawn_path, is_dir=True)
                if not nspawn: continue
                for name_path in nspawn_path.glob("*"):
                    for iter_path in name_path.glob("*"):
                        niter = fetch_n("iter", iter_path, is_dir=False)
                        res.append({
                            "nworker": nworker,
                            "nspawner": nspawner,
                            "nspawn": nspawn,
                            "niter": niter,
                            "path": iter_path,
                            "name": name_path.name,
                        })
    return res


def group_by(grouper: str, data: list[dict[str, int | str | lpath.Path]]) -> dict[
    int | str, list[dict[str, int | str | lpath.Path]]]:
    diffs = set(map(lambda i: i[grouper], data))

    res = {}
    for key in diffs:
        assert (not isinstance(key, lpath.Path))
        res[key] = [d for d in data if d[grouper] == key]

    return res


metrics = {
    "global_queue_depth": "Global Queue Depth",
    "total_steal_count": "Total Steal Count",
    "total_local_queue_depth": "Total Local Queue Depth",
}


def scatter_plot(df, result_path: lpath.Path):
    fig, axs = plt.subplots(1, len(metrics), figsize=(5 * len(metrics), 4))
    for n_ax, (metric, m_name) in enumerate(metrics.items()):
        x_ = df["time_nanos"].to_numpy()
        y_ = df[metric].to_numpy()
        ax = axs[n_ax]
        sns.scatterplot(x=x_, y=y_, ax=ax, alpha=0.5, s=10)
        ax.set_xlabel("Time (nanoseconds)")
        ax.set_ylabel(metrics.get(metric))

    plt.tight_layout()
    fig.savefig(result_path / ("scatterplot"))
    plt.close()


def rolling_mean_plot(df, result_path: lpath.Path, window=500):
    fig, axs = plt.subplots(1, len(metrics), figsize=(5 * len(metrics), 4))
    sorted_df = df.sort_values(by="time_nanos")
    for n_ax, (metric, m_name) in enumerate(metrics.items()):
        x_ = sorted_df["time_nanos"].to_numpy()
        y_ = sorted_df[metric]
        rolling_mean = y_.rolling(window=window, min_periods=1).mean().to_numpy()
        ax = axs[n_ax]
        sns.scatterplot(x=x_, y=y_, ax=ax, alpha=0.5, s=10)
        sns.lineplot(x=x_, y=rolling_mean, color='blue', label=f'Rolling Mean (window={window})', ax=ax)
        ax.set_xlabel("Time (nanoseconds)")
        ax.set_ylabel(metrics.get(metric))
        ax.legend()

    plt.tight_layout()
    fig.savefig(result_path / f"rolling_mean_{window}")
    plt.close()


def run_sampling():
    def mk_resdir(*, name: str, nworker: int, nspawner: int, nspawn: int):
        header = p.RESULT_PATH / "sampling" / name
        res_dir = header / f"nworker_{nworker}" / f"nspawner_{nspawner}" / f"nspawn_{nspawn}"
        res_dir.mkdir(mode=0o777, parents=True, exist_ok=True)

        return res_dir

    def merge_iterations(data: list[dict[str, int | str | lpath.Path]]) -> pd.DataFrame:
        all_data = []
        for d in data:
            df = pd.read_csv(d["path"])
            df['iteration'] = d["niter"]
            all_data.append(df)
        return pd.concat(all_data, ignore_index=True)

    data = fetch_sampling_iters()

    for name, name_data in group_by("name", data).items():
        for nworker, nworker_data in group_by("nworker", name_data).items():
            for nspawner, nspawner_data in group_by("nspawner", nworker_data).items():
                for nspawn, nspawn_data in group_by("nspawn", nspawner_data).items():
                    df = merge_iterations(nspawn_data)
                    res_dir = mk_resdir(name=name, nworker=nworker, nspawner=nspawner, nspawn=nspawn)

                    df["time_nanos"] = df.groupby('iteration')['elapsed'].cumsum()

                    rolling_mean_plot(df, res_dir)


def fetch_total() -> list[dict[str, int | str | lpath.Path]]:
    res = []
    for worker_path in (p.METRICS_PATH / "total").glob("*"):
        nworker = fetch_n("nworker", worker_path, is_dir=True)
        if not nworker: continue
        for nspawner_path in worker_path.glob("*"):
            nspawner = fetch_n("nspawner", nspawner_path, is_dir=True)
            if not nspawner: continue
            for nspawn_path in nspawner_path.glob("*"):
                nspawn = fetch_n("nspawn", nspawn_path, is_dir=True)
                if not nspawn: continue
                for name_path in nspawn_path.glob("*"):
                    for json_path in name_path.glob("total.json"):
                        res.append({
                            "nworker": nworker,
                            "nspawner": nspawner,
                            "nspawn": nspawn,
                            "name": name_path.name,
                            "path": json_path,
                        })
    return res


def run_sum_total():
    def plot_sum_total(*, name: str, nworker: int, nspawn: int, data: list[dict[str, int | str | lpath.Path]],
                       res_dir: lpath.Path) -> None:
        metrics = ["worker_local_schedule_count", "worker_steal_operations", "worker_overflow_count"]
        lable = ["local shedule", "steal operations", "overflow count"]

        fig, axs = plt.subplots(1, len(metrics))
        data = sorted(data, key=lambda t: t["nspawner"])

        nspawner = list(map(lambda t: t["nspawner"], data))
        json_data = list(map(lambda t: json.load(t["path"].open()), data))

        for ind, metric in enumerate(metrics):
            ax = axs[ind]

            y_points = list(map(lambda j: sum(j[metric]), json_data))
            if any(y > 0 for y in y_points):
                ax.set_yscale('log')

            x_points = nspawner

            ax.stem(x_points, y_points)

            ax.set_xlabel("number of spawners")
            ax.set_title(lable[ind])

        print("total_sum saved in in:", res_dir)

        fig.set_figwidth(10)
        fig.set_figheight(10)

        fig.suptitle(f"Benchmark: {name} workers: {nworker} leaf tasks: {nspawn}")
        fig.savefig(res_dir / "total_sum")

    data = fetch_total()

    for name, name_data in group_by("name", data).items():
        for nworker, nworker_data in group_by("nworker", name_data).items():
            for nspawn, nspawn_data in group_by("nspawn", nworker_data).items():
                print(f"runing total for {name}: nworker: {nworker}, nspawn: {nspawn}")

                res_dir = p.RESULT_PATH / "total" / name / f"nworker:{nworker}" / f"nspawn:{nspawn}"
                res_dir.mkdir(mode=0o777, parents=True, exist_ok=True)

                plot_sum_total(name=name, nworker=nworker, nspawn=nspawn, data=nspawn_data, res_dir=res_dir)
