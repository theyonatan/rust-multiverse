use std::collections::HashSet;
use std::thread;
use crate::universe::{new_universe_id, UniverseId};

pub struct Universe {
    pub(crate) id: UniverseId,
    tick: u64,
    state: String,
    enemies: HashSet<UniverseId>,
    brothers: HashSet<UniverseId>,
    log: Vec<String>,
}

impl Universe {
    pub(crate) fn new() -> Universe {
        let id = new_universe_id();

        Universe {
            id,
            tick: 0,
            state: "".to_string(),
            enemies: Default::default(),
            brothers: Default::default(),
            log: vec![],

        }
    }

    pub(crate) fn step(&mut self) {
        thread::sleep(std::time::Duration::from_millis(100));
        self.tick += 1;
        self.state = "after tick".to_string();
        self.log.push(format!("tick-tock {}", self.tick));
    }
}

// todo: supervisor
// a universe manager that main calls
// has a function to add a new universe (gets all other universes, randomly decides if the other universes are friends or enemies)
// send commands to universes via name (I have a hashmap of universes name to id):
// start, pause, inject event(kill, heal, custom event(string), any other acceptable events), request state (basically all UniverseCommands)
//
// receives commands from the user in terminal, runs on it's own thread or probably better: runs on an async task. idk how it's supposed to be done.

// todo: communication logic
// every step a universe sends a single event to a single either a friend or an enemy via random rng.

// todo: split to multiple .rs files
