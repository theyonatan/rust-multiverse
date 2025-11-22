use std::str::FromStr;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fmt, thread};
use std::process::Command;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::io::{self, AsyncBufReadExt};

static NEXT_UNIVERSE_ID: AtomicUsize = AtomicUsize::new(1);
pub type UniverseId = usize;

#[derive(Debug)]
enum UniverseEvent {
    Empty,
    ChangeState(String),
    Shatter,                // damage the universe a bit
    Crash(UniverseId),      // the universe is damaged so bad, it crashed. UniverseId must be "Enemy" for this to apply.
    Ping(UniverseId),       // just sends a ping, expect a pong
    Pong(UniverseId),       // after a ping, this is the response
    Shutdown(UniverseId),   // same as crash but via force, UniverseId must be "brother" for this to apply.
}


#[derive(Debug)]
pub enum UniverseCommand {
    Start, // Resume
    Stop, // Pause
    InjectEvent(UniverseEvent), // TODO: Supervisor sends an event to the Universe which he unpacks and deals with.
    RequestState(), // TODO: Supervisor sends a request to get an event, the Universe somehow returns a response.
    Shutdown, // Shuts down entirely
    UnknownCommand,
}

impl From<&String> for UniverseCommand {
    fn from(s: &String) -> Self {
        match s.to_lowercase().as_str() {
            "start" => UniverseCommand::Start,
            "stop" => UniverseCommand::Stop,
            "do_event" | "event" | "do" => UniverseCommand::InjectEvent(UniverseEvent::Empty),
            "request_state" | "state" | "request" => UniverseCommand::RequestState(),
            "shutdown" => UniverseCommand::Shutdown,
            _ => UniverseCommand::UnknownCommand,
        }
    }
}
impl From<&String> for UniverseEvent {
    fn from(s: &String) -> Self {
        match s.to_lowercase().as_str() {
            "ChangeState" => UniverseEvent::ChangeState(s.clone()),
            "Shatter" => UniverseEvent::Shatter,
            "Crash" => UniverseEvent::Crash(0),
            "Ping" => UniverseEvent::Ping(0),
            "Pong" => UniverseEvent::Pong(0),
            "Shutdown" => UniverseEvent::Shutdown(0),
            _ => UniverseEvent::Empty,
        }
    }
}

#[derive(Debug)]
pub enum UniverseLookupError {
    IdNotFoundForName(String),
    UniverseNotFoundForId(UniverseId),
}

pub struct SupervisorHandle {
    existing_universes: HashMap<UniverseId, UniverseHandle>,
    universes_via_name: HashMap<String, UniverseId>,
}

impl fmt::Display for UniverseLookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UniverseLookupError::IdNotFoundForName(name) => {
                write!(f, "id not found for name: {}", name)
            }

            UniverseLookupError::UniverseNotFoundForId(id) => {
                write!(f, "universe not found for id: {}", id)
            }
        }
    }
}

impl std::error::Error for UniverseLookupError {}

impl SupervisorHandle {
    pub fn new() -> SupervisorHandle {
        SupervisorHandle {
            existing_universes: HashMap::new(),
            universes_via_name: HashMap::new(),
        }
    }

    fn get_universe_handle_by_name(
        &self,
        name: &str
    ) -> Result<&UniverseHandle, UniverseLookupError> {
        let universe_id = self
            .universes_via_name
            .get(name)
            .ok_or_else(|| UniverseLookupError::IdNotFoundForName(name.to_string()))?;

        self.existing_universes
            .get(universe_id)
            .ok_or_else(|| UniverseLookupError::UniverseNotFoundForId(universe_id.clone()))
    }

    pub fn get_all_existing_universes(&self) -> Vec<&String> {
        self.universes_via_name.keys().collect()
    }
    pub fn add_new_universe(&mut self, name: String) {
        // new universe
        let universe_handle = create_universe();

        // add to universe db
        (&mut self.universes_via_name).insert(name, universe_handle.handle_id.clone());
        (&mut self.existing_universes).insert(universe_handle.handle_id.clone(), universe_handle);
    }

    pub async fn send_universe_command(&self, universe_name: String, command: UniverseCommand) {
        // get universe
        let universe = match self.get_universe_handle_by_name(&universe_name) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Lookup error: {}", e);
                return;
            }
        };

        // use universe to send command
        if let Err(e) = universe.commander_tx.send(command).await {
            eprintln!("Failed to send command: {}", e);
        }
    }

    pub async fn wait_for_all_tasks_to_finish(&mut self) {
        for (_id, universe) in self.existing_universes.drain() {
            let _ = universe.universe_task_handle.await;
        }
    }
}

pub struct UserSupervisor {
    supervisor: SupervisorHandle,
}

impl UserSupervisor {
    pub fn new() -> Self {
        UserSupervisor {
            supervisor: SupervisorHandle::new(),
        }
    }

    pub async fn main_loop(&mut self) {
        // todo: create supervisor

        // todo: collect input from user
        loop {
            let should_shutdown = self.ask_for_input_commands().await;

            if should_shutdown {
                break;
            }

            self.supervisor.send_universe_command("Universe 1".to_string(), UniverseCommand::Shutdown).await;


            // todo: match input to different actions
            // _ => bad input (same for events)

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        self.supervisor.wait_for_all_tasks_to_finish().await;
    }

    async fn ask_for_input_commands(&mut self) -> bool {
        println!("Enter universe command:");
        println!("Start");
        println!("Stop");
        println!("do_event");
        println!("request_state");
        println!("Shutdown");

        let input = match self.read_input().await {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Error reading input: {}", e);

                // it's ok, just recollect input again.
                return false;
            }
        };

        self.activate_input_commands(input).await;

        false
    }

    async fn read_input(&mut self) -> Result<String, io::Error> {
        let mut line = String::new();

        io::BufReader::new(io::stdin()).read_line(&mut line).await?;

        Ok(line.trim_end().to_string())
    }

    async fn activate_input_commands(&mut self, input: String) {
        let command: UniverseCommand = UniverseCommand::from(&input);


    }

    async fn activate_input_events(&mut self) {

    }

    fn new_universe(&mut self) {
        // todo: input name
        let name = "".to_string();

        self.supervisor.add_new_universe("Universe 1".to_string());
    }
}

pub struct UniverseHandle {
    pub(crate) handle_id: UniverseId,
    pub(crate) commander_tx: Sender<UniverseCommand>,
    pub(crate) universe_task_handle: tokio::task::JoinHandle<()>,
}

impl UniverseHandle {
    fn new(mut universe: Universe) -> UniverseHandle {
        let handle_id = universe.id.clone();

        let (commander_tx, mut command_rx) = tokio::sync::mpsc::channel::<UniverseCommand>(10);
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<UniverseEvent>();

        let universe_task_handle = tokio::spawn(async move{
            println!("Spawned universe async instance!");

            loop {
                tokio::select! {
                    Some(command) = command_rx.recv() => {
                        println!("Given universe command: {:?}", command);
                        match command {
                            UniverseCommand::Start => {
                                // todo - universe starts doing .step only if universe.activated
                                // this starts the universe
                            }
                            UniverseCommand::Stop => {
                                // todo - this just does universe.activated = false
                            }
                            UniverseCommand::InjectEvent(event) => {
                                // todo, new function to run universe event each step and this changes the event
                                // or pushes event to an event queue
                                // default event after event is finished should be somehting like "default"
                                // or "TickEvent".
                            }
                            UniverseCommand::RequestState() => {
                                // todo: logger thread
                                // should push a message to a queue that goes to the output stream
                                // or log, which a logger then logs on a different thread.
                                // good usage for threads here, use a logger, this pushes to logger,
                                // on a different thread a logger exists and prints all from queue.
                            }
                            UniverseCommand::Shutdown => {
    tokio::time::sleep(std::time::Duration::from_millis(4000)).await;

                                return;
                                // todo:
                                // better shutdown, although this might really be best.
                                // maybe a shutdown function which does universe.shut_down_next_tick = true;
                                // then later bellow actually shut down
                                // and with that function, I can also clean up stuff if I need to.
                            }
                            UniverseCommand::UnknownCommand => {
                                
                            }
                        }
                    }

                    Some(event) = event_rx.recv() => {
                        // todo: universe events
                        // think about different events in universe,
                        // probably damage enemies and heal brothers
                    }
                }

                (&mut universe).step();
            }
        });

        UniverseHandle {
            handle_id,
            commander_tx,
            universe_task_handle,
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
