#! /usr/bin/env python

import pandas as pd
import matplotlib.pyplot as plt

def __main__(args):
    df1 = pd.read_csv(args[1], sep='\t')
    df1 = df1[["time", "ids", "slots"]]
    df2 = pd.read_csv(args[2], sep='\t')
    df2 = df2[["time", "slots"]]
#    df3.rename(columns=lambda x: "time" if x == "time" else x + " 100%", inplace=True)
    df4 = pd.merge(df1, df2, on=["time"], suffixes=[" (1%)", " (10%)"])
    df4.time = df4.time / 1000.0
    plt.figure()
    df4.plot(x="time", logy=True)
    #df4.plot(x="time", ylim=[0, 1200])
    plt.savefig(args[3] + '.pdf')


import sys

__main__(sys.argv)

