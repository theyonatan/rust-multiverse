use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use tokio::sync::mpsc::{Receiver, Sender};

static NEXT_UNIVERSE_ID: AtomicUsize = AtomicUsize::new(1);
pub type UniverseId = usize;

#[derive(Debug)]
enum UniverseEvent {
    ChangeState(String),
    Shatter,                // damage the universe a bit
    Crash(UniverseId),      // the universe is damaged so bad, it crashed. UniverseId must be "Enemy" for this to apply.
    Ping(UniverseId),       // just sends a ping, expect a pong
    Pong(UniverseId),       // after a ping, this is the response
    Shutdown(UniverseId),   // same as crash but via force, UniverseId must be "brother" for this to apply.
}

#[derive(Debug)]
pub enum UniverseCommand {
    Start,
    Stop,
    InjectEvent(UniverseEvent), // TODO: Supervisor sends an event to the Universe which he unpacks and deals with.
    RequestState(), // TODO: Supervisor sends a request to get an event, the Universe somehow returns a response.
}



pub struct UniverseHandle {
    universe: Arc<Mutex<Universe>>,
    pub(crate) commander: std::sync::mpsc::SyncSender<UniverseCommand>,
    events_receiver: std::sync::mpsc::Receiver<UniverseEvent>,
    pub(crate) universe_thread: std::thread::JoinHandle<()>,
}

impl UniverseHandle {
    fn new(universe: Universe) -> UniverseHandle {
        let (apply_universe_command_tx, apply_universe_command_rx) = std::sync::mpsc::sync_channel::<UniverseCommand>(10);
        let (universe_to_router_tx, router_from_universe_rx) = std::sync::mpsc::channel::<UniverseEvent>();

        let rx = Arc::new(Mutex::new(apply_universe_command_rx));
        let universe = Arc::new(Mutex::new(universe));

        let handle = thread::spawn({
            println!("Spawned universe thread");
            let rx = Arc::clone(&rx);
            let universe = Arc::clone(&universe);

            move || loop {
                // TODO: loop with tokio select over: command_rx and event_rx
                
                let given_command = rx.lock().unwrap().recv().unwrap();
                println!("Given universe command: {:?}", given_command);

                match given_command {
                    UniverseCommand::Start => {

                    }
                    UniverseCommand::Stop => {
                        return;
                    }
                    UniverseCommand::InjectEvent(event) => {

                    }
                    UniverseCommand::RequestState() => {

                    }
                }

                universe.lock().unwrap().step();
            }
        });

        UniverseHandle {
            universe,
            commander: apply_universe_command_tx,
            events_receiver: router_from_universe_rx,
            universe_thread: handle,
        }
    }
}

pub fn create_universe() -> UniverseHandle {
    let universe = Universe::new();

    UniverseHandle::new(universe)
}

struct Universe {
    id: UniverseId,
    tick: u64,
    state: String,
    enemies: HashSet<UniverseId>,
    brothers: HashSet<UniverseId>,
    log: Vec<String>,
}

impl Universe {
    fn new() -> Universe {
        let id = NEXT_UNIVERSE_ID.fetch_add(1, Ordering::Relaxed);

        Universe {
            id,
            tick: 0,
            state: "".to_string(),
            enemies: Default::default(),
            brothers: Default::default(),
            log: vec![],
            
            
        }
    }

    fn step(&mut self) {
        thread::sleep(std::time::Duration::from_millis(100));
        self.tick += 1;
        self.state = "after tick".to_string();
        self.log.push(format!("tick-tock {}", self.tick));
    }
}
