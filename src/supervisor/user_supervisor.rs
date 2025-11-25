use crate::supervisor::supervisor::SupervisorHandle;
use crate::universe::{UniverseCommand, UniverseId, UniverseIntent};

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

    // --- Helpers ---
    pub(crate) async fn new_universe(&mut self, name: String) {
        self.supervisor.add_new_universe(name.clone());
    }

    pub(crate) fn get_list_universes(&self) -> Vec<&String> {
        self.supervisor.get_all_existing_universes()
    }

    async fn shut_down_all(&mut self) {
        for (universe_name, _universe) in self.supervisor.universes_via_name.iter() {
            self.supervisor.send_universe_command(universe_name.clone(), UniverseCommand::Shutdown).await;
        }
    }

    /// gets called in the main loop, this is the supervisor acting as a server,
    /// checking for incoming messages (intents) from the universes and processing them.
    async fn process_universe_events(&mut self) {
        // collect all intents
        let mut pending_intents: Vec<(UniverseId, UniverseIntent)> = Vec::new();

        // Drain pending intents from all universes
        for handle in self.supervisor.existing_universes.values_mut() {
            if let Ok(intent) = handle.intent_rx.try_recv() {
                pending_intents.push((handle.handle_id, intent));
            }
        }

        // process all intents
        for (source_id, intent) in pending_intents {
            self.supervisor.process_intent(source_id, intent).await;
        }
    }
}