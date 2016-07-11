# Handoff Counters Simulator

Simulates distributed executions of *Handoff Counters* - eventually consistent
distributed counters, as presented in http://arxiv.org/abs/1307.3207


## Usage

Have Rust installed. Then, clone repo:

```
git clone git@github.com:pssalmeida/handoff_counter_simulator-rs.git
```

Change to created dir and build project:

```
cd handoff_counter_simulator-rs
cargo build --release
```

There are 4 executables (`cn1`, `cn2`, `cn3`, `cn4`), which can be invoked as, e.g.,
```
./target/release/cn1 10 100 1000 10 10000 10 1000000 1000
```

Or better, go to the `runs` dir and use `sim.py`, which takes as arguments the
name of one of these executables and its arguments, and redirects the output to
a suitably named file. E.g., invoking:

```
./sim.py cn1 10 100 1000 10 10000 10 1000000 1000
```

will execute `cn1` as above, redirecting the output to file
`cn1-10-100-1000-10-10000-10-1000000-1000`.

In the `runs` dir there are already several files corresponding to some runs,
as well as some other Python scripts to produce plots (through [pandas](http://pandas.pydata.org) and [Matplotlib](http://matplotlib.org/)).


## Scenarios

All 4 executables simulate scenarios with three tiers, with a fixed number of
tier 0 nodes and tier 1 nodes, and a varying number of tier 2 nodes along the
run. We call *clients* to tier 2 nodes and *servers* to tier 1 nodes.

- `cn1` where clients keep affinity to a randomly choosen server;

- `cn2` where clients disconnect abruptly and choose a different random server
  when reconnecting, exchanging messages only with this new server;

- `cn3` as `cn2` but now with more intelligent message exchange, to discard
  unwanted slots;

- `cn4` with client retirement, where clients both arrive and retire at a
    given rate, while keeping a given number of active clients.

These executables take from 8 to 10 arguments:

```
cn1 t0 t1 t2 arrival_period activity_period active_percentage end_time stat_interval
```
```
cn2 t0 t1 t2 arrival_period activity_period active_percentage end_time stat_interval
```
```
cn3 t0 t1 t2 arrival_period activity_period active_percentage handler_period end_time stat_interval
```
```
cn4 t0 t1 t2 arrival_period activity_period active_percentage handler_period partition_probability end_time stat_interval
```

These arguments determine some simulation parameters:

- `t0`: number of tier 0 nodes;
- `t1`: number of tier 1 nodes;
- `t2`: starting number of tier 2 nodes;
- `arrival_period`: one new tier 2 node arrives each `arrival_period` miliseconds;
- `activity_period`: time for each period of client behavior;
- `active_percentage`: percentage of the `activity_period` that a client is active;
- `handler_period`: period of the invocations of the periodic handlers at both
  clients and servers, that send messages to appropriate peers;
- `partition_probability`: probability that a client is partitioned when it
  decides to retire;
- `end_time`: time at which the simulation ends;
- `stat_interval`: statistics are gathered each `stat_interval` miliseconds;

