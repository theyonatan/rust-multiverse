use std::time::Duration;
use tokio::sync::mpsc::{Sender, unbounded_channel, channel, UnboundedReceiver};
use tokio::time::{interval, MissedTickBehavior};
use rgb::{Rgb, RGB8};
use rand::Rng;
use crate::universe::intent::UniverseIntent;
use crate::universe::id::UniverseId;
use crate::universe::relationship::Relationship;
use crate::universe::universe::Universe;
use crate::universe::universe_command::UniverseCommand;
use crate::universe::universe_event::UniverseEvent;

pub struct UniverseHandle {
    pub(crate) handle_id: UniverseId,
    pub(crate) own_name: String,
    pub(crate) color: RGB8,
    pub(crate) commander_tx: Sender<UniverseCommand>,
    pub(crate) universe_task_handle: tokio::task::JoinHandle<()>,
    pub(crate) intent_rx: UnboundedReceiver<UniverseIntent>,
}

impl UniverseHandle {
    fn new(mut universe: Universe, intent_rx: UnboundedReceiver<UniverseIntent>, own_name: String, color: Rgb<u8>) -> UniverseHandle {
        let handle_id = universe.id.clone();

        let (commander_tx, mut command_rx) = channel::<UniverseCommand>(10);

        let universe_task_handle = tokio::spawn(async move{
            let mut ticker = interval(Duration::from_millis(80)); // 0.4s per step
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    // commands from supervisor
                    Some(command) = command_rx.recv() => {
                        handle_given_command(&command, &mut universe);
                    }

                    // other -> nothing is pending, perform step
                    _ = ticker.tick() => {
                        // step, while waiting for rx
                    }

                }

                if universe.is_collapsed() {
                    return
                }

                (&mut universe).step();
            }
        });

        UniverseHandle {
            handle_id,
            own_name,
            color,
            commander_tx,
            universe_task_handle,
            intent_rx,
        }
    }
}

pub fn create_universe_handle(name: String) -> UniverseHandle {
    let (intent_tx, intent_rx) = unbounded_channel::<UniverseIntent>();

    let color = RGB8::new(
        rand::rng().random_range(50..255),
        rand::rng().random_range(50..255),
        rand::rng().random_range(50..255),
    );

    let universe = Universe::new(intent_tx);

    UniverseHandle::new(universe, intent_rx, name, color)
}

fn handle_given_command(command: &UniverseCommand, universe: &mut Universe) {
    match command {
        UniverseCommand::Start => {
            universe.executes = true;
        }
        UniverseCommand::Stop => {
            universe.executes = false;
        }
        UniverseCommand::InjectEvent(event) => {
            handle_given_event(event, universe);
        }
        UniverseCommand::SetRelationship(id, relationship) => {
            match relationship {
                Relationship::Enemy => { universe.enemies.insert(id.clone()); }
                Relationship::Brother => { universe.brothers.insert(id.clone()); }
            }
        }
        UniverseCommand::Shutdown => {
            universe.executes = false;
            universe.shutdown();
            
            return;
        }
    }
}

fn handle_given_event(event: &UniverseEvent, universe: &mut Universe) {
    match event {
        UniverseEvent::Shatter(strength) => {
            universe.take_damage(*strength);
        }
        UniverseEvent::Crash => {
            universe.take_damage(999);  // insta-kill
        }
        UniverseEvent::Heal(strength) => {
            universe.heal(*strength);
        }
        UniverseEvent::UniverseCollapsed(collapsed_id) => {
            if universe.is_brother(*collapsed_id) {
                universe.brothers.remove(&collapsed_id);
            }
            if universe.is_enemy(*collapsed_id) {
                universe.enemies.remove(&collapsed_id);
            }
        }
    }
}