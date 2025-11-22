use std::collections::HashMap;
use std::io;
use tokio::io::AsyncBufReadExt;
use crate::supervisor::error::UniverseLookupError;
use crate::universe;
use crate::universe::{UniverseCommand, UniverseHandle, UniverseId};

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

        tokio::io::BufReader::new(tokio::io::stdin()).read_line(&mut line).await?;

        Ok(line.trim_end().to_string())
    }

    async fn activate_input_commands(&mut self, input: String) {
        let command: UniverseCommand = UniverseCommand::from(&input);

        // todo: get universe name

        // todo: new menu: new universe, get existing, send command
        // new universe: enter name
        // get existing: print all

        // send command: request name
        // request command

        // inside command possibly request event.

        // self.supervisor.send_universe_command("Universe 1".to_string(), UniverseCommand::Shutdown).await;
    }

    async fn activate_input_events(&mut self) {

    }

    fn new_universe(&mut self) {
        // todo: input name
        let name = "".to_string();

        self.supervisor.add_new_universe("Universe 1".to_string());
    }
}

pub struct SupervisorHandle {
    existing_universes: HashMap<UniverseId, UniverseHandle>,
    universes_via_name: HashMap<String, UniverseId>,
}

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
        let universe_handle = universe::create_universe_handle();

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