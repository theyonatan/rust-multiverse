use std::collections::HashSet;
use std::thread;
use crate::universe::{new_universe_id, UniverseId};

pub struct Universe {
    pub(crate) id: UniverseId,
    executes: bool,
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
            executes: false,
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
