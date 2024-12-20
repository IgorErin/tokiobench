import json
import os
import pathlib as lpath
import re
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

import params as p

sns.set_theme()

DIRS: list[os.DirEntry[str]] = [f for f in os.scandir(p.CRITERION_PATH) if f.is_dir()]

def nspawn_nspawner(path: lpath.Path):
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
            sample_path = Path(bench.path) / "new" / "sample.json"

            estp = json.load(estp_path.open())
            min_time = min(json.load(sample_path.open())["times"])
            bpath = json.load(bpath_path.open())
            # thr = bpath["throughput"]["Elements"] / min_time * 10 ** 9
            thr = (bpath["throughput"]["Elements"] / estp["mean"]["point_estimate"]) * 10 ** 9
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

def nspawn_nworker(path: lpath.Path):
    pattern = re.compile(r"nspawn\((\d+)\)_nworker\((\d+)\)")
    name_pattern = re.compile(r"nspawn\((\d+)\)/nworker\((\d+)\)")
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

            thrpt = (bpath["throughput"]["Elements"] / estp["mean"]["point_estimate"]) * 10 ** 9
            name = bpath["function_id"]
            match = name_pattern.match(name)

            ntask = int(match.group(1))
            nworker = int(match.group(2))
            data.append({
                "ntask": ntask,
                "nworker": nworker,
                "thrpt": thrpt
            })

        df = pd.DataFrame(data)

        filename = path / f"{d.name}_scatterplot.png"
        plt.figure(figsize=(12, 6))
        sns.scatterplot(data=df, x="nworker", y="ntask", size="thrpt", sizes=(20, 200), hue="thrpt",
                        palette="cool", legend="auto")
        plt.savefig(filename, dpi=300, bbox_inches='tight')
        plt.close()

def run():
    common = p.PLOTS_PATH / "common"
    common.mkdir(mode=0o777, parents=True, exist_ok=True)

    nspawn_nspawner(common)
    nspawn_nworker(common)
