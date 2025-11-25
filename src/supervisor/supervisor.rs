use std::collections::HashMap;
use rgb::RGB8;
use crate::logging::log;
use crate::supervisor::error::UniverseLookupError;
use crate::universe;
use crate::universe::{UniverseCommand, UniverseEvent, UniverseHandle, UniverseId};
use crate::universe::Relationship;
use crate::universe::UniverseIntent;

pub struct SupervisorHandle {
    pub(crate) existing_universes: HashMap<UniverseId, UniverseHandle>,
    pub(crate) universes_via_name: HashMap<String, UniverseId>,
}

impl SupervisorHandle {
    pub fn new() -> SupervisorHandle {
        SupervisorHandle {
            existing_universes: HashMap::new(),
            universes_via_name: HashMap::new(),
        }
    }

    ///------------------------
    /// get universes
    ///------------------------
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

    fn get_universe_name_by_id(
        &self,
        source_id: &UniverseId
    ) -> String {
        self.universes_via_name
            .iter()
            .find_map(|(name, &id)| if id == *source_id { Some(name.clone()) } else { None })
            .unwrap_or(source_id.to_string())
    }

    pub fn get_all_existing_universes(&self) -> Vec<&String> {
        self.universes_via_name.keys().collect()
    }

    ///------------------------
    /// manage from UI
    ///------------------------
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

    ///------------------------
    /// runtime
    ///------------------------

    pub async fn process_intent(&mut self, source_id: UniverseId, intent: UniverseIntent) {
        // get universe name via id
        let source_name = self.get_universe_name_by_id(&source_id);

        match intent {
            UniverseIntent::Attack { target, damage } => {
                self.attack_intent(source_name, target, damage).await;
            }
            UniverseIntent::Heal { target, amount } => {
                self.heal_intent(source_name, target, amount).await;
            }
            UniverseIntent::Dead { target } => {
                self.kill_intent(source_name, target).await;
            }
        }
    }

    /// -----------------
    /// helpers
    /// -----------------
    pub async fn attack_intent(&mut self, source_name: String, target_id: UniverseId, damage: i32) {
        let target_name = self.get_universe_name_by_id(&target_id);

        self.send_universe_command(target_name, UniverseCommand::InjectEvent(UniverseEvent::Shatter(damage))).await;

        // todo: log(format!("[{}] attacked [{}] -{} HP", source_name, target_name, damage), Some(*color));
    }

    pub async fn heal_intent(&mut self, source_name: String, target_id: UniverseId, amount: i32) {
        let target_name = self.get_universe_name_by_id(&target_id);

        self.send_universe_command(target_name, UniverseCommand::InjectEvent(UniverseEvent::Heal(amount))).await;

        // todo: log(format!("[{}] attacked [{}] -{} HP", source_name, target_name, damage), Some(*color));
    }

    pub async fn kill_intent(&mut self, source_name: String, target_id: UniverseId) {
        let target_name = self.get_universe_name_by_id(&target_id);

        self.send_universe_command(target_name, UniverseCommand::InjectEvent(UniverseEvent::Crash)).await;

        // todo: log(format!("[{}] attacked [{}] -{} HP", source_name, target_name, damage), Some(*color));
    }

    pub async fn roll_brothers_enemies_on_new_universe(&mut self, universe_handle: UniverseHandle) {
        if self.existing_universes.len() == 0 {
            // this is the first universe
            return;
        }
        
        // roll enemy or friend on random
        let all_universes_ids = self.existing_universes.keys().cloned();

        // Send command to set relationships (we’ll add this command next)
        for target_id in all_universes_ids {
            if rand::random() {
                // 50/50 enemy or brother
                let _ = universe_handle.commander_tx.try_send(
                    UniverseCommand::SetRelationship(target_id, Relationship::Enemy)
                );
            } else {
                let _ = universe_handle.commander_tx.try_send(
                    UniverseCommand::SetRelationship(target_id, Relationship::Brother)
                );
            }
        }
    }

    /// for when shutting down system
    pub async fn wait_for_all_tasks_to_finish(&mut self) {
        for (_id, universe) in self.existing_universes.drain() {
            let _ = universe.universe_task_handle.await;
        }
    }
}
