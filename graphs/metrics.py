import os
import pathlib as lpath
import re
import sys

import matplotlib
import numpy as np
import pandas as pd
import seaborn as sns
from statsmodels.nonparametric.smoothers_lowess import lowess

import params as p

NAMES = [f for f in os.scandir(p.METRICS_PATH) if f.is_dir()]


# https://github.com/mwaskom/seaborn/issues/552#issuecomment-1668374877
def regplot_lowess_ci(data, x, y, ci_level, n_boot, **kwargs):
    x_ = data[x].to_numpy()
    y_ = data[y].to_numpy()
    x_grid = np.linspace(start=x_.min(), stop=x_.max(), num=1000)

    def reg_func(_x, _y):
        return lowess(exog=_x, endog=_y, xvals=x_grid, frac=0.5)

    beta_boots = sns.algorithms.bootstrap(
        x_, y_,
        func=reg_func,
        n_boot=n_boot,
    )
    err_bands = sns.utils.ci(beta_boots, ci_level, axis=0)
    y_plt = reg_func(x_, y_)

    ax = sns.lineplot(x=x_grid, y=y_plt, color='red', **kwargs)
    sns.scatterplot(x=x_, y=y_, ax=ax, **kwargs)
    ax.fill_between(x_grid, *err_bands, alpha=.15, **kwargs)
    return ax


def scatter_plot(df, labels: list[str], result_path: lpath.Path, from_i: int = 0, to_i: int = sys.maxsize):
    filtered = df[(df.iteration >= from_i) & (df.iteration <= to_i)]
    for label in labels:
        sns.relplot(
            data=filtered,
            x="time_nanos", y=label,
        ).savefig(result_path / (label+"scatter"))
        matplotlib.pyplot.close()


def lowess_plot(df, labels: list[str], result_path: lpath.Path, from_i: int = 0, to_i: int = sys.maxsize):
    filtered = df[(df.iteration >= from_i) & (df.iteration <= to_i)]
    for label in labels:
        regplot_lowess_ci(
            data=filtered,
            x="time_nanos", y=label, ci_level=95, n_boot=100
        ).figure.savefig(result_path / (label+"lowess"))
        print("saved")
        matplotlib.pyplot.close()

def merge_iters(base_dir):
    all_data = []
    if os.path.isdir(base_dir):
        for file_name in os.listdir(base_dir):
            if file_name.endswith('.csv'):
                file_path = os.path.join(base_dir, file_name)
                iteration_match = re.search(r'iter\((\d+)\)\.csv', file_name)

                if iteration_match:
                    iteration = int(iteration_match.group(1))
                    print(iteration)
                    df = pd.read_csv(file_path)
                    df['iteration'] = iteration
                    all_data.append(df)

    if all_data:
        result_df = pd.concat(all_data, ignore_index=True)
        return result_df
    else:
        return pd.DataFrame()

def process_df(df, plot_dir):
    print("runned")
    labels = ["global_queue_depth", "total_steal_count", "total_local_queue_depth", ]
    df["time_nanos"] = df.groupby('iteration')['elapsed'].cumsum()
    print("grouped")
    print(df.shape)
    lowess_plot(df, labels, plot_dir)
    print("finished")

def run():

    spawner_pattern = re.compile(r"sampling\((.*?)\)_nspawn\((\d+)\)_nworkers\((\d+)\)")
    workload_pattern = re.compile(r"sampling\((.*?)\)_nspawn\((\d+)\)_nspawner\((\d+)\)")
    sns.set_theme()
    for bname in NAMES:
        print(bname)
        match = spawner_pattern.match(bname.name)
        if not match:
            match = workload_pattern.match(bname.name)
        if not match or not os.path.isdir(bname):
            continue
        sampling = match.group(1)
        nspawn = match.group(2)
        nworkers = match.group(3)
        print("matched")
        bdir = p.PLOTS_PATH / sampling
        bdir.mkdir(mode=0o777, parents=True, exist_ok=True)
        plot_dir = bdir / f"ns({nspawn})nw({nworkers})"
        plot_dir.mkdir(mode=0o777, parents=True, exist_ok=True)
        df = merge_iters(bname)
        process_df(df, plot_dir)

if __name__=="__main__":
    run()

