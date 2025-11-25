use std::collections::HashMap;
use crate::supervisor::menu_items::*;
use crate::supervisor::error::UniverseLookupError;
use crate::universe;
use crate::universe::{UniverseCommand, UniverseHandle, UniverseId, UniverseEvent};

pub struct UserSupervisor {
    pub(crate) supervisor: SupervisorHandle,
}

impl UserSupervisor {
    pub fn new() -> Self {
        // todo: change good morning to be good - anything
        println!("Good Morning - Supervisor Initialized.");

        UserSupervisor {
            supervisor: SupervisorHandle::new(),
        }
    }

    // --- Sub-Menus ---
    async fn handle_manage_universe(&mut self) {
        let name = menu_request_universe_name("Manage").await;
        if name.is_empty() { return; }

        // Verify existence before entering loop
        if self.supervisor.get_universe_handle_by_name(&name).is_err() {
            // println!("Universe '{}' not found.", name);
            return;
        }

        loop {
            print_command_menu(&name);
            let input = match menu_read_input().await {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Special navigation commands
            match input.trim().to_lowercase().as_str() {
                "back" => break,
                "event" | "do_event" => {
                    self.handle_event_menu(&name).await;
                    continue;
                }
                _ => {} // Fall through to standard command parsing
            }

            // Use the existing From<&String> implementation
            // This handles Start, Stop, Shutdown, RequestState
            let command = UniverseCommand::from(&input);

            match command {
                UniverseCommand::UnknownCommand => {
                    // println!("Unknown command. Type 'Start', 'Stop', 'Event', 'State', or 'Shutdown'.");
                }
                UniverseCommand::Shutdown => {
                    // Send shutdown and exit this menu because the universe will be gone
                    self.supervisor.send_universe_command(name.clone(), command).await;

                    let id = self.supervisor.get_universe_handle_by_name(&name).unwrap().handle_id;
                    self.supervisor.universes_via_name.remove(&name);
                    self.supervisor.existing_universes.remove(&id);

                    // println!("Exiting management for '{}'", name);
                    break;
                }
                _ => {
                    self.supervisor.send_universe_command(name.clone(), command).await;
                }
            }
        }
    }



    async fn handle_event_menu(&self, name: &str) {
        loop {
            print_event_menu(name);
            let input = match menu_read_input().await {
                Ok(s) => s,
                Err(_) => continue,
            };

            let trimmed = input.trim();

            // Navigation
            if trimmed.eq_ignore_ascii_case("back") {
                break;
            }

            // Special Case: ChangeState requires specific input that the "From" impl
            // in universe_event.rs doesn't capture separately (it just clones the command name).
            // We handle it manually here for better UX.
            if trimmed.eq_ignore_ascii_case("changestate") {
                // println!("Enter new state string:");
                let state_val = menu_read_input().await.unwrap_or_default();
                let event = UniverseEvent::ChangeState(state_val);
                self.supervisor.send_universe_command(name.to_string(), UniverseCommand::InjectEvent(event)).await;
                continue;
            }

            // General Case: Use existing From implementation
            // This handles Shatter, Crash, Heal, Ping, Pong
            let event = UniverseEvent::from(&input);

            match event {
                UniverseEvent::Empty => println!("Invalid event name."),
                _ => {
                    let command = UniverseCommand::InjectEvent(event);
                    self.supervisor.send_universe_command(name.to_string(), command).await;
                }
            }
        }
    }

    // --- Helpers ---
    pub(crate) async fn new_universe(&mut self, name: String) {
        self.supervisor.add_new_universe(name.clone());
        // println!("Universe '{}' created.", name);
    }

    pub(crate) fn handle_list_universes(&self) {
        // println!("\n--- Existing Universes ---");
        let universes = self.supervisor.get_all_existing_universes();
        if universes.is_empty() {
            // println!("(No universes found)");
        } else {
            for name in universes {
                // println!("- {}", name);
            }
        }
    }

    async fn shut_down_all(&mut self) {
        for (universe_name, _universe) in self.supervisor.universes_via_name.iter() {
            self.supervisor.send_universe_command(universe_name.clone(), UniverseCommand::Shutdown).await;
        }
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
                // eprintln!("Lookup error: {}", e);
                return;
            }
        };

        // use universe to send command
        if let Err(e) = universe.commander_tx.send(command).await {
            // eprintln!("Failed to send command: {}", e);
        }
    }

    pub async fn wait_for_all_tasks_to_finish(&mut self) {
        for (_id, universe) in self.existing_universes.drain() {
            let _ = universe.universe_task_handle.await;
        }
    }
}
