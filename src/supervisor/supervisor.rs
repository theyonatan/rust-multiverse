use std::collections::HashMap;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel, Sender};
use crate::universe::universe_event::UniverseOutboundEvent;
use crate::universe::{self, UniverseCommand, UniverseEvent, UniverseHandle, UniverseId};
use crate::universe::logger::LogMessage;

pub struct SupervisorHandle {
    existing_universes: HashMap<UniverseId, UniverseHandle>,
    universes_via_name: HashMap<String, UniverseId>,

    universe_hp: HashMap<UniverseId, i32>,
    
    global_tx: UnboundedSender<UniverseOutboundEvent>,
    _global_tx: UnboundedSender<UniverseOutboundEvent>, // keep alive
    global_rx: Option<UnboundedReceiver<UniverseOutboundEvent>>,

    // Channel to send logs back to the UI
    logger_tx: UnboundedSender<LogMessage>,

    // Internal Router
    router_tx: Option<UnboundedSender<(UniverseId, Sender<UniverseCommand>)>>,
}

impl SupervisorHandle {
    pub fn new(logger_tx: UnboundedSender<LogMessage>) -> SupervisorHandle {
        let (global_tx, global_rx) = unbounded_channel();
        SupervisorHandle {
            existing_universes: HashMap::new(),
            universes_via_name: HashMap::new(),
            universe_hp: HashMap::new(),
            _global_tx: global_tx.clone(),
            global_tx,
            global_rx: Some(global_rx),
            logger_tx,
            router_tx: None,
        }
    }

    pub fn start(&mut self) {
        let (router_tx, mut router_rx) = unbounded_channel::<(UniverseId, Sender<UniverseCommand>)>();
        self.router_tx = Some(router_tx);

        let mut global_rx = self.global_rx.take().unwrap();
        let logger_tx = self.logger_tx.clone();

        // We need a way to mutate the supervisor state from the async loop,
        // but we can't share `self`.
        // Instead, the router handles routing, and we handle structure updates via `remove_universe`.
        // But since the router runs in a spawn, it can't mutate `SupervisorHandle`.
        // SOLUTION: The Router detects Death, logs it, but we need a way to clean up `existing_universes`.
        // For this simple architecture, we will let the router run independently,
        // but we will create a separate command channel for internal cleanup if we wanted 100% purity.
        // HOWEVER, to fix the "fighting after death" bug, we just need to ensure the router stops routing to dead IDs.

        tokio::spawn(async move {
            let mut routes: HashMap<UniverseId, Sender<UniverseCommand>> = HashMap::new();

            loop {
                tokio::select! {
            Some((id, tx)) = router_rx.recv() => {
                routes.insert(id, tx);
            }
            Some(event) = global_rx.recv() => {
                match event {
                    UniverseOutboundEvent::Log { name, color, message } => {
                        let _ = logger_tx.send(LogMessage::UniverseLog { name, color, message });
                    }
                    UniverseOutboundEvent::BroadcastDeath(dead_id) => {
                        let _ = logger_tx.send(LogMessage::Info(format!("SYSTEM: Universe {} died.", dead_id)));
                        routes.remove(&dead_id);
                    }
                    UniverseOutboundEvent::BroadcastPeerDied(dead_id) => {
                        // Forward to all living universes
                        for (&id, tx) in routes.iter() {
                            if id != dead_id {
                                let _ = tx.send(UniverseCommand::InjectEvent(
                                    UniverseEvent::PeerDied
                                ));
                            }
                        }
                    }
                    UniverseOutboundEvent::MessagePeer { target_id, event } => {
                        if let Some(tx) = routes.get(&target_id) {
                            let _ = tx.send(UniverseCommand::InjectEvent(event));
                        }
                    }
                }
            }
            else => {
                // Both channels closed → supervisor is shutting down
                break;
            }
        }
            }
        });
    }

    pub async fn add_new_universe(&mut self, name: String) {
        if self.universes_via_name.contains_key(&name) { return; }

        let universe_handle = universe::create_universe_handle(
            name.clone(),
            self.global_tx.clone(),
            self.logger_tx.clone()
        );

        let id = universe_handle.handle_id;
        let cmd_tx = universe_handle.commander_tx.clone();

        // 1. Register with Router
        if let Some(router) = &self.router_tx {
            let _ = router.send((id, cmd_tx.clone()));
        }

        // 2. Introduce to existing peers
        for (existing_name, existing_id) in &self.universes_via_name {
            // Tell new about old
            let _ = cmd_tx.send(UniverseCommand::MeetPeer { id: *existing_id, name: existing_name.clone() }).await;

            // Tell old about new (We need to find the handle)
            if let Some(handle) = self.existing_universes.get(existing_id) {
                let _ = handle.commander_tx.send(UniverseCommand::MeetPeer { id, name: name.clone() }).await;
            }
        }

        // 3. Store
        self.universes_via_name.insert(name.clone(), id);
        self.existing_universes.insert(id, universe_handle);
        self.universe_hp.insert(id, 100);

        let _ = self.logger_tx.send(LogMessage::Info(format!("Created universe '{}' (ID: {})", name, id)));
    }

    pub fn exists(&self, name: &str) -> bool {
        self.universes_via_name.contains_key(name)
    }

    pub fn get_all_existing_universes(&self) -> Vec<String> {
        self.universes_via_name.keys().cloned().collect()
    }

    pub async fn send_universe_command(&mut self, name: String, command: UniverseCommand) {
        // Check for shutdown command to clean up local map
        let is_shutdown = matches!(command, UniverseCommand::Shutdown);

        if let Some(id) = self.universes_via_name.get(&name) {
            if let Some(u) = self.existing_universes.get(id) {
                let _ = u.commander_tx.send(command).await;
            }

            // If we manually shut it down, remove it from our lists
            if is_shutdown {
                let id_copy = *id;
                self.existing_universes.remove(&id_copy);
                self.universes_via_name.remove(&name);
            }
        }
    }
}