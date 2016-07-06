#! /usr/bin/env python

import pandas as pd
import matplotlib.pyplot as plt

def __main__(args):
    df = pd.read_csv(args[1], sep='\t')
    df.time = df.time / 1000.0
    df = df[args[2:]]
    plt.figure()
    #df.plot(x=args[2], logy=True, logx=True, figsize=(6, 4.5))
    #df.plot(x=args[2], logy=True, figsize=(6, 4.5))
    #df.plot(x=args[2], figsize=(6, 4.5), ylim=[0, 1200])
    df.plot(x=args[2], figsize=(6, 4.5))
    plt.savefig(args[1] + '.pdf')

import sys

__main__(sys.argv)

