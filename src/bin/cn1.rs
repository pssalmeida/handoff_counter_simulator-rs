extern crate handoff_counter;
extern crate devesim;
extern crate rand;
extern crate roaring;
extern crate counter_simulator;


//use rand::{Rng};
//use std::cmp::{min, max};
use roaring::RoaringBitmap;

use devesim::{Simulator};

type Bitmap = RoaringBitmap<u32>;

use counter_simulator::network;
use counter_simulator::network::{Id, Counter, Event, EventData, State, Global};

struct Node {
    counter : Counter,
    peers: Vec<Id>,
    incrs : u64,
    bitmap : Bitmap,
    active: bool,
    active_time: u64,
    inactive_time: u64,
}

enum NodeEvent {
    Activate,
    Deactivate,
    Msg(Counter, Bitmap),
}

use NodeEvent::*;

impl Node {

    pub fn send(&self, id2: Id, global: &mut Global) -> Event<Self> {
        let c = &self.counter;
        let bm = &self.bitmap;
        let tier2 = global.tier(id2);
        let c3 = c.view(id2, tier2);
        let bm3 = if tier2 < 2 { bm.clone() } else { Bitmap::new() };
        Event {
            time: global.time + global.latency(c.tier(), tier2),
            data: EventData::NodeEvent(id2, NodeEvent::Msg(c3, bm3)),
        }
    }

    fn event(&self, time: u64, node_event: NodeEvent) -> Event<Self> {
        Event {
            time: time,
            data: EventData::NodeEvent(self.counter.id(), node_event),
        }
    }

    fn ids_len(&self) -> usize { self.bitmap.len() as usize }
}

impl network::Node for Node {
    type NodeEvent = NodeEvent;

    fn init(&mut self, global: &mut Global) -> Vec<Event<Self>> {
        self.incrs += 1;
        self.counter.incr();
        let mut events = vec![];
        if self.counter.tier() < 2 {
            for &peer in &self.peers {
                let e = self.send(peer, global);
                events.push(e);
            }
        } else {
            let delta = 0; //global.rng.gen_range(0, 1000);
            events.push(self.event(global.time + delta, Activate));
        }
        events
    }

    fn handle(&mut self, event: NodeEvent, global: &mut Global) ->
                      Vec<Event<Self>> {
        match event {
            Msg(c2, bm2) => {
//if c2.tier() == 2 && self.counter.tier() == 1 { println!("received: {:?}", c2); }
                if self.active {
                    self.counter.incr();
                    self.incrs += 1;
                }
                self.counter.merge(&c2);
//if c2.tier() == 2 && self.counter.tier() == 1 { println!("merged:{:?}", self.counter); }
                if self.counter.tier() < 2 {
                    self.bitmap.union_with(&bm2);
                }
                if self.counter.tier() < 2 || self.counter.needs_to_handoff() {
                    vec![self.send(c2.id(), global)]
                } else {
                    vec![]
                }
            }
            Deactivate => {
                self.active = false;
                vec![self.event(global.time + self.inactive_time, Activate)]
            }
            Activate => {
                self.active = true;
                self.counter.incr();
                self.incrs += 1;
                let mut events = vec![self.send(self.peers[0], global)];
                if self.inactive_time != 0 {
                    events.push(self.event(global.time + self.active_time, Deactivate));
                }
                events
            }
        }
    }
}

fn statistics(state: &mut State<Node>) -> Vec<Event<Node>> {
    let mut ids_sum = 0;
    let mut slots_sum = 0;
    let mut active = 0;
    for node in state.nodes.values() {
        let c = &node.counter;
        if c.tier() == 1 {
            slots_sum += c.slots().len();
            ids_sum += node.ids_len();
        }
        if node.active { active += 1; }
    }
    let t1s = state.t1ids.len();
    let t2s = state.t2ids.len();
    let slots_t1s = slots_sum as f64 / t1s as f64;
    if state.global.time == 0 {
        println!("time\tclients\tactive\tids\tslots");
    } else {
    println!("{}\t{}\t{}\t{}\t{}",
             state.global.time, t2s, active, ids_sum / t1s, slots_t1s);
    }
    vec![]
}

struct Args {
    t0: usize,
    t1: usize,
    t2: usize,
    arrival_period: u64,
    activity_period: u64,
    active_percentage: u64,
    end_time: u64,
    stat_interval : u64,
}

fn run(args: &Args) {
    let percentage = args.active_percentage;
    let period = args.activity_period;
    let new_node = move |id: Id, tier: usize, peers: Vec<Id>| -> Node {
        Node {
            counter : Counter::new(id, tier),
            peers: peers,
            incrs : 0,
            bitmap: [id as u32].into_iter().collect(),
            active: false,
            active_time: percentage * period / 100,
            inactive_time: (100 - percentage) * period / 100,
        }
    };
    let mut state: State<Node> =
        State::new(args.t0, args.t1, args.t2, Box::new(new_node));
    let events = state.init();
    let mut sim = Simulator::new(state, events);
    let ed = EventData::Periodic(args.stat_interval, Box::new(statistics));
    sim.push_event(Event { time: 0, data: ed });
    let ed = EventData::Periodic(args.arrival_period, Box::new(State::new_node));
    sim.push_event(Event { time: 0, data: ed });
    sim.run(args.end_time);
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    run(&Args {
        t0: args[1].parse().unwrap(),
        t1: args[2].parse().unwrap(),
        t2: args[3].parse().unwrap(),
        arrival_period: args[4].parse().unwrap(),
        activity_period: args[5].parse().unwrap(),
        active_percentage: args[6].parse().unwrap(),
        end_time: args[7].parse().unwrap(),
        stat_interval: args[8].parse().unwrap(),
    });
}

