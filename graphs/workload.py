import itertools as it
import json
import os
import pathlib as lpath
import re
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

import expwrap as ew
import params as p

sns.set_theme()

NAMES = it.product(["workload_local"], ["Geometric", "Uniform"])
DIRS = [f for f in os.scandir(p.CRITERION_PATH) if f.is_dir()]

def plot(*, bench: str, path: lpath.Path, tsplit: str, nsplits: list[int]) -> None:
    plt.xlabel("nsplit")
    plt.ylabel("mean time, ms")

    legend = []

    for nwork in p.NS_WORKERS:
        data_x: list[int] = []
        data_y: list[int] = []

        for nsplit in nsplits:
            name = f"nspawn({p.N_SPAWN_GLOBAL})_nwork({nwork})_nsplit({nsplit}, {tsplit})"
            est_path = p.CRITERION_PATH / bench / name / "new" / "estimates.json"

            instance = json.load(est_path.open())

            data_x.append(nsplit)

            mean = instance["mean"]["point_estimate"]
            data_y.append(mean)

        plt.errorbar(data_x, data_y, label=str(nsplit))

        legend.append(f"{nwork} workers")

    plt.legend(legend)
    plt.savefig(path)

    plt.clf()


def scatter_plots(path: lpath.Path):
    pattern = re.compile(r"nspawn\((\d+)\)_nspawner\((\d+)\)")
    name_pattern = re.compile(r"nspawn\((\d+)\)/nspawner\((\d+)\)")
    for d in DIRS:
        benches = [f for f in os.scandir(d.path) if f.is_dir() and pattern.match(f.name)]
        print(d.name)
        if len(benches) == 0:
            continue
        data = []
        for bench in benches:
            estp_path = Path(bench.path) / "new" / "estimates.json"
            bpath_path = Path(bench.path) / "new" / "benchmark.json"

            estp = json.load(estp_path.open())
            bpath = json.load(bpath_path.open())
            thr = bpath["throughput"]["Elements"] / estp["mean"]["point_estimate"] * 10 ** 9
            name = bpath["function_id"]
            match = name_pattern.match(name)
            spawn_num = int(match.group(1))
            nspawner = int(match.group(2))
            data.append({
                "spawn_num": spawn_num,
                "nspawner": nspawner,
                "throughput": thr
            })

        df = pd.DataFrame(data)

        filename = path / f"{d.name}_scatterplot.png"
        plt.figure(figsize=(12, 6))
        sns.scatterplot(data=df, x="nspawner", y="spawn_num", size="throughput", sizes=(20, 200), hue="throughput",
                        palette="cool", legend="auto")
        plt.savefig(filename, dpi=300, bbox_inches='tight')
        plt.close()


def run():
    for (bench_name, tsplit) in NAMES:
        path = p.PLOTS_PATH / f"{bench_name}_{tsplit}"

        nsplit = p.NS_SPLIT_LOCAL if "local" in bench_name else p.NS_SPLIT_GLOBAL

        ew.trylog(lambda: plot(bench=bench_name, path=path, tsplit=tsplit, nsplits=nsplit))
    common = p.PLOTS_PATH / "common"
    common.mkdir(mode=0o777, parents=True, exist_ok=True)
    scatter_plots(common)
