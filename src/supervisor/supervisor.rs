use std::collections::HashMap;
use rgb::RGB8;
use crate::supervisor::log_messages::Log;
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

    fn get_color_by_id(&self, id: &UniverseId) -> RGB8 {
        self.existing_universes
            .get(id)
            .map(|h| h.color)
            .unwrap_or(RGB8::new(255, 255, 255))
    }

    ///------------------------
    /// manage from UI
    ///------------------------
    pub async fn add_new_universe(&mut self, name: String) {
        // new universe
        let universe_handle = universe::create_universe_handle(name.to_owned());

        // Log
        Log::created(&name, universe_handle.color);

        // declare brothers or enemies
        self.roll_brothers_enemies_on_new_universe(&universe_handle).await;

        // add to universe db
        (&mut self.universes_via_name).insert(name, universe_handle.handle_id.clone());
        (&mut self.existing_universes).insert(universe_handle.handle_id.clone(), universe_handle);
    }

    pub async fn send_universe_command(&self, universe_name: String, command: UniverseCommand) {
        // get universe
        let universe = match self.get_universe_handle_by_name(&universe_name) {
            Ok(u) => u,
            Err(_e) => {
                // eprintln!("Lookup error: {}", _e);
                return;
            }
        };

        // use universe to send command
        if let Err(_e) = universe.commander_tx.send(command).await {
            // eprintln!("Failed to send command: {}", _e);
        }
    }

    ///------------------------
    /// runtime
    ///------------------------

    pub async fn process_intent(&mut self, source_id: UniverseId, intent: UniverseIntent) {
        match intent {
            UniverseIntent::Attack { target, damage } => {
                self.attack_intent(source_id, target, damage).await;
            }
            UniverseIntent::Heal { target, amount } => {
                self.heal_intent(source_id, target, amount).await;
            }
            UniverseIntent::Dead { target } => {
                self.collapsed_intent(target).await;
            }
        }
    }

    /// -----------------
    /// helpers
    /// -----------------
    pub async fn attack_intent(
        &mut self,
        source_id: UniverseId,
        target_id: UniverseId,
        damage: i32) {
        // log attack
        let source_name = self.get_universe_name_by_id(&source_id);
        let target_name = self.get_universe_name_by_id(&target_id);
        let source_handle = self.existing_universes.get(&source_id).unwrap();
        let target_handle = self.existing_universes.get(&target_id).unwrap();

        Log::attack(&source_name, source_handle.color, &target_name, target_handle.color, damage);

        // send the universe shatter command
        self.send_universe_command(target_name, UniverseCommand::InjectEvent(UniverseEvent::Shatter(damage))).await;
    }

    pub async fn heal_intent(
        &mut self,
        source_id: UniverseId,
        target_id: UniverseId,
        amount: i32) {
        // log attack
        let source_name = self.get_universe_name_by_id(&source_id);
        let target_name = self.get_universe_name_by_id(&target_id);
        let source_handle = self.existing_universes.get(&source_id).unwrap();
        let target_handle = self.existing_universes.get(&target_id).unwrap();

        Log::heal(&source_name, source_handle.color, &target_name, target_handle.color, amount);

        // send the universe heal command
        self.send_universe_command(target_name, UniverseCommand::InjectEvent(UniverseEvent::Heal(amount))).await;
    }

    pub async fn collapsed_intent(
        &mut self,
        target_id: UniverseId,) {
        let target_name = self.get_universe_name_by_id(&target_id);
        let target_handle = self.existing_universes.get(&target_id).unwrap();

        Log::collapsed(&target_name, target_handle.color);

        // broadcast everyone it collapsed
        self.broadcast_collapsed_universe(target_id);

        // send the universe collapse command
        self.send_universe_command(target_name, UniverseCommand::Shutdown).await;

        // remove from own hashmaps
        self.existing_universes.remove(&target_id);
        self.universes_via_name.retain(|_, &mut id| id != target_id);
    }

    fn broadcast_collapsed_universe(&self, collapsed_id: UniverseId) {
        for (id, survivor_handle) in &self.existing_universes {
            if *id != collapsed_id {
                let _ = survivor_handle.commander_tx.try_send(
                    UniverseCommand::InjectEvent(UniverseEvent::UniverseCollapsed(collapsed_id))
                );
            }
        }
    }

    pub async fn roll_brothers_enemies_on_new_universe(&mut self, universe_handle: &UniverseHandle) {
        if self.existing_universes.len() == 0 {
            // this is the first universe
            return;
        }

        // roll enemy or friend on random
        let all_universes_ids: Vec<UniverseId> = self.existing_universes.keys().cloned().collect();

        // Send command to set relationships
        for target_id in all_universes_ids {
            // 50/50 enemy or brother
            if rand::random() {
                self.set_relationship(&universe_handle, target_id, Relationship::Enemy).await;
            } else {
                self.set_relationship(&universe_handle, target_id, Relationship::Brother).await;
            }
        }
    }

    async fn set_relationship(&mut self, universe_handle: &UniverseHandle, target_id: UniverseId, relationship: Relationship) {
        let _ = universe_handle.commander_tx.try_send(
            UniverseCommand::SetRelationship(target_id, relationship)
        );
        let _ = self.get_universe_handle_by_name(self.get_universe_name_by_id(&target_id).as_str()).unwrap()
            .commander_tx.try_send(
            UniverseCommand::SetRelationship(universe_handle.handle_id, relationship)
        );

        // log
        self.log_relationship(universe_handle, target_id, relationship);
    }

    fn log_relationship(&mut self, universe_handle: &UniverseHandle, target_id: UniverseId, relationship: Relationship) {
        let own_name = universe_handle.own_name.clone();
        let target_name = self.get_universe_name_by_id(&target_id);

        let new_color = universe_handle.color;
        let target_color = self.get_color_by_id(&target_id);

        match relationship {
            Relationship::Enemy => {
                Log::relationship_announcement(
                    &target_name, target_color,
                    &own_name, new_color,
                    "ENEMIES",
                    "⚔️ War has been declared",
                );
            }
            Relationship::Brother => {
                Log::relationship_announcement(
                    &target_name, target_color,
                    &own_name, new_color,
                    "BROTHERS",
                    "An alliance forged in the void",
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
