import os
import pathlib as lpath
import sys

import matplotlib
import numpy as np
import pandas as pd
import seaborn as sns
from statsmodels.nonparametric.smoothers_lowess import lowess

import params as p

N_ITER = 20

SUBNAME = ["local", "current"]
# NAMES = [ f"spawner_{subname}_nwork({nwork})" for nwork in p.NS_WORKERS for subname in SUBNAME ]

NAMES = [f.name for f in os.scandir(p.METRICS_PATH) if f.is_dir()]


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
            x="time_micros", y=label,
        ).savefig(result_path / (label+"scatter"))
        matplotlib.pyplot.close()


def lowess_plot(df, labels: list[str], result_path: lpath.Path, from_i: int = 0, to_i: int = sys.maxsize):
    filtered = df[(df.iteration >= from_i) & (df.iteration <= to_i)]
    for label in labels:
        regplot_lowess_ci(
            data=filtered,
            x="time_micros", y=label, ci_level=95, n_boot=100
        ).figure.savefig(result_path / (label+"lowess"))
        matplotlib.pyplot.close()

def run():
    labels = ["global_queue_depth", "total_steal_count", "total_local_queue_depth", ]
    sns.set_theme()
    for bname in NAMES:
        files = [f for f in os.scandir(p.METRICS_PATH / bname) if f.is_file()]
        for f in files:
            df = pd.read_csv(f.path)
            # Add time to df
            df["time_micros"] = df.groupby('iteration')['elapsed'].cumsum()
            bdir = p.PLOTS_PATH / bname
            bdir.mkdir(mode=0o777, parents=True, exist_ok=True)
            plot_dir = bdir / f.name
            plot_dir.mkdir(mode=0o777, parents=True, exist_ok=True)
            lowess_plot(df, labels, plot_dir)
