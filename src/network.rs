
use std::cmp::{min, max};
use std::collections::HashMap;
use rand::{Rng, ThreadRng, thread_rng};

use devesim::distributions::weibull;


pub type Id = usize;
pub type Counter = ::handoff_counter::Counter<Id>;

pub struct Global {
    pub rng : ThreadRng,
    pub time: u64,
    tier0: usize,
    tier1: usize,
}

impl Global {

    pub fn latency(&mut self, tier_a: usize, tier_b: usize) -> u64 {
        (match (min(tier_a, tier_b), max(tier_a, tier_b)) {
            (0, 0) => 50.0 + weibull(&mut self.rng, 50.0, 2.0),
            (0, 1) => 25.0 + weibull(&mut self.rng, 25.0, 2.0),
            (1, 1) => 25.0 + weibull(&mut self.rng, 25.0, 2.0),
            (1, 2) => 25.0 + weibull(&mut self.rng, 25.0, 2.0),
            (_, _) => panic!("invalid communication"),
        }) as u64
    }

    pub fn tier(&self, id: Id) -> usize {
        if id <= self.tier0 { 0 }
        else if id <= self.tier0 + self.tier1 { 1 }
        else { 2}
    }

    pub fn choose_server(&mut self) -> Id {
        self.rng.gen_range(self.tier0 + 1, self.tier0 + self.tier1 + 1)
    }

    pub fn is_server(&self, id: Id) -> bool { id <= self.tier0 + self.tier1 }

}

pub trait Node where Self: Sized {
    type NodeEvent;
    fn init(&mut self, global: &mut Global) -> Vec<Event<Self>>;
    fn handle(&mut self, event: Self::NodeEvent, global: &mut Global)
        -> Vec<Event<Self>>;
}

pub enum EventData<N: Node> {
    NodeEvent(Id, N::NodeEvent),
    Function(Box<Fn(&mut State<N>) -> Vec<Event<N>>>),
    Periodic(u64, Box<Fn(&mut State<N>) -> Vec<Event<N>>>),
    NodePeriodic(Id, u64, Box<Fn(&mut N, &mut Global) -> Vec<Event<N>>>),
    RetireNode(Id),
}

pub struct Event<N: Node> {
    pub time: u64,
    pub data: EventData<N>,
}

impl<N: Node> ::devesim::Event for Event<N> {
    fn time(&self) -> u64 { self.time }
}

pub struct State<N: Node> {
    pub nodes: HashMap<Id, N>,
    pub retired: HashMap<Id, N>,
    next_id : Id,
    pub t0ids : Vec<Id>,
    pub t1ids : Vec<Id>,
    pub t2ids : Vec<Id>,
    pub global: Global,
    new_node: Box<Fn(Id, usize, Vec<Id>) -> N>,
}

impl<N: Node> State<N> {
    pub fn new(tier0: usize, tier1: usize, tier2: usize,
               new_node: Box<Fn(Id, usize, Vec<Id>) -> N>) -> State<N> {
        let mut rng = thread_rng();
        let mut nodes = HashMap::new();

        let t0ids : Vec<Id> = (1..tier0+1).collect();
        for &id in &t0ids {
            let mut v = t0ids.clone();
            v.remove(id-1);
            nodes.insert(id, new_node(id, 0, v));
        }

        let mut id = tier0+1;
        let mut t1ids : Vec<Id> = Vec::with_capacity(tier1);
        for _ in 0..tier1 {
            t1ids.push(id);
            nodes.insert(id, new_node(id, 1, vec![*rng.choose(&t0ids).unwrap()]));
            id += 1;
        }

        let mut t2ids : Vec<Id> = Vec::with_capacity(tier2);
        for _ in 0..tier2 {
            t2ids.push(id);
            nodes.insert(id, new_node(id, 2, vec![*rng.choose(&t1ids).unwrap()]));
            id += 1;
        }

        State {
            next_id : id,
            nodes: nodes,
            retired: HashMap::new(),
            t0ids: t0ids,
            t1ids: t1ids,
            t2ids: t2ids,
            global: Global { time: 0, rng: rng, tier0: tier0, tier1: tier1 },
            new_node: new_node,
        }
    }

    pub fn init(&mut self) -> Vec<Event<N>> {
        let mut events = vec![];
        for (_, node) in self.nodes.iter_mut() {
            let evs = node.init(&mut self.global);
            events.extend(evs);
        }
        events
    }

    pub fn new_node(&mut self) -> Vec<Event<N>> {
        let peers = vec![*self.global.rng.choose(&self.t1ids).unwrap()];
        let id = self.next_id;
        self.next_id += 1;
        self.t2ids.push(id);
        let mut node = (self.new_node)(id, 2, peers);
        let events = node.init(&mut self.global);
        self.nodes.insert(id, node);
        events
    }

}

impl<N: Node> ::devesim::State<Event<N>> for State<N> {

    fn handle(&mut self, event: Event<N>) -> Vec<Event<N>> {
        self.global.time = event.time;
        match event.data {
            EventData::NodeEvent(id, node_event) => {
                match self.nodes.get_mut(&id) {
                    Some(node) => node.handle(node_event, &mut self.global),
                    None => vec![]
                }
            }
            EventData::Function(f) => {
                f(self)
            }
            EventData::Periodic(time, f) => {
                if time == 0 { return vec![]; }
                let mut events = f(self);
                let e = EventData::Periodic(time, f);
                events.push(Event{time: event.time + time, data: e});
                events
            }
            EventData::NodePeriodic(id, time, f) => {
                if time == 0 { return vec![]; }
                match self.nodes.get_mut(&id) {
                    Some(node) => {
                        let mut events = f(node, &mut self.global);
                        let e = EventData::NodePeriodic(id, time, f);
                        events.push(Event{time: event.time + time, data: e});
                        events
                    }
                    None => vec![]
                }
            }
            EventData::RetireNode(id) => {
                let node = self.nodes.remove(&id).unwrap();
                self.retired.insert(id, node);
                vec![]
            }
        }
    }

}

