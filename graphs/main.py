#!/usr/bin/env python

import json
import pathlib as lpath
import matplotlib.pyplot as plt
import argparse
import itertools as it
import numpy as np

import params as p

def on_path_or(p: lpath.Path, run, message: str):
    if p.exists():
        run()
    else:
        # logging lib
        print(message)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog='graphpy',
        description='Draw cringe graphs for my diploma',
        epilog="Duty dies last")

    parser.add_argument("-p", "--profile", default="default")
    parser.add_argument("-c", "--criterion", default=p.CRITERION_PATH)
    parser.add_argument("-r", "--result", default=p.PLOTS_PATH)

    args = parser.parse_args()

    p.PLOTS_PATH = lpath.Path(args.result)
    p.CRITERION_PATH = lpath.Path(args.criterion)

    lpath.Path(args.result).mkdir(mode=0o777, parents=True, exist_ok=True)

    plt.figure(figsize=(10,10))

    import scatter

    scatter.run()