#! /usr/bin/env python

import subprocess

def __main__(args):
    cmd = "../target/release/" + args[1]
    out = "-".join(args[1:])
    ex_cmd = cmd + " " + " ".join(args[2:]) + " > " + out
    subprocess.call(ex_cmd, shell=True)

import sys

__main__(sys.argv)

