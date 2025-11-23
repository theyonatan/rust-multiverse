use tokio::sync::mpsc::{Sender, UnboundedSender};
use tokio::time::{interval, Duration};

use crate::universe::id::UniverseId;
use crate::universe::universe::Universe;
use crate::universe::universe_command::UniverseCommand;
use crate::universe::universe_event::{UniverseEvent, UniverseOutboundEvent};
use crate::universe::logger::LogMessage;

pub struct UniverseHandle {
    pub(crate) handle_id: UniverseId,
    pub(crate) commander_tx: Sender<UniverseCommand>,
}

impl UniverseHandle {
    pub fn new(
        mut universe: Universe,
        global_bus: UnboundedSender<UniverseOutboundEvent>,
        logger: UnboundedSender<LogMessage>
    ) -> UniverseHandle {
        let handle_id = universe.id.clone();
        let name = universe.name.clone();
        let color = universe.color.clone();

        let (commander_tx, mut command_rx) = tokio::sync::mpsc::channel::<UniverseCommand>(20);

        universe.set_outbound_tx(global_bus.clone());

        let _ = tokio::spawn(async move{
            // Initial log
            let _ = logger.send(LogMessage::UniverseLog {
                name: name.clone(),
                color,
                message: "Spawned.".to_string()
            });

            let mut ticker = interval(Duration::from_millis(500));

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                         universe.step();
                    }
                    Some(command) = command_rx.recv() => {
                        match command {
                            UniverseCommand::Shutdown => {
                                let _ = logger.send(LogMessage::UniverseLog {
                                    name: name.clone(),
                                    color,
                                    message: "Shutting down.".to_string()
                                });
                                return;
                            }
                            UniverseCommand::InjectEvent(event) => {
                                match event {
                                    UniverseEvent::Shatter(damage) => universe.handle_shatter(damage),
                                    UniverseEvent::Heal(amount) => universe.handle_heal(amount),
                                    UniverseEvent::Ping | UniverseEvent::Pong => {
                                        universe.log_internal(format!("Received {:?}", event));
                                    }
                                    UniverseEvent::Crash(id) => {
                                        universe.log_internal(format!("CRASH signal from {}", id));
                                        universe.hp = 0;
                                        universe.die();
                                    }
                                    _ => {
                                        universe.log_internal(format!("Ignored event: {:?}", event));
                                    }
                                }
                            }
                            UniverseCommand::MeetPeer { id, name } => universe.meet_peer(id, &name),
                            UniverseCommand::Start => {},
                            UniverseCommand::Stop => {},
                            UniverseCommand::RequestState() => {
                                universe.report_state();
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        UniverseHandle {
            handle_id,
            commander_tx,
        }
    }
}

pub fn create_universe_handle(
    name: String,
    global_bus: UnboundedSender<UniverseOutboundEvent>,
    logger: UnboundedSender<LogMessage>
) -> UniverseHandle {
    let universe = Universe::new(name);
    UniverseHandle::new(universe, global_bus, logger)
}